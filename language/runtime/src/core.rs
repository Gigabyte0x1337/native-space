// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Exact finite Native Space values, syntax, parser, evaluator, and optimizer.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display};
use std::str::FromStr;

use num_bigint::{BigInt, BigUint};
use num_rational::BigRational;
use num_traits::{One as _, ToPrimitive as _, Zero as _};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const FLAT_STACK_CAMERA: &str = "flat-stack-v1";
pub const AST_SCHEMA_VERSION: u64 = 1;
pub const UTF8_BYTE_DIRECTION_MAX: u64 = 256;
pub const UTF8_POSITION_DIRECTION_START: u64 = UTF8_BYTE_DIRECTION_MAX + 1;
const MAX_CANONICAL_ORIENTATION: i64 = 3;

pub(crate) fn is_canonical_orientation(turns: i64) -> bool {
    (0..=MAX_CANONICAL_ORIENTATION).contains(&turns)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Span {
    #[must_use]
    pub const fn join(self, end: Self) -> Self {
        Self {
            end_line: end.end_line,
            end_column: end.end_column,
            ..self
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub source_name: String,
    pub span: Option<Span>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageError(pub Diagnostic);

impl Display for LanguageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = self.0.span {
            write!(
                f,
                "{}:{}:{}: {}: {}",
                self.0.source_name, span.start_line, span.start_column, self.0.code, self.0.message
            )
        } else {
            write!(
                f,
                "{}: {}: {}",
                self.0.source_name, self.0.code, self.0.message
            )
        }
    }
}

impl std::error::Error for LanguageError {}

fn fail(
    code: &str,
    message: impl Into<String>,
    source_name: &str,
    span: Option<Span>,
) -> LanguageError {
    LanguageError(Diagnostic {
        code: code.into(),
        message: message.into(),
        source_name: source_name.into(),
        span,
    })
}

pub type Rational = BigRational;

/// Parse an exact integer or rational literal.
///
/// # Errors
///
/// Returns an error when either integer is invalid or the denominator is zero.
pub fn rational(text: &str) -> Result<Rational, String> {
    let (numerator, denominator) = text.split_once('/').map_or((text, "1"), |parts| parts);
    let numerator =
        BigInt::from_str(numerator).map_err(|_parse_error| format!("invalid rational {text:?}"))?;
    let denominator = BigInt::from_str(denominator)
        .map_err(|_parse_error| format!("invalid rational {text:?}"))?;
    if denominator.is_zero() {
        return Err(format!("invalid rational {text:?}"));
    }
    Ok(BigRational::new(numerator, denominator))
}

fn rational_text(value: &Rational) -> String {
    if value.denom().is_one() {
        value.numer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NativeScalar {
    pub real: Rational,
    pub imag: Rational,
}

impl NativeScalar {
    #[must_use]
    pub fn zero() -> Self {
        Self {
            real: Rational::zero(),
            imag: Rational::zero(),
        }
    }
    #[must_use]
    pub fn one() -> Self {
        Self {
            real: Rational::one(),
            imag: Rational::zero(),
        }
    }
    #[must_use]
    pub fn quarter_turn() -> Self {
        Self {
            real: Rational::zero(),
            imag: Rational::one(),
        }
    }
    /// Construct an exact scalar from two rational literals.
    ///
    /// # Errors
    ///
    /// Returns an error when either coordinate is not an exact rational.
    pub fn from_text(real: &str, imag: &str) -> Result<Self, String> {
        Ok(Self {
            real: rational(real)?,
            imag: rational(imag)?,
        })
    }
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.real.is_zero() && self.imag.is_zero()
    }
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            real: &self.real + &other.real,
            imag: &self.imag + &other.imag,
        }
    }
    #[must_use]
    pub fn negate(&self) -> Self {
        Self {
            real: -&self.real,
            imag: -&self.imag,
        }
    }
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            real: &self.real * &other.real - &self.imag * &other.imag,
            imag: &self.real * &other.imag + &self.imag * &other.real,
        }
    }
    /// Apply one canonical native orientation.
    ///
    /// The implementation derives orientation from repeated multiplication by
    /// the native quarter-turn value instead of reducing an arbitrary count.
    ///
    /// # Panics
    ///
    /// Panics when `turns` is outside the canonical range from zero through
    /// three. Text, AST, strand, and bytecode inputs reject that condition with
    /// a diagnostic before reaching this method.
    #[must_use]
    pub fn orient(&self, turns: i64) -> Self {
        assert!(
            is_canonical_orientation(turns),
            "orientation turns must be from zero through three"
        );
        let quarter_turn = Self::quarter_turn();
        let mut oriented = self.clone();
        for _ in 0..turns {
            oriented = quarter_turn.multiply(&oriented);
        }
        oriented
    }
    #[must_use]
    pub fn to_data(&self) -> Value {
        json!({"real": rational_text(&self.real), "imag": rational_text(&self.imag)})
    }
    /// Decode a scalar from its schema-1 JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error for a malformed object or coordinate.
    pub fn from_data(value: &Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or("serialized scalar must be an object")?;
        Self::from_text(
            object
                .get("real")
                .and_then(Value::as_str)
                .ok_or("serialized scalar coordinates must be strings")?,
            object
                .get("imag")
                .and_then(Value::as_str)
                .ok_or("serialized scalar coordinates must be strings")?,
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct MultiIndex(pub BTreeMap<u64, BigUint>);

impl MultiIndex {
    /// Build a canonical multi-index from direction/depth pairs.
    ///
    /// # Errors
    ///
    /// Returns an error when a direction is zero.
    pub fn from_depths(depths: impl IntoIterator<Item = (u64, u64)>) -> Result<Self, String> {
        let mut entries = BTreeMap::new();
        for (direction, depth) in depths {
            if direction == 0 {
                return Err("index directions must be positive".into());
            }
            if depth > 0 {
                *entries.entry(direction).or_default() += BigUint::from(depth);
            }
        }
        Ok(Self(entries))
    }

    /// Decode a canonical multi-index from runtime-state data.
    ///
    /// Duplicate directions are combined exactly and zero-depth entries are
    /// omitted, so the returned value is canonical.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed entries, direction zero, or a depth that
    /// is not a nonnegative decimal integer string.
    pub fn from_data(value: &Value) -> Result<Self, String> {
        let entries = value
            .as_array()
            .ok_or("serialized index must be an array")?;
        let mut result = BTreeMap::<u64, BigUint>::new();
        for entry in entries {
            let object = entry
                .as_object()
                .ok_or("serialized index entries must be objects")?;
            let direction = object
                .get("direction")
                .and_then(Value::as_u64)
                .ok_or("serialized index directions must be positive integers")?;
            if direction == 0 {
                return Err("index directions must be positive".into());
            }
            let depth_text = object
                .get("depth")
                .and_then(Value::as_str)
                .ok_or("serialized index depths must be decimal integer strings")?;
            let depth = BigUint::from_str(depth_text)
                .map_err(|_parse_error| "serialized index depth is not a nonnegative integer")?;
            if !depth.is_zero() {
                *result.entry(direction).or_default() += depth;
            }
        }
        Ok(Self(result))
    }

    #[must_use]
    pub fn compose(&self, other: &Self) -> Self {
        let mut result = self.0.clone();
        for (&direction, depth) in &other.0 {
            *result.entry(direction).or_default() += depth;
        }
        Self(result)
    }
    /// Shift one direction by an exact nonnegative depth.
    ///
    /// # Errors
    ///
    /// Returns an error when the direction is zero.
    pub fn shift_by(&self, direction: u64, depth: u64) -> Result<Self, String> {
        if direction == 0 {
            return Err("index directions must be positive".into());
        }
        let mut result = self.0.clone();
        if depth > 0 {
            *result.entry(direction).or_default() += BigUint::from(depth);
        }
        Ok(Self(result))
    }

    /// Select and optionally remap one present coordinate direction.
    ///
    /// Returns `None` when `from_direction` is absent. Destination direction
    /// zero removes the selected coordinate; any positive destination retains
    /// its exact depth under that direction.
    #[must_use]
    pub fn camera(&self, from_direction: u64, to_direction: u64) -> Option<Self> {
        let mut result = self.0.clone();
        let depth = result.remove(&from_direction)?;
        if to_direction > 0 {
            *result.entry(to_direction).or_default() += depth;
        }
        Some(Self(result))
    }
    #[must_use]
    pub fn depth(&self, direction: u64) -> BigUint {
        self.0.get(&direction).cloned().unwrap_or_default()
    }
    #[must_use]
    pub fn total_depth(&self) -> BigUint {
        self.0.values().sum()
    }
    #[must_use]
    pub fn to_data(&self) -> Value {
        Value::Array(
            self.0
                .iter()
                .map(|(direction, depth)| {
                    json!({"direction":direction,"depth":depth.to_str_radix(10)})
                })
                .collect(),
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NativeState(pub BTreeMap<MultiIndex, NativeScalar>);

impl NativeState {
    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn one() -> Self {
        Self::scalar(NativeScalar::one())
    }
    #[must_use]
    pub fn scalar(value: NativeScalar) -> Self {
        if value.is_zero() {
            Self::zero()
        } else {
            Self(BTreeMap::from([(MultiIndex::default(), value)]))
        }
    }
    pub fn from_terms(terms: impl IntoIterator<Item = (MultiIndex, NativeScalar)>) -> Self {
        let mut result = BTreeMap::new();
        for (index, value) in terms {
            let combined = result.get(&index).map_or_else(
                || value.clone(),
                |current: &NativeScalar| current.add(&value),
            );
            if combined.is_zero() {
                result.remove(&index);
            } else {
                result.insert(index, combined);
            }
        }
        Self(result)
    }
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self::from_terms(
            self.0
                .iter()
                .chain(&other.0)
                .map(|(i, c)| (i.clone(), c.clone())),
        )
    }
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        Self::from_terms(self.0.iter().flat_map(|(li, lc)| {
            other
                .0
                .iter()
                .map(move |(ri, rc)| (li.compose(ri), lc.multiply(rc)))
        }))
    }
    /// Apply one canonical orientation to every coefficient.
    ///
    /// # Panics
    ///
    /// Panics when `turns` is outside the canonical range from zero through
    /// three. All executable document boundaries validate this invariant.
    #[must_use]
    pub fn orient(&self, turns: i64) -> Self {
        assert!(
            is_canonical_orientation(turns),
            "orientation turns must be from zero through three"
        );
        Self::from_terms(self.0.iter().map(|(i, c)| (i.clone(), c.orient(turns))))
    }
    /// Apply INDEX with explicit native multiplicity.
    ///
    /// # Errors
    ///
    /// Returns an error when the direction is zero.
    pub fn index_power(&self, direction: u64, depth: u64) -> Result<Self, String> {
        Ok(Self::from_terms(
            self.0
                .iter()
                .map(|(i, c)| Ok((i.shift_by(direction, depth)?, c.clone())))
                .collect::<Result<Vec<_>, String>>()?,
        ))
    }

    /// Apply an exact coordinate-selection camera.
    ///
    /// Terms without `from_direction` are discarded. Matching terms preserve
    /// their coefficients and exact depth while moving that direction to
    /// `to_direction`. Destination zero unwraps the selected direction.
    #[must_use]
    pub fn camera(&self, from_direction: u64, to_direction: u64) -> Self {
        Self::from_terms(self.0.iter().filter_map(|(index, coefficient)| {
            index
                .camera(from_direction, to_direction)
                .map(|mapped| (mapped, coefficient.clone()))
        }))
    }
    #[must_use]
    pub fn to_data(&self) -> Value {
        json!({"camera":FLAT_STACK_CAMERA,"terms":self.0.iter().map(|(index,coefficient)|json!({"index":index.to_data(),"coefficient":coefficient.to_data()})).collect::<Vec<_>>()})
    }

    /// Decode and canonicalize a flat-stack runtime state.
    ///
    /// Duplicate terms are combined and exact zero coefficients disappear.
    ///
    /// # Errors
    ///
    /// Returns an error for a missing or unsupported camera, malformed terms,
    /// malformed indices, or malformed exact coefficients.
    pub fn from_data(value: &Value) -> Result<Self, String> {
        let root = value
            .as_object()
            .ok_or("serialized native state must be an object")?;
        if root.get("camera").and_then(Value::as_str) != Some(FLAT_STACK_CAMERA) {
            return Err("unsupported or missing native-state camera".into());
        }
        let terms = root
            .get("terms")
            .and_then(Value::as_array)
            .ok_or("serialized native-state terms must be an array")?;
        let decoded = terms
            .iter()
            .map(|term| {
                let object = term
                    .as_object()
                    .ok_or("serialized native-state terms must be objects")?;
                Ok((
                    MultiIndex::from_data(object.get("index").ok_or("term index is required")?)?,
                    NativeScalar::from_data(
                        object
                            .get("coefficient")
                            .ok_or("term coefficient is required")?,
                    )?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self::from_terms(decoded))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expr {
    Reflect {
        operation: crate::reflection::Operation,
        arguments: Vec<Expr>,
        span: Option<Span>,
    },
    Zero {
        span: Option<Span>,
    },
    One {
        span: Option<Span>,
    },
    Scalar {
        real: String,
        imag: String,
        span: Option<Span>,
    },
    Reference {
        name: String,
        span: Option<Span>,
    },
    Spread {
        name: String,
        span: Option<Span>,
    },
    Call {
        function: String,
        arguments: Vec<Expr>,
        span: Option<Span>,
    },
    Concat {
        direction: u64,
        values: Vec<Expr>,
        span: Option<Span>,
    },
    Fold {
        function: String,
        initial: Box<Expr>,
        values: Vec<Expr>,
        span: Option<Span>,
    },
    Camera {
        from_direction: u64,
        to_direction: u64,
        value: Box<Expr>,
        span: Option<Span>,
    },
    Trace {
        function: String,
        span: Option<Span>,
    },
    Length {
        value: Box<Expr>,
        span: Option<Span>,
    },
    Untrace {
        value: Box<Expr>,
        rank: String,
        span: Option<Span>,
    },
    RankDescent {
        value: Box<Expr>,
        target_rank: Option<String>,
        minimum_agreement: Option<String>,
        span: Option<Span>,
    },
    Apply {
        pattern: Box<Expr>,
        position: u64,
        span: Option<Span>,
    },
    Add {
        operands: Vec<Expr>,
        span: Option<Span>,
    },
    Multiply {
        operands: Vec<Expr>,
        span: Option<Span>,
    },
    Orient {
        turns: i64,
        value: Box<Expr>,
        span: Option<Span>,
    },
    Index {
        direction: u64,
        multiplicity: u64,
        value: Box<Expr>,
        span: Option<Span>,
    },
}

impl Expr {
    #[must_use]
    pub const fn span(&self) -> Option<Span> {
        match self {
            Self::Zero { span }
            | Self::Reflect { span, .. }
            | Self::One { span }
            | Self::Scalar { span, .. }
            | Self::Reference { span, .. }
            | Self::Spread { span, .. }
            | Self::Call { span, .. }
            | Self::Concat { span, .. }
            | Self::Fold { span, .. }
            | Self::Camera { span, .. }
            | Self::Trace { span, .. }
            | Self::Length { span, .. }
            | Self::Untrace { span, .. }
            | Self::RankDescent { span, .. }
            | Self::Apply { span, .. }
            | Self::Add { span, .. }
            | Self::Multiply { span, .. }
            | Self::Orient { span, .. }
            | Self::Index { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub name: String,
    pub span: Option<Span>,
    pub value: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub variadic: bool,
    pub body: Expr,
    pub span: Option<Span>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
    pub bindings: Vec<Binding>,
    pub goal: Goal,
    pub output_kind: OutputKind,
    pub result: Expr,
    pub source_name: String,
    pub span: Option<Span>,
}

/// The exact condition requested by the final statement of a state document.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Goal {
    /// Evaluate and expose the resulting native state.
    Emit,
    /// Require the resulting native state to equal exact native zero.
    ProveZero,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    Auto,
    String,
    Number,
    Vector,
    Pattern,
    Boolean,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) text: String,
    pub(crate) span: Span,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    Ident,
    Number,
    String,
    LParen,
    RParen,
    Comma,
    Semicolon,
    Equals,
    Arrow,
    Ellipsis,
    Colon,
    Operator,
    Eof,
}

fn operator_character(ch: char) -> bool {
    !ch.is_whitespace()
        && !ch.is_alphanumeric()
        && !matches!(ch, '_' | '"' | '#' | '(' | ')' | ',' | ';' | '=' | ':')
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LanguageNameKind {
    CoreOperation,
    ExactGrammar,
    FunctionGrammar,
    LogicGrammar,
}

pub(crate) const LANGUAGE_NAMESPACE: &[(&str, LanguageNameKind)] = &[
    ("add", LanguageNameKind::CoreOperation),
    ("multiply", LanguageNameKind::CoreOperation),
    ("orient", LanguageNameKind::CoreOperation),
    ("index", LanguageNameKind::CoreOperation),
    ("ADD", LanguageNameKind::CoreOperation),
    ("MULTIPLY", LanguageNameKind::CoreOperation),
    ("ORIENT", LanguageNameKind::CoreOperation),
    ("INDEX", LanguageNameKind::CoreOperation),
    ("trace", LanguageNameKind::ExactGrammar),
    ("length", LanguageNameKind::ExactGrammar),
    ("untrace", LanguageNameKind::ExactGrammar),
    ("rank_descent", LanguageNameKind::ExactGrammar),
    ("apply", LanguageNameKind::ExactGrammar),
    ("rewrite", LanguageNameKind::ExactGrammar),
    ("concat", LanguageNameKind::ExactGrammar),
    ("fold", LanguageNameKind::ExactGrammar),
    ("camera", LanguageNameKind::ExactGrammar),
    ("zero", LanguageNameKind::ExactGrammar),
    ("one", LanguageNameKind::ExactGrammar),
    ("scalar", LanguageNameKind::ExactGrammar),
    ("let", LanguageNameKind::ExactGrammar),
    ("output", LanguageNameKind::ExactGrammar),
    ("as", LanguageNameKind::ExactGrammar),
    ("operator", LanguageNameKind::ExactGrammar),
    ("import", LanguageNameKind::ExactGrammar),
    ("string", LanguageNameKind::ExactGrammar),
    ("number", LanguageNameKind::ExactGrammar),
    ("vector", LanguageNameKind::ExactGrammar),
    ("pattern", LanguageNameKind::ExactGrammar),
    ("boolean", LanguageNameKind::ExactGrammar),
    ("=>", LanguageNameKind::ExactGrammar),
    ("=", LanguageNameKind::ExactGrammar),
    ("...", LanguageNameKind::FunctionGrammar),
    ("parameter", LanguageNameKind::LogicGrammar),
    ("bool", LanguageNameKind::LogicGrammar),
    ("prove", LanguageNameKind::LogicGrammar),
    ("by", LanguageNameKind::LogicGrammar),
    ("truth_table", LanguageNameKind::LogicGrammar),
    ("true", LanguageNameKind::LogicGrammar),
    ("false", LanguageNameKind::LogicGrammar),
    ("not", LanguageNameKind::LogicGrammar),
    ("and", LanguageNameKind::LogicGrammar),
    ("or", LanguageNameKind::LogicGrammar),
    ("xor", LanguageNameKind::LogicGrammar),
    ("implies", LanguageNameKind::LogicGrammar),
    ("iff", LanguageNameKind::LogicGrammar),
];

pub(crate) fn language_name(name: &str) -> Option<LanguageNameKind> {
    LANGUAGE_NAMESPACE
        .iter()
        .find_map(|(candidate, kind)| (*candidate == name).then_some(*kind))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeclaredNameKind {
    Function,
    Operator,
    Binding,
}

#[derive(Debug)]
struct Namespace {
    declarations: BTreeMap<String, DeclaredNameKind>,
}

impl Namespace {
    fn declare(
        &mut self,
        name: &str,
        kind: DeclaredNameKind,
        source_name: &str,
        span: Option<Span>,
    ) -> Result<(), LanguageError> {
        if let Some(language_kind) = language_name(name) {
            return Err(fail(
                "NSS008",
                format!("name {name:?} collides with {language_kind:?}"),
                source_name,
                span,
            ));
        }
        if let Some(existing) = self.declarations.insert(name.to_owned(), kind) {
            return Err(fail(
                "NSS009",
                format!("name {name:?} collides between {existing:?} and {kind:?}"),
                source_name,
                span,
            ));
        }
        Ok(())
    }
}

fn signed_number_can_start_after(tokens: &[Token]) -> bool {
    tokens.last().is_none_or(|token| {
        matches!(
            token.kind,
            TokenKind::LParen
                | TokenKind::Comma
                | TokenKind::Equals
                | TokenKind::Arrow
                | TokenKind::Colon
                | TokenKind::Operator
        ) || (token.kind == TokenKind::Ident && token.text == "output")
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "one lexer loop keeps source-position updates centralized"
)]
pub(crate) fn lex(
    source: &str,
    source_name: &str,
    prefix: &str,
) -> Result<Vec<Token>, LanguageError> {
    let chars: Vec<char> = source.chars().collect();
    let (mut offset, mut line, mut column) = (0, 1, 1);
    let mut tokens = Vec::new();
    while offset < chars.len() {
        let ch = chars[offset];
        if ch.is_whitespace() {
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
            offset += 1;
            continue;
        }
        if ch == '#' {
            while offset < chars.len() && chars[offset] != '\n' {
                offset += 1;
                column += 1;
            }
            continue;
        }
        let (sl, sc) = (line, column);
        if ch == '"' {
            let start = offset;
            offset += 1;
            column += 1;
            let mut escaped = false;
            let mut terminated = false;
            while offset < chars.len() {
                let current = chars[offset];
                if current == '\n' || current == '\r' {
                    return Err(fail(
                        &format!("{prefix}L003"),
                        "string literals cannot contain an unescaped line break",
                        source_name,
                        Some(Span {
                            start_line: sl,
                            start_column: sc,
                            end_line: line,
                            end_column: column,
                        }),
                    ));
                }
                offset += 1;
                column += 1;
                if escaped {
                    escaped = false;
                } else if current == '\\' {
                    escaped = true;
                } else if current == '"' {
                    terminated = true;
                    break;
                }
            }
            if !terminated {
                return Err(fail(
                    &format!("{prefix}L003"),
                    "unterminated string literal",
                    source_name,
                    Some(Span {
                        start_line: sl,
                        start_column: sc,
                        end_line: line,
                        end_column: column,
                    }),
                ));
            }
            let raw: String = chars[start..offset].iter().collect();
            let text = serde_json::from_str(&raw).map_err(|_decode_error| {
                fail(
                    &format!("{prefix}L003"),
                    "invalid UTF-8 string escape",
                    source_name,
                    Some(Span {
                        start_line: sl,
                        start_column: sc,
                        end_line: line,
                        end_column: column,
                    }),
                )
            })?;
            tokens.push(Token {
                kind: TokenKind::String,
                text,
                span: Span {
                    start_line: sl,
                    start_column: sc,
                    end_line: line,
                    end_column: column,
                },
            });
            continue;
        }
        if ch == '.'
            && offset + 2 < chars.len()
            && chars[offset + 1] == '.'
            && chars[offset + 2] == '.'
        {
            offset += 3;
            column += 3;
            tokens.push(Token {
                kind: TokenKind::Ellipsis,
                text: "...".into(),
                span: Span {
                    start_line: sl,
                    start_column: sc,
                    end_line: line,
                    end_column: column,
                },
            });
            continue;
        }
        if ch == '=' && offset + 1 < chars.len() && chars[offset + 1] == '>' {
            offset += 2;
            column += 2;
            tokens.push(Token {
                kind: TokenKind::Arrow,
                text: "=>".into(),
                span: Span {
                    start_line: sl,
                    start_column: sc,
                    end_line: line,
                    end_column: column,
                },
            });
            continue;
        }
        let punctuation = match ch {
            '(' => Some(TokenKind::LParen),
            ')' => Some(TokenKind::RParen),
            ',' => Some(TokenKind::Comma),
            ';' => Some(TokenKind::Semicolon),
            '=' => Some(TokenKind::Equals),
            ':' => Some(TokenKind::Colon),
            _ => None,
        };
        if let Some(kind) = punctuation {
            offset += 1;
            column += 1;
            tokens.push(Token {
                kind,
                text: ch.to_string(),
                span: Span {
                    start_line: sl,
                    start_column: sc,
                    end_line: line,
                    end_column: column,
                },
            });
            continue;
        }
        if ch.is_alphabetic() || ch == '_' {
            let start = offset;
            while offset < chars.len()
                && (chars[offset].is_alphanumeric() || chars[offset] == '_' || chars[offset] == '-')
            {
                offset += 1;
                column += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Ident,
                text: chars[start..offset].iter().collect(),
                span: Span {
                    start_line: sl,
                    start_column: sc,
                    end_line: line,
                    end_column: column,
                },
            });
            continue;
        }
        if ch.is_ascii_digit()
            || (ch == '-'
                && offset + 1 < chars.len()
                && chars[offset + 1].is_ascii_digit()
                && signed_number_can_start_after(&tokens))
        {
            let start = offset;
            if ch == '-' {
                offset += 1;
                column += 1;
            }
            while offset < chars.len() && chars[offset].is_ascii_digit() {
                offset += 1;
                column += 1;
            }
            if offset < chars.len() && chars[offset] == '/' {
                offset += 1;
                column += 1;
                let denominator = offset;
                while offset < chars.len() && chars[offset].is_ascii_digit() {
                    offset += 1;
                    column += 1;
                }
                if denominator == offset {
                    return Err(fail(
                        &format!("{prefix}L002"),
                        "expected denominator digits after '/'",
                        source_name,
                        Some(Span {
                            start_line: sl,
                            start_column: sc,
                            end_line: line,
                            end_column: column,
                        }),
                    ));
                }
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                text: chars[start..offset].iter().collect(),
                span: Span {
                    start_line: sl,
                    start_column: sc,
                    end_line: line,
                    end_column: column,
                },
            });
            continue;
        }
        if operator_character(ch) {
            let start = offset;
            while offset < chars.len() && operator_character(chars[offset]) {
                offset += 1;
                column += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Operator,
                text: chars[start..offset].iter().collect(),
                span: Span {
                    start_line: sl,
                    start_column: sc,
                    end_line: line,
                    end_column: column,
                },
            });
            continue;
        }
        return Err(fail(
            &format!("{prefix}L001"),
            format!("unexpected character {ch:?}"),
            source_name,
            Some(Span {
                start_line: sl,
                start_column: sc,
                end_line: line,
                end_column: column + 1,
            }),
        ));
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
        text: String::new(),
        span: Span {
            start_line: line,
            start_column: column,
            end_line: line,
            end_column: column,
        },
    });
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    source_name: String,
    operators: BTreeMap<String, usize>,
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
    fn expect(
        &mut self,
        kind: TokenKind,
        code: &str,
        message: &str,
    ) -> Result<Token, LanguageError> {
        if self.current().kind != kind {
            return Err(fail(
                code,
                message,
                self.source_name.as_str(),
                Some(self.current().span),
            ));
        }
        Ok(self.advance())
    }
    fn keyword(&self, text: &str) -> bool {
        self.current().kind == TokenKind::Ident && self.current().text == text
    }

    fn optional_semicolon(&mut self) {
        if self.current().kind == TokenKind::Semicolon {
            self.advance();
        }
    }

    fn starts_function(&self) -> bool {
        if !self.keyword("let")
            || self.tokens.get(self.position + 1).map(|token| token.kind) != Some(TokenKind::Ident)
            || self.tokens.get(self.position + 2).map(|token| token.kind) != Some(TokenKind::Equals)
            || self.tokens.get(self.position + 3).map(|token| token.kind) != Some(TokenKind::LParen)
        {
            return false;
        }
        let mut cursor = self.position + 4;
        while matches!(
            self.tokens.get(cursor).map(|token| token.kind),
            Some(TokenKind::Ident | TokenKind::Comma | TokenKind::Ellipsis)
        ) {
            cursor += 1;
        }
        self.tokens.get(cursor).map(|token| token.kind) == Some(TokenKind::RParen)
            && self.tokens.get(cursor + 1).map(|token| token.kind) == Some(TokenKind::Arrow)
    }

    fn starts_operator(&self) -> bool {
        self.keyword("operator")
    }

    fn function(&mut self) -> Result<Function, LanguageError> {
        let start = self.advance().span;
        let name = self.expect(TokenKind::Ident, "NSP028", "expected function name")?;
        self.expect(
            TokenKind::Equals,
            "NSP035",
            "expected '=' after function name",
        )?;
        self.expect(
            TokenKind::LParen,
            "NSP029",
            "expected '(' after function name",
        )?;
        let mut parameters = Vec::new();
        let mut parameter_names = BTreeSet::new();
        let mut variadic = false;
        if self.current().kind != TokenKind::RParen {
            loop {
                let parameter =
                    self.expect(TokenKind::Ident, "NSP030", "expected parameter name")?;
                if language_name(&parameter.text).is_some() {
                    return Err(fail(
                        "NSS008",
                        format!(
                            "parameter name {:?} collides with the language",
                            parameter.text
                        ),
                        &self.source_name,
                        Some(parameter.span),
                    ));
                }
                if !parameter_names.insert(parameter.text.clone()) {
                    return Err(fail(
                        "NSS006",
                        format!("duplicate parameter {:?}", parameter.text),
                        &self.source_name,
                        Some(parameter.span),
                    ));
                }
                parameters.push(parameter.text);
                if self.current().kind == TokenKind::Ellipsis {
                    self.advance();
                    variadic = true;
                    if self.current().kind != TokenKind::RParen {
                        return Err(fail(
                            "NSP054",
                            "a variadic parameter must be last",
                            &self.source_name,
                            Some(self.current().span),
                        ));
                    }
                    break;
                }
                if self.current().kind != TokenKind::Comma {
                    break;
                }
                self.advance();
            }
        }
        self.expect(TokenKind::RParen, "NSP031", "expected ')' after parameters")?;
        self.expect(
            TokenKind::Arrow,
            "NSP032",
            "expected '=>' before function body",
        )?;
        let body = self.expression()?;
        self.optional_semicolon();
        Ok(Function {
            name: name.text,
            parameters,
            variadic,
            body,
            span: Some(start.join(self.tokens[self.position.saturating_sub(1)].span)),
        })
    }

    fn operator_function(&mut self) -> Result<Function, LanguageError> {
        let start = self.advance().span;
        let name = self.expect(
            TokenKind::String,
            "NSP038",
            "expected a quoted operator name",
        )?;
        if name.text.is_empty() || name.text.chars().any(char::is_whitespace) {
            return Err(fail(
                "NSS006",
                "an operator name must be nonempty and contain no whitespace",
                &self.source_name,
                Some(name.span),
            ));
        }
        if language_name(&name.text).is_some() {
            return Err(fail(
                "NSS008",
                format!("operator name {:?} collides with the language", name.text),
                &self.source_name,
                Some(name.span),
            ));
        }
        let is_word = name
            .text
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_');
        if !is_word && !name.text.chars().all(operator_character) {
            return Err(fail(
                "NSS007",
                "an operator name must be one identifier or one punctuation sequence",
                &self.source_name,
                Some(name.span),
            ));
        }
        self.expect(
            TokenKind::Equals,
            "NSP039",
            "expected '=' after operator name",
        )?;
        self.expect(
            TokenKind::LParen,
            "NSP040",
            "expected '(' after operator name",
        )?;
        let left = self.expect(TokenKind::Ident, "NSP041", "expected left parameter")?;
        self.expect(
            TokenKind::Comma,
            "NSP042",
            "expected ',' between operator parameters",
        )?;
        let right = self.expect(TokenKind::Ident, "NSP043", "expected right parameter")?;
        for parameter in [&left, &right] {
            if language_name(&parameter.text).is_some() {
                return Err(fail(
                    "NSS008",
                    format!(
                        "parameter name {:?} collides with the language",
                        parameter.text
                    ),
                    &self.source_name,
                    Some(parameter.span),
                ));
            }
        }
        if left.text == right.text {
            return Err(fail(
                "NSS006",
                format!("duplicate parameter {:?}", left.text),
                &self.source_name,
                Some(right.span),
            ));
        }
        self.expect(
            TokenKind::RParen,
            "NSP044",
            "expected ')' after operator parameters",
        )?;
        self.expect(
            TokenKind::Arrow,
            "NSP045",
            "expected '=>' before operator body",
        )?;
        let body = self.expression()?;
        self.optional_semicolon();
        Ok(Function {
            name: name.text,
            parameters: vec![left.text, right.text],
            variadic: false,
            body,
            span: Some(start.join(self.tokens[self.position.saturating_sub(1)].span)),
        })
    }

    fn current_operator(&self) -> Option<(&str, usize)> {
        matches!(self.current().kind, TokenKind::Ident | TokenKind::Operator)
            .then_some(self.current().text.as_str())
            .and_then(|name| self.operators.get(name).copied().map(|order| (name, order)))
    }

    fn expression(&mut self) -> Result<Expr, LanguageError> {
        self.operator_expression(0)
    }

    fn operator_expression(&mut self, minimum_precedence: usize) -> Result<Expr, LanguageError> {
        let mut left = self.atom()?;
        while let Some((name, precedence)) = self
            .current_operator()
            .map(|(name, precedence)| (name.to_owned(), precedence))
        {
            if precedence < minimum_precedence {
                break;
            }
            self.advance();
            let right = self.operator_expression(precedence + 1)?;
            let span = left
                .span()
                .zip(right.span())
                .map(|(first, last)| first.join(last));
            left = Expr::Call {
                function: name,
                arguments: vec![left, right],
                span,
            };
        }
        Ok(left)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive grammar match keeps the closed expression syntax auditable"
    )]
    fn atom(&mut self) -> Result<Expr, LanguageError> {
        if self.current().kind == TokenKind::LParen {
            self.advance();
            let value = self.expression()?;
            self.expect(TokenKind::RParen, "NSP007", "expected ')' after expression")?;
            return Ok(value);
        }
        if self.current().kind == TokenKind::String {
            let literal = self.advance();
            return utf8_expression(&literal.text, Some(literal.span), &self.source_name);
        }
        if self.current().kind == TokenKind::Number {
            let literal = self.advance();
            rational(&literal.text).map_err(|message| {
                fail("NST005", message, &self.source_name, Some(literal.span))
            })?;
            return Ok(Expr::Scalar {
                real: literal.text,
                imag: "0".into(),
                span: Some(literal.span),
            });
        }
        let start = self.expect(TokenKind::Ident, "NSP008", "expected a state expression")?;
        match start.text.as_str() {
            "zero" => Ok(Expr::Zero {
                span: Some(start.span),
            }),
            "one" => Ok(Expr::One {
                span: Some(start.span),
            }),
            "scalar" => {
                self.expect(TokenKind::LParen, "NSP010", "expected '(' after 'scalar'")?;
                let real = self.fraction()?;
                self.expect(
                    TokenKind::Comma,
                    "NSP011",
                    "expected ',' between scalar coordinates",
                )?;
                let imag = self.fraction()?;
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP012",
                    "expected ')' after scalar coordinates",
                )?;
                Ok(Expr::Scalar {
                    real,
                    imag,
                    span: Some(start.span.join(end.span)),
                })
            }
            "add" | "multiply" => {
                self.expect(TokenKind::LParen, "NSP013", "expected '(' after operation")?;
                let mut operands = vec![self.expression()?];
                while self.current().kind == TokenKind::Comma {
                    self.advance();
                    operands.push(self.expression()?);
                }
                let end =
                    self.expect(TokenKind::RParen, "NSP014", "expected ')' after operands")?;
                if operands.len() < 2
                    && !operands
                        .iter()
                        .any(|operand| matches!(operand, Expr::Spread { .. }))
                {
                    return Err(fail(
                        "NST001",
                        format!("{} requires at least two operands", start.text),
                        &self.source_name,
                        Some(start.span),
                    ));
                }
                let span = Some(start.span.join(end.span));
                if start.text == "add" {
                    Ok(Expr::Add { operands, span })
                } else {
                    Ok(Expr::Multiply { operands, span })
                }
            }
            "orient" => {
                self.expect(TokenKind::LParen, "NSP015", "expected '(' after 'orient'")?;
                let turns_span = self.current().span;
                let turns = self.integer("NST002", "orient turns must be an integer", false)?;
                if !is_canonical_orientation(turns) {
                    return Err(fail(
                        "NST002",
                        "orient turns must be an integer from 0 through 3; retain repeated counts with INDEX",
                        &self.source_name,
                        Some(turns_span),
                    ));
                }
                self.expect(
                    TokenKind::Comma,
                    "NSP016",
                    "expected ',' after orient turns",
                )?;
                let value = Box::new(self.expression()?);
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP017",
                    "expected ')' after orient expression",
                )?;
                Ok(Expr::Orient {
                    turns,
                    value,
                    span: Some(start.span.join(end.span)),
                })
            }
            "index" => {
                self.expect(TokenKind::LParen, "NSP018", "expected '(' after 'index'")?;
                let direction =
                    self.positive("NST003", "index direction must be a positive integer")?;
                self.expect(
                    TokenKind::Comma,
                    "NSP019",
                    "expected ',' after index direction",
                )?;
                let value = Box::new(self.expression()?);
                let multiplicity = if self.current().kind == TokenKind::Comma {
                    self.advance();
                    self.positive("NST003", "index multiplicity must be a positive integer")?
                } else {
                    1
                };
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP020",
                    "expected ')' after index expression",
                )?;
                Ok(Expr::Index {
                    direction,
                    multiplicity,
                    value,
                    span: Some(start.span.join(end.span)),
                })
            }
            "concat" => {
                self.expect(TokenKind::LParen, "NSP055", "expected '(' after 'concat'")?;
                let direction =
                    self.positive("NST006", "concat direction must be a positive integer")?;
                self.expect(
                    TokenKind::Comma,
                    "NSP056",
                    "expected ',' after concat direction",
                )?;
                let mut values = vec![self.expression()?];
                while self.current().kind == TokenKind::Comma {
                    self.advance();
                    values.push(self.expression()?);
                }
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP057",
                    "expected ')' after concat values",
                )?;
                Ok(Expr::Concat {
                    direction,
                    values,
                    span: Some(start.span.join(end.span)),
                })
            }
            "fold" => {
                self.expect(TokenKind::LParen, "NSP058", "expected '(' after 'fold'")?;
                let function = self.expect(
                    TokenKind::Ident,
                    "NSP059",
                    "fold expects a source function name",
                )?;
                self.expect(
                    TokenKind::Comma,
                    "NSP060",
                    "expected ',' after fold function",
                )?;
                let initial = Box::new(self.expression()?);
                self.expect(
                    TokenKind::Comma,
                    "NSP061",
                    "expected ',' after fold initial value",
                )?;
                let mut values = vec![self.expression()?];
                while self.current().kind == TokenKind::Comma {
                    self.advance();
                    values.push(self.expression()?);
                }
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP062",
                    "expected ')' after fold values",
                )?;
                Ok(Expr::Fold {
                    function: function.text,
                    initial,
                    values,
                    span: Some(start.span.join(end.span)),
                })
            }
            "camera" => {
                self.expect(TokenKind::LParen, "NSP063", "expected '(' after 'camera'")?;
                let from_direction = self.positive(
                    "NST008",
                    "camera source direction must be a positive integer",
                )?;
                self.expect(
                    TokenKind::Comma,
                    "NSP064",
                    "expected ',' after camera source direction",
                )?;
                let to_direction = self.nonnegative(
                    "NST008",
                    "camera destination direction must be a nonnegative integer",
                )?;
                self.expect(
                    TokenKind::Comma,
                    "NSP065",
                    "expected ',' after camera destination direction",
                )?;
                let value = Box::new(self.expression()?);
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP066",
                    "expected ')' after camera value",
                )?;
                Ok(Expr::Camera {
                    from_direction,
                    to_direction,
                    value,
                    span: Some(start.span.join(end.span)),
                })
            }
            "trace" => {
                self.expect(TokenKind::LParen, "NSP046", "expected '(' after 'trace'")?;
                let function = self.expect(
                    TokenKind::Ident,
                    "NSP047",
                    "trace expects a source function name",
                )?;
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP048",
                    "expected ')' after trace target",
                )?;
                Ok(Expr::Trace {
                    function: function.text,
                    span: Some(start.span.join(end.span)),
                })
            }
            "length" => {
                self.expect(TokenKind::LParen, "NSP052", "expected '(' after 'length'")?;
                let value = Box::new(self.expression()?);
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP053",
                    "expected ')' after length argument",
                )?;
                Ok(Expr::Length {
                    value,
                    span: Some(start.span.join(end.span)),
                })
            }
            "untrace" => {
                self.expect(TokenKind::LParen, "NSP049", "expected '(' after 'untrace'")?;
                let value = Box::new(self.expression()?);
                let rank = if self.current().kind == TokenKind::Comma {
                    self.advance();
                    self.unit_ratio(
                        "NSP051",
                        "untrace rank must be an exact number from 0 through 1",
                    )?
                } else {
                    "1".into()
                };
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP050",
                    "expected ')' after untrace arguments",
                )?;
                Ok(Expr::Untrace {
                    value,
                    rank,
                    span: Some(start.span.join(end.span)),
                })
            }
            "rank_descent" => {
                self.expect(
                    TokenKind::LParen,
                    "NSP067",
                    "expected '(' after 'rank_descent'",
                )?;
                let value = Box::new(self.expression()?);
                let target_rank = match self.current().kind {
                    TokenKind::Comma => {
                        self.advance();
                        Some(self.nonzero_unit_ratio(
                            "NSP069",
                            "rank_descent target rank must be exact, greater than zero through one",
                        )?)
                    }
                    _ => None,
                };
                let minimum_agreement = match self.current().kind {
                    TokenKind::Comma => {
                        self.advance();
                        Some(self.unit_ratio(
                            "NSP074",
                            "rank_descent minimum agreement must be exact, from zero through one",
                        )?)
                    }
                    _ => None,
                };
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP068",
                    "expected ')' after rank_descent arguments",
                )?;
                Ok(Expr::RankDescent {
                    value,
                    target_rank,
                    minimum_agreement,
                    span: Some(start.span.join(end.span)),
                })
            }
            "rewrite" => {
                self.expect(TokenKind::LParen, "NSP075", "expected '(' after rewrite")?;
                let mut arguments = vec![self.expression()?];
                while self.current().kind == TokenKind::Comma {
                    self.advance();
                    arguments.push(self.expression()?);
                }
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP075",
                    "expected ')' after rewrite arguments",
                )?;
                if arguments.len() != 3 {
                    return Err(fail(
                        "NSP075",
                        "rewrite expects graph, pattern, and replacement",
                        &self.source_name,
                        Some(start.span),
                    ));
                }
                Ok(Expr::Reflect {
                    operation: crate::reflection::Operation::Rewrite,
                    arguments,
                    span: Some(start.span.join(end.span)),
                })
            }
            "apply" => {
                self.expect(TokenKind::LParen, "NSP070", "expected '(' after 'apply'")?;
                let pattern = Box::new(self.expression()?);
                if !matches!(pattern.as_ref(), Expr::RankDescent { .. }) {
                    let mut arguments = vec![*pattern];
                    while self.current().kind == TokenKind::Comma {
                        self.advance();
                        arguments.push(self.expression()?);
                    }
                    let end = self.expect(
                        TokenKind::RParen,
                        "NSP071",
                        "expected ')' after apply arguments",
                    )?;
                    return Ok(Expr::Reflect {
                        operation: crate::reflection::Operation::Apply,
                        arguments,
                        span: Some(start.span.join(end.span)),
                    });
                }
                self.expect(
                    TokenKind::Comma,
                    "NSP071",
                    "expected ',' after apply pattern",
                )?;
                let position =
                    self.positive("NSP072", "apply position must be a positive integer")?;
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP071",
                    "expected ')' after apply position",
                )?;
                Ok(Expr::Apply {
                    pattern,
                    position,
                    span: Some(start.span.join(end.span)),
                })
            }
            "let" | "output" => Err(fail(
                "NSP009",
                format!("reserved word {:?} is not an expression", start.text),
                &self.source_name,
                Some(start.span),
            )),
            name => {
                if self.current().kind == TokenKind::Ellipsis {
                    let end = self.advance();
                    return Ok(Expr::Spread {
                        name: name.into(),
                        span: Some(start.span.join(end.span)),
                    });
                }
                if self.current().kind != TokenKind::LParen {
                    return Ok(Expr::Reference {
                        name: name.into(),
                        span: Some(start.span),
                    });
                }
                self.advance();
                let mut arguments = Vec::new();
                if self.current().kind != TokenKind::RParen {
                    arguments.push(self.expression()?);
                    while self.current().kind == TokenKind::Comma {
                        self.advance();
                        arguments.push(self.expression()?);
                    }
                }
                let end = self.expect(
                    TokenKind::RParen,
                    "NSP033",
                    "expected ')' after function arguments",
                )?;
                Ok(Expr::Call {
                    function: name.into(),
                    arguments,
                    span: Some(start.span.join(end.span)),
                })
            }
        }
    }
    fn fraction(&mut self) -> Result<String, LanguageError> {
        let token = self.expect(
            TokenKind::Number,
            "NST004",
            "expected an exact rational number",
        )?;
        rational(&token.text).map_err(|_rational_error| {
            fail(
                "NST005",
                format!("invalid rational literal {:?}", token.text),
                &self.source_name,
                Some(token.span),
            )
        })?;
        Ok(token.text)
    }
    fn integer(&mut self, code: &str, message: &str, positive: bool) -> Result<i64, LanguageError> {
        let token = self.expect(TokenKind::Number, code, message)?;
        if token.text.contains('/') {
            return Err(fail(code, message, &self.source_name, Some(token.span)));
        }
        let value = token
            .text
            .parse::<i64>()
            .map_err(|_parse_error| fail(code, message, &self.source_name, Some(token.span)))?;
        if positive && value <= 0 {
            return Err(fail(code, message, &self.source_name, Some(token.span)));
        }
        Ok(value)
    }
    fn positive(&mut self, code: &str, message: &str) -> Result<u64, LanguageError> {
        self.unsigned(code, message, true)
    }

    fn nonnegative(&mut self, code: &str, message: &str) -> Result<u64, LanguageError> {
        self.unsigned(code, message, false)
    }

    fn unsigned(
        &mut self,
        code: &str,
        message: &str,
        positive: bool,
    ) -> Result<u64, LanguageError> {
        // Directions/depths use the same unsigned carrier as native states.
        // Parsing through i64 made the runtime's own trace coordinates unreadable.
        let token = self.expect(TokenKind::Number, code, message)?;
        let value = token
            .text
            .parse::<u64>()
            .map_err(|_parse_error| fail(code, message, &self.source_name, Some(token.span)))?;
        if positive && value == 0 {
            return Err(fail(code, message, &self.source_name, Some(token.span)));
        }
        Ok(value)
    }

    fn unit_ratio(&mut self, code: &str, message: &str) -> Result<String, LanguageError> {
        let token = self.expect(TokenKind::Number, code, message)?;
        let value = rational(&token.text)
            .map_err(|_parse_error| fail(code, message, &self.source_name, Some(token.span)))?;
        if value < BigRational::zero() || value > BigRational::one() {
            return Err(fail(code, message, &self.source_name, Some(token.span)));
        }
        Ok(token.text)
    }

    fn nonzero_unit_ratio(&mut self, code: &str, message: &str) -> Result<String, LanguageError> {
        let token = self.expect(TokenKind::Number, code, message)?;
        let value = rational(&token.text)
            .map_err(|_parse_error| fail(code, message, &self.source_name, Some(token.span)))?;
        if value <= BigRational::zero() || value > BigRational::one() {
            return Err(fail(code, message, &self.source_name, Some(token.span)));
        }
        Ok(token.text)
    }
}

