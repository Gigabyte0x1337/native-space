// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Encodes source functions as nested native operation strands.
//!
//! A trace is a reflective camera, not a fifth algebra operation. Every
//! returned strand is an expression containing only exact constants, ADD,
//! ORIENT, and INDEX. A node stores its instruction under the head coordinate
//! and stores the remainder of the strand under the continuation coordinate.

use std::collections::{BTreeMap, BTreeSet};

use crate::core::{Diagnostic, Expr, Function, LanguageError, NativeScalar, NativeState, Span};

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

/// Return whether a state carries the operation-strand start marker.
#[must_use]
pub(crate) fn is_operation_strand(state: &NativeState) -> bool {
    let marker = crate::core::MultiIndex::from_depths([(HEAD_DIRECTION, 1), (KIND_DIRECTION, 1)])
        .expect("operation-strand directions are positive");
    state.0.get(&marker) == Some(&NativeScalar::one())
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
    Ok(nest(coordinates))
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
        Expr::Trace { function, span } => {
            let mut coordinate = Coordinate::new(TRACE, *span);
            coordinate.text_a = Some(function.clone());
            coordinates.push(coordinate);
            called.push((function.clone(), *span));
        }
        Expr::Untrace {
            value,
            maximum_error_ratio,
            span,
        } => {
            let mut coordinate = Coordinate::new(UNTRACE, *span);
            coordinate.number_a = Some("2".into());
            coordinate.number_b = Some(maximum_error_ratio.clone());
            coordinates.push(coordinate);
            collect_expression(value, coordinates, called);
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
    coordinates
        .into_iter()
        .rev()
        .fold(Expr::Zero { span: None }, |continuation, coordinate| {
            Expr::Add {
                operands: vec![
                    indexed(HEAD_DIRECTION, coordinate_expression(coordinate)),
                    indexed(CONTINUATION_DIRECTION, continuation),
                ],
                span: None,
            }
        })
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
    fn trace_preserves_the_untrace_error_ratio() {
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
    fn untrace_is_idempotent_for_an_existing_operation_strand() {
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
