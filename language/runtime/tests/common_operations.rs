// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Checks the source examples against independent arithmetic and numerical references.

use native_space_language::core::{self, NativeScalar, NativeState, Program};
use native_space_language::{bytecode, retained};
use num_traits::ToPrimitive as _;

fn program() -> Program {
    core::parse(
        include_str!("../../../examples/common-operations.ns"),
        "common-operations.ns",
    )
    .unwrap()
}

fn scalar(value: &str) -> NativeState {
    NativeState::scalar(NativeScalar::from_text(value, "0").unwrap())
}

fn call(program: &Program, name: &str, inputs: &[NativeState]) -> NativeState {
    core::exact_function(program, name)
        .unwrap()
        .apply(inputs)
        .unwrap()
}

fn word(program: &Program, value: u8) -> NativeState {
    let bits = [0, 1, 2, 3].map(|shift| scalar(&((value >> shift) & 1).to_string()));
    call(program, "word4", &bits)
}

fn number(value: &NativeState) -> f64 {
    if value.is_zero() {
        return 0.0;
    }
    assert_eq!(value.0.len(), 1);
    let (index, coefficient) = value.0.first_key_value().unwrap();
    assert!(index.0.is_empty());
    assert_eq!(coefficient.imag, core::rational("0").unwrap());
    coefficient.real.to_f64().unwrap()
}

#[test]
fn common_demo_agrees_in_source_native_graph_and_compiled_vm() {
    let program = program();
    let reference = core::interpret(&program).unwrap();
    let native = retained::interpret(&program).unwrap();
    let artifact = bytecode::compile(&program).unwrap();
    let replay = bytecode::execute_retained(&artifact).unwrap();
    assert!(replay.same_structure(&native));
    assert_eq!(replay.project(), &reference);
    assert!(
        retained::State::from_data(&replay.native_data())
            .unwrap()
            .same_structure(&replay)
    );
    assert_eq!(reference.camera(101, 0), scalar("5"));
    assert_eq!(reference.camera(102, 0), scalar("6"));
    assert_eq!(reference.camera(103, 0), scalar("-1"));
    assert_eq!(reference.camera(118, 0), scalar("1"));
    assert_eq!(reference.camera(119, 0), scalar("4"));
    let mut traced = program.clone();
    traced.result = core::Expr::Trace {
        function: "pi".into(),
        span: None,
    };
    let trace = core::interpret(&traced).unwrap();
    assert!(!trace.is_zero());
    assert_eq!(
        bytecode::execute(&bytecode::compile(&traced).unwrap()).unwrap(),
        trace
    );
    for (tag, label) in [
        (104, "7/3"),
        (105, "pi"),
        (106, "sqrt(5/4)"),
        (107, "exp(1)"),
        (108, "log(5/4)"),
        (109, "sin(1/2)"),
        (110, "cos(1/2)"),
        (111, "tan(1/2)"),
    ] {
        println!("{label}: {:.12}", number(&reference.camera(tag, 0)));
    }
}

#[test]
fn boolean_arithmetic_matches_every_truth_table_and_can_select_indexed_states() {
    let program = program();
    for a in 0..=1 {
        for b in 0..=1 {
            let inputs = [scalar(&a.to_string()), scalar(&b.to_string())];
            for (name, expected) in [
                ("bool_and", a & b),
                ("bool_or", a | b),
                ("bool_xor", a ^ b),
                ("bool_equal", i32::from(a == b)),
            ] {
                assert_eq!(
                    call(&program, name, &inputs),
                    scalar(&expected.to_string()),
                    "{name}({a},{b})"
                );
            }
        }
        assert_eq!(
            call(&program, "bool_not", &[scalar(&a.to_string())]),
            scalar(&(1 - a).to_string())
        );
        let left = scalar("7").index_power(41, 3).unwrap();
        let right = scalar("100").index_power(42, 1).unwrap();
        let selected = call(
            &program,
            "select",
            &[scalar(&a.to_string()), left.clone(), right.clone()],
        );
        assert_eq!(selected, if a == 1 { left } else { right });
    }
}

#[test]
fn basic_arithmetic_and_variadic_polynomials_are_exact() {
    let program = program();
    assert_eq!(call(&program, "negate", &[scalar("-7/3")]), scalar("7/3"));
    assert_eq!(
        call(&program, "subtract", &[scalar("7/3"), scalar("-1/6")]),
        scalar("5/2")
    );
    assert_eq!(call(&program, "square", &[scalar("-3/2")]), scalar("9/4"));
    assert_eq!(call(&program, "cube", &[scalar("-3/2")]), scalar("-27/8"));
    assert_eq!(
        call(
            &program,
            "polynomial",
            &[scalar("2"), scalar("3"), scalar("2"), scalar("1")]
        ),
        scalar("17")
    );
    assert_eq!(call(&program, "polynomial", &[scalar("7")]), scalar("0"));
    assert_eq!(
        call(&program, "polynomial", &[scalar("7"), scalar("-1/3")]),
        scalar("-1/3")
    );
}

