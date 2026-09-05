// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Runs a native Vulkan water visualization.
//!
//! The default mode evaluates the complete indexed WCSPH state transition on
//! the CPU and uploads only the resulting state for Vulkan rendering. Indexed
//! and all-pairs Vulkan modes remain available as comparison baselines.
//! Rendering is a separate camera: the point view draws state directly, while
//! the fluid view derives screen-space depth and thickness without feeding
//! either value back into the simulation.

mod cpu_fluid;

use std::{
    error::Error,
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use bytemuck::{Pod, Zeroable};
use clap::{Parser, Subcommand};
use mimalloc::MiMalloc;
use num_traits::ToPrimitive as _;
use wgpu::util::DeviceExt as _;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

const MAX_PARTICLE_COUNT: u32 = 1_048_576;
// Quadratic traversal is retained only as a finite comparison baseline.
const MAX_ALL_PAIRS_PARTICLE_COUNT: u32 = 16_384;
const DEFAULT_PRESET: usize = 2;
const WORKGROUP_SIZE: u32 = 128;
const FIXED_STEP_SECONDS: f32 = 1.0 / 240.0;
// The 16K preset is the highest resolution observed stable at one solve per step.
const REFERENCE_STABLE_SMOOTHING_RADIUS: f32 = 0.16;
const MAX_STEPS_PER_FRAME: usize = 8;
const TITLE_UPDATE_INTERVAL: Duration = Duration::from_millis(500);
const INITIAL_WIDTH: f64 = 1_280.0;
const INITIAL_HEIGHT: f64 = 800.0;
const REST_DENSITY: f32 = 1_000.0;
const PRESSURE_STIFFNESS: f32 = 1_600.0;
const VISCOSITY: f32 = 0.16;
const GRAVITY: f32 = -9.81;
const BOX_BOUNDS: [f32; 3] = [3.0, 2.0, 1.0];
const BOUNDARY_DAMPING: f32 = 0.45;
const CAMERA_DRAG_RADIANS_PER_PIXEL: f32 = 0.005;
const MIN_CAMERA_PITCH: f32 = -1.45;
const MAX_CAMERA_PITCH: f32 = 1.45;
const MIN_CAMERA_ZOOM: f32 = 0.45;
const MAX_CAMERA_ZOOM: f32 = 2.8;
// The fitted default leaves the shader's camera margin visible on every side.
const FIT_CAMERA_YAW: f32 = -0.35;
const FIT_CAMERA_PITCH: f32 = 0.18;
const FIT_CAMERA_ZOOM: f32 = 0.92;
const DEFAULT_BENCHMARK_STEPS: u32 = 240;
const DEFAULT_BENCHMARK_WARMUP_STEPS: u32 = 16;

const PARTICLE_PRESETS: [ParticlePreset; 12] = [
    ParticlePreset::new(512, [16, 8, 4], 0.19, 0.35),
    ParticlePreset::new(1_024, [16, 16, 4], 0.16, 0.30),
    ParticlePreset::new(2_048, [32, 16, 4], 0.14, 0.26),
    ParticlePreset::new(4_096, [32, 16, 8], 0.12, 0.225),
    ParticlePreset::new(8_192, [32, 32, 8], 0.10, 0.188),
    ParticlePreset::new(16_384, [64, 32, 8], 0.085, 0.16),
    ParticlePreset::new(32_768, [64, 32, 16], 0.075, 0.141),
    ParticlePreset::new(65_536, [64, 64, 16], 0.052, 0.098),
    ParticlePreset::new(131_072, [128, 64, 16], 0.041, 0.077),
    ParticlePreset::new(262_144, [128, 64, 32], 0.039, 0.073),
    ParticlePreset::new(524_288, [128, 128, 32], 0.026, 0.049),
    ParticlePreset::new(1_048_576, [128, 128, 64], 0.0255, 0.048),
];

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ParticleState {
    position: [f32; 4],
    velocity: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct SimParameters {
    particle_count: u32,
    delta_time: f32,
    smoothing_radius: f32,
    rest_density: f32,
    pressure_stiffness: f32,
    viscosity: f32,
    mass: f32,
    gravity: f32,
    bounds: [f32; 4],
    /// x is aspect ratio and y is particle radius.
    projection: [f32; 4],
    /// x is yaw, y is pitch, and z is zoom.
    camera: [f32; 4],
    /// xyz are cell counts and w is their checked product.
    grid: [u32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ParticlePreset {
    count: u32,
    dimensions: [usize; 3],
    spacing: f32,
    smoothing_radius: f32,
}

#[derive(Debug, Parser)]
#[command(about = "Run the Vulkan water view or its headless CPU benchmark")]
struct Arguments {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Benchmark only the indexed CPU state transition without Vulkan.
    BenchmarkCpu {
        /// Particle count; must match one of the application's presets.
        #[arg(long, default_value_t = PARTICLE_PRESETS[DEFAULT_PRESET].count)]
        particles: u32,
        /// Timed physical substeps after warm-up.
        #[arg(long, default_value_t = DEFAULT_BENCHMARK_STEPS, value_parser = clap::value_parser!(u32).range(1..))]
        steps: u32,
        /// Untimed physical substeps used to settle caches and state.
        #[arg(long, default_value_t = DEFAULT_BENCHMARK_WARMUP_STEPS)]
        warmup_steps: u32,
    },
}

impl ParticlePreset {
    const fn new(count: u32, dimensions: [usize; 3], spacing: f32, smoothing_radius: f32) -> Self {
        Self {
            count,
            dimensions,
            spacing,
            smoothing_radius,
        }
    }
}

#[derive(Debug)]
struct NativeWater {
    window: Arc<Window>,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    parameter_buffer: wgpu::Buffer,
    state_buffers: [wgpu::Buffer; 2],
    compute_bind_groups: [wgpu::BindGroup; 2],
    render_bind_groups: [wgpu::BindGroup; 2],
    clear_grid_pipeline: wgpu::ComputePipeline,
    index_points_pipeline: wgpu::ComputePipeline,
    indexed_density_pipeline: wgpu::ComputePipeline,
    indexed_integrate_pipeline: wgpu::ComputePipeline,
    all_pairs_density_pipeline: wgpu::ComputePipeline,
    all_pairs_integrate_pipeline: wgpu::ComputePipeline,
    guide_pipeline: wgpu::RenderPipeline,
    render_pipeline: wgpu::RenderPipeline,
    fluid_renderer: FluidRenderer,
    initial_state: Vec<ParticleState>,
    cpu_fluid: cpu_fluid::CpuFluid,
    preset_index: usize,
    current_state: usize,
    previous_frame: Instant,
    accumulator: Duration,
    title_started: Instant,
    title_frames: u32,
    title_steps: u64,
    title_dropped_steps: u64,
    title_cpu_time: Duration,
    simulation_mode: SimulationMode,
    render_mode: RenderMode,
    paused: bool,
    camera_yaw: f32,
    camera_pitch: f32,
    camera_zoom: f32,
    dragging: bool,
    cursor_position: Option<PhysicalPosition<f64>>,
    adapter_name: String,
}

#[derive(Debug)]
struct VulkanDisplay {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    configuration: wgpu::SurfaceConfiguration,
    adapter_name: String,
}

#[derive(Debug)]
struct VisualResources {
    parameter_buffer: wgpu::Buffer,
    state_buffers: [wgpu::Buffer; 2],
    compute_bind_groups: [wgpu::BindGroup; 2],
    render_bind_groups: [wgpu::BindGroup; 2],
    clear_grid_pipeline: wgpu::ComputePipeline,
    index_points_pipeline: wgpu::ComputePipeline,
    indexed_density_pipeline: wgpu::ComputePipeline,
    indexed_integrate_pipeline: wgpu::ComputePipeline,
    all_pairs_density_pipeline: wgpu::ComputePipeline,
    all_pairs_integrate_pipeline: wgpu::ComputePipeline,
    guide_pipeline: wgpu::RenderPipeline,
    render_pipeline: wgpu::RenderPipeline,
    fluid_renderer: FluidRenderer,
    initial_state: Vec<ParticleState>,
}

#[derive(Debug)]
struct VisualPipelines {
    clear_grid: wgpu::ComputePipeline,
    index_points: wgpu::ComputePipeline,
    indexed_density: wgpu::ComputePipeline,
    indexed_integrate: wgpu::ComputePipeline,
    all_pairs_density: wgpu::ComputePipeline,
    all_pairs_integrate: wgpu::ComputePipeline,
    guide: wgpu::RenderPipeline,
    render: wgpu::RenderPipeline,
}

/// Screen-space fluid rendering is a derived view of the current particle
/// state. These resources never participate in the simulation update.
#[derive(Debug)]
struct FluidRenderer {
    depth_pipeline: wgpu::RenderPipeline,
    thickness_pipeline: wgpu::RenderPipeline,
    horizontal_blur_pipeline: wgpu::RenderPipeline,
    vertical_blur_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    texture_layout: wgpu::BindGroupLayout,
    targets: FluidTargets,
}

#[derive(Debug)]
struct FluidTargets {
    _raw_depth: wgpu::Texture,
    raw_depth_view: wgpu::TextureView,
    _thickness: wgpu::Texture,
    thickness_view: wgpu::TextureView,
    _blur_a: wgpu::Texture,
    blur_a_view: wgpu::TextureView,
    _blur_b: wgpu::Texture,
    blur_b_view: wgpu::TextureView,
    horizontal_bind_group: wgpu::BindGroup,
    vertical_bind_group: wgpu::BindGroup,
    composite_bind_group: wgpu::BindGroup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrameOutcome {
    Presented,
    Skipped,
    Reconfigure,
    Recreate,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum SimulationMode {
    #[default]
    CpuIndexed,
    GpuIndexed,
    AllPairs,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum RenderMode {
    Points,
    #[default]
    Surface,
}

impl RenderMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Points => "particle view",
            Self::Surface => "water surface",
        }
    }

    const fn toggled(self) -> Self {
        match self {
            Self::Points => Self::Surface,
            Self::Surface => Self::Points,
        }
    }
}

impl SimulationMode {
    const fn label(self) -> &'static str {
        match self {
            Self::CpuIndexed => "CPU indexed WCSPH",
            Self::GpuIndexed => "GPU indexed WCSPH",
            Self::AllPairs => "GPU all-pairs WCSPH",
        }
    }

    const fn toggled(self, particle_count: u32) -> Self {
        match self {
            Self::CpuIndexed => Self::GpuIndexed,
            Self::GpuIndexed if particle_count <= MAX_ALL_PAIRS_PARTICLE_COUNT => Self::AllPairs,
            Self::GpuIndexed | Self::AllPairs => Self::CpuIndexed,
        }
    }
}

impl NativeWater {
    async fn new(window: Arc<Window>) -> AppResult<Self> {
        let VulkanDisplay {
            instance,
            surface,
            device,
            queue,
            configuration: surface_configuration,
            adapter_name,
        } = create_vulkan_display(&window).await?;
        let VisualResources {
            parameter_buffer,
            state_buffers,
            compute_bind_groups,
            render_bind_groups,
            clear_grid_pipeline,
            index_points_pipeline,
            indexed_density_pipeline,
            indexed_integrate_pipeline,
            all_pairs_density_pipeline,
            all_pairs_integrate_pipeline,
            guide_pipeline,
            render_pipeline,
            fluid_renderer,
            initial_state,
        } = create_visual_resources(&device, &surface_configuration)?;
        let initial_cpu_parameters = parameters(
            surface_configuration.width,
            surface_configuration.height,
            PARTICLE_PRESETS[DEFAULT_PRESET],
            FIT_CAMERA_YAW,
            FIT_CAMERA_PITCH,
            FIT_CAMERA_ZOOM,
            solver_substeps(PARTICLE_PRESETS[DEFAULT_PRESET]),
        );
        let cpu_fluid = cpu_fluid::CpuFluid::new(&initial_state, &initial_cpu_parameters);

        Ok(Self {
            window,
            instance,
            surface,
            device,
            queue,
            surface_configuration,
            parameter_buffer,
            state_buffers,
            compute_bind_groups,
            render_bind_groups,
            clear_grid_pipeline,
            index_points_pipeline,
            indexed_density_pipeline,
            indexed_integrate_pipeline,
            all_pairs_density_pipeline,
            all_pairs_integrate_pipeline,
            guide_pipeline,
            render_pipeline,
            fluid_renderer,
            cpu_fluid,
            initial_state,
            preset_index: DEFAULT_PRESET,
            current_state: 0,
            previous_frame: Instant::now(),
            accumulator: Duration::ZERO,
            title_started: Instant::now(),
            title_frames: 0,
            title_steps: 0,
            title_dropped_steps: 0,
            title_cpu_time: Duration::ZERO,
            simulation_mode: SimulationMode::default(),
            render_mode: RenderMode::default(),
            paused: false,
            camera_yaw: FIT_CAMERA_YAW,
            camera_pitch: FIT_CAMERA_PITCH,
            camera_zoom: FIT_CAMERA_ZOOM,
            dragging: false,
            cursor_position: None,
            adapter_name,
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
        self.fluid_renderer.resize(&self.device, size);
    }

    fn reconfigure(&mut self) {
        self.surface
            .configure(&self.device, &self.surface_configuration);
    }

    fn recreate_surface(&mut self) -> AppResult<()> {
        let surface = self.instance.create_surface(Arc::clone(&self.window))?;
        surface.configure(&self.device, &self.surface_configuration);
        self.surface = surface;
        Ok(())
    }

    fn reset(&mut self) {
        let bytes = bytemuck::cast_slice(&self.initial_state);
        self.queue.write_buffer(&self.state_buffers[0], 0, bytes);
        self.queue.write_buffer(&self.state_buffers[1], 0, bytes);
        let parameters = self.current_parameters();
        self.cpu_fluid.reset(&self.initial_state, &parameters);
        self.current_state = 0;
        self.accumulator = Duration::ZERO;
        self.previous_frame = Instant::now();
    }

    fn select_more_particles(&mut self) -> AppResult<()> {
        let next = (self.preset_index + 1).min(PARTICLE_PRESETS.len() - 1);
        self.select_preset(next)
    }

    fn select_fewer_particles(&mut self) -> AppResult<()> {
        self.select_preset(self.preset_index.saturating_sub(1))
    }

    fn select_preset(&mut self, preset_index: usize) -> AppResult<()> {
        if preset_index == self.preset_index {
            return Ok(());
        }
        let initial_state = initial_particles(PARTICLE_PRESETS[preset_index])?;
        self.initial_state = initial_state;
        self.preset_index = preset_index;
        let particle_count = PARTICLE_PRESETS[preset_index].count;
        if self.simulation_mode == SimulationMode::AllPairs
            && particle_count > MAX_ALL_PAIRS_PARTICLE_COUNT
        {
            self.simulation_mode = SimulationMode::GpuIndexed;
        }
        self.reset();
        let now = Instant::now();
        self.title_started = now.checked_sub(TITLE_UPDATE_INTERVAL).unwrap_or(now);
        Ok(())
    }

    fn set_dragging(&mut self, dragging: bool) {
        self.dragging = dragging;
    }

    fn move_cursor(&mut self, position: PhysicalPosition<f64>) {
        if self.dragging
            && let Some(previous) = self.cursor_position
        {
            let delta_x = (position.x - previous.x)
                .to_f32()
                .expect("finite cursor delta has an f32 representation");
            let delta_y = (position.y - previous.y)
                .to_f32()
                .expect("finite cursor delta has an f32 representation");
            self.camera_yaw += delta_x * CAMERA_DRAG_RADIANS_PER_PIXEL;
            self.camera_pitch = (self.camera_pitch + delta_y * CAMERA_DRAG_RADIANS_PER_PIXEL)
                .clamp(MIN_CAMERA_PITCH, MAX_CAMERA_PITCH);
        }
        self.cursor_position = Some(position);
    }

    fn zoom(&mut self, lines: f32) {
        self.camera_zoom =
            (self.camera_zoom * (lines * 0.12).exp()).clamp(MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM);
    }

    fn fit_camera(&mut self) {
        self.camera_yaw = FIT_CAMERA_YAW;
        self.camera_pitch = FIT_CAMERA_PITCH;
        self.camera_zoom = FIT_CAMERA_ZOOM;
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.previous_frame = Instant::now();
    }

    fn toggle_simulation_mode(&mut self) {
        let particle_count = PARTICLE_PRESETS[self.preset_index].count;
        self.simulation_mode = self.simulation_mode.toggled(particle_count);
        // Every mode restarts from one state so behavior remains comparable.
        self.reset();
        let now = Instant::now();
        self.title_started = now.checked_sub(TITLE_UPDATE_INTERVAL).unwrap_or(now);
    }

    fn toggle_render_mode(&mut self) {
        self.render_mode = self.render_mode.toggled();
        let now = Instant::now();
        self.title_started = now.checked_sub(TITLE_UPDATE_INTERVAL).unwrap_or(now);
    }

    fn render(&mut self) -> AppResult<FrameOutcome> {
        let now = Instant::now();
        let frame_time = now
            .saturating_duration_since(self.previous_frame)
            .min(Duration::from_millis(50));
        self.previous_frame = now;
        if !self.paused {
            self.accumulator += frame_time;
        }
        let fixed_step = Duration::from_secs_f32(FIXED_STEP_SECONDS);
        let available_steps = self.accumulator.as_nanos() / fixed_step.as_nanos();
        let maximum_steps =
            u128::try_from(MAX_STEPS_PER_FRAME).expect("the small frame-step limit fits u128");
        let step_count = usize::try_from(available_steps.min(maximum_steps))
            .expect("the bounded frame-step count fits usize");
        let dropped_steps = available_steps.saturating_sub(maximum_steps);
        self.accumulator = self.accumulator.saturating_sub(fixed_step.saturating_mul(
            u32::try_from(available_steps).expect("the clamped frame duration fits u32 steps"),
        ));
        self.title_steps += u64::try_from(step_count).expect("frame steps fit u64");
        self.title_dropped_steps +=
            u64::try_from(dropped_steps).expect("dropped frame steps fit u64");

        let parameters = self.current_parameters();
        self.queue
            .write_buffer(&self.parameter_buffer, 0, bytemuck::bytes_of(&parameters));

        if self.simulation_mode == SimulationMode::CpuIndexed && step_count > 0 {
            let started = Instant::now();
            for _ in 0..step_count {
                for _ in 0..self.active_solver_substeps() {
                    self.cpu_fluid.step(&parameters);
                }
            }
            self.title_cpu_time += started.elapsed();
            self.current_state = 0;
            self.queue.write_buffer(
                &self.state_buffers[self.current_state],
                0,
                bytemuck::cast_slice(self.cpu_fluid.state()),
            );
        }

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(FrameOutcome::Skipped);
            }
            wgpu::CurrentSurfaceTexture::Outdated => return Ok(FrameOutcome::Reconfigure),
            wgpu::CurrentSurfaceTexture::Lost => return Ok(FrameOutcome::Recreate),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(app_error(
                    "Vulkan rejected the configured presentation surface",
                ));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Native water visual frame"),
            });
        if self.simulation_mode != SimulationMode::CpuIndexed {
            for _ in 0..step_count {
                self.encode_simulation_step(&mut encoder);
            }
        }
        self.encode_render(&mut encoder, &view);
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        self.update_title();
        Ok(FrameOutcome::Presented)
    }

    fn encode_simulation_step(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let substeps = self.active_solver_substeps();
        for _ in 0..substeps {
            self.encode_solver_substep(encoder);
        }
    }

    fn active_solver_substeps(&self) -> u32 {
        solver_substeps(PARTICLE_PRESETS[self.preset_index])
    }

    fn current_parameters(&self) -> SimParameters {
        parameters(
            self.surface_configuration.width,
            self.surface_configuration.height,
            PARTICLE_PRESETS[self.preset_index],
            self.camera_yaw,
            self.camera_pitch,
            self.camera_zoom,
            self.active_solver_substeps(),
        )
    }

    fn encode_solver_substep(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let particle_count = PARTICLE_PRESETS[self.preset_index].count;
        let bind_group = &self.compute_bind_groups[self.current_state];

        if self.simulation_mode == SimulationMode::GpuIndexed {
            let cell_count = grid_shape(PARTICLE_PRESETS[self.preset_index].smoothing_radius)[3];
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Native water visual clear cell membership pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.clear_grid_pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups(cell_count.div_ceil(WORKGROUP_SIZE), 1, 1);
            drop(pass);

            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Native water visual point indexing pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.index_points_pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups(particle_count.div_ceil(WORKGROUP_SIZE), 1, 1);
            drop(pass);
        }

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Native water visual density pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(match self.simulation_mode {
            SimulationMode::CpuIndexed => {
                unreachable!("CPU mode is handled before command encoding")
            }
            SimulationMode::GpuIndexed => &self.indexed_density_pipeline,
            SimulationMode::AllPairs => &self.all_pairs_density_pipeline,
        });
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(particle_count.div_ceil(WORKGROUP_SIZE), 1, 1);
        drop(pass);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Native water visual integration pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(match self.simulation_mode {
            SimulationMode::CpuIndexed => {
                unreachable!("CPU mode is handled before command encoding")
            }
            SimulationMode::GpuIndexed => &self.indexed_integrate_pipeline,
            SimulationMode::AllPairs => &self.all_pairs_integrate_pipeline,
        });
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(particle_count.div_ceil(WORKGROUP_SIZE), 1, 1);
        drop(pass);
        self.current_state = 1 - self.current_state;
    }

    fn encode_render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        if self.render_mode == RenderMode::Surface {
            self.fluid_renderer.encode(
                encoder,
                view,
                &self.render_bind_groups[self.current_state],
                PARTICLE_PRESETS[self.preset_index].count,
            );
            self.encode_guides(encoder, view);
            return;
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Native water visual particle pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.006,
                        g: 0.012,
                        b: 0.028,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_bind_group(1, &self.render_bind_groups[self.current_state], &[]);
        pass.set_pipeline(&self.guide_pipeline);
        pass.draw(0..30, 0..1);
        pass.set_pipeline(&self.render_pipeline);
        pass.draw(0..6, 0..PARTICLE_PRESETS[self.preset_index].count);
    }

    fn encode_guides(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Native water visual guide overlay"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_bind_group(1, &self.render_bind_groups[self.current_state], &[]);
        pass.set_pipeline(&self.guide_pipeline);
        pass.draw(0..30, 0..1);
    }

    fn update_title(&mut self) {
        self.title_frames += 1;
        let title_elapsed = self.title_started.elapsed();
        if title_elapsed < TITLE_UPDATE_INTERVAL {
            return;
        }
        let frames_per_second = f64::from(self.title_frames) / title_elapsed.as_secs_f64();
        let simulation_steps_per_second = self
            .title_steps
            .to_f64()
            .expect("u64 step count has a finite f64 representation")
            / title_elapsed.as_secs_f64();
        let state = if self.paused { "paused" } else { "running" };
        let particle_count = PARTICLE_PRESETS[self.preset_index].count;
        let substeps = self.active_solver_substeps();
        let simulation_timing =
            if self.simulation_mode == SimulationMode::CpuIndexed && self.title_steps > 0 {
                format!(
                    "CPU {:.3} ms/step",
                    self.title_cpu_time.as_secs_f64() * 1_000.0
                        / self
                            .title_steps
                            .to_f64()
                            .expect("u64 step count has a finite f64 representation")
                )
            } else {
                "GPU simulation".to_owned()
            };
        let mode_control = if particle_count > MAX_ALL_PAIRS_PARTICLE_COUNT {
            "I CPU/GPU indexed, ↑↓ load"
        } else {
            "I CPU/GPU indexed/all-pairs, ↑↓ load"
        };
        self.window.set_title(&format!(
            "Native Water — {:>7} particles — {} — {} — solver {}x — {:>3.0} FPS — sim {:>3.0}/240 steps/s — {} — dropped {} — {} — {}, W view, drag rotate, wheel zoom, F fit — {}",
            particle_count,
            self.simulation_mode.label(),
            self.render_mode.label(),
            substeps,
            frames_per_second,
            simulation_steps_per_second,
            simulation_timing,
            self.title_dropped_steps,
            state,
            mode_control,
            self.adapter_name
        ));
        self.title_started = Instant::now();
        self.title_frames = 0;
        self.title_steps = 0;
        self.title_dropped_steps = 0;
        self.title_cpu_time = Duration::ZERO;
    }
}

async fn create_vulkan_display(window: &Arc<Window>) -> AppResult<VulkanDisplay> {
    let mut instance_descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    instance_descriptor.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(instance_descriptor);
    let surface = instance.create_surface(Arc::clone(window))?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        })
        .await
        .map_err(|error| app_error(format!("no Vulkan display adapter is available: {error}")))?;
    let adapter_info = adapter.get_info();
    if adapter_info.backend != wgpu::Backend::Vulkan {
        return Err(app_error(format!(
            "requested Vulkan but selected {:?}",
            adapter_info.backend
        )));
    }
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("Native water visual device"),
            required_limits: adapter.limits(),
            ..Default::default()
        })
        .await?;
    let configuration = choose_surface_configuration(&surface, &adapter, window.inner_size())?;
    surface.configure(&device, &configuration);
    Ok(VulkanDisplay {
        instance,
        surface,
        device,
        queue,
        configuration,
        adapter_name: adapter_info.name,
    })
}

