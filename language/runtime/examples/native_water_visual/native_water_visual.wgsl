struct SimParameters {
    particle_count: u32,
    delta_time: f32,
    smoothing_radius: f32,
    rest_density: f32,
    pressure_stiffness: f32,
    viscosity: f32,
    mass: f32,
    gravity: f32,
    bounds: vec4<f32>,
    projection: vec4<f32>,
    camera: vec4<f32>,
    grid: vec4<u32>,
}

struct ParticleState {
    position: vec4<f32>,
    velocity: vec4<f32>,
}

@group(0) @binding(0) var<uniform> parameters: SimParameters;
@group(0) @binding(1) var<storage, read> input_state: array<ParticleState>;
@group(0) @binding(2) var<storage, read_write> density_pressure: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read_write> output_state: array<ParticleState>;
@group(0) @binding(4) var<storage, read_write> cell_heads: array<atomic<i32>>;
@group(0) @binding(5) var<storage, read_write> next_particle: array<i32>;

const PI: f32 = 3.14159265358979323846;
const MAX_PARTICLE_SPEED: f32 = 6.0;

fn point_cell(position: vec3<f32>) -> vec3<u32> {
    let unbounded = floor((position + parameters.bounds.xyz) / parameters.smoothing_radius);
    let maximum = vec3<f32>(parameters.grid.xyz - vec3<u32>(1u));
    return vec3<u32>(clamp(unbounded, vec3<f32>(0.0), maximum));
}

fn cell_index(cell: vec3<u32>) -> u32 {
    return cell.x + parameters.grid.x * (cell.y + parameters.grid.y * cell.z);
}

fn limited_velocity(value: vec3<f32>) -> vec3<f32> {
    var velocity = value;
    let speed = length(velocity);
    if speed > MAX_PARTICLE_SPEED {
        velocity *= MAX_PARTICLE_SPEED / speed;
    }
    return velocity;
}

fn write_constrained_state(
    particle: u32,
    unconstrained_position: vec3<f32>,
    unconstrained_velocity: vec3<f32>,
) {
    var next_position = unconstrained_position;
    var velocity = unconstrained_velocity;
    let damping = parameters.bounds.w;

    if next_position.x < -parameters.bounds.x {
        next_position.x = -parameters.bounds.x;
        velocity.x = abs(velocity.x) * damping;
    } else if next_position.x > parameters.bounds.x {
        next_position.x = parameters.bounds.x;
        velocity.x = -abs(velocity.x) * damping;
    }
    if next_position.y < -parameters.bounds.y {
        next_position.y = -parameters.bounds.y;
        velocity.y = abs(velocity.y) * damping;
    } else if next_position.y > parameters.bounds.y {
        next_position.y = parameters.bounds.y;
        velocity.y = -abs(velocity.y) * damping;
    }
    if next_position.z < -parameters.bounds.z {
        next_position.z = -parameters.bounds.z;
        velocity.z = abs(velocity.z) * damping;
    } else if next_position.z > parameters.bounds.z {
        next_position.z = parameters.bounds.z;
        velocity.z = -abs(velocity.z) * damping;
    }

    output_state[particle] = ParticleState(
        vec4<f32>(next_position, 1.0),
        vec4<f32>(velocity, 0.0),
    );
}

fn write_integrated_state(
    particle: u32,
    state: ParticleState,
    acceleration: vec3<f32>,
) {
    let velocity = limited_velocity(
        state.velocity.xyz + acceleration * parameters.delta_time,
    );
    let next_position = state.position.xyz + velocity * parameters.delta_time;
    write_constrained_state(particle, next_position, velocity);
}

@compute @workgroup_size(128)
fn clear_grid_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let cell = invocation.x;
    if cell < parameters.grid.w {
        atomicStore(&cell_heads[cell], -1);
    }
}

@compute @workgroup_size(128)
fn index_points_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let particle = invocation.x;
    if particle >= parameters.particle_count {
        return;
    }

    let cell = cell_index(point_cell(input_state[particle].position.xyz));
    next_particle[particle] = atomicExchange(&cell_heads[cell], i32(particle));
}