#[test]
fn unsigned_comparisons_and_divmod_are_exhaustive_over_their_declared_width() {
    let program = program();
    let words = (0_u8..16)
        .map(|value| word(&program, value))
        .collect::<Vec<_>>();
    let divide = core::exact_function(&program, "divmod").unwrap();
    let unsigned = core::exact_function(&program, "unsigned").unwrap();
    let comparisons = ["lt", "gt", "le", "ge", "eq", "ne"]
        .map(|name| core::exact_function(&program, name).unwrap());
    for a in 0_u8..16 {
        for b in 0_u8..16 {
            let inputs = [words[usize::from(a)].clone(), words[usize::from(b)].clone()];
            let expected = [a < b, a > b, a <= b, a >= b, a == b, a != b];
            for (function, expected) in comparisons.iter().zip(expected) {
                assert_eq!(
                    function.apply(&inputs).unwrap(),
                    scalar(if expected { "1" } else { "0" }),
                    "a={a}, b={b}"
                );
            }
            assert_eq!(call(&program, "min", &inputs), words[usize::from(a.min(b))]);
            assert_eq!(call(&program, "max", &inputs), words[usize::from(a.max(b))]);
            let result = divide.apply(&inputs).unwrap();
            assert_eq!(result.camera(13, 0), scalar(if b == 0 { "0" } else { "1" }));
            if b != 0 {
                let remainder = unsigned.apply(&[result.camera(10, 0)]).unwrap();
                let quotient = unsigned.apply(&[result.camera(11, 0)]).unwrap();
                assert_eq!(
                    remainder,
                    scalar(&(a % b).to_string()),
                    "remainder({a},{b})"
                );
                assert_eq!(quotient, scalar(&(a / b).to_string()), "quotient({a},{b})");
                assert_eq!(
                    quotient.multiply(&scalar(&b.to_string())).add(&remainder),
                    scalar(&a.to_string())
                );
            }
        }
    }
}

#[test]
fn fourier_sign_normalization_and_inverse_are_exact_for_complex_inputs() {
    let program = program();
    let inputs = [scalar("1"), scalar("2"), scalar("3"), scalar("4")];
    let spectrum = call(&program, "fourier4", &inputs);
    let expected = [
        NativeScalar::from_text("10", "0").unwrap(),
        NativeScalar::from_text("-2", "2").unwrap(),
        NativeScalar::from_text("-2", "0").unwrap(),
        NativeScalar::from_text("-2", "-2").unwrap(),
    ];
    for (bin, expected) in (1..=4).zip(expected) {
        assert_eq!(spectrum.camera(bin, 0), NativeState::scalar(expected));
    }
    // A basis checks every forward sign, not only the one demonstration vector.
    for source in 0..4 {
        let mut basis = [
            NativeState::zero(),
            NativeState::zero(),
            NativeState::zero(),
            NativeState::zero(),
        ];
        basis[source] = NativeState::scalar(NativeScalar::from_text("2/3", "-5/7").unwrap());
        let transformed = call(&program, "fourier4", &basis);
        for bin in 0..4 {
            let turns = i64::try_from((4 - (bin * source) % 4) % 4).unwrap();
            assert_eq!(
                transformed.camera(u64::try_from(bin + 1).unwrap(), 0),
                basis[source].phase(turns)
            );
        }
        let restored = call(&program, "inverse_fourier4", &[transformed]);
        assert_eq!(restored, call(&program, "tuple4", &basis));
    }
}

#[test]
fn finite_approximations_stay_within_documented_bounds_on_a_grid_and_endpoints() {
    let program = program();
    for (name, reference, bound, center, radius) in [
        ("exp", f64::exp as fn(f64) -> f64, 5e-10, 0, 20),
        ("sin", f64::sin as fn(f64) -> f64, 8e-13, 0, 20),
        ("cos", f64::cos as fn(f64) -> f64, 1.2e-11, 0, 20),
        ("tan", f64::tan as fn(f64) -> f64, 5e-11, 0, 20),
        ("atan", f64::atan as fn(f64) -> f64, 8e-14, 0, 4),
        ("log", f64::ln as fn(f64) -> f64, 5e-8, 20, 10),
        ("sqrt", f64::sqrt as fn(f64) -> f64, 7e-8, 20, 10),
    ] {
        let function = core::exact_function(&program, name).unwrap();
        let mut maximum = 0.0_f64;
        for numerator in (center - radius)..=(center + radius) {
            let actual = number(
                &function
                    .apply(&[scalar(&format!("{numerator}/20"))])
                    .unwrap(),
            );
            let error = (actual - reference(f64::from(numerator) / 20.0)).abs();
            maximum = maximum.max(error);
            assert!(
                error <= bound,
                "{name}({numerator}/20): error {error} exceeds {bound}"
            );
        }
        println!("{name}: largest sampled error {maximum:.3e}");
    }
    assert!((number(&call(&program, "pi", &[])) - std::f64::consts::PI).abs() < 1.3e-12);
    // Exact rational checks avoid a float comparison claiming 19-digit accuracy.
    for numerator in 10..=30 {
        let y = scalar(&format!("{numerator}/20"));
        let result = call(&program, "reciprocal", std::slice::from_ref(&y));
        let residual = NativeState::one().add(&y.multiply(&result).phase(2));
        let expected = NativeState::one().add(&y.phase(2));
        let expected = (0..6).fold(expected, |error, _| error.multiply(&error));
        assert_eq!(residual, expected);
    }
    assert!(
        (number(&call(
            &program,
            "divide_scaled",
            &[scalar("7"), scalar("3"), scalar("1/4")]
        )) - 7.0 / 3.0)
            .abs()
            < 1e-14
    );
}
