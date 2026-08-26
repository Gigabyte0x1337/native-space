// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Contains serializable operation-expansion types.

use serde::Serialize;

/// Names the four Native Space algebra operations.
pub const CORE_OPERATIONS: [&str; 4] = ["ADD", "MULTIPLY", "ORIENT", "INDEX"];

/// Identifies one expansion-step category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub enum StepKind {
    /// One of the four algebra operations.
    CoreOperation,
    /// A transparent function expansion used only for navigation.
    FunctionExpansion,
}

/// Locates an operation in its Native Space source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLocation {
    /// Repository-relative source path when compiled from this repository.
    pub file: String,
    /// One-based source line.
    pub line: u32,
}

impl SourceLocation {
    /// Captures a Native Space source span.
    #[must_use]
    pub(crate) fn from_span(file: &str, span: crate::core::Span) -> Self {
        Self {
            file: file.replace('\\', "/"),
            line: u32::try_from(span.start_line).unwrap_or(u32::MAX),
        }
    }
}

/// Contains one fully expanded primitive operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrimitiveStep {
    /// One-based position after all function calls are erased.
    pub number: usize,
    /// Step category.
    pub kind: StepKind,
    /// Core operation name.
    pub name: String,
    /// Explicit primitive arguments, such as an orientation turn.
    pub arguments: Vec<String>,
    /// Exact definition source location.
    pub source: SourceLocation,
    /// Nested function path that emitted this step.
    pub function_trace: Vec<String>,
}

/// Contains one diagnostic expansion event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TraceStep {
    /// Function active when this event was emitted.
    pub function: String,
    /// Zero-based nesting depth.
    pub depth: usize,
    /// Event category.
    pub kind: StepKind,
    /// Operation or expanded function name.
    pub name: String,
    /// Explicit primitive arguments.
    pub arguments: Vec<String>,
    /// Exact definition source location.
    pub source: SourceLocation,
    /// Nested function path active at this event.
    pub function_trace: Vec<String>,
}

/// Records one source reference that closes a finite pattern graph.
///
/// This is graph structure, not a fifth Native Space operation. The reference
/// points to a function already present on `function_trace`, so derivation can
/// preserve self-modeling source without attempting an unbounded expansion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PatternReference {
    /// Function containing the reference.
    pub from_function: String,
    /// Previously entered function selected by the reference.
    pub to_function: String,
    /// Arguments carried into the next observation of the pattern.
    pub arguments: Vec<String>,
    /// Exact source location of the closing reference.
    pub source: SourceLocation,
    /// Active path with the repeated target appended at the closing position.
    pub function_trace: Vec<String>,
}

/// Contains one fully expanded function derivation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivationReport {
    /// Stable report schema name.
    pub schema: &'static str,
    /// Stable report schema version.
    pub version: u32,
    /// Requested function name.
    pub function: String,
    /// Dynamic function arguments supplied to the function.
    pub applied_arguments: Vec<String>,
    /// Fixed language operation list.
    pub language_operations: [&'static str; 4],
    /// Operations that occur at least once in this expansion.
    pub used_operations: Vec<String>,
    /// Authoritative primitive listing in finite source-graph traversal order.
    pub primitive_steps: Vec<PrimitiveStep>,
    /// Self-references that make this finite source graph a pattern.
    pub pattern_references: Vec<PatternReference>,
    /// Full function expansion trace for navigation only.
    pub function_trace: Vec<TraceStep>,
}
