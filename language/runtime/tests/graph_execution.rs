// SPDX-License-Identifier: AGPL-3.0-or-later

use native_space_language::{
    core,
    retained::State,
    strand::execution::{Graph, Value},
};

fn number(value: i64) -> Value {
    Value::State(State::scalar(
        core::NativeScalar::from_text(&value.to_string(), "0").unwrap(),
    ))
}

#[test]
fn currying_reuses_the_graph_and_distinguishes_a_bound_zero() {
    let program = core::parse(
        "let f = (a,b) => add(multiply(a,2),b)\noutput 0",
        "curry.ns",
    )
    .unwrap();
    let graph = Graph::compile(&program).unwrap();
    let f = graph.function("f").unwrap();
    let Value::Function(g) = f.call(vec![number(3)]).unwrap() else {
        panic!("expected partial function")
    };
    assert!(f.shares_graph(&g));
    let result = g
        .call(vec![number(4)])
        .unwrap()
        .state("test", None)
        .unwrap();
    assert_eq!(
        result.project(),
        &core::NativeState::scalar(core::NativeScalar::from_text("10", "0").unwrap())
    );
    let Value::Function(z) = f.call(vec![number(0)]).unwrap() else {
        panic!("zero must bind one slot")
    };
    assert!(z.shares_graph(&f));
    assert_eq!(
        z.call(vec![number(4)])
            .unwrap()
            .state("test", None)
            .unwrap()
            .project(),
        number(4).state("test", None).unwrap().project()
    );
    assert_eq!(
        g.call(vec![number(5)])
            .unwrap()
            .state("test", None)
            .unwrap()
            .project(),
        number(11).state("test", None).unwrap().project()
    );
}

#[test]
fn chained_source_calls_execute_the_strand_records() {
    let program = core::parse(
        "let f = (a,b) => add(multiply(a,2),b)\nlet g = f(3)\noutput add(g(4), f(0)(5))",
        "source.ns",
    )
    .unwrap();
    let graph = Graph::compile(&program).unwrap();
    assert_eq!(
        graph.run().unwrap().state("test", None).unwrap().project(),
        number(15).state("test", None).unwrap().project()
    );
}

#[test]
fn recursive_calls_are_finite_references_with_bounded_execution() {
    let program = core::parse("let f = (x) => f(add(x,1))\noutput 0", "recursive.ns").unwrap();
    let graph = Graph::compile(&program).unwrap();
    let f = graph.function("f").unwrap();
    let error = f.call(vec![number(0)]).unwrap_err();
    assert!(error.to_string().contains("call-depth limit"));
    assert!(error.to_string().contains("recursive.ns"));
}

#[test]
fn compilation_and_loading_do_not_execute_recursive_bodies() {
    use native_space_language::compiled;
    let program = core::parse("let f = (x) => f(x)\noutput f(0)", "cycle.ns").unwrap();
    let artifact = compiled::compile(&program).unwrap();
    let restored = compiled::Artifact::from_data(&artifact.to_data()).unwrap();
    assert!(
        compiled::execute_retained(&restored)
            .unwrap_err()
            .to_string()
            .contains("call-depth limit")
    );
}

#[test]
fn portable_partial_function_preserves_zero_ray_and_indexed_arguments() {
    use native_space_language::retained::{Depth, Scalar};
    let program = core::parse("let f = (a,b) => a\noutput 0", "portable.ns").unwrap();
    let graph = Graph::compile(&program).unwrap();
    let zero = State::native_scalar(
        &Scalar::from_coordinates(
            Depth::of(&core::NativeScalar::zero()),
            Some(core::NativeScalar::from_text("0", "1").unwrap()),
        )
        .unwrap(),
    )
    .index_power(1024, 7)
    .unwrap();
    let partial = graph
        .function("f")
        .unwrap()
        .call(vec![Value::State(zero)])
        .unwrap();
    let encoded = partial.native().unwrap();
    let restored = Value::State(State::from_data(&encoded.native_data()).unwrap());
    let expected = partial
        .call(vec![number(3)])
        .unwrap()
        .state("test", None)
        .unwrap();
    let actual = restored
        .call(vec![number(3)])
        .unwrap()
        .state("test", None)
        .unwrap();
    assert!(actual.same_structure(&expected));
}

