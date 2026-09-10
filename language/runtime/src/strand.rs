// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Encodes source functions as nested native operation strands.
//!
//! A trace is a reflective camera, not a fifth algebra operation. Every
//! returned strand is an expression containing only exact constants, ADD,
//! PHASE, and INDEX. A node stores its instruction under the head coordinate
//! and stores the remainder of the strand under the continuation coordinate.

use std::collections::{BTreeMap, BTreeSet};

pub mod execution;

use num_bigint::BigUint;
use num_traits::{One as _, ToPrimitive as _, Zero as _};

use crate::core::{
    Diagnostic, Expr, Function, Goal, LanguageError, NativeScalar, NativeState, OutputKind,
    Program, Span, is_canonical_phase,
};

/// Version of the native operation-strand coordinate layout.
pub const OPERATION_STRAND_VERSION: u64 = 1;

// Trace coordinates occupy a camera-owned high-direction namespace. They must
// not be added directly to another camera without an explicit coordinate map.
const TRACE_DIRECTION_START: u64 = u64::MAX - 31;
const HEAD_DIRECTION: u64 = TRACE_DIRECTION_START;
const CONTINUATION_DIRECTION: u64 = TRACE_DIRECTION_START + 1;
const KIND_DIRECTION: u64 = TRACE_DIRECTION_START + 2;
const OPCODE_DIRECTION: u64 = TRACE_DIRECTION_START + 3;
const NAME_DIRECTION: u64 = TRACE_DIRECTION_START + 4;
const SOURCE_DIRECTION: u64 = TRACE_DIRECTION_START + 5;
const TEXT_A_DIRECTION: u64 = TRACE_DIRECTION_START + 6;
const TEXT_B_DIRECTION: u64 = TRACE_DIRECTION_START + 7;
const NUMBER_A_DIRECTION: u64 = TRACE_DIRECTION_START + 8;
const NUMBER_B_DIRECTION: u64 = TRACE_DIRECTION_START + 9;
const START_LINE_DIRECTION: u64 = TRACE_DIRECTION_START + 10;
const START_COLUMN_DIRECTION: u64 = TRACE_DIRECTION_START + 11;
const END_LINE_DIRECTION: u64 = TRACE_DIRECTION_START + 12;
const END_COLUMN_DIRECTION: u64 = TRACE_DIRECTION_START + 13;
const TEXT_POSITION_DIRECTION: u64 = TRACE_DIRECTION_START + 14;
const NUMBER_C_DIRECTION: u64 = TRACE_DIRECTION_START + 15;

const TRACE_START: u64 = 1;
const FUNCTION_START: u64 = 2;
const FUNCTION_END: u64 = 3;
const PARAMETER: u64 = 4;
const LITERAL: u64 = 7;
const REFERENCE: u64 = 8;
const ADD: u64 = 10;
const MULTIPLY: u64 = 11;
const PHASE: u64 = 12;
const INDEX: u64 = 13;
const SPREAD: u64 = 17;
// Staged reflection expressions retain their source identity in traces.
const REFLECT: u64 = 23;
// Template binder syntax, not another executable Native operation.
const INDEX_CAPTURE: u64 = 24;
// Postfix function-value calls preserve their callee as an explicit edge.
const CALL: u64 = 25;

/// Maximum expression nesting decoded from one operation strand.
///
/// The bound converts adversarially deep coordinate chains into a located
/// diagnostic instead of risking a process-level stack overflow. Ordinary
/// source is already constrained by the parser and remains far below it.
const MAX_DECODE_DEPTH: usize = 1_024;

/// Return whether a state carries the operation-strand start marker.
#[must_use]
pub(crate) fn is_operation_strand(state: &NativeState) -> bool {
    let marker = crate::core::MultiIndex::from_depths([(HEAD_DIRECTION, 1), (KIND_DIRECTION, 1)])
        .expect("operation-strand directions are positive");
    state.0.get(&marker) == Some(&NativeScalar::one())
}

/// Project an operation strand onto its unary instruction length.
///
/// The returned value counts canonical trace coordinates, including the trace
/// start, function boundaries, parameters, expression nodes, and call edges.
/// Source provenance and span fields decorate coordinates but do not add
/// instruction positions.
///
/// # Errors
///
/// Returns `NSL001` when `state` is not a canonical operation strand or its
/// finite coordinate count cannot be represented by the language's INDEX
/// multiplicity.
pub(crate) fn operation_length(
    state: &NativeState,
    source_name: &str,
    span: Option<Span>,
) -> Result<u64, LanguageError> {
    let malformed = || {
        LanguageError(Diagnostic {
            code: "NSL001".into(),
            message: "length expects one canonical operation strand".into(),
            source_name: source_name.into(),
            span,
        })
    };
    if !is_operation_strand(state) {
        return Err(malformed());
    }
    coordinate_length(state, source_name, span)
}