fn choose_surface_configuration(
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
    size: PhysicalSize<u32>,
) -> AppResult<wgpu::SurfaceConfiguration> {
    let capabilities = surface.get_capabilities(adapter);
    let format = capabilities
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
        .or_else(|| capabilities.formats.first().copied())
        .ok_or_else(|| app_error("Vulkan surface exposes no texture formats"))?;
    let present_mode = capabilities
        .present_modes
        .iter()
        .copied()
        .find(|mode| *mode == wgpu::PresentMode::Fifo)
        .or_else(|| capabilities.present_modes.first().copied())
        .ok_or_else(|| app_error("Vulkan surface exposes no present modes"))?;
    let alpha_mode = capabilities
        .alpha_modes
        .first()
        .copied()
        .ok_or_else(|| app_error("Vulkan surface exposes no alpha modes"))?;
    Ok(wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space: wgpu::SurfaceColorSpace::Auto,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    })
}

fn create_visual_resources(
    device: &wgpu::Device,
    surface_configuration: &wgpu::SurfaceConfiguration,
) -> AppResult<VisualResources> {
    let initial_state = initial_particles(PARTICLE_PRESETS[DEFAULT_PRESET])?;
    let capacity = usize::try_from(MAX_PARTICLE_COUNT)?;
    let mut state_capacity = vec![ParticleState::zeroed(); capacity];
    state_capacity[..initial_state.len()].copy_from_slice(&initial_state);
    let parameter_buffer = initial_parameter_buffer(device, surface_configuration);
    let state_buffers = [
        initialized_buffer(
            device,
            "Native water visual state A",
            bytemuck::cast_slice(&state_capacity),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        ),
        initialized_buffer(
            device,
            "Native water visual state B",
            bytemuck::cast_slice(&state_capacity),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        ),
    ];
    let density_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Native water visual density and pressure"),
        size: byte_size::<[f32; 2]>(capacity)?,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let maximum_cell_count = PARTICLE_PRESETS
        .iter()
        .map(|preset| grid_shape(preset.smoothing_radius)[3])
        .max()
        .expect("the preset table is not empty");
    let cell_heads_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Native water visual cell heads"),
        size: byte_size::<i32>(usize::try_from(maximum_cell_count)?)?,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let next_particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Native water visual cell links"),
        size: byte_size::<i32>(capacity)?,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let compute_layout = compute_bind_group_layout(device);
    let render_layout = render_bind_group_layout(device);
    let compute_bind_groups = bidirectional_compute_bind_groups(
        device,
        &compute_layout,
        &parameter_buffer,
        &state_buffers,
        &density_buffer,
        &cell_heads_buffer,
        &next_particle_buffer,
    );
    let render_bind_groups =
        state_render_bind_groups(device, &render_layout, &parameter_buffer, &state_buffers);
    let (
        VisualPipelines {
            clear_grid: clear_grid_pipeline,
            index_points: index_points_pipeline,
            indexed_density: indexed_density_pipeline,
            indexed_integrate: indexed_integrate_pipeline,
            all_pairs_density: all_pairs_density_pipeline,
            all_pairs_integrate: all_pairs_integrate_pipeline,
            guide: guide_pipeline,
            render: render_pipeline,
        },
        fluid_renderer,
    ) = create_rendering_resources(
        device,
        &compute_layout,
        &render_layout,
        surface_configuration,
    );
    Ok(VisualResources {
        parameter_buffer,
        state_buffers,
        compute_bind_groups,
        render_bind_groups,
        clear_grid_pipeline,
        index_points_pipeline,
        indexed_density_pipeline,
        indexed_integrate_pipeline,
        all_pairs_density_pipeline,
        all_pairs_integrate_pipeline,
        guide_pipeline,
        render_pipeline,
        fluid_renderer,
        initial_state,
    })
}

