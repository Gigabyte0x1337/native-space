// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

use native_space_language::core::{self, NativeScalar, OutputKind};
use native_space_language::retained::{self, Depth, Scalar, State};
use native_space_language::{Document, bytecode, parse_document};
use serde_json::json;

fn scalar(real: &str, imag: &str) -> State {
    State::scalar(NativeScalar::from_text(real, imag).unwrap())
}

fn run(source: &str) -> State {
    let Document::State(program) = parse_document(source, "retained.ns").unwrap() else {
        panic!("expected an executable document");
    };
    let direct = retained::interpret(&program).unwrap();
    let artifact = bytecode::compile(&program).unwrap();
    let vm = bytecode::execute_retained(&artifact).unwrap();
    assert!(direct.same_structure(&vm));
    assert!(direct.same_projection(&vm));
    assert_eq!(direct.project(), &core::interpret(&program).unwrap());
    let decoded = State::from_data(&vm.to_data()).unwrap();
    assert!(decoded.same_structure(&vm));
    assert!(decoded.same_projection(&vm));
    vm
}

#[test]
fn positive_one_is_the_depth_origin_and_zero_is_a_distinct_boundary() {
    for (value, expected) in [
        ("1/8", "1/64"),
        ("1/2", "1/4"),
        ("1", "1"),
        ("2", "4"),
        ("8", "64"),
    ] {
        let state = scalar(value, "0");
        assert_eq!(
            state.to_data()["coordinates"]["terms"][0]["depth"],
            if expected == "1" {
                json!({"kind":"finite", "value":"0"})
            } else {
                json!({"kind":"logarithmic", "base":"e", "factor":"1/2", "argument":expected})
            }
        );
    }
    assert!(Depth::of(&NativeScalar::one()).is_origin());
    assert!(Depth::of(&NativeScalar::zero()).is_zero_boundary());
    assert!(!Depth::of(&NativeScalar::zero()).is_origin());
    assert!(!State::one().same_projection(&State::zero()));
    assert_eq!(State::zero().to_data()["coordinates"]["phase"], json!(null));
}

#[test]
fn natural_log_depth_remains_exact_and_phase_is_separate() {
    let state = scalar("3/2", "0");
    assert_eq!(
        state.to_data()["coordinates"]["terms"][0]["depth"],
        json!({"kind":"logarithmic", "base":"e", "factor":"1/2", "argument":"9/4"})
    );
    for turns in 0..4 {
        let phased = state.phase(turns);
        assert_eq!(
            phased.to_data()["coordinates"]["terms"][0]["depth"],
            state.to_data()["coordinates"]["terms"][0]["depth"]
        );
    }
    assert_eq!(
        State::one().phase(1).to_data()["coordinates"]["terms"][0]["phase"],
        json!({"real":"0", "imag":"1"})
    );
}

#[test]
fn zero_multiplication_keeps_the_input_and_nested_operations_keep_the_branch() {
    let seven = scalar("7", "0");
    let hundred = scalar("100", "0");
    let cancelled = seven.multiply(&State::zero());
    let other = hundred.multiply(&State::zero());
    assert!(cancelled.same_projection(&other));
    assert!(!cancelled.same_structure(&other));
    assert!(!cancelled.same_structure(&State::zero()));
    assert!(cancelled.inputs()[0].same_structure(&seven));
    let next = cancelled.add(&State::one()).multiply(&scalar("3", "0"));
    assert!(next.same_projection(&scalar("3", "0")));
    assert!(next.inputs()[0].inputs()[0].same_structure(&cancelled));
    let saved = State::from_data(&next.to_data()).unwrap();
    assert!(saved.same_structure(&next));
}

