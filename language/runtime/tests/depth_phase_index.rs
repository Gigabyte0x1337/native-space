// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

use native_space_language::{
    bytecode,
    core::{self, MultiIndex, NativeScalar, OutputKind, Rational},
    retained::{
        self, Depth, Scalar, State,
        coordinates::{Point, quadratic_pair},
        numeric,
    },
};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use serde_json::json;

fn scalar(real: &str, imag: &str) -> Scalar {
    Scalar::from_classical(&NativeScalar::from_text(real, imag).unwrap())
}

fn run(source: &str) -> State {
    let program = core::parse(source, "model.ns").unwrap();
    let direct = retained::interpret(&program).unwrap();
    let artifact = bytecode::lower(&program).unwrap();
    let restored = bytecode::BytecodeProgram::from_data(&artifact.to_data()).unwrap();
    let vm = bytecode::execute_retained(&restored).unwrap();
    assert!(vm.same_structure(&direct));
    assert!(vm.same_projection(&direct));
    vm
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 2e-12, "{actual} != {expected}");
}

#[test]
fn exact_point_round_trip_retains_index_and_unevaluated_logarithms_and_radicals() {
    for index in [
        BigUint::zero(),
        BigUint::from(30_u8),
        BigUint::one() << 150_u32,
    ] {
        for (a, b) in [("3", "4"), ("1", "1"), ("-7/13", "2/11"), ("0", "0")] {
            let point = Point::new(scalar(a, b), index.clone());
            assert_eq!(Point::from_data(&point.to_data()).unwrap(), point);
            assert_eq!(
                point.scalar().project(),
                NativeScalar::from_text(a, b).unwrap()
            );
            // Classical values alone omit k. Supply it explicitly on the return trip.
            assert_eq!(
                Point::new(
                    Scalar::from_classical(&point.scalar().project()),
                    index.clone()
                ),
                point
            );
            let mut forged = point.to_data();
            forged["vector"][1] = json!(123);
            Point::from_data(&forged).unwrap_err();
        }
    }
    let diagonal = Point::new(scalar("1", "1"), BigUint::zero());
    assert_eq!(
        diagonal.vector()[0],
        json!({"kind":"logarithmic", "base":"e", "factor":"1/2", "argument":"2"})
    );
    assert_eq!(
        diagonal.vector()[1],
        json!({"kind":"radical", "factor":"1", "radicand":"1/2"})
    );
    let origin = Point::new(scalar("1", "0"), BigUint::zero());
    assert_eq!(
        origin.to_f64().unwrap().map(f64::to_bits),
        [0.0_f64, 1.0, 0.0].map(f64::to_bits)
    );
}

#[test]
fn multiplication_square_reciprocal_and_phase_match_complex_arithmetic() {
    for a in -3..=3 {
        for b in -3..=3 {
            let point = Point::new(scalar(&a.to_string(), &b.to_string()), 21_u8.into());
            let factor = scalar("3/5", "4/5");
            assert_eq!(
                point.multiply(&factor).scalar().project(),
                point.scalar().project().multiply(&factor.project())
            );
            assert_eq!(
                point.square().scalar().project(),
                point.scalar().project().multiply(&point.scalar().project())
            );
            assert_eq!(point.square().index(), point.index());
            if a == 0 && b == 0 {
                point.reciprocal().unwrap_err();
            } else {
                assert_eq!(
                    point
                        .multiply(point.reciprocal().unwrap().scalar())
                        .scalar()
                        .project(),
                    NativeScalar::one()
                );
                assert_eq!(point.reciprocal().unwrap().reciprocal().unwrap(), point);
            }
        }
    }
    for (turns, expected) in [
        (0, [1.0, 0.0]),
        (1, [0.0, 1.0]),
        (2, [-1.0, 0.0]),
        (3, [0.0, -1.0]),
    ] {
        let value = run(&format!("output phase({turns}, 1)"));
        let point = Point::new(
            Scalar::from_classical(&value.project().0[&MultiIndex::default()]),
            BigUint::zero(),
        );
        let [x, y, z] = point.to_f64().unwrap();
        close(x, 0.0);
        close(y, expected[0]);
        close(z, expected[1]);
    }
    let removed = core::parse("output orient(1, 1)", "removed.ns").unwrap();
    bytecode::lower(&removed).unwrap_err();
}

