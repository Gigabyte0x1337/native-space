// SPDX-License-Identifier: AGPL-3.0-or-later

use core::{NativeState, OutputKind};
use native_space_language::{bytecode, core, retained};

fn run(source: &str) -> NativeState {
    let program = core::parse(source, "value-reflection.ns").unwrap();
    let expected = core::interpret(&program).unwrap();
    let state = retained::interpret(&program).unwrap();
    assert_eq!(state.project(), &expected);
    let code = bytecode::lower(&program).unwrap();
    let replay = bytecode::execute_retained(&code).unwrap();
    assert!(replay.same_structure(&state));
    assert_eq!(replay.project(), &expected);
    let loaded = retained::State::from_data(&state.to_data()).unwrap();
    assert!(loaded.same_structure(&state));
    assert_eq!(loaded.project(), &expected);
    expected
}

#[test]
fn depth_capture_removes_and_rebuilds_the_whole_direction() {
    for depth in [1, 2, 17, u64::MAX] {
        let subject = format!("multiply(3, index(7, index(8, 2, 5), {depth}))");
        assert_eq!(
            run(&format!("output reflect({subject}, index(7, v, d), v)")),
            run("output index(8, 6, 5)")
        );
        assert_eq!(
            run(&format!(
                "output reflect({subject}, index(7, v, d), index(9, v, d))"
            )),
            run(&format!("output index(9, index(8, 6, 5), {depth})"))
        );
    }
}

#[test]
fn depth_capture_is_exact_beyond_u64_and_survives_serialization() {
    let subject = "index(7, index(7, phase(1, 6), 18446744073709551615), 18446744073709551615)";
    assert_eq!(
        run(&format!("output reflect({subject}, index(7, v, d), d)")),
        run("output 36893488147419103230")
    );
    assert_eq!(
        run(&format!(
            "output reflect({subject}, index(7, v, d), index(9, v, d))"
        )),
        run("output index(9, index(9, phase(1, 6), 18446744073709551615), 18446744073709551615)")
    );
}

#[test]
fn repeated_depth_name_is_an_equality_constraint() {
    for (a, b, expected) in [(3, 3, 6), (3, 4, 0)] {
        assert_eq!(
            run(&format!(
                "output reflect(index(7, index(8, 6, {b}), {a}), index(7, index(8, v, d), d), v)"
            )),
            run(&format!("output {expected}"))
        );
    }
}

#[test]
fn complete_depth_routing_agrees_with_camera_including_destination_collisions() {
    let subject = "add(index(7, index(9, 2, 3), 4), index(7, 5, 2), index(8, 99), index(7, -5, 2))";
    assert_eq!(
        run(&format!(
            "output reflect({subject}, index(7, v, d), index(9, v, d))"
        )),
        run(&format!(
            "output reflect({subject}, index(7, route_value, route_depth), index(9, route_value, route_depth))"
        ))
    );
    assert_eq!(
        run(&format!("output reflect({subject}, index(7, v, d), v)")),
        run(&format!(
            "output reflect({subject}, index(7, route_value, route_depth), route_value)"
        ))
    );
}

#[test]
fn ambiguous_and_out_of_scope_depth_captures_are_rejected() {
    for source in [
        "output index(7, 2, depth)",
        "output reflect(index(7, 2), index(7, v, v), v)",
        "output reflect(index(7, 2), index(7, index(7, v), d), v)",
        "output reflect(index(7, 2), index(7, index(7, v, a), b), v)",
        "output reflect(index(7, 2), index(7, v, d), index(8, v, unknown))",
    ] {
        let program = core::parse(source, "invalid-depth.ns").unwrap();
        assert!(core::interpret(&program).is_err(), "{source}");
        assert!(bytecode::lower(&program).is_err(), "{source}");
    }
}

#[test]
fn traced_depth_template_preserves_its_bindings() {
    assert_eq!(
        run(
            "let route = (x) => reflect(x, index(7, v, d), index(9, v, d))\noutput (route)(index(7, 6, 12))"
        ),
        run("output index(9, 6, 12)")
    );
}

#[test]
fn reflection_observes_scaled_values_not_construction_operands() {
    assert_eq!(
        run("output reflect(multiply(3, index(7, 2)), index(7, value), value)"),
        run("output 6")
    );
}

