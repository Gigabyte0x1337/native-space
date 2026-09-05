// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Benchmarks retained SPH coefficients on Rust CPU and Vulkan compute.
//!
//! The CPU and GPU consume one identical, finite unordered-pair graph. The
//! benchmark reports graph construction, CPU evaluation, GPU kernel timestamp,
//! synchronized GPU dispatch, setup/upload cost, and numerical disagreement
//! separately. It does not claim that the Native Space compiler generated the
//! graph or shader.

use std::{
    error::Error,
    hint::black_box,
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

use bytemuck::{Pod, Zeroable};
use clap::Parser;
use mimalloc::MiMalloc;
use num_traits::ToPrimitive as _;
use wgpu::util::DeviceExt as _;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

const SPACING: f32 = 0.28;
const SMOOTHING_RADIUS: f32 = 0.52;
const REST_DENSITY: f32 = 1_000.0;
const PRESSURE_STIFFNESS: f32 = 1_400.0;
const VISCOSITY: f32 = 0.14;
const GRAVITY: f32 = -9.81;
const WORKGROUP_SIZE: u32 = 128;
const WARMUP_ROUNDS: usize = 6;
const TIMESTAMP_QUERY_BYTES: u64 = 16;
// SPACING / SMOOTHING_RADIUS is exactly 0.28 / 0.52 = 7 / 13. Keeping the
// grid mapping integral avoids a second, rounding-sensitive representation of
// the same lattice geometry.
const GRID_SPACING_UNITS: usize = 7;
const GRID_RADIUS_UNITS: usize = 13;

#[derive(Debug, Parser)]
#[command(about = "Compare Rust CPU and Vulkan retained-coefficient SPH kernels")]
struct Arguments {
    /// Lattice dimensions separated by commas, for example 19x15x14,48x32x32.
    #[arg(long, default_value = "19x15x14,48x32x32")]
    counts: String,

    /// Measured rounds per implementation and lattice.
    #[arg(long, default_value_t = 15)]
    rounds: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Parameters {
    particle_count: u32,
    rest_density: f32,
    pressure_stiffness: f32,
    self_density: f32,
    gravity: f32,
    padding_a: u32,
    padding_b: u32,
    padding_c: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuPair {
    /// xyz is left minus right; w is the shared density gain.
    geometry: [f32; 4],
    /// x is pressure gain and y is viscosity gain.
    coefficients: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuAdjacency {
    pair_index: u32,
    other_particle: u32,
    direction: i32,
    padding: u32,
}

#[derive(Clone, Debug)]
struct PairGraph {
    pairs: Vec<GpuPair>,
    offsets: Vec<u32>,
    adjacency: Vec<GpuAdjacency>,
    candidate_checks: u64,
}

#[derive(Clone, Debug)]
struct ParticleState {
    positions: Vec<[f32; 4]>,
    velocities: Vec<[f32; 4]>,
}

#[derive(Clone, Debug)]
struct CoefficientOutput {
    density_pressure: Vec<[f32; 2]>,
    accelerations: Vec<[f32; 4]>,
}

impl CoefficientOutput {
    fn new(particle_count: usize) -> Self {
        Self {
            density_pressure: vec![[0.0; 2]; particle_count],
            accelerations: vec![[0.0; 4]; particle_count],
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct GpuTiming {
    kernel: Duration,
    synchronized: Duration,
}

#[derive(Debug)]
struct GpuRun {
    timing: GpuTiming,
    output: Option<CoefficientOutput>,
}

#[derive(Debug)]
struct VulkanContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_name: String,
    timestamp_period: f32,
    bind_group_layout: wgpu::BindGroupLayout,
    density_pipeline: wgpu::ComputePipeline,
    force_pipeline: wgpu::ComputePipeline,
}

#[derive(Debug)]
struct VulkanCase<'a> {
    context: &'a VulkanContext,
    particle_count: u32,
    bind_group: wgpu::BindGroup,
    density_pressure: wgpu::Buffer,
    accelerations: wgpu::Buffer,
    query_set: wgpu::QuerySet,
    query_resolve: wgpu::Buffer,
    query_readback: wgpu::Buffer,
    density_readback: wgpu::Buffer,
    acceleration_readback: wgpu::Buffer,
}

impl VulkanContext {
    async fn new() -> AppResult<Self> {
        let mut instance_descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        instance_descriptor.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(instance_descriptor);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|error| app_error(format!("no Vulkan adapter is available: {error}")))?;
        let adapter_info = adapter.get_info();
        if adapter_info.backend != wgpu::Backend::Vulkan {
            return Err(app_error(format!(
                "requested Vulkan but selected {:?}",
                adapter_info.backend
            )));
        }
        if !adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return Err(app_error(format!(
                "Vulkan adapter {:?} does not expose timestamp queries",
                adapter_info.name
            )));
        }
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Native water Vulkan device"),
                required_features: wgpu::Features::TIMESTAMP_QUERY,
                required_limits: adapter.limits(),
                ..Default::default()
            })
            .await
            .map_err(|error| app_error(format!("could not create Vulkan device: {error}")))?;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Native water coefficient bindings"),
            entries: &[
                buffer_layout(0, wgpu::BufferBindingType::Uniform),
                buffer_layout(1, storage_binding(true)),
                buffer_layout(2, storage_binding(true)),
                buffer_layout(3, storage_binding(true)),
                buffer_layout(4, storage_binding(true)),
                buffer_layout(5, storage_binding(false)),
                buffer_layout(6, storage_binding(false)),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Native water coefficient pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Native water retained-coefficient shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("native_water_native.wgsl").into()),
        });
        let density_pipeline = compute_pipeline(
            &device,
            &pipeline_layout,
            &shader,
            "Native water density pipeline",
            "density_main",
        );
        let force_pipeline = compute_pipeline(
            &device,
            &pipeline_layout,
            &shader,
            "Native water force pipeline",
            "force_main",
        );

        Ok(Self {
            device,
            timestamp_period: queue.get_timestamp_period(),
            queue,
            adapter_name: adapter_info.name,
            bind_group_layout,
            density_pipeline,
            force_pipeline,
        })
    }
}

