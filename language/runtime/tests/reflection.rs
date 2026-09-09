// SPDX-License-Identifier: AGPL-3.0-or-later
use native_space_language::core::{Expr, NativeState};
use native_space_language::{bytecode, core};

#[test]
fn counted_index_and_full_unsigned_directions_round_trip() {
    let value = run("output index(18446744073709551615, 7, 18446744073709551615)");
    assert_eq!(value.0.len(), 1);
    let (index, _) = value.0.first_key_value().unwrap();
    assert_eq!(index.depth(u64::MAX).to_string(), u64::MAX.to_string());
    assert_eq!(
        run("output index(9, 7, 3)"),
        run("output index(9, index(9, index(9, 7)))")
    );
    assert_eq!(
        run("output camera(18446744073709551615, 0, index(18446744073709551615, 7))"),
        run("output 7")
    );
    for source in [
        "output index(18446744073709551616, 1)",
        "output index(1, 1, 0)",
        "output index(1, 1, -1)",
        "output index(1, 1, 1/2)",
    ] {
        let failure = core::parse(source, "index-error.ns").unwrap_err();
        assert!(failure.0.span.is_some());
    }
}
#[test]
fn apply_accepts_zero_arguments_and_variadic_graph_parameters() {
    assert_eq!(
        run("let f = () => 8\noutput apply(trace(f))"),
        run("output 8")
    );
    assert_eq!(
        run("let f = (xs...) => add(xs...)\noutput apply(trace(f), 2, 3, 4)"),
        run("output 9")
    );
}
#[test]
fn inserted_replacements_are_not_implicitly_revisited() {
    assert_eq!(
        run(r"
let from = (x) => multiply(x, one)
let to = (x) => multiply(add(x, 1), one)
let f = (x) => multiply(x, one)
let first = rewrite(trace(f), trace(from), trace(to))
output apply(rewrite(first, trace(from), trace(to)), 0)"),
        run("output 2")
    );
}
#[test]
fn rule_signatures_and_recursive_rule_roots_are_rejected() {
    rejects(
        r"
let from = (xs...) => add(xs...)
let to = () => 0
let f = () => add(1, 2)
output rewrite(trace(f), trace(from), trace(to))",
        "fixed parameters",
    );
    rejects(
        r"
let from = (x) => from(x)
let to = (x) => x
let f = (x) => x
output rewrite(trace(f), trace(from), trace(to))",
        "reference themselves",
    );
}
fn run(source: &str) -> NativeState {
    let program = core::parse(source, "reflection-test.ns").unwrap();
    assert_eq!(
        core::program_from_data(&core::program_to_data(&program)).unwrap(),
        program
    );
    let direct = core::interpret(&program).unwrap();
    let compiled = bytecode::compile(&program).unwrap();
    assert_eq!(direct, bytecode::execute(&compiled).unwrap());
    let expanded =
        native_space_language::expand_source(&native_space_language::Document::State(program))
            .unwrap();
    let reparsed = core::parse(&expanded, "expanded.ns").unwrap();
    assert_eq!(direct, core::interpret(&reparsed).unwrap());
    direct
}

fn rejects(source: &str, message: &str) {
    let program = core::parse(source, "rejected.ns").unwrap();
    for error in [
        core::interpret(&program).unwrap_err(),
        bytecode::compile(&program).unwrap_err(),
    ] {
        assert!(error.0.message.contains(message), "{error}");
        assert!(!error.0.source_name.is_empty());
        assert!(error.0.span.is_some());
    }
}
const RULE: &str = r"
let before = (a, b, x) => add(multiply(a, x), multiply(b, x))
let after = (a, b, x) => multiply(add(a, b), x)
";

#[test]
fn scalar_rewrite_executes_the_rebuilt_graph_and_keeps_inputs_dynamic() {
    let source = format!(
        "{RULE}
let f = (x) => index(9, add(multiply(2, x), multiply(3, x)))
let optimize = (g) => rewrite(g, trace(before), trace(after))
let g = optimize(trace(f))
output apply(g, 7)"
    );
    assert_eq!(run(&source), run("output index(9, 35)"));
    let program = core::parse(
        &format!(
            "{RULE}
let f = (x) => add(multiply(2, x), multiply(3, x))
let dynamic = (x) => apply(rewrite(trace(f), trace(before), trace(after)), x)
output 0"
        ),
        "dynamic.ns",
    )
    .unwrap();
    let callable = core::exact_function(&program, "dynamic").unwrap();
    for n in ["-2", "0", "3/7"] {
        assert_eq!(
            callable.apply(&[run(&format!("output {n}"))]).unwrap(),
            run(&format!("output multiply(5, {n})"))
        );
    }
}
#[test]
fn repeated_placeholder_mismatch_does_not_rewrite() {
    assert!(
        run(&format!(
            "{RULE}
let f = (x, y) => add(multiply(2, x), multiply(3, y))
let g = rewrite(trace(f), trace(before), trace(after))
output add(apply(g, 7, 11), -47)"
        ))
        .is_zero()
    );
}
#[test]
fn no_match_preserves_trace_and_nested_matches_are_bottom_up() {
    assert!(
        run(&format!(
            "{RULE}
let f = (x) => index(5, x)
output add(rewrite(trace(f), trace(before), trace(after)), phase(2, trace(f)))"
        ))
        .is_zero()
    );
    assert_eq!(
        run(r"
let from = (x) => multiply(x, one)
let to = (x) => x
let f = (x) => multiply(multiply(x, one), one)
output apply(rewrite(trace(f), trace(from), trace(to)), 9)"),
        run("output 9")
    );
}
#[test]
fn rewriting_is_not_an_equivalence_claim() {
    assert_eq!(
        run(r"
let from = (x) => multiply(x, one)
let to = (x) => add(x, 100)
let f = (x) => multiply(x, one)
output apply(rewrite(trace(f), trace(from), trace(to)), 7)"),
        run("output 107")
    );
}
#[test]
fn replacement_may_introduce_valid_helpers() {
    assert_eq!(
        run(r"
let from = (x) => multiply(x, one)
let twice = (x) => add(x, x)
let to = (x) => twice(x)
let f = (x) => multiply(x, one)
output apply(rewrite(trace(f), trace(from), trace(to)), 6)"),
        run("output 12")
    );
}
#[test]
fn invalid_rule_parameters_and_graphs_are_located_errors() {
    rejects("output apply(1, 7)", "canonical operation strand");
    rejects(
        "let f = (x) => x\noutput apply(trace(f))",
        "expects 1 arguments",
    );
    rejects(
        r"
let from = (x) => multiply(x, one)
let to = (y) => y
let f = (x) => multiply(x, one)
output rewrite(trace(f), trace(from), trace(to))",
        "bound by the pattern",
    );
    rejects(
        r"
let from = (x, y) => x
let to = (y) => y
let f = (x) => x
output rewrite(trace(f), trace(from), trace(to))",
        "does not occur",
    );
}
#[test]
fn recursive_graphs_remain_traceable_but_are_not_executable() {
    rejects("let f = (x) => f(x)\noutput apply(trace(f), 1)", "cyclic");
    rejects(
        "let f = (x) => apply(trace(f), x)\noutput apply(trace(f), 1)",
        "nested graph application",
    );
    assert!(!run("let f = (x) => f(x)\noutput trace(f)").is_zero());
}
#[test]
fn reflection_nodes_survive_strand_round_trips() {
    assert!(
        !run(&format!(
            "{RULE}
let meta = (g) => rewrite(g, trace(before), trace(after))
output untrace(trace(meta))"
        ))
        .is_zero()
    );
}

#[test]
fn compiled_application_contains_transformed_operations_not_just_an_answer() {
    let program = core::parse(
        &format!(
            "{RULE}
let f = (x) => add(multiply(2, x), multiply(3, x))
output apply(rewrite(trace(f), trace(before), trace(after)), 7)"
        ),
        "compiled.ns",
    )
    .unwrap();
    let code = bytecode::compile(&program).unwrap();
    // The rebuilt result is a product, not the constant 35. Counting all VM
    // instructions would also count identities and retained source graphs.
    let state = bytecode::execute_retained(&code).unwrap();
    let data = state.native_data();
    let root = usize::try_from(data["root"].as_u64().unwrap()).unwrap();
    assert_eq!(data["nodes"][root]["operator"]["operation"], "multiply");
    assert_eq!(state.inputs()[0].project(), &run("output 5"));
    assert_eq!(state.inputs()[1].project(), &run("output 7"));
    assert!(!state.retained_inputs().is_empty());
    assert_eq!(bytecode::execute(&code).unwrap(), run("output 35"));
    assert!(matches!(program.result, Expr::Reflect { .. }));
}
