// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Source-defined Native Space functions.
//!
//! This module knows only generic function structure, the four core operations,
//! and calls. It contains no mathematical function names, theorem IDs, or
//! domain-specific claim schemas.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{Diagnostic, LanguageError, Span, Token, TokenKind, language_name, lex};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Operation {
    Add,
    Multiply,
    Orient,
    Index,
}

impl Operation {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Add => "ADD",
            Self::Multiply => "MULTIPLY",
            Self::Orient => "ORIENT",
            Self::Index => "INDEX",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Instruction {
    Operation {
        operation: Operation,
        arguments: Vec<String>,
        span: Span,
    },
    Call {
        function: String,
        arguments: Vec<String>,
        span: Span,
    },
}

impl Instruction {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Operation { span, .. } | Self::Call { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub variadic: bool,
    pub body: Vec<Instruction>,
    pub span: Span,
    pub source_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Library {
    pub imports: Vec<Import>,
    pub functions: Vec<Function>,
    pub source_name: String,
}

/// Validate every call in an import-resolved function library.
///
/// Calls through a function parameter are dynamic pattern edges and therefore
/// remain valid without a statically known target. All named calls must resolve
/// to a declared function, and every statically knowable arity must match.
///
/// # Errors
///
/// Returns the first located unknown-call, invalid-arity, or invalid dynamic
/// call diagnostic.
pub fn validate(library: &Library) -> Result<(), LanguageError> {
    let catalog = library
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();

    for function in &library.functions {
        let parameters = function
            .parameters
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let variadic_parameter = function.variadic.then(|| function.parameters[0].as_str());

        for instruction in &function.body {
            let Instruction::Call {
                function: called,
                arguments,
                span,
            } = instruction
            else {
                continue;
            };

            if parameters.contains(called.as_str()) {
                if !arguments.is_empty() {
                    return Err(import_error(
                        "NSF-S009",
                        format!("dynamic function parameter {called:?} cannot receive arguments"),
                        &function.source_name,
                        Some(*span),
                    ));
                }
                continue;
            }

            let target = catalog.get(called.as_str()).ok_or_else(|| {
                import_error(
                    "NSF-S007",
                    format!("unknown function {called:?}"),
                    &function.source_name,
                    Some(*span),
                )
            })?;
            let expands_variadic_argument = variadic_parameter
                .is_some_and(|parameter| arguments.iter().any(|argument| argument == parameter));

            if target.variadic {
                if arguments.is_empty() {
                    return Err(import_error(
                        "NSF-S008",
                        format!("function {called:?} expects at least 1 argument, found 0"),
                        &function.source_name,
                        Some(*span),
                    ));
                }
                continue;
            }
            if expands_variadic_argument {
                return Err(import_error(
                    "NSF-S008",
                    format!(
                        "function {called:?} expects {} arguments, but a variable number of arguments are supplied",
                        target.parameters.len()
                    ),
                    &function.source_name,
                    Some(*span),
                ));
            }
            if arguments.len() != target.parameters.len() {
                return Err(import_error(
                    "NSF-S008",
                    format!(
                        "function {called:?} expects {} arguments, found {}",
                        target.parameters.len(),
                        arguments.len()
                    ),
                    &function.source_name,
                    Some(*span),
                ));
            }
        }
    }
    Ok(())
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    source_name: String,
}

impl Parser {
    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if token.kind != TokenKind::Eof {
            self.position += 1;
        }
        token
    }

    fn error(&self, code: &str, message: impl Into<String>, span: Span) -> LanguageError {
        LanguageError(Diagnostic {
            code: code.into(),
            message: message.into(),
            source_name: self.source_name.clone(),
            span: Some(span),
        })
    }

    fn expect(
        &mut self,
        kind: TokenKind,
        code: &str,
        message: &str,
    ) -> Result<Token, LanguageError> {
        if self.current().kind != kind {
            return Err(self.error(code, message, self.current().span));
        }
        Ok(self.advance())
    }

    fn keyword(&self, text: &str) -> bool {
        self.current().kind == TokenKind::Ident && self.current().text == text
    }

