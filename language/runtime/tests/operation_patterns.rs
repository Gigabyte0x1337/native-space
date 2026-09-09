// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Checks native charts independently of their classical readouts.

use native_space_language::core::{self, NativeScalar, NativeState, Program};
use native_space_language::{bytecode, retained};
use num_rational::BigRational;
use num_traits::{ToPrimitive as _, Zero as _};

const SOURCE: &str = include_str!("../../../examples/operation-patterns.ns");

fn program() -> Program {
    core::parse(SOURCE, "operation-patterns.ns").unwrap()
}

fn baseline() -> Program {
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

fn coefficient(value: &NativeState) -> BigRational {
    if value.is_zero() {
        return core::rational("0").unwrap();
    }
    assert_eq!(value.0.len(), 1, "readout expects one scalar");
    let (index, value) = value.0.first_key_value().unwrap();
    assert!(index.0.is_empty());
    assert!(value.imag.is_zero());
    value.real.clone()
}

// This decoder is explicitly outside the NS calculation. It does not discover
// an algorithm or replace a native step; it checks the stated n:d interpretation.
// Refusing a zero denominator prevents a cross-product equality from admitting 0:0.
fn read_ratio(pattern: &NativeState) -> Option<BigRational> {
    let denominator = coefficient(&pattern.camera(2, 0));
    if denominator.is_zero() {
        None
    } else {
        Some(coefficient(&pattern.camera(1, 0)) / denominator)
    }
}

fn ratio(program: &Program, numerator: &str, denominator: &str) -> NativeState {
    call(program, "ratio", &[scalar(numerator), scalar(denominator)])
}

fn word(program: &Program, value: u8) -> NativeState {
    let phases =
        [0, 1, 2, 3].map(|shift| scalar(if (value >> shift) & 1 == 0 { "1" } else { "-1" }));
    call(program, "word4", &phases)
}

#[test]
fn ratio_arithmetic_is_exact_without_evaluating_reciprocals() {
    let program = program();
    let values = [("0", "7"), ("7", "3"), ("-11", "5"), ("2", "-9")];
    for (a, b) in values {
        let p = ratio(&program, a, b);
        let x = read_ratio(&p).unwrap();
        assert_eq!(
            read_ratio(&call(&program, "square", std::slice::from_ref(&p))).unwrap(),
            &x * &x
        );
        assert_eq!(
            read_ratio(&call(&program, "cube", std::slice::from_ref(&p))).unwrap(),
            &x * &x * &x
        );
        for (c, d) in values {
            let q = ratio(&program, c, d);
            let y = read_ratio(&q).unwrap();
            for (name, expected) in [("plus", &x + &y), ("subtract", &x - &y), ("times", &x * &y)] {
                assert_eq!(
                    read_ratio(&call(&program, name, &[p.clone(), q.clone()])).unwrap(),
                    expected
                );
            }
            let division = read_ratio(&call(&program, "divide", &[p.clone(), q]));
            assert_eq!(division, (!y.is_zero()).then(|| &x / &y));
        }
        let reframed = call(&program, "reframe", &[p.clone(), scalar("-5/7")]);
        assert_eq!(read_ratio(&reframed).unwrap(), x);
        assert!(call(&program, "ratio_residual", &[p, reframed]).is_zero());
    }
    let seven = call(&program, "lift", &[scalar("7")]);
    let zero = call(&program, "lift", &[scalar("0")]);
    let undefined = call(&program, "divide", &[seven.clone(), zero]);
    assert_eq!(undefined.camera(1, 0), scalar("7"));
    assert!(read_ratio(&undefined).is_none());
    let empty = ratio(&program, "0", "0");
    assert!(read_ratio(&empty).is_none());
    assert!(call(&program, "ratio_residual", &[empty, seven]).is_zero());
}

#[test]
fn supplied_depth_moves_preserve_their_stated_positive_magnitude_chart() {
    let program = program();
    for u in -4..=4 {
        let p = call(&program, "depth", &[scalar(&u.to_string())]);
        assert_eq!(p.camera(20, 0), scalar("2"));
        assert_eq!(
            call(&program, "depth_log2", std::slice::from_ref(&p)),
            scalar(&u.to_string())
        );
        for (name, expected) in [
            ("depth_square", format!("{}", 2 * u)),
            ("depth_sqrt", format!("{u}/2")),
            ("depth_reciprocal", format!("{}", -u)),
            ("depth_double", format!("{}", u + 1)),
        ] {
            let moved = call(&program, name, std::slice::from_ref(&p));
            assert_eq!(moved.camera(21, 0), scalar(&expected));
        }
        for v in -4..=4 {
            let q = call(&program, "depth", &[scalar(&v.to_string())]);
            for (name, expected) in [("depth_multiply", u + v), ("depth_divide", u - v)] {
                let moved = call(&program, name, &[p.clone(), q.clone()]);
                assert_eq!(moved.camera(21, 0), scalar(&expected.to_string()));
            }
        }
    }
    // Halfway to depth one is retained exactly, even though sqrt(2) cannot be
    // an exact rational scalar. No fake rational square-root readout is supplied.
    let root = call(
        &program,
        "depth_sqrt",
        &[call(&program, "depth", &[scalar("1")])],
    );
    assert_eq!(root.camera(21, 0), scalar("1/2"));
}

#[test]
fn rational_directions_compose_exactly_and_need_no_angle_conversion() {
    let program = program();
    let identity = call(&program, "rotation", &[scalar("0")]);
    let movement = call(&program, "rotation", &[scalar("1/2")]);
    for (name, expected) in [("cos", "3/5"), ("sin", "4/5"), ("tan", "4/3")] {
        assert_eq!(
            read_ratio(&call(&program, name, std::slice::from_ref(&movement))).unwrap(),
            core::rational(expected).unwrap()
        );
    }
    for n in -10..=10 {
        let rotation = call(&program, "rotation", &[scalar(&format!("{n}/5"))]);
        let x = coefficient(&rotation.camera(1, 0));
        let y = coefficient(&rotation.camera(2, 0));
        let w = coefficient(&rotation.camera(3, 0));
        assert_eq!(&x * &x + &y * &y, &w * &w);
        let moved = call(&program, "turn", &[rotation.clone(), movement.clone()]);
        let cosine = read_ratio(&call(&program, "cos", std::slice::from_ref(&moved))).unwrap();
        let sine = read_ratio(&call(&program, "sin", &[moved])).unwrap();
        assert_eq!(
            cosine,
            (&x * core::rational("3/5").unwrap() - &y * core::rational("4/5").unwrap()) / &w
        );
        assert_eq!(
            sine,
            (&x * core::rational("4/5").unwrap() + &y * core::rational("3/5").unwrap()) / &w
        );
        let mut four = rotation.clone();
        for _ in 0..4 {
            four = call(&program, "quarter", &[four]);
        }
        assert_eq!(four, rotation);
    }
    let quarter = call(&program, "rotation", &[scalar("1")]);
    assert!(read_ratio(&call(&program, "tan", std::slice::from_ref(&quarter))).is_none());
    let twice = call(&program, "turn", &[quarter.clone(), quarter]);
    let half = call(&program, "half_turn", &[]);
    assert_eq!(
        read_ratio(&call(&program, "cos", &[twice])).unwrap(),
        core::rational("-1").unwrap()
    );
    assert_eq!(
        read_ratio(&call(&program, "cos", &[half])).unwrap(),
        core::rational("-1").unwrap()
    );
    assert_eq!(
        call(&program, "walk", &[identity.clone(), movement.clone()]),
        identity
    );
    let walk = call(
        &program,
        "walk",
        &[identity, movement.clone(), scalar("0"), scalar("0")],
    );
    assert_eq!(walk, call(&program, "turn", &[movement.clone(), movement]));
}

#[test]
fn phase_logic_matches_all_boolean_inputs_and_selects_complete_values() {
    let program = program();
    for a in 0..=1 {
        let p = call(&program, "bit", &[scalar(&a.to_string())]);
        assert_eq!(
            call(&program, "bit_value", std::slice::from_ref(&p)),
            scalar(&a.to_string())
        );
        for b in 0..=1 {
            let q = call(&program, "bit", &[scalar(&b.to_string())]);
            for (name, expected) in [
                ("bool_and", a & b),
                ("bool_or", a | b),
                ("bool_xor", a ^ b),
                ("bool_equal", i32::from(a == b)),
            ] {
                let phase = call(&program, name, &[p.clone(), q.clone()]);
                assert_eq!(
                    call(&program, "bit_value", &[phase]),
                    scalar(&expected.to_string())
                );
            }
        }
        let yes = scalar("7").index_power(41, 2).unwrap();
        let no = scalar("100").index_power(42, 3).unwrap();
        assert_eq!(
            call(&program, "select", &[p, yes.clone(), no.clone()]),
            if a == 1 { yes } else { no }
        );
    }
}

#[test]
fn phase_word_order_and_remainder_match_every_four_bit_case() {
    let program = program();
    let words = (0_u8..16).map(|x| word(&program, x)).collect::<Vec<_>>();
    let comparisons = ["lt", "gt", "le", "ge", "eq", "ne"]
        .map(|name| core::exact_function(&program, name).unwrap());
    let division = core::exact_function(&program, "divmod").unwrap();
    for a in 0_u8..16 {
        for b in 0_u8..16 {
            let inputs = [words[usize::from(a)].clone(), words[usize::from(b)].clone()];
            for (function, expected) in
                comparisons
                    .iter()
                    .zip([a < b, a > b, a <= b, a >= b, a == b, a != b])
            {
                assert_eq!(
                    function.apply(&inputs).unwrap(),
                    scalar(if expected { "-1" } else { "1" }),
                    "a={a}, b={b}"
                );
            }
            assert_eq!(call(&program, "min", &inputs), words[usize::from(a.min(b))]);
            assert_eq!(call(&program, "max", &inputs), words[usize::from(a.max(b))]);
            let result = division.apply(&inputs).unwrap();
            assert_eq!(
                result.camera(13, 0),
                scalar(if b == 0 { "1" } else { "-1" })
            );
            if b != 0 {
                assert_eq!(
                    result.camera(10, 0),
                    words[usize::from(a % b)],
                    "remainder({a},{b})"
                );
                assert_eq!(
                    result.camera(11, 0),
                    words[usize::from(a / b)],
                    "quotient({a},{b})"
                );
            }
        }
        assert_eq!(
            call(&program, "unsigned", &[words[usize::from(a)].clone()]),
            scalar(&a.to_string())
        );
    }
    let inputs = [word(&program, 13), word(&program, 3)];
    assert_eq!(call(&program, "mod", &inputs), word(&program, 1));
    assert_eq!(call(&program, "int_divide", &inputs), word(&program, 4));
}

#[test]
fn root_reflection_squares_the_exact_residual_and_selects_the_positive_branch() {
    let program = program();
    let tolerance = core::rational("1/10000000000000000000000000000000000000000000").unwrap();
    for n in 10..=30 {
        let input = scalar(&format!("{n}/20"));
        let x = coefficient(&input);
        let moved = call(
            &program,
            "root_move",
            &[input.clone(), scalar("7/3"), scalar("5/4")],
        );
        let a = coefficient(&moved.camera(1, 0));
        let b = coefficient(&moved.camera(2, 0));
        let old_a = core::rational("7/3").unwrap();
        let old_b = core::rational("5/4").unwrap();
        let old_residual = &old_a * &old_a - &x * &old_b * &old_b;
        assert_eq!(&a * &a - &x * &b * &b, &old_residual * &old_residual);
        let root = read_ratio(&call(&program, "sqrt", &[input])).unwrap();
        let residual = &root * &root - &x;
        assert!(root > core::rational("0").unwrap());
        assert!(residual >= core::rational("0").unwrap() && residual < tolerance);
    }
}

#[test]
fn odd_walk_generates_its_own_coefficients_and_accepts_any_finite_tick_pack() {
    let program = program();
    let z = ratio(&program, "2", "7");
    let value = read_ratio(&z).unwrap();
    for sign in [-1, 1] {
        let mut inputs = vec![z.clone(), scalar(&sign.to_string())];
        let mut total = core::rational("0").unwrap();
        let mut term = value.clone();
        let step = core::rational(&sign.to_string()).unwrap() * &value * &value;
        for count in 0..=6 {
            assert_eq!(
                read_ratio(&call(&program, "odd_terms", &inputs)).unwrap(),
                total
            );
            total += &term / core::rational(&(2 * count + 1).to_string()).unwrap();
            term *= &step;
            inputs.push(scalar("0"));
        }
    }
    assert_eq!(
        read_ratio(&call(&program, "sqrt_steps", &[scalar("5/4")])).unwrap(),
        core::rational("1").unwrap()
    );
}

#[test]
fn shrinking_patterns_match_the_reference_and_independent_numeric_readouts() {
    let program = program();
    let baseline = baseline();
    for (name, reference, bound, old_bound, center, radius) in [
        ("log", f64::ln as fn(f64) -> f64, 1.1e-13, 5e-8, 20, 10),
        ("atan", f64::atan as fn(f64) -> f64, 1e-16, 8e-14, 0, 4),
        ("sqrt", f64::sqrt as fn(f64) -> f64, 5e-16, 7e-8, 20, 10),
    ] {
        let mut maximum = 0.0_f64;
        for n in (center - radius)..=(center + radius) {
            let input = scalar(&format!("{n}/20"));
            let actual = read_ratio(&call(&program, name, std::slice::from_ref(&input)))
                .unwrap()
                .to_f64()
                .unwrap();
            let old = coefficient(&call(&baseline, name, &[input]))
                .to_f64()
                .unwrap();
            let error = (actual - reference(f64::from(n) / 20.0)).abs();
            maximum = maximum.max(error);
            assert!(error <= bound, "{name}({n}/20): error={error}");
            assert!((actual - old).abs() <= old_bound + bound);
        }
        println!("{name}: largest sampled readout error {maximum:.3e}");
    }
    let pi = read_ratio(&call(&program, "pi", &[])).unwrap();
    println!("pi pattern readout: {:.15}", pi.to_f64().unwrap());
    // A rational enclosure checks more than f64's precision. The analytic tail
    // argument in the source establishes the bound; samples alone would not.
    assert!(pi > core::rational("3141592653589793236/1000000000000000000").unwrap());
    assert!(pi < core::rational("3141592653589793241/1000000000000000000").unwrap());
    let p = ratio(&program, "7", "3");
    let balanced = read_ratio(&call(&program, "balance", std::slice::from_ref(&p))).unwrap();
    let reflected = call(&program, "reciprocal", &[p]);
    assert_eq!(
        read_ratio(&call(&program, "balance", &[reflected])).unwrap(),
        -balanced
    );
}

#[test]
fn growth_flow_matches_exp_while_fraction_rotation_has_no_series_error() {
    let program = program();
    let baseline = baseline();
    let mut maximum = 0.0_f64;
    for n in -10..=10 {
        let x = scalar(&format!("{n}/10"));
        let result = call(&program, "exp", std::slice::from_ref(&x));
        assert!(result.camera(2, 0).is_zero());
        let actual = coefficient(&result.camera(1, 0)).to_f64().unwrap();
        let error = (actual - (f64::from(n) / 10.0).exp()).abs();
        maximum = maximum.max(error);
        assert!(error < 5.1e-13, "exp({n}/10): {error}");
        let old = coefficient(&call(&baseline, "exp", &[x])).to_f64().unwrap();
        assert!((actual - old).abs() < 5.1e-10);
    }
    println!("exp: largest sampled readout error {maximum:.3e}");
    for n in -5..=5 {
        let t = f64::from(n) / 10.0;
        let direction = call(&program, "rotation", &[scalar(&format!("{n}/10"))]);
        // Only the independent comparison converts the fraction to conventional
        // angular input; the NS rotation and its sine/cosine use no angle or pi.
        let angle = 2.0 * t.atan();
        for (name, expected) in [
            ("cos", angle.cos()),
            ("sin", angle.sin()),
            ("tan", angle.tan()),
        ] {
            let actual = read_ratio(&call(&program, name, std::slice::from_ref(&direction)))
                .unwrap()
                .to_f64()
                .unwrap();
            assert!((actual - expected).abs() < 5e-16, "{name}, t={t}");
        }
    }
}

#[test]
fn frequency_walk_matches_all_fourier_basis_vectors_and_keeps_its_clock() {
    let program = program();
    let baseline = baseline();
    for position in 0..4 {
        let mut samples = [
            NativeState::zero(),
            NativeState::zero(),
            NativeState::zero(),
            NativeState::zero(),
        ];
        samples[position] = NativeState::scalar(NativeScalar::from_text("2/3", "-5/7").unwrap());
        let transformed = call(&program, "fourier4", &samples);
        assert_eq!(transformed, call(&baseline, "fourier4", &samples));
        let original = call(&program, "tuple4", &samples);
        assert_eq!(call(&program, "inverse_fourier4", &[transformed]), original);
    }
    let mut inputs = vec![NativeState::scalar(
        NativeScalar::from_text("0", "-1").unwrap(),
    )];
    inputs.extend([scalar("1"), scalar("2"), scalar("3"), scalar("4")]);
    let walked = call(&program, "frequency", &inputs);
    assert_eq!(walked.camera(13, 0), scalar("4"));
    assert_eq!(walked.camera(11, 0), scalar("1"));
    let empty = call(&program, "frequency", &[scalar("1")]);
    assert!(empty.camera(12, 0).is_zero());
    assert!(empty.camera(13, 0).is_zero());
}

#[test]
fn complete_examples_agree_in_source_retained_vm_and_serialization() {
    for expression in [
        "fourier4(1, 2, 3, 4)",
        "pi()",
        "sqrt(5/4)",
        "log(5/4)",
        "exp(1)",
        "rotation(1/2)",
        "tan(rotation(1/2))",
        "depth_sqrt(depth(1))",
        "divmod(word4(-1,1,-1,-1), word4(-1,-1,1,1))",
        "trace(flow)",
    ] {
        let source = SOURCE.replace(
            "output fourier4(1, 2, 3, 4) as pattern",
            &format!("output {expression} as pattern"),
        );
        let program = core::parse(&source, "operation-patterns.ns").unwrap();
        let native = retained::interpret(&program).unwrap();
        let compiled = bytecode::compile(&program).unwrap();
        let replay = bytecode::execute_retained(&compiled).unwrap();
        assert!(replay.same_structure(&native), "{expression}");
        assert_eq!(replay.project(), &core::interpret(&program).unwrap());
        assert!(
            retained::State::from_data(&replay.native_data())
                .unwrap()
                .same_structure(&replay)
        );
    }
}

#[test]
fn closed_rotation_retains_its_steps_and_zero_scale_does_not_erase_inputs() {
    let program = program();
    let identity = retained::State::from_projection(&call(&program, "rotation", &[scalar("0")]));
    let quarter = retained::State::from_projection(&call(&program, "rotation", &[scalar("1")]));
    let tick = retained::State::from_projection(&scalar("0"));
    let walked = core::exact_function(&program, "walk")
        .unwrap()
        .apply_retained(&[
            identity.clone(),
            quarter,
            tick.clone(),
            tick.clone(),
            tick.clone(),
            tick,
        ])
        .unwrap();
    assert!(!walked.same_structure(&identity));
    assert_eq!(
        read_ratio(&call(&program, "cos", &[walked.project().clone()])).unwrap(),
        core::rational("1").unwrap()
    );
    assert!(walked.project().camera(2, 0).is_zero());
    let raw = retained::State::from_projection(&ratio(&program, "7", "3"));
    let scaled = core::exact_function(&program, "reframe")
        .unwrap()
        .apply_retained(&[raw.clone(), retained::State::from_projection(&scalar("0"))])
        .unwrap();
    assert!(scaled.project().is_zero());
    assert!(!scaled.same_structure(&retained::State::from_projection(&NativeState::zero())));
    assert!(
        scaled
            .retained_inputs()
            .iter()
            .any(|input| input.same_structure(&raw)),
        "the full ratio input must survive a zero scale"
    );
}