#[test]
fn boundary_phase_and_index_survive_multiplication_serialization_and_vm_storage() {
    let boundary =
        Scalar::from_coordinates(Depth::of(&NativeScalar::zero()), Some(NativeScalar::one()))
            .unwrap();
    let point = Point::new(boundary.clone(), 17_u8.into());
    let rotated = point.multiply(&scalar("0", "1"));
    assert_eq!(
        rotated.to_f64().unwrap().map(f64::to_bits),
        [f64::NEG_INFINITY, 0.0, 18.0].map(f64::to_bits)
    );
    assert_eq!(Point::from_data(&rotated.to_data()).unwrap(), rotated);
    assert_eq!(rotated.scalar().project(), NativeScalar::zero());
    let state = State::native_scalar(rotated.scalar())
        .index_power(7, 17)
        .unwrap();
    assert!(
        State::from_data(&state.native_data())
            .unwrap()
            .same_structure(&state)
    );
    let mut code = bytecode::lower(&core::parse("output 0", "boundary.ns").unwrap()).unwrap();
    for instruction in &mut code.instructions {
        if instruction.opcode == bytecode::Opcode::PushScalar {
            instruction.operand = Some(bytecode::Operand::Scalar {
                coordinates: retained::ScalarData {
                    squared_magnitude: "0".into(),
                    direction: Some(retained::RayData {
                        real: "0".into(),
                        imag: "1".into(),
                    }),
                },
            });
        }
    }
    let restored = bytecode::execute_retained(&code).unwrap();
    assert_eq!(
        restored.output_data(OutputKind::Vector).unwrap()["value"][2],
        json!({"kind":"finite", "value":"1"})
    );
    let view = numeric::view(&state, 7, 0, false).unwrap();
    assert_eq!(
        view["locations"].as_array().unwrap().last().unwrap()["points"][0]["point"]["index"],
        "17"
    );
    let rounded = numeric::view(&state, 7, 0, true).unwrap();
    assert_eq!(
        rounded["locations"].as_array().unwrap().last().unwrap()["points"][0]["point"]["vector"],
        json!(["-infinity", 0.0, 18.0])
    );
    let turned = numeric::view(&state.phase(1), 7, 0, true).unwrap();
    assert_eq!(
        turned["locations"].as_array().unwrap().last().unwrap()["points"][0]["point"]["vector"],
        json!(["-infinity", -18.0, 0.0])
    );
}

#[test]
fn constant_phase_step_is_recovered_from_spiral_and_index_survives_wrapping() {
    let step = scalar("3/5", "4/5");
    let mut point = Point::new(scalar("1", "0"), BigUint::zero());
    let expected_step = 4.0_f64.atan2(3.0);
    for k in 1_u32..=96 {
        let before = point.to_f64().unwrap();
        point = point.multiply(&step).advance(&BigUint::one());
        let after = point.to_f64().unwrap();
        close(
            (before[1] * after[2] - before[2] * after[1])
                .atan2(before[1] * after[1] + before[2] * after[2]),
            expected_step,
        );
        close(after[1].hypot(after[2]), f64::from(k + 1));
        assert_eq!(point.index(), &BigUint::from(k));
    }
    let mut wrapped = Point::new(scalar("1", "0"), BigUint::zero());
    for _ in 0..8 {
        wrapped = wrapped.multiply(&scalar("0", "1")).advance(&BigUint::one());
    }
    assert_eq!(wrapped.scalar(), &scalar("1", "0"));
    assert_eq!(wrapped.index(), &BigUint::from(8_u8));
}

