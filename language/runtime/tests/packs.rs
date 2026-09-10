// SPDX-License-Identifier: AGPL-3.0-or-later

use native_space_language::{bytecode, compiled, core, retained, strand::execution::Value};

fn check(source: &str, expected: &str) {
    let program = core::parse(source, "pack.ns").unwrap();
    let expected = core::interpret(&core::parse(expected, "expected.ns").unwrap()).unwrap();
    let state = retained::interpret(&program).unwrap();
    assert_eq!(state.project(), &expected);
    let code = bytecode::lower(&program).unwrap();
    assert!(
        bytecode::execute_retained(&code)
            .unwrap()
            .same_structure(&state)
    );
    let artifact = compiled::compile(&program).unwrap();
    let loaded = compiled::Artifact::from_data(&artifact.to_data()).unwrap();
    assert_eq!(
        compiled::execute_retained(&loaded).unwrap().project(),
        &expected
    );
    let expanded =
        native_space_language::expand_source(&native_space_language::Document::State(program))
            .unwrap();
    let expanded = core::parse(&expanded, "expanded.ns").unwrap();
    assert_eq!(core::interpret(&expanded).unwrap(), expected);
}

#[test]
fn bare_pack_builds_one_based_indices_including_empty_and_zero_inputs() {
    for (arguments, expected) in [
        ("", "output 0"),
        ("0", "output index(1, 0)"),
        (
            "2, 0, 5",
            "output add(index(1, 2), index(2, 0), index(3, 5))",
        ),
        (
            "phase(1, 2), index(1, 3)",
            "output add(index(1, phase(1, 2)), index(2, index(1, 3)))",
        ),
    ] {
        check(
            &format!("let collect = (items...) => items\noutput collect({arguments})"),
            expected,
        );
    }
}

#[test]
fn bare_and_spread_uses_share_arguments_without_changing_each_other() {
    check(
        "let f = (items...) => add(items, add(items...), items)\noutput f(2,3)",
        "output add(index(1,4), index(2,6), 5)",
    );
    check(
        "let collect = (items...) => items\nlet forward = (head, tail...) => collect(tail...)\noutput forward(99,2,3)",
        "output add(index(1,2),index(2,3))",
    );
    check(
        "let first = (items...) => reflect(items, index(1,value),value)\noutput first(2,3,5)",
        "output 2",
    );
}

#[test]
fn zero_items_keep_their_retained_construction() {
    let program = core::parse(
        "let collect = (items...) => items\noutput collect(multiply(7,0))",
        "zero.ns",
    )
    .unwrap();
    let state = retained::interpret(&program).unwrap();
    assert!(state.project().is_zero());
    assert!(!state.same_structure(&retained::State::zero()));
    assert!(
        state.native_data()["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["operator"]["coordinates"]["squared_magnitude"] == "49")
    );
    let restored = retained::State::from_data(&state.to_data()).unwrap();
    assert!(restored.same_structure(&state));
}

#[test]
fn functions_in_packs_remain_callable_after_reflection() {
    let program = core::parse(
        "let collect = (items...) => items\nlet twice = (x) => multiply(2,x)\nlet first = (items) => reflect(items,index(1,value),value)\noutput first(collect(twice))(7)",
        "function-pack.ns",
    ).unwrap();
    let artifact = compiled::compile(&program).unwrap();
    let artifact = compiled::Artifact::from_data(&artifact.to_data()).unwrap();
    assert_eq!(
        compiled::execute_retained(&artifact).unwrap().project(),
        &core::interpret(&core::parse("output 14", "expected.ns").unwrap()).unwrap()
    );
    // Host input uses the same pack-building path, without source substitution.
    let graph = native_space_language::strand::execution::Graph::compile(&program).unwrap();
    let collected = graph
        .function("collect")
        .unwrap()
        .call(vec![Value::Function(graph.function("twice").unwrap())])
        .unwrap();
    let selected = graph
        .function("first")
        .unwrap()
        .call(vec![collected])
        .unwrap();
    let result = selected
        .call(vec![Value::State(retained::State::one())])
        .unwrap();
    assert_eq!(
        result.native().unwrap().project(),
        &core::interpret(&core::parse("output 2", "two.ns").unwrap()).unwrap()
    );
}
