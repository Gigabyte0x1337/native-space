// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Expands source-defined functions without assigning meaning to their names.

use std::collections::BTreeMap;

use crate::derivation::{Function, Instruction, Library, parse, validate};

use super::{
    DeriveError,
    model::{
        CORE_OPERATIONS, DerivationReport, PatternReference, PrimitiveStep, SourceLocation,
        StepKind, TraceStep,
    },
};

const LIBRARY_SOURCE: &str = include_str!("../../../functions.ns");
const LIBRARY_NAME: &str = "language/functions.ns";

#[derive(Debug)]
struct TraceState {
    active: Vec<String>,
    events: Vec<TraceStep>,
    pattern_references: Vec<PatternReference>,
}

#[derive(Debug)]
struct Catalog<'a> {
    functions: BTreeMap<&'a str, &'a Function>,
}

impl<'a> Catalog<'a> {
    fn new(library: &'a Library) -> Self {
        Self {
            functions: library
                .functions
                .iter()
                .map(|function| (function.name.as_str(), function))
                .collect(),
        }
    }

    fn function(&self, name: &str) -> Result<&'a Function, DeriveError> {
        self.functions
            .get(name)
            .copied()
            .ok_or_else(|| DeriveError::unknown_function(name))
    }
}

fn shipped_library() -> Result<Library, DeriveError> {
    let library = parse(LIBRARY_SOURCE, LIBRARY_NAME)
        .map_err(|error| DeriveError::invalid_source(error.to_string()))?;
    validate(&library).map_err(|error| DeriveError::invalid_source(error.to_string()))?;
    Ok(library)
}

pub(crate) fn function_names() -> Result<Vec<String>, DeriveError> {
    Ok(shipped_library()?
        .functions
        .into_iter()
        .map(|function| function.name)
        .collect())
}

/// Derive a function from the shipped Native Space source library.
///
/// # Errors
///
/// Returns a located source, arity, or unknown-function diagnostic.
pub fn derive(
    function: impl AsRef<str>,
    arguments: &[String],
) -> Result<DerivationReport, DeriveError> {
    derive_from_source(LIBRARY_SOURCE, LIBRARY_NAME, function, arguments)
}

/// Derive a function from caller-provided Native Space source.
///
/// This is the purity boundary: the implementation parses and expands generic
/// syntax only. It contains no catalog of mathematical names or theorem shapes.
///
/// # Errors
///
/// Returns a located source, arity, or unknown-function diagnostic.
pub fn derive_from_source(
    source: &str,
    source_name: &str,
    function: impl AsRef<str>,
    arguments: &[String],
) -> Result<DerivationReport, DeriveError> {
    let library = parse(source, source_name)
        .map_err(|error| DeriveError::invalid_source(error.to_string()))?;
    validate(&library).map_err(|error| DeriveError::invalid_source(error.to_string()))?;
    derive_from_library(&library, function, arguments)
}

/// Derive a function from a parsed and import-resolved library.
///
/// Recursive calls close finite pattern references instead of expanding again.
///
/// # Errors
///
/// Returns an arity or unknown-function diagnostic.
pub fn derive_from_library(
    library: &Library,
    function: impl AsRef<str>,
    arguments: &[String],
) -> Result<DerivationReport, DeriveError> {
    validate(library).map_err(|error| DeriveError::invalid_source(error.to_string()))?;
    let catalog = Catalog::new(library);
    let function = function.as_ref();
    let definition = catalog.function(function)?;
    let mut state = TraceState {
        active: Vec::new(),
        events: Vec::new(),
        pattern_references: Vec::new(),
    };
    trace_function(definition, arguments, &catalog, &mut state, 0)?;

    let primitive_steps = state
        .events
        .iter()
        .filter(|event| event.kind != StepKind::FunctionExpansion)
        .enumerate()
        .map(|(index, event)| PrimitiveStep {
            number: index + 1,
            kind: event.kind,
            name: event.name.clone(),
            arguments: event.arguments.clone(),
            source: event.source.clone(),
            function_trace: event.function_trace.clone(),
        })
        .collect::<Vec<_>>();

    let used_operations = CORE_OPERATIONS
        .iter()
        .filter(|operation| {
            primitive_steps
                .iter()
                .any(|step| step.kind == StepKind::CoreOperation && step.name == **operation)
        })
        .map(|operation| (*operation).to_owned())
        .collect();

    Ok(DerivationReport {
        schema: "native-space-operation-derivation",
        version: 1,
        function: definition.name.clone(),
        applied_arguments: arguments.to_vec(),
        language_operations: CORE_OPERATIONS,
        used_operations,
        primitive_steps,
        pattern_references: state.pattern_references,
        function_trace: state.events,
    })
}

