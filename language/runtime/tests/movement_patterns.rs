// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Verifies direct movement transformations before applying numerical interpretations.

use native_space_language::core::{self, NativeScalar, NativeState, Program};
use native_space_language::{bytecode, retained};
use num_traits::{ToPrimitive as _, Zero as _};

const SOURCE: &str = include_str!("../../../examples/movement-patterns.ns");

fn with_body(body: &str) -> Program {
    let (definitions, _) = SOURCE.rsplit_once("\noutput ").unwrap();
    core::parse(&format!("{definitions}\n{body}\n"), "movement-patterns.ns").unwrap()
}

fn program() -> Program {
    core::parse(SOURCE, "movement-patterns.ns").unwrap()
}

fn scalar(value: &str) -> NativeState {
    NativeState::scalar(NativeScalar::from_text(value, "0").unwrap())
}

fn call(program: &Program, name: &str, values: &[NativeState]) -> NativeState {
    core::exact_function(program, name)
        .unwrap()
        .apply(values)
        .unwrap()
}

fn movement(program: &Program, depth: &str, turns: &str) -> NativeState {
    call(program, "movement", &[scalar(depth), scalar(turns)])
}

// An independent exact endpoint camera for the tested dyadic, quarter-turn
// subset. Unsupported irrational endpoints return None, never rounded values.
// This is only a test readout, not part of the source movement transformation.
fn dyadic_endpoint(value: &NativeState) -> Option<NativeState> {
    let read = |direction| {
        let field = value.camera(direction, 0);
        if field.is_zero() {
            return Some(core::rational("0").unwrap());
        }
        if field.0.len() != 1 {
            return None;
        }
        let (index, coefficient) = field.0.first_key_value()?;
        (index.0.is_empty() && coefficient.imag.is_zero()).then(|| coefficient.real.clone())
    };
    if read(3)? != core::rational("2").unwrap() {
        return None;
    }
    let depth = read(1)?;
    let quarters = read(2)? * core::rational("4").unwrap();
    if !depth.is_integer() || !quarters.is_integer() {
        return None;
    }
    let depth = depth.to_i32()?;
    // Bound fixture decoding so an accidental test input cannot allocate an
    // enormous endpoint; movement arithmetic itself has no such depth bound.
    if !(-32..=32).contains(&depth) {
        return None;
    }
    let quarters = quarters.to_i64()?;
    let step = scalar(if depth < 0 { "1/2" } else { "2" });
    let magnitude = (0..depth.unsigned_abs()).fold(NativeState::one(), |p, _| p.multiply(&step));
    Some(magnitude.phase(quarters.rem_euclid(4)))
}

#[test]
fn defining_laws_cancel_symbolically_not_only_on_sampled_coordinates() {
    // Fresh INDEX directions serve as polynomial indeterminates. The source
    // operations are linear in these coordinates, so the exact zero residual
    // remains zero under every rational substitution. This uses the existing
    // arithmetic checker, not a new theorem-specific rule or a floating tolerance.
    let variables = "let p = movement(index(100,1),index(101,1))\n\
                     let q = movement(index(102,1),index(103,1))\n";
    for (left, right) in [
        ("compose(sqrt(p),sqrt(p))", "p"),
        ("sqrt(square(p))", "p"),
        ("compose(p,reverse(p))", "identity()"),
        ("compose(quotient(p,q),q)", "p"),
        ("compose(p,between(p,q))", "q"),
        ("interpolate(p,q,0)", "p"),
        ("interpolate(p,q,1)", "q"),
        (
            "portion(compose(p,q),1/3)",
            "compose(portion(p,1/3),portion(q,1/3))",
        ),
        ("portion(p,0)", "identity()"),
        ("cube(p)", "chain(p,p,p)"),
        ("power(p,3/7)", "portion(p,3/7)"),
        ("chain()", "identity()"),
    ] {
        let proof = with_body(&format!("{variables}difference({left},{right}) = 0"));
        assert!(
            core::interpret(&proof).unwrap().is_zero(),
            "{left} = {right}"
        );
        assert!(
            bytecode::execute(&bytecode::compile(&proof).unwrap())
                .unwrap()
                .is_zero()
        );
    }
}

#[test]
fn roots_are_exact_fractional_movements_inward_outward_and_around() {
    let program = program();
    for (depth, turns, half_depth, half_turns) in [
        ("2", "0", "1", "0"),
        ("-2", "0", "-1", "0"),
        ("1", "0", "1/2", "0"),
        ("0", "1/2", "0", "1/4"),
        ("7/3", "-9/5", "7/6", "-9/10"),
        ("0", "5/4", "0", "5/8"),
    ] {
        let original = movement(&program, depth, turns);
        let root = call(&program, "sqrt", std::slice::from_ref(&original));
        assert_eq!(root, movement(&program, half_depth, half_turns));
        assert_eq!(call(&program, "compose", &[root.clone(), root]), original);
    }
    for (depth, turns, expected) in [
        ("2", "0", scalar("2")),
        ("-2", "0", scalar("1/2")),
        ("0", "1/2", NativeState::one().phase(1)),
    ] {
        let input = movement(&program, depth, turns);
        let root = call(&program, "sqrt", std::slice::from_ref(&input));
        let endpoint = dyadic_endpoint(&root).unwrap();
        assert_eq!(endpoint, expected);
        assert_eq!(
            endpoint.multiply(&endpoint),
            dyadic_endpoint(&input).unwrap()
        );
    }
    let irrational_root = call(&program, "sqrt", &[movement(&program, "1", "0")]);
    assert!(dyadic_endpoint(&irrational_root).is_none());
}

