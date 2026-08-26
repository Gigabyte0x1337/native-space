// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Finite Boolean fragment and exhaustive tautology certificates.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::LANGUAGE_VERSION;
use crate::core::{Diagnostic, LanguageError, Span, Token, TokenKind, language_name, lex};

pub const MAX_VARIABLES: usize = 16;
pub const CERTIFICATE_VERSION: u64 = 1;
pub const TRUST_BOUNDARY: &str = "The verifier decides finite propositional tautologies by exhaustive Boolean valuation. It does not implement quantifiers, induction, native-state equality, or analytic proof rules.";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub span: Option<Span>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expr {
    Reference {
        name: String,
        span: Option<Span>,
    },
    Call {
        function: String,
        arguments: Vec<Expr>,
        span: Option<Span>,
    },
}
impl Expr {
    #[must_use]
    pub const fn span(&self) -> Option<Span> {
        match self {
            Self::Reference { span, .. } | Self::Call { span, .. } => *span,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub parameters: Vec<Parameter>,
    pub proposition: Expr,
    pub method: String,
    pub source_name: String,
    pub span: Option<Span>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Verification {
    pub valid: bool,
    pub proof_status: String,
    pub variables: Vec<String>,
    pub valuation_count: u64,
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    source: String,
}
impl Parser {
    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }
    fn advance(&mut self) -> Token {
        let t = self.current().clone();
        if t.kind != TokenKind::Eof {
            self.position += 1;
        }
        t
    }
    fn error(&self, code: &str, message: impl Into<String>, span: Span) -> LanguageError {
        LanguageError(Diagnostic {
            code: code.into(),
            message: message.into(),
            source_name: self.source.clone(),
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
    fn expr(&mut self) -> Result<Expr, LanguageError> {
        let start = self.expect(TokenKind::Ident, "NSB-P011", "expected Boolean expression")?;
        if self.current().kind != TokenKind::LParen {
            return Ok(Expr::Reference {
                name: start.text,
                span: Some(start.span),
            });
        }
        self.advance();
        let mut arguments = Vec::new();
        if self.current().kind != TokenKind::RParen {
            arguments.push(self.expr()?);
            while self.current().kind == TokenKind::Comma {
                self.advance();
                arguments.push(self.expr()?);
            }
        }
        let end = self.expect(
            TokenKind::RParen,
            "NSB-P012",
            "expected ')' after Boolean function",
        )?;
        Ok(Expr::Call {
            function: start.text,
            arguments,
            span: Some(start.span.join(end.span)),
        })
    }
}

/// Parse a finite Boolean proof in the single Native Space 1.0 format.
///
/// # Errors
///
/// Returns a located diagnostic for invalid syntax or proof method.
pub fn parse(source: &str, source_name: &str) -> Result<Program, LanguageError> {
    let tokens = lex(source, source_name, "NSB-")?;
    let mut p = Parser {
        tokens,
        position: 0,
        source: source_name.into(),
    };
    let start = p.current().span;
    let mut parameters = Vec::new();
    while p.keyword("parameter") {
        let begin = p.advance().span;
        let name = p.expect(TokenKind::Ident, "NSB-P007", "expected parameter name")?;
        if reserved(&name.text) {
            return Err(p.error(
                "NSS008",
                "parameter name collides with the language",
                name.span,
            ));
        }
        p.expect(TokenKind::Colon, "NSB-P008", "expected ':' after parameter")?;
        p.keyword_token("bool", "NSB-P009", "expected bool parameter type")?;
        p.optional_semicolon();
        parameters.push(Parameter {
            name: name.text,
            span: Some(begin.join(p.tokens[p.position.saturating_sub(1)].span)),
        });
    }
    p.keyword_token("prove", "NSB-P001", "expected 'prove'")?;
    let proposition = p.expr()?;
    p.keyword_token("by", "NSB-P002", "expected 'by' after proposition")?;
    let method = p.expect(TokenKind::Ident, "NSB-P003", "expected proof method")?;
    if method.text != "truth_table" {
        return Err(p.error("NSB-P004", "expected truth_table proof method", method.span));
    }
    p.optional_semicolon();
    let end = p.tokens[p.position.saturating_sub(1)].span;
    p.expect(TokenKind::Eof, "NSB-P006", "unexpected content after proof")?;
    Ok(Program {
        parameters,
        proposition,
        method: method.text,
        source_name: source_name.into(),
        span: Some(start.join(end)),
    })
}
fn reserved(name: &str) -> bool {
    language_name(name).is_some()
}
fn error(
    program: &Program,
    code: &str,
    message: impl Into<String>,
    span: Option<Span>,
) -> LanguageError {
    LanguageError(Diagnostic {
        code: code.into(),
        message: message.into(),
        source_name: program.source_name.clone(),
        span,
    })
}
fn arity(function: &str) -> Option<usize> {
    match function {
        "not" => Some(1),
        "and" | "or" | "xor" | "implies" | "iff" => Some(2),
        _ => None,
    }
}
fn validate_expr(
    expr: &Expr,
    variables: &BTreeSet<String>,
    program: &Program,
) -> Result<(), LanguageError> {
    match expr {
        Expr::Reference { name, span } => {
            if name != "true" && name != "false" && !variables.contains(name) {
                Err(error(
                    program,
                    "NSB-S004",
                    format!("unknown Boolean name {name:?}"),
                    *span,
                ))
            } else {
                Ok(())
            }
        }
        Expr::Call {
            function,
            arguments,
            span,
        } => {
            let Some(expected) = arity(function) else {
                return Err(error(
                    program,
                    "NSB-S006",
                    format!("unknown Boolean function {function:?}"),
                    *span,
                ));
            };
            if arguments.len() != expected {
                return Err(error(
                    program,
                    "NSB-S007",
                    format!(
                        "{function} expects {expected} arguments, found {}",
                        arguments.len()
                    ),
                    *span,
                ));
            }
            for argument in arguments {
                validate_expr(argument, variables, program)?;
            }
            Ok(())
        }
    }
}
fn variables(program: &Program) -> Result<Vec<String>, LanguageError> {
    if program.method != "truth_table" {
        return Err(error(
            program,
            "NSB-S001",
            "unsupported proof method",
            program.span,
        ));
    }
    let mut seen = BTreeSet::new();
    let mut variables = Vec::new();
    for parameter in &program.parameters {
        if reserved(&parameter.name) {
            return Err(error(
                program,
                "NSB-S008",
                "invalid Boolean parameter",
                parameter.span,
            ));
        }
        if !seen.insert(parameter.name.clone()) {
            return Err(error(
                program,
                "NSB-S002",
                format!("duplicate Boolean parameter {:?}", parameter.name),
                parameter.span,
            ));
        }
        variables.push(parameter.name.clone());
    }
    if variables.len() > MAX_VARIABLES {
        return Err(error(
            program,
            "NSB-S003",
            format!(
                "truth_table supports at most {MAX_VARIABLES} variables; found {}",
                variables.len()
            ),
            program.span,
        ));
    }
    validate_expr(&program.proposition, &seen, program)?;
    Ok(variables)
}
fn evaluate(expr: &Expr, values: &BTreeMap<String, bool>) -> Result<bool, String> {
    match expr {
        Expr::Reference { name, .. } => match name.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => values
                .get(name)
                .copied()
                .ok_or_else(|| format!("missing Boolean value for {name:?}")),
        },
        Expr::Call {
            function,
            arguments,
            ..
        } => {
            let values = arguments
                .iter()
                .map(|x| evaluate(x, values))
                .collect::<Result<Vec<_>, _>>()?;
            match function.as_str() {
                "not" => Ok(!values[0]),
                "and" => Ok(values[0] && values[1]),
                "or" => Ok(values[0] || values[1]),
                "xor" => Ok(values[0] != values[1]),
                "implies" => Ok(!values[0] || values[1]),
                "iff" => Ok(values[0] == values[1]),
                _ => Err(format!("unknown Boolean function {function:?}")),
            }
        }
    }
}
fn checked(program: &Program) -> Result<(Vec<String>, u64), LanguageError> {
    let variables = variables(program)?;
    let count = 1_u64 << variables.len();
    for mask in 0..count {
        let valuation: BTreeMap<_, _> = variables
            .iter()
            .enumerate()
            .map(|(index, name)| (name.clone(), mask & (1_u64 << index) != 0))
            .collect();
        if !evaluate(&program.proposition, &valuation)
            .map_err(|message| error(program, "NSB-S009", message, program.proposition.span()))?
        {
            let assignment = variables
                .iter()
                .map(|name| format!("{name}={}", valuation[name]))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(error(
                program,
                "NSB-S005",
                format!("proposition is not a tautology; counterexample: {assignment}"),
                program.proposition.span(),
            ));
        }
    }
    Ok((variables, count))
}
fn certificate_data(program: &Program, variables: &[String], count: u64) -> Value {
    json!({"schema":"native-space-logic-certificate","version":CERTIFICATE_VERSION,"language_version":LANGUAGE_VERSION,"proof_status":"proved-tautology","trust_boundary":TRUST_BOUNDARY,"variables":variables,"valuation_count":count,"program":program})
}
/// Exhaustively prove a finite propositional tautology and emit its certificate.
///
/// # Errors
///
/// Returns a diagnostic for an invalid formula or the first counterexample.
pub fn compile(program: &Program) -> Result<Value, LanguageError> {
    let (variables, count) = checked(program)?;
    Ok(certificate_data(program, &variables, count))
}
/// Recompute and verify a finite Boolean certificate.
///
/// # Errors
///
/// Returns an error for unsupported schemas, invalid proofs, or altered metadata.
pub fn verify(value: &Value) -> Result<Verification, String> {
    let root = value
        .as_object()
        .ok_or("logic certificate root must be an object")?;
    if root.get("schema").and_then(Value::as_str) != Some("native-space-logic-certificate")
        || root.get("version").and_then(Value::as_u64) != Some(CERTIFICATE_VERSION)
        || root.get("language_version").and_then(Value::as_str) != Some(LANGUAGE_VERSION)
    {
        return Err("unsupported Native Space logic certificate".into());
    }
    let program: Program = serde_json::from_value(
        root.get("program")
            .cloned()
            .ok_or("logic certificate program is missing")?,
    )
    .map_err(|e| e.to_string())?;
    let (variables, count) = checked(&program).map_err(|e| e.to_string())?;
    if *value != certificate_data(&program, &variables, count) {
        return Err("logic certificate metadata does not match verified program".into());
    }
    Ok(Verification {
        valid: true,
        proof_status: "proved-tautology".into(),
        variables,
        valuation_count: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tautology_compiles_and_tampering_fails() {
        let program=parse("parameter a: bool; parameter b: bool; prove iff(not(and(a, b)), or(not(a), not(b))) by truth_table;","logic.ns").unwrap();
        let data = compile(&program).unwrap();
        assert_eq!(verify(&data).unwrap().valuation_count, 4);
        let mut changed = data;
        changed["valuation_count"] = json!(8);
        verify(&changed).unwrap_err();
    }
    #[test]
    fn counterexample_is_exact() {
        let program = parse("parameter a: bool; prove a by truth_table;", "false.ns").unwrap();
        let error = compile(&program).unwrap_err();
        assert_eq!(error.0.code, "NSB-S005");
        assert!(error.to_string().contains("a=false"));
    }
}
