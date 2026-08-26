// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Native Space 1.0 language implementation.
//!
//! One `.ns` source format contains finite exact programs, source-defined
//! functions, and finite Boolean proofs. Mathematical names and theorem IDs
//! are source data; Rust assigns them no privileged semantics.

pub mod bytecode;
pub mod core;
pub mod derivation;
pub mod expansion;
pub mod logic;

pub const LANGUAGE_VERSION: &str = "1.0";

use serde_json::{Value, json};
use std::fs;
use std::path::Path;

use crate::core::{LanguageError, TokenKind};

/// One parsed Native Space 1.0 document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Document {
    State(core::Program),
    Functions(derivation::Library),
    Logic(logic::Program),
}

/// Parse any Native Space 1.0 source document.
///
/// Function declarations and values begin with `let`, result programs use
/// `output`, zero proofs end in equality, and Boolean documents use `prove` or
/// `parameter ...: bool`.
///
/// # Errors
///
/// Returns a located diagnostic for invalid syntax, literals, or parameter types.
pub fn parse_document(source: &str, source_name: &str) -> Result<Document, LanguageError> {
    let tokens = core::lex(source, source_name, "NS")?;
    let first = &tokens[0];
    if first.kind == TokenKind::Ident && first.text == "import" {
        return derivation::parse(source, source_name).map(Document::Functions);
    }
    if first.kind == TokenKind::Ident
        && first.text == "let"
        && tokens.iter().any(|token| token.kind == TokenKind::Arrow)
    {
        return match core::parse(source, source_name) {
            Ok(program) => Ok(Document::State(program)),
            Err(core_error) => derivation::parse(source, source_name)
                .map(Document::Functions)
                .map_err(|_function_error| core_error),
        };
    }
    if first.kind == TokenKind::Ident && first.text == "prove" {
        return logic::parse(source, source_name).map(Document::Logic);
    }
    if first.kind == TokenKind::Ident && first.text == "parameter" {
        let parameter_type = tokens
            .windows(2)
            .find(|pair| pair[0].kind == TokenKind::Colon)
            .and_then(|pair| (pair[1].kind == TokenKind::Ident).then_some(pair[1].text.as_str()));
        return match parameter_type {
            Some("bool") => logic::parse(source, source_name).map(Document::Logic),
            _ => Err(LanguageError(core::Diagnostic {
                code: "NST009".into(),
                message: "only Boolean proof parameters use typed declarations".into(),
                source_name: source_name.into(),
                span: Some(first.span),
            })),
        };
    }
    core::parse(source, source_name).map(Document::State)
}

/// Load one Native Space document from disk and resolve relative imports.
///
/// Imports are supported by source-defined function libraries. Exact-state and
/// Boolean documents remain self-contained in Language 1.0.
///
/// # Errors
///
/// Returns a located file, import, syntax, or structural diagnostic.
pub fn load_document(path: impl AsRef<Path>) -> Result<Document, LanguageError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(|error| {
        LanguageError(core::Diagnostic {
            code: "NST010".into(),
            message: format!("could not read {}: {error}", path.display()),
            source_name: path.display().to_string(),
            span: None,
        })
    })?;
    let source_name = path.display().to_string();
    match parse_document(&source, &source_name)? {
        Document::Functions(library) if !library.imports.is_empty() => {
            derivation::load(path).map(Document::Functions)
        }
        Document::Functions(library) => {
            derivation::validate(&library)?;
            Ok(Document::Functions(library))
        }
        document => Ok(document),
    }
}

/// Return one schema-1 JSON representation for inspection.
#[must_use]
pub fn inspect(document: &Document) -> Value {
    match document {
        Document::State(program) => core::program_to_data(program),
        Document::Functions(library) => {
            json!({"schema":"native-space-function-library","version":1,"language_version":LANGUAGE_VERSION,"library":library})
        }
        Document::Logic(program) => {
            json!({"schema":"native-space-logic-ast","version":1,"language_version":LANGUAGE_VERSION,"program":program})
        }
    }
}

/// Compile a document to bytecode, a function-library artifact, or a Boolean
/// proof certificate.
///
/// # Errors
///
/// Returns the first semantic, type, or proof diagnostic.
pub fn compile(document: &Document) -> Result<Value, LanguageError> {
    match document {
        Document::State(program) => bytecode::compile(program).map(|artifact| artifact.to_data()),
        Document::Functions(library) if library.imports.is_empty() => {
            derivation::validate(library)?;
            Ok(
                json!({"schema":"native-space-function-library","version":1,"language_version":LANGUAGE_VERSION,"library":library}),
            )
        }
        Document::Functions(library) => Err(LanguageError(core::Diagnostic {
            code: "NST011".into(),
            message: "function-library imports must be resolved with load_document".into(),
            source_name: library.source_name.clone(),
            span: library.imports.first().map(|import| import.span),
        })),
        Document::Logic(program) => logic::compile(program),
    }
}

#[cfg(test)]
mod unified_tests {
    use super::*;

    #[test]
    fn one_parser_recognizes_all_language_constructs() {
        assert!(matches!(
            parse_document("output 1", "state.ns"),
            Ok(Document::State(_))
        ));
        assert!(matches!(
            parse_document("let alignment = () =>\nORIENT(2)\nADD()", "functions.ns"),
            Ok(Document::Functions(_))
        ));
        assert!(matches!(
            parse_document(
                "parameter a: bool; prove implies(a, a) by truth_table;",
                "logic.ns"
            ),
            Ok(Document::Logic(_))
        ));
    }
}
