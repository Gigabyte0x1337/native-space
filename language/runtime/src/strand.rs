// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Encodes source functions as nested native operation strands.
//!
//! A trace is a reflective camera, not a fifth algebra operation. Every
//! returned strand is an expression containing only exact constants, ADD,
//! ORIENT, and INDEX. A node stores its instruction under the head coordinate
//! and stores the remainder of the strand under the continuation coordinate.

use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigUint;
use num_traits::{One as _, ToPrimitive as _, Zero as _};

use crate::core::{
    Diagnostic, Expr, Function, Goal, LanguageError, NativeScalar, NativeState, OutputKind,
    Program, Span, is_canonical_orientation,
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
const ZERO: u64 = 5;
const ONE: u64 = 6;
const SCALAR: u64 = 7;
const REFERENCE: u64 = 8;
const CALL: u64 = 9;
const ADD: u64 = 10;
const MULTIPLY: u64 = 11;
const ORIENT: u64 = 12;
const INDEX: u64 = 13;
const TRACE: u64 = 14;
const UNTRACE: u64 = 15;
const LENGTH: u64 = 16;
const SPREAD: u64 = 17;
const CONCAT: u64 = 18;
const FOLD: u64 = 19;
const CAMERA: u64 = 20;
const RANK_DESCENT: u64 = 21;
const APPLY: u64 = 22;

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

    let mut positions = BTreeSet::<BigUint>::new();
    for index in state.0.keys() {
        if index.depth(HEAD_DIRECTION) != BigUint::from(1_u8)
            || index.0.keys().any(|direction| {
                *direction < TRACE_DIRECTION_START || *direction > TEXT_POSITION_DIRECTION
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

/// Build the nested native strand returned by `trace(function)`.
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
pub(crate) fn optimize_operation_strand(
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

    let original_length = operation_length(state, source_name, span)?;
    let decoded = decode_operation_strand(state, source_name, span)?;
    let program = Program {
        functions: decoded.functions,
        bindings: Vec::new(),
        goal: Goal::Emit,
        output_kind: OutputKind::Pattern,
        result: Expr::Trace {
            function: decoded.root.clone(),
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
struct DecodedStrand {
    root: String,
    encoded_source: String,
    functions: Vec<Function>,
}

#[derive(Debug, Default)]
struct RawCoordinate {
    kind: Option<u64>,
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

fn decode_operation_strand(
    state: &NativeState,
    source_name: &str,
    span: Option<Span>,
) -> Result<DecodedStrand, LanguageError> {
    let length = operation_length(state, source_name, span)?;
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
    decode_coordinate_sequence(&coordinates, source_name, span)
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
                // The opcode orientation is redundant with the exact kind and
                // is validated by reconstruction rather than used as control.
                if coefficient.is_zero() {
                    return Err(malformed());
                }
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
        kind: raw
            .kind
            .ok_or_else(|| malformed_strand(source_name, span))?,
        opcode_turn: None,
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

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive instruction match keeps strand reconstruction auditable"
)]
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
        ZERO => Expr::Zero {
            span: coordinate.span,
        },
        ONE => Expr::One {
            span: coordinate.span,
        },
        SCALAR => Expr::Scalar {
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
        TRACE => Expr::Trace {
            function: required_text(coordinate.text_a.as_deref(), source_name, span)?.to_owned(),
            span: coordinate.span,
        },
        LENGTH => Expr::Length {
            value: Box::new(child(cursor)?),
            span: coordinate.span,
        },
        UNTRACE => Expr::Untrace {
            value: Box::new(child(cursor)?),
            rank: coordinate.number_b.clone().unwrap_or_else(|| "0".into()),
            span: coordinate.span,
        },
        RANK_DESCENT => {
            let argument_count = u64_or_zero(coordinate.number_a.as_deref(), source_name, span)?;
            let (target_rank, minimum_agreement) = match argument_count {
                1 => (None, None),
                2 => (
                    Some(coordinate.number_b.clone().unwrap_or_else(|| "0".into())),
                    None,
                ),
                3 => (
                    Some(coordinate.number_b.clone().unwrap_or_else(|| "0".into())),
                    Some(coordinate.number_c.clone().unwrap_or_else(|| "0".into())),
                ),
                _ => return Err(malformed_strand(source_name, span)),
            };
            Expr::RankDescent {
                value: Box::new(child(cursor)?),
                target_rank,
                minimum_agreement,
                span: coordinate.span,
            }
        }
        APPLY => Expr::Apply {
            pattern: Box::new(child(cursor)?),
            position: required_u64(coordinate.number_a.as_deref(), source_name, span)?,
            span: coordinate.span,
        },
        CALL => Expr::Call {
            function: required_text(coordinate.name.as_deref(), source_name, span)?.to_owned(),
            arguments: children(
                usize_or_zero(coordinate.number_a.as_deref(), source_name, span)?,
                cursor,
            )?,
            span: coordinate.span,
        },
        CONCAT => Expr::Concat {
            direction: required_u64(coordinate.number_a.as_deref(), source_name, span)?,
            values: children(
                usize_or_zero(coordinate.number_b.as_deref(), source_name, span)?,
                cursor,
            )?,
            span: coordinate.span,
        },
        FOLD => {
            let value_count = usize_or_zero(coordinate.number_a.as_deref(), source_name, span)?;
            let initial = child(cursor)?;
            Expr::Fold {
                function: required_text(coordinate.name.as_deref(), source_name, span)?.to_owned(),
                initial: Box::new(initial),
                values: children(value_count, cursor)?,
                span: coordinate.span,
            }
        }
        CAMERA => Expr::Camera {
            from_direction: required_u64(coordinate.number_a.as_deref(), source_name, span)?,
            to_direction: u64_or_zero(coordinate.number_b.as_deref(), source_name, span)?,
            value: Box::new(child(cursor)?),
            span: coordinate.span,
        },
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
        ORIENT => {
            let turns = i64_or_zero(coordinate.number_a.as_deref(), source_name, span)?;
            if !is_canonical_orientation(turns) {
                return Err(malformed_strand(source_name, span));
            }
            Expr::Orient {
                turns,
                value: Box::new(child(cursor)?),
                span: coordinate.span,
            }
        }
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

fn u64_or_zero(
    value: Option<&str>,
    source_name: &str,
    span: Option<Span>,
) -> Result<u64, LanguageError> {
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
    collect_expression(&function.body, coordinates, &mut called);

    let mut end = Coordinate::new(FUNCTION_END, function.span);
    end.name = Some(function.name.clone());
    coordinates.push(end);

    for (called_name, span) in called {
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

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive expression match keeps the trace schema auditable"
)]
fn collect_expression(
    expression: &Expr,
    coordinates: &mut Vec<Coordinate>,
    called: &mut Vec<(String, Option<Span>)>,
) {
    match expression {
        Expr::Zero { span } => coordinates.push(Coordinate::new(ZERO, *span)),
        Expr::One { span } => coordinates.push(Coordinate::new(ONE, *span)),
        Expr::Scalar { real, imag, span } => {
            let mut coordinate = Coordinate::new(SCALAR, *span);
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
        Expr::Trace { function, span } => {
            let mut coordinate = Coordinate::new(TRACE, *span);
            coordinate.text_a = Some(function.clone());
            coordinates.push(coordinate);
            called.push((function.clone(), *span));
        }
        Expr::Length { value, span } => {
            let mut coordinate = Coordinate::new(LENGTH, *span);
            coordinate.number_a = Some("1".into());
            coordinates.push(coordinate);
            collect_expression(value, coordinates, called);
        }
        Expr::Untrace { value, rank, span } => {
            let mut coordinate = Coordinate::new(UNTRACE, *span);
            coordinate.number_a = Some("2".into());
            coordinate.number_b = Some(rank.clone());
            coordinates.push(coordinate);
            collect_expression(value, coordinates, called);
        }
        Expr::RankDescent {
            value,
            target_rank,
            minimum_agreement,
            span,
        } => {
            let mut coordinate = Coordinate::new(RANK_DESCENT, *span);
            coordinate.number_a = Some(
                match (target_rank, minimum_agreement) {
                    (None, None) => "1",
                    (Some(_), None) => "2",
                    (Some(_), Some(_)) => "3",
                    (None, Some(_)) => unreachable!("agreement requires a target rank"),
                }
                .into(),
            );
            coordinate.number_b.clone_from(target_rank);
            coordinate.number_c.clone_from(minimum_agreement);
            coordinates.push(coordinate);
            collect_expression(value, coordinates, called);
        }
        Expr::Apply {
            pattern,
            position,
            span,
        } => {
            let mut coordinate = Coordinate::new(APPLY, *span);
            coordinate.number_a = Some(position.to_string());
            coordinates.push(coordinate);
            collect_expression(pattern, coordinates, called);
        }
        Expr::Call {
            function,
            arguments,
            span,
        } => {
            let mut coordinate = Coordinate::new(CALL, *span);
            coordinate.name = Some(function.clone());
            coordinate.number_a = Some(arguments.len().to_string());
            coordinates.push(coordinate);
            for argument in arguments {
                collect_expression(argument, coordinates, called);
            }
            called.push((function.clone(), *span));
        }
        Expr::Concat {
            direction,
            values,
            span,
        } => {
            let mut coordinate = Coordinate::new(CONCAT, *span);
            coordinate.number_a = Some(direction.to_string());
            coordinate.number_b = Some(values.len().to_string());
            coordinates.push(coordinate);
            for value in values {
                collect_expression(value, coordinates, called);
            }
        }
        Expr::Fold {
            function,
            initial,
            values,
            span,
        } => {
            let mut coordinate = Coordinate::new(FOLD, *span);
            coordinate.name = Some(function.clone());
            coordinate.number_a = Some(values.len().to_string());
            coordinates.push(coordinate);
            collect_expression(initial, coordinates, called);
            for value in values {
                collect_expression(value, coordinates, called);
            }
            called.push((function.clone(), *span));
        }
        Expr::Camera {
            from_direction,
            to_direction,
            value,
            span,
        } => {
            let mut coordinate = Coordinate::new(CAMERA, *span);
            coordinate.number_a = Some(from_direction.to_string());
            coordinate.number_b = Some(to_direction.to_string());
            coordinates.push(coordinate);
            collect_expression(value, coordinates, called);
        }
        Expr::Add { operands, span } => {
            let mut coordinate = Coordinate::new(ADD, *span);
            coordinate.opcode_turn = Some(0);
            coordinate.number_a = Some(operands.len().to_string());
            coordinates.push(coordinate);
            for operand in operands {
                collect_expression(operand, coordinates, called);
            }
        }
        Expr::Multiply { operands, span } => {
            let mut coordinate = Coordinate::new(MULTIPLY, *span);
            coordinate.opcode_turn = Some(1);
            coordinate.number_a = Some(operands.len().to_string());
            coordinates.push(coordinate);
            for operand in operands {
                collect_expression(operand, coordinates, called);
            }
        }
        Expr::Orient { turns, value, span } => {
            let mut coordinate = Coordinate::new(ORIENT, *span);
            coordinate.opcode_turn = Some(2);
            coordinate.number_a = Some(turns.to_string());
            coordinates.push(coordinate);
            collect_expression(value, coordinates, called);
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
            collect_expression(value, coordinates, called);
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
            Expr::Orient {
                turns,
                value: Box::new(Expr::One { span: None }),
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
    Expr::Scalar {
        real,
        imag: "0".into(),
        span: None,
    }
}

fn sum(mut expressions: Vec<Expr>) -> Expr {
    match expressions.len() {
        0 => Expr::Zero { span: None },
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

    #[test]
    fn strand_decoder_rejects_noncanonical_orientation() {
        let mut orientation = Coordinate::new(ORIENT, None);
        orientation.number_a = Some("4".into());
        let coordinates = [orientation, Coordinate::new(ONE, None)];
        let mut cursor = 0;

        let error = decode_expression(&coordinates, &mut cursor, 0, "strand.ns", None).unwrap_err();
        assert_eq!(error.0.code, "NSI003");
    }

    #[test]
    fn trace_is_a_nested_coordinate_strand_and_preserves_distinct_programs() {
        let first = parse(
            "let sample = (x) => add(x, orient(1, x))\noutput trace(sample) as pattern",
            "first.ns",
        )
        .unwrap();
        let second = parse(
            "let sample = (x) => multiply(x, orient(1, x))\noutput trace(sample) as pattern",
            "second.ns",
        )
        .unwrap();
        let first_state = interpret(&first).unwrap();
        let second_state = interpret(&second).unwrap();

        assert_ne!(first_state, NativeState::zero());
        assert_ne!(first_state, second_state);
        assert!(
            first_state
                .0
                .keys()
                .any(|index| index.depth(CONTINUATION_DIRECTION) > 0_u8.into())
        );
    }

    #[test]
    fn compiled_trace_observes_source_before_optimization() {
        let program = parse(
            "let sample = (x) => add(zero, x)\noutput trace(sample) as pattern",
            "source-before-optimization.ns",
        )
        .unwrap();
        let direct = interpret(&program).unwrap();
        let bytecode = crate::bytecode::compile(&program).unwrap();

        assert_eq!(crate::bytecode::execute(&bytecode).unwrap(), direct);
    }

    #[test]
    fn trace_is_pure_across_distinct_call_locations() {
        let program = parse(
            "let sample = (x) => orient(1, x)\ntrace(sample) = trace(sample)",
            "pure-trace.ns",
        )
        .unwrap();

        let direct = interpret(&program).unwrap();
        let bytecode = crate::bytecode::compile(&program).unwrap();
        assert!(direct.is_zero());
        assert_eq!(crate::bytecode::execute(&bytecode).unwrap(), direct);
    }

    #[test]
    fn trace_preserves_the_untrace_rank() {
        let fifth = parse(
            "let sample = (x) => untrace(x, 1/5)\noutput trace(sample) as pattern",
            "fifth.ns",
        )
        .unwrap();
        let quarter = parse(
            "let sample = (x) => untrace(x, 1/4)\noutput trace(sample) as pattern",
            "quarter.ns",
        )
        .unwrap();

        assert_ne!(interpret(&fifth).unwrap(), interpret(&quarter).unwrap());
    }

    #[test]
    fn trace_preserves_rank_descent_strategy() {
        let adaptive = parse(
            "let sample = (x) => rank_descent(x)\noutput trace(sample) as pattern",
            "adaptive.ns",
        )
        .unwrap();
        let static_exact = parse(
            "let sample = (x) => rank_descent(x, 1/4)\noutput trace(sample) as pattern",
            "static-exact.ns",
        )
        .unwrap();
        let static_lossy = parse(
            "let sample = (x) => rank_descent(x, 1/4, 99/100)\noutput trace(sample) as pattern",
            "static-lossy.ns",
        )
        .unwrap();

        assert_ne!(
            interpret(&adaptive).unwrap(),
            interpret(&static_exact).unwrap()
        );
        assert_ne!(
            interpret(&static_exact).unwrap(),
            interpret(&static_lossy).unwrap()
        );
    }

    #[test]
    fn trace_preserves_variadic_pack_and_concat_pattern_nodes() {
        let program = parse(
            "let parameters = (values...) => concat(9, values...)\n\
             output trace(parameters) as pattern",
            "variadic-trace.ns",
        )
        .unwrap();
        let strand = interpret(&program).unwrap();
        let kinds = strand
            .0
            .iter()
            .filter(|(index, _coefficient)| {
                index.depth(KIND_DIRECTION) == BigUint::from(1_u8)
                    && index.0.keys().all(|direction| {
                        matches!(
                            *direction,
                            HEAD_DIRECTION | CONTINUATION_DIRECTION | KIND_DIRECTION
                        )
                    })
            })
            .map(|(_index, coefficient)| coefficient)
            .collect::<Vec<_>>();

        assert!(kinds.contains(&&NativeScalar::from_text(&SPREAD.to_string(), "0").unwrap()));
        assert!(kinds.contains(&&NativeScalar::from_text(&CONCAT.to_string(), "0").unwrap()));
    }

    #[test]
    fn trace_preserves_fold_and_camera_pattern_nodes() {
        let program = parse(
            "let step = (left, right) => add(left, right)\n\
             let model = (values...) => camera(7, 0, fold(step, zero, values...))\n\
             output trace(model) as pattern",
            "model-trace.ns",
        )
        .unwrap();
        let strand = interpret(&program).unwrap();
        let kinds = strand
            .0
            .iter()
            .filter(|(index, _coefficient)| {
                index.depth(KIND_DIRECTION) == BigUint::from(1_u8)
                    && index.0.keys().all(|direction| {
                        matches!(
                            *direction,
                            HEAD_DIRECTION | CONTINUATION_DIRECTION | KIND_DIRECTION
                        )
                    })
            })
            .map(|(_index, coefficient)| coefficient)
            .collect::<Vec<_>>();

        assert!(kinds.contains(&&NativeScalar::from_text(&SPREAD.to_string(), "0").unwrap()));
        assert!(kinds.contains(&&NativeScalar::from_text(&FOLD.to_string(), "0").unwrap()));
        assert!(kinds.contains(&&NativeScalar::from_text(&CAMERA.to_string(), "0").unwrap()));
    }

    #[test]
    fn length_projects_trace_extent_to_native_index_depth() {
        let measured = parse(
            "let shorter = (value) => value\noutput length(trace(shorter)) as pattern",
            "measured-length.ns",
        )
        .unwrap();
        let measured = interpret(&measured).unwrap();
        let (index, coefficient) = measured.0.iter().next().unwrap();

        assert_eq!(measured.0.len(), 1);
        assert_eq!(index.depth(1), BigUint::from(5_u8));
        assert_eq!(coefficient, &NativeScalar::one());

        let source = "let shorter = (value) => value\n\
                      let longer = (value) => add(value, one)\n\
                      let saved = () => multiply(index(1, one), index(1, one))\n\
                      multiply(length(trace(shorter)), saved()) = length(trace(longer))";
        let program = parse(source, "length.ns").unwrap();
        let direct = interpret(&program).unwrap();
        let bytecode = crate::bytecode::compile(&program).unwrap();

        assert!(direct.is_zero());
        assert_eq!(crate::bytecode::execute(&bytecode).unwrap(), direct);
    }

    #[test]
    fn length_rejects_values_without_operation_strand_shape() {
        let program = parse("output length(one) as pattern", "not-strand.ns").unwrap();
        let error = interpret(&program).unwrap_err();

        assert_eq!(error.0.code, "NSL001");
        assert_eq!(error.0.span.unwrap().start_line, 1);
    }

    #[test]
    fn untrace_synthesizes_a_recursive_operation_strand() {
        let program = parse(
            "output untrace(add(index(1, 1), index(2, 1), index(3, 2), index(4, 3), index(5, 5), index(6, 8))) as pattern",
            "untrace.ns",
        )
        .unwrap();
        let direct = interpret(&program).unwrap();
        let bytecode = crate::bytecode::compile(&program).unwrap();

        assert!(is_operation_strand(&direct));
        assert_eq!(crate::bytecode::execute(&bytecode).unwrap(), direct);
    }

    #[test]
    fn untrace_keeps_a_strand_without_rewrite_opportunities() {
        let program = parse(
            "output untrace(untrace(add(index(1, 1), index(2, 1), index(3, 2), index(4, 3), index(5, 5)))) as pattern",
            "double-untrace.ns",
        )
        .unwrap();
        let direct = interpret(&program).unwrap();
        let bytecode = crate::bytecode::compile(&program).unwrap();

        assert!(is_operation_strand(&direct));
        assert_eq!(crate::bytecode::execute(&bytecode).unwrap(), direct);
    }

    #[test]
    fn untrace_rank_one_rebuilds_a_shorter_exact_instruction_strand() {
        let original = parse(
            "let redundant = (value) => add(add(value, zero), multiply(value, one))\n\
             output trace(redundant) as pattern",
            "instruction-original.ns",
        )
        .unwrap();
        let optimized = parse(
            "let redundant = (value) => add(add(value, zero), multiply(value, one))\n\
             output untrace(trace(redundant), 1) as pattern",
            "instruction-optimized.ns",
        )
        .unwrap();
        let original_state = interpret(&original).unwrap();
        let optimized_state = interpret(&optimized).unwrap();

        assert_eq!(operation_length(&original_state, "test", None).unwrap(), 11);
        assert_eq!(operation_length(&optimized_state, "test", None).unwrap(), 7);
        assert_eq!(
            crate::bytecode::execute(&crate::bytecode::compile(&optimized).unwrap()).unwrap(),
            optimized_state
        );

        let decoded = decode_operation_strand(&optimized_state, "test", None).unwrap();
        let reconstructed_call = Program {
            functions: decoded.functions,
            bindings: Vec::new(),
            goal: Goal::Emit,
            output_kind: OutputKind::Auto,
            result: Expr::Call {
                function: decoded.root,
                arguments: vec![Expr::Scalar {
                    real: "7".into(),
                    imag: "0".into(),
                    span: None,
                }],
                span: None,
            },
            source_name: "reconstructed-call.ns".into(),
            span: None,
        };
        let original_call = parse(
            "let redundant = (value) => add(add(value, zero), multiply(value, one))\n\
             output redundant(7)",
            "original-call.ns",
        )
        .unwrap();
        assert_eq!(
            interpret(&reconstructed_call).unwrap(),
            interpret(&original_call).unwrap()
        );
    }

    #[test]
    fn instruction_untrace_rejects_lossy_ranks() {
        let program = parse(
            "let redundant = (value) => add(value, zero)\n\
             output untrace(trace(redundant), 1/2) as pattern",
            "lossy-instruction-rank.ns",
        )
        .unwrap();
        let error = interpret(&program).unwrap_err();

        assert_eq!(error.0.code, "NSI002");
        assert_eq!(error.0.span.unwrap().start_line, 2);
    }

    #[test]
    fn instruction_untrace_keeps_an_irreducible_matrix_graph() {
        let source = "let matrix = (a, b, c, d) => add(index(2, a), index(3, b), index(4, c), index(5, d))\n\
                      let matrix_multiply = (left, right) => matrix(\n\
                        multiply(left, right), multiply(left, right),\n\
                        multiply(left, right), multiply(left, right)\n\
                      )\n";
        let original = parse(
            &format!("{source}output trace(matrix_multiply) as pattern"),
            "matrix.ns",
        )
        .unwrap();
        let optimized = parse(
            &format!("{source}output untrace(trace(matrix_multiply), 1) as pattern"),
            "matrix.ns",
        )
        .unwrap();

        assert_eq!(
            interpret(&optimized).unwrap(),
            interpret(&original).unwrap()
        );
    }

    #[test]
    fn instruction_untrace_preserves_recursive_output_and_continuation_branches() {
        let program = parse(
            "let repeat = (position, value) => add(\
               index(1, value), \
               index(2, repeat(index(1, position), value))\
             )\n\
             output untrace(trace(repeat), 1) as pattern",
            "recursive-instruction.ns",
        )
        .unwrap();
        let state = interpret(&program).unwrap();
        let decoded = decode_operation_strand(&state, "test", None).unwrap();
        let repeat = decoded
            .functions
            .iter()
            .find(|function| function.name == "repeat")
            .unwrap();
        let Expr::Add { operands, .. } = &repeat.body else {
            panic!("recursive pattern must retain its output and continuation branches");
        };
        assert_eq!(operands.len(), 2);
        assert!(matches!(operands[0], Expr::Index { direction: 1, .. }));
        let Expr::Index {
            direction: 2,
            value,
            ..
        } = &operands[1]
        else {
            panic!("second branch must retain the continuation coordinate");
        };
        let Expr::Call { function, .. } = value.as_ref() else {
            panic!("continuation coordinate must retain the recursive call edge");
        };
        assert_eq!(function, "repeat");
        assert_eq!(
            crate::bytecode::execute(&crate::bytecode::compile(&program).unwrap()).unwrap(),
            state
        );
    }

    #[test]
    fn traced_recursion_is_finite_but_executed_recursion_is_rejected() {
        let traced = parse(
            "let repeat = (x) => repeat(orient(1, x))\noutput trace(repeat) as pattern",
            "traced.ns",
        )
        .unwrap();
        let direct = interpret(&traced).unwrap();
        let bytecode = crate::bytecode::compile(&traced).unwrap();
        assert!(!direct.is_zero());
        assert_eq!(crate::bytecode::execute(&bytecode).unwrap(), direct);

        let executed = parse(
            "let repeat = (x) => repeat(orient(1, x))\noutput repeat(one)",
            "executed.ns",
        )
        .unwrap();
        let error = interpret(&executed).unwrap_err();
        assert_eq!(error.0.code, "NSS007");
    }
}