fn initial_parameter_buffer(
    device: &wgpu::Device,
    surface_configuration: &wgpu::SurfaceConfiguration,
) -> wgpu::Buffer {
    let initial_parameters = parameters(
        surface_configuration.width,
        surface_configuration.height,
        PARTICLE_PRESETS[DEFAULT_PRESET],
        FIT_CAMERA_YAW,
        FIT_CAMERA_PITCH,
        FIT_CAMERA_ZOOM,
        solver_substeps(PARTICLE_PRESETS[DEFAULT_PRESET]),
    );
    initialized_buffer(
        device,
        "Native water visual parameters",
        bytemuck::bytes_of(&initial_parameters),
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    )
}

fn create_rendering_resources(
    device: &wgpu::Device,
    compute_layout: &wgpu::BindGroupLayout,
    render_layout: &wgpu::BindGroupLayout,
    surface_configuration: &wgpu::SurfaceConfiguration,
) -> (VisualPipelines, FluidRenderer) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Native water visual shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("native_water_visual.wgsl").into()),
    });
    let pipelines = create_visual_pipelines(
        device,
        compute_layout,
        render_layout,
        &shader,
        surface_configuration.format,
    );
    let fluid_renderer = FluidRenderer::new(
        device,
        render_layout,
        &shader,
        surface_configuration.format,
        PhysicalSize::new(surface_configuration.width, surface_configuration.height),
    );
    (pipelines, fluid_renderer)
}

