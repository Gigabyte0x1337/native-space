// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Optional exact GPU batch execution.
//!
//! The public API remains available in CPU-only builds. Calling [`execute`]
//! without the `gpu` Cargo feature returns a diagnostic that names the feature
//! required to enable the backend. It never falls back to CPU implicitly.

#[cfg(not(feature = "gpu"))]
mod disabled;
#[cfg(feature = "gpu")]
mod enabled;

#[cfg(not(feature = "gpu"))]
pub use disabled::{GpuBatchResult, execute};
#[cfg(feature = "gpu")]
pub use enabled::{GpuBatchResult, execute};