    fn keyword_token(
        &mut self,
        text: &str,
        code: &str,
        message: &str,
    ) -> Result<Token, LanguageError> {
        if !self.keyword(text) {
            return Err(self.error(code, message, self.current().span));
        }
        Ok(self.advance())
    }

    fn optional_semicolon(&mut self) {
        if self.current().kind == TokenKind::Semicolon {
            self.advance();
        }
    }

    fn names(&mut self) -> Result<(Vec<String>, bool), LanguageError> {
        let mut names = Vec::new();
        let mut variadic = false;
        if self.current().kind == TokenKind::RParen {
            return Ok((names, variadic));
        }
        loop {
            let name = self.expect(TokenKind::Ident, "NSF-P006", "expected a name")?;
            if language_name(&name.text).is_some() {
                return Err(self.error(
                    "NSF-S006",
                    format!("parameter name {:?} collides with the language", name.text),
                    name.span,
                ));
            }
            names.push(name.text);
            if self.current().kind == TokenKind::Ellipsis {
                self.advance();
                variadic = true;
            }
            if self.current().kind != TokenKind::Comma {
                break;
            }
            if variadic {
                return Err(self.error(
                    "NSF-S005",
                    "a variadic parameter must be last",
                    self.current().span,
                ));
            }
            self.advance();
        }
        Ok((names, variadic))
    }

    fn instruction(&mut self) -> Result<Instruction, LanguageError> {
        let start = self.expect(
            TokenKind::Ident,
            "NSF-P010",
            "expected an operation or function call",
        )?;
        self.expect(
            TokenKind::LParen,
            "NSF-P011",
            "expected '(' after instruction name",
        )?;

        let mut values = Vec::new();
        if self.current().kind != TokenKind::RParen {
            loop {
                let value = self.advance();
                if !matches!(
                    value.kind,
                    TokenKind::Ident | TokenKind::Number | TokenKind::String
                ) {
                    return Err(self.error(
                        "NSF-P021",
                        "instruction arguments must be names, numbers, or strings",
                        value.span,
                    ));
                }
                values.push(value.text);
                if self.current().kind != TokenKind::Comma {
                    break;
                }
                self.advance();
            }
        }
        let close = self.expect(
            TokenKind::RParen,
            "NSF-P022",
            "expected ')' after instruction arguments",
        )?;
        self.optional_semicolon();
        let span = start.span.join(close.span);
        let operation = match start.text.as_str() {
            "ADD" => Some(Operation::Add),
            "MULTIPLY" => Some(Operation::Multiply),
            "ORIENT" => Some(Operation::Orient),
            "INDEX" => Some(Operation::Index),
            _ => None,
        };
        if let Some(operation) = operation {
            let expected = usize::from(operation == Operation::Orient);
            if values.len() != expected {
                return Err(self.error(
                    "NSF-P024",
                    format!(
                        "{} expects {expected} arguments, found {}",
                        operation.name(),
                        values.len()
                    ),
                    start.span.join(close.span),
                ));
            }
            Ok(Instruction::Operation {
                operation,
                arguments: values,
                span,
            })
        } else {
            Ok(Instruction::Call {
                function: start.text,
                arguments: values,
                span,
            })
        }
    }