fn create_visual_pipelines(
    device: &wgpu::Device,
    compute_layout: &wgpu::BindGroupLayout,
    render_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> VisualPipelines {
    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Native water visual compute pipeline layout"),
        bind_group_layouts: &[Some(compute_layout)],
        immediate_size: 0,
    });
    let clear_grid = compute_pipeline(
        device,
        &compute_pipeline_layout,
        shader,
        "Native water visual clear grid pipeline",
        "clear_grid_main",
    );
    let index_points = compute_pipeline(
        device,
        &compute_pipeline_layout,
        shader,
        "Native water visual point indexing pipeline",
        "index_points_main",
    );
    let indexed_density = compute_pipeline(
        device,
        &compute_pipeline_layout,
        shader,
        "Native water visual indexed density pipeline",
        "indexed_density_main",
    );
    let indexed_integrate = compute_pipeline(
        device,
        &compute_pipeline_layout,
        shader,
        "Native water visual indexed integration pipeline",
        "indexed_integrate_main",
    );
    let all_pairs_density = compute_pipeline(
        device,
        &compute_pipeline_layout,
        shader,
        "Native water visual all-pairs density pipeline",
        "all_pairs_density_main",
    );
    let all_pairs_integrate = compute_pipeline(
        device,
        &compute_pipeline_layout,
        shader,
        "Native water visual all-pairs integration pipeline",
        "all_pairs_integrate_main",
    );
    let guide = guide_pipeline(device, render_layout, shader, format);
    let render = render_pipeline(device, render_layout, shader, format);
    VisualPipelines {
        clear_grid,
        index_points,
        indexed_density,
        indexed_integrate,
        all_pairs_density,
        all_pairs_integrate,
        guide,
        render,
    }
}