#[test]
fn add_cancellation_and_zero_phase_and_index_remain_distinct_states() {
    let x = scalar("3/7", "-5/2").index_power(4, 9).unwrap();
    let cancelled = x.add(&x.phase(2));
    assert!(cancelled.project().is_zero());
    assert_eq!(cancelled.inputs().len(), 2);
    let rotated = cancelled.phase(1);
    let indexed = rotated.index_power(11, 2).unwrap();
    assert!(indexed.project().is_zero());
    assert!(!indexed.same_structure(&rotated));
    assert!(indexed.inputs()[0].inputs()[0].same_structure(&cancelled));
    State::zero().index_power(0, 2).unwrap_err();
    State::one().index_power(0, 2).unwrap_err();
}

#[test]
fn square_doubles_depth_and_self_addition_shifts_it_by_ln_two() {
    for value in ["1/8", "1/2", "1", "4"] {
        let x = scalar(value, "0");
        let q = Depth::of(&NativeScalar::from_text(value, "0").unwrap());
        assert_eq!(
            x.multiply(&x).to_data()["coordinates"]["terms"][0]["depth"],
            q.compose(&q).to_data()
        );
        assert_eq!(
            x.add(&x).to_data()["coordinates"]["terms"][0]["depth"],
            q.compose(&Depth::of(&NativeScalar::from_text("2", "0").unwrap()))
                .to_data()
        );
    }
}

#[test]
fn exact_complex_products_add_radial_depth_over_a_finite_rational_grid() {
    for a in -4..=4 {
        for b in -3..=3 {
            for c in -4..=4 {
                let left = NativeScalar::from_text(&format!("{a}/7"), &format!("{b}/3")).unwrap();
                let right = NativeScalar::from_text(&format!("{c}/5"), "2/9").unwrap();
                assert_eq!(
                    Depth::of(&left).compose(&Depth::of(&right)),
                    Depth::of(&left.multiply(&right))
                );
                let x = State::scalar(left.clone());
                let y = State::scalar(right.clone());
                assert!(x.add(&y).same_projection(&State::scalar(left.add(&right))));
                assert!(
                    x.multiply(&y)
                        .same_projection(&State::scalar(left.multiply(&right)))
                );
            }
        }
    }
}

#[test]
fn equal_mirror_mixture_squares_to_i_over_two_with_both_channels_retained() {
    let half = scalar("1/2", "0");
    let mixture = half.add(&half.phase(1));
    let squared = mixture.multiply(&mixture);
    assert!(squared.same_projection(&scalar("0", "1/2")));
    assert!(squared.inputs()[0].same_structure(&mixture));
    assert_eq!(squared.inputs()[0].inputs().len(), 2);
}