fn utf8_expression(
    text: &str,
    span: Option<Span>,
    source_name: &str,
) -> Result<Expr, LanguageError> {
    let mut terms = Vec::with_capacity(text.len());
    for (position, byte) in text.bytes().enumerate() {
        let position = u64::try_from(position).map_err(|_conversion_error| {
            fail("NST011", "UTF-8 literal is too long", source_name, span)
        })?;
        let position_direction = UTF8_POSITION_DIRECTION_START
            .checked_add(position)
            .ok_or_else(|| fail("NST011", "UTF-8 literal is too long", source_name, span))?;
        let byte_direction = u64::from(byte) + 1;
        terms.push(Expr::Index {
            direction: position_direction,
            multiplicity: 1,
            value: Box::new(Expr::Index {
                direction: byte_direction,
                multiplicity: 1,
                value: Box::new(Expr::One { span }),
                span,
            }),
            span,
        });
    }
    let mut terms = terms.into_iter();
    let Some(first) = terms.next() else {
        return Ok(Expr::Zero { span });
    };
    let Some(second) = terms.next() else {
        return Ok(first);
    };
    let mut operands = vec![first, second];
    operands.extend(terms);
    Ok(Expr::Add { operands, span })
}

/// Decode a state produced by an exact UTF-8 string literal.
///
/// # Errors
///
/// Returns an error unless every term has coefficient one, one byte direction,
/// one unique contiguous position direction, and the recovered bytes are UTF-8.
pub fn decode_utf8(state: &NativeState) -> Result<String, String> {
    let mut bytes = BTreeMap::new();
    for (index, coefficient) in &state.0 {
        if coefficient != &NativeScalar::one() || index.0.len() != 2 {
            return Err("state is not a canonical UTF-8 index state".into());
        }
        let mut byte = None;
        let mut position = None;
        for (&direction, depth) in &index.0 {
            if depth != &BigUint::one() {
                return Err("UTF-8 index depths must equal one".into());
            }
            if direction <= UTF8_BYTE_DIRECTION_MAX {
                byte = Some(u8::try_from(direction - 1).map_err(|error| error.to_string())?);
            } else {
                position = Some(direction - UTF8_POSITION_DIRECTION_START);
            }
        }
        let byte = byte.ok_or("UTF-8 state term has no byte direction")?;
        let position = position.ok_or("UTF-8 state term has no position direction")?;
        if bytes.insert(position, byte).is_some() {
            return Err("UTF-8 state repeats a position".into());
        }
    }
    let length = u64::try_from(bytes.len()).map_err(|error| error.to_string())?;
    if bytes.keys().copied().ne(0..length) {
        return Err("UTF-8 state positions must be contiguous from zero".into());
    }
    String::from_utf8(bytes.into_values().collect()).map_err(|error| error.to_string())
}