impl FluidRenderer {
    fn new(
        device: &wgpu::Device,
        particle_layout: &wgpu::BindGroupLayout,
        shader: &wgpu::ShaderModule,
        surface_format: wgpu::TextureFormat,
        size: PhysicalSize<u32>,
    ) -> Self {
        let texture_layout = fluid_texture_bind_group_layout(device);
        let particle_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Native water fluid particle pipeline layout"),
                bind_group_layouts: &[None, Some(particle_layout)],
                immediate_size: 0,
            });
        let fullscreen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Native water fluid fullscreen pipeline layout"),
                bind_group_layouts: &[None, None, Some(&texture_layout)],
                immediate_size: 0,
            });
        let depth_pipeline = fluid_particle_pipeline(
            device,
            &particle_pipeline_layout,
            shader,
            "Native water fluid depth pipeline",
            "surface_depth_fragment",
            Some(minimum_blend()),
        );
        let thickness_pipeline = fluid_particle_pipeline(
            device,
            &particle_pipeline_layout,
            shader,
            "Native water fluid thickness pipeline",
            "surface_thickness_fragment",
            Some(additive_blend()),
        );
        let horizontal_blur_pipeline = fluid_fullscreen_pipeline(
            device,
            &fullscreen_pipeline_layout,
            shader,
            "Native water horizontal bilateral blur pipeline",
            "horizontal_blur_fragment",
            wgpu::TextureFormat::Rgba16Float,
        );
        let vertical_blur_pipeline = fluid_fullscreen_pipeline(
            device,
            &fullscreen_pipeline_layout,
            shader,
            "Native water vertical bilateral blur pipeline",
            "vertical_blur_fragment",
            wgpu::TextureFormat::Rgba16Float,
        );
        let composite_pipeline = fluid_fullscreen_pipeline(
            device,
            &fullscreen_pipeline_layout,
            shader,
            "Native water fluid composite pipeline",
            "fluid_composite_fragment",
            surface_format,
        );
        let targets = FluidTargets::new(device, &texture_layout, size);
        Self {
            depth_pipeline,
            thickness_pipeline,
            horizontal_blur_pipeline,
            vertical_blur_pipeline,
            composite_pipeline,
            texture_layout,
            targets,
        }
    }

    fn resize(&mut self, device: &wgpu::Device, size: PhysicalSize<u32>) {
        self.targets = FluidTargets::new(device, &self.texture_layout, size);
    }

    fn encode(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        particle_bind_group: &wgpu::BindGroup,
        particle_count: u32,
    ) {
        Self::encode_particle_target(
            encoder,
            &self.targets.raw_depth_view,
            particle_bind_group,
            particle_count,
            &self.depth_pipeline,
            wgpu::Color::WHITE,
            "Native water fluid depth pass",
        );
        Self::encode_particle_target(
            encoder,
            &self.targets.thickness_view,
            particle_bind_group,
            particle_count,
            &self.thickness_pipeline,
            wgpu::Color::TRANSPARENT,
            "Native water fluid thickness pass",
        );
        Self::encode_fullscreen_target(
            encoder,
            &self.targets.blur_a_view,
            &self.targets.horizontal_bind_group,
            &self.horizontal_blur_pipeline,
            "Native water horizontal depth blur pass",
        );
        Self::encode_fullscreen_target(
            encoder,
            &self.targets.blur_b_view,
            &self.targets.vertical_bind_group,
            &self.vertical_blur_pipeline,
            "Native water vertical depth blur pass",
        );
        Self::encode_fullscreen_target(
            encoder,
            surface_view,
            &self.targets.composite_bind_group,
            &self.composite_pipeline,
            "Native water fluid composite pass",
        );
    }

    fn encode_particle_target(
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        particle_bind_group: &wgpu::BindGroup,
        particle_count: u32,
        pipeline: &wgpu::RenderPipeline,
        clear: wgpu::Color,
        label: &'static str,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(1, particle_bind_group, &[]);
        pass.draw(0..6, 0..particle_count);
    }

    fn encode_fullscreen_target(
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        texture_bind_group: &wgpu::BindGroup,
        pipeline: &wgpu::RenderPipeline,
        label: &'static str,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(2, texture_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

impl FluidTargets {
    fn new(
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        size: PhysicalSize<u32>,
    ) -> Self {
        let raw_depth = fluid_texture(device, size, "Native water raw fluid depth");
        let raw_depth_view = raw_depth.create_view(&wgpu::TextureViewDescriptor::default());
        let thickness = fluid_texture(device, size, "Native water fluid thickness");
        let thickness_view = thickness.create_view(&wgpu::TextureViewDescriptor::default());
        let horizontal_depth = fluid_texture(device, size, "Native water horizontal fluid depth");
        let horizontal_depth_view =
            horizontal_depth.create_view(&wgpu::TextureViewDescriptor::default());
        let vertical_depth = fluid_texture(device, size, "Native water vertical fluid depth");
        let vertical_depth_view =
            vertical_depth.create_view(&wgpu::TextureViewDescriptor::default());

        let horizontal_bind_group = fluid_texture_bind_group(
            device,
            texture_layout,
            [&raw_depth_view, &thickness_view, &thickness_view],
            "Native water horizontal blur textures",
        );
        let vertical_bind_group = fluid_texture_bind_group(
            device,
            texture_layout,
            [&raw_depth_view, &horizontal_depth_view, &thickness_view],
            "Native water vertical blur textures",
        );
        let composite_bind_group = fluid_texture_bind_group(
            device,
            texture_layout,
            [&raw_depth_view, &vertical_depth_view, &thickness_view],
            "Native water composite textures",
        );

        Self {
            _raw_depth: raw_depth,
            raw_depth_view,
            _thickness: thickness,
            thickness_view,
            _blur_a: horizontal_depth,
            blur_a_view: horizontal_depth_view,
            _blur_b: vertical_depth,
            blur_b_view: vertical_depth_view,
            horizontal_bind_group,
            vertical_bind_group,
            composite_bind_group,
        }
    }
}

fn fluid_texture(
    device: &wgpu::Device,
    size: PhysicalSize<u32>,
    label: &'static str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn fluid_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Native water fluid texture bindings"),
        entries: &[texture_entry(0), texture_entry(1), texture_entry(2)],
    })
}

