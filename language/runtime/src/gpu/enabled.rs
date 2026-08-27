// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-08-27

//! Enabled GPU backend for the supported signed-32-bit scalar subset.
//!
//! One compute invocation owns one input point and performs its requested
//! steps sequentially. Invocations are independent and therefore execute in
//! parallel. Unsupported states, unsupported operations, unavailable adapters,
//! and arithmetic overflow are errors; this backend never falls back to CPU or
//! substitutes floating-point arithmetic.

use std::{borrow::Cow, sync::mpsc};

use num_traits::{One as _, ToPrimitive as _, Zero as _};
use wgpu::util::DeviceExt as _;

use crate::batch::DataPoint;
use crate::core::{
    Diagnostic, Expr, LanguageError, NativeScalar, NativeState, Program, expanded_unary_expression,
};

const MAX_GPU_STEPS: u64 = 1_000_000;
const WORKGROUP_SIZE: u32 = 64;

/// Exact results and the physical adapter that executed them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GpuBatchResult {
    pub results: Vec<NativeState>,
    pub adapter_name: String,
}

/// Execute independent exact integer-scalar points on a GPU.
///
/// The selected function may use exact integer constants, ADD, MULTIPLY, and
/// even ORIENT turns. Every intermediate value must stay in signed 32-bit
/// range. INDEX, odd ORIENT turns, rational or complex values, reflective
/// forms, and non-scalar states are rejected rather than approximated.
///
/// # Errors
///
/// Returns an `NSG` diagnostic for unsupported semantics, unavailable GPU
/// facilities, device failures, or exact integer overflow.
pub async fn execute(
    program: &Program,
    function_name: &str,
    inputs: &[DataPoint],
    steps: u64,
) -> Result<GpuBatchResult, LanguageError> {
    if steps > MAX_GPU_STEPS {
        return Err(gpu_error(
            "NSG005",
            format!("GPU steps exceed the explicit limit of {MAX_GPU_STEPS}"),
            &program.source_name,
        ));
    }
    let steps = u32::try_from(steps).map_err(|_capacity_error| {
        gpu_error(
            "NSG005",
            "GPU step count does not fit the exact backend",
            &program.source_name,
        )
    })?;
    let expression = expanded_unary_expression(program, function_name)?;
    let shader = shader_source(&expression, &program.source_name)?;
    let integers = inputs
        .iter()
        .enumerate()
        .map(|(position, input)| {
            scalar_i32(input.state()).map_err(|message| {
                gpu_error(
                    "NSG002",
                    format!("unsupported GPU data item {}: {message}", position + 1),
                    &program.source_name,
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if integers.is_empty() {
        return Ok(GpuBatchResult {
            results: Vec::new(),
            adapter_name: "no dispatch: empty data".into(),
        });
    }

    let (device, queue, adapter_name) = gpu_device(&program.source_name).await?;
    let count = u32::try_from(integers.len()).map_err(|_capacity_error| {
        gpu_error(
            "NSG003",
            "the GPU data set has more than 4,294,967,295 points",
            &program.source_name,
        )
    })?;
    let (output_values, error_values) = dispatch(
        &device,
        &queue,
        &shader,
        &integers,
        steps,
        count,
        &program.source_name,
    )?;
    if let Some(position) = error_values.iter().position(|flag| *flag != 0) {
        return Err(gpu_error(
            "NSG004",
            format!(
                "exact signed-32-bit overflow at data item {}; no approximate result was returned",
                position + 1
            ),
            &program.source_name,
        ));
    }
    let results = output_values
        .into_iter()
        .map(|value| {
            NativeScalar::from_text(&value.to_string(), "0")
                .map(NativeState::scalar)
                .map_err(|message| gpu_error("NSG001", message, &program.source_name))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GpuBatchResult {
        results,
        adapter_name,
    })
}

async fn gpu_device(
    source_name: &str,
) -> Result<(wgpu::Device, wgpu::Queue, String), LanguageError> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        })
        .await
        .map_err(|error| {
            gpu_error(
                "NSG001",
                format!("no compatible GPU adapter: {error}"),
                source_name,
            )
        })?;
    let adapter_name = adapter.get_info().name;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("Native Space exact batch device"),
            ..Default::default()
        })
        .await
        .map_err(|error| {
            gpu_error(
                "NSG001",
                format!("could not create the GPU device: {error}"),
                source_name,
            )
        })?;

    Ok((device, queue, adapter_name))
}

fn dispatch(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shader: &str,
    integers: &[i32],
    steps: u32,
    count: u32,
    source_name: &str,
) -> Result<(Vec<i32>, Vec<u32>), LanguageError> {
    let parameters = [steps, count, 0, 0];
    let parameter_buffer = buffer(
        device,
        "Native Space batch parameters",
        bytemuck::cast_slice(&parameters),
        wgpu::BufferUsages::UNIFORM,
    );
    let input_buffer = buffer(
        device,
        "Native Space batch inputs",
        bytemuck::cast_slice(integers),
        wgpu::BufferUsages::STORAGE,
    );
    let output_size = u64::from(count)
        * u64::try_from(size_of::<i32>()).map_err(|_capacity_error| {
            gpu_error("NSG003", "invalid GPU scalar size", source_name)
        })?;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Native Space batch outputs"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let error_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Native Space batch overflow flags"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let output_readback = readback_buffer(device, "Native Space output readback", output_size);
    let error_readback = readback_buffer(device, "Native Space error readback", output_size);

    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Native Space exact batch shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader)),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Native Space exact batch pipeline"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let layout = pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Native Space exact batch bindings"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: parameter_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: error_buffer.as_entire_binding(),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Native Space exact batch commands"),
    });
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Native Space exact batch pass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(count.div_ceil(WORKGROUP_SIZE), 1, 1);
    drop(pass);
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &output_readback, 0, output_size);
    encoder.copy_buffer_to_buffer(&error_buffer, 0, &error_readback, 0, output_size);
    queue.submit([encoder.finish()]);

    Ok((
        read_i32(device, &output_readback, source_name)?,
        read_u32(device, &error_readback, source_name)?,
    ))
}