fn coordinate_length(
    state: &NativeState,
    source_name: &str,
    span: Option<Span>,
) -> Result<u64, LanguageError> {
    let malformed = || malformed_strand(source_name, span);

    let mut positions = BTreeSet::<BigUint>::new();
    for index in state.0.keys() {
        if index.depth(HEAD_DIRECTION) != BigUint::from(1_u8)
            || index.0.keys().any(|direction| {
                *direction < TRACE_DIRECTION_START || *direction > NUMBER_C_DIRECTION
            })
        {
            return Err(malformed());
        }
        if index.depth(KIND_DIRECTION) == BigUint::from(1_u8)
            && index.0.keys().all(|direction| {
                matches!(
                    *direction,
                    HEAD_DIRECTION | CONTINUATION_DIRECTION | KIND_DIRECTION
                )
            })
        {
            positions.insert(index.depth(CONTINUATION_DIRECTION));
        }
    }
    if positions.is_empty()
        || positions
            .iter()
            .enumerate()
            .any(|(expected, actual)| *actual != BigUint::from(expected))
    {
        return Err(malformed());
    }
    let length = u64::try_from(positions.len()).map_err(|_capacity_error| malformed())?;
    if state
        .0
        .keys()
        .any(|index| index.depth(CONTINUATION_DIRECTION) >= BigUint::from(length))
    {
        return Err(malformed());
    }
    Ok(length)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Coordinate {
    kind: u64,
    opcode_turn: Option<i64>,
    name: Option<String>,
    source: Option<String>,
    text_a: Option<String>,
    text_b: Option<String>,
    number_a: Option<String>,
    number_b: Option<String>,
    number_c: Option<String>,
    span: Option<Span>,
}

impl Coordinate {
    fn new(kind: u64, span: Option<Span>) -> Self {
        Self {
            kind,
            opcode_turn: None,
            name: None,
            source: None,
            text_a: None,
            text_b: None,
            number_a: None,
            number_b: None,
            number_c: None,
            span,
        }
    }
}

/// Build the nested native strand returned by `function`.
///
/// The root function and every transitively referenced source function are
/// encoded once. Calls remain explicit edges, so direct and mutual recursion
/// produce finite self-modeling graphs instead of unbounded expansion.
///
/// # Errors
///
/// Returns a located diagnostic when the requested function or one of its
/// called functions is absent from the supplied catalog.
pub(crate) fn operation_strand(
    function: &str,
    functions: &BTreeMap<String, &Function>,
    encoded_source: &str,
    diagnostic_source: &str,
    span: Option<Span>,
) -> Result<Expr, LanguageError> {
    Ok(nest(operation_coordinates(
        function,
        functions,
        encoded_source,
        diagnostic_source,
        span,
    )?))
}

fn operation_coordinates(
    function: &str,
    functions: &BTreeMap<String, &Function>,
    encoded_source: &str,
    diagnostic_source: &str,
    span: Option<Span>,
) -> Result<Vec<Coordinate>, LanguageError> {
    let mut coordinates = vec![Coordinate {
        kind: TRACE_START,
        name: Some(function.to_owned()),
        source: Some(encoded_source.to_owned()),
        number_a: Some(OPERATION_STRAND_VERSION.to_string()),
        ..Coordinate::new(TRACE_START, None)
    }];
    let mut visited = BTreeSet::new();
    collect_function(
        function,
        functions,
        diagnostic_source,
        span,
        &mut visited,
        &mut coordinates,
    )?;
    Ok(coordinates)
}

/// Optimize one canonical instruction strand at exact rank one.
///
/// The strand is decoded into its source-function graph, every occurrence of
/// the theorem-authorized rewrite rules is applied, and the graph is encoded
/// again. The original strand remains authoritative unless the candidate has
/// fewer instruction coordinates. Ranks below one are rejected because
/// dropping instruction relationships has no exact equivalence rule yet.
///
/// # Errors
///
/// Returns a located diagnostic for a malformed strand, a non-unit rank, or an
/// invalid reconstructed function graph.
pub fn optimize_operation_strand(
    state: &NativeState,
    rank: &str,
    source_name: &str,
    span: Option<Span>,
) -> Result<Option<Expr>, LanguageError> {
    let rank = crate::core::rational(rank).map_err(|_parse_error| {
        instruction_error(
            "NSI001",
            "instruction optimization rank must be exact",
            source_name,
            span,
        )
    })?;
    if rank != crate::core::Rational::one() {
        return Err(instruction_error(
            "NSI002",
            "exact instruction optimization currently requires rank one",
            source_name,
            span,
        ));
    }

    let selected = execution::FunctionValue::source_graph(
        &crate::retained::State::from_projection(state),
        source_name,
    )?;
    let original_length = operation_length(selected.project(), source_name, span)?;
    let decoded = decode_operation_strand(state, source_name, span)?;
    let program = Program {
        functions: decoded.functions,
        bindings: Vec::new(),
        goal: Goal::Emit,
        output_kind: OutputKind::Pattern,
        result: Expr::Reference {
            name: decoded.root.clone(),
            span,
        },
        source_name: source_name.into(),
        span,
    };
    let optimized = crate::core::optimize(&program)?;
    if optimized.events.is_empty() {
        return Ok(None);
    }

    let functions = optimized
        .program
        .functions
        .iter()
        .map(|function| (function.name.clone(), function))
        .collect::<BTreeMap<_, _>>();
    let coordinates = operation_coordinates(
        &decoded.root,
        &functions,
        &decoded.encoded_source,
        source_name,
        span,
    )?;
    let candidate_length = u64::try_from(coordinates.len()).map_err(|_capacity_error| {
        instruction_error(
            "NSI003",
            "optimized instruction count exceeds the supported size",
            source_name,
            span,
        )
    })?;
    if candidate_length >= original_length {
        return Ok(None);
    }
    Ok(Some(nest(coordinates)))
}

#[derive(Debug)]
pub(crate) struct DecodedStrand {
    pub(crate) root: String,
    pub(crate) encoded_source: String,
    pub(crate) functions: Vec<Function>,
}

#[derive(Debug, Default)]
struct RawCoordinate {
    kind: Option<u64>,
    opcode: Option<NativeScalar>,
    name: BTreeMap<u64, u8>,
    source: BTreeMap<u64, u8>,
    text_a: BTreeMap<u64, u8>,
    text_b: BTreeMap<u64, u8>,
    number_a: Option<String>,
    number_b: Option<String>,
    number_c: Option<String>,
    start_line: Option<usize>,
    start_column: Option<usize>,
    end_line: Option<usize>,
    end_column: Option<usize>,
}

fn instruction_error(
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

fn malformed_strand(source_name: &str, span: Option<Span>) -> LanguageError {
    instruction_error(
        "NSI003",
        "instruction optimization expects one canonical decodable operation strand",
        source_name,
        span,
    )
}

pub(crate) fn decode_operation_strand(
    state: &NativeState,
    source_name: &str,
    span: Option<Span>,
) -> Result<DecodedStrand, LanguageError> {
    let selected = execution::FunctionValue::source_graph(
        &crate::retained::State::from_projection(state),
        source_name,
    )?;
    let coordinates = decode_coordinates(selected.project(), source_name, span)?;
    decode_coordinate_sequence(&coordinates, source_name, span)
}

fn decode_coordinates(
    state: &NativeState,
    source_name: &str,
    span: Option<Span>,
) -> Result<Vec<Coordinate>, LanguageError> {
    let length = coordinate_length(state, source_name, span)?;
    let length =
        usize::try_from(length).map_err(|_capacity_error| malformed_strand(source_name, span))?;
    let mut raw = (0..length)
        .map(|_| RawCoordinate::default())
        .collect::<Vec<_>>();
    for (index, coefficient) in &state.0 {
        decode_coordinate_term(index, coefficient, &mut raw, source_name, span)?;
    }
    let coordinates = raw
        .into_iter()
        .map(|coordinate| finish_coordinate(coordinate, source_name, span))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(coordinates)
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive field match keeps strand-schema validation auditable"
)]
fn decode_coordinate_term(
    index: &crate::core::MultiIndex,
    coefficient: &NativeScalar,
    coordinates: &mut [RawCoordinate],
    source_name: &str,
    span: Option<Span>,
) -> Result<(), LanguageError> {
    let malformed = || malformed_strand(source_name, span);
    let mut fields = index.0.clone();
    if fields.remove(&HEAD_DIRECTION) != Some(BigUint::one()) {
        return Err(malformed());
    }
    let position = fields
        .remove(&CONTINUATION_DIRECTION)
        .unwrap_or_default()
        .to_usize()
        .ok_or_else(malformed)?;
    let coordinate = coordinates.get_mut(position).ok_or_else(malformed)?;

    if fields.len() == 1 {
        let (&direction, depth) = fields.first_key_value().ok_or_else(malformed)?;
        if *depth != BigUint::one() {
            return Err(malformed());
        }
        match direction {
            KIND_DIRECTION => {
                set_once(
                    &mut coordinate.kind,
                    exact_u64(coefficient).ok_or_else(malformed)?,
                    source_name,
                    span,
                )?;
            }
            OPCODE_DIRECTION => {
                set_once(
                    &mut coordinate.opcode,
                    coefficient.clone(),
                    source_name,
                    span,
                )?;
            }
            NUMBER_A_DIRECTION => {
                set_once(
                    &mut coordinate.number_a,
                    exact_scalar_text(coefficient, source_name, span)?,
                    source_name,
                    span,
                )?;
            }
            NUMBER_B_DIRECTION => {
                set_once(
                    &mut coordinate.number_b,
                    exact_scalar_text(coefficient, source_name, span)?,
                    source_name,
                    span,
                )?;
            }
            NUMBER_C_DIRECTION => {
                set_once(
                    &mut coordinate.number_c,
                    exact_scalar_text(coefficient, source_name, span)?,
                    source_name,
                    span,
                )?;
            }
            START_LINE_DIRECTION => {
                set_once(
                    &mut coordinate.start_line,
                    exact_usize(coefficient).ok_or_else(malformed)?,
                    source_name,
                    span,
                )?;
            }
            START_COLUMN_DIRECTION => {
                set_once(
                    &mut coordinate.start_column,
                    exact_usize(coefficient).ok_or_else(malformed)?,
                    source_name,
                    span,
                )?;
            }
            END_LINE_DIRECTION => {
                set_once(
                    &mut coordinate.end_line,
                    exact_usize(coefficient).ok_or_else(malformed)?,
                    source_name,
                    span,
                )?;
            }
            END_COLUMN_DIRECTION => {
                set_once(
                    &mut coordinate.end_column,
                    exact_usize(coefficient).ok_or_else(malformed)?,
                    source_name,
                    span,
                )?;
            }
            _ => return Err(malformed()),
        }
        return Ok(());
    }

    if fields.len() != 2 {
        return Err(malformed());
    }
    let text_position = fields
        .remove(&TEXT_POSITION_DIRECTION)
        .ok_or_else(malformed)?
        .to_u64()
        .ok_or_else(malformed)?;
    if text_position == 0 || fields.len() != 1 {
        return Err(malformed());
    }
    let (&direction, depth) = fields.first_key_value().ok_or_else(malformed)?;
    if *depth != BigUint::one() {
        return Err(malformed());
    }
    let byte = exact_u64(coefficient)
        .and_then(|value| value.checked_sub(1))
        .and_then(|value| u8::try_from(value).ok())
        .ok_or_else(malformed)?;
    let text = match direction {
        NAME_DIRECTION => &mut coordinate.name,
        SOURCE_DIRECTION => &mut coordinate.source,
        TEXT_A_DIRECTION => &mut coordinate.text_a,
        TEXT_B_DIRECTION => &mut coordinate.text_b,
        _ => return Err(malformed()),
    };
    if text.insert(text_position, byte).is_some() {
        return Err(malformed());
    }
    Ok(())
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    source_name: &str,
    span: Option<Span>,
) -> Result<(), LanguageError> {
    if slot.replace(value).is_some() {
        return Err(instruction_error(
            "NSI003",
            "operation strand contains a duplicate instruction field",
            source_name,
            span,
        ));
    }
    Ok(())
}

fn exact_u64(value: &NativeScalar) -> Option<u64> {
    if !value.imag.is_zero() || !value.real.denom().is_one() {
        return None;
    }
    value.real.numer().to_u64()
}

fn exact_usize(value: &NativeScalar) -> Option<usize> {
    exact_u64(value).and_then(|number| usize::try_from(number).ok())
}

fn exact_scalar_text(
    value: &NativeScalar,
    source_name: &str,
    span: Option<Span>,
) -> Result<String, LanguageError> {
    if !value.imag.is_zero() {
        return Err(malformed_strand(source_name, span));
    }
    if value.real.denom().is_one() {
        Ok(value.real.numer().to_string())
    } else {
        Ok(format!("{}/{}", value.real.numer(), value.real.denom()))
    }
}

fn finish_coordinate(
    raw: RawCoordinate,
    source_name: &str,
    span: Option<Span>,
) -> Result<Coordinate, LanguageError> {
    let kind = raw
        .kind
        .ok_or_else(|| malformed_strand(source_name, span))?;
    let opcode_turn = match kind {
        ADD => Some(0),
        MULTIPLY => Some(1),
        PHASE => Some(2),
        INDEX | INDEX_CAPTURE => Some(3),
        _ => None,
    };
    if raw.opcode != opcode_turn.map(|turns| NativeScalar::one().phase(turns)) {
        return Err(malformed_strand(source_name, span));
    }
    let positions = [
        raw.start_line,
        raw.start_column,
        raw.end_line,
        raw.end_column,
    ];
    let coordinate_span = if positions.iter().all(Option::is_none) {
        None
    } else if let [
        Some(start_line),
        Some(start_column),
        Some(end_line),
        Some(end_column),
    ] = positions
    {
        Some(Span {
            start_line,
            start_column,
            end_line,
            end_column,
        })
    } else {
        return Err(malformed_strand(source_name, span));
    };
    Ok(Coordinate {
        kind,
        opcode_turn,
        name: finish_text(raw.name, source_name, span)?,
        source: finish_text(raw.source, source_name, span)?,
        text_a: finish_text(raw.text_a, source_name, span)?,
        text_b: finish_text(raw.text_b, source_name, span)?,
        number_a: raw.number_a,
        number_b: raw.number_b,
        number_c: raw.number_c,
        span: coordinate_span,
    })
}

fn finish_text(
    bytes: BTreeMap<u64, u8>,
    source_name: &str,
    span: Option<Span>,
) -> Result<Option<String>, LanguageError> {
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes
        .keys()
        .enumerate()
        .any(|(position, actual)| *actual != u64::try_from(position + 1).unwrap_or(u64::MAX))
    {
        return Err(malformed_strand(source_name, span));
    }
    String::from_utf8(bytes.into_values().collect())
        .map(Some)
        .map_err(|_utf8_error| malformed_strand(source_name, span))
}

fn decode_coordinate_sequence(
    coordinates: &[Coordinate],
    source_name: &str,
    span: Option<Span>,
) -> Result<DecodedStrand, LanguageError> {
    let start = coordinates
        .first()
        .ok_or_else(|| malformed_strand(source_name, span))?;
    if start.kind != TRACE_START || start.number_a.as_deref() != Some("1") {
        return Err(malformed_strand(source_name, span));
    }
    let root = required_text(start.name.as_deref(), source_name, span)?.to_owned();
    let encoded_source = required_text(start.source.as_deref(), source_name, span)?.to_owned();
    let mut cursor = 1;
    let mut functions = Vec::new();
    while cursor < coordinates.len() {
        functions.push(decode_function(
            coordinates,
            &mut cursor,
            source_name,
            span,
        )?);
    }
    if functions.iter().all(|function| function.name != root) {
        return Err(malformed_strand(source_name, span));
    }
    Ok(DecodedStrand {
        root,
        encoded_source,
        functions,
    })
}

fn decode_function(
    coordinates: &[Coordinate],
    cursor: &mut usize,
    source_name: &str,
    span: Option<Span>,
) -> Result<Function, LanguageError> {
    let start = take_coordinate(coordinates, cursor, source_name, span)?;
    if start.kind != FUNCTION_START {
        return Err(malformed_strand(source_name, span));
    }
    let name = required_text(start.name.as_deref(), source_name, span)?.to_owned();
    let parameter_count = usize_or_zero(start.number_a.as_deref(), source_name, span)?;
    let variadic = match start.number_b.as_deref() {
        None | Some("0") => false,
        Some("1") => true,
        _ => return Err(malformed_strand(source_name, span)),
    };
    let mut parameters = Vec::with_capacity(parameter_count);
    for expected_position in 0..parameter_count {
        let parameter = take_coordinate(coordinates, cursor, source_name, span)?;
        if parameter.kind != PARAMETER
            || usize_or_zero(parameter.number_a.as_deref(), source_name, span)? != expected_position
        {
            return Err(malformed_strand(source_name, span));
        }
        parameters.push(required_text(parameter.name.as_deref(), source_name, span)?.to_owned());
    }
    let body = decode_expression(coordinates, cursor, 0, source_name, span)?;
    let end = take_coordinate(coordinates, cursor, source_name, span)?;
    if end.kind != FUNCTION_END || end.name.as_deref() != Some(name.as_str()) {
        return Err(malformed_strand(source_name, span));
    }
    Ok(Function {
        name,
        parameters,
        variadic,
        body,
        span: start.span,
    })
}

fn decode_expression(
    coordinates: &[Coordinate],
    cursor: &mut usize,
    depth: usize,
    source_name: &str,
    span: Option<Span>,
) -> Result<Expr, LanguageError> {
    if depth >= MAX_DECODE_DEPTH {
        return Err(instruction_error(
            "NSI004",
            format!("instruction expression nesting exceeds {MAX_DECODE_DEPTH}"),
            source_name,
            span,
        ));
    }
    let coordinate = take_coordinate(coordinates, cursor, source_name, span)?;
    let child =
        |cursor: &mut usize| decode_expression(coordinates, cursor, depth + 1, source_name, span);
    let children = |count: usize, cursor: &mut usize| {
        (0..count)
            .map(|_| child(cursor))
            .collect::<Result<Vec<_>, _>>()
    };
    let expression = match coordinate.kind {
        CALL => Expr::Call {
            callee: Box::new(child(cursor)?),
            arguments: children(
                usize_or_zero(coordinate.number_a.as_deref(), source_name, span)?,
                cursor,
            )?,
            span: coordinate.span,
        },
        LITERAL => Expr::Literal {
            real: required_text(coordinate.text_a.as_deref(), source_name, span)?.to_owned(),
            imag: required_text(coordinate.text_b.as_deref(), source_name, span)?.to_owned(),
            span: coordinate.span,
        },
        REFERENCE => Expr::Reference {
            name: required_text(coordinate.name.as_deref(), source_name, span)?.to_owned(),
            span: coordinate.span,
        },
        SPREAD => Expr::Spread {
            name: required_text(coordinate.name.as_deref(), source_name, span)?.to_owned(),
            span: coordinate.span,
        },
        REFLECT => {
            let name = required_text(coordinate.name.as_deref(), source_name, span)?;
            if name != "reflect" {
                return Err(malformed_strand(source_name, span));
            }
            let arguments = children(
                usize_or_zero(coordinate.number_a.as_deref(), source_name, span)?,
                cursor,
            )?;
            if arguments.len() != 3 {
                return Err(malformed_strand(source_name, span));
            }
            Expr::Reflect {
                arguments,
                span: coordinate.span,
            }
        }
        ADD => Expr::Add {
            operands: children(
                usize_or_zero(coordinate.number_a.as_deref(), source_name, span)?,
                cursor,
            )?,
            span: coordinate.span,
        },
        MULTIPLY => Expr::Multiply {
            operands: children(
                usize_or_zero(coordinate.number_a.as_deref(), source_name, span)?,
                cursor,
            )?,
            span: coordinate.span,
        },
        PHASE => {
            let turns = i64_or_zero(coordinate.number_a.as_deref(), source_name, span)?;
            if !is_canonical_phase(turns) {
                return Err(malformed_strand(source_name, span));
            }
            Expr::Phase {
                turns,
                value: Box::new(child(cursor)?),
                span: coordinate.span,
            }
        }
        INDEX_CAPTURE => Expr::IndexCapture {
            direction: required_u64(coordinate.number_a.as_deref(), source_name, span)?,
            depth: required_text(coordinate.name.as_deref(), source_name, span)?.to_owned(),
            value: Box::new(child(cursor)?),
            span: coordinate.span,
        },
        INDEX => Expr::Index {
            direction: required_u64(coordinate.number_a.as_deref(), source_name, span)?,
            multiplicity: required_u64(coordinate.number_b.as_deref(), source_name, span)?,
            value: Box::new(child(cursor)?),
            span: coordinate.span,
        },
        _ => return Err(malformed_strand(source_name, span)),
    };
    Ok(expression)
}

fn take_coordinate<'a>(
    coordinates: &'a [Coordinate],
    cursor: &mut usize,
    source_name: &str,
    span: Option<Span>,
) -> Result<&'a Coordinate, LanguageError> {
    let coordinate = coordinates
        .get(*cursor)
        .ok_or_else(|| malformed_strand(source_name, span))?;
    *cursor += 1;
    Ok(coordinate)
}