fn fluid_texture_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    views: [&wgpu::TextureView; 3],
    label: &'static str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(views[0]),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(views[1]),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(views[2]),
            },
        ],
    })
}

fn fluid_particle_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    label: &'static str,
    fragment_entry: &'static str,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("surface_vertex"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn fluid_fullscreen_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    label: &'static str,
    fragment_entry: &'static str,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("fullscreen_vertex"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

const fn minimum_blend() -> wgpu::BlendState {
    let component = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Min,
    };
    wgpu::BlendState {
        color: component,
        alpha: component,
    }
}

const fn additive_blend() -> wgpu::BlendState {
    let component = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    wgpu::BlendState {
        color: component,
        alpha: component,
    }
}

#[derive(Debug, Default)]
struct NativeWaterApplication {
    water: Option<NativeWater>,
}

impl ApplicationHandler for NativeWaterApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.water.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Native Water — starting Vulkan")
            .with_inner_size(LogicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("Could not create the native water window: {error}");
                event_loop.exit();
                return;
            }
        };
        match pollster::block_on(NativeWater::new(window)) {
            Ok(water) => {
                water.window.request_redraw();
                self.water = Some(water);
            }
            Err(error) => {
                eprintln!("Could not initialize native Vulkan water: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(water) = self.water.as_mut() else {
            return;
        };
        if window_id != water.window.id() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => water.resize(size),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                    PhysicalKey::Code(KeyCode::Space) => water.toggle_pause(),
                    PhysicalKey::Code(KeyCode::KeyR) => water.reset(),
                    PhysicalKey::Code(KeyCode::KeyF | KeyCode::Home) => water.fit_camera(),
                    PhysicalKey::Code(KeyCode::KeyI) => water.toggle_simulation_mode(),
                    PhysicalKey::Code(KeyCode::KeyW) => water.toggle_render_mode(),
                    PhysicalKey::Code(KeyCode::ArrowUp | KeyCode::Equal) => {
                        if let Err(error) = water.select_more_particles() {
                            eprintln!("Could not increase the particle count: {error}");
                            event_loop.exit();
                        }
                    }
                    PhysicalKey::Code(KeyCode::ArrowDown | KeyCode::Minus) => {
                        if let Err(error) = water.select_fewer_particles() {
                            eprintln!("Could not decrease the particle count: {error}");
                            event_loop.exit();
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => water.set_dragging(state == ElementState::Pressed),
            WindowEvent::CursorMoved { position, .. } => water.move_cursor(position),
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, vertical) => vertical,
                    MouseScrollDelta::PixelDelta(position) => {
                        position
                            .y
                            .to_f32()
                            .expect("finite wheel delta has an f32 representation")
                            / 120.0
                    }
                };
                water.zoom(lines);
            }
            WindowEvent::RedrawRequested => {
                match water.render() {
                    Ok(FrameOutcome::Presented | FrameOutcome::Skipped) => {}
                    Ok(FrameOutcome::Reconfigure) => water.reconfigure(),
                    Ok(FrameOutcome::Recreate) => {
                        if let Err(error) = water.recreate_surface() {
                            eprintln!("Could not recreate the Vulkan surface: {error}");
                            event_loop.exit();
                        }
                    }
                    Err(error) => {
                        eprintln!("Native Vulkan water stopped: {error}");
                        event_loop.exit();
                    }
                }
                water.window.request_redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(water) = &self.water {
            water.window.request_redraw();
        }
    }
}