#[test]
fn interpreter_and_vm_retain_zero_products_and_function_expansion() {
    for source in [
        "output multiply(7, zero)",
        "output add(7, phase(2, 7))",
        "let x = index(4, 7)\noutput multiply(add(x, phase(2, x)), 9)",
        "let cancel = (x) => multiply(x, zero)\noutput add(cancel(7), 1)",
    ] {
        run(source);
    }
    let result = run("output multiply(7, zero)");
    assert!(
        result.to_data()["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["operator"]["operation"] == "multiply")
    );
}

#[test]
fn projection_output_is_explicit_and_does_not_mutate_retained_state() {
    let state = run("output multiply(7, zero)");
    assert_eq!(
        state.output_data(OutputKind::Number).unwrap(),
        json!({"kind":"number", "value":"0"})
    );
    assert_eq!(
        state.output_data(OutputKind::Auto).unwrap()["value"],
        state.to_data()
    );
    assert_eq!(
        state.output_data(OutputKind::Pattern).unwrap()["value"],
        state.to_data()
    );
    assert!(
        state.inputs()[0]
            .project()
            .0
            .values()
            .any(|value| value == &NativeScalar::from_text("7", "0").unwrap())
    );
}

#[test]
fn deserialization_rejects_forged_observations_invalid_edges_and_operators() {
    let data = scalar("7", "0").multiply(&State::zero()).to_data();
    for bad in [
        {
            let mut d = data.clone();
            d["projection"] = State::one().project().to_data();
            d
        },
        {
            let mut d = data.clone();
            d["coordinates"]["depth"] = json!({"kind":"finite", "value":"0"});
            d
        },
        {
            let mut d = data.clone();
            d["nodes"][0]["inputs"] = json!([0]);
            d
        },
        {
            let mut d = data.clone();
            d["root"] = json!(0);
            d
        },
        {
            let mut d = data.clone();
            d["nodes"][2]["operator"] = json!({"operation":"phase", "turns":9});
            d
        },
        {
            let mut d = data.clone();
            d["nodes"][2]["operator"] = json!({"operation":"unknown"});
            d
        },
    ] {
        State::from_data(&bad).unwrap_err();
    }
}

#[test]
fn feedback_and_save_load_preserve_shared_inputs_without_recursive_serialization() {
    let seed = scalar("7", "0");
    let zero = State::zero();
    let mut state = seed.clone();
    for _ in 0..128 {
        state = state.multiply(&zero).add(&State::one());
        state = State::from_data(&state.to_data()).unwrap();
    }
    assert!(state.same_projection(&State::one()));
    let data = state.to_data();
    assert!(
        data["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["operator"]["coordinates"]["squared_magnitude"] == "49")
    );
    let doubled = seed.add(&seed).to_data();
    assert_eq!(doubled["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(doubled["nodes"][1]["inputs"], json!([0, 0]));
}

#[test]
fn deep_shared_states_compare_serialize_and_drop_without_unfolding_the_host_stack() {
    let mut state = State::zero();
    for _ in 0..10_000 {
        state = state.phase(1);
    }
    let copy = State::from_data(&state.to_data()).unwrap();
    assert!(state.same_structure(&copy));
    assert!(state.same_projection(&copy));
    drop(state);
    drop(copy);
}

#[test]
fn native_scalar_coordinates_round_trip_and_multiply_before_classical_projection() {
    for a in -5..=5 {
        for b in -4..=4 {
            let x = NativeScalar::from_text(&format!("{a}/7"), &format!("{b}/3")).unwrap();
            let y = NativeScalar::from_text("5/9", "-3/4").unwrap();
            let native_x = Scalar::from_classical(&x);
            let native_y = Scalar::from_classical(&y);
            assert_eq!(native_x.project(), x);
            let reconstructed =
                Scalar::from_coordinates(native_x.depth().clone(), native_x.direction().cloned())
                    .unwrap();
            assert_eq!(native_x, reconstructed);
            assert_eq!(native_x.multiply(&native_y).project(), x.multiply(&y));
            assert_eq!(native_x.add(&native_y).project(), x.add(&y));
        }
    }
    let origin =
        Scalar::from_coordinates(Depth::of(&NativeScalar::one()), Some(NativeScalar::one()))
            .unwrap();
    assert!(origin.depth().is_origin());
    assert_eq!(origin.project(), NativeScalar::one());
    Scalar::from_coordinates(Depth::of(&NativeScalar::one()), None).unwrap_err();
    Scalar::from_coordinates(Depth::of(&NativeScalar::zero()), Some(NativeScalar::one())).unwrap();
    // A unit diagonal would require irrational coefficients; never round it.
    Scalar::from_coordinates(
        Depth::of(&NativeScalar::one()),
        Some(NativeScalar::from_text("1", "1").unwrap()),
    )
    .unwrap_err();
}

#[test]
fn cli_runs_retained_states_and_explicit_projections_and_names_zero_proofs() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state.ns");
    let invoke = |command: &str, source: &str| {
        std::fs::write(&path, source).unwrap();
        let result = std::process::Command::new(env!("CARGO_BIN_EXE_native-space"))
            .arg(command)
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout).unwrap()
    };
    let raw = invoke("run", "output multiply(7, zero)");
    let state = State::from_data(&serde_json::from_str(&raw).unwrap()).unwrap();
    assert!(state.project().is_zero());
    assert!(!state.same_structure(&State::zero()));
    let number = invoke("run", "output multiply(7, zero) as number");
    assert_eq!(number.trim(), "0");
    let proof = invoke("check", "multiply(7, zero) = 0");
    assert!(proof.contains("Valid classical-projection zero proof"));
    let one: serde_json::Value = serde_json::from_str(&invoke("run", "output 1")).unwrap();
    assert_eq!(one["coordinates"]["terms"][0]["depth"]["value"], "0");
}

#[test]
fn source_calls_cameras_folds_and_reflection_keep_hidden_inputs() {
    for source in [
        "let discard = (x) => 1\noutput discard(multiply(7, 0))",
        "let x = multiply(7, 0)\noutput 1",
        "output camera(3, 0, add(index(3, 1), index(4, multiply(7, 0))))",
        "let sum = (a, b) => add(a, b)\nlet f = (xs...) => fold(sum, 0, xs...)\noutput f(multiply(7, 0), 1)",
        "let discard = (x) => 1\noutput apply(trace(discard), multiply(7, 0))",
    ] {
        let state = run(source);
        assert!(state.same_projection(&State::one()));
        assert!(
            state.native_data()["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|node| node["operator"]["coordinates"]["squared_magnitude"] == "49"),
            "{source}"
        );
        assert!(
            state.native_data()["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|node| !node["retained"].as_array().unwrap().is_empty()
                    || node["operator"]["operation"] == "camera"),
            "{source}"
        );
    }
}

#[test]
fn branch_coordinates_follow_the_selected_frame_without_replacing_the_source() {
    let state = scalar("7", "0").multiply(&State::zero());
    let native = state.native_data();
    for (turns, real, imag) in [(0, "7", "0"), (1, "0", "7"), (2, "-7", "0"), (3, "0", "-7")] {
        let view = retained::numeric::view(&state, 1, turns, false).unwrap();
        assert_eq!(view["state"], native);
        assert_eq!(
            retained::coordinates::Point::from_data(&view["locations"][0]["points"][0]["point"])
                .unwrap()
                .scalar()
                .project(),
            NativeScalar::from_text(real, imag).unwrap()
        );
        assert_eq!(
            view["locations"][2]["points"][0]["point"]["vector"][0]["kind"],
            "zero_boundary"
        );
        let restored = State::from_data(&view["state"]).unwrap();
        assert!(restored.same_structure(&state));
    }
    assert!(
        state
            .branch(&[0])
            .unwrap()
            .same_projection(&scalar("7", "0"))
    );
    assert!(state.branch(&[2]).is_none());
    assert_eq!(state.native_data(), native);
}

#[test]
fn camera_keeps_the_entire_source_including_terms_outside_its_view() {
    let seven = scalar("7", "0").index_power(4, 1).unwrap();
    let source = seven.add(&scalar("100", "0").index_power(8, 1).unwrap());
    let selected = source.camera(4, 0);
    assert!(selected.same_projection(&scalar("7", "0")));
    assert!(selected.inputs()[0].same_structure(&source));
    let saved = State::from_data(&selected.native_data()).unwrap();
    assert!(saved.same_structure(&selected));
    assert!(
        saved.inputs()[0]
            .camera(8, 0)
            .same_projection(&scalar("100", "0"))
    );
}

#[test]
fn scope_edges_are_validated_and_sharing_is_preserved_without_camera_observations() {
    let input = scalar("7", "0").multiply(&State::zero());
    let state = State::one().retaining(&[input.clone(), input]);
    let native = state.native_data();
    let root = usize::try_from(native["root"].as_u64().unwrap()).unwrap();
    assert_eq!(
        native["nodes"][root]["retained"][0],
        native["nodes"][root]["retained"][1]
    );
    assert!(State::from_data(&native).unwrap().same_structure(&state));
    let mut bad = native.clone();
    bad["nodes"][root]["retained"] = json!([root]);
    State::from_data(&bad).unwrap_err();
    let mut bad = native;
    bad["nodes"][0]["retained"] = json!([9999]);
    State::from_data(&bad).unwrap_err();
}

#[test]
fn full_states_survive_cli_json_binary_and_feedback_round_trips() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("step.ns");
    let input = directory.path().join("input.json");
    let binary = directory.path().join("input.nsb");
    let saved = directory.path().join("saved.json");
    std::fs::write(
        &source,
        "let step = (x) => add(x, 1)\nlet first = (a, b) => a\noutput multiply(7, 0)",
    )
    .unwrap();
    let seeds = [scalar("7", "0"), scalar("100", "0")].map(|value| value.multiply(&State::zero()));
    std::fs::write(
        &input,
        serde_json::to_vec(&seeds.each_ref().map(State::native_data)).unwrap(),
    )
    .unwrap();
    let invoke = |args: &[&std::ffi::OsStr]| {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_native-space"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    };
    invoke(&["pack-data".as_ref(), input.as_os_str(), binary.as_os_str()]);
    let loaded = native_space_language::batch::read_data(&binary).unwrap();
    for (point, seed) in loaded.iter().zip(&seeds) {
        assert!(point.state().same_structure(seed));
    }
    let bytes = invoke(&[
        "batch".as_ref(),
        source.as_os_str(),
        "--function".as_ref(),
        "step".as_ref(),
        "--data".as_ref(),
        binary.as_os_str(),
        "--steps".as_ref(),
        "2".as_ref(),
    ]);
    let output: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(output["results"], json!(["2", "2"]));
    std::fs::write(&saved, bytes).unwrap();
    let loaded = native_space_language::batch::read_data(&saved).unwrap();
    assert!(!loaded[0].state().same_structure(loaded[1].state()));
    for (point, seed) in loaded.iter().zip(&seeds) {
        let previous = &point.state().retained_inputs()[0];
        assert!(previous.retained_inputs()[0].same_structure(seed));
    }
    let bytes = invoke(&[
        "run".as_ref(),
        source.as_os_str(),
        "--function".as_ref(),
        "first".as_ref(),
        "--data".as_ref(),
        saved.as_os_str(),
    ]);
    let state = State::from_data(&serde_json::from_slice(&bytes).unwrap()).unwrap();
    assert!(state.same_projection(&scalar("2", "0")));
    assert!(
        state
            .retained_inputs()
            .iter()
            .any(|branch| branch.same_structure(loaded[1].state()))
    );
    let view: serde_json::Value = serde_json::from_slice(&invoke(&[
        "view".as_ref(),
        source.as_os_str(),
        "--turns".as_ref(),
        "1".as_ref(),
    ]))
    .unwrap();
    assert!(
        State::from_data(&view["state"])
            .unwrap()
            .project()
            .is_zero()
    );
}

#[test]
fn simultaneous_classical_readers_do_not_change_native_state() {
    let state = scalar("7", "0").multiply(&State::zero()).add(&State::one());
    let before = state.native_data();
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..32 {
                    assert!(state.same_projection(&State::one()));
                    assert_eq!(state.native_data(), before);
                }
            });
        }
    });
}

#[test]
fn classical_input_lifting_preserves_index_depths_larger_than_u64() {
    let index = core::MultiIndex::from_depths([(9, u64::MAX)]).unwrap();
    let index = index.compose(&index);
    let projected = core::NativeState::from_terms([(index, NativeScalar::one())]);
    let state = State::from_projection(&projected);
    assert_eq!(state.project(), &projected);
    assert!(
        State::from_data(&state.native_data())
            .unwrap()
            .same_structure(&state)
    );
}

#[test]
fn vm_rejects_malformed_retention_with_a_source_location() {
    let program = core::parse("let f = (x) => 1\noutput f(7)", "scope.ns").unwrap();
    let artifact = bytecode::compile(&program).unwrap();
    for count in [-1, 0, i64::MAX] {
        let mut bad = artifact.clone();
        let instruction = bad
            .instructions
            .iter_mut()
            .find(|i| i.opcode == bytecode::Opcode::Retain)
            .unwrap();
        let span = instruction.span;
        instruction.operand = Some(bytecode::Operand::Integer(count));
        let error = bytecode::execute_retained(&bad).unwrap_err();
        assert_eq!(error.0.source_name, "scope.ns");
        assert_eq!(error.0.span, span);
        assert!(span.is_some());
    }
}
