// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-08-27

//! Runs one exact unary Native Space function over independent data points.
//!
//! Steps are sequential within a data point. CPU workers and GPU invocations
//! distribute only independent points, preserving input order.

use std::{collections::BTreeMap, fs, path::Path, str::FromStr as _, thread};

use num_bigint::BigUint;
use num_traits::{ToPrimitive as _, Zero as _};
use serde_json::{Value, json};

use crate::core::{
    Diagnostic, LanguageError, MultiIndex, NativeScalar, NativeState, OutputKind, Program,
    unary_function,
};

const BINARY_MAGIC: &[u8; 8] = b"NSBATCH\0";
const BINARY_VERSION: u16 = 1;
const NO_SHAPE: u16 = u16::MAX;
const MAX_ARRAY_RANK: usize = 64;

/// One ordered batch item and its requested host-array shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataPoint {
    state: NativeState,
    shape: Option<Vec<usize>>,
}

impl DataPoint {
    /// Return the exact native state supplied to the unary function.
    #[must_use]
    pub const fn state(&self) -> &NativeState {
        &self.state
    }

    /// Consume the host data point and return its complete native state.
    #[must_use]
    pub fn into_state(self) -> NativeState {
        self.state
    }

    /// Return the host-array shape retained for readable output.
    #[must_use]
    pub fn shape(&self) -> Option<&[usize]> {
        self.shape.as_deref()
    }
}

/// Read an ordered JSON or Native Space binary batch of exact data points.
///
/// An item may be a real rational string, a scalar object with `real` and
/// `imag` strings, one canonical flat-stack state object, or a nonempty
/// rectangular array of any rank up to 64.
///
/// # Errors
///
/// Returns `NSB003` when the file, JSON, or an item is invalid.
pub fn read_data(path: impl AsRef<Path>) -> Result<Vec<DataPoint>, LanguageError> {
    let path = path.as_ref();
    let source_name = path.display().to_string();
    let source = fs::read(path).map_err(|error| {
        batch_error(
            "NSB003",
            format!("could not read batch data: {error}"),
            &source_name,
        )
    })?;
    if source.starts_with(BINARY_MAGIC) {
        return decode_binary(&source).map_err(|message| {
            batch_error(
                "NSB003",
                format!("invalid Native Space batch binary: {message}"),
                &source_name,
            )
        });
    }
    let source = std::str::from_utf8(&source).map_err(|error| {
        batch_error(
            "NSB003",
            format!("batch JSON is not UTF-8: {error}"),
            &source_name,
        )
    })?;
    read_json_data(source).map_err(|message| batch_error("NSB003", message, &source_name))
}