#[test]
fn functions_are_native_data_and_reflected_data_is_callable() {
    let program = core::parse(
        "let f = (x) => multiply(2,x)\nlet p = reflect(f, value, value)\noutput p(7)",
        "reflect-call.ns",
    )
    .unwrap();
    assert_eq!(
        core::interpret(&program).unwrap(),
        number(14).native().unwrap().project().clone()
    );
}

#[test]
fn higher_order_calls_and_empty_partial_calls_keep_their_bindings() {
    let program = core::parse("let twice = (x) => multiply(2,x)\nlet use = (f,x) => f(x)\nlet p = use(twice)\noutput p()(9)", "higher-order.ns").unwrap();
    assert_eq!(
        core::interpret(&program).unwrap(),
        number(18).native().unwrap().project().clone()
    );
}

#[test]
fn a_function_parameter_shadows_the_global_function_signature() {
    let program = core::parse("let f = (x) => x\nlet product = (a,b) => multiply(a,b)\nlet use = (f) => f(2,3)\noutput use(product)", "shadow.ns").unwrap();
    assert_eq!(
        core::interpret(&program).unwrap(),
        number(6).native().unwrap().project().clone()
    );
}

#[test]
fn a_function_argument_survives_partial_function_serialization() {
    let program = core::parse(
        "let twice = (x) => multiply(2,x)\nlet use = (f,x) => f(x)\noutput use(twice)",
        "saved-higher-order.ns",
    )
    .unwrap();
    let partial = Graph::compile(&program)
        .unwrap()
        .run()
        .unwrap()
        .native()
        .unwrap();
    let saved = State::from_data(&partial.native_data()).unwrap();
    let output = Value::State(saved)
        .call(vec![number(9)])
        .unwrap()
        .native()
        .unwrap();
    assert_eq!(output.project(), number(18).native().unwrap().project());
}

#[test]
fn traced_and_simplified_higher_order_functions_keep_their_dependencies() {
    let source = "let twice = (x) => multiply(2,x)\nlet use = (f,x) => f(x)\nlet wrapper = (x) => add(0,use(twice,x))\noutput wrapper";
    let program = core::parse(source, "dependencies.ns").unwrap();
    assert_eq!(
        {
            let state = core::interpret(&program).unwrap();
            let expression = native_space_language::strand::optimize_operation_strand(
                &state,
                "1",
                "dependencies.ns",
                None,
            )
            .unwrap()
            .unwrap();
            let candidate = core::Program {
                functions: vec![],
                bindings: vec![],
                result: expression,
                goal: core::Goal::Emit,
                output_kind: core::OutputKind::Pattern,
                source_name: "optimized.ns".into(),
                span: None,
            };
            Value::State(State::from_projection(
                &core::interpret(&candidate).unwrap(),
            ))
            .call(vec![number(7)])
            .unwrap()
            .native()
            .unwrap()
            .project()
            .clone()
        },
        number(14).native().unwrap().project().clone()
    );
}

#[test]
fn cli_runs_currying_and_checks_a_serialized_graph() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("calls.ns");
    std::fs::write(
        &path,
        "let f = (a,b) => add(multiply(a,2),b)\noutput f(3)(4) as number",
    )
    .unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_native-space"))
        .arg("run")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8(result.stdout).unwrap().trim(), "10");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_native-space"))
        .arg("check")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn stored_program_rejects_unknown_metadata() {
    let program = core::parse("output 1", "metadata.ns").unwrap();
    let mut data = native_space_language::compiled::compile(&program)
        .unwrap()
        .to_data();
    data["unexpected"] = true.into();
    native_space_language::compiled::Artifact::from_data(&data).unwrap_err();
}