fn buffer(
    device: &wgpu::Device,
    label: &str,
    contents: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents,
        usage,
    })
}

fn readback_buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn read_i32(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    source_name: &str,
) -> Result<Vec<i32>, LanguageError> {
    read_bytes(device, buffer, source_name).map(|bytes| bytemuck::cast_slice(&bytes).to_vec())
}

fn read_u32(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    source_name: &str,
) -> Result<Vec<u32>, LanguageError> {
    read_bytes(device, buffer, source_name).map(|bytes| bytemuck::cast_slice(&bytes).to_vec())
}

fn read_bytes(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    source_name: &str,
) -> Result<Vec<u8>, LanguageError> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| gpu_error("NSG001", format!("GPU wait failed: {error}"), source_name))?;
    receiver
        .recv()
        .map_err(|error| {
            gpu_error(
                "NSG001",
                format!("GPU readback stopped: {error}"),
                source_name,
            )
        })?
        .map_err(|error| {
            gpu_error(
                "NSG001",
                format!("GPU readback failed: {error}"),
                source_name,
            )
        })?;
    let view = slice.get_mapped_range().map_err(|error| {
        gpu_error(
            "NSG001",
            format!("GPU mapped range failed: {error}"),
            source_name,
        )
    })?;
    let bytes = view.to_vec();
    drop(view);
    buffer.unmap();
    Ok(bytes)
}

