// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Formats operation derivations for people.

use std::fmt::Write;
use std::path::Path;

use super::DerivationReport;

/// Makes derivation source paths relative to `root` when possible.
///
/// Paths outside `root` remain unchanged. Windows verbatim prefixes and path
/// separators are normalized for portable reports.
pub fn relativize_paths(report: &mut DerivationReport, root: impl AsRef<Path>) {
    let normalized_root = normalize_path(root.as_ref().to_string_lossy().as_ref());
    let root_prefix = format!("{normalized_root}/");
    let root_prefix_lower = root_prefix.to_ascii_lowercase();
    for source in report
        .primitive_steps
        .iter_mut()
        .map(|step| &mut step.source)
        .chain(
            report
                .function_trace
                .iter_mut()
                .map(|step| &mut step.source),
        )
        .chain(
            report
                .pattern_references
                .iter_mut()
                .map(|reference| &mut reference.source),
        )
    {
        let normalized_file = normalize_path(&source.file);
        if normalized_file
            .to_ascii_lowercase()
            .starts_with(&root_prefix_lower)
        {
            normalized_file[root_prefix.len()..].clone_into(&mut source.file);
        }
    }
}

fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    normalized
        .strip_prefix("//?/")
        .unwrap_or(&normalized)
        .trim_end_matches('/')
        .to_owned()
}

/// Formats a concise, user-facing operation report.
#[must_use]
pub fn format_report(report: &DerivationReport) -> String {
    let mut output = String::new();
    if report.pattern_references.is_empty() {
        let _ = writeln!(output, "Derived: {}", report.function);
        let _ = writeln!(
            output,
            "Primitive operations: {}",
            report.primitive_steps.len()
        );
    } else {
        let _ = writeln!(output, "Derived pattern: {}", report.function);
        let _ = writeln!(
            output,
            "Primitive operations in its finite source: {}",
            report.primitive_steps.len()
        );
    }

    for step in &report.primitive_steps {
        let arguments = if step.arguments.is_empty() {
            String::new()
        } else {
            step.arguments.join(", ")
        };
        let _ = writeln!(
            output,
            "{}. {}({}) — {}:{}",
            step.number, step.name, arguments, step.source.file, step.source.line
        );
    }

    for reference in &report.pattern_references {
        let _ = writeln!(
            output,
            "Self-reference: {} — {}:{}",
            reference.function_trace.join(" -> "),
            reference.source.file,
            reference.source.line
        );
    }

    output.pop();
    output
}