fn trace_function(
    function: &Function,
    arguments: &[String],
    catalog: &Catalog<'_>,
    state: &mut TraceState,
    depth: usize,
) -> Result<(), DeriveError> {
    validate_arity(function, arguments.len())?;

    let environment = bind(function, arguments);
    state.active.push(function.name.clone());
    for instruction in &function.body {
        match instruction {
            Instruction::Operation {
                operation,
                arguments,
                span,
            } => state.events.push(TraceStep {
                function: function.name.clone(),
                depth,
                kind: StepKind::CoreOperation,
                name: operation.name().to_owned(),
                arguments: arguments.clone(),
                source: SourceLocation::from_span(&function.source_name, *span),
                function_trace: state.active.clone(),
            }),
            Instruction::Call {
                function: called,
                arguments: supplied,
                span,
            } => {
                let targets = environment
                    .get(called.as_str())
                    .cloned()
                    .unwrap_or_else(|| vec![called.clone()]);
                if environment.contains_key(called.as_str()) && !supplied.is_empty() {
                    return Err(DeriveError::invalid_source(format!(
                        "{}:{}: dynamic function parameter {called:?} cannot receive arguments",
                        function.source_name, span.start_line
                    )));
                }
                let resolved_arguments = supplied
                    .iter()
                    .flat_map(|argument| {
                        environment
                            .get(argument.as_str())
                            .cloned()
                            .unwrap_or_else(|| vec![argument.clone()])
                    })
                    .collect::<Vec<_>>();
                for target in targets {
                    let source = SourceLocation::from_span(&function.source_name, *span);
                    state.events.push(TraceStep {
                        function: function.name.clone(),
                        depth,
                        kind: StepKind::FunctionExpansion,
                        name: target.clone(),
                        arguments: resolved_arguments.clone(),
                        source: source.clone(),
                        function_trace: state.active.clone(),
                    });
                    let target_function = catalog.function(&target)?;
                    validate_arity(target_function, resolved_arguments.len())?;
                    if state.active.iter().any(|active| active == &target) {
                        let mut function_trace = state.active.clone();
                        function_trace.push(target.clone());
                        state.pattern_references.push(PatternReference {
                            from_function: function.name.clone(),
                            to_function: target,
                            arguments: resolved_arguments.clone(),
                            source,
                            function_trace,
                        });
                    } else {
                        trace_function(
                            target_function,
                            &resolved_arguments,
                            catalog,
                            state,
                            depth + 1,
                        )?;
                    }
                }
            }
        }
    }
    let removed = state.active.pop();
    debug_assert_eq!(removed.as_deref(), Some(function.name.as_str()));
    Ok(())
}

fn bind(function: &Function, arguments: &[String]) -> BTreeMap<String, Vec<String>> {
    if function.variadic {
        return BTreeMap::from([(function.parameters[0].clone(), arguments.to_vec())]);
    }
    function
        .parameters
        .iter()
        .cloned()
        .zip(arguments.iter().cloned().map(|argument| vec![argument]))
        .collect()
}

fn validate_arity(function: &Function, actual: usize) -> Result<(), DeriveError> {
    if function.variadic {
        if actual == 0 {
            return Err(DeriveError::invalid_arguments(
                &function.name,
                "at least 1",
                actual,
            ));
        }
    } else if actual != function.parameters.len() {
        return Err(DeriveError::invalid_arguments(
            &function.name,
            function.parameters.len().to_string(),
            actual,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn shipped_source_expands_every_function() {
        let library = shipped_library().unwrap();
        let samples = library
            .functions
            .iter()
            .filter(|function| !function.variadic && function.parameters.is_empty())
            .map(|function| function.name.clone())
            .take(1)
            .collect::<Vec<_>>();
        assert_eq!(samples.len(), 1);
        for function in &library.functions {
            let arguments = if function.variadic {
                vec![samples[0].clone()]
            } else {
                function
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(index, _)| samples[index.min(samples.len() - 1)].clone())
                    .collect()
            };
            derive(&function.name, &arguments)
                .unwrap_or_else(|error| panic!("{}: {}", function.name, error.summary()));
        }
    }

    #[test]
    fn source_names_have_no_rust_privileges() {
        let source = r"
let custom = () =>
ORIENT(2);
ADD();
";
        let report = derive_from_source(source, "custom.ns", "custom", &[]).unwrap();
        assert_eq!(report.used_operations, ["ADD", "ORIENT"]);
        assert_eq!(report.primitive_steps.len(), 2);
    }

    #[test]
    fn self_reference_closes_a_finite_pattern() {
        let source = r"
let first = () =>
ORIENT(1)
second();
let second = () =>
INDEX()
first();
";
        let report = derive_from_source(source, "pattern.ns", "first", &[]).unwrap();
        assert_eq!(report.primitive_steps.len(), 2);
        assert_eq!(report.pattern_references.len(), 1);
        assert_eq!(
            report.pattern_references[0].function_trace,
            ["first", "second", "first"]
        );
    }

    #[test]
    fn empty_self_reference_is_still_a_finite_pattern_description() {
        let source = r"
let still = () =>
still()
";
        let report = derive_from_source(source, "still.ns", "still", &[]).unwrap();
        assert!(report.primitive_steps.is_empty());
        assert_eq!(report.pattern_references.len(), 1);
        assert_eq!(
            report.pattern_references[0].function_trace,
            ["still", "still"]
        );
    }

    #[test]
    fn operations_after_a_self_reference_remain_in_the_finite_graph() {
        let source = r"
let reflected = () =>
reflected()
ORIENT(2)
";
        let report = derive_from_source(source, "reflected.ns", "reflected", &[]).unwrap();
        assert_eq!(report.primitive_steps.len(), 1);
        assert_eq!(report.primitive_steps[0].name, "ORIENT");
        assert_eq!(report.pattern_references.len(), 1);
    }

    #[test]
    fn self_reference_retains_next_observation_arguments() {
        let source = r"
let carry = (next) =>
ADD()
carry(next)
";
        let report = derive_from_source(
            source,
            "carry.ns",
            "carry",
            &["next-observation".to_owned()],
        )
        .unwrap();
        assert_eq!(report.pattern_references[0].arguments, ["next-observation"]);
    }

    #[test]
    fn imported_steps_keep_their_original_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/math-functions.ns");
        let library = crate::derivation::load(path).unwrap();
        let report = derive_from_library(&library, "centered_re_perspective", &[]).unwrap();
        assert!(report.primitive_steps.iter().any(|step| {
            step.source
                .file
                .replace('\\', "/")
                .ends_with("examples/math-functions.ns")
        }));
        assert!(report.primitive_steps.iter().any(|step| {
            step.source
                .file
                .replace('\\', "/")
                .ends_with("language/functions.ns")
        }));
    }
}