impl<'a> VulkanCase<'a> {
    fn new(
        context: &'a VulkanContext,
        state: &ParticleState,
        graph: &PairGraph,
        parameters: Parameters,
    ) -> AppResult<Self> {
        let particle_count = u32::try_from(state.positions.len())?;
        let parameter_buffer = initialized_buffer(
            &context.device,
            "Native water parameters",
            bytemuck::bytes_of(&parameters),
            wgpu::BufferUsages::UNIFORM,
        );
        let velocity_buffer = initialized_buffer(
            &context.device,
            "Native water velocities",
            bytemuck::cast_slice(&state.velocities),
            wgpu::BufferUsages::STORAGE,
        );
        let pair_buffer = initialized_buffer(
            &context.device,
            "Native water coefficient pairs",
            bytemuck::cast_slice(&graph.pairs),
            wgpu::BufferUsages::STORAGE,
        );
        let adjacency_buffer = initialized_buffer(
            &context.device,
            "Native water adjacency",
            bytemuck::cast_slice(&graph.adjacency),
            wgpu::BufferUsages::STORAGE,
        );
        let offset_buffer = initialized_buffer(
            &context.device,
            "Native water adjacency offsets",
            bytemuck::cast_slice(&graph.offsets),
            wgpu::BufferUsages::STORAGE,
        );
        let density_size = byte_size::<[f32; 2]>(state.positions.len())?;
        let acceleration_size = byte_size::<[f32; 4]>(state.positions.len())?;
        let density_pressure = output_buffer(
            &context.device,
            "Native water density and pressure",
            density_size,
        );
        let accelerations = output_buffer(
            &context.device,
            "Native water accelerations",
            acceleration_size,
        );
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Native water coefficient bind group"),
                layout: &context.bind_group_layout,
                entries: &[
                    binding(0, &parameter_buffer),
                    binding(1, &velocity_buffer),
                    binding(2, &pair_buffer),
                    binding(3, &adjacency_buffer),
                    binding(4, &offset_buffer),
                    binding(5, &density_pressure),
                    binding(6, &accelerations),
                ],
            });
        let query_set = context.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Native water kernel timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        let query_resolve = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Native water timestamp resolve"),
            size: TIMESTAMP_QUERY_BYTES,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let query_readback = readback_buffer(
            &context.device,
            "Native water timestamp readback",
            TIMESTAMP_QUERY_BYTES,
        );
        let density_readback = readback_buffer(
            &context.device,
            "Native water density readback",
            density_size,
        );
        let acceleration_readback = readback_buffer(
            &context.device,
            "Native water acceleration readback",
            acceleration_size,
        );

        Ok(Self {
            context,
            particle_count,
            bind_group,
            density_pressure,
            accelerations,
            query_set,
            query_resolve,
            query_readback,
            density_readback,
            acceleration_readback,
        })
    }

    fn run(&self, read_output: bool) -> AppResult<GpuRun> {
        let synchronized_start = Instant::now();
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Native water coefficient commands"),
                });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Native water density pass"),
            timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                query_set: &self.query_set,
                beginning_of_pass_write_index: Some(0),
                end_of_pass_write_index: None,
            }),
        });
        pass.set_pipeline(&self.context.density_pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.particle_count.div_ceil(WORKGROUP_SIZE), 1, 1);
        drop(pass);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Native water force pass"),
            timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                query_set: &self.query_set,
                beginning_of_pass_write_index: None,
                end_of_pass_write_index: Some(1),
            }),
        });
        pass.set_pipeline(&self.context.force_pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.particle_count.div_ceil(WORKGROUP_SIZE), 1, 1);
        drop(pass);
        encoder.resolve_query_set(&self.query_set, 0..2, &self.query_resolve, 0);
        encoder.copy_buffer_to_buffer(
            &self.query_resolve,
            0,
            &self.query_readback,
            0,
            TIMESTAMP_QUERY_BYTES,
        );
        if read_output {
            encoder.copy_buffer_to_buffer(
                &self.density_pressure,
                0,
                &self.density_readback,
                0,
                self.density_readback.size(),
            );
            encoder.copy_buffer_to_buffer(
                &self.accelerations,
                0,
                &self.acceleration_readback,
                0,
                self.acceleration_readback.size(),
            );
        }
        self.context.queue.submit([encoder.finish()]);

        let mut buffers = vec![&self.query_readback];
        if read_output {
            buffers.push(&self.density_readback);
            buffers.push(&self.acceleration_readback);
        }
        let bytes = read_buffers(&self.context.device, &buffers)?;
        let synchronized = synchronized_start.elapsed();
        let timestamps = bytes_to_u64(&bytes[0])?;
        let ticks = timestamps[1].saturating_sub(timestamps[0]);
        let ticks = ticks
            .to_f64()
            .ok_or_else(|| app_error("Vulkan timestamp does not fit f64"))?;
        let kernel = Duration::from_secs_f64(
            ticks * f64::from(self.context.timestamp_period) / 1_000_000_000.0,
        );
        let output = read_output
            .then(|| -> AppResult<CoefficientOutput> {
                Ok(CoefficientOutput {
                    density_pressure: bytes_to_arrays::<2>(&bytes[1])?,
                    accelerations: bytes_to_arrays::<4>(&bytes[2])?,
                })
            })
            .transpose()?;
        Ok(GpuRun {
            timing: GpuTiming {
                kernel,
                synchronized,
            },
            output,
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> AppResult<()> {
    let arguments = Arguments::parse();
    if arguments.rounds == 0 {
        return Err(app_error("rounds must be greater than zero"));
    }
    let cases = parse_cases(&arguments.counts)?;
    let context_start = Instant::now();
    let context = VulkanContext::new().await?;
    let context_time = context_start.elapsed();
    println!("Vulkan adapter: {}", context.adapter_name);
    println!(
        "Vulkan context and pipelines: {:.3} ms",
        milliseconds(context_time)
    );
    println!("Measured rounds per case: {}", arguments.rounds);

    for counts in cases {
        run_case(&context, counts, arguments.rounds)?;
    }
    Ok(())
}

fn run_case(context: &VulkanContext, counts: [usize; 3], rounds: usize) -> AppResult<()> {
    let state = particle_state(counts)?;
    let graph_start = Instant::now();
    let (graph, parameters) = build_pair_graph(&state, counts)?;
    let graph_time = graph_start.elapsed();
    let mut cpu_output = CoefficientOutput::new(state.positions.len());

    for _ in 0..WARMUP_ROUNDS {
        evaluate_cpu(&graph, &state.velocities, parameters, &mut cpu_output);
    }
    let mut cpu_times = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let start = Instant::now();
        evaluate_cpu(&graph, &state.velocities, parameters, &mut cpu_output);
        cpu_times.push(start.elapsed());
        black_box(cpu_output.accelerations.last());
    }

    let gpu_setup_start = Instant::now();
    let gpu_case = VulkanCase::new(context, &state, &graph, parameters)?;
    let gpu_setup_time = gpu_setup_start.elapsed();
    for _ in 0..WARMUP_ROUNDS {
        black_box(gpu_case.run(false)?);
    }
    let verification = gpu_case.run(true)?;
    let gpu_output = verification
        .output
        .ok_or_else(|| app_error("verification dispatch returned no output"))?;
    let error = maximum_error(&cpu_output, &gpu_output);
    validate_disagreement(error)?;

    let mut kernel_times = Vec::with_capacity(rounds);
    let mut synchronized_times = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let timing = gpu_case.run(false)?.timing;
        kernel_times.push(timing.kernel);
        synchronized_times.push(timing.synchronized);
    }

    let cpu = median(&mut cpu_times);
    let kernel = median(&mut kernel_times);
    let synchronized = median(&mut synchronized_times);
    println!();
    println!(
        "{}x{}x{} = {} particles",
        counts[0],
        counts[1],
        counts[2],
        state.positions.len()
    );
    println!(
        "retained pairs: {} | directed adjacency: {} | candidate checks: {}",
        graph.pairs.len(),
        graph.adjacency.len(),
        graph.candidate_checks
    );
    println!(
        "Rust graph construction: {:.3} ms",
        milliseconds(graph_time)
    );
    println!("Rust CPU coefficient pass: {:.3} ms", milliseconds(cpu));
    println!(
        "Vulkan setup and upload: {:.3} ms",
        milliseconds(gpu_setup_time)
    );
    println!("Vulkan kernel timestamp: {:.3} ms", milliseconds(kernel));
    println!(
        "Vulkan synchronized dispatch: {:.3} ms",
        milliseconds(synchronized)
    );
    println!("CPU / Vulkan kernel: {:.2}x", ratio(cpu, kernel));
    println!(
        "CPU / synchronized Vulkan: {:.2}x",
        ratio(cpu, synchronized)
    );
    println!(
        "maximum disagreement: density {:.3e}, pressure {:.3e}, acceleration {:.3e}",
        error[0], error[1], error[2]
    );
    Ok(())
}