    fn function(&mut self) -> Result<Function, LanguageError> {
        let start = self.keyword_token("let", "NSF-P001", "expected 'let'")?;
        let name = self.expect(TokenKind::Ident, "NSF-P002", "expected function name")?;
        if language_name(&name.text).is_some() {
            return Err(self.error(
                "NSF-S006",
                format!("function name {:?} collides with the language", name.text),
                name.span,
            ));
        }
        self.expect(
            TokenKind::Equals,
            "NSF-P029",
            "expected '=' after function name",
        )?;
        self.expect(
            TokenKind::LParen,
            "NSF-P003",
            "expected '(' after function name",
        )?;
        let (parameters, variadic) = self.names()?;
        self.expect(
            TokenKind::RParen,
            "NSF-P004",
            "expected ')' after parameters",
        )?;
        self.expect(
            TokenKind::Arrow,
            "NSF-P030",
            "expected '=>' before function body",
        )?;
        if variadic && parameters.len() != 1 {
            return Err(self.error(
                "NSF-S001",
                "a variadic function must declare exactly one sequence parameter",
                name.span,
            ));
        }
        let mut seen = BTreeSet::new();
        if let Some(duplicate) = parameters.iter().find(|parameter| !seen.insert(*parameter)) {
            return Err(self.error(
                "NSF-S002",
                format!("duplicate parameter {duplicate:?}"),
                name.span,
            ));
        }
        let mut body = Vec::new();
        while !self.keyword("let") && self.current().kind != TokenKind::Eof {
            body.push(self.instruction()?);
        }
        let end = body.last().map_or(name.span, Instruction::span);
        Ok(Function {
            name: name.text,
            parameters,
            variadic,
            body,
            span: start.span.join(end),
            source_name: self.source_name.clone(),
        })
    }

