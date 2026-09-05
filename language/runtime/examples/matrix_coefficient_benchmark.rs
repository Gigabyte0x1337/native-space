// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

use std::hint::black_box;
use std::time::{Duration, Instant};

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const SIZES: [usize; 4] = [64, 128, 256, 512];
const ROUNDS: usize = 7;
const TARGET_ROUND: Duration = Duration::from_millis(200);
const MAX_EXECUTIONS_PER_ROUND: usize = 4_096;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("size,dense_us,contracted_us,speedup,max_error");
    for size in SIZES {
        let u = values(size, 17, 3);
        let v = values(size, 19, 5);
        let x = values(size, 23, 7);
        let y = values(size, 29, 11);
        let left = outer_product(&u, &v, size);
        let right = outer_product(&x, &y, size);
        let mut dense_output = vec![0.0; size * size];
        let mut contracted_output = vec![0.0; size * size];

        dense_multiply(&left, &right, &mut dense_output, size);
        coefficient_contract(&u, &v, &x, &y, &mut contracted_output, size);
        let max_error = maximum_error(&dense_output, &contracted_output);
        if max_error > 1.0e-9 {
            return Err(format!("size {size} exceeded the numerical error bound").into());
        }

        let dense = measure(|| {
            dense_multiply(
                black_box(&left),
                black_box(&right),
                black_box(&mut dense_output),
                size,
            );
        });
        let contracted = measure(|| {
            coefficient_contract(
                black_box(&u),
                black_box(&v),
                black_box(&x),
                black_box(&y),
                black_box(&mut contracted_output),
                size,
            );
        });

        println!(
            "{size},{:.3},{:.3},{:.2},{max_error:.3e}",
            micros(dense),
            micros(contracted),
            dense.as_secs_f64() / contracted.as_secs_f64()
        );
    }
    Ok(())
}

fn values(size: usize, modulus: usize, offset: usize) -> Vec<f64> {
    let denominator = f64::from(u32::try_from(modulus).expect("benchmark modulus fits u32"));
    (0..size)
        .map(|index| {
            let numerator = u32::try_from((index * offset % modulus) + 1)
                .expect("benchmark numerator fits u32");
            f64::from(numerator) / denominator
        })
        .collect()
}

fn outer_product(left: &[f64], right: &[f64], size: usize) -> Vec<f64> {
    let mut output = vec![0.0; size * size];
    for row in 0..size {
        for column in 0..size {
            output[row * size + column] = left[row] * right[column];
        }
    }
    output
}

#[inline(never)]
fn dense_multiply(left: &[f64], right: &[f64], output: &mut [f64], size: usize) {
    output.fill(0.0);
    for row in 0..size {
        for inner in 0..size {
            let coefficient = left[row * size + inner];
            for column in 0..size {
                output[row * size + column] += coefficient * right[inner * size + column];
            }
        }
    }
    black_box(output);
}

#[inline(never)]
fn coefficient_contract(
    u: &[f64],
    v: &[f64],
    x: &[f64],
    y: &[f64],
    output: &mut [f64],
    size: usize,
) {
    let coefficient = v
        .iter()
        .zip(x)
        .fold(0.0, |sum, (left, right)| sum + left * right);
    for row in 0..size {
        let row_coefficient = u[row] * coefficient;
        for column in 0..size {
            output[row * size + column] = row_coefficient * y[column];
        }
    }
    black_box(output);
}

fn maximum_error(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f64::max)
}

fn measure(mut operation: impl FnMut()) -> Duration {
    operation();
    let probe_started = Instant::now();
    operation();
    let probe = probe_started.elapsed();
    let executions = executions_for(probe);
    let mut samples = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        let started = Instant::now();
        for _ in 0..executions {
            operation();
        }
        let divisor = u32::try_from(executions).expect("benchmark execution count fits u32");
        samples.push(started.elapsed() / divisor);
    }
    samples.sort_unstable();
    samples[ROUNDS / 2]
}

fn executions_for(probe: Duration) -> usize {
    if probe.is_zero() {
        return MAX_EXECUTIONS_PER_ROUND;
    }
    let executions = TARGET_ROUND
        .as_nanos()
        .div_ceil(probe.as_nanos())
        .clamp(1, MAX_EXECUTIONS_PER_ROUND as u128);
    usize::try_from(executions).expect("bounded benchmark execution count fits usize")
}

fn micros(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000_000.0
}
