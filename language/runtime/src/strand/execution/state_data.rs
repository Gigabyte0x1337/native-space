// SPDX-License-Identifier: AGPL-3.0-or-later

//! Portable argument graphs use the same addressed Native records as programs.
//! Metadata for a scalar or compiled rule is a leaf; operation and scope edges
//! remain explicit references. No argument projection is needed to encode it.
use super::{
    Coordinate, LanguageError, MAX_RECORDS, NativeState, State, decode_coordinates, diagnostic,
    literal, nest, required_text, usize_or_zero,
};
use crate::retained::{Operation, Step};

const STATE_ROOT: u64 = 28;
const STATE_NODE: u64 = 29;
const INPUT: u64 = 30;
const SCOPE: u64 = 31;

pub(super) fn encode(state: &State) -> Result<State, LanguageError> {
    let plan = state.plan();
    let mut header = Coordinate::new(STATE_ROOT, None);
    header.number_a = Some(plan.len().to_string());
    let mut records = vec![header];
    for node in plan {
        let mut record = Coordinate::new(STATE_NODE, node.span);
        // Operation parameters contain no child graphs; edges are separate records.
        record.text_a = Some(
            serde_json::to_string(&node.operator)
                .map_err(|error| diagnostic(error.to_string(), "<bindings>", node.span))?,
        );
        record.number_a = Some(node.inputs.len().to_string());
        record.number_b = Some(node.retained.len().to_string());
        records.push(record);
        for (kind, edges) in [(INPUT, node.inputs), (SCOPE, node.retained)] {
            for edge in edges {
                let mut reference = Coordinate::new(kind, None);
                reference.number_a = Some(edge.to_string());
                records.push(reference);
            }
        }
    }
    literal(&nest(records))
}

pub(super) fn decode(state: &NativeState) -> Result<State, LanguageError> {
    let source = "<bindings>";
    let records = decode_coordinates(state, source, None)?;
    let invalid = || diagnostic("invalid retained argument graph", source, None);
    let header = records.first().ok_or_else(invalid)?;
    if header.kind != STATE_ROOT {
        return Err(invalid());
    }
    let count = usize_or_zero(header.number_a.as_deref(), source, None)?;
    if count == 0 || count > records.len() || records.len() > MAX_RECORDS {
        return Err(invalid());
    }
    let mut cursor = 1;
    let mut nodes = Vec::new();
    for _ in 0..count {
        let record = records.get(cursor).ok_or_else(invalid)?;
        cursor += 1;
        if record.kind != STATE_NODE {
            return Err(invalid());
        }
        let operator: Operation = serde_json::from_str(required_text(
            record.text_a.as_deref(),
            source,
            record.span,
        )?)
        .map_err(|_error| invalid())?;
        let mut edges = |kind, count| -> Result<Vec<usize>, LanguageError> {
            let count = usize_or_zero(count, source, record.span)?;
            if count > records.len().saturating_sub(cursor) {
                return Err(invalid());
            }
            let mut result = Vec::new();
            for _ in 0..count {
                let edge = &records[cursor];
                cursor += 1;
                if edge.kind != kind {
                    return Err(invalid());
                }
                result.push(usize_or_zero(edge.number_a.as_deref(), source, edge.span)?);
            }
            Ok(result)
        };
        let inputs = edges(INPUT, record.number_a.as_deref())?;
        let retained = edges(SCOPE, record.number_b.as_deref())?;
        nodes.push(Step {
            operator,
            inputs,
            retained,
            span: record.span,
        });
    }
    if cursor != records.len() {
        return Err(invalid());
    }
    State::from_steps(&nodes, nodes.len() - 1).map_err(|message| diagnostic(message, source, None))
}