fn parse_cases(source: &str) -> AppResult<Vec<[usize; 3]>> {
    source
        .split(',')
        .map(|case| {
            let values = case
                .split(['x', 'X'])
                .map(str::parse::<usize>)
                .collect::<Result<Vec<_>, _>>()?;
            if values.len() != 3 || values.contains(&0) {
                return Err(app_error(format!(
                    "particle count {case:?} must contain three positive dimensions"
                )));
            }
            Ok([values[0], values[1], values[2]])
        })
        .collect()
}

#[expect(
    clippy::cast_precision_loss,
    reason = "lattice dimensions are deliberately bounded well within exact f32 integers"
)]
fn particle_state(counts: [usize; 3]) -> AppResult<ParticleState> {
    let particle_count = counts
        .iter()
        .try_fold(1_usize, |total, count| total.checked_mul(*count))
        .ok_or_else(|| app_error("particle count overflow"))?;
    let mut positions = Vec::with_capacity(particle_count);
    let mut velocities = Vec::with_capacity(particle_count);
    for y in 0..counts[1] {
        for z in 0..counts[2] {
            for x in 0..counts[0] {
                positions.push([
                    x as f32 * SPACING,
                    y as f32 * SPACING,
                    z as f32 * SPACING,
                    0.0,
                ]);
                let phase = (x + y * counts[0] + z * counts[0] * counts[1]) as f32;
                velocities.push([
                    (phase * 0.017).sin() * 0.35,
                    (phase * 0.013).cos() * 0.2,
                    (phase * 0.011).sin() * 0.28,
                    0.0,
                ]);
            }
        }
    }
    Ok(ParticleState {
        positions,
        velocities,
    })
}