@compute @workgroup_size(128)
fn indexed_density_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let particle = invocation.x;
    if particle >= parameters.particle_count {
        return;
    }

    let position = input_state[particle].position.xyz;
    let own_cell = vec3<i32>(point_cell(position));
    let dimensions = vec3<i32>(parameters.grid.xyz);
    let h = parameters.smoothing_radius;
    let h_squared = h * h;
    let poly6 = 315.0 / (64.0 * PI * pow(h, 9.0));
    var density = parameters.mass * poly6 * pow(h_squared, 3.0);

    for (var z: i32 = -1; z <= 1; z += 1) {
        for (var y: i32 = -1; y <= 1; y += 1) {
            for (var x: i32 = -1; x <= 1; x += 1) {
                let neighbor = own_cell + vec3<i32>(x, y, z);
                if any(neighbor < vec3<i32>(0)) || any(neighbor >= dimensions) {
                    continue;
                }
                var other = atomicLoad(&cell_heads[cell_index(vec3<u32>(neighbor))]);
                while other >= 0 {
                    let other_particle = u32(other);
                    if other_particle != particle {
                        let displacement = position - input_state[other_particle].position.xyz;
                        let distance_squared = dot(displacement, displacement);
                        if distance_squared < h_squared {
                            density += parameters.mass * poly6
                                * pow(h_squared - distance_squared, 3.0);
                        }
                    }
                    other = next_particle[other_particle];
                }
            }
        }
    }

    density = max(density, parameters.rest_density * 0.05);
    let pressure = parameters.pressure_stiffness
        * max(density - parameters.rest_density, 0.0);
    density_pressure[particle] = vec2<f32>(density, pressure);
}

@compute @workgroup_size(128)
fn indexed_integrate_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let particle = invocation.x;
    if particle >= parameters.particle_count {
        return;
    }

    let state = input_state[particle];
    let position = state.position.xyz;
    let own_cell = vec3<i32>(point_cell(position));
    let dimensions = vec3<i32>(parameters.grid.xyz);
    let own_density_pressure = density_pressure[particle];
    let h = parameters.smoothing_radius;
    let h_squared = h * h;
    let spiky_gradient = -45.0 / (PI * pow(h, 6.0));
    let viscosity_laplacian = 45.0 / (PI * pow(h, 6.0));
    var acceleration = vec3<f32>(0.0, parameters.gravity, 0.0);

    for (var z: i32 = -1; z <= 1; z += 1) {
        for (var y: i32 = -1; y <= 1; y += 1) {
            for (var x: i32 = -1; x <= 1; x += 1) {
                let neighbor = own_cell + vec3<i32>(x, y, z);
                if any(neighbor < vec3<i32>(0)) || any(neighbor >= dimensions) {
                    continue;
                }
                var other = atomicLoad(&cell_heads[cell_index(vec3<u32>(neighbor))]);
                while other >= 0 {
                    let other_particle = u32(other);
                    if other_particle != particle {
                        let other_state = input_state[other_particle];
                        let displacement = position - other_state.position.xyz;
                        let distance_squared = dot(displacement, displacement);
                        if distance_squared > 1.0e-8 && distance_squared < h_squared {
                            let distance = sqrt(distance_squared);
                            let remainder = h - distance;
                            let other_density_pressure = density_pressure[other_particle];
                            let pressure_gain = -parameters.mass * spiky_gradient
                                * remainder * remainder / distance;
                            let pressure_scale = pressure_gain
                                * (own_density_pressure.y
                                    / (own_density_pressure.x * own_density_pressure.x)
                                    + other_density_pressure.y
                                    / (other_density_pressure.x * other_density_pressure.x));
                            acceleration += pressure_scale * displacement;

                            let viscosity_gain = parameters.viscosity * parameters.mass
                                * viscosity_laplacian * remainder;
                            acceleration += viscosity_gain
                                * (other_state.velocity.xyz - state.velocity.xyz)
                                / other_density_pressure.x;
                        }
                    }
                    other = next_particle[other_particle];
                }
            }
        }
    }

    write_integrated_state(particle, state, acceleration);
}

