// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

use std::path::{Path, PathBuf};

use native_space_language::{Document, compile, expand_source, load_document, parse_document};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn relative_imports_load_the_canonical_function_library() {
    let path = repository().join("language/runtime/tests/fixtures/import-root.ns");
    let Document::Functions(library) = load_document(&path).unwrap() else {
        panic!("proof import must resolve to a function library");
    };
    assert!(library.imports.is_empty());
    assert!(
        library
            .functions
            .iter()
            .any(|function| function.name == "local_pattern")
    );
    assert!(
        library
            .functions
            .iter()
            .any(|function| function.name == "identity_phase")
    );
}

#[test]
fn import_cycles_report_the_cycle() {
    let path = repository().join("language/runtime/tests/fixtures/cycle-a.ns");
    let error = load_document(&path).unwrap_err();
    assert_eq!(error.0.code, "NSF-I002");
    assert!(error.0.message.contains("cycle-a.ns"));
    assert!(error.0.message.contains("cycle-b.ns"));
}

#[test]
fn duplicate_names_across_imports_are_rejected() {
    let path = repository().join("language/runtime/tests/fixtures/duplicate-root.ns");
    let error = load_document(&path).unwrap_err();
    assert_eq!(error.0.code, "NSF-I004");
    assert!(error.0.message.contains("shared"));
}

#[test]
fn complete_function_libraries_are_validated_before_check_or_compile() {
    for (name, code) in [
        ("invalid-unknown-call.ns", "NSF-S007"),
        ("invalid-arity.ns", "NSF-S008"),
    ] {
        let path = repository()
            .join("language/runtime/tests/fixtures")
            .join(name);
        let error = load_document(&path).unwrap_err();
        assert_eq!(error.0.code, code, "{name}");
        assert_eq!(error.0.source_name, path.display().to_string(), "{name}");
        assert!(error.0.span.is_some(), "{name}");
    }

    let invalid = parse_document("let broken = () =>\nmissing()", "memory.ns").unwrap();
    let error = compile(&invalid).unwrap_err();
    assert_eq!(error.0.code, "NSF-S007");
    assert_eq!(error.0.source_name, "memory.ns");
}

#[test]
fn indexed_layout_expansion_is_visible_pure_source() {
    let document = parse_document(
        "let parameters = (a,b) => add(index(9,a),index(9,b,2))\n\
         add(parameters(2, 3), phase(2, add(index(9, 2), index(9, index(9, 3))))) = 0",
        "variadic.ns",
    )
    .unwrap();
    let expanded = expand_source(&document).unwrap();

    assert!(!expanded.contains("concat"));
    assert!(!expanded.contains("..."));
    assert!(!expanded.contains("parameters("));
    assert!(expanded.contains("index(9"));
    let Document::State(program) = parse_document(&expanded, "expanded.ns").unwrap() else {
        panic!("expanded source must remain an exact-state document");
    };
    assert!(
        native_space_language::core::interpret(&program)
            .unwrap()
            .is_zero()
    );
}

#[test]
fn retired_builtins_are_ordinary_names_not_hidden_operations() {
    use native_space_language::core;
    for name in [
        "trace",
        "untrace",
        "camera",
        "length",
        "rank_descent",
        "rewrite",
        "concat",
        "fold",
    ] {
        let source = format!("let {name} = (x) => x\noutput {name}(7)");
        let program = core::parse(&source, "ordinary.ns").unwrap();
        assert_eq!(
            core::interpret(&program).unwrap(),
            core::NativeState::scalar(core::NativeScalar::from_text("7", "0").unwrap())
        );
        let undeclared = core::parse(&format!("output {name}(7)"), "missing.ns").unwrap();
        core::interpret(&undeclared).unwrap_err();
    }
}
#[test]
fn literal_and_call_ast_have_one_form_each() {
    use native_space_language::core::{self, Expr};
    for value in ["0", "1", "zero", "one", "scalar(2,3)"] {
        assert!(matches!(
            core::parse(&format!("output {value}"), "literal.ns")
                .unwrap()
                .result,
            Expr::Literal { .. }
        ));
    }
    for value in ["f(0)", "(f)(0)", "f(0)(1)"] {
        assert!(matches!(
            core::parse(&format!("let f = (x) => x\noutput {value}"), "call.ns")
                .unwrap()
                .result,
            Expr::Call { .. }
        ));
    }
    for kind in [
        "zero",
        "one",
        "scalar",
        "invoke",
        "trace",
        "untrace",
        "camera",
        "length",
        "rank_descent",
        "sequence_at",
        "concat",
        "fold",
    ] {
        serde_json::from_value::<Expr>(serde_json::json!({"kind":kind,"span":null})).unwrap_err();
    }
}
#[test]
fn calling_a_non_program_is_a_located_runtime_error() {
    let Document::State(program) = parse_document("output (one)(1)", "invalid.ns").unwrap() else {
        panic!("state")
    };
    let error = native_space_language::core::interpret(&program).unwrap_err();
    assert!(error.0.span.is_some());
}