fn required_text<'a>(
    value: Option<&'a str>,
    source_name: &str,
    span: Option<Span>,
) -> Result<&'a str, LanguageError> {
    value.ok_or_else(|| malformed_strand(source_name, span))
}

fn required_u64(
    value: Option<&str>,
    source_name: &str,
    span: Option<Span>,
) -> Result<u64, LanguageError> {
    required_text(value, source_name, span)?
        .parse()
        .map_err(|_parse_error| malformed_strand(source_name, span))
}

fn i64_or_zero(
    value: Option<&str>,
    source_name: &str,
    span: Option<Span>,
) -> Result<i64, LanguageError> {
    value.map_or(Ok(0), |value| {
        value
            .parse()
            .map_err(|_parse_error| malformed_strand(source_name, span))
    })
}

fn usize_or_zero(
    value: Option<&str>,
    source_name: &str,
    span: Option<Span>,
) -> Result<usize, LanguageError> {
    value.map_or(Ok(0), |value| {
        value
            .parse()
            .map_err(|_parse_error| malformed_strand(source_name, span))
    })
}

fn collect_function(
    name: &str,
    functions: &BTreeMap<String, &Function>,
    diagnostic_source: &str,
    call_span: Option<Span>,
    visited: &mut BTreeSet<String>,
    coordinates: &mut Vec<Coordinate>,
) -> Result<(), LanguageError> {
    if !visited.insert(name.to_owned()) {
        return Ok(());
    }
    let function = functions.get(name).ok_or_else(|| {
        LanguageError(Diagnostic {
            code: "NSS003".into(),
            message: format!("unknown function {name:?}"),
            source_name: diagnostic_source.into(),
            span: call_span,
        })
    })?;

    let mut start = Coordinate::new(FUNCTION_START, function.span);
    start.name = Some(function.name.clone());
    start.number_a = Some(function.parameters.len().to_string());
    start.number_b = Some(u8::from(function.variadic).to_string());
    coordinates.push(start);
    for (position, parameter) in function.parameters.iter().enumerate() {
        let mut coordinate = Coordinate::new(PARAMETER, function.span);
        coordinate.name = Some(parameter.clone());
        coordinate.number_a = Some(position.to_string());
        coordinates.push(coordinate);
    }

    let mut called = Vec::new();
    collect_expression(&function.body, coordinates);
    let mut pending = vec![&function.body];
    while let Some(expression) = pending.pop() {
        if let Expr::Reference { name, span } = expression
            && functions.contains_key(name)
            && !function.parameters.contains(name)
        {
            called.push((name.clone(), *span));
        }
        if let Expr::Reflect { arguments, .. } = expression {
            pending.extend(arguments.first());
        } else {
            pending.extend(crate::reflection::children(expression).into_iter().rev());
        }
    }

    let mut end = Coordinate::new(FUNCTION_END, function.span);
    end.name = Some(function.name.clone());
    coordinates.push(end);

    for (called_name, span) in called {
        if function.parameters.contains(&called_name) {
            continue;
        }
        collect_function(
            &called_name,
            functions,
            diagnostic_source,
            span,
            visited,
            coordinates,
        )?;
    }
    Ok(())
}