#[test]
fn equivalent_sources_have_equal_matches_and_keep_distinct_provenance() {
    for source in [
        "multiply(3, index(7, 2))",
        "index(7, multiply(3, 2))",
        "add(index(7, 1), index(7, 5))",
    ] {
        assert_eq!(
            run(&format!("output reflect({source}, index(7, v), v)")),
            run("output 6")
        );
    }
    let program = core::parse(
        "output reflect(multiply(3, index(7, 2)), index(7, v), v)",
        "retained.ns",
    )
    .unwrap();
    let state = retained::interpret(&program).unwrap();
    assert!(state.native_data().to_string().contains("multiply"));
}

#[test]
fn scaled_phased_added_and_cancelled_terms_are_matched_canonically() {
    assert_eq!(
        run(
            "output reflect(phase(1, multiply(3, add(index(7, 2), index(8, 3), index(7, 5)))), index(7, v), v)"
        ),
        run("output scalar(0, 21)")
    );
    assert_eq!(
        run("output reflect(add(index(7, 2), index(7, -2)), index(7, v), 99)"),
        NativeState::zero()
    );
    assert_eq!(
        run("output reflect(add(index(7, 2), index(7, 5)), index(7, v), 1)"),
        run("output 1")
    );
}

#[test]
fn full_source_defined_cartesian_transform_is_exact() {
    let source = r"
let rotate = (p) => add(
    reflect(p, index(1, v), index(2, v)),
    reflect(p, index(2, v), index(1, phase(2, v))),
    reflect(p, index(3, v), index(3, v))
)
let stretch = (p) => multiply(3, p)
let move = (p) => add(p, index(1, 5), index(2, -1), index(3, 2))
let transform = (p) => move(stretch(rotate(p)))
output transform(add(index(1, 2), index(2, 3), index(3, 4)))
";
    assert_eq!(
        run(source),
        run("output add(index(1, -4), index(2, 5), index(3, 14))")
    );
}

#[test]
fn templates_bind_lexically_without_capturing_outer_parameters() {
    assert_eq!(
        run("let f = (v) => reflect(v, index(7, v), v)\noutput f(index(7, 6))"),
        run("output 6")
    );
}

#[test]
fn index_order_is_canonical_and_one_wrapper_is_removed() {
    assert_eq!(
        run("output reflect(index(8, index(7, 6)), index(7, v), v)"),
        run("output index(8, 6)")
    );
    assert_eq!(
        run("output reflect(index(7, 6, 3), index(7, v), index(9, v))"),
        run("output index(9, index(7, 6, 2))")
    );
}

#[test]
fn pattern_phase_and_multiplicative_factor_are_inverted_exactly() {
    assert_eq!(
        run("output reflect(index(7, scalar(0, 6)), phase(1, multiply(3, index(7, v))), v)"),
        run("output 2")
    );
}

#[test]
fn no_match_is_zero_and_replacement_can_reuse_the_capture() {
    assert_eq!(
        run("output reflect(index(8, 6), index(7, v), 99)"),
        NativeState::zero()
    );
    assert_eq!(
        run("output reflect(index(7, 6), index(7, v), multiply(v, v))"),
        run("output 36")
    );
}

#[test]
fn ambiguous_or_unbound_patterns_fail_with_a_location() {
    for source in [
        "output reflect(6, add(a, b), a)",
        "output reflect(6, multiply(v, v), v)",
        "output reflect(6, multiply(0, v), v)",
        "output reflect(6, v, missing)",
        "output reflect(6, 6, 9)",
    ] {
        let program = core::parse(source, "invalid-rule.ns").unwrap();
        let error = core::interpret(&program).unwrap_err();
        assert_eq!(error.0.code, "NSR002", "{source}");
        assert!(error.0.span.is_some());
    }
}

#[test]
fn reflection_is_a_deferred_bytecode_instruction() {
    let program = core::parse(
        "output reflect(index(7, 6), index(7, v), multiply(v, v))",
        "compiled.ns",
    )
    .unwrap();
    let code = bytecode::lower(&program).unwrap();
    assert!(
        code.instructions
            .iter()
            .any(|i| i.opcode == bytecode::Opcode::Reflect)
    );
    assert_eq!(
        code.instructions
            .iter()
            .filter(|i| i.opcode == bytecode::Opcode::Reflect)
            .count(),
        1
    );
}