#[test]
fn optimizer_executes_every_authorized_rule_and_preserves_the_state() {
    let source = r"
output add(
    add(add(2, 3), zero),
    multiply(multiply(2, 3), one),
    multiply(zero, one),
    phase(0, one),
    phase(1, phase(3, one))
)
";
    let Document::State(program) = parse_document(source, "optimizer.ns").unwrap() else {
        panic!("optimizer source must be an exact state document");
    };
    let direct = native_space_language::core::interpret(&program).unwrap();
    let result = native_space_language::core::optimize(&program).unwrap();
    let optimized = native_space_language::core::interpret(&result.program).unwrap();
    assert_eq!(optimized, direct);

    let actual = result
        .events
        .iter()
        .map(|event| event.rule_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let expected = std::collections::BTreeSet::from([
        "OPT-ADD-FLATTEN-1",
        "OPT-ADD-ZERO-1",
        "OPT-MUL-FLATTEN-1",
        "OPT-MUL-ONE-1",
        "OPT-MUL-ZERO-1",
        "OPT-PHASE-COMBINE-1",
        "OPT-PHASE-IDENTITY-1",
        "OPT-PHASE-NORMALIZE-1",
    ]);
    assert_eq!(actual, expected);

    // Check the emitted rule metadata directly; archived research prose is not
    // a runtime dependency or a substitute for the state-equivalence check above.
    for event in &result.events {
        let expected = match event.rule_id.as_str() {
            "OPT-ADD-FLATTEN-1" | "OPT-ADD-ZERO-1" => "L-NS-2",
            "OPT-MUL-ZERO-1" => "L-NS-8",
            "OPT-MUL-FLATTEN-1" => "L-NS-5",
            "OPT-MUL-ONE-1" => "L-NS-6",
            "OPT-PHASE-COMBINE-1" | "OPT-PHASE-IDENTITY-1" | "OPT-PHASE-NORMALIZE-1" => "L-SEP-5",
            rule => panic!("unexpected optimizer rule {rule}"),
        };
        assert_eq!(event.theorem_ids, [expected], "{}", event.rule_id);
    }
}

#[test]
fn operator_definition_order_sets_precedence() {
    let Document::State(program) = parse_document(
        "operator \"*\" = (left, right) => multiply(left, right)\n\
         operator \"-\" = (left, right) => add(left, phase(2, right))\n\
         output 10 - 2 * 3",
        "operators.ns",
    )
    .unwrap() else {
        panic!("operator example must be an exact state document");
    };
    let result = native_space_language::core::interpret(&program).unwrap();
    assert_eq!(
        native_space_language::core::output_data(&result, program.output_kind).unwrap()["value"],
        "4"
    );
}

#[test]
fn core_operations_cannot_be_overridden() {
    for source in [
        "let add = (left, right) => left\noutput 1",
        "operator \"ADD\" = (left, right) => left\noutput 1",
        "let ADD = () =>\nADD()",
    ] {
        let error = parse_document(source, "override.ns").unwrap_err();
        assert!(
            matches!(error.0.code.as_str(), "NSS008" | "NSF-S006"),
            "{}",
            error.0.code
        );
    }
    for name in ["=>", "=", "let", "output", "as", "import"] {
        let source = format!("operator {name:?} = (left, right) => left\noutput 1");
        let error = parse_document(&source, "reserved-operator.ns").unwrap_err();
        assert_eq!(error.0.code, "NSS008", "{name}");
    }
}

#[test]
fn one_namespace_rejects_declaration_collisions() {
    for source in [
        "let same = (value) => value\noperator \"same\" = (left, right) => left\noutput 1",
        "operator \"same\" = (left, right) => left\noperator \"same\" = (left, right) => right\noutput 1",
        "let same = (value) => value\nlet same = 1\noutput same",
    ] {
        let error = parse_document(source, "collision.ns").unwrap_err();
        assert_eq!(error.0.code, "NSS009", "{source}");
    }
}

#[test]
fn parameters_cannot_collide_with_language_names() {
    for source in [
        "let f = (add) => add\noutput 1",
        "operator \"-\" = (left, output) => left\noutput 1",
        "parameter add: bool\nprove true by truth_table",
    ] {
        let error = parse_document(source, "parameter-collision.ns").unwrap_err();
        assert_eq!(error.0.code, "NSS008", "{source}");
    }
}

#[test]
fn boolean_example_is_recomputed_exhaustively() {
    let Document::Logic(program) = parse_document("parameter a: bool\nparameter b: bool\nprove iff(not(and(a, b)), or(not(a), not(b))) by truth_table", "logic.ns").unwrap() else {
        panic!()
    };
    let report = native_space_language::logic::verify(
        &native_space_language::logic::compile(&program).unwrap(),
    )
    .unwrap();
    assert_eq!(report.valuation_count, 4);
}

#[test]
fn every_compiled_artifact_starts_at_schema_version_one() {
    for source in [
        "output add(1, 2)",
        "let identity = () =>\nPHASE(0)",
        "parameter a: bool\nparameter b: bool\nprove iff(not(and(a, b)), or(not(a), not(b))) by truth_table",
    ] {
        let document = parse_document(source, "schema.ns").unwrap();
        let artifact = compile(&document).unwrap();
        assert_eq!(artifact["version"], 1, "{source}");
    }
}

#[test]
fn parser_reports_a_source_location() {
    let error = parse_document("output scalar(1/, 0)", "broken.ns").unwrap_err();
    assert_eq!(error.0.span.unwrap().start_line, 1);
    assert_eq!(error.0.source_name, "broken.ns");
}

#[test]
fn phase_accepts_only_canonical_source_turns() {
    for turns in ["-1", "4", "8"] {
        let source = format!("output phase({turns}, one)");
        let error = parse_document(&source, "invalid-phase.ns").unwrap_err();

        assert_eq!(error.0.code, "NST002", "{turns}");
        assert_eq!(error.0.span.unwrap().start_line, 1, "{turns}");
    }
}

#[test]
fn semantic_analysis_rejects_noncanonical_phase_ast() {
    let Document::State(mut program) =
        parse_document("output phase(1, one)", "invalid-phase-ast.ns").unwrap()
    else {
        panic!("source must parse as an exact state");
    };
    let native_space_language::core::Expr::Phase { turns, .. } = &mut program.result else {
        panic!("source result must remain a phase expression");
    };
    *turns = 4;

    let compile_error = native_space_language::bytecode::lower(&program).unwrap_err();
    assert_eq!(compile_error.0.code, "NST002");
    assert!(compile_error.0.span.is_some());

    let error = native_space_language::core::interpret(&program).unwrap_err();
    assert_eq!(error.0.code, "NST002");
    assert!(error.0.span.is_some());
}

#[test]
fn functions_and_bindings_may_be_interleaved() {
    let source = "let value = one\n\
                  let identity = (input) => input\n\
                  output identity(value)";
    let Document::State(program) = parse_document(source, "interleaved.ns").unwrap() else {
        panic!("source must parse as an exact state");
    };

    assert_eq!(
        native_space_language::core::interpret(&program).unwrap(),
        native_space_language::core::NativeState::one()
    );
}

#[test]
fn quarter_turn_cycle_and_indexed_helix_are_checked_separately() {
    let cycle = "let j = scalar(0, 1)\nmultiply(j, j, j, j) = one";
    let Document::State(cycle) = parse_document(cycle, "phase-cycle.ns").unwrap() else {
        panic!("cycle must parse as an exact zero proof");
    };
    assert!(
        native_space_language::core::interpret(&cycle)
            .unwrap()
            .is_zero()
    );

    let helix = "let step = (value) => add(index(7, value), phase(1, value))\n\
                 let once = step(one)\n\
                 let twice = step(once)\n\
                 let three = step(twice)\n\
                 let four = step(three)\n\
                 output four";
    let Document::State(helix) = parse_document(helix, "indexed-helix.ns").unwrap() else {
        panic!("helix must parse as an exact state");
    };
    let direct = native_space_language::core::interpret(&helix).unwrap();
    let bytecode = native_space_language::bytecode::lower(&helix).unwrap();

    assert_ne!(direct, native_space_language::core::NativeState::one());
    assert_eq!(
        native_space_language::bytecode::execute(&bytecode).unwrap(),
        direct
    );
}

#[test]
fn unified_calls_check_arity_without_rejecting_partial_or_shadowed_calls() {
    use native_space_language::core;
    let invalid = core::parse("let f = (x) => x\noutput f(1, 2)", "arity.ns").unwrap();
    assert!(
        core::analyze(&invalid)
            .iter()
            .any(|diagnostic| diagnostic.code == "NSS004")
    );
    core::interpret(&invalid).unwrap_err();
    for source in [
        "let f = (x, y) => add(x, y)\noutput f(1)(2)",
        "let f = (x) => x\nlet g = (a,b) => add(a,b)\nlet invoke = (f) => f(1,2)\noutput invoke(g)",
    ] {
        let program = core::parse(source, "calls.ns").unwrap();
        assert_eq!(
            core::interpret(&program).unwrap(),
            core::NativeState::scalar(core::NativeScalar::from_text("3", "0").unwrap())
        );
    }
}