fn collect_expression(expression: &Expr, coordinates: &mut Vec<Coordinate>) {
    match expression {
        Expr::Call {
            callee,
            arguments,
            span,
        } => {
            let mut coordinate = Coordinate::new(CALL, *span);
            coordinate.number_a = Some(arguments.len().to_string());
            coordinates.push(coordinate);
            collect_expression(callee, coordinates);
            for argument in arguments {
                collect_expression(argument, coordinates);
            }
        }
        Expr::Literal { real, imag, span } => {
            let mut coordinate = Coordinate::new(LITERAL, *span);
            coordinate.text_a = Some(real.clone());
            coordinate.text_b = Some(imag.clone());
            coordinates.push(coordinate);
        }
        Expr::Reference { name, span } => {
            let mut coordinate = Coordinate::new(REFERENCE, *span);
            coordinate.name = Some(name.clone());
            coordinates.push(coordinate);
        }
        Expr::Spread { name, span } => {
            let mut coordinate = Coordinate::new(SPREAD, *span);
            coordinate.name = Some(name.clone());
            coordinates.push(coordinate);
        }
        Expr::Reflect { arguments, span } => {
            let mut coordinate = Coordinate::new(REFLECT, *span);
            coordinate.name = Some("reflect".into());
            coordinate.number_a = Some(arguments.len().to_string());
            coordinates.push(coordinate);
            for argument in arguments {
                collect_expression(argument, coordinates);
            }
        }
        Expr::Add { operands, span } => {
            let mut coordinate = Coordinate::new(ADD, *span);
            coordinate.opcode_turn = Some(0);
            coordinate.number_a = Some(operands.len().to_string());
            coordinates.push(coordinate);
            for operand in operands {
                collect_expression(operand, coordinates);
            }
        }
        Expr::Multiply { operands, span } => {
            let mut coordinate = Coordinate::new(MULTIPLY, *span);
            coordinate.opcode_turn = Some(1);
            coordinate.number_a = Some(operands.len().to_string());
            coordinates.push(coordinate);
            for operand in operands {
                collect_expression(operand, coordinates);
            }
        }
        Expr::Phase { turns, value, span } => {
            let mut coordinate = Coordinate::new(PHASE, *span);
            coordinate.opcode_turn = Some(2);
            coordinate.number_a = Some(turns.to_string());
            coordinates.push(coordinate);
            collect_expression(value, coordinates);
        }
        Expr::IndexCapture {
            direction,
            depth,
            value,
            span,
        } => {
            let mut coordinate = Coordinate::new(INDEX_CAPTURE, *span);
            coordinate.opcode_turn = Some(3);
            coordinate.number_a = Some(direction.to_string());
            coordinate.name = Some(depth.clone());
            coordinates.push(coordinate);
            collect_expression(value, coordinates);
        }
        Expr::Index {
            direction,
            multiplicity,
            value,
            span,
        } => {
            let mut coordinate = Coordinate::new(INDEX, *span);
            coordinate.opcode_turn = Some(3);
            coordinate.number_a = Some(direction.to_string());
            coordinate.number_b = Some(multiplicity.to_string());
            coordinates.push(coordinate);
            collect_expression(value, coordinates);
        }
    }
}