@compute @workgroup_size(128)
fn all_pairs_density_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let particle = invocation.x;
    if particle >= parameters.particle_count {
        return;
    }

    let position = input_state[particle].position.xyz;
    let h = parameters.smoothing_radius;
    let h_squared = h * h;
    let poly6 = 315.0 / (64.0 * PI * pow(h, 9.0));
    var density = parameters.mass * poly6 * pow(h_squared, 3.0);

    for (var other = 0u; other < parameters.particle_count; other += 1u) {
        if other == particle {
            continue;
        }
        let displacement = position - input_state[other].position.xyz;
        let distance_squared = dot(displacement, displacement);
        if distance_squared < h_squared {
            density += parameters.mass * poly6 * pow(h_squared - distance_squared, 3.0);
        }
    }

    density = max(density, parameters.rest_density * 0.05);
    let pressure = parameters.pressure_stiffness
        * max(density - parameters.rest_density, 0.0);
    density_pressure[particle] = vec2<f32>(density, pressure);
}

@compute @workgroup_size(128)
fn all_pairs_integrate_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let particle = invocation.x;
    if particle >= parameters.particle_count {
        return;
    }

    let state = input_state[particle];
    let position = state.position.xyz;
    let own_density_pressure = density_pressure[particle];
    let h = parameters.smoothing_radius;
    let h_squared = h * h;
    let spiky_gradient = -45.0 / (PI * pow(h, 6.0));
    let viscosity_laplacian = 45.0 / (PI * pow(h, 6.0));
    var acceleration = vec3<f32>(0.0, parameters.gravity, 0.0);

    for (var other = 0u; other < parameters.particle_count; other += 1u) {
        if other == particle {
            continue;
        }
        let other_state = input_state[other];
        let displacement = position - other_state.position.xyz;
        let distance_squared = dot(displacement, displacement);
        if distance_squared <= 1.0e-8 || distance_squared >= h_squared {
            continue;
        }

        let distance = sqrt(distance_squared);
        let remainder = h - distance;
        let other_density_pressure = density_pressure[other];
        let pressure_gain = -parameters.mass * spiky_gradient
            * remainder * remainder / distance;
        let pressure_scale = pressure_gain
            * (own_density_pressure.y
                / (own_density_pressure.x * own_density_pressure.x)
                + other_density_pressure.y
                / (other_density_pressure.x * other_density_pressure.x));
        acceleration += pressure_scale * displacement;

        let viscosity_gain = parameters.viscosity * parameters.mass
            * viscosity_laplacian * remainder;
        acceleration += viscosity_gain
            * (other_state.velocity.xyz - state.velocity.xyz)
            / other_density_pressure.x;
    }

    write_integrated_state(particle, state, acceleration);
}

@group(1) @binding(0) var<uniform> render_parameters: SimParameters;
@group(1) @binding(1) var<storage, read> render_state: array<ParticleState>;

const QUAD = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
);

// Leaves room for particle radius and guide-line thickness at the fitted view.
const CAMERA_FIT_MARGIN: f32 = 1.14;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) center_depth: f32,
    @location(3) depth_radius: f32,
}

