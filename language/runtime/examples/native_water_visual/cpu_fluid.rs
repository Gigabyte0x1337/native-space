// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Evaluates the indexed WCSPH state transition on one CPU thread.
//!
//! Every substep has three explicit phases: rebuild cell membership from the
//! complete input state, derive density and pressure from that same state, and
//! write integration results to a separate output state. This preserves the
//! application invariant that no newly produced particle can affect another
//! particle until the next substep.
//!
//! The `cpu-simd` feature substitutes `glam::Vec3A` for the scalar three-value
//! vector while instantiating the same solver source. It is diagnostic rather
//! than the default: neighbor discovery is a data-dependent linked-list walk,
//! so SIMD across x/y/z can lose to scalar arithmetic at larger particle
//! counts. This comparison avoids unsafe gather code and semantic drift.

use num_traits::ToPrimitive as _;
#[cfg(not(feature = "cpu-simd"))]
use std::ops::{Add, AddAssign, Mul, Sub};

#[cfg(feature = "cpu-simd")]
type Vector3 = glam::Vec3A;

#[cfg(not(feature = "cpu-simd"))]
#[derive(Clone, Copy, Debug)]
struct Vector3([f32; 3]);

#[cfg(not(feature = "cpu-simd"))]
impl Vector3 {
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self([x, y, z])
    }

    fn length_squared(self) -> f32 {
        self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2]
    }

    fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    const fn to_array(self) -> [f32; 3] {
        self.0
    }
}

#[cfg(not(feature = "cpu-simd"))]
impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        )
    }
}

#[cfg(not(feature = "cpu-simd"))]
impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[cfg(not(feature = "cpu-simd"))]
impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        )
    }
}

#[cfg(not(feature = "cpu-simd"))]
impl Mul<f32> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.0[0] * rhs, self.0[1] * rhs, self.0[2] * rhs)
    }
}

use super::{ParticleState, SimParameters};

const EMPTY_LINK: usize = usize::MAX;
const MAX_PARTICLE_SPEED: f32 = 6.0;

#[derive(Debug)]
pub(super) struct CpuFluid {
    states: [Vec<ParticleState>; 2],
    density_pressure: Vec<[f32; 2]>,
    cell_heads: Vec<usize>,
    next_particle: Vec<usize>,
}

impl CpuFluid {
    pub(super) fn new(initial: &[ParticleState], parameters: &SimParameters) -> Self {
        let mut fluid = Self {
            states: [initial.to_vec(), initial.to_vec()],
            density_pressure: vec![[0.0; 2]; initial.len()],
            cell_heads: vec![EMPTY_LINK; grid_cell_count(parameters)],
            next_particle: vec![EMPTY_LINK; initial.len()],
        };
        fluid.validate(parameters);
        fluid.rebuild_index(parameters);
        fluid
    }

    pub(super) fn reset(&mut self, initial: &[ParticleState], parameters: &SimParameters) {
        self.states[0].clear();
        self.states[0].extend_from_slice(initial);
        self.states[1].clear();
        self.states[1].extend_from_slice(initial);
        self.density_pressure.resize(initial.len(), [0.0; 2]);
        self.next_particle.resize(initial.len(), EMPTY_LINK);
        self.cell_heads
            .resize(grid_cell_count(parameters), EMPTY_LINK);
        self.validate(parameters);
        self.rebuild_index(parameters);
    }

    pub(super) fn state(&self) -> &[ParticleState] {
        &self.states[0]
    }

    pub(super) fn step(&mut self, parameters: &SimParameters) {
        self.validate(parameters);
        self.rebuild_index(parameters);
        self.compute_density_pressure(parameters);
        self.integrate(parameters);
        self.states.swap(0, 1);
    }

    fn validate(&self, parameters: &SimParameters) {
        let particle_count = usize::try_from(parameters.particle_count)
            .expect("u32 particle count always fits usize on supported targets");
        assert_eq!(self.states[0].len(), particle_count);
        assert_eq!(self.states[1].len(), particle_count);
        assert_eq!(self.density_pressure.len(), particle_count);
        assert_eq!(self.next_particle.len(), particle_count);
        assert_eq!(self.cell_heads.len(), grid_cell_count(parameters));
    }

    fn rebuild_index(&mut self, parameters: &SimParameters) {
        self.cell_heads.fill(EMPTY_LINK);
        for (particle, state) in self.states[0].iter().enumerate() {
            let cell = point_cell(state.position, parameters);
            let cell = cell_index(cell, parameters.grid);
            self.next_particle[particle] = self.cell_heads[cell];
            self.cell_heads[cell] = particle;
        }
    }

    fn compute_density_pressure(&mut self, parameters: &SimParameters) {
        let input = &self.states[0];
        let h = parameters.smoothing_radius;
        let h_squared = h * h;
        let poly6 = 315.0 / (64.0 * std::f32::consts::PI * h.powi(9));
        let self_density = parameters.mass * poly6 * h_squared.powi(3);

        for particle in 0..input.len() {
            let position = position3(input[particle]);
            let mut density = self_density;
            visit_neighbors(
                particle,
                input[particle].position,
                input,
                &self.cell_heads,
                &self.next_particle,
                parameters,
                |other| {
                    let displacement = position - position3(input[other]);
                    let distance_squared = displacement.length_squared();
                    if distance_squared < h_squared {
                        density += parameters.mass * poly6 * (h_squared - distance_squared).powi(3);
                    }
                },
            );
            density = density.max(parameters.rest_density * 0.05);
            let pressure =
                parameters.pressure_stiffness * (density - parameters.rest_density).max(0.0);
            self.density_pressure[particle] = [density, pressure];
        }
    }