fn nest(coordinates: Vec<Coordinate>) -> Expr {
    let terms = coordinates
        .into_iter()
        .enumerate()
        .map(|(position, coordinate)| {
            let head = indexed(HEAD_DIRECTION, coordinate_expression(coordinate));
            if position == 0 {
                return head;
            }
            let multiplicity = u64::try_from(position)
                .expect("usize operation-strand positions fit u64 on supported Rust targets");
            Expr::Index {
                direction: CONTINUATION_DIRECTION,
                multiplicity,
                value: Box::new(head),
                span: None,
            }
        })
        .collect::<Vec<_>>();
    sum(terms)
}

fn coordinate_expression(coordinate: Coordinate) -> Expr {
    let mut fields = vec![indexed(KIND_DIRECTION, number(&coordinate.kind))];
    if let Some(turns) = coordinate.opcode_turn {
        fields.push(indexed(
            OPCODE_DIRECTION,
            Expr::Phase {
                turns,
                value: Box::new(Expr::integer(1, None)),
                span: None,
            },
        ));
    }
    push_text(&mut fields, NAME_DIRECTION, coordinate.name);
    push_text(&mut fields, SOURCE_DIRECTION, coordinate.source);
    push_text(&mut fields, TEXT_A_DIRECTION, coordinate.text_a);
    push_text(&mut fields, TEXT_B_DIRECTION, coordinate.text_b);
    push_number(&mut fields, NUMBER_A_DIRECTION, coordinate.number_a);
    push_number(&mut fields, NUMBER_B_DIRECTION, coordinate.number_b);
    push_number(&mut fields, NUMBER_C_DIRECTION, coordinate.number_c);
    if let Some(span) = coordinate.span {
        fields.push(indexed(START_LINE_DIRECTION, number(&span.start_line)));
        fields.push(indexed(START_COLUMN_DIRECTION, number(&span.start_column)));
        fields.push(indexed(END_LINE_DIRECTION, number(&span.end_line)));
        fields.push(indexed(END_COLUMN_DIRECTION, number(&span.end_column)));
    }
    sum(fields)
}