fn build_pair_graph(
    state: &ParticleState,
    counts: [usize; 3],
) -> AppResult<(PairGraph, Parameters)> {
    let grid_counts = [
        grid_extent(counts[0]),
        grid_extent(counts[1]),
        grid_extent(counts[2]),
    ];
    let buckets = build_buckets(state.positions.len(), counts, grid_counts)?;

    let coefficient = kernel_coefficients();
    let pair_capacity = state
        .positions
        .len()
        .checked_mul(24)
        .ok_or_else(|| app_error("retained-pair capacity overflow"))?;
    let mut pair_records = Vec::with_capacity(pair_capacity);
    let mut candidate_checks = 0_u64;
    let forward_offsets = forward_cell_offsets();
    for z in 0..grid_counts[2] {
        for y in 0..grid_counts[1] {
            for x in 0..grid_counts[0] {
                let cell = [x, y, z];
                let source = &buckets[grid_index(cell, grid_counts)];
                for left_position in 0..source.len() {
                    for right_position in (left_position + 1)..source.len() {
                        candidate_checks += 1;
                        append_pair(
                            source[left_position],
                            source[right_position],
                            state,
                            coefficient,
                            &mut pair_records,
                        );
                    }
                }
                for offset in &forward_offsets {
                    let Some(neighbor_cell) = offset_cell(cell, *offset, grid_counts) else {
                        continue;
                    };
                    let neighbor = &buckets[grid_index(neighbor_cell, grid_counts)];
                    for &left in source {
                        for &right in neighbor {
                            candidate_checks += 1;
                            append_pair(left, right, state, coefficient, &mut pair_records);
                        }
                    }
                }
            }
        }
    }

    let mut per_particle = vec![Vec::<GpuAdjacency>::new(); state.positions.len()];
    let mut pairs = Vec::with_capacity(pair_records.len());
    for (pair_index, (left, right, pair)) in pair_records.into_iter().enumerate() {
        let pair_index = u32::try_from(pair_index)?;
        pairs.push(pair);
        per_particle[native_index(left)].push(GpuAdjacency {
            pair_index,
            other_particle: right,
            direction: 1,
            padding: 0,
        });
        per_particle[native_index(right)].push(GpuAdjacency {
            pair_index,
            other_particle: left,
            direction: -1,
            padding: 0,
        });
    }
    let adjacency_len = per_particle.iter().map(Vec::len).sum();
    let mut offsets = Vec::with_capacity(state.positions.len() + 1);
    let mut adjacency = Vec::with_capacity(adjacency_len);
    offsets.push(0);
    for edges in per_particle {
        adjacency.extend(edges);
        offsets.push(u32::try_from(adjacency.len())?);
    }
    let particle_count = u32::try_from(state.positions.len())?;
    let parameters = Parameters {
        particle_count,
        rest_density: REST_DENSITY,
        pressure_stiffness: PRESSURE_STIFFNESS,
        self_density: coefficient.self_density,
        gravity: GRAVITY,
        padding_a: 0,
        padding_b: 0,
        padding_c: 0,
    };
    Ok((
        PairGraph {
            pairs,
            offsets,
            adjacency,
            candidate_checks,
        },
        parameters,
    ))
}

