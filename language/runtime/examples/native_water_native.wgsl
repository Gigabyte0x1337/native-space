// SPDX-License-Identifier: AGPL-3.0-or-later

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

struct Pair {
    geometry: vec4<f32>,
    coefficients: vec4<f32>,
}

struct Adjacency {
    pair_index: u32,
    other_particle: u32,
    direction: i32,
    padding: u32,
}

@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var<storage, read> velocities: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> pairs: array<Pair>;
@group(0) @binding(3) var<storage, read> adjacency: array<Adjacency>;
@group(0) @binding(4) var<storage, read> offsets: array<u32>;
@group(0) @binding(5) var<storage, read_write> density_pressure: array<vec2<f32>>;
@group(0) @binding(6) var<storage, read_write> accelerations: array<vec4<f32>>;

@compute @workgroup_size(128)
fn density_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let particle = invocation.x;
    if particle >= parameters.particle_count {
        return;
    }

    var density = parameters.self_density;
    var position = offsets[particle];
    let end = offsets[particle + 1u];
    while position < end {
        density += pairs[adjacency[position].pair_index].geometry.w;
        position += 1u;
    }
    density = max(density, parameters.rest_density * 0.05);
    let pressure = parameters.pressure_stiffness * max(0.0, density - parameters.rest_density);
    density_pressure[particle] = vec2<f32>(density, pressure);
}

@compute @workgroup_size(128)
fn force_main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let particle = invocation.x;
    if particle >= parameters.particle_count {
        return;
    }

    let own_density_pressure = density_pressure[particle];
    let own_velocity = velocities[particle].xyz;
    var acceleration = vec3<f32>(0.0, parameters.gravity, 0.0);
    var position = offsets[particle];
    let end = offsets[particle + 1u];
    while position < end {
        let edge = adjacency[position];
        let pair = pairs[edge.pair_index];
        let other_density_pressure = density_pressure[edge.other_particle];
        let displacement = pair.geometry.xyz * f32(edge.direction);
        let pressure_scale = pair.coefficients.x * (
            own_density_pressure.y / (own_density_pressure.x * own_density_pressure.x)
            + other_density_pressure.y / (other_density_pressure.x * other_density_pressure.x)
        );
        acceleration += pressure_scale * displacement;

        let velocity_difference = velocities[edge.other_particle].xyz - own_velocity;
        acceleration += (
            pair.coefficients.y / other_density_pressure.x
        ) * velocity_difference;
        position += 1u;
    }
    accelerations[particle] = vec4<f32>(acceleration, 0.0);
}