fn push_text(fields: &mut Vec<Expr>, direction: u64, value: Option<String>) {
    if let Some(value) = value {
        fields.extend(value.bytes().enumerate().map(|(position, byte)| {
            let depth = u64::try_from(position).expect("an in-memory string position fits u64") + 1;
            let encoded_byte = u64::from(byte) + 1;
            indexed(
                direction,
                Expr::Index {
                    direction: TEXT_POSITION_DIRECTION,
                    multiplicity: depth,
                    value: Box::new(number(&encoded_byte)),
                    span: None,
                },
            )
        }));
    }
}

fn push_number(fields: &mut Vec<Expr>, direction: u64, value: Option<String>) {
    if let Some(value) = value {
        fields.push(indexed(direction, scalar(value)));
    }
}

fn indexed(direction: u64, value: Expr) -> Expr {
    Expr::Index {
        direction,
        multiplicity: 1,
        value: Box::new(value),
        span: None,
    }
}

fn number(value: &impl ToString) -> Expr {
    scalar(value.to_string())
}

fn scalar(real: String) -> Expr {
    Expr::Literal {
        real,
        imag: "0".into(),
        span: None,
    }
}

fn sum(mut expressions: Vec<Expr>) -> Expr {
    match expressions.len() {
        0 => Expr::integer(0, None),
        1 => expressions.remove(0),
        _ => Expr::Add {
            operands: expressions,
            span: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{interpret, parse};
    use crate::retained::State;
    use execution::Value;

    fn graph(body: &str) -> NativeState {
        interpret(&parse(body, "graph.ns").unwrap()).unwrap()
    }
    fn evaluate(expression: Expr) -> NativeState {
        interpret(&Program {
            functions: vec![],
            bindings: vec![],
            result: expression,
            goal: Goal::Emit,
            output_kind: OutputKind::Pattern,
            source_name: "tool.ns".into(),
            span: None,
        })
        .unwrap()
    }
    fn call(state: &NativeState, value: i64) -> NativeState {
        Value::State(State::from_projection(state))
            .call(vec![Value::State(State::scalar(
                NativeScalar::from_text(&value.to_string(), "0").unwrap(),
            ))])
            .unwrap()
            .native()
            .unwrap()
            .project()
            .clone()
    }
    #[test]
    fn program_state_preserves_distinct_operations_and_indexed_records() {
        let a = graph("let f = (x) => add(x, phase(1,x))\noutput f");
        let b = graph("let f = (x) => multiply(x, phase(1,x))\noutput f");
        assert_ne!(a, b);
        assert!(
            a.0.keys()
                .any(|i| i.depth(CONTINUATION_DIRECTION) > BigUint::from(0_u8))
        );
        assert!(graph("let f = (x) => phase(1,x)\nf = f").is_zero());
    }
    #[test]
    fn compiled_function_data_preserves_source_before_optimization() {
        let program = parse("let f = (x) => add(0,x)\noutput f", "source.ns").unwrap();
        assert_eq!(
            interpret(&program).unwrap(),
            crate::bytecode::execute(&crate::bytecode::lower(&program).unwrap()).unwrap()
        );
    }
    #[test]
    fn host_optimizer_shortens_exact_graph_and_preserves_results() {
        let original = graph("let f = (x) => add(add(x,0),multiply(x,1))\noutput f");
        let shorter = evaluate(
            optimize_operation_strand(&original, "1", "tool.ns", None)
                .unwrap()
                .unwrap(),
        );
        let selected =
            execution::FunctionValue::source_graph(&State::from_projection(&original), "tool.ns")
                .unwrap();
        assert!(
            operation_length(&shorter, "tool.ns", None).unwrap()
                < operation_length(selected.project(), "tool.ns", None).unwrap()
        );
        for value in -8..9 {
            assert_eq!(call(&shorter, value), call(&original, value));
        }
        assert!(
            optimize_operation_strand(&shorter, "1", "tool.ns", None)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            optimize_operation_strand(&original, "1/2", "tool.ns", None)
                .unwrap_err()
                .0
                .code,
            "NSI002"
        );
    }
    #[test]
    fn recursive_and_variadic_function_records_round_trip() {
        for source in [
            "let f = (x) => add(index(1,x),index(2,f(x)))\noutput f",
            "let f = (xs...) => add(xs...)\noutput f",
        ] {
            let state = graph(source);
            let decoded = decode_operation_strand(&state, "tool.ns", None).unwrap();
            let catalog = decoded
                .functions
                .iter()
                .map(|f| (f.name.clone(), f))
                .collect();
            let rebuilt = evaluate(
                operation_strand(
                    &decoded.root,
                    &catalog,
                    &decoded.encoded_source,
                    "tool.ns",
                    None,
                )
                .unwrap(),
            );
            let second = decode_operation_strand(&rebuilt, "tool.ns", None).unwrap();
            assert_eq!(decoded.functions, second.functions);
        }
        let state = graph("let f = (x) => f(x)\noutput f");
        assert!(
            Value::State(State::from_projection(&state))
                .call(vec![Value::State(State::one())])
                .unwrap_err()
                .0
                .message
                .contains("call-depth limit")
        );
    }
    #[test]
    fn host_discovery_exports_a_native_program_without_a_language_builtin() {
        let samples =
            graph("output add(index(1,1),index(2,1),index(3,2),index(4,3),index(5,5),index(6,8))");
        let found = crate::discovery::discover(&samples, "1", "tool.ns", None).unwrap();
        let exported = evaluate(found.expression("tool.ns", None).unwrap());
        assert!(is_operation_strand(&exported));
        assert!(
            !decode_operation_strand(&exported, "tool.ns", None)
                .unwrap()
                .functions
                .is_empty()
        );
    }
    #[test]
    fn obsolete_records_and_invalid_phase_are_rejected() {
        for kind in [5, 6, 9, 14, 15, 16, 18, 19, 20, 21, 22] {
            let mut cursor = 0;
            decode_expression(
                &[Coordinate::new(kind, None)],
                &mut cursor,
                0,
                "invalid.ns",
                None,
            )
            .unwrap_err();
        }
        let mut phase = Coordinate::new(PHASE, None);
        phase.number_a = Some("4".into());
        decode_expression(&[phase], &mut 0, 0, "invalid.ns", None).unwrap_err();
        operation_length(&NativeState::one(), "invalid.ns", None).unwrap_err();
    }
    #[test]
    fn source_tools_do_not_silently_drop_partial_bindings() {
        let state = graph("let f = (a,b) => add(a,b)\noutput f(0)");
        assert!(
            optimize_operation_strand(&state, "1", "tool.ns", None)
                .unwrap_err()
                .0
                .message
                .contains("unbound")
        );
    }
}