/// Project one exact state to a requested user-facing output kind.
///
/// # Errors
///
/// Returns an error when the state is not representable by the requested
/// camera.
pub fn output_data(state: &NativeState, kind: OutputKind) -> Result<Value, String> {
    fn number(state: &NativeState) -> Option<String> {
        if state.is_zero() {
            return Some("0".into());
        }
        let mut terms = state.0.iter();
        let (index, coefficient) = terms.next()?;
        if terms.next().is_some() {
            return None;
        }
        (index.0.is_empty() && coefficient.imag.is_zero()).then(|| rational_text(&coefficient.real))
    }

    fn cone_vector(state: &NativeState) -> Option<[String; 3]> {
        if state.is_zero() {
            return Some(["0".into(), "0".into(), "0".into()]);
        }
        let mut terms = state.0.iter();
        let (index, coefficient) = terms.next()?;
        if terms.next().is_some() || !index.0.is_empty() {
            return None;
        }
        let x_squared = &coefficient.real * &coefficient.real;
        let y_squared = &coefficient.imag * &coefficient.imag;
        let difference = &x_squared - &y_squared;
        let mix =
            BigRational::from_integer(BigInt::from(2)) * &coefficient.real * &coefficient.imag;
        let size = x_squared + y_squared;
        Some([
            rational_text(&difference),
            rational_text(&mix),
            rational_text(&size),
        ])
    }

    match kind {
        OutputKind::Number => number(state)
            .map(|value| json!({"kind":"number","value":value}))
            .ok_or("result is not a real number".into()),
        OutputKind::Vector => cone_vector(state)
            .map(|value| json!({"kind":"vector","value":value}))
            .ok_or("result is not one unindexed oriented scalar".into()),
        OutputKind::String => {
            decode_utf8(state).map(|value| json!({"kind":"string","value":value}))
        }
        OutputKind::Boolean => {
            if state.is_zero() {
                Ok(json!({"kind":"boolean","value":false}))
            } else if state == &NativeState::one() {
                Ok(json!({"kind":"boolean","value":true}))
            } else {
                Err("result is neither Boolean zero nor Boolean one".into())
            }
        }
        OutputKind::Pattern => Ok(json!({"kind":"pattern","value":state.to_data()})),
        OutputKind::Auto => {
            if let Some(value) = number(state) {
                Ok(json!({"kind":"number","value":value}))
            } else if let Ok(value) = decode_utf8(state) {
                Ok(json!({"kind":"string","value":value}))
            } else {
                Ok(json!({"kind":"pattern","value":state.to_data()}))
            }
        }
    }
}