fn main() -> AppResult<()> {
    match Arguments::parse().command {
        Some(Command::BenchmarkCpu {
            particles,
            steps,
            warmup_steps,
        }) => return benchmark_cpu(particles, steps, warmup_steps),
        None => {}
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut application = NativeWaterApplication::default();
    event_loop.run_app(&mut application)?;
    Ok(())
}

fn benchmark_cpu(particle_count: u32, steps: u32, warmup_steps: u32) -> AppResult<()> {
    let preset = PARTICLE_PRESETS
        .into_iter()
        .find(|preset| preset.count == particle_count)
        .ok_or_else(|| {
            app_error(format!(
                "particle count {particle_count} is not a native-water preset"
            ))
        })?;
    let substeps = solver_substeps(preset);
    let parameters = parameters(1, 1, preset, 0.0, 0.0, 1.0, substeps);
    let initial = initial_particles(preset)?;
    let mut fluid = cpu_fluid::CpuFluid::new(&initial, &parameters);

    for _ in 0..warmup_steps {
        fluid.step(&parameters);
    }

    let started = Instant::now();
    for _ in 0..steps {
        fluid.step(&parameters);
    }
    let elapsed = started.elapsed();
    std::hint::black_box(fluid.state());

    let elapsed_seconds = elapsed.as_secs_f64();
    let milliseconds_per_step = elapsed_seconds * 1_000.0 / f64::from(steps);
    let math_backend = if cfg!(feature = "cpu-simd") {
        "SIMD"
    } else {
        "scalar"
    };
    println!(
        "CPU indexed WCSPH ({math_backend}): {particle_count} particles, {steps} steps, \
         {milliseconds_per_step:.3} ms/step, {:.1} steps/s",
        f64::from(steps) / elapsed_seconds
    );
    Ok(())
}

fn grid_shape(smoothing_radius: f32) -> [u32; 4] {
    assert!(
        smoothing_radius.is_finite() && smoothing_radius > 0.0,
        "a particle preset must have a positive finite smoothing radius"
    );
    let cells_for_bound = |bound: f32| {
        ((bound * 2.0) / smoothing_radius)
            .ceil()
            .to_u32()
            .expect("the finite simulation box has a u32 cell count")
            .max(1)
    };
    let x = cells_for_bound(BOX_BOUNDS[0]);
    let y = cells_for_bound(BOX_BOUNDS[1]);
    let z = cells_for_bound(BOX_BOUNDS[2]);
    let count = x
        .checked_mul(y)
        .and_then(|xy| xy.checked_mul(z))
        .expect("the finite simulation grid cell count fits u32");
    [x, y, z, count]
}

fn solver_substeps(preset: ParticlePreset) -> u32 {
    (REFERENCE_STABLE_SMOOTHING_RADIUS / preset.smoothing_radius)
        .ceil()
        .to_u32()
        .expect("the finite preset ratio has a u32 substep count")
        .max(1)
}

fn parameters(
    width: u32,
    height: u32,
    preset: ParticlePreset,
    camera_yaw: f32,
    camera_pitch: f32,
    camera_zoom: f32,
    substeps: u32,
) -> SimParameters {
    let mass = REST_DENSITY * preset.spacing.powi(3);
    let aspect = width
        .to_f32()
        .expect("u32 window width has a finite f32 representation")
        / height
            .max(1)
            .to_f32()
            .expect("u32 window height has a finite f32 representation");
    SimParameters {
        particle_count: preset.count,
        delta_time: FIXED_STEP_SECONDS
            / substeps
                .to_f32()
                .expect("the small substep count is exactly representable"),
        smoothing_radius: preset.smoothing_radius,
        rest_density: REST_DENSITY,
        pressure_stiffness: PRESSURE_STIFFNESS,
        viscosity: VISCOSITY,
        mass,
        gravity: GRAVITY,
        bounds: [
            BOX_BOUNDS[0],
            BOX_BOUNDS[1],
            BOX_BOUNDS[2],
            BOUNDARY_DAMPING,
        ],
        projection: [
            aspect,
            (preset.spacing * 0.093).clamp(0.0015, 0.018),
            preset.spacing * 0.50,
            0.0,
        ],
        camera: [camera_yaw, camera_pitch, camera_zoom, 0.0],
        grid: grid_shape(preset.smoothing_radius),
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "the finite particle lattice dimensions are exactly representable as f32"
)]
fn initial_particles(preset: ParticlePreset) -> AppResult<Vec<ParticleState>> {
    let expected_count = preset
        .dimensions
        .iter()
        .try_fold(1_usize, |total, count| total.checked_mul(*count))
        .ok_or_else(|| app_error("particle lattice size overflow"))?;
    if expected_count != usize::try_from(preset.count)? {
        return Err(app_error(
            "particle preset dimensions and declared count differ",
        ));
    }
    let mut particles = Vec::with_capacity(expected_count);
    let width = (preset.dimensions[0] - 1) as f32 * preset.spacing;
    let height = (preset.dimensions[1] - 1) as f32 * preset.spacing;
    let depth = (preset.dimensions[2] - 1) as f32 * preset.spacing;
    for y in 0..preset.dimensions[1] {
        for z in 0..preset.dimensions[2] {
            for x in 0..preset.dimensions[0] {
                let phase = (x
                    + z * preset.dimensions[0]
                    + y * preset.dimensions[0] * preset.dimensions[2])
                    as f32;
                particles.push(ParticleState {
                    position: [
                        x as f32 * preset.spacing - width * 0.5,
                        y as f32 * preset.spacing - height * 0.5 - 0.30,
                        z as f32 * preset.spacing - depth * 0.5,
                        1.0,
                    ],
                    velocity: [
                        (phase * 0.017).sin() * 0.08,
                        0.0,
                        (phase * 0.013).cos() * 0.04,
                        0.0,
                    ],
                });
            }
        }
    }
    Ok(particles)
}

fn compute_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Native water visual compute bindings"),
        entries: &[
            buffer_layout(
                0,
                wgpu::BufferBindingType::Uniform,
                wgpu::ShaderStages::COMPUTE,
            ),
            buffer_layout(1, storage_binding(true), wgpu::ShaderStages::COMPUTE),
            buffer_layout(2, storage_binding(false), wgpu::ShaderStages::COMPUTE),
            buffer_layout(3, storage_binding(false), wgpu::ShaderStages::COMPUTE),
            buffer_layout(4, storage_binding(false), wgpu::ShaderStages::COMPUTE),
            buffer_layout(5, storage_binding(false), wgpu::ShaderStages::COMPUTE),
        ],
    })
}

fn render_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Native water visual render bindings"),
        entries: &[
            buffer_layout(
                0,
                wgpu::BufferBindingType::Uniform,
                wgpu::ShaderStages::VERTEX,
            ),
            buffer_layout(1, storage_binding(true), wgpu::ShaderStages::VERTEX),
        ],
    })
}

fn buffer_layout(
    binding: u32,
    ty: wgpu::BufferBindingType,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_binding(read_only: bool) -> wgpu::BufferBindingType {
    wgpu::BufferBindingType::Storage { read_only }
}

fn bidirectional_compute_bind_groups(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    parameters: &wgpu::Buffer,
    states: &[wgpu::Buffer; 2],
    density: &wgpu::Buffer,
    cell_heads: &wgpu::Buffer,
    next_particle: &wgpu::Buffer,
) -> [wgpu::BindGroup; 2] {
    [
        compute_bind_group(
            device,
            layout,
            ComputeBuffers {
                parameters,
                input: &states[0],
                density,
                output: &states[1],
                cell_heads,
                next_particle,
            },
            "Native water visual A to B",
        ),
        compute_bind_group(
            device,
            layout,
            ComputeBuffers {
                parameters,
                input: &states[1],
                density,
                output: &states[0],
                cell_heads,
                next_particle,
            },
            "Native water visual B to A",
        ),
    ]
}

#[derive(Clone, Copy, Debug)]
struct ComputeBuffers<'a> {
    parameters: &'a wgpu::Buffer,
    input: &'a wgpu::Buffer,
    density: &'a wgpu::Buffer,
    output: &'a wgpu::Buffer,
    cell_heads: &'a wgpu::Buffer,
    next_particle: &'a wgpu::Buffer,
}

fn state_render_bind_groups(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    parameters: &wgpu::Buffer,
    states: &[wgpu::Buffer; 2],
) -> [wgpu::BindGroup; 2] {
    [
        render_bind_group(
            device,
            layout,
            parameters,
            &states[0],
            "Native water visual render A",
        ),
        render_bind_group(
            device,
            layout,
            parameters,
            &states[1],
            "Native water visual render B",
        ),
    ]
}

fn compute_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffers: ComputeBuffers<'_>,
    label: &'static str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            binding(0, buffers.parameters),
            binding(1, buffers.input),
            binding(2, buffers.density),
            binding(3, buffers.output),
            binding(4, buffers.cell_heads),
            binding(5, buffers.next_particle),
        ],
    })
}