fn project_world(world: vec3<f32>) -> vec3<f32> {
    let yaw_cos = cos(render_parameters.camera.x);
    let yaw_sin = sin(render_parameters.camera.x);
    let yaw_position = vec3<f32>(
        yaw_cos * world.x + yaw_sin * world.z,
        world.y,
        -yaw_sin * world.x + yaw_cos * world.z,
    );
    let pitch_cos = cos(render_parameters.camera.y);
    let pitch_sin = sin(render_parameters.camera.y);
    let pitch_position = vec3<f32>(
        yaw_position.x,
        pitch_cos * yaw_position.y - pitch_sin * yaw_position.z,
        pitch_sin * yaw_position.y + pitch_cos * yaw_position.z,
    );

    // The orthographic camera must fit the rotated box, not its unrotated axes.
    // A single world-space scale preserves shape while the aspect ratio maps x.
    let yaw_x_extent = abs(yaw_cos) * render_parameters.bounds.x
        + abs(yaw_sin) * render_parameters.bounds.z;
    let yaw_z_extent = abs(yaw_sin) * render_parameters.bounds.x
        + abs(yaw_cos) * render_parameters.bounds.z;
    let pitch_y_extent = abs(pitch_cos) * render_parameters.bounds.y
        + abs(pitch_sin) * yaw_z_extent;
    let pitch_z_extent = abs(pitch_sin) * render_parameters.bounds.y
        + abs(pitch_cos) * yaw_z_extent;
    let aspect = max(render_parameters.projection.x, 1.0e-6);
    let view_scale = max(pitch_y_extent, yaw_x_extent / aspect) * CAMERA_FIT_MARGIN;
    let center = vec2<f32>(
        pitch_position.x / (view_scale * aspect),
        pitch_position.y / view_scale,
    ) * render_parameters.camera.z;

    // Vulkan clips depth outside 0..1. Centering raw camera depth around zero
    // discarded the negative half of the box as if a plane cut through it.
    let depth_scale = max(pitch_z_extent * CAMERA_FIT_MARGIN, 1.0e-6);
    let depth = clamp(0.5 - pitch_position.z / (2.0 * depth_scale), 0.0, 1.0);
    return vec3<f32>(center, depth);
}

fn projected_depth_radius(world_radius: f32) -> f32 {
    let yaw_cos = cos(render_parameters.camera.x);
    let yaw_sin = sin(render_parameters.camera.x);
    let yaw_z_extent = abs(yaw_sin) * render_parameters.bounds.x
        + abs(yaw_cos) * render_parameters.bounds.z;
    let pitch_cos = cos(render_parameters.camera.y);
    let pitch_sin = sin(render_parameters.camera.y);
    let pitch_z_extent = abs(pitch_sin) * render_parameters.bounds.y
        + abs(pitch_cos) * yaw_z_extent;
    let depth_scale = max(pitch_z_extent * CAMERA_FIT_MARGIN, 1.0e-6);
    return world_radius / (2.0 * depth_scale);
}

fn particle_vertex(particle: u32, corner: vec2<f32>, radius_scale: f32) -> VertexOutput {
    let state = render_state[particle];
    let world = state.position.xyz;
    let projected = project_world(world);
    let radius = render_parameters.projection.y * render_parameters.camera.z * radius_scale;
    let offset = vec2<f32>(
        corner.x * radius / render_parameters.projection.x,
        corner.y * radius,
    );
    let speed = min(length(state.velocity.xyz) / 4.0, 1.0);
    let depth = clamp((world.z / render_parameters.bounds.z + 1.0) * 0.5, 0.0, 1.0);
    let deep = vec3<f32>(0.02, 0.34, 0.78);
    let bright = vec3<f32>(0.18, 0.88, 1.0);
    let color = mix(deep, bright, 0.18 + speed * 0.62 + depth * 0.20);

    var output: VertexOutput;
    output.position = vec4<f32>(projected.xy + offset, projected.z, 1.0);
    output.local_position = corner;
    output.color = color;
    output.center_depth = projected.z;
    output.depth_radius = projected_depth_radius(render_parameters.projection.z);
    return output;
}

@vertex
fn vertex_main(
    @builtin(vertex_index) vertex: u32,
    @builtin(instance_index) particle: u32,
) -> VertexOutput {
    return particle_vertex(particle, QUAD[vertex], 1.0);
}

@vertex
fn surface_vertex(
    @builtin(vertex_index) vertex: u32,
    @builtin(instance_index) particle: u32,
) -> VertexOutput {
    return particle_vertex(particle, QUAD[vertex], 2.2);
}