fn parse_result(
    parser: &mut Parser,
    source_name: &str,
) -> Result<(Goal, OutputKind, Expr), LanguageError> {
    let output = parser.keyword("output");
    if output {
        parser.advance();
    }
    let left = parser.expression()?;
    let mut output_kind = OutputKind::Auto;
    if output && parser.keyword("as") {
        parser.advance();
        let kind = parser.expect(TokenKind::Ident, "NSP036", "expected output kind")?;
        output_kind = match kind.text.as_str() {
            "string" => OutputKind::String,
            "number" => OutputKind::Number,
            "vector" => OutputKind::Vector,
            "pattern" => OutputKind::Pattern,
            "boolean" => OutputKind::Boolean,
            _ => {
                return Err(fail(
                    "NSP037",
                    "output kind must be string, number, vector, pattern, or boolean",
                    source_name,
                    Some(kind.span),
                ));
            }
        };
    }
    let (goal, result) = if output {
        (Goal::Emit, left)
    } else if parser.current().kind == TokenKind::Equals {
        parser.advance();
        let right = parser.expression()?;
        let span = left.span().zip(right.span()).map(|(a, b)| a.join(b));
        (
            Goal::ProveZero,
            Expr::Add {
                operands: vec![
                    left,
                    Expr::Orient {
                        turns: 2,
                        value: Box::new(right),
                        span,
                    },
                ],
                span,
            },
        )
    } else {
        return Err(fail(
            "NSP034",
            "use 'output expression' for a result or 'left = right' for a zero proof",
            source_name,
            left.span(),
        ));
    };
    Ok((goal, output_kind, result))
}

/// Parse an exact-state Native Space 1.0 document.
///
/// # Errors
///
/// Returns a located diagnostic when lexical or grammatical validation fails.
pub fn parse(source: &str, source_name: &str) -> Result<Program, LanguageError> {
    let tokens = lex(source, source_name, "NS")?;
    let declared_operators = tokens
        .windows(2)
        .filter(|pair| pair[0].kind == TokenKind::Ident && pair[0].text == "operator")
        .filter(|pair| pair[1].kind == TokenKind::String)
        .map(|pair| pair[1].text.clone())
        .collect::<Vec<_>>();
    let operator_count = declared_operators.len();
    let operators = declared_operators
        .into_iter()
        .enumerate()
        .map(|(index, name)| (name, operator_count - index))
        .collect();
    let mut parser = Parser {
        tokens,
        position: 0,
        source_name: source_name.into(),
        operators,
    };
    let start = parser.current().span;
    let mut namespace = Namespace {
        declarations: BTreeMap::new(),
    };
    let mut functions = Vec::new();
    let mut bindings = Vec::new();
    while parser.starts_function() || parser.starts_operator() || parser.keyword("let") {
        if parser.starts_function() || parser.starts_operator() {
            let is_operator = parser.starts_operator();
            let function = if is_operator {
                parser.operator_function()?
            } else {
                parser.function()?
            };
            namespace.declare(
                &function.name,
                if is_operator {
                    DeclaredNameKind::Operator
                } else {
                    DeclaredNameKind::Function
                },
                source_name,
                function.span,
            )?;
            functions.push(function);
            continue;
        }
        let binding_start = parser.advance().span;
        let name = parser.expect(
            TokenKind::Ident,
            "NSP004",
            "expected binding name after 'let'",
        )?;
        namespace.declare(
            &name.text,
            DeclaredNameKind::Binding,
            source_name,
            Some(name.span),
        )?;
        parser.expect(
            TokenKind::Equals,
            "NSP005",
            "expected '=' after binding name",
        )?;
        let value = parser.expression()?;
        parser.optional_semicolon();
        bindings.push(Binding {
            name: name.text,
            value,
            span: Some(binding_start.join(parser.tokens[parser.position.saturating_sub(1)].span)),
        });
    }
    if parser.current().kind == TokenKind::Eof {
        return Err(fail(
            "NSP001",
            "expected a result expression or zero equality",
            source_name,
            Some(parser.current().span),
        ));
    }
    let (goal, output_kind, result) = parse_result(&mut parser, source_name)?;
    parser.optional_semicolon();
    let end = parser.tokens[parser.position.saturating_sub(1)].span;
    parser.expect(TokenKind::Eof, "NSP003", "unexpected content after program")?;
    Ok(Program {
        functions,
        bindings,
        goal,
        output_kind,
        result,
        source_name: source_name.into(),
        span: Some(start.join(end)),
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive expression match keeps spread-context validation auditable"
)]
fn analyze_expr(
    expr: &Expr,
    names: &BTreeMap<String, bool>,
    functions: &BTreeMap<String, &Function>,
    source: &str,
    out: &mut Vec<Diagnostic>,
    spread_allowed: bool,
) {
    match expr {
        Expr::Reference { name, span } => match names.get(name) {
            None => out.push(
                fail(
                    "NSS002",
                    format!("unknown or forward reference {name:?}"),
                    source,
                    *span,
                )
                .0,
            ),
            Some(true) => out.push(
                fail(
                    "NSS009",
                    format!("variadic parameter {name:?} must be spread as {name}..."),
                    source,
                    *span,
                )
                .0,
            ),
            Some(false) => {}
        },
        Expr::Spread { name, span } => match names.get(name) {
            Some(true) if spread_allowed => {}
            Some(true) => out.push(
                fail(
                    "NSS009",
                    "a variadic parameter can only be spread inside an argument or operand list",
                    source,
                    *span,
                )
                .0,
            ),
            _ => out.push(
                fail(
                    "NSS009",
                    format!("{name:?} is not a variadic parameter"),
                    source,
                    *span,
                )
                .0,
            ),
        },
        Expr::Call {
            function,
            arguments,
            span,
        } => {
            if let Some(definition) = functions.get(function) {
                let has_spread = arguments
                    .iter()
                    .any(|argument| matches!(argument, Expr::Spread { .. }));
                if !has_spread && !accepts_arity(definition, arguments.len()) {
                    out.push(
                        fail(
                            "NSS004",
                            format!(
                                "function {function:?} expects {} arguments, found {}",
                                expected_arity(definition),
                                arguments.len()
                            ),
                            source,
                            *span,
                        )
                        .0,
                    );
                }
            } else {
                out.push(
                    fail(
                        "NSS003",
                        format!("unknown function {function:?}"),
                        source,
                        *span,
                    )
                    .0,
                );
            }
            arguments
                .iter()
                .for_each(|item| analyze_expr(item, names, functions, source, out, true));
        }
        Expr::Trace { function, span } => {
            if !functions.contains_key(function) {
                out.push(
                    fail(
                        "NSS003",
                        format!("unknown function {function:?}"),
                        source,
                        *span,
                    )
                    .0,
                );
            }
        }
        Expr::Reflect {
            operation,
            arguments,
            span,
        } => {
            if !operation.accepts(arguments.len()) {
                out.push(fail("NSR001", "invalid reflection argument count", source, *span).0);
            }
            arguments
                .iter()
                .for_each(|item| analyze_expr(item, names, functions, source, out, true));
        }
        Expr::Add { operands, .. } | Expr::Multiply { operands, .. } => operands
            .iter()
            .for_each(|item| analyze_expr(item, names, functions, source, out, true)),
        Expr::Concat { values, .. } => values
            .iter()
            .for_each(|item| analyze_expr(item, names, functions, source, out, true)),
        Expr::Fold {
            function,
            initial,
            values,
            span,
        } => {
            match functions.get(function) {
                Some(definition) if !definition.variadic && definition.parameters.len() == 2 => {}
                Some(_) => out.push(
                    fail(
                        "NSS010",
                        format!("fold function {function:?} must accept exactly two parameters"),
                        source,
                        *span,
                    )
                    .0,
                ),
                None => out.push(
                    fail(
                        "NSS003",
                        format!("unknown function {function:?}"),
                        source,
                        *span,
                    )
                    .0,
                ),
            }
            analyze_expr(initial, names, functions, source, out, false);
            values
                .iter()
                .for_each(|item| analyze_expr(item, names, functions, source, out, true));
        }
        Expr::Orient { turns, value, span } => {
            if !is_canonical_orientation(*turns) {
                out.push(
                    fail(
                        "NST002",
                        "orient turns must be an integer from 0 through 3; retain repeated counts with INDEX",
                        source,
                        *span,
                    )
                    .0,
                );
            }
            analyze_expr(value, names, functions, source, out, false);
        }
        Expr::Index { value, .. }
        | Expr::Camera { value, .. }
        | Expr::Length { value, .. }
        | Expr::Untrace { value, .. }
        | Expr::RankDescent { value, .. } => {
            analyze_expr(value, names, functions, source, out, false);
        }
        Expr::Apply { pattern, .. } => {
            analyze_expr(pattern, names, functions, source, out, false);
        }
        _ => {}
    }
}

fn expression_calls<'a>(expr: &'a Expr, calls: &mut Vec<(&'a str, Option<Span>)>) {
    match expr {
        Expr::Reflect { arguments, .. } => {
            arguments
                .iter()
                .for_each(|argument| expression_calls(argument, calls));
        }
        Expr::Call {
            function,
            arguments,
            span,
        } => {
            calls.push((function, *span));
            arguments
                .iter()
                .for_each(|argument| expression_calls(argument, calls));
        }
        Expr::Add { operands, .. } | Expr::Multiply { operands, .. } => operands
            .iter()
            .for_each(|operand| expression_calls(operand, calls)),
        Expr::Concat { values, .. } => values
            .iter()
            .for_each(|value| expression_calls(value, calls)),
        Expr::Fold {
            function,
            initial,
            values,
            span,
        } => {
            calls.push((function, *span));
            expression_calls(initial, calls);
            values
                .iter()
                .for_each(|value| expression_calls(value, calls));
        }
        Expr::Orient { value, .. }
        | Expr::Index { value, .. }
        | Expr::Camera { value, .. }
        | Expr::Length { value, .. }
        | Expr::Untrace { value, .. }
        | Expr::RankDescent { value, .. } => {
            expression_calls(value, calls);
        }
        Expr::Apply { pattern, .. } => expression_calls(pattern, calls),
        Expr::Zero { .. }
        | Expr::One { .. }
        | Expr::Scalar { .. }
        | Expr::Reference { .. }
        | Expr::Spread { .. }
        | Expr::Trace { .. } => {}
    }
}

fn find_function_cycle<'a>(
    function: &'a str,
    functions: &BTreeMap<String, &'a Function>,
    states: &mut BTreeMap<&'a str, u8>,
    active: &mut Vec<&'a str>,
) -> Option<(Vec<String>, Option<Span>)> {
    states.insert(function, 1);
    active.push(function);
    let definition = functions.get(function)?;
    let mut calls = Vec::new();
    expression_calls(&definition.body, &mut calls);
    for (called, span) in calls {
        if !functions.contains_key(called) {
            continue;
        }
        match states.get(called).copied().unwrap_or_default() {
            0 => {
                if let Some(cycle) = find_function_cycle(called, functions, states, active) {
                    return Some(cycle);
                }
            }
            1 => {
                let start = active.iter().position(|name| *name == called).unwrap_or(0);
                let cycle = active[start..]
                    .iter()
                    .copied()
                    .chain(std::iter::once(called))
                    .map(str::to_owned)
                    .collect();
                return Some((cycle, span));
            }
            _ => {}
        }
    }
    active.pop();
    states.insert(function, 2);
    None
}