fn build_buckets(
    particle_count: usize,
    counts: [usize; 3],
    grid_counts: [usize; 3],
) -> AppResult<Vec<Vec<u32>>> {
    let grid_len = grid_counts
        .iter()
        .try_fold(1_usize, |total, count| total.checked_mul(*count))
        .ok_or_else(|| app_error("grid size overflow"))?;
    let mut buckets = vec![Vec::<u32>::new(); grid_len];
    for particle in 0..particle_count {
        let x = particle % counts[0];
        let remainder = particle / counts[0];
        let z = remainder % counts[2];
        let y = remainder / counts[2];
        let cell = [
            x * GRID_SPACING_UNITS / GRID_RADIUS_UNITS,
            y * GRID_SPACING_UNITS / GRID_RADIUS_UNITS,
            z * GRID_SPACING_UNITS / GRID_RADIUS_UNITS,
        ];
        buckets[grid_index(cell, grid_counts)].push(u32::try_from(particle)?);
    }
    Ok(buckets)
}

#[derive(Clone, Copy, Debug)]
struct KernelCoefficients {
    density_scale: f32,
    pressure_scale: f32,
    viscosity_scale: f32,
    self_density: f32,
}

fn kernel_coefficients() -> KernelCoefficients {
    let mass = REST_DENSITY * SPACING.powi(3);
    let h = SMOOTHING_RADIUS;
    let poly6 = 315.0 / (64.0 * std::f32::consts::PI * h.powi(9));
    let spiky_gradient = -45.0 / (std::f32::consts::PI * h.powi(6));
    let viscosity_laplacian = 45.0 / (std::f32::consts::PI * h.powi(6));
    let density_scale = mass * poly6;
    KernelCoefficients {
        density_scale,
        pressure_scale: -mass * spiky_gradient,
        viscosity_scale: VISCOSITY * mass * viscosity_laplacian,
        self_density: density_scale * (h * h).powi(3),
    }
}