fn scalar_i32(state: &NativeState) -> Result<i32, String> {
    if state.is_zero() {
        return Ok(0);
    }
    if state.0.len() != 1 {
        return Err("GPU input must be one scalar, not a multi-term pattern".into());
    }
    let (index, scalar) = state
        .0
        .first_key_value()
        .expect("nonzero state has one term");
    if !index.0.is_empty() {
        return Err("GPU input must not carry INDEX coordinates".into());
    }
    if !scalar.imag.is_zero() || !scalar.real.denom().is_one() {
        return Err("GPU input must be one real integer".into());
    }
    scalar
        .real
        .numer()
        .to_i32()
        .ok_or_else(|| "GPU input is outside signed 32-bit range".into())
}

#[derive(Debug)]
struct ShaderBuilder {
    lines: Vec<String>,
    next_value: usize,
    source_name: String,
}

impl ShaderBuilder {
    fn expression(&mut self, expression: &Expr) -> Result<String, LanguageError> {
        match expression {
            Expr::Zero { .. } => Ok("0i".into()),
            Expr::One { .. } => Ok("1i".into()),
            Expr::Scalar { real, imag, .. } => self.integer_constant(real, imag),
            Expr::Reference { name, .. } if name == "input" => Ok("input".into()),
            Expr::Add { operands, .. } => self.fold(operands, "checked_add"),
            Expr::Multiply { operands, .. } => self.fold(operands, "checked_multiply"),
            Expr::Orient { turns, value, .. } if turns.rem_euclid(4) == 0 => self.expression(value),
            Expr::Orient { turns, value, .. } if turns.rem_euclid(4) == 2 => {
                let value = self.expression(value)?;
                Ok(self.checked("checked_negate", &value, None))
            }
            Expr::Reference { name, span } => Err(gpu_semantic_error(
                format!("unresolved GPU reference {name:?}"),
                &self.source_name,
                *span,
            )),
            unsupported => Err(gpu_semantic_error(
                "GPU batch supports integer scalars, ADD, MULTIPLY, and even ORIENT turns only",
                &self.source_name,
                unsupported.span(),
            )),
        }
    }

    fn integer_constant(&self, real: &str, imag: &str) -> Result<String, LanguageError> {
        let scalar = NativeScalar::from_text(real, imag)
            .map_err(|message| gpu_error("NSG002", message, &self.source_name))?;
        if !scalar.imag.is_zero() || !scalar.real.denom().is_one() {
            return Err(gpu_error(
                "NSG002",
                "GPU function constants must be real integers",
                &self.source_name,
            ));
        }
        scalar
            .real
            .numer()
            .to_i32()
            .map(|value| {
                if value == i32::MIN {
                    "bitcast<i32>(2147483648u)".into()
                } else {
                    format!("{value}i")
                }
            })
            .ok_or_else(|| {
                gpu_error(
                    "NSG002",
                    "GPU function constant is outside signed 32-bit range",
                    &self.source_name,
                )
            })
    }

    fn fold(&mut self, operands: &[Expr], operation: &str) -> Result<String, LanguageError> {
        let mut values = operands.iter();
        let mut result =
            self.expression(values.next().expect("parser enforces operation arity"))?;
        for operand in values {
            let right = self.expression(operand)?;
            result = self.checked(operation, &result, Some(&right));
        }
        Ok(result)
    }

    fn checked(&mut self, operation: &str, left: &str, right: Option<&str>) -> String {
        let id = self.next_value;
        self.next_value += 1;
        let arguments = right.map_or_else(|| left.to_owned(), |right| format!("{left}, {right}"));
        self.lines
            .push(format!("    let checked_{id} = {operation}({arguments});"));
        self.lines
            .push(format!("    overflow |= checked_{id}.overflow;"));
        self.lines
            .push(format!("    let value_{id} = checked_{id}.value;"));
        format!("value_{id}")
    }
}