fn executed_function_cycle(
    program: &Program,
    functions: &BTreeMap<String, &Function>,
) -> Option<(Vec<String>, Option<Span>)> {
    let mut states = BTreeMap::new();
    let mut active = Vec::new();
    let mut roots = Vec::new();
    for binding in &program.bindings {
        expression_calls(&binding.value, &mut roots);
    }
    expression_calls(&program.result, &mut roots);
    for (function, _) in roots {
        if functions.contains_key(function)
            && states.get(function).copied().unwrap_or_default() == 0
            && let Some(cycle) = find_function_cycle(function, functions, &mut states, &mut active)
        {
            return Some(cycle);
        }
    }
    None
}

fn fixed_parameter_count(function: &Function) -> usize {
    function
        .parameters
        .len()
        .saturating_sub(usize::from(function.variadic))
}

fn accepts_arity(function: &Function, actual: usize) -> bool {
    if function.variadic {
        actual >= fixed_parameter_count(function)
    } else {
        actual == function.parameters.len()
    }
}

fn expected_arity(function: &Function) -> String {
    if function.variadic {
        format!("at least {}", fixed_parameter_count(function))
    } else {
        function.parameters.len().to_string()
    }
}

#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "one ordered analysis pass preserves deterministic first-diagnostic behavior"
)]
pub fn analyze(program: &Program) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut function_names = BTreeMap::new();
    for function in &program.functions {
        if function_names
            .insert(function.name.clone(), function)
            .is_some()
        {
            out.push(
                fail(
                    "NSS005",
                    format!("duplicate function {:?}", function.name),
                    &program.source_name,
                    function.span,
                )
                .0,
            );
        }
        let mut parameters = BTreeSet::new();
        if function.variadic && function.parameters.is_empty() {
            out.push(
                fail(
                    "NSS009",
                    "a variadic function requires a trailing pack parameter",
                    &program.source_name,
                    function.span,
                )
                .0,
            );
        }
        for parameter in &function.parameters {
            if !parameters.insert(parameter.clone()) {
                out.push(
                    fail(
                        "NSS006",
                        format!("duplicate parameter {parameter:?}"),
                        &program.source_name,
                        function.span,
                    )
                    .0,
                );
            }
        }
    }
    for function in &program.functions {
        let variadic = function
            .variadic
            .then(|| function.parameters.last())
            .flatten();
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| (parameter.clone(), variadic == Some(parameter)))
            .collect();
        analyze_expr(
            &function.body,
            &parameters,
            &function_names,
            &program.source_name,
            &mut out,
            false,
        );
    }
    if let Some((cycle, span)) = executed_function_cycle(program, &function_names) {
        out.push(
            fail(
                "NSS007",
                format!("cyclic function call: {}", cycle.join(" -> ")),
                &program.source_name,
                span,
            )
            .0,
        );
    }
    let mut names = BTreeMap::new();
    for binding in &program.bindings {
        analyze_expr(
            &binding.value,
            &names,
            &function_names,
            &program.source_name,
            &mut out,
            false,
        );
        if names.insert(binding.name.clone(), false).is_some() {
            out.push(
                fail(
                    "NSS001",
                    format!("duplicate binding {:?}", binding.name),
                    &program.source_name,
                    binding.span,
                )
                .0,
            );
        }
    }
    analyze_expr(
        &program.result,
        &names,
        &function_names,
        &program.source_name,
        &mut out,
        false,
    );
    out
}
pub(crate) fn validate(program: &Program) -> Result<(), LanguageError> {
    analyze(program)
        .into_iter()
        .next()
        .map_or(Ok(()), |diagnostic| Err(LanguageError(diagnostic)))
}

/// Evaluate a valid exact-state document directly.
///
/// # Errors
///
/// Returns the first semantic or exact-value diagnostic.
pub fn interpret(program: &Program) -> Result<NativeState, LanguageError> {
    validate(program)?;
    let functions = program
        .functions
        .iter()
        .map(|function| (function.name.clone(), function))
        .collect::<BTreeMap<_, _>>();
    let mut env = BTreeMap::new();
    for binding in &program.bindings {
        env.insert(
            binding.name.clone(),
            ExactBinding::Value(evaluate(
                &binding.value,
                &env,
                &functions,
                &mut Vec::new(),
                &program.source_name,
            )?),
        );
    }
    evaluate(
        &program.result,
        &env,
        &functions,
        &mut Vec::new(),
        &program.source_name,
    )
}

/// A validated source-defined function callable with exact native states.
///
/// This is the host boundary for applying ordinary Native Space source logic
/// to runtime data. It does not assign the function name privileged semantics
/// and uses the same evaluator as a complete source document.
#[derive(Debug)]
pub struct ExactFunction<'a> {
    definition: &'a Function,
    functions: BTreeMap<String, &'a Function>,
    source_name: &'a str,
}

impl ExactFunction<'_> {
    /// Return the number of declared parameter slots.
    ///
    /// A variadic pack occupies the final declared slot. Use
    /// [`Self::minimum_parameter_count`] when preparing a call.
    #[must_use]
    pub fn parameter_count(&self) -> usize {
        self.definition.parameters.len()
    }

    /// Return the minimum accepted argument count.
    #[must_use]
    pub fn minimum_parameter_count(&self) -> usize {
        fixed_parameter_count(self.definition)
    }

    /// Return whether the final parameter accepts a variadic pack.
    #[must_use]
    pub const fn is_variadic(&self) -> bool {
        self.definition.variadic
    }

    /// Apply the source-defined function to exact native-state arguments.
    ///
    /// # Errors
    ///
    /// Returns `NSE002` for the wrong argument count or the first ordinary
    /// language diagnostic raised while evaluating the function body.
    pub fn apply(&self, arguments: &[NativeState]) -> Result<NativeState, LanguageError> {
        if !accepts_arity(self.definition, arguments.len()) {
            return Err(fail(
                "NSE002",
                format!(
                    "function {:?} expects {} arguments, found {}",
                    self.definition.name,
                    expected_arity(self.definition),
                    arguments.len()
                ),
                self.source_name,
                self.definition.span,
            ));
        }
        let environment = bind_arguments(self.definition, arguments.to_vec());
        evaluate(
            &self.definition.body,
            &environment,
            &self.functions,
            &mut vec![self.definition.name.clone()],
            self.source_name,
        )
    }
}

/// Select one source-defined function for repeated exact invocation.
///
/// # Errors
///
/// Returns `NSE001` when the function does not exist, or the first validation
/// diagnostic from the source document.
pub fn exact_function<'a>(
    program: &'a Program,
    name: &str,
) -> Result<ExactFunction<'a>, LanguageError> {
    validate(program)?;
    let functions = program
        .functions
        .iter()
        .map(|function| (function.name.clone(), function))
        .collect::<BTreeMap<_, _>>();
    let definition = functions.get(name).copied().ok_or_else(|| {
        fail(
            "NSE001",
            format!("unknown exact function {name:?}"),
            &program.source_name,
            program.span,
        )
    })?;
    Ok(ExactFunction {
        definition,
        functions,
        source_name: &program.source_name,
    })
}

pub(crate) struct UnaryFunction<'a> {
    definition: &'a Function,
    functions: BTreeMap<String, &'a Function>,
    source_name: &'a str,
}

impl UnaryFunction<'_> {
    pub(crate) fn apply(
        &self,
        mut value: NativeState,
        steps: u64,
    ) -> Result<NativeState, LanguageError> {
        let parameter = &self.definition.parameters[0];
        for _ in 0..steps {
            let environment = BTreeMap::from([(parameter.clone(), ExactBinding::Value(value))]);
            value = evaluate(
                &self.definition.body,
                &environment,
                &self.functions,
                &mut vec![self.definition.name.clone()],
                self.source_name,
            )?;
        }
        Ok(value)
    }
}

pub(crate) fn unary_function<'a>(
    program: &'a Program,
    name: &str,
) -> Result<UnaryFunction<'a>, LanguageError> {
    validate(program)?;
    let functions = program
        .functions
        .iter()
        .map(|function| (function.name.clone(), function))
        .collect::<BTreeMap<_, _>>();
    let definition = functions.get(name).copied().ok_or_else(|| {
        fail(
            "NSB001",
            format!("unknown batch function {name:?}"),
            &program.source_name,
            program.span,
        )
    })?;
    if definition.variadic || definition.parameters.len() != 1 {
        return Err(fail(
            "NSB002",
            format!(
                "batch function {name:?} must have exactly one parameter, found {}",
                definition.parameters.len()
            ),
            &program.source_name,
            definition.span,
        ));
    }
    Ok(UnaryFunction {
        definition,
        functions,
        source_name: &program.source_name,
    })
}

#[cfg(feature = "gpu")]
pub(crate) fn expanded_unary_expression(
    program: &Program,
    name: &str,
) -> Result<Expr, LanguageError> {
    let selected = unary_function(program, name)?;
    let parameter = selected.definition.parameters[0].clone();
    expand_expr(
        &selected.definition.body,
        &selected.functions,
        &BTreeMap::from([(
            parameter,
            ExprBinding::Value(Expr::Reference {
                name: "input".into(),
                span: selected.definition.span,
            }),
        )]),
        &mut vec![selected.definition.name.clone()],
        selected.source_name,
    )
}

#[derive(Clone, Debug)]
enum ExactBinding {
    Value(NativeState),
    Pack(Vec<NativeState>),
}

fn bind_arguments(function: &Function, values: Vec<NativeState>) -> BTreeMap<String, ExactBinding> {
    let fixed = fixed_parameter_count(function);
    let mut values = values.into_iter();
    let mut environment = function
        .parameters
        .iter()
        .take(fixed)
        .cloned()
        .zip(values.by_ref().take(fixed).map(ExactBinding::Value))
        .collect::<BTreeMap<_, _>>();
    if function.variadic {
        let parameter = function
            .parameters
            .last()
            .expect("validated variadic functions have a pack parameter");
        environment.insert(parameter.clone(), ExactBinding::Pack(values.collect()));
    }
    environment
}

fn evaluate_list(
    expressions: &[Expr],
    env: &BTreeMap<String, ExactBinding>,
    functions: &BTreeMap<String, &Function>,
    active: &mut Vec<String>,
    source: &str,
) -> Result<Vec<NativeState>, LanguageError> {
    let mut values = Vec::new();
    for expression in expressions {
        if let Expr::Spread { name, span } = expression {
            match env.get(name) {
                Some(ExactBinding::Pack(pack)) => values.extend(pack.iter().cloned()),
                _ => {
                    return Err(fail(
                        "NSS009",
                        format!("{name:?} is not an available variadic pack"),
                        source,
                        *span,
                    ));
                }
            }
        } else {
            values.push(evaluate(expression, env, functions, active, source)?);
        }
    }
    Ok(values)
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive expression match keeps direct denotation auditable"
)]
fn evaluate(
    expr: &Expr,
    env: &BTreeMap<String, ExactBinding>,
    functions: &BTreeMap<String, &Function>,
    active: &mut Vec<String>,
    source: &str,
) -> Result<NativeState, LanguageError> {
    match expr {
        Expr::Reflect {
            operation,
            arguments,
            span,
        } => {
            let values = evaluate_list(arguments, env, functions, active, source)?;
            crate::reflection::evaluate(*operation, &values, source, *span)
        }
        Expr::Zero { .. } => Ok(NativeState::zero()),
        Expr::One { .. } => Ok(NativeState::one()),
        Expr::Scalar { real, imag, span } => Ok(NativeState::scalar(
            NativeScalar::from_text(real, imag)
                .map_err(|message| fail("NST005", message, source, *span))?,
        )),
        Expr::Reference { name, span } => match env.get(name) {
            Some(ExactBinding::Value(value)) => Ok(value.clone()),
            Some(ExactBinding::Pack(_)) => Err(fail(
                "NSS009",
                format!("variadic parameter {name:?} must be spread as {name}..."),
                source,
                *span,
            )),
            None => Err(fail(
                "NSS002",
                format!("unknown reference {name:?}"),
                source,
                *span,
            )),
        },
        Expr::Spread { name, span } => Err(fail(
            "NSS009",
            format!("variadic parameter {name:?} must occur inside an argument or operand list"),
            source,
            *span,
        )),
        Expr::Call {
            function,
            arguments,
            span,
        } => {
            if active.iter().any(|name| name == function) {
                let mut cycle = active.clone();
                cycle.push(function.clone());
                return Err(fail(
                    "NSS007",
                    format!("cyclic function call: {}", cycle.join(" -> ")),
                    source,
                    *span,
                ));
            }
            let definition = functions.get(function).ok_or_else(|| {
                fail(
                    "NSS003",
                    format!("unknown function {function:?}"),
                    source,
                    *span,
                )
            })?;
            let values = evaluate_list(arguments, env, functions, active, source)?;
            if !accepts_arity(definition, values.len()) {
                return Err(fail(
                    "NSS004",
                    format!(
                        "function {function:?} expects {} arguments, found {}",
                        expected_arity(definition),
                        values.len()
                    ),
                    source,
                    *span,
                ));
            }
            let local = bind_arguments(definition, values);
            active.push(function.clone());
            let result = evaluate(&definition.body, &local, functions, active, source);
            active.pop();
            result
        }
        Expr::Trace { function, span } => {
            let strand =
                crate::strand::operation_strand(function, functions, source, source, *span)?;
            evaluate(&strand, env, functions, active, source)
        }
        Expr::Length { value, span } => {
            let strand = evaluate(value, env, functions, active, source)?;
            let length = crate::strand::operation_length(&strand, source, *span)?;
            Ok(NativeState::one()
                .index_power(1, length)
                .expect("the length camera uses a positive direction"))
        }
        Expr::Untrace { value, rank, span } => {
            let state = evaluate(value, env, functions, active, source)?;
            if crate::strand::is_operation_strand(&state) {
                return crate::strand::optimize_operation_strand(&state, rank, source, *span)?
                    .map_or(Ok(state), |candidate| {
                        evaluate(&candidate, env, functions, active, source)
                    });
            }
            let pattern = crate::discovery::discover(&state, rank, source, *span)?;
            let expression = pattern.expression(source, *span)?;
            evaluate(&expression, env, functions, active, source)
        }
        Expr::RankDescent {
            value,
            target_rank,
            minimum_agreement,
            span,
        } => {
            let state = evaluate(value, env, functions, active, source)?;
            let search = rank_descent_search(
                &state,
                target_rank.as_deref(),
                minimum_agreement.as_deref(),
                source,
                *span,
            )?;
            let expression = search.final_pattern().expression(source, *span)?;
            evaluate(&expression, env, functions, active, source)
        }
        Expr::Apply {
            pattern,
            position,
            span,
        } => {
            let Expr::RankDescent {
                value,
                target_rank,
                minimum_agreement,
                ..
            } = pattern.as_ref()
            else {
                return Err(fail(
                    "NSE003",
                    "apply expects rank_descent(...) as its first argument",
                    source,
                    pattern.span(),
                ));
            };
            let state = evaluate(value, env, functions, active, source)?;
            let search = rank_descent_search(
                &state,
                target_rank.as_deref(),
                minimum_agreement.as_deref(),
                source,
                *span,
            )?;
            crate::rank_descent::replay_at(search.final_pattern(), *position, source).map_err(
                |mut error| {
                    if error.0.span.is_none() {
                        error.0.span = *span;
                    }
                    error
                },
            )
        }
        Expr::Concat {
            direction,
            values,
            span,
        } => {
            let values = evaluate_list(values, env, functions, active, source)?;
            if values.is_empty() {
                return Err(fail(
                    "NST007",
                    "concat requires at least one value after pack expansion",
                    source,
                    *span,
                ));
            }
            let mut out = NativeState::zero();
            for (position, value) in values.into_iter().enumerate() {
                let depth = u64::try_from(position + 1).map_err(|_capacity_error| {
                    fail(
                        "NST007",
                        "concat has too many values to represent as INDEX depth",
                        source,
                        *span,
                    )
                })?;
                out = out.add(
                    &value
                        .index_power(*direction, depth)
                        .map_err(|message| fail("NST007", message, source, *span))?,
                );
            }
            Ok(out)
        }
        Expr::Fold {
            function,
            initial,
            values,
            span,
        } => {
            let definition = functions.get(function).ok_or_else(|| {
                fail(
                    "NSS003",
                    format!("unknown function {function:?}"),
                    source,
                    *span,
                )
            })?;
            if definition.variadic || definition.parameters.len() != 2 {
                return Err(fail(
                    "NSS010",
                    format!("fold function {function:?} must accept exactly two parameters"),
                    source,
                    *span,
                ));
            }
            let mut accumulator = evaluate(initial, env, functions, active, source)?;
            for value in evaluate_list(values, env, functions, active, source)? {
                let local = bind_arguments(definition, vec![accumulator, value]);
                active.push(function.clone());
                let result = evaluate(&definition.body, &local, functions, active, source);
                active.pop();
                accumulator = result?;
            }
            Ok(accumulator)
        }
        Expr::Camera {
            from_direction,
            to_direction,
            value,
            ..
        } => {
            Ok(evaluate(value, env, functions, active, source)?
                .camera(*from_direction, *to_direction))
        }
        Expr::Add { operands, .. } => {
            let mut out = NativeState::zero();
            for value in evaluate_list(operands, env, functions, active, source)? {
                out = out.add(&value);
            }
            Ok(out)
        }
        Expr::Multiply { operands, .. } => {
            let mut out = NativeState::one();
            for value in evaluate_list(operands, env, functions, active, source)? {
                out = out.multiply(&value);
            }
            Ok(out)
        }
        Expr::Orient { turns, value, .. } => {
            Ok(evaluate(value, env, functions, active, source)?.orient(*turns))
        }
        Expr::Index {
            direction,
            multiplicity,
            value,
            span,
        } => evaluate(value, env, functions, active, source)?
            .index_power(*direction, *multiplicity)
            .map_err(|message| fail("NST003", message, source, *span)),
    }
}