fn append_pair(
    left: u32,
    right: u32,
    state: &ParticleState,
    coefficient: KernelCoefficients,
    pairs: &mut Vec<(u32, u32, GpuPair)>,
) {
    let left_position = state.positions[native_index(left)];
    let right_position = state.positions[native_index(right)];
    let displacement = [
        left_position[0] - right_position[0],
        left_position[1] - right_position[1],
        left_position[2] - right_position[2],
    ];
    let distance_squared = displacement.iter().map(|value| value * value).sum::<f32>();
    if distance_squared >= SMOOTHING_RADIUS * SMOOTHING_RADIUS {
        return;
    }
    let density_remainder = SMOOTHING_RADIUS * SMOOTHING_RADIUS - distance_squared;
    let distance = distance_squared.sqrt();
    let (pressure, viscosity) = if distance == 0.0 {
        (0.0, 0.0)
    } else {
        let force_remainder = SMOOTHING_RADIUS - distance;
        (
            coefficient.pressure_scale * force_remainder * force_remainder / distance,
            coefficient.viscosity_scale * force_remainder,
        )
    };
    pairs.push((
        left,
        right,
        GpuPair {
            geometry: [
                displacement[0],
                displacement[1],
                displacement[2],
                coefficient.density_scale * density_remainder.powi(3),
            ],
            coefficients: [pressure, viscosity, 0.0, 0.0],
        },
    ));
}

fn evaluate_cpu(
    graph: &PairGraph,
    velocities: &[[f32; 4]],
    parameters: Parameters,
    output: &mut CoefficientOutput,
) {
    let particle_count = velocities.len();
    assert_eq!(graph.offsets.len(), particle_count + 1);
    assert_eq!(output.density_pressure.len(), particle_count);
    assert_eq!(output.accelerations.len(), particle_count);

    for particle in 0..particle_count {
        let mut density = parameters.self_density;
        for edge in adjacency(graph, particle) {
            density += graph.pairs[native_index(edge.pair_index)].geometry[3];
        }
        density = density.max(parameters.rest_density * 0.05);
        let pressure = parameters.pressure_stiffness * (density - parameters.rest_density).max(0.0);
        output.density_pressure[particle] = [density, pressure];
    }

    for particle in 0..particle_count {
        let own = output.density_pressure[particle];
        let own_velocity = velocities[particle];
        let mut acceleration = [0.0, parameters.gravity, 0.0, 0.0];
        for edge in adjacency(graph, particle) {
            let pair = graph.pairs[native_index(edge.pair_index)];
            let other = native_index(edge.other_particle);
            let other_density_pressure = output.density_pressure[other];
            let direction = match edge.direction {
                -1 => -1.0,
                1 => 1.0,
                value => panic!("invalid retained-pair direction {value}"),
            };
            let pressure_scale = pair.coefficients[0]
                * (own[1] / (own[0] * own[0])
                    + other_density_pressure[1]
                        / (other_density_pressure[0] * other_density_pressure[0]));
            let viscosity_scale = pair.coefficients[1] / other_density_pressure[0];
            for (axis, value) in acceleration[..3].iter_mut().enumerate() {
                *value += pressure_scale * pair.geometry[axis] * direction;
                *value += viscosity_scale * (velocities[other][axis] - own_velocity[axis]);
            }
        }
        output.accelerations[particle] = acceleration;
    }
}