#[test]
fn fourier_with_multiple_channels_survives_exact_coordinate_round_trip() {
    let samples = [
        scalar("1", "2"),
        scalar("3", "-1"),
        scalar("-2", "4"),
        scalar("7", "1/2"),
    ];
    let restored: Vec<_> = samples
        .iter()
        .enumerate()
        .map(|(k, value)| {
            let point = Point::new(value.clone(), BigUint::from(k));
            Point::from_data(&point.to_data())
                .unwrap()
                .scalar()
                .project()
        })
        .collect();
    let dft = |input: &[NativeScalar], inverse: bool| -> Vec<NativeScalar> {
        (0_usize..4)
            .map(|k| {
                (0_usize..4).fold(NativeScalar::zero(), |sum, n| {
                    let phase = i64::try_from((k * n) % 4).unwrap();
                    let turns = if inverse { phase } else { (4 - phase) % 4 };
                    sum.add(&input[n].phase(turns))
                })
            })
            .collect()
    };
    let spectrum = dft(&restored, false);
    assert!(spectrum.iter().filter(|v| !v.is_zero()).count() > 1);
    let recovered = dft(&spectrum, true);
    for (sample, recovered) in samples.iter().zip(recovered) {
        assert_eq!(
            sample.project(),
            recovered.multiply(&NativeScalar::from_text("1/4", "0").unwrap())
        );
    }
}

#[test]
fn compact_and_quadratic_views_are_derived_not_the_core_or_number_theory_proofs() {
    let a = Point::new(scalar("3", "4"), 123_u8.into());
    let b = Point::new(scalar("-2", "1"), 456_u16.into());
    let sphere = a.compact_f64().unwrap();
    let half_depth = 5.0_f64.ln() / 2.0;
    close(sphere[0], half_depth.tanh());
    close(sphere[1], half_depth.cosh().recip() * 0.6);
    close(sphere[2], half_depth.cosh().recip() * 0.8);
    close(sphere.iter().map(|v| v * v).sum(), 1.0);
    assert_eq!(
        a.advance(&BigUint::one())
            .compact_f64()
            .unwrap()
            .map(f64::to_bits),
        sphere.map(f64::to_bits)
    );
    let pair = quadratic_pair(&a, &b);
    let total = a.scalar().depth().squared_magnitude() + b.scalar().depth().squared_magnitude();
    assert_eq!(
        pair.iter().map(|v| v * v).sum::<Rational>(),
        &total * &total
    );
    // R+Y is intensity |a+b|², not the complex value a+b.
    assert_eq!(
        &total + &pair[1],
        Depth::of(&a.scalar().project().add(&b.scalar().project()))
            .squared_magnitude()
            .clone()
    );
    assert_eq!(
        quadratic_pair(
            &a.multiply(&scalar("0", "1")),
            &b.multiply(&scalar("0", "1"))
        ),
        pair
    );
    // The z -> z² coordinate map is checked above. No Collatz or RH theorem follows.
}

#[test]
fn exact_depth_handles_extreme_magnitudes_without_floating_zero_collapse() {
    for exponent in [1000_usize, 3000] {
        let denominator = BigUint::from(10_u8).pow(u32::try_from(exponent).unwrap());
        let point = Point::new(scalar(&format!("1/{denominator}"), "0"), BigUint::zero());
        assert!(!point.scalar().depth().is_zero_boundary());
        let x = point.to_f64().unwrap()[0];
        assert!(x.is_finite() && x < -2000.0);
        Point::from_data(&point.to_data()).unwrap();
    }
}