fn rank_descent_search(
    state: &NativeState,
    target_rank: Option<&str>,
    minimum_agreement: Option<&str>,
    source: &str,
    span: Option<Span>,
) -> Result<crate::rank_descent::RankDescent, LanguageError> {
    let (_first_index, values) = crate::continuation::indexed_observations(state, source, span)?;
    let strategy = target_rank.map_or_else(crate::rank_descent::RankStrategy::adaptive, |rank| {
        crate::rank_descent::RankStrategy::static_target(rank, minimum_agreement.unwrap_or("1"))
    });
    crate::rank_descent::descend_states(&values, values.len(), &strategy, source).map_err(
        |mut error| {
            if error.0.span.is_none() {
                error.0.span = span;
            }
            error
        },
    )
}

pub(crate) fn expand_functions(program: &Program) -> Result<Program, LanguageError> {
    validate(program)?;
    let functions = program
        .functions
        .iter()
        .map(|function| (function.name.clone(), function))
        .collect::<BTreeMap<_, _>>();
    let bindings = program
        .bindings
        .iter()
        .map(|binding| {
            Ok(Binding {
                name: binding.name.clone(),
                span: binding.span,
                value: expand_expr(
                    &binding.value,
                    &functions,
                    &BTreeMap::new(),
                    &mut Vec::new(),
                    &program.source_name,
                )?,
            })
        })
        .collect::<Result<Vec<_>, LanguageError>>()?;
    let result = expand_expr(
        &program.result,
        &functions,
        &BTreeMap::new(),
        &mut Vec::new(),
        &program.source_name,
    )?;
    let expanded = Program {
        functions: Vec::new(),
        bindings,
        goal: program.goal,
        output_kind: program.output_kind,
        result,
        source_name: program.source_name.clone(),
        span: program.span,
    };
    lower_reflective_expressions(&expanded)
}

pub(crate) fn expanded_source(program: &Program) -> Result<String, LanguageError> {
    let expanded = expand_functions(program)?;
    let mut lines = expanded
        .bindings
        .iter()
        .map(|binding| {
            format!(
                "let {} = {}",
                binding.name,
                expression_source(&binding.value)
            )
        })
        .collect::<Vec<_>>();
    let result = expression_source(&expanded.result);
    lines.push(match expanded.goal {
        Goal::ProveZero => format!("{result} = 0"),
        Goal::Emit => {
            let camera = match expanded.output_kind {
                OutputKind::Auto => "",
                OutputKind::String => " as string",
                OutputKind::Number => " as number",
                OutputKind::Vector => " as vector",
                OutputKind::Pattern => " as pattern",
                OutputKind::Boolean => " as boolean",
            };
            format!("output {result}{camera}")
        }
    });
    Ok(lines.join("\n"))
}

pub(crate) fn expression_source(expression: &Expr) -> String {
    match expression {
        Expr::Reflect {
            operation,
            arguments,
            ..
        } => format!(
            "{}({})",
            operation.name(),
            arguments
                .iter()
                .map(expression_source)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Zero { .. } => "zero".into(),
        Expr::One { .. } => "one".into(),
        Expr::Scalar { real, imag, .. } if imag == "0" => real.clone(),
        Expr::Scalar { real, imag, .. } => format!("scalar({real}, {imag})"),
        Expr::Reference { name, .. } => name.clone(),
        Expr::Spread { name, .. } => format!("{name}..."),
        Expr::Call {
            function,
            arguments,
            ..
        } => format!("{function}({})", expression_list_source(arguments)),
        Expr::Concat {
            direction, values, ..
        } => format!("concat({direction}, {})", expression_list_source(values)),
        Expr::Fold {
            function,
            initial,
            values,
            ..
        } => format!(
            "fold({function}, {}, {})",
            expression_source(initial),
            expression_list_source(values)
        ),
        Expr::Camera {
            from_direction,
            to_direction,
            value,
            ..
        } => format!(
            "camera({from_direction}, {to_direction}, {})",
            expression_source(value)
        ),
        Expr::Trace { function, .. } => format!("trace({function})"),
        Expr::Length { value, .. } => format!("length({})", expression_source(value)),
        Expr::Untrace { value, rank, .. } if rank == "1" => {
            format!("untrace({})", expression_source(value))
        }
        Expr::Untrace { value, rank, .. } => {
            format!("untrace({}, {rank})", expression_source(value))
        }
        Expr::RankDescent {
            value,
            target_rank,
            minimum_agreement,
            ..
        } => rank_descent_source(value, target_rank.as_deref(), minimum_agreement.as_deref()),
        Expr::Apply {
            pattern, position, ..
        } => apply_source(pattern, *position),
        Expr::Add { operands, .. } => format!("add({})", expression_list_source(operands)),
        Expr::Multiply { operands, .. } => {
            format!("multiply({})", expression_list_source(operands))
        }
        Expr::Orient { turns, value, .. } => {
            format!("orient({turns}, {})", expression_source(value))
        }
        Expr::Index {
            direction,
            multiplicity,
            value,
            ..
        } => {
            if *multiplicity == 1 {
                format!("index({direction}, {})", expression_source(value))
            } else if *multiplicity == 0 {
                expression_source(value)
            } else {
                format!(
                    "index({direction}, {}, {multiplicity})",
                    expression_source(value)
                )
            }
        }
    }
}

fn expression_list_source(expressions: &[Expr]) -> String {
    expressions
        .iter()
        .map(expression_source)
        .collect::<Vec<_>>()
        .join(", ")
}

fn rank_descent_source(
    value: &Expr,
    target_rank: Option<&str>,
    minimum_agreement: Option<&str>,
) -> String {
    match (target_rank, minimum_agreement) {
        (None, None) => format!("rank_descent({})", expression_source(value)),
        (Some(rank), None) => format!("rank_descent({}, {rank})", expression_source(value)),
        (Some(rank), Some(agreement)) => format!(
            "rank_descent({}, {rank}, {agreement})",
            expression_source(value)
        ),
        (None, Some(_)) => unreachable!("agreement is parsed only after a target rank"),
    }
}

fn apply_source(pattern: &Expr, position: u64) -> String {
    format!("apply({}, {position})", expression_source(pattern))
}

#[derive(Clone, Debug)]
enum ExprBinding {
    Value(Expr),
    Pack(Vec<Expr>),
}

fn bind_expressions(function: &Function, values: Vec<Expr>) -> BTreeMap<String, ExprBinding> {
    let fixed = fixed_parameter_count(function);
    let mut values = values.into_iter();
    let mut bindings = function
        .parameters
        .iter()
        .take(fixed)
        .cloned()
        .zip(values.by_ref().take(fixed).map(ExprBinding::Value))
        .collect::<BTreeMap<_, _>>();
    if function.variadic {
        let parameter = function
            .parameters
            .last()
            .expect("validated variadic functions have a pack parameter");
        bindings.insert(parameter.clone(), ExprBinding::Pack(values.collect()));
    }
    bindings
}

fn expand_list(
    expressions: &[Expr],
    functions: &BTreeMap<String, &Function>,
    parameters: &BTreeMap<String, ExprBinding>,
    active: &mut Vec<String>,
    source: &str,
) -> Result<Vec<Expr>, LanguageError> {
    let mut expanded = Vec::new();
    for expression in expressions {
        if let Expr::Spread { name, span } = expression {
            match parameters.get(name) {
                Some(ExprBinding::Pack(pack)) => expanded.extend(pack.iter().cloned()),
                _ => {
                    return Err(fail(
                        "NSS009",
                        format!("{name:?} is not an available variadic pack"),
                        source,
                        *span,
                    ));
                }
            }
        } else {
            expanded.push(expand_expr(
                expression, functions, parameters, active, source,
            )?);
        }
    }
    Ok(expanded)
}

fn fold_expression(additive: bool, mut values: Vec<Expr>, span: Option<Span>) -> Expr {
    match values.len() {
        0 if additive => Expr::Zero { span },
        0 => Expr::One { span },
        1 => values.remove(0),
        _ if additive => Expr::Add {
            operands: values,
            span,
        },
        _ => Expr::Multiply {
            operands: values,
            span,
        },
    }
}

fn concat_expression(
    direction: u64,
    values: Vec<Expr>,
    span: Option<Span>,
    source: &str,
) -> Result<Expr, LanguageError> {
    if values.is_empty() {
        return Err(fail(
            "NST007",
            "concat requires at least one value after pack expansion",
            source,
            span,
        ));
    }
    let indexed = values
        .into_iter()
        .enumerate()
        .map(|(position, value)| {
            let multiplicity = u64::try_from(position + 1).map_err(|_capacity_error| {
                fail(
                    "NST007",
                    "concat has too many values to represent as INDEX depth",
                    source,
                    span,
                )
            })?;
            Ok(Expr::Index {
                direction,
                multiplicity,
                value: Box::new(value),
                span,
            })
        })
        .collect::<Result<Vec<_>, LanguageError>>()?;
    Ok(fold_expression(true, indexed, span))
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive expression match keeps source elaboration auditable"
)]
fn expand_expr(
    expr: &Expr,
    functions: &BTreeMap<String, &Function>,
    parameters: &BTreeMap<String, ExprBinding>,
    active: &mut Vec<String>,
    source: &str,
) -> Result<Expr, LanguageError> {
    match expr {
        Expr::Reflect {
            operation,
            arguments,
            span,
        } => Ok(Expr::Reflect {
            operation: *operation,
            arguments: expand_list(arguments, functions, parameters, active, source)?,
            span: *span,
        }),
        Expr::Reference { name, span } if parameters.contains_key(name) => {
            match &parameters[name] {
                ExprBinding::Value(value) => Ok(value.clone()),
                ExprBinding::Pack(_) => Err(fail(
                    "NSS009",
                    format!("variadic parameter {name:?} must be spread as {name}..."),
                    source,
                    *span,
                )),
            }
        }
        Expr::Spread { name, span } => Err(fail(
            "NSS009",
            format!("variadic parameter {name:?} must occur inside an argument or operand list"),
            source,
            *span,
        )),
        Expr::Call {
            function,
            arguments,
            span,
        } => {
            if active.iter().any(|name| name == function) {
                let mut cycle = active.clone();
                cycle.push(function.clone());
                return Err(fail(
                    "NSS007",
                    format!("cyclic function call: {}", cycle.join(" -> ")),
                    source,
                    *span,
                ));
            }
            let definition = functions.get(function).ok_or_else(|| {
                fail(
                    "NSS003",
                    format!("unknown function {function:?}"),
                    source,
                    *span,
                )
            })?;
            let arguments = expand_list(arguments, functions, parameters, active, source)?;
            if !accepts_arity(definition, arguments.len()) {
                return Err(fail(
                    "NSS004",
                    format!(
                        "function {function:?} expects {} arguments, found {}",
                        expected_arity(definition),
                        arguments.len()
                    ),
                    source,
                    *span,
                ));
            }
            let local = bind_expressions(definition, arguments);
            active.push(function.clone());
            let result = expand_expr(&definition.body, functions, &local, active, source);
            active.pop();
            result
        }
        Expr::Trace { function, span } => {
            crate::strand::operation_strand(function, functions, source, source, *span)
        }
        Expr::Length { value, span } => Ok(Expr::Length {
            value: Box::new(expand_expr(value, functions, parameters, active, source)?),
            span: *span,
        }),
        Expr::Untrace { value, rank, span } => Ok(Expr::Untrace {
            value: Box::new(expand_expr(value, functions, parameters, active, source)?),
            rank: rank.clone(),
            span: *span,
        }),
        Expr::RankDescent {
            value,
            target_rank,
            minimum_agreement,
            span,
        } => Ok(Expr::RankDescent {
            value: Box::new(expand_expr(value, functions, parameters, active, source)?),
            target_rank: target_rank.clone(),
            minimum_agreement: minimum_agreement.clone(),
            span: *span,
        }),
        Expr::Apply {
            pattern,
            position,
            span,
        } => Ok(Expr::Apply {
            pattern: Box::new(expand_expr(pattern, functions, parameters, active, source)?),
            position: *position,
            span: *span,
        }),
        Expr::Concat {
            direction,
            values,
            span,
        } => concat_expression(
            *direction,
            expand_list(values, functions, parameters, active, source)?,
            *span,
            source,
        ),
        Expr::Fold {
            function,
            initial,
            values,
            span,
        } => {
            let definition = functions.get(function).ok_or_else(|| {
                fail(
                    "NSS003",
                    format!("unknown function {function:?}"),
                    source,
                    *span,
                )
            })?;
            if definition.variadic || definition.parameters.len() != 2 {
                return Err(fail(
                    "NSS010",
                    format!("fold function {function:?} must accept exactly two parameters"),
                    source,
                    *span,
                ));
            }
            let mut accumulator = expand_expr(initial, functions, parameters, active, source)?;
            for value in expand_list(values, functions, parameters, active, source)? {
                let local = bind_expressions(definition, vec![accumulator, value]);
                active.push(function.clone());
                let result = expand_expr(&definition.body, functions, &local, active, source);
                active.pop();
                accumulator = result?;
            }
            Ok(accumulator)
        }
        Expr::Camera {
            from_direction,
            to_direction,
            value,
            span,
        } => Ok(Expr::Camera {
            from_direction: *from_direction,
            to_direction: *to_direction,
            value: Box::new(expand_expr(value, functions, parameters, active, source)?),
            span: *span,
        }),
        Expr::Add { operands, span } => Ok(fold_expression(
            true,
            expand_list(operands, functions, parameters, active, source)?,
            *span,
        )),
        Expr::Multiply { operands, span } => Ok(fold_expression(
            false,
            expand_list(operands, functions, parameters, active, source)?,
            *span,
        )),
        Expr::Orient { turns, value, span } => Ok(Expr::Orient {
            turns: *turns,
            value: Box::new(expand_expr(value, functions, parameters, active, source)?),
            span: *span,
        }),
        Expr::Index {
            direction,
            multiplicity,
            value,
            span,
        } => Ok(Expr::Index {
            direction: *direction,
            multiplicity: *multiplicity,
            value: Box::new(expand_expr(value, functions, parameters, active, source)?),
            span: *span,
        }),
        _ => Ok(expr.clone()),
    }
}