const GUIDE_POINTS = array<vec3<f32>, 30>(
    vec3<f32>(-1.0, -1.0, -1.0), vec3<f32>(1.0, -1.0, -1.0),
    vec3<f32>(1.0, -1.0, -1.0), vec3<f32>(1.0, 1.0, -1.0),
    vec3<f32>(1.0, 1.0, -1.0), vec3<f32>(-1.0, 1.0, -1.0),
    vec3<f32>(-1.0, 1.0, -1.0), vec3<f32>(-1.0, -1.0, -1.0),
    vec3<f32>(-1.0, -1.0, 1.0), vec3<f32>(1.0, -1.0, 1.0),
    vec3<f32>(1.0, -1.0, 1.0), vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(-1.0, 1.0, 1.0),
    vec3<f32>(-1.0, 1.0, 1.0), vec3<f32>(-1.0, -1.0, 1.0),
    vec3<f32>(-1.0, -1.0, -1.0), vec3<f32>(-1.0, -1.0, 1.0),
    vec3<f32>(1.0, -1.0, -1.0), vec3<f32>(1.0, -1.0, 1.0),
    vec3<f32>(1.0, 1.0, -1.0), vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(-1.0, 1.0, -1.0), vec3<f32>(-1.0, 1.0, 1.0),
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0),
);

struct GuideOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn guide_vertex(@builtin(vertex_index) vertex: u32) -> GuideOutput {
    let world = GUIDE_POINTS[vertex] * render_parameters.bounds.xyz;
    let projected = project_world(world);
    var color = vec3<f32>(0.22, 0.34, 0.48);
    if vertex >= 24u && vertex < 26u {
        color = vec3<f32>(1.0, 0.22, 0.18);
    } else if vertex < 28u && vertex >= 26u {
        color = vec3<f32>(0.20, 1.0, 0.38);
    } else if vertex >= 28u {
        color = vec3<f32>(0.18, 0.48, 1.0);
    }
    var output: GuideOutput;
    output.position = vec4<f32>(projected.xy, projected.z, 1.0);
    output.color = color;
    return output;
}

@fragment
fn guide_fragment(input: GuideOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 0.72);
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let radius_squared = dot(input.local_position, input.local_position);
    if radius_squared > 1.0 {
        discard;
    }
    let edge = 1.0 - smoothstep(0.45, 1.0, radius_squared);
    let highlight = pow(max(0.0, 1.0 - length(input.local_position + vec2<f32>(0.32, 0.32))), 5.0);
    return vec4<f32>(input.color + highlight * 0.55, 0.42 + edge * 0.48);
}

@fragment
fn surface_depth_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let radius_squared = dot(input.local_position, input.local_position);
    if radius_squared > 1.0 {
        discard;
    }
    let sphere_height = sqrt(max(1.0 - radius_squared, 0.0));
    let depth = clamp(input.center_depth - sphere_height * input.depth_radius, 0.0, 1.0);
    return vec4<f32>(depth);
}

@fragment
fn surface_thickness_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let radius_squared = dot(input.local_position, input.local_position);
    if radius_squared > 1.0 {
        discard;
    }
    let thickness = 2.0 * sqrt(max(1.0 - radius_squared, 0.0));
    return vec4<f32>(thickness, 0.0, 0.0, thickness);
}

@group(2) @binding(0) var fluid_depth: texture_2d<f32>;
@group(2) @binding(1) var fluid_blur: texture_2d<f32>;
@group(2) @binding(2) var fluid_thickness: texture_2d<f32>;

@vertex
fn fullscreen_vertex(@builtin(vertex_index) vertex: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex], 0.0, 1.0);
}

fn clamp_texel(position: vec2<i32>, dimensions: vec2<u32>) -> vec2<i32> {
    return clamp(position, vec2<i32>(0), vec2<i32>(dimensions) - vec2<i32>(1));
}