    fn integrate(&mut self, parameters: &SimParameters) {
        let (input_states, output_states) = self.states.split_at_mut(1);
        let input = &input_states[0];
        let output = &mut output_states[0];
        let density_pressure = &self.density_pressure;
        let h = parameters.smoothing_radius;
        let h_squared = h * h;
        let spiky_gradient = -45.0 / (std::f32::consts::PI * h.powi(6));
        let viscosity_laplacian = 45.0 / (std::f32::consts::PI * h.powi(6));

        for particle in 0..input.len() {
            let state = input[particle];
            let position = position3(state);
            let own_density_pressure = density_pressure[particle];
            let mut acceleration = Vector3::new(0.0, parameters.gravity, 0.0);
            visit_neighbors(
                particle,
                state.position,
                input,
                &self.cell_heads,
                &self.next_particle,
                parameters,
                |other| {
                    let other_state = input[other];
                    let displacement = position - position3(other_state);
                    let distance_squared = displacement.length_squared();
                    if distance_squared <= 1.0e-8 || distance_squared >= h_squared {
                        return;
                    }

                    let distance = distance_squared.sqrt();
                    let remainder = h - distance;
                    let other_density_pressure = density_pressure[other];
                    let pressure_gain =
                        -parameters.mass * spiky_gradient * remainder * remainder / distance;
                    let pressure_scale = pressure_gain
                        * (own_density_pressure[1]
                            / (own_density_pressure[0] * own_density_pressure[0])
                            + other_density_pressure[1]
                                / (other_density_pressure[0] * other_density_pressure[0]));
                    acceleration += displacement * pressure_scale;

                    let viscosity_gain =
                        parameters.viscosity * parameters.mass * viscosity_laplacian * remainder
                            / other_density_pressure[0];
                    let velocity_difference = velocity3(other_state) - velocity3(state);
                    acceleration += velocity_difference * viscosity_gain;
                },
            );

            let velocity = limit_velocity(velocity3(state) + acceleration * parameters.delta_time);
            let position = position + velocity * parameters.delta_time;
            output[particle] = constrained_particle(position, velocity, parameters);
        }
    }
}

fn visit_neighbors(
    particle: usize,
    position: [f32; 4],
    states: &[ParticleState],
    cell_heads: &[usize],
    next_particle: &[usize],
    parameters: &SimParameters,
    mut visit: impl FnMut(usize),
) {
    let own_cell = point_cell(position, parameters);
    for z_offset in -1_i32..=1 {
        for y_offset in -1_i32..=1 {
            for x_offset in -1_i32..=1 {
                let neighbor = [
                    i32::try_from(own_cell[0]).expect("grid coordinate fits i32") + x_offset,
                    i32::try_from(own_cell[1]).expect("grid coordinate fits i32") + y_offset,
                    i32::try_from(own_cell[2]).expect("grid coordinate fits i32") + z_offset,
                ];
                if neighbor.iter().enumerate().any(|(axis, value)| {
                    *value < 0
                        || *value
                            >= parameters.grid[axis]
                                .to_i32()
                                .expect("grid dimension fits i32")
                }) {
                    continue;
                }
                let neighbor = neighbor.map(|value| {
                    u32::try_from(value).expect("validated neighbor coordinate is nonnegative")
                });
                let mut other = cell_heads[cell_index(neighbor, parameters.grid)];
                while other != EMPTY_LINK {
                    assert!(
                        other < states.len(),
                        "cell link must reference one input particle"
                    );
                    if other != particle {
                        visit(other);
                    }
                    other = next_particle[other];
                }
            }
        }
    }
}

fn point_cell(position: [f32; 4], parameters: &SimParameters) -> [u32; 3] {
    std::array::from_fn(|axis| {
        ((position[axis] + parameters.bounds[axis]) / parameters.smoothing_radius)
            .floor()
            .clamp(
                0.0,
                parameters.grid[axis]
                    .saturating_sub(1)
                    .to_f32()
                    .expect("grid dimension has a finite f32 representation"),
            )
            .to_u32()
            .expect("finite clamped cell coordinate fits u32")
    })
}

fn cell_index(cell: [u32; 3], grid: [u32; 4]) -> usize {
    let index = cell[0] + grid[0] * (cell[1] + grid[1] * cell[2]);
    usize::try_from(index).expect("u32 grid index fits usize on supported targets")
}

fn grid_cell_count(parameters: &SimParameters) -> usize {
    usize::try_from(parameters.grid[3]).expect("u32 grid count fits usize on supported targets")
}

fn position3(state: ParticleState) -> Vector3 {
    Vector3::new(state.position[0], state.position[1], state.position[2])
}

fn velocity3(state: ParticleState) -> Vector3 {
    Vector3::new(state.velocity[0], state.velocity[1], state.velocity[2])
}

fn limit_velocity(velocity: Vector3) -> Vector3 {
    let speed = velocity.length();
    if speed > MAX_PARTICLE_SPEED {
        velocity * (MAX_PARTICLE_SPEED / speed)
    } else {
        velocity
    }
}

fn constrained_particle(
    position: Vector3,
    velocity: Vector3,
    parameters: &SimParameters,
) -> ParticleState {
    let mut position = position.to_array();
    let mut velocity = velocity.to_array();
    for axis in 0..3 {
        let bound = parameters.bounds[axis];
        if position[axis] < -bound {
            position[axis] = -bound;
            velocity[axis] = velocity[axis].abs() * parameters.bounds[3];
        } else if position[axis] > bound {
            position[axis] = bound;
            velocity[axis] = -velocity[axis].abs() * parameters.bounds[3];
        }
    }
    ParticleState {
        position: [position[0], position[1], position[2], 1.0],
        velocity: [velocity[0], velocity[1], velocity[2], 0.0],
    }
}
