// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Derives Native Space functions into explicit primitive operations.
//!
//! The crate keeps the language kernel fixed at `ADD`, `MULTIPLY`, `ORIENT`,
//! and `INDEX`. Named functions are recursively erased until an active function
//! is referenced again. That reference closes a finite self-modeling pattern
//! graph instead of creating an error or materializing unbounded output. Every
//! emitted operation retains its source location and nested function trace.

mod error;
mod model;
mod report;
mod trace;

#[doc(inline)]
pub use error::DeriveError;
#[doc(inline)]
pub use model::{
    DerivationReport, PatternReference, PrimitiveStep, SourceLocation, StepKind, TraceStep,
};
#[doc(inline)]
pub use report::{format_report, relativize_paths};
#[doc(inline)]
pub use trace::{derive, derive_from_library};

/// Lists all functions defined in the shipped Native Space source library.
///
/// # Errors
///
/// Returns a source diagnostic if the shipped library is invalid.
pub fn function_names() -> Result<Vec<String>, DeriveError> {
    trace::function_names()
}
