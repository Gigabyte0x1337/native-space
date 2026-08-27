// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! CPU-only behavior for the optional GPU backend.

use crate::batch::DataPoint;
use crate::core::{Diagnostic, LanguageError, NativeState, Program};

/// Exact results and the physical adapter that executed them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GpuBatchResult {
    pub results: Vec<NativeState>,
    pub adapter_name: String,
}

/// Report that this binary was built without GPU support.
///
/// This function keeps the library and CLI APIs stable across feature sets.
/// It never executes the input on CPU as an implicit fallback.
///
/// # Errors
///
/// Always returns `NSG001` because the `gpu` Cargo feature is disabled.
pub async fn execute(
    program: &Program,
    _function_name: &str,
    _inputs: &[DataPoint],
    _steps: u64,
) -> Result<GpuBatchResult, LanguageError> {
    Err(LanguageError(Diagnostic {
        code: "NSG001".into(),
        message: "GPU support is not compiled; rebuild with --features gpu".into(),
        source_name: program.source_name.clone(),
        span: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::parse;

    #[tokio::test]
    async fn disabled_backend_names_the_required_feature() {
        let program = parse("let step = (value) => value\noutput 0", "gpu.ns").unwrap();

        let error = execute(&program, "step", &[], 1).await.unwrap_err();

        assert_eq!(error.0.code, "NSG001");
        assert!(error.0.message.contains("--features gpu"));
    }
}