fn render_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    parameters: &wgpu::Buffer,
    state: &wgpu::Buffer,
    label: &'static str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[binding(0, parameters), binding(1, state)],
    })
}

fn binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn initialized_buffer(
    device: &wgpu::Device,
    label: &'static str,
    contents: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents,
        usage,
    })
}

fn compute_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    label: &'static str,
    entry_point: &'static str,
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

fn guide_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Native water visual guide pipeline layout"),
        bind_group_layouts: &[None, Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Native water visual guide pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("guide_vertex"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("guide_fragment"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn render_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Native water visual render pipeline layout"),
        bind_group_layouts: &[None, Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Native water visual particle pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vertex_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fragment_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn byte_size<T>(length: usize) -> AppResult<u64> {
    let bytes = length
        .checked_mul(size_of::<T>())
        .ok_or_else(|| app_error("GPU buffer size overflow"))?;
    Ok(u64::try_from(bytes)?)
}

fn app_error(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    Box::new(io::Error::other(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_initial_state_matches_its_declared_particle_count() {
        for preset in PARTICLE_PRESETS {
            let particles = initial_particles(preset).unwrap();

            assert_eq!(particles.len(), usize::try_from(preset.count).unwrap());
            assert!(particles.iter().all(|particle| {
                particle
                    .position
                    .iter()
                    .chain(&particle.velocity)
                    .all(|value| value.is_finite())
            }));
            assert!(particles.iter().all(|particle| {
                particle.position[0].abs() <= BOX_BOUNDS[0]
                    && particle.position[1].abs() <= BOX_BOUNDS[1]
                    && particle.position[2].abs() <= BOX_BOUNDS[2]
            }));
        }
    }

    #[test]
    fn gpu_layouts_keep_expected_alignment() {
        assert_eq!(size_of::<ParticleState>(), 32);
        assert_eq!(size_of::<SimParameters>(), 96);
    }

    #[test]
    fn grid_shape_covers_each_axis_with_smoothing_radius_cells() {
        for preset in PARTICLE_PRESETS {
            let shape = grid_shape(preset.smoothing_radius);
            assert_eq!(shape[3], shape[0] * shape[1] * shape[2]);
            for (axis, bound) in BOX_BOUNDS.into_iter().enumerate() {
                let cells = shape[axis]
                    .to_f32()
                    .expect("the small grid dimension is exactly representable");
                assert!(cells * preset.smoothing_radius >= bound * 2.0);
                assert!((cells - 1.0) * preset.smoothing_radius < bound * 2.0);
            }
        }
    }

    #[test]
    fn presets_are_ordered_and_fit_the_gpu_capacity() {
        assert!(
            PARTICLE_PRESETS
                .windows(2)
                .all(|pair| pair[0].count < pair[1].count)
        );
        assert_eq!(PARTICLE_PRESETS.last().unwrap().count, MAX_PARTICLE_COUNT);
        assert!(PARTICLE_PRESETS.iter().all(|preset| {
            let support_ratio = preset.smoothing_radius / preset.spacing;
            (1.8..=2.0).contains(&support_ratio)
        }));
    }

    #[test]
    fn solver_substeps_keep_translation_scale_within_the_stable_reference() {
        let counts = PARTICLE_PRESETS.map(solver_substeps);
        assert!(counts.windows(2).all(|pair| pair[0] <= pair[1]));
        assert_eq!(counts[0], 1);
        assert_eq!(counts[counts.len() - 1], 4);
        for preset in PARTICLE_PRESETS {
            let substeps = solver_substeps(preset)
                .to_f32()
                .expect("the small substep count is exactly representable");
            let translation_ratio = FIXED_STEP_SECONDS / substeps / preset.smoothing_radius;
            let stable_ratio = FIXED_STEP_SECONDS / REFERENCE_STABLE_SMOOTHING_RADIUS;
            assert!(translation_ratio <= stable_ratio + f32::EPSILON);
        }
    }

    #[test]
    fn render_mode_toggle_is_closed_and_reversible() {
        let initial = RenderMode::default();

        assert_eq!(initial, RenderMode::Surface);
        assert_eq!(initial.toggled(), RenderMode::Points);
        assert_eq!(initial.toggled().toggled(), initial);
    }

    #[test]
    fn supported_simulation_modes_form_one_closed_cycle() {
        let particle_count = MAX_ALL_PAIRS_PARTICLE_COUNT;
        let initial = SimulationMode::default();
        let gpu_indexed = initial.toggled(particle_count);
        let all_pairs = gpu_indexed.toggled(particle_count);

        assert_eq!(initial, SimulationMode::CpuIndexed);
        assert_eq!(gpu_indexed, SimulationMode::GpuIndexed);
        assert_eq!(all_pairs, SimulationMode::AllPairs);
        assert_eq!(all_pairs.toggled(particle_count), initial);
    }

    #[test]
    fn large_particle_mode_cycle_keeps_cpu_and_gpu_indexed_available() {
        let particle_count = MAX_ALL_PAIRS_PARTICLE_COUNT + 1;
        let initial = SimulationMode::default();
        let gpu_indexed = initial.toggled(particle_count);

        assert_eq!(initial, SimulationMode::CpuIndexed);
        assert_eq!(gpu_indexed, SimulationMode::GpuIndexed);
        assert_eq!(gpu_indexed.toggled(particle_count), initial);
    }

    #[test]
    fn cpu_indexed_step_applies_gravity_to_an_isolated_particle() {
        let input = vec![ParticleState {
            position: [0.0, 0.0, 0.0, 1.0],
            velocity: [0.0; 4],
        }];
        let mut parameters = parameters(1280, 800, PARTICLE_PRESETS[0], 0.0, 0.0, 1.0, 1);
        parameters.particle_count = 1;
        let mut fluid = cpu_fluid::CpuFluid::new(&input, &parameters);

        fluid.step(&parameters);

        let output = fluid.state()[0];
        let expected_velocity = GRAVITY * parameters.delta_time;
        let expected_position = expected_velocity * parameters.delta_time;
        assert!((output.velocity[1] - expected_velocity).abs() < 1.0e-7);
        assert!((output.position[1] - expected_position).abs() < 1.0e-7);
        assert_eq!(output.position[3], 1.0);
        assert_eq!(output.velocity[3], 0.0);
    }

    #[test]
    fn cpu_indexed_step_stays_finite_and_inside_the_box() {
        let preset = PARTICLE_PRESETS[DEFAULT_PRESET];
        let input = initial_particles(preset).unwrap();
        let parameters = parameters(1280, 800, preset, 0.0, 0.0, 1.0, solver_substeps(preset));
        let mut fluid = cpu_fluid::CpuFluid::new(&input, &parameters);

        for _ in 0..solver_substeps(preset) {
            fluid.step(&parameters);
        }

        assert!(fluid.state().iter().all(|particle| {
            particle
                .position
                .iter()
                .chain(&particle.velocity)
                .all(|value| value.is_finite())
        }));
        assert!(fluid.state().iter().all(|particle| {
            particle.position[0].abs() <= BOX_BOUNDS[0]
                && particle.position[1].abs() <= BOX_BOUNDS[1]
                && particle.position[2].abs() <= BOX_BOUNDS[2]
        }));
    }
}
