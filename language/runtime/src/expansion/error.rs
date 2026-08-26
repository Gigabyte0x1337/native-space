// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Defines recoverable derivation errors.

use std::{backtrace::Backtrace, error::Error, fmt};

#[derive(Debug)]
pub(crate) enum DeriveErrorKind {
    InvalidSource(String),
    UnknownFunction(String),
    InvalidArguments {
        function: String,
        expected: String,
        actual: usize,
    },
}

/// Reports invalid derivation input.
#[derive(Debug)]
pub struct DeriveError {
    kind: DeriveErrorKind,
    backtrace: Backtrace,
}

impl DeriveError {
    pub(crate) fn invalid_source(message: impl Into<String>) -> Self {
        Self {
            kind: DeriveErrorKind::InvalidSource(message.into()),
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn unknown_function(name: impl Into<String>) -> Self {
        Self {
            kind: DeriveErrorKind::UnknownFunction(name.into()),
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn invalid_arguments(
        function: impl Into<String>,
        expected: impl Into<String>,
        actual: usize,
    ) -> Self {
        Self {
            kind: DeriveErrorKind::InvalidArguments {
                function: function.into(),
                expected: expected.into(),
                actual,
            },
            backtrace: Backtrace::capture(),
        }
    }

    /// Returns a concise message suitable for users.
    #[must_use]
    pub fn summary(&self) -> String {
        match &self.kind {
            DeriveErrorKind::InvalidSource(message) => message.clone(),
            DeriveErrorKind::UnknownFunction(name) => {
                format!("unknown function '{name}'")
            }
            DeriveErrorKind::InvalidArguments {
                function,
                expected,
                actual,
            } => format!("function '{function}' expects {expected}; received {actual}"),
        }
    }

    /// Returns the captured diagnostic backtrace.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for DeriveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.summary(), self.backtrace)
    }
}

impl Error for DeriveError {}