/// Pack JSON batch data into the versioned Native Space binary format.
///
/// # Errors
///
/// Returns `NSB003` for invalid input data and `NSB005` when the binary output
/// cannot be encoded or written.
pub fn pack_data(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<usize, LanguageError> {
    let input = input.as_ref();
    let output = output.as_ref();
    let inputs = read_data(input)?;
    let bytes = encode_binary(&inputs).map_err(|message| {
        batch_error(
            "NSB005",
            format!("could not encode batch binary: {message}"),
            &output.display().to_string(),
        )
    })?;
    fs::write(output, bytes).map_err(|error| {
        batch_error(
            "NSB005",
            format!("could not write batch binary: {error}"),
            &output.display().to_string(),
        )
    })?;
    Ok(inputs.len())
}

fn read_json_data(source: &str) -> Result<Vec<DataPoint>, String> {
    let value = serde_json::from_str::<Value>(source)
        .map_err(|error| format!("invalid batch JSON: {error}"))?;
    let items = value
        .as_array()
        .ok_or("batch data must be one ordered JSON array")?;
    items
        .iter()
        .enumerate()
        .map(|(position, item)| {
            data_point(item)
                .map_err(|message| format!("invalid data item {}: {message}", position + 1))
        })
        .collect()
}

/// Evaluate each input independently on bounded CPU workers.
///
/// # Errors
///
/// Returns the first language diagnostic in input order, or `NSB004` if a
/// worker terminates unexpectedly.
pub fn execute_cpu(
    program: &Program,
    function_name: &str,
    inputs: &[DataPoint],
    steps: u64,
) -> Result<Vec<NativeState>, LanguageError> {
    let function = unary_function(program, function_name)?;
    if inputs.is_empty() {
        return Ok(Vec::new());
    }
    let available = thread::available_parallelism().map_or(1, usize::from);
    let workers = available.min(inputs.len());
    let chunk_size = inputs.len().div_ceil(workers);
    let chunks = thread::scope(|scope| {
        inputs
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(|| {
                    chunk
                        .iter()
                        .map(|input| function.apply(input.state.clone(), steps))
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(std::thread::ScopedJoinHandle::join)
            .collect::<Vec<_>>()
    });

    let mut results = Vec::with_capacity(inputs.len());
    for chunk in chunks {
        let chunk = chunk.map_err(|_panic| {
            batch_error(
                "NSB004",
                "a CPU batch worker terminated before rejoining",
                &program.source_name,
            )
        })?;
        for result in chunk {
            results.push(result?);
        }
    }
    Ok(results)
}

/// Create the stable JSON output for one batch run.
#[must_use]
pub fn output_data(
    backend: &str,
    steps: u64,
    inputs: &[DataPoint],
    results: &[NativeState],
) -> Value {
    json!({
        "backend": backend,
        "steps": steps,
        "results": inputs.iter().zip(results).map(|(input, result)| readable_value(result, input.shape.as_deref())).collect::<Vec<_>>()
    })
}

fn readable_value(state: &NativeState, shape: Option<&[usize]>) -> Value {
    if let Some(shape) = shape
        && let Some(array) = readable_array(state, shape)
    {
        return array;
    }
    crate::core::output_data(state, OutputKind::Auto)
        .ok()
        .and_then(|output| output.get("value").cloned())
        .unwrap_or_else(|| state.to_data())
}

fn data_point(value: &Value) -> Result<DataPoint, String> {
    if value.is_array() {
        let mut terms = Vec::new();
        let mut path = Vec::new();
        let shape = lower_array(value, 1, &mut path, &mut terms)?;
        return Ok(DataPoint {
            state: NativeState::from_terms(terms),
            shape: Some(shape),
        });
    }
    if let Some(real) = value.as_str() {
        return NativeScalar::from_text(real, "0").map(|scalar| DataPoint {
            state: NativeState::scalar(scalar),
            shape: None,
        });
    }
    if value.get("camera").is_some() {
        return NativeState::from_data(value).map(|state| DataPoint { state, shape: None });
    }
    NativeScalar::from_data(value).map(|scalar| DataPoint {
        state: NativeState::scalar(scalar),
        shape: None,
    })
}

fn lower_array(
    value: &Value,
    axis: usize,
    path: &mut Vec<(u64, u64)>,
    terms: &mut Vec<(MultiIndex, NativeScalar)>,
) -> Result<Vec<usize>, String> {
    if axis > MAX_ARRAY_RANK {
        return Err(format!("array rank exceeds {MAX_ARRAY_RANK}"));
    }
    let values = value.as_array().ok_or("array level must be an array")?;
    if values.is_empty() {
        return Err("arrays must be nonempty".into());
    }
    let direction = u64::try_from(axis).map_err(|_capacity_error| "array rank exceeds u64")?;
    let mut child_shape = None;
    for (position, child) in values.iter().enumerate() {
        let depth =
            u64::try_from(position + 1).map_err(|_capacity_error| "array position exceeds u64")?;
        path.push((direction, depth));
        let current_shape = if child.is_array() {
            lower_array(child, axis + 1, path, terms)?
        } else {
            let scalar = scalar_value(child)?;
            let index = MultiIndex::from_depths(path.iter().copied())?;
            terms.push((index, scalar));
            Vec::new()
        };
        path.pop();
        if let Some(expected) = &child_shape {
            if expected != &current_shape {
                return Err("arrays must be rectangular and have one consistent rank".into());
            }
        } else {
            child_shape = Some(current_shape);
        }
    }
    let mut shape = vec![values.len()];
    shape.extend(child_shape.unwrap_or_default());
    Ok(shape)
}

fn scalar_value(value: &Value) -> Result<NativeScalar, String> {
    value.as_str().map_or_else(
        || NativeScalar::from_data(value),
        |real| NativeScalar::from_text(real, "0"),
    )
}

fn readable_array(state: &NativeState, shape: &[usize]) -> Option<Value> {
    let element_count = shape
        .iter()
        .try_fold(1_usize, |count, &extent| count.checked_mul(extent))?;
    let mut values = vec![Value::String("0".into()); element_count];
    for (index, scalar) in &state.0 {
        if index.0.len() != shape.len() {
            return None;
        }
        let mut offset = 0_usize;
        for (axis, &extent) in shape.iter().enumerate() {
            let direction = u64::try_from(axis + 1).ok()?;
            let position = index.0.get(&direction)?.to_usize()?;
            if position == 0 || position > extent {
                return None;
            }
            offset = offset.checked_mul(extent)?.checked_add(position - 1)?;
        }
        values[offset] = readable_scalar(scalar)?;
    }
    Some(nest_values(&values, shape))
}

fn readable_scalar(scalar: &NativeScalar) -> Option<Value> {
    let data = scalar.to_data();
    if scalar.imag.is_zero() {
        return Some(Value::String(data.get("real")?.as_str()?.into()));
    }
    Some(data)
}

fn nest_values(values: &[Value], shape: &[usize]) -> Value {
    if shape.len() == 1 {
        return Value::Array(values.to_vec());
    }
    let stride = shape[1..].iter().product::<usize>();
    Value::Array(
        values
            .chunks(stride)
            .map(|chunk| nest_values(chunk, &shape[1..]))
            .collect(),
    )
}

fn encode_binary(inputs: &[DataPoint]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(BINARY_MAGIC);
    write_u16(&mut bytes, BINARY_VERSION);
    write_u16(&mut bytes, 0);
    write_u64(
        &mut bytes,
        u64::try_from(inputs.len()).map_err(|_capacity_error| "batch has too many items")?,
    );
    for input in inputs {
        if let Some(shape) = &input.shape {
            if shape.is_empty() || shape.len() > MAX_ARRAY_RANK {
                return Err("array shape rank is outside the binary format limit".into());
            }
            write_u16(
                &mut bytes,
                u16::try_from(shape.len()).map_err(|_capacity_error| "array rank exceeds u16")?,
            );
            for &extent in shape {
                if extent == 0 {
                    return Err("array shape extents must be positive".into());
                }
                write_u64(
                    &mut bytes,
                    u64::try_from(extent).map_err(|_capacity_error| "array extent exceeds u64")?,
                );
            }
        } else {
            write_u16(&mut bytes, NO_SHAPE);
        }
        write_u64(
            &mut bytes,
            u64::try_from(input.state.0.len())
                .map_err(|_capacity_error| "native state has too many terms")?,
        );
        for (index, scalar) in &input.state.0 {
            write_u16(
                &mut bytes,
                u16::try_from(index.0.len())
                    .map_err(|_capacity_error| "native index has too many directions")?,
            );
            for (&direction, depth) in &index.0 {
                write_u64(&mut bytes, direction);
                write_text(&mut bytes, &depth.to_string())?;
            }
            let scalar = scalar.to_data();
            write_text(
                &mut bytes,
                scalar["real"]
                    .as_str()
                    .ok_or("native scalar real coordinate is not text")?,
            )?;
            write_text(
                &mut bytes,
                scalar["imag"]
                    .as_str()
                    .ok_or("native scalar imaginary coordinate is not text")?,
            )?;
        }
    }
    Ok(bytes)
}

fn decode_binary(source: &[u8]) -> Result<Vec<DataPoint>, String> {
    let mut reader = BinaryReader::new(source);
    if reader.read_exact(BINARY_MAGIC.len(), "magic")? != BINARY_MAGIC {
        return Err("binary magic does not match Native Space batch data".into());
    }
    let version = reader.read_u16("version")?;
    if version != BINARY_VERSION {
        return Err(format!(
            "unsupported batch binary version {version}; expected {BINARY_VERSION}"
        ));
    }
    if reader.read_u16("flags")? != 0 {
        return Err("batch binary contains unsupported flags".into());
    }
    let item_count = reader.read_usize("item count")?;
    let mut inputs = Vec::with_capacity(item_count.min(reader.remaining() / 10));
    for item_position in 0..item_count {
        let shape_rank = reader.read_u16("shape rank")?;
        let shape = if shape_rank == NO_SHAPE {
            None
        } else {
            let rank = usize::from(shape_rank);
            if rank == 0 || rank > MAX_ARRAY_RANK {
                return Err(format!(
                    "item {} has an array rank outside 1..={MAX_ARRAY_RANK}",
                    item_position + 1
                ));
            }
            let mut shape = Vec::with_capacity(rank);
            for _ in 0..rank {
                let extent = reader.read_usize("shape extent")?;
                if extent == 0 {
                    return Err(format!(
                        "item {} has a zero array extent",
                        item_position + 1
                    ));
                }
                shape.push(extent);
            }
            shape.iter().try_fold(1_usize, |count, &extent| {
                count
                    .checked_mul(extent)
                    .ok_or("array element count exceeds usize")
            })?;
            Some(shape)
        };
        let term_count = reader.read_usize("term count")?;
        let mut terms = BTreeMap::new();
        for _ in 0..term_count {
            let index_count = usize::from(reader.read_u16("index direction count")?);
            let mut depths = BTreeMap::new();
            for _ in 0..index_count {
                let direction = reader.read_u64("index direction")?;
                if direction == 0 {
                    return Err("binary index directions must be positive".into());
                }
                let depth_text = reader.read_text("index depth")?;
                let depth = BigUint::from_str(depth_text)
                    .map_err(|_parse_error| "binary index depth is not a decimal integer")?;
                if depth.is_zero() {
                    return Err("binary index depths must be positive".into());
                }
                if depths.insert(direction, depth).is_some() {
                    return Err("binary index repeats one direction".into());
                }
            }
            let real = reader.read_text("real scalar coordinate")?;
            let imag = reader.read_text("imaginary scalar coordinate")?;
            let scalar = NativeScalar::from_text(real, imag)?;
            if scalar.is_zero() {
                return Err("binary native states must omit zero terms".into());
            }
            if terms.insert(MultiIndex(depths), scalar).is_some() {
                return Err("binary native state repeats one index".into());
            }
        }
        let state = NativeState(terms);
        if let Some(shape) = &shape
            && readable_array(&state, shape).is_none()
        {
            return Err(format!(
                "item {} contains an index outside its declared array shape",
                item_position + 1
            ));
        }
        inputs.push(DataPoint { state, shape });
    }
    if reader.remaining() != 0 {
        return Err("batch binary has trailing bytes".into());
    }
    Ok(inputs)
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_text(output: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let length = u32::try_from(value.len()).map_err(|_capacity_error| "text exceeds u32 bytes")?;
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

#[derive(Debug)]
struct BinaryReader<'a> {
    source: &'a [u8],
    position: usize,
}

impl<'a> BinaryReader<'a> {
    const fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    fn remaining(&self) -> usize {
        self.source.len().saturating_sub(self.position)
    }

    fn read_exact(&mut self, length: usize, field: &str) -> Result<&'a [u8], String> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| format!("{field} length overflows usize"))?;
        let value = self
            .source
            .get(self.position..end)
            .ok_or_else(|| format!("batch binary ends inside {field}"))?;
        self.position = end;
        Ok(value)
    }

    fn read_u16(&mut self, field: &str) -> Result<u16, String> {
        let bytes = self.read_exact(size_of::<u16>(), field)?;
        Ok(u16::from_le_bytes(
            bytes
                .try_into()
                .expect("read_exact returned one u16 byte width"),
        ))
    }

    fn read_u64(&mut self, field: &str) -> Result<u64, String> {
        let bytes = self.read_exact(size_of::<u64>(), field)?;
        Ok(u64::from_le_bytes(
            bytes
                .try_into()
                .expect("read_exact returned one u64 byte width"),
        ))
    }

    fn read_usize(&mut self, field: &str) -> Result<usize, String> {
        usize::try_from(self.read_u64(field)?)
            .map_err(|_capacity_error| format!("{field} exceeds usize"))
    }

    fn read_text(&mut self, field: &str) -> Result<&'a str, String> {
        let length_bytes = self.read_exact(size_of::<u32>(), &format!("{field} length"))?;
        let length = usize::try_from(u32::from_le_bytes(
            length_bytes
                .try_into()
                .expect("read_exact returned one u32 byte width"),
        ))
        .map_err(|_capacity_error| format!("{field} length exceeds usize"))?;
        let bytes = self.read_exact(length, field)?;
        std::str::from_utf8(bytes).map_err(|error| format!("{field} is not UTF-8: {error}"))
    }
}

fn batch_error(code: &str, message: impl Into<String>, source_name: &str) -> LanguageError {
    LanguageError(Diagnostic {
        code: code.into(),
        message: message.into(),
        source_name: source_name.into(),
        span: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::parse;

    #[test]
    fn cpu_batch_keeps_points_independent_and_steps_sequential() {
        let program = parse(
            "let step = (value) => add(multiply(value, 2), 1)\noutput 0",
            "batch.ns",
        )
        .unwrap();
        let inputs = ["1", "2", "3"].map(|value| DataPoint {
            state: NativeState::scalar(NativeScalar::from_text(value, "0").unwrap()),
            shape: None,
        });

        let results = execute_cpu(&program, "step", &inputs, 3).unwrap();
        let expected = ["15", "23", "31"]
            .map(|value| NativeState::scalar(NativeScalar::from_text(value, "0").unwrap()));

        assert_eq!(results, expected);
    }

    #[test]
    fn zero_steps_preserve_every_input() {
        let program = parse("let step = (value) => add(value, 1)\noutput 0", "batch.ns").unwrap();
        let inputs =
            [NativeState::one(), NativeState::zero()].map(|state| DataPoint { state, shape: None });

        assert_eq!(
            execute_cpu(&program, "step", &inputs, 0).unwrap(),
            inputs
                .iter()
                .map(|input| input.state.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn one_dimensional_array_uses_one_axis_and_position_depth() {
        let point = data_point(&json!(["3", "4", "5"])).unwrap();

        assert_eq!(point.shape(), Some([3].as_slice()));
        assert_eq!(
            readable_array(&point.state, point.shape().unwrap()),
            Some(json!(["3", "4", "5"]))
        );
        assert_eq!(
            point
                .state
                .0
                .get(&MultiIndex::from_depths([(1, 2)]).unwrap()),
            Some(&NativeScalar::from_text("4", "0").unwrap())
        );
    }

    #[test]
    fn multidimensional_positions_do_not_collide() {
        let point = data_point(&json!([["1", "2"], ["3", "4"]])).unwrap();
        let upper_right = MultiIndex::from_depths([(1, 1), (2, 2)]).unwrap();
        let lower_left = MultiIndex::from_depths([(1, 2), (2, 1)]).unwrap();

        assert_ne!(upper_right, lower_left);
        assert_eq!(point.state.0.len(), 4);
        assert_eq!(
            point.state.0.get(&upper_right),
            Some(&NativeScalar::from_text("2", "0").unwrap())
        );
        assert_eq!(
            point.state.0.get(&lower_left),
            Some(&NativeScalar::from_text("3", "0").unwrap())
        );
        assert_eq!(
            readable_array(&point.state, point.shape().unwrap()),
            Some(json!([["1", "2"], ["3", "4"]]))
        );
    }

    #[test]
    fn three_dimensional_complex_array_round_trips() {
        let value = json!([[["1", {"real": "2", "imag": "3"}]], [["4", "5"]]]);
        let point = data_point(&value).unwrap();

        assert_eq!(point.shape(), Some([2, 1, 2].as_slice()));
        assert_eq!(
            readable_array(&point.state, point.shape().unwrap()),
            Some(value)
        );
    }

    #[test]
    fn ragged_mixed_rank_and_empty_arrays_are_rejected() {
        for value in [
            json!([["1"], ["2", "3"]]),
            json!(["1", ["2"]]),
            json!([]),
            json!([["1"], []]),
        ] {
            data_point(&value).unwrap_err();
        }
    }

    #[test]
    fn coordinate_shape_survives_zero_cancellation() {
        let input = data_point(&json!(["1", "-1"])).unwrap();
        let result = data_point(&json!(["2", "0"])).unwrap().state;

        assert_eq!(
            output_data("cpu", 1, &[input], &[result])["results"],
            json!([["2", "0"]])
        );
    }

    #[test]
    fn binary_round_trip_preserves_exact_states_and_shapes() {
        let inputs = read_json_data(
            r#"[
                "7/3",
                {"real":"1/2","imag":"-3/4"},
                [["0","2"],["3","4"]]
            ]"#,
        )
        .unwrap();

        let encoded = encode_binary(&inputs).unwrap();
        let decoded = decode_binary(&encoded).unwrap();
        let input_states = inputs
            .iter()
            .map(|input| input.state.clone())
            .collect::<Vec<_>>();
        let decoded_states = decoded
            .iter()
            .map(|input| input.state.clone())
            .collect::<Vec<_>>();

        assert_eq!(decoded, inputs);
        assert_eq!(
            output_data("cpu", 0, &decoded, &decoded_states),
            output_data("cpu", 0, &inputs, &input_states)
        );
    }

    #[test]
    fn binary_decoder_rejects_version_and_trailing_data() {
        let inputs = read_json_data(r#"[[["1","2"]]]"#).unwrap();
        let mut wrong_version = encode_binary(&inputs).unwrap();
        wrong_version[BINARY_MAGIC.len()] = 2;
        decode_binary(&wrong_version).unwrap_err();

        let mut trailing = encode_binary(&inputs).unwrap();
        trailing.push(0);
        decode_binary(&trailing).unwrap_err();
    }
}