#[test]
fn rounded_mode_rounds_each_step_and_reflection_remains_in_the_graph() {
    for source in [
        "output add(add(10000000000000000, 1), -10000000000000000) as number",
        "output reflect(add(add(index(7, 10000000000000000), index(7, 1)), index(7, -10000000000000000)), index(7, route_value, route_depth), route_value) as number",
    ] {
        let state = run(source);
        assert_eq!(state.output_data(OutputKind::Number).unwrap()["value"], "1");
        assert_eq!(
            numeric::output(&state, OutputKind::Number).unwrap()["value"],
            "0"
        );
        let rounded = numeric::output(&state, OutputKind::Pattern).unwrap();
        assert_eq!(rounded["value"]["state"], state.native_data());
        assert_eq!(rounded["value"]["approximate"], true);
        State::from_data(&rounded["value"]).unwrap_err();
    }
    let source =
        run("output reflect(index(7, 1), index(7, route_value, route_depth), route_value)");
    assert!(
        source.native_data()["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["operator"]["operation"] == "reflect")
    );
}

#[test]
fn numerical_failures_are_explicit_located_and_never_proofs() {
    let huge = "1".to_owned() + &"0".repeat(400);
    for source in [
        format!("output {huge}"),
        format!("output 1/{huge}"),
        format!(
            "let tiny = 1/{}\noutput multiply(tiny, tiny)",
            "1".to_owned() + &"0".repeat(200)
        ),
    ] {
        let state = run(&source);
        let error = numeric::output(&state, OutputKind::Pattern).unwrap_err();
        assert!(
            error.contains("operation") && error.contains("line"),
            "{error}"
        );
        assert!(!state.project().is_zero());
    }
    let state = run("output 1/3");
    assert_eq!(
        state.output_data(OutputKind::Number).unwrap()["value"],
        "1/3"
    );
    numeric::output(&state, OutputKind::Boolean).unwrap_err();
    numeric::output(&state, OutputKind::String).unwrap_err();
    let unused =
        State::one().retaining(&[State::scalar(NativeScalar::from_text(&huge, "0").unwrap())]);
    assert_eq!(
        numeric::output(&unused, OutputKind::Number).unwrap()["value"],
        "1"
    );
    // Asking to view every branch does evaluate that otherwise-unused literal.
    numeric::view(&unused, 1, 0, true).unwrap_err();
}

#[test]
fn empty_selection_reads_as_zero_without_erasing_indexed_zero_locations() {
    let empty = run("output reflect(1, index(7, route_value, route_depth), route_value)");
    assert_eq!(
        empty.output_data(OutputKind::Vector).unwrap()["value"],
        json!([{"kind":"zero_boundary"}, null, null])
    );
    assert_eq!(
        numeric::output(&empty, OutputKind::Vector).unwrap()["value"],
        json!(["-infinity", null, null])
    );
    let indexed_zero = run("output index(7, 0)");
    indexed_zero.output_data(OutputKind::Vector).unwrap_err();
    numeric::output(&indexed_zero, OutputKind::Vector).unwrap_err();
    let view = numeric::view(&indexed_zero, 7, 0, false).unwrap();
    assert_eq!(
        view["locations"].as_array().unwrap().last().unwrap()["points"][0]["point"]["index"],
        "1"
    );
}

#[test]
fn cli_precision_choice_and_default_vector_are_end_to_end() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("precision.ns");
    std::fs::write(
        &file,
        "output add(add(10000000000000000, 1), -10000000000000000) as number",
    )
    .unwrap();
    let invoke = |extra: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_native-space"))
            .arg("run")
            .arg(&file)
            .args(extra)
            .output()
            .unwrap()
    };
    assert_eq!(String::from_utf8(invoke(&[]).stdout).unwrap().trim(), "1");
    assert_eq!(
        String::from_utf8(invoke(&["--numeric", "f64"]).stdout)
            .unwrap()
            .trim(),
        "0"
    );
    std::fs::write(&file, "output add(3, phase(1, 4)) as vector").unwrap();
    let exact = invoke(&[]);
    assert!(exact.status.success());
    let exact: serde_json::Value = serde_json::from_slice(&exact.stdout).unwrap();
    assert_eq!(exact[0]["argument"], "25");
    let float = invoke(&["--numeric", "f64"]);
    assert!(float.status.success());
    let float: serde_json::Value = serde_json::from_slice(&float.stdout).unwrap();
    close(float[0].as_f64().unwrap(), 5.0_f64.ln());
    close(float[1].as_f64().unwrap(), 0.6);
    close(float[2].as_f64().unwrap(), 0.8);
}