    fn import(&mut self) -> Result<Import, LanguageError> {
        let start = self.keyword_token("import", "NSF-P031", "expected 'import'")?;
        let path = self.expect(
            TokenKind::String,
            "NSF-P032",
            "expected a quoted relative .ns path after 'import'",
        )?;
        self.optional_semicolon();
        Ok(Import {
            path: path.text,
            span: start.span.join(path.span),
        })
    }
}

/// Parse a Native Space function library.
///
/// # Errors
///
/// Returns a located syntax or structural diagnostic.
pub fn parse(source: &str, source_name: &str) -> Result<Library, LanguageError> {
    let tokens = lex(source, source_name, "NSF-")?;
    let mut parser = Parser {
        tokens,
        position: 0,
        source_name: source_name.into(),
    };
    let mut imports = Vec::new();
    while parser.keyword("import") {
        imports.push(parser.import()?);
    }
    let mut functions = Vec::new();
    let mut names = BTreeSet::new();
    while parser.current().kind != TokenKind::Eof {
        let function = parser.function()?;
        if !names.insert(function.name.clone()) {
            return Err(parser.error(
                "NSF-S003",
                format!("duplicate function {:?}", function.name),
                function.span,
            ));
        }
        functions.push(function);
    }
    if functions.is_empty() && imports.is_empty() {
        return Err(parser.error(
            "NSF-S004",
            "a function library must contain at least one function",
            parser.current().span,
        ));
    }
    Ok(Library {
        imports,
        functions,
        source_name: source_name.into(),
    })
}

fn import_error(
    code: &str,
    message: impl Into<String>,
    source_name: impl Into<String>,
    span: Option<Span>,
) -> LanguageError {
    LanguageError(Diagnostic {
        code: code.into(),
        message: message.into(),
        source_name: source_name.into(),
        span,
    })
}

fn display_path(path: &Path) -> String {
    let rendered = path.to_string_lossy();
    if let Some(unc) = rendered.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{unc}");
    }
    rendered
        .strip_prefix(r"\\?\")
        .unwrap_or(&rendered)
        .to_owned()
}

/// Load one function library and all of its relative `.ns` imports.
///
/// Imported functions are merged before local functions. Every file is loaded
/// once, import cycles are rejected, absolute imports are rejected, and a
/// duplicate function name across files is an error.
///
/// # Errors
///
/// Returns a located I/O, syntax, cycle, path, or duplicate-name diagnostic.
pub fn load(path: impl AsRef<Path>) -> Result<Library, LanguageError> {
    load_internal(path.as_ref(), None)
}

/// Loads a function library confined to one directory tree.
///
/// The root source and every recursively resolved import must remain below
/// `root` after canonicalization. This is intended for tool interfaces that
/// accept source paths from clients.
///
/// # Errors
///
/// Returns a located I/O, syntax, cycle, path, scope, or duplicate-name
/// diagnostic.
pub fn load_within(
    path: impl AsRef<Path>,
    root: impl AsRef<Path>,
) -> Result<Library, LanguageError> {
    let root = root.as_ref().canonicalize().map_err(|error| {
        import_error(
            "NSF-I005",
            format!("could not resolve source root: {error}"),
            display_path(root.as_ref()),
            None,
        )
    })?;
    load_internal(path.as_ref(), Some(&root))
}

fn load_internal(path: &Path, allowed_root: Option<&Path>) -> Result<Library, LanguageError> {
    let absolute = path.canonicalize().map_err(|error| {
        import_error(
            "NSF-I001",
            format!("could not open {}: {error}", path.display()),
            display_path(path),
            None,
        )
    })?;
    ensure_within_root(&absolute, allowed_root, &display_path(path), None)?;
    let mut active = Vec::new();
    let mut loaded = BTreeSet::new();
    let mut functions = Vec::new();
    let mut names = BTreeSet::new();
    load_recursive(
        &absolute,
        &mut active,
        &mut loaded,
        &mut functions,
        &mut names,
        allowed_root,
    )?;
    let library = Library {
        imports: Vec::new(),
        functions,
        source_name: display_path(&absolute),
    };
    validate(&library)?;
    Ok(library)
}

fn load_recursive(
    path: &Path,
    active: &mut Vec<PathBuf>,
    loaded: &mut BTreeSet<PathBuf>,
    functions: &mut Vec<Function>,
    names: &mut BTreeSet<String>,
    allowed_root: Option<&Path>,
) -> Result<(), LanguageError> {
    if let Some(cycle_start) = active.iter().position(|entry| entry == path) {
        let cycle = active[cycle_start..]
            .iter()
            .map(PathBuf::as_path)
            .chain(std::iter::once(path))
            .map(display_path)
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(import_error(
            "NSF-I002",
            format!("import cycle: {cycle}"),
            display_path(path),
            None,
        ));
    }
    if loaded.contains(path) {
        return Ok(());
    }

    let source = fs::read_to_string(path).map_err(|error| {
        import_error(
            "NSF-I001",
            format!("could not read {}: {error}", path.display()),
            display_path(path),
            None,
        )
    })?;
    let source_name = display_path(path);
    let library = parse(&source, &source_name)?;
    active.push(path.to_path_buf());

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    for import in &library.imports {
        let requested = Path::new(&import.path);
        if requested.is_absolute()
            || requested.extension().and_then(|value| value.to_str()) != Some("ns")
        {
            return Err(import_error(
                "NSF-I003",
                "imports must use a relative path ending in .ns",
                &source_name,
                Some(import.span),
            ));
        }
        let resolved = parent.join(requested).canonicalize().map_err(|error| {
            import_error(
                "NSF-I001",
                format!("could not resolve import {:?}: {error}", import.path),
                &source_name,
                Some(import.span),
            )
        })?;
        ensure_within_root(&resolved, allowed_root, &source_name, Some(import.span))?;
        load_recursive(&resolved, active, loaded, functions, names, allowed_root)?;
    }

    for function in library.functions {
        if !names.insert(function.name.clone()) {
            return Err(import_error(
                "NSF-I004",
                format!("duplicate imported function {:?}", function.name),
                &function.source_name,
                Some(function.span),
            ));
        }
        functions.push(function);
    }
    active.pop();
    loaded.insert(path.to_path_buf());
    Ok(())
}

fn ensure_within_root(
    path: &Path,
    allowed_root: Option<&Path>,
    source_name: &str,
    span: Option<Span>,
) -> Result<(), LanguageError> {
    if allowed_root.is_some_and(|root| !path.starts_with(root)) {
        return Err(import_error(
            "NSF-I005",
            "source and imports must remain inside the configured source root",
            source_name,
            span,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_names_are_ordinary_source_names() {
        let library = parse(
            r"
let anything = (value) =>
value()
",
            "functions.ns",
        )
        .unwrap();
        assert_eq!(library.functions[0].name, "anything");
    }

    #[test]
    fn only_four_operation_spellings_are_primitive() {
        let library = parse(
            r"
let four = () =>
ADD()
MULTIPLY()
ORIENT(2)
INDEX()
",
            "four.ns",
        )
        .unwrap();
        assert_eq!(library.functions[0].body.len(), 4);
    }
}