fn adjacency(graph: &PairGraph, particle: usize) -> &[GpuAdjacency] {
    let start = native_index(graph.offsets[particle]);
    let end = native_index(graph.offsets[particle + 1]);
    &graph.adjacency[start..end]
}

fn maximum_error(left: &CoefficientOutput, right: &CoefficientOutput) -> [f32; 3] {
    let mut error = [0.0_f32; 3];
    for (left_value, right_value) in left.density_pressure.iter().zip(&right.density_pressure) {
        error[0] = error[0].max((left_value[0] - right_value[0]).abs());
        error[1] = error[1].max((left_value[1] - right_value[1]).abs());
    }
    for (left_value, right_value) in left.accelerations.iter().zip(&right.accelerations) {
        for (left_axis, right_axis) in left_value[..3].iter().zip(&right_value[..3]) {
            error[2] = error[2].max((left_axis - right_axis).abs());
        }
    }
    error
}

fn validate_disagreement(error: [f32; 3]) -> AppResult<()> {
    const MAX_DENSITY_ERROR: f32 = 1.0e-3;
    const MAX_PRESSURE_ERROR: f32 = 1.0e-1;
    const MAX_ACCELERATION_ERROR: f32 = 1.0e-4;
    if error[0] <= MAX_DENSITY_ERROR
        && error[1] <= MAX_PRESSURE_ERROR
        && error[2] <= MAX_ACCELERATION_ERROR
    {
        return Ok(());
    }
    Err(app_error(format!(
        "Vulkan output diverged from Rust CPU: density {:.3e}, pressure {:.3e}, acceleration {:.3e}",
        error[0], error[1], error[2]
    )))
}

fn grid_extent(count: usize) -> usize {
    (count - 1) * GRID_SPACING_UNITS / GRID_RADIUS_UNITS + 1
}

fn native_index(value: u32) -> usize {
    usize::try_from(value).expect("u32 indices fit every supported Rust target")
}

fn grid_index(cell: [usize; 3], grid_counts: [usize; 3]) -> usize {
    (cell[2] * grid_counts[1] + cell[1]) * grid_counts[0] + cell[0]
}

fn forward_cell_offsets() -> Vec<[i32; 3]> {
    let mut offsets = Vec::with_capacity(13);
    for z in -1_i32..=1 {
        for y in -1_i32..=1 {
            for x in -1_i32..=1 {
                if z > 0 || (z == 0 && y > 0) || (z == 0 && y == 0 && x > 0) {
                    offsets.push([x, y, z]);
                }
            }
        }
    }
    offsets
}

fn offset_cell(cell: [usize; 3], offset: [i32; 3], grid_counts: [usize; 3]) -> Option<[usize; 3]> {
    let mut result = [0; 3];
    for axis in 0..3 {
        let value = i64::try_from(cell[axis]).ok()? + i64::from(offset[axis]);
        if value < 0 || value >= i64::try_from(grid_counts[axis]).ok()? {
            return None;
        }
        result[axis] = usize::try_from(value).ok()?;
    }
    Some(result)
}