fn lower_reflective_expressions(program: &Program) -> Result<Program, LanguageError> {
    let functions = BTreeMap::new();
    let mut env = BTreeMap::new();
    let mut bindings = Vec::with_capacity(program.bindings.len());
    for binding in &program.bindings {
        let value = lower_reflective_expr(&binding.value, &env, &program.source_name)?;
        let state = evaluate(
            &value,
            &env,
            &functions,
            &mut Vec::new(),
            &program.source_name,
        )?;
        env.insert(binding.name.clone(), ExactBinding::Value(state));
        bindings.push(Binding {
            name: binding.name.clone(),
            span: binding.span,
            value,
        });
    }
    let result = lower_reflective_expr(&program.result, &env, &program.source_name)?;
    Ok(Program {
        functions: Vec::new(),
        bindings,
        goal: program.goal,
        output_kind: program.output_kind,
        result,
        source_name: program.source_name.clone(),
        span: program.span,
    })
}

fn lower_graph_operation(
    operation: crate::reflection::Operation,
    arguments: &[Expr],
    span: Option<Span>,
    env: &BTreeMap<String, ExactBinding>,
    source: &str,
) -> Result<Expr, LanguageError> {
    let arguments = arguments
        .iter()
        .map(|argument| lower_reflective_expr(argument, env, source))
        .collect::<Result<Vec<_>, _>>()?;
    let graph = arguments
        .first()
        .ok_or_else(|| fail("NSR001", "reflection requires a graph", source, span))?;
    let state = evaluate(graph, env, &BTreeMap::new(), &mut Vec::new(), source)?;
    if operation == crate::reflection::Operation::Apply {
        let (functions, root) = crate::reflection::application_graph(&state, source, span)?;
        let catalog = functions.iter().map(|f| (f.name.clone(), f)).collect();
        let call = Expr::Call {
            function: root,
            arguments: arguments[1..].to_vec(),
            span,
        };
        let expanded = expand_expr(&call, &catalog, &BTreeMap::new(), &mut Vec::new(), source)?;
        lower_reflective_expr(&expanded, env, source)
    } else {
        let values = arguments
            .iter()
            .map(|argument| evaluate(argument, env, &BTreeMap::new(), &mut Vec::new(), source))
            .collect::<Result<Vec<_>, _>>()?;
        crate::reflection::rewrite(&values, source, span)
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive staging match keeps core lowering behavior auditable"
)]
fn lower_reflective_expr(
    expr: &Expr,
    env: &BTreeMap<String, ExactBinding>,
    source: &str,
) -> Result<Expr, LanguageError> {
    match expr {
        Expr::Reflect {
            operation,
            arguments,
            span,
        } => lower_graph_operation(*operation, arguments, *span, env, source),
        Expr::Length { value, span } => {
            let value = lower_reflective_expr(value, env, source)?;
            let strand = evaluate(&value, env, &BTreeMap::new(), &mut Vec::new(), source)?;
            let length = crate::strand::operation_length(&strand, source, *span)?;
            Ok(Expr::Index {
                direction: 1,
                multiplicity: length,
                value: Box::new(Expr::One { span: None }),
                span: *span,
            })
        }
        Expr::Untrace { value, rank, .. } => {
            let value = lower_reflective_expr(value, env, source)?;
            let state = evaluate(&value, env, &BTreeMap::new(), &mut Vec::new(), source)?;
            if crate::strand::is_operation_strand(&state) {
                let candidate =
                    crate::strand::optimize_operation_strand(&state, rank, source, expr.span())?;
                // Both expressions are already canonical core-operation
                // strands. Preserving their nested form avoids expanding a
                // large graph into a recursion-heavy flat state expression.
                return Ok(candidate.unwrap_or(value));
            }
            let pattern = crate::discovery::discover(&state, rank, source, expr.span())?;
            let expression = pattern.expression(source, expr.span())?;
            lower_reflective_expr(&expression, env, source)
        }
        Expr::RankDescent {
            value,
            target_rank,
            minimum_agreement,
            ..
        } => {
            let value = lower_reflective_expr(value, env, source)?;
            let state = evaluate(&value, env, &BTreeMap::new(), &mut Vec::new(), source)?;
            let search = rank_descent_search(
                &state,
                target_rank.as_deref(),
                minimum_agreement.as_deref(),
                source,
                expr.span(),
            )?;
            let expression = search.final_pattern().expression(source, expr.span())?;
            lower_reflective_expr(&expression, env, source)
        }
        Expr::Apply {
            pattern, position, ..
        } => lower_pattern_apply(pattern, *position, env, source, expr.span()),
        Expr::Add { operands, span } => Ok(Expr::Add {
            operands: operands
                .iter()
                .map(|operand| lower_reflective_expr(operand, env, source))
                .collect::<Result<Vec<_>, _>>()?,
            span: *span,
        }),
        Expr::Multiply { operands, span } => Ok(Expr::Multiply {
            operands: operands
                .iter()
                .map(|operand| lower_reflective_expr(operand, env, source))
                .collect::<Result<Vec<_>, _>>()?,
            span: *span,
        }),
        Expr::Orient { turns, value, span } => Ok(Expr::Orient {
            turns: *turns,
            value: Box::new(lower_reflective_expr(value, env, source)?),
            span: *span,
        }),
        Expr::Index {
            direction,
            multiplicity,
            value,
            span,
        } => Ok(Expr::Index {
            direction: *direction,
            multiplicity: *multiplicity,
            value: Box::new(lower_reflective_expr(value, env, source)?),
            span: *span,
        }),
        Expr::Camera {
            from_direction,
            to_direction,
            value,
            ..
        } => {
            let value = lower_reflective_expr(value, env, source)?;
            let state = evaluate(&value, env, &BTreeMap::new(), &mut Vec::new(), source)?;
            Ok(state_expression(
                &state.camera(*from_direction, *to_direction),
            ))
        }
        Expr::Call { .. }
        | Expr::Concat { .. }
        | Expr::Fold { .. }
        | Expr::Spread { .. }
        | Expr::Trace { .. } => {
            unreachable!("calls, packs, concat, fold, and trace are lowered before staging")
        }
        _ => Ok(expr.clone()),
    }
}

fn lower_pattern_apply(
    pattern: &Expr,
    position: u64,
    env: &BTreeMap<String, ExactBinding>,
    source: &str,
    span: Option<Span>,
) -> Result<Expr, LanguageError> {
    let Expr::RankDescent {
        value,
        target_rank,
        minimum_agreement,
        ..
    } = pattern
    else {
        return Err(fail(
            "NSE003",
            "apply expects rank_descent(...) as its first argument",
            source,
            pattern.span(),
        ));
    };
    let value = lower_reflective_expr(value, env, source)?;
    let state = evaluate(&value, env, &BTreeMap::new(), &mut Vec::new(), source)?;
    let search = rank_descent_search(
        &state,
        target_rank.as_deref(),
        minimum_agreement.as_deref(),
        source,
        span,
    )?;
    let generated = crate::rank_descent::replay_at(search.final_pattern(), position, source)
        .map_err(|mut error| {
            if error.0.span.is_none() {
                error.0.span = span;
            }
            error
        })?;
    Ok(state_expression(&generated))
}

pub(crate) fn state_expression(state: &NativeState) -> Expr {
    let mut terms = state
        .0
        .iter()
        .map(|(index, coefficient)| {
            let mut value = if *coefficient == NativeScalar::one() {
                Expr::One { span: None }
            } else {
                Expr::Scalar {
                    real: rational_text(&coefficient.real),
                    imag: rational_text(&coefficient.imag),
                    span: None,
                }
            };
            for (&direction, depth) in &index.0 {
                let mut remaining = depth.clone();
                while !remaining.is_zero() {
                    let multiplicity = remaining.to_u64().unwrap_or(u64::MAX);
                    remaining -= BigUint::from(multiplicity);
                    value = Expr::Index {
                        direction,
                        multiplicity,
                        value: Box::new(value),
                        span: None,
                    };
                }
            }
            value
        })
        .collect::<Vec<_>>();
    match terms.len() {
        0 => Expr::Zero { span: None },
        1 => terms.remove(0),
        _ => Expr::Add {
            operands: terms,
            span: None,
        },
    }
}