fn shader_source(expression: &Expr, source_name: &str) -> Result<String, LanguageError> {
    let mut builder = ShaderBuilder {
        lines: Vec::new(),
        next_value: 0,
        source_name: source_name.into(),
    };
    let result = builder.expression(expression)?;
    let body = builder.lines.join("\n");
    Ok(format!(
        r"
struct Parameters {{ steps: u32, count: u32, padding_a: u32, padding_b: u32 }}
struct Checked {{ value: i32, overflow: u32 }}

@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var<storage, read> inputs: array<i32>;
@group(0) @binding(2) var<storage, read_write> outputs: array<i32>;
@group(0) @binding(3) var<storage, read_write> errors: array<u32>;

fn checked_add(left: i32, right: i32) -> Checked {{
    let result = bitcast<i32>(bitcast<u32>(left) + bitcast<u32>(right));
    let overflow = u32((right > 0i && result < left) || (right < 0i && result > left));
    return Checked(result, overflow);
}}

fn magnitude(value: i32) -> u32 {{
    let bits = bitcast<u32>(value);
    return select(bits, 0u - bits, value < 0i);
}}

fn checked_multiply(left: i32, right: i32) -> Checked {{
    let left_magnitude = magnitude(left);
    let right_magnitude = magnitude(right);
    let negative = (left < 0i) != (right < 0i);
    let limit = select(2147483647u, 2147483648u, negative);
    let divisor = max(right_magnitude, 1u);
    let overflow = u32(right_magnitude != 0u && left_magnitude > limit / divisor);
    let product = left_magnitude * right_magnitude;
    let bits = select(product, 0u - product, negative);
    return Checked(bitcast<i32>(bits), overflow);
}}

fn checked_negate(value: i32) -> Checked {{
    return Checked(bitcast<i32>(0u - bitcast<u32>(value)), u32(value == bitcast<i32>(2147483648u)));
}}

fn step(input: i32) -> Checked {{
    var overflow = 0u;
{body}
    return Checked({result}, overflow);
}}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {{
    let position = invocation.x;
    if position >= parameters.count {{ return; }}
    var value = inputs[position];
    var overflow = 0u;
    var current_step = 0u;
    while current_step < parameters.steps && overflow == 0u {{
        let next = step(value);
        value = next.value;
        overflow |= next.overflow;
        current_step += 1u;
    }}
    outputs[position] = value;
    errors[position] = overflow;
}}
"
    ))
}

fn gpu_semantic_error(
    message: impl Into<String>,
    source_name: &str,
    span: Option<crate::core::Span>,
) -> LanguageError {
    LanguageError(Diagnostic {
        code: "NSG002".into(),
        message: message.into(),
        source_name: source_name.into(),
        span,
    })
}

fn gpu_error(code: &str, message: impl Into<String>, source_name: &str) -> LanguageError {
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
    fn shader_compiler_preserves_sequential_step_expression() {
        let program = parse(
            "let step = (value) => add(multiply(value, 2), 1)\noutput 0",
            "gpu.ns",
        )
        .unwrap();
        let expression = expanded_unary_expression(&program, "step").unwrap();
        let shader = shader_source(&expression, "gpu.ns").unwrap();

        assert!(shader.contains("checked_multiply(input, 2i)"));
        assert!(shader.contains("checked_add(value_0, 1i)"));
        assert!(shader.contains("while current_step < parameters.steps"));
    }

    #[test]
    fn shader_compiler_rejects_indexed_and_fractional_operations() {
        let indexed = parse("let step = (value) => index(1, value)\noutput 0", "gpu.ns").unwrap();
        let fraction = parse("let step = (value) => add(value, 1/2)\noutput 0", "gpu.ns").unwrap();

        assert_eq!(
            shader_source(
                &expanded_unary_expression(&indexed, "step").unwrap(),
                "gpu.ns"
            )
            .unwrap_err()
            .0
            .code,
            "NSG002"
        );
        assert_eq!(
            shader_source(
                &expanded_unary_expression(&fraction, "step").unwrap(),
                "gpu.ns"
            )
            .unwrap_err()
            .0
            .code,
            "NSG002"
        );
    }
}