#[test]
fn winding_and_the_alternative_root_are_not_silently_normalized() {
    let program = program();
    let empty = call(&program, "identity", &[]);
    let full = call(&program, "turn", &[scalar("1")]);
    assert_ne!(full, empty);
    assert_eq!(dyadic_endpoint(&full), dyadic_endpoint(&empty));
    assert_eq!(
        call(&program, "sqrt", &[full]),
        movement(&program, "0", "1/2")
    );
    assert_eq!(call(&program, "sqrt", &[empty.clone()]), empty);
    let p = movement(&program, "3/7", "5/4");
    let other = call(&program, "other_sqrt", std::slice::from_ref(&p));
    let squared = call(&program, "square", &[other]);
    assert_ne!(squared, p);
    assert_eq!(squared, movement(&program, "3/7", "9/4"));
    assert_eq!(
        call(&program, "between", &[p, squared]),
        movement(&program, "0", "1")
    );
    let p = movement(&program, "2", "1/2");
    let alternate = call(&program, "other_sqrt", std::slice::from_ref(&p));
    let endpoint = dyadic_endpoint(&alternate).unwrap();
    assert_eq!(endpoint.multiply(&endpoint), dyadic_endpoint(&p).unwrap());
}

#[test]
fn log_and_exp_names_only_encode_and_read_the_supplied_base_two_depth() {
    let program = program();
    for n in -10..=10 {
        let coordinate = scalar(&format!("{n}/3"));
        let grown = call(&program, "exp2", std::slice::from_ref(&coordinate));
        assert_eq!(grown.camera(3, 0), scalar("2"));
        assert!(grown.camera(2, 0).is_zero());
        assert_eq!(call(&program, "log2", &[grown]), coordinate);
    }
}

#[test]
fn subdivision_keeps_both_children_and_preserves_start_join_and_finish() {
    let proof = with_body(
        "\
        let s = segment(movement(index(100,1),index(101,1)),movement(index(102,1),index(103,1)))
        let children = bisect(s)
        add(index(30,difference(start_of(first_half(children)),start_of(s))),
            index(31,difference(finish_of(first_half(children)),start_of(second_half(children)))),
            index(32,difference(finish_of(second_half(children)),finish_of(s)))) = 0",
    );
    assert!(core::interpret(&proof).unwrap().is_zero());
    assert!(
        bytecode::execute(&bytecode::compile(&proof).unwrap())
            .unwrap()
            .is_zero()
    );
    let program = program();
    let mut segment = call(
        &program,
        "segment",
        &[movement(&program, "0", "0"), movement(&program, "0", "1/4")],
    );
    for denominator in [8, 16, 32, 64, 128, 256] {
        segment = call(
            &program,
            "first_half",
            &[call(&program, "bisect", &[segment])],
        );
        assert_eq!(
            call(&program, "advance_of", &[segment.clone()]),
            movement(&program, "0", &format!("1/{denominator}"))
        );
    }
    let four = call(&program, "four_way", &[]);
    let expected = (0..4).fold(NativeState::zero(), |total, n| {
        let side = call(
            &program,
            "segment",
            &[
                movement(&program, "0", &format!("{n}/4")),
                movement(&program, "0", "1/4"),
            ],
        );
        total.add(
            &call(&program, "bisect", &[side])
                .index_power(9, n + 1)
                .unwrap(),
        )
    });
    assert_eq!(four, expected);
}

#[test]
fn executable_graphs_and_serialization_preserve_the_full_source_history() {
    for expression in [
        "sqrt(grow(2))",
        "sqrt(movement(7/3,5/4))",
        "four_way()",
        "trace(sqrt)",
    ] {
        let program = with_body(&format!("output {expression} as pattern"));
        let native = retained::interpret(&program).unwrap();
        let compiled = bytecode::compile(&program).unwrap();
        let replay = bytecode::execute_retained(&compiled).unwrap();
        assert!(native.same_structure(&replay));
        assert_eq!(replay.project(), &core::interpret(&program).unwrap());
        assert!(
            retained::State::from_data(&replay.native_data())
                .unwrap()
                .same_structure(&replay)
        );
    }
    let program = program();
    let original = retained::State::from_projection(&movement(&program, "2", "5/4"));
    let root = core::exact_function(&program, "sqrt")
        .unwrap()
        .apply_retained(std::slice::from_ref(&original))
        .unwrap();
    let recovered = core::exact_function(&program, "compose")
        .unwrap()
        .apply_retained(&[root.clone(), root])
        .unwrap();
    assert!(recovered.same_projection(&original));
    assert!(!recovered.same_structure(&original));
}
