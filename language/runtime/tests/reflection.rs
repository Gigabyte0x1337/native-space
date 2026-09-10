// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-level rewrite tooling is explicit Rust API, not an extra Native operation.
use native_space_language::{core, reflection, retained::State, strand::execution::Value};

fn function(source: &str, name: &str) -> core::NativeState {
    core::interpret(&core::parse(&format!("{source}\noutput {name}"), "rewrite.ns").unwrap())
        .unwrap()
}
fn rewrite(
    target: &core::NativeState,
    pattern: &core::NativeState,
    replacement: &core::NativeState,
) -> Result<core::NativeState, core::LanguageError> {
    let expression = reflection::rewrite(
        &[target.clone(), pattern.clone(), replacement.clone()],
        "rewrite.ns",
        None,
    )?;
    core::interpret(&core::Program {
        functions: vec![],
        bindings: vec![],
        result: expression,
        goal: core::Goal::Emit,
        output_kind: core::OutputKind::Pattern,
        source_name: "rewrite.ns".into(),
        span: None,
    })
}
fn call(program: &core::NativeState, input: i64) -> core::NativeState {
    Value::State(State::from_projection(program))
        .call(vec![Value::State(State::scalar(
            core::NativeScalar::from_text(&input.to_string(), "0").unwrap(),
        ))])
        .unwrap()
        .native()
        .unwrap()
        .project()
        .clone()
}
fn number(value: i64) -> core::NativeState {
    core::NativeState::scalar(core::NativeScalar::from_text(&value.to_string(), "0").unwrap())
}
#[test]
fn host_rewrite_preserves_calls_and_captured_values() {
    let target = function("let f = (x) => add(multiply(x,1),2)", "f");
    let pattern = function("let before = (x) => multiply(x,1)", "before");
    let replacement = function("let after = (x) => x", "after");
    let result = rewrite(&target, &pattern, &replacement).unwrap();
    for input in -8..9 {
        assert_eq!(call(&result, input), call(&target, input));
    }
}
#[test]
fn inserted_replacements_are_not_implicitly_revisited() {
    let target = function("let f = (x) => multiply(x,1)", "f");
    let pattern = function("let before = (x) => multiply(x,1)", "before");
    let replacement = function("let after = (x) => multiply(add(x,1),1)", "after");
    let once = rewrite(&target, &pattern, &replacement).unwrap();
    let twice = rewrite(&once, &pattern, &replacement).unwrap();
    assert_eq!(call(&once, 0), number(1));
    assert_eq!(call(&twice, 0), number(2));
}
#[test]
fn repeated_placeholder_mismatch_does_not_rewrite() {
    let pattern = function("let before = (x) => multiply(x,x)", "before");
    let replacement = function("let after = (x) => 0", "after");
    let target = function("let f = (x) => multiply(x,2)", "f");
    assert_eq!(
        call(&rewrite(&target, &pattern, &replacement).unwrap(), 3),
        number(6)
    );
}
#[test]
fn replacement_can_intentionally_change_the_result() {
    let target = function("let f = (x) => multiply(x,1)", "f");
    let pattern = function("let before = (x) => multiply(x,1)", "before");
    let replacement = function("let after = (x) => add(x,1)", "after");
    assert_eq!(
        call(&rewrite(&target, &pattern, &replacement).unwrap(), 7),
        number(8)
    );
}
#[test]
fn helper_calls_survive_replacement() {
    let target = function("let f = (x) => multiply(x,1)", "f");
    let pattern = function("let before = (x) => multiply(x,1)", "before");
    let replacement = function(
        "let helper = (x) => add(x,2)\nlet after = (x) => helper(x)",
        "after",
    );
    assert_eq!(
        call(&rewrite(&target, &pattern, &replacement).unwrap(), 7),
        number(9)
    );
}
#[test]
fn invalid_rule_signatures_and_self_references_are_rejected() {
    let target = function("let f = (x) => x", "f");
    let replacement = function("let after = (x) => x", "after");
    for pattern in [
        function("let before = (xs...) => add(xs...)", "before"),
        function("let before = (x) => before(x)", "before"),
    ] {
        rewrite(&target, &pattern, &replacement).unwrap_err();
    }
    let missing = function("let after = (x,y) => add(x,y)", "after");
    let pattern = function("let before = (x) => multiply(x,1)", "before");
    rewrite(&target, &pattern, &missing).unwrap_err();
}
#[test]
fn malformed_and_bound_programs_are_rejected_by_source_tooling() {
    let pattern = function("let before = (x) => multiply(x,1)", "before");
    let replacement = function("let after = (x) => x", "after");
    rewrite(&number(1), &pattern, &replacement).unwrap_err();
    let partial = core::interpret(
        &core::parse("let f = (a,b) => add(a,b)\noutput f(0)", "partial.ns").unwrap(),
    )
    .unwrap();
    assert!(
        rewrite(&partial, &pattern, &replacement)
            .unwrap_err()
            .0
            .message
            .contains("unbound")
    );
}
#[test]
fn recursive_graph_is_data_but_execution_is_bounded() {
    let graph = function("let f = (x) => f(x)", "f");
    assert!(!graph.is_zero());
    let error = Value::State(State::from_projection(&graph))
        .call(vec![Value::State(State::one())])
        .unwrap_err();
    assert!(error.0.message.contains("call-depth limit"));
}
#[test]
fn colliding_helper_definitions_are_rejected() {
    let target = function(
        "let helper = (x) => add(x,1)\nlet f = (x) => multiply(helper(x),1)",
        "f",
    );
    let pattern = function("let before = (x) => multiply(x,1)", "before");
    let replacement = function(
        "let helper = (x) => add(x,2)\nlet after = (x) => helper(x)",
        "after",
    );
    rewrite(&target, &pattern, &replacement).unwrap_err();
}
