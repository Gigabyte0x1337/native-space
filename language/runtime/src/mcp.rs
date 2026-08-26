// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Exposes operation derivation through MCP over standard input and output.

use std::ffi::OsStr;
use std::path::{Component, Path};

use native_space_language::expansion::{DerivationReport, derive, format_report, relativize_paths};
use rmcp::{
    ErrorData, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DeriveOperationsParameters {
    /// Source-defined function to expand.
    function: String,
    /// Dynamic function names passed to the selected function.
    #[serde(default)]
    arguments: Vec<String>,
    /// Relative `.ns` function-library path; omit for the generic library.
    #[serde(default)]
    source: Option<String>,
}

#[derive(Debug, Clone)]
struct OperationTools {
    tool_router: ToolRouter<Self>,
}

impl OperationTools {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl OperationTools {
    #[expect(
        clippy::unused_self,
        reason = "rmcp tool handlers require a receiver for router dispatch"
    )]
    #[tool(
        description = "Derive a Native Space function as primitive operations and finite self-references"
    )]
    fn derive_operations(
        &self,
        Parameters(parameters): Parameters<DeriveOperationsParameters>,
    ) -> Result<String, ErrorData> {
        let root = std::env::current_dir().map_err(|error| {
            ErrorData::internal_error(
                format!("could not resolve the working directory: {error}"),
                None,
            )
        })?;
        derive_report(&root, &parameters).map(|report| format_report(&report))
    }
}

fn derive_report(
    root: &Path,
    parameters: &DeriveOperationsParameters,
) -> Result<DerivationReport, ErrorData> {
    let Some(source) = parameters.source.as_deref() else {
        return derive(&parameters.function, &parameters.arguments)
            .map_err(|error| ErrorData::invalid_params(error.summary(), None));
    };

    let relative = Path::new(source);
    if relative.extension() != Some(OsStr::new("ns"))
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ErrorData::invalid_params(
            "source must be a relative .ns path inside the working directory",
            None,
        ));
    }

    let canonical_root = root.canonicalize().map_err(|error| {
        ErrorData::internal_error(
            format!("could not resolve the working directory: {error}"),
            None,
        )
    })?;
    let canonical_source = root.join(relative).canonicalize().map_err(|error| {
        ErrorData::invalid_params(
            format!("could not resolve source {source:?}: {error}"),
            None,
        )
    })?;
    if !canonical_source.starts_with(&canonical_root) {
        return Err(ErrorData::invalid_params(
            "source must remain inside the working directory",
            None,
        ));
    }

    let library =
        native_space_language::derivation::load_within(&canonical_source, &canonical_root)
            .map_err(|error| ErrorData::invalid_params(error.to_string(), None))?;
    let mut report = native_space_language::expansion::derive_from_library(
        &library,
        &parameters.function,
        &parameters.arguments,
    )
    .map_err(|error| ErrorData::invalid_params(error.summary(), None))?;
    relativize_paths(&mut report, &canonical_root);
    Ok(report)
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for OperationTools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("native-space", env!("CARGO_PKG_VERSION"))
                    .with_title("Native Space Operation Tools")
                    .with_description("Transparent Native Space function derivation"),
            )
            .with_instructions(
                "Use derive_operations to inspect core operations and finite pattern references.",
            )
    }
}

/// Runs the operation tools over MCP stdio until the client disconnects.
///
/// Standard output is reserved exclusively for MCP protocol frames.
///
/// # Errors
///
/// Returns an error when the MCP transport cannot start or terminates with a
/// protocol or I/O failure.
pub(crate) async fn run_stdio() -> Result<(), Box<dyn std::error::Error>> {
    let server = OperationTools::new().serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_tool_uses_the_human_derivation_report() {
        let result = OperationTools::new()
            .derive_operations(Parameters(DeriveOperationsParameters {
                function: "axis_subtract".to_owned(),
                arguments: vec![
                    "identity_orientation".to_owned(),
                    "identity_orientation".to_owned(),
                ],
                source: None,
            }))
            .expect("built-in derivation must succeed");

        assert!(result.starts_with("Derived: axis_subtract"));
        assert!(result.contains("Primitive operations:"));
        assert!(result.contains("ORIENT(0)"));
        assert!(result.contains("ADD()"));
    }

    #[test]
    fn mcp_tool_loads_a_scoped_mathematical_library() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = derive_report(
            &root,
            &DeriveOperationsParameters {
                function: "zeta_classical_pattern".to_owned(),
                arguments: Vec::new(),
                source: Some("examples/math-functions.ns".to_owned()),
            },
        )
        .expect("repository example must derive");

        assert_eq!(report.function, "zeta_classical_pattern");
        assert!(!report.primitive_steps.is_empty());
        assert!(
            report
                .primitive_steps
                .iter()
                .all(|step| !Path::new(&step.source.file).is_absolute())
        );
    }

    #[test]
    fn mcp_tool_rejects_sources_outside_the_working_directory() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let error = derive_report(
            &root,
            &DeriveOperationsParameters {
                function: "axis_subtract".to_owned(),
                arguments: Vec::new(),
                source: Some("../outside.ns".to_owned()),
            },
        )
        .expect_err("parent traversal must be rejected");

        assert!(error.message.contains("relative .ns path"));
    }

    #[test]
    fn scoped_loading_rejects_imports_outside_the_source_root() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
        let error = derive_report(
            &root,
            &DeriveOperationsParameters {
                function: "zeta_classical_pattern".to_owned(),
                arguments: Vec::new(),
                source: Some("math-functions.ns".to_owned()),
            },
        )
        .expect_err("the import of ../language/functions.ns must leave this narrow root");

        assert!(error.message.contains("configured source root"));
    }
}