@fragment
fn horizontal_blur_fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let dimensions = textureDimensions(fluid_depth);
    let center_position = clamp_texel(vec2<i32>(position.xy), dimensions);
    let center = textureLoad(fluid_depth, center_position, 0).r;
    if center >= 0.9999 {
        return vec4<f32>(1.0);
    }

    var total = 0.0;
    var weight_total = 0.0;
    for (var offset = -4; offset <= 4; offset += 1) {
        let sample_position = clamp_texel(center_position + vec2<i32>(offset, 0), dimensions);
        let sample_depth = textureLoad(fluid_depth, sample_position, 0).r;
        if sample_depth >= 0.9999 {
            continue;
        }
        let spatial_weight = exp(-0.5 * f32(offset * offset) / 6.25);
        let range_weight = exp(-abs(sample_depth - center) * 220.0);
        let weight = spatial_weight * range_weight;
        total += sample_depth * weight;
        weight_total += weight;
    }
    let depth = total / max(weight_total, 1.0e-6);
    return vec4<f32>(depth);
}

@fragment
fn vertical_blur_fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let dimensions = textureDimensions(fluid_blur);
    let center_position = clamp_texel(vec2<i32>(position.xy), dimensions);
    let center = textureLoad(fluid_blur, center_position, 0).r;
    if center >= 0.9999 {
        return vec4<f32>(1.0);
    }

    var total = 0.0;
    var weight_total = 0.0;
    for (var offset = -4; offset <= 4; offset += 1) {
        let sample_position = clamp_texel(center_position + vec2<i32>(0, offset), dimensions);
        let sample_depth = textureLoad(fluid_blur, sample_position, 0).r;
        if sample_depth >= 0.9999 {
            continue;
        }
        let spatial_weight = exp(-0.5 * f32(offset * offset) / 6.25);
        let range_weight = exp(-abs(sample_depth - center) * 220.0);
        let weight = spatial_weight * range_weight;
        total += sample_depth * weight;
        weight_total += weight;
    }
    let depth = total / max(weight_total, 1.0e-6);
    return vec4<f32>(depth);
}

@fragment
fn fluid_composite_fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let dimensions = textureDimensions(fluid_blur);
    let texel = clamp_texel(vec2<i32>(position.xy), dimensions);
    let depth = textureLoad(fluid_blur, texel, 0).r;
    let uv = position.xy / vec2<f32>(dimensions);
    let background = mix(
        vec3<f32>(0.004, 0.010, 0.024),
        vec3<f32>(0.012, 0.028, 0.055),
        clamp(uv.y, 0.0, 1.0),
    );
    if depth >= 0.9999 {
        return vec4<f32>(background, 1.0);
    }

    let left = textureLoad(fluid_blur, clamp_texel(texel + vec2<i32>(-1, 0), dimensions), 0).r;
    let right = textureLoad(fluid_blur, clamp_texel(texel + vec2<i32>(1, 0), dimensions), 0).r;
    let up = textureLoad(fluid_blur, clamp_texel(texel + vec2<i32>(0, -1), dimensions), 0).r;
    let down = textureLoad(fluid_blur, clamp_texel(texel + vec2<i32>(0, 1), dimensions), 0).r;
    let normal = normalize(vec3<f32>(
        (left - right) * f32(dimensions.x) * 0.55,
        (up - down) * f32(dimensions.y) * 0.55,
        1.0,
    ));

    let thickness = textureLoad(fluid_thickness, texel, 0).r;
    let opacity = 1.0 - exp(-thickness * 0.055);
    let light = normalize(vec3<f32>(-0.38, 0.72, 0.58));
    let diffuse = 0.28 + 0.72 * max(dot(normal, light), 0.0);
    let fresnel = 0.025 + 0.975 * pow(1.0 - max(normal.z, 0.0), 5.0);
    let reflected_light = reflect(-light, normal);
    let specular = pow(max(reflected_light.z, 0.0), 72.0);
    let shallow = vec3<f32>(0.08, 0.66, 0.86);
    let deep = vec3<f32>(0.006, 0.14, 0.34);
    let body = mix(shallow, deep, clamp(opacity, 0.0, 1.0)) * diffuse;
    let reflection = mix(vec3<f32>(0.12, 0.26, 0.42), vec3<f32>(0.62, 0.88, 1.0), fresnel);
    let water = mix(background, body, 0.45 + 0.50 * opacity)
        + reflection * fresnel * 0.38
        + vec3<f32>(specular * 0.85);
    return vec4<f32>(water, 1.0);
}