#[test]
fn rounded_execution_observes_prior_rounding() {
    let program = core::parse(
        "output reflect(index(7, add(9007199254740992, 1)), index(7, v), v) as number",
        "rounded.ns",
    )
    .unwrap();
    let state = retained::interpret(&program).unwrap();
    assert_eq!(state.project(), &run("output 9007199254740993"));
    let observed = retained::numeric::output(&state, OutputKind::Number).unwrap();
    assert!(observed.to_string().contains("9007199254740992"));
}

#[test]
fn traced_reflect_rules_round_trip_with_their_capture_scope() {
    assert_eq!(
        run("let f = (x) => reflect(x, index(7, v), v)\noutput (f)(index(7, 6))"),
        run("output 6")
    );
}

#[test]
fn function_graph_is_native_data_for_the_same_value_matcher() {
    // Compare two observations of the same graph, not different source catalogs.
    let subject = run("let f = (x) => multiply(3, index(7, 2))\noutput f");
    let source = format!(
        "let select = (t) => reflect(t, index({}, v, depth), v)\noutput 0",
        u64::MAX - 28
    );
    let program = core::parse(&source, "selector.ns").unwrap();
    let selector = core::exact_function(&program, "select").unwrap();
    let reflected = selector.apply(std::slice::from_ref(&subject)).unwrap();
    assert_eq!(reflected, subject.camera(u64::MAX - 28, 0));
    assert!(!reflected.is_zero());
}

#[test]
fn repeated_function_inputs_do_not_freeze_the_first_observation() {
    let program = core::parse(
        "let f = (x) => reflect(x, index(7, v), multiply(v, v))\noutput 0",
        "feedback.ns",
    )
    .unwrap();
    let function = core::exact_function(&program, "f").unwrap();
    for number in [2, 3, 5] {
        let input = run(&format!("output index(7, {number})"));
        assert_eq!(
            function.apply(&[input]).unwrap(),
            run(&format!("output {}", number * number))
        );
    }
}

#[test]
fn binary_batch_round_trip_preserves_the_rule_and_subject() {
    let program = core::parse(
        "output reflect(index(7, 6), index(7, v), multiply(v, v))",
        "binary.ns",
    )
    .unwrap();
    let state = retained::interpret(&program).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let json = directory.path().join("input.json");
    let binary = directory.path().join("input.nsb");
    std::fs::write(
        &json,
        serde_json::to_vec(&serde_json::json!([state.native_data()])).unwrap(),
    )
    .unwrap();
    native_space_language::batch::pack_data(&json, &binary).unwrap();
    let loaded = native_space_language::batch::read_data(&binary).unwrap();
    assert!(loaded[0].state().same_structure(&state));
    assert_eq!(loaded[0].projection(), state.project());
}

#[test]
fn malformed_compiled_rule_is_rejected_before_execution() {
    let program = core::parse(
        "output reflect(index(7, 6), index(7, v), multiply(v, v))",
        "invalid-wire.ns",
    )
    .unwrap();
    let mut code = bytecode::lower(&program).unwrap();
    let instruction = code
        .instructions
        .iter_mut()
        .find(|i| i.opcode == bytecode::Opcode::Reflect)
        .unwrap();
    let Some(bytecode::Operand::Rule(rule)) = &instruction.operand else {
        panic!("expected rule")
    };
    let mut data = serde_json::to_value(rule).unwrap();
    data["instructions"][2]["left"] = serde_json::json!(99);
    instruction.operand = Some(bytecode::Operand::Rule(
        serde_json::from_value(data).unwrap(),
    ));
    let error = bytecode::execute_retained(&code).unwrap_err();
    assert_eq!(error.0.code, "NSV011");
    assert!(error.0.message.contains("backward"));
}

#[test]
fn returned_value_can_be_reflected_again_after_scaling() {
    assert_eq!(
        run("let x = reflect(index(7, 2), index(7, v), index(9, v))\n\
             output reflect(multiply(3, x), index(9, v), v)"),
        run("output 6")
    );
}