fn buffer_layout(binding: u32, ty: wgpu::BufferBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

const fn storage_binding(read_only: bool) -> wgpu::BufferBindingType {
    wgpu::BufferBindingType::Storage { read_only }
}

fn compute_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    label: &str,
    entry_point: &str,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        module: shader,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn initialized_buffer(
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

fn output_buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
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

fn binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn read_buffers(device: &wgpu::Device, buffers: &[&wgpu::Buffer]) -> AppResult<Vec<Vec<u8>>> {
    let mut receivers = Vec::with_capacity(buffers.len());
    for buffer in buffers {
        let (sender, receiver) = mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });
        receivers.push(receiver);
    }
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| app_error(format!("Vulkan wait failed: {error}")))?;

    let mut values = Vec::with_capacity(buffers.len());
    for (buffer, receiver) in buffers.iter().zip(receivers) {
        receiver
            .recv()
            .map_err(|error| app_error(format!("Vulkan map callback stopped: {error}")))?
            .map_err(|error| app_error(format!("Vulkan readback failed: {error}")))?;
        let slice = buffer.slice(..);
        let view = slice
            .get_mapped_range()
            .map_err(|error| app_error(format!("Vulkan mapped range failed: {error}")))?;
        values.push(view.to_vec());
        drop(view);
        buffer.unmap();
    }
    Ok(values)
}

fn bytes_to_u64(bytes: &[u8]) -> AppResult<Vec<u64>> {
    if !bytes.len().is_multiple_of(size_of::<u64>()) {
        return Err(app_error("timestamp readback has invalid length"));
    }
    Ok(bytes
        .chunks_exact(size_of::<u64>())
        .map(|chunk| {
            let array: [u8; 8] = chunk
                .try_into()
                .expect("chunks_exact guarantees the timestamp width");
            u64::from_ne_bytes(array)
        })
        .collect())
}

fn bytes_to_arrays<const WIDTH: usize>(bytes: &[u8]) -> AppResult<Vec<[f32; WIDTH]>> {
    let stride = WIDTH
        .checked_mul(size_of::<f32>())
        .ok_or_else(|| app_error("GPU output stride overflow"))?;
    if !bytes.len().is_multiple_of(stride) {
        return Err(app_error("GPU output has invalid length"));
    }
    bytes
        .chunks_exact(stride)
        .map(|chunk| {
            let mut values = [0.0; WIDTH];
            for (position, value) in values.iter_mut().enumerate() {
                let start = position * size_of::<f32>();
                let array: [u8; 4] = chunk[start..start + size_of::<f32>()]
                    .try_into()
                    .expect("chunk bounds guarantee one f32");
                *value = f32::from_ne_bytes(array);
            }
            Ok(values)
        })
        .collect()
}

fn byte_size<T>(count: usize) -> AppResult<u64> {
    let bytes = count
        .checked_mul(size_of::<T>())
        .ok_or_else(|| app_error("GPU buffer size overflow"))?;
    Ok(u64::try_from(bytes)?)
}

fn median(values: &mut [Duration]) -> Duration {
    values.sort_unstable();
    values[values.len() / 2]
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn ratio(numerator: Duration, denominator: Duration) -> f64 {
    numerator.as_secs_f64() / denominator.as_secs_f64()
}

fn app_error(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    Box::new(io::Error::other(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_pairs_expand_to_two_directed_edges() {
        let counts = [5, 4, 3];
        let state = particle_state(counts).unwrap();
        let (graph, _) = build_pair_graph(&state, counts).unwrap();

        assert_eq!(graph.adjacency.len(), graph.pairs.len() * 2);
        assert_eq!(graph.offsets.len(), state.positions.len() + 1);
    }

    #[test]
    fn cpu_coefficients_are_finite() {
        let counts = [5, 4, 3];
        let state = particle_state(counts).unwrap();
        let (graph, parameters) = build_pair_graph(&state, counts).unwrap();
        let mut output = CoefficientOutput::new(state.positions.len());

        evaluate_cpu(&graph, &state.velocities, parameters, &mut output);

        assert!(
            output
                .density_pressure
                .iter()
                .flatten()
                .chain(output.accelerations.iter().flatten())
                .all(|value| value.is_finite())
        );
    }
}
