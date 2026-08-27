// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

use std::fs;
use std::path::{Path, PathBuf};

use native_space_language::{Document, compile, load_document, parse_document};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn document(name: &str) -> Document {
    let path = repository().join("examples").join(name);
    load_document(&path).unwrap()
}

#[test]
fn every_example_is_source_or_valid_host_data() {
    fn check(directory: &Path) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                check(&path);
                continue;
            }
            match path.extension().and_then(|value| value.to_str()) {
                Some("ns") => {
                    load_document(&path)
                        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                }
                Some("json") => {
                    native_space_language::batch::read_data(&path)
                        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                }
                Some("csv") => {
                    native_space_language::continuation::read_observations_csv(&path)
                        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                }
                extension => panic!(
                    "unexpected example format {extension:?}: {}",
                    path.display()
                ),
            }
        }
    }
    check(&repository().join("examples"));
}

#[test]
fn relative_imports_load_the_canonical_function_library() {
    let path = repository().join("examples/math-functions.ns");
    let Document::Functions(library) = load_document(&path).unwrap() else {
        panic!("proof import must resolve to a function library");
    };
    assert!(library.imports.is_empty());
    assert!(
        library
            .functions
            .iter()
            .any(|function| function.name == "centered_re_perspective")
    );
    assert!(
        library
            .functions
            .iter()
            .any(|function| function.name == "zeta_classical_pattern")
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
fn application_witnesses_are_exact_zero_proofs() {
    for name in [
        "matrix-distributivity.ns",
        "dynamics-orient-equivariance.ns",
        "nbody-perspective-zero.ns",
        "navier-stokes-index-composition.ns",
        "molecular-interaction-order.ns",
        "programming-language-data.ns",
    ] {
        let path = repository().join("examples/applications").join(name);
        let source = fs::read_to_string(&path).unwrap();
        let Document::State(program) = parse_document(&source, path.to_str().unwrap()).unwrap()
        else {
            panic!("{name} must be an exact state proof");
        };
        assert_eq!(program.goal, native_space_language::core::Goal::ProveZero);
        let direct = native_space_language::core::interpret(&program).unwrap();
        let bytecode = native_space_language::bytecode::compile(&program).unwrap();
        assert!(direct.is_zero(), "{name}");
        assert_eq!(
            native_space_language::bytecode::execute(&bytecode).unwrap(),
            direct,
            "{name}"
        );
        assert_eq!(bytecode.goal, program.goal);
        assert_eq!(bytecode.output_kind, program.output_kind);
    }
}

#[test]
fn projection_counterexample_proves_both_exact_residuals() {
    let Document::State(program) = document("projection-zero-fiber-counterexample.ns") else {
        panic!("counterexample must be an exact state proof");
    };
    assert_eq!(program.goal, native_space_language::core::Goal::ProveZero);
    assert!(
        native_space_language::core::interpret(&program)
            .unwrap()
            .is_zero()
    );
}

#[test]
fn finite_examples_agree_between_evaluator_and_vm() {
    for name in [
        "basic.ns",
        "classic_identities.ns",
        "operators.ns",
        "orientation_zero.ns",
        "primes.ns",
        "trace.ns",
        "untrace-with-errors.ns",
        "untrace.ns",
        "utf8.ns",
    ] {
        let Document::State(program) = document(name) else {
            panic!("{name} must be a state document");
        };
        let direct = native_space_language::core::interpret(&program).unwrap();
        let bytecode = native_space_language::bytecode::compile(&program).unwrap();
        assert_eq!(
            native_space_language::bytecode::execute(&bytecode).unwrap(),
            direct,
            "{name}"
        );
        assert_eq!(bytecode.version, 1);
        assert_eq!(bytecode.output_kind, program.output_kind);
    }
}

#[test]
fn untrace_error_ratio_is_an_exact_number_from_zero_through_one() {
    for invalid in ["-1", "3/2"] {
        let source =
            format!("output untrace(add(index(1, 1), index(2, 1), index(3, 1)), {invalid})");
        let error = parse_document(&source, "invalid-budget.ns").unwrap_err();

        assert_eq!(error.0.code, "NSP051", "{invalid}");
        assert!(error.0.span.is_some(), "{invalid}");
    }
}

#[test]
fn optimizer_executes_every_authorized_rule_and_preserves_the_state() {
    let source = r"
output add(
    add(add(2, 3), zero),
    multiply(multiply(2, 3), one),
    multiply(zero, one),
    orient(8, one),
    orient(1, orient(3, one))
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
        "OPT-ORIENT-COMBINE-1",
        "OPT-ORIENT-IDENTITY-1",
        "OPT-ORIENT-NORMALIZE-1",
    ]);
    assert_eq!(actual, expected);

    let ledger = fs::read_to_string(repository().join("proofs/00-dependency-ledger.md")).unwrap();
    for event in &result.events {
        for theorem in &event.theorem_ids {
            assert!(
                ledger.contains(&format!("| {theorem} |")),
                "optimizer theorem {theorem} is missing from the dependency ledger"
            );
        }
    }
}

#[test]
fn operator_definition_order_sets_precedence() {
    let Document::State(program) = document("operators.ns") else {
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
fn mathematical_function_examples_are_source_defined() {
    for name in ["zeta.ns", "dual_alignment.ns", "re_critical_line.ns"] {
        let Document::Functions(library) = document(name) else {
            panic!("{name} must contain source-defined functions")
        };
        assert!(!library.functions.is_empty(), "{name}");
    }
}

#[test]
fn boolean_example_is_recomputed_exhaustively() {
    let Document::Logic(program) = document("boolean_logic.ns") else {
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
    for name in ["basic.ns", "zeta.ns", "boolean_logic.ns"] {
        let artifact = compile(&document(name)).unwrap();
        assert_eq!(artifact["version"], 1, "{name}");
    }
}

#[test]
fn parser_reports_a_source_location() {
    let error = parse_document("output scalar(1/, 0)", "broken.ns").unwrap_err();
    assert_eq!(error.0.span.unwrap().start_line, 1);
    assert_eq!(error.0.source_name, "broken.ns");
}