#[must_use]
pub fn program_to_data(program: &Program) -> Value {
    json!({"schema":"native-space-ast","version":AST_SCHEMA_VERSION,"source_name":program.source_name,"span":program.span,"functions":program.functions,"bindings":program.bindings,"goal":program.goal,"output_kind":program.output_kind,"result":program.result})
}
/// Decode the schema-1 exact-state AST.
///
/// # Errors
///
/// Returns an error for a malformed object or unsupported schema.
pub fn program_from_data(data: &Value) -> Result<Program, String> {
    let root = data.as_object().ok_or("AST root must be an object")?;
    if root.get("schema").and_then(Value::as_str) != Some("native-space-ast")
        || root.get("version").and_then(Value::as_u64) != Some(AST_SCHEMA_VERSION)
    {
        return Err("unsupported Native Space AST schema or version".into());
    }
    Ok(Program {
        source_name: root
            .get("source_name")
            .and_then(Value::as_str)
            .unwrap_or("<json>")
            .into(),
        span: serde_json::from_value(root.get("span").cloned().unwrap_or(Value::Null))
            .map_err(|e| e.to_string())?,
        functions: serde_json::from_value(
            root.get("functions")
                .cloned()
                .ok_or("functions are required")?,
        )
        .map_err(|e| e.to_string())?,
        bindings: serde_json::from_value(
            root.get("bindings")
                .cloned()
                .ok_or("bindings are required")?,
        )
        .map_err(|e| e.to_string())?,
        goal: serde_json::from_value(root.get("goal").cloned().ok_or("goal is required")?)
            .map_err(|e| e.to_string())?,
        output_kind: serde_json::from_value(
            root.get("output_kind")
                .cloned()
                .ok_or("output_kind is required")?,
        )
        .map_err(|e| e.to_string())?,
        result: serde_json::from_value(root.get("result").cloned().ok_or("result is required")?)
            .map_err(|e| e.to_string())?,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RewriteEvent {
    pub rule_id: String,
    pub theorem_ids: Vec<String>,
    pub span: Option<Span>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationResult {
    pub program: Program,
    pub events: Vec<RewriteEvent>,
}
/// Apply only theorem-authorized finite AST rewrites.
///
/// # Errors
///
/// Returns the first name-analysis diagnostic.
pub fn optimize(program: &Program) -> Result<OptimizationResult, LanguageError> {
    validate(program)?;
    let mut events = Vec::new();
    let bindings = program
        .bindings
        .iter()
        .map(|b| Binding {
            name: b.name.clone(),
            span: b.span,
            value: optimize_expr(&b.value, &mut events),
        })
        .collect();
    let result = optimize_expr(&program.result, &mut events);
    Ok(OptimizationResult {
        program: Program {
            functions: program
                .functions
                .iter()
                .map(|function| Function {
                    name: function.name.clone(),
                    parameters: function.parameters.clone(),
                    variadic: function.variadic,
                    body: optimize_expr(&function.body, &mut events),
                    span: function.span,
                })
                .collect(),
            bindings,
            goal: program.goal,
            output_kind: program.output_kind,
            result,
            source_name: program.source_name.clone(),
            span: program.span,
        },
        events,
    })
}
fn event(events: &mut Vec<RewriteEvent>, rule: &str, theorems: &[&str], span: Option<Span>) {
    events.push(RewriteEvent {
        rule_id: rule.into(),
        theorem_ids: theorems.iter().map(|x| (*x).into()).collect(),
        span,
    });
}
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive match keeps every authorized rewrite visible"
)]
fn optimize_expr(expr: &Expr, events: &mut Vec<RewriteEvent>) -> Expr {
    // Reflection arguments describe syntax. Ordinary value rewrites must not
    // alter their rule templates before the explicit reflection stage.
    if matches!(expr, Expr::Reflect { .. }) {
        return expr.clone();
    }
    match expr {
        Expr::Add { operands, span } => {
            let optimized: Vec<_> = operands.iter().map(|x| optimize_expr(x, events)).collect();
            let mut flat = Vec::new();
            let mut flattened = false;
            for item in optimized {
                if let Expr::Add { operands, .. } = item {
                    flat.extend(operands);
                    flattened = true;
                } else {
                    flat.push(item);
                }
            }
            if flattened {
                event(events, "OPT-ADD-FLATTEN-1", &["L-NS-2"], *span);
            }
            let before = flat.len();
            flat.retain(|x| !matches!(x, Expr::Zero { .. }));
            if flat.len() != before {
                event(events, "OPT-ADD-ZERO-1", &["L-NS-2"], *span);
            }
            match flat.len() {
                0 => Expr::Zero { span: *span },
                1 => flat.remove(0),
                _ => Expr::Add {
                    operands: flat,
                    span: *span,
                },
            }
        }
        Expr::Multiply { operands, span } => {
            let optimized: Vec<_> = operands.iter().map(|x| optimize_expr(x, events)).collect();
            if optimized.iter().any(|x| matches!(x, Expr::Zero { .. })) {
                event(events, "OPT-MUL-ZERO-1", &["L-NS-8"], *span);
                return Expr::Zero { span: *span };
            }
            let mut flat = Vec::new();
            let mut flattened = false;
            for item in optimized {
                if let Expr::Multiply { operands, .. } = item {
                    flat.extend(operands);
                    flattened = true;
                } else {
                    flat.push(item);
                }
            }
            if flattened {
                event(events, "OPT-MUL-FLATTEN-1", &["L-NS-5"], *span);
            }
            let before = flat.len();
            flat.retain(|x| !matches!(x, Expr::One { .. }));
            if flat.len() != before {
                event(events, "OPT-MUL-ONE-1", &["L-NS-6"], *span);
            }
            match flat.len() {
                0 => Expr::One { span: *span },
                1 => flat.remove(0),
                _ => Expr::Multiply {
                    operands: flat,
                    span: *span,
                },
            }
        }
        Expr::Orient { turns, value, span } => {
            let value = optimize_expr(value, events);
            if *turns == 0 {
                event(events, "OPT-ORIENT-IDENTITY-1", &["L-SEP-5"], *span);
                return value;
            }
            if let Expr::Orient {
                turns: inner,
                value: inner_value,
                ..
            } = value
            {
                event(events, "OPT-ORIENT-COMBINE-1", &["L-SEP-5"], *span);
                let total = *turns + inner;
                let combined = total.rem_euclid(MAX_CANONICAL_ORIENTATION + 1);
                if combined != total {
                    event(events, "OPT-ORIENT-NORMALIZE-1", &["L-SEP-5"], *span);
                }
                if combined == 0 {
                    event(events, "OPT-ORIENT-IDENTITY-1", &["L-SEP-5"], *span);
                    *inner_value
                } else {
                    Expr::Orient {
                        turns: combined,
                        value: inner_value,
                        span: *span,
                    }
                }
            } else {
                Expr::Orient {
                    turns: *turns,
                    value: Box::new(value),
                    span: *span,
                }
            }
        }
        Expr::Index {
            direction,
            multiplicity,
            value,
            span,
        } => Expr::Index {
            direction: *direction,
            multiplicity: *multiplicity,
            value: Box::new(optimize_expr(value, events)),
            span: *span,
        },
        Expr::Fold {
            function,
            initial,
            values,
            span,
        } => Expr::Fold {
            function: function.clone(),
            initial: Box::new(optimize_expr(initial, events)),
            values: values
                .iter()
                .map(|value| optimize_expr(value, events))
                .collect(),
            span: *span,
        },
        Expr::Camera {
            from_direction,
            to_direction,
            value,
            span,
        } => Expr::Camera {
            from_direction: *from_direction,
            to_direction: *to_direction,
            value: Box::new(optimize_expr(value, events)),
            span: *span,
        },
        Expr::Concat {
            direction,
            values,
            span,
        } => Expr::Concat {
            direction: *direction,
            values: values
                .iter()
                .map(|value| optimize_expr(value, events))
                .collect(),
            span: *span,
        },
        Expr::Call {
            function,
            arguments,
            span,
        } => Expr::Call {
            function: function.clone(),
            arguments: arguments
                .iter()
                .map(|argument| optimize_expr(argument, events))
                .collect(),
            span: *span,
        },
        Expr::Trace { function, span } => Expr::Trace {
            function: function.clone(),
            span: *span,
        },
        Expr::Length { value, span } => Expr::Length {
            value: Box::new(optimize_expr(value, events)),
            span: *span,
        },
        Expr::Untrace { value, rank, span } => Expr::Untrace {
            value: Box::new(optimize_expr(value, events)),
            rank: rank.clone(),
            span: *span,
        },
        Expr::RankDescent {
            value,
            target_rank,
            minimum_agreement,
            span,
        } => Expr::RankDescent {
            value: Box::new(optimize_expr(value, events)),
            target_rank: target_rank.clone(),
            minimum_agreement: minimum_agreement.clone(),
            span: *span,
        },
        Expr::Apply {
            pattern,
            position,
            span,
        } => Expr::Apply {
            pattern: Box::new(optimize_expr(pattern, events)),
            position: *position,
            span: *span,
        },
        _ => expr.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "orientation turns must be from zero through three")]
    fn scalar_orientation_does_not_silently_normalize_invalid_api_input() {
        let _ = NativeScalar::one().orient(4);
    }

    #[test]
    fn every_language_namespace_entry_is_reserved_for_operators() {
        for (name, _) in LANGUAGE_NAMESPACE {
            let source = format!("operator {name:?} = (left, right) => left\noutput 1");
            let error = parse(&source, "namespace.ns").unwrap_err();
            assert_eq!(error.0.code, "NSS008", "{name}");
        }
    }
    #[test]
    fn exact_core_and_parser() {
        let program = parse(
            "let x = index(1, scalar(2, 0))\noutput add(x, orient(2, x))",
            "core.ns",
        )
        .unwrap();
        assert!(interpret(&program).unwrap().is_zero());
        assert_eq!(
            program_from_data(&program_to_data(&program)).unwrap(),
            program
        );
    }

    #[test]
    fn index_depth_accumulation_is_unbounded_and_exact() {
        let maximum = MultiIndex::from_depths([(1, u64::MAX)]).unwrap();
        let shifted = maximum.shift_by(1, 1).unwrap();
        assert_eq!(shifted.depth(1), BigUint::from(u64::MAX) + BigUint::one());
        assert_eq!(shifted.total_depth(), shifted.depth(1));
    }

    #[test]
    fn runtime_state_data_round_trips_and_canonicalizes() {
        let index = MultiIndex::from_depths([(3, u64::MAX)])
            .unwrap()
            .shift_by(3, 1)
            .unwrap();
        let state =
            NativeState::from_terms([(index, NativeScalar::from_text("7/3", "-2").unwrap())]);
        assert_eq!(NativeState::from_data(&state.to_data()).unwrap(), state);

        let duplicate_data = json!({
            "camera": FLAT_STACK_CAMERA,
            "terms": [
                {
                    "index": [
                        {"direction": 1, "depth": "1"},
                        {"direction": 1, "depth": "2"}
                    ],
                    "coefficient": {"real": "1", "imag": "0"}
                },
                {
                    "index": [{"direction": 1, "depth": "3"}],
                    "coefficient": {"real": "-1", "imag": "0"}
                },
                {
                    "index": [{"direction": 4, "depth": "1"}],
                    "coefficient": {"real": "0", "imag": "0"}
                }
            ]
        });
        assert!(NativeState::from_data(&duplicate_data).unwrap().is_zero());
        NativeState::from_data(&json!({"terms": []})).unwrap_err();
        NativeState::from_data(&json!({"camera": "other", "terms": []})).unwrap_err();
    }

    #[test]
    fn exact_state_cycles_are_rejected_during_analysis() {
        let program = parse(
            "let repeat = (value) => repeat(value)\noutput repeat(1)",
            "cycle.ns",
        )
        .unwrap();
        let diagnostic = analyze(&program)
            .into_iter()
            .find(|item| item.code == "NSS007")
            .expect("cycle diagnostic");
        assert!(diagnostic.message.contains("repeat -> repeat"));
        assert_eq!(
            crate::bytecode::compile(&program).unwrap_err().0.code,
            "NSS007"
        );
    }
    #[test]
    fn zero_proof_goal_round_trips() {
        let program = parse(
            "let x = index(1, scalar(2, 1))\nadd(x, orient(2, x)) = 0",
            "proof.ns",
        )
        .unwrap();
        assert_eq!(program.goal, Goal::ProveZero);
        assert!(interpret(&program).unwrap().is_zero());
        assert_eq!(
            program_from_data(&program_to_data(&program)).unwrap(),
            program
        );
    }

    #[test]
    fn axis_cancellation_is_automatic_and_preserves_index_residuals() {
        let reduced = interpret(
            &parse(
                "output add(scalar(3, 2), orient(2, scalar(3, 1)))",
                "residual.ns",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            reduced,
            NativeState::scalar(NativeScalar::from_text("0", "1").unwrap())
        );

        let indexed = interpret(
            &parse(
                "output add(index(1, 1), orient(2, index(2, 1)))",
                "indexed-residual.ns",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(indexed.0.len(), 2);
        assert!(!indexed.is_zero());
    }

    #[test]
    fn utf8_surface_is_exact_ordered_and_core_only() {
        let program = parse(
            "let text = \"hé\\nλ\"\nadd(text, orient(2, \"h\\u00e9\\nλ\")) = 0",
            "utf8.ns",
        )
        .unwrap();
        let state = evaluate(
            &program.bindings[0].value,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &mut Vec::new(),
            &program.source_name,
        )
        .unwrap();
        assert_eq!(decode_utf8(&state).unwrap(), "hé\nλ");
        assert!(interpret(&program).unwrap().is_zero());
        let encoded = program_to_data(&program).to_string();
        assert!(!encoded.contains("hé"));
        assert!(!encoded.contains("string"));

        let ab = interpret(&parse("output \"ab\"", "ab.ns").unwrap()).unwrap();
        let ba = interpret(&parse("output \"ba\"", "ba.ns").unwrap()).unwrap();
        assert_ne!(ab, ba);
    }

    #[test]
    fn source_function_outputs_and_zero_equalities() {
        let output = parse("let Re = (x) => x\noutput Re(1)", "output.ns").unwrap();
        assert_eq!(interpret(&output).unwrap(), NativeState::one());
        let proof = parse("let Re = (x) => x\nRe(1) = 1", "proof.ns").unwrap();
        assert_eq!(proof.goal, Goal::ProveZero);
        assert!(interpret(&proof).unwrap().is_zero());
        assert!(program_to_data(&proof).to_string().contains("call"));
        let bytecode = crate::bytecode::compile(&proof).unwrap();
        assert!(crate::bytecode::execute(&bytecode).unwrap().is_zero());
    }

    #[test]
    fn output_cameras_cover_number_string_vector_pattern_and_boolean() {
        let number = interpret(&parse("output 3/2", "number.ns").unwrap()).unwrap();
        assert_eq!(
            output_data(&number, OutputKind::Auto).unwrap()["kind"],
            "number"
        );

        let string = interpret(&parse("output \"plain\"", "string.ns").unwrap()).unwrap();
        assert_eq!(
            output_data(&string, OutputKind::Auto).unwrap()["kind"],
            "string"
        );

        let pattern = interpret(&parse("output index(1, 1)", "pattern.ns").unwrap()).unwrap();
        assert_eq!(
            output_data(&pattern, OutputKind::Pattern).unwrap()["kind"],
            "pattern"
        );

        let vector = NativeState::scalar(NativeScalar::from_text("3", "4").unwrap());
        assert_eq!(
            output_data(&vector, OutputKind::Vector).unwrap()["value"],
            json!(["-7", "24", "25"])
        );
        assert_eq!(
            output_data(&NativeState::zero(), OutputKind::Vector).unwrap()["value"],
            json!(["0", "0", "0"])
        );
        assert_eq!(
            output_data(&pattern, OutputKind::Vector).unwrap_err(),
            "result is not one unindexed oriented scalar"
        );

        assert_eq!(
            output_data(&NativeState::one(), OutputKind::Boolean).unwrap()["value"],
            true
        );
    }

    #[test]
    fn source_function_accepts_repeated_exact_host_invocation() {
        let program = parse(
            "let combine = (left, right) => add(left, right)\noutput zero",
            "callable.ns",
        )
        .unwrap();
        let function = exact_function(&program, "combine").unwrap();
        let two = NativeState::scalar(NativeScalar::from_text("2", "0").unwrap());
        let three = NativeState::scalar(NativeScalar::from_text("3", "0").unwrap());

        assert_eq!(function.parameter_count(), 2);
        assert_eq!(
            function.apply(&[two, three]).unwrap(),
            NativeState::scalar(NativeScalar::from_text("5", "0").unwrap())
        );
        assert_eq!(function.apply(&[]).unwrap_err().0.code, "NSE002");
    }

    #[test]
    fn variadic_concat_elaborates_to_exact_indexed_addition() {
        let source = "let parameters = (head, tail...) => concat(9, head, tail...)\n\
                      let forwarded = (values...) => parameters(values...)\n\
                      let expected = () => add(index(9, 2), index(9, index(9, 3)), index(9, index(9, index(9, 5))))\n\
                      add(forwarded(2, 3, 5), orient(2, expected())) = 0";
        let program = parse(source, "variadic.ns").unwrap();

        assert!(program.functions[0].variadic);
        assert!(program.functions[1].variadic);
        assert!(interpret(&program).unwrap().is_zero());
        let bytecode = crate::bytecode::compile(&program).unwrap();
        assert!(crate::bytecode::execute(&bytecode).unwrap().is_zero());
        assert!(bytecode.instructions.iter().all(|instruction| {
            matches!(
                instruction.opcode,
                crate::bytecode::Opcode::PushZero
                    | crate::bytecode::Opcode::PushOne
                    | crate::bytecode::Opcode::PushScalar
                    | crate::bytecode::Opcode::Load
                    | crate::bytecode::Opcode::Store
                    | crate::bytecode::Opcode::Add
                    | crate::bytecode::Opcode::Multiply
                    | crate::bytecode::Opcode::Orient
                    | crate::bytecode::Opcode::Index
                    | crate::bytecode::Opcode::Halt
            )
        }));
    }

    #[test]
    fn variadic_host_function_accepts_any_pack_length() {
        let program = parse(
            "let parameters = (head, tail...) => add(head, tail...)\noutput zero",
            "host-variadic.ns",
        )
        .unwrap();
        let function = exact_function(&program, "parameters").unwrap();
        let two = NativeState::scalar(NativeScalar::from_text("2", "0").unwrap());
        let three = NativeState::scalar(NativeScalar::from_text("3", "0").unwrap());
        let five = NativeState::scalar(NativeScalar::from_text("5", "0").unwrap());

        assert!(function.is_variadic());
        assert_eq!(function.parameter_count(), 2);
        assert_eq!(function.minimum_parameter_count(), 1);
        assert_eq!(function.apply(std::slice::from_ref(&two)).unwrap(), two);
        assert_eq!(
            function.apply(&[two, three, five]).unwrap(),
            NativeState::scalar(NativeScalar::from_text("10", "0").unwrap())
        );
        assert_eq!(function.apply(&[]).unwrap_err().0.code, "NSE002");
    }

    #[test]
    fn variadic_pack_errors_are_located() {
        let nonfinal = parse(
            "let invalid = (values..., tail) => tail\noutput zero",
            "nonfinal-pack.ns",
        )
        .unwrap_err();
        assert_eq!(nonfinal.0.code, "NSP054");

        let ordinary = parse(
            "let invalid = (value) => concat(1, value...)\noutput invalid(one)",
            "ordinary-spread.ns",
        )
        .unwrap();
        let ordinary_error = interpret(&ordinary).unwrap_err();
        assert_eq!(ordinary_error.0.code, "NSS009");
        assert_eq!(ordinary_error.0.span.unwrap().start_line, 1);

        let misplaced = parse(
            "let invalid = (values...) => index(1, values...)\noutput invalid(one)",
            "misplaced-spread.ns",
        )
        .unwrap();
        assert_eq!(interpret(&misplaced).unwrap_err().0.code, "NSS009");

        let empty = parse(
            "let invalid = (values...) => concat(1, values...)\noutput invalid()",
            "empty-concat.ns",
        )
        .unwrap();
        assert_eq!(interpret(&empty).unwrap_err().0.code, "NST007");
    }

    #[test]
    fn concat_uses_ordinary_index_depth_addition_on_an_existing_axis() {
        let program = parse(
            "output concat(1, index(1, one), one) as pattern",
            "concat-existing-axis.ns",
        )
        .unwrap();
        let state = interpret(&program).unwrap();
        let (index, coefficient) = state.0.iter().next().unwrap();

        assert_eq!(state.0.len(), 1);
        assert_eq!(index.depth(1), BigUint::from(2_u8));
        assert_eq!(coefficient, &NativeScalar::from_text("2", "0").unwrap());
    }

    #[test]
    fn finite_fold_elaborates_to_ordered_binary_calls() {
        let source = "let sum = (left, right) => add(left, right)\n\
                      let total = (values...) => fold(sum, zero, values...)\n\
                      total(1, 2, 3, 4) = 10";
        let program = parse(source, "fold.ns").unwrap();

        assert!(interpret(&program).unwrap().is_zero());
        let expanded = expanded_source(&program).unwrap();
        assert!(!expanded.contains("fold("));
        let expanded_program = parse(&expanded, "expanded-fold.ns").unwrap();
        assert!(interpret(&expanded_program).unwrap().is_zero());

        let host_program = parse(
            "let sum = (left, right) => add(left, right)\n\
             let total = (values...) => fold(sum, zero, values...)\n\
             output zero",
            "host-fold.ns",
        )
        .unwrap();
        let total = exact_function(&host_program, "total").unwrap();
        assert!(total.apply(&[]).unwrap().is_zero());
        assert_eq!(
            total
                .apply(&[
                    NativeState::one(),
                    NativeState::scalar(NativeScalar::from_text("2", "0").unwrap()),
                ])
                .unwrap(),
            NativeState::scalar(NativeScalar::from_text("3", "0").unwrap())
        );
    }

    #[test]
    fn camera_selects_unwraps_and_remaps_one_index_direction() {
        let source = "let selected = camera(7, 9, add(index(7, index(7, 3)), index(8, 5)))\n\
                      selected = index(9, index(9, 3))";
        let program = parse(source, "camera.ns").unwrap();

        assert!(interpret(&program).unwrap().is_zero());
        let bytecode = crate::bytecode::compile(&program).unwrap();
        assert!(crate::bytecode::execute(&bytecode).unwrap().is_zero());

        let unwrap = parse(
            "camera(7, 0, add(index(7, 2), index(8, 5))) = 2",
            "camera-unwrap.ns",
        )
        .unwrap();
        assert!(interpret(&unwrap).unwrap().is_zero());
    }
}
