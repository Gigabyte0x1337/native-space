// SPDX-License-Identifier: AGPL-3.0-or-later
//! Optional frame policy. It never changes the raw authoritative triple.
use crate::algebra::{Rational, State, Transform, pow2};
use num_traits::{One, Signed, Zero};
pub mod scale;
pub fn balanced_frame(p: &State) -> Result<Transform, String> {
    balanced_frame_with(p, Balance::Exact)
}
/// Explicit policy for local scaling; authoritative raw state never changes.
#[derive(Clone, Copy, Debug)]
pub enum Balance {
    /// Put the largest absolute local coordinate exactly at one.
    Exact,
    /// Use a dyadic scale, keeping the largest local magnitude in (1/2, 1].
    PowerOfTwo,
}
/// Chooses a reversible local scale without evaluating a floating-point logarithm.
///
/// # Errors
/// Returns an error if the required dyadic exponent is outside i32.
pub fn balanced_frame_with(p: &State, policy: Balance) -> Result<Transform, String> {
    let largest = p
        .coordinates()
        .into_iter()
        .map(|x| x.abs())
        .max()
        .expect("three coordinates");
    let scale = if largest.is_zero() {
        Rational::one()
    } else {
        match policy {
            Balance::Exact => largest.recip(),
            Balance::PowerOfTwo => {
                let bits_n = i64::try_from(largest.numer().bits())
                    .map_err(|_| "scale exponent too large")?;
                let bits_d = i64::try_from(largest.denom().bits())
                    .map_err(|_| "scale exponent too large")?;
                let mut exponent =
                    i32::try_from(bits_n - bits_d).map_err(|_| "scale exponent too large")?;
                // Bit lengths bracket log2 within one integer. Compare exactly
                // to select the ceiling, including negative powers and exact powers.
                if largest > pow2(exponent) {
                    exponent = exponent.checked_add(1).ok_or("scale exponent too large")?;
                }
                pow2(exponent.checked_neg().ok_or("scale exponent too large")?)
            }
        }
    };
    Transform::new(std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            if i == j {
                scale.clone()
            } else {
                Rational::zero()
            }
        })
    }))
}
/// A four-sample numerical hypothesis, including an unseen-scale residual.
/// Agreement does not establish a convergence theorem or authorize exact rewrites.
#[derive(Clone, Debug)]
pub struct ScaleProbe {
    pub ratio: f64,
    pub sign: i8,
    pub alpha: f64,
    pub limit: f64,
    pub predicted: f64,
    pub error: f64,
    pub residual: [f64; 4],
}
pub fn probe(samples: [f64; 4]) -> Result<ScaleProbe, String> {
    if !samples.iter().all(|x| x.is_finite()) {
        return Err("samples must be finite".into());
    }
    let d = samples[1] - samples[0];
    let r = (samples[2] - samples[1]) / d;
    if !r.is_finite() || r == 0.0 || r.abs() >= 1.0 {
        return Err("no real decaying single-scale model".into());
    }
    let limit = (samples[1] - r * samples[0]) / (1.0 - r);
    let predicted = samples[2] + r * (samples[2] - samples[1]);
    let residual =
        std::array::from_fn(|i| samples[i] - (limit + (samples[0] - limit) * r.powi(i as i32)));
    let error = (samples[3] - predicted).abs();
    if !limit.is_finite()
        || !predicted.is_finite()
        || !error.is_finite()
        || !residual.iter().all(|x| x.is_finite())
    {
        return Err("scale inference overflow".into());
    }
    Ok(ScaleProbe {
        ratio: r,
        sign: if r < 0.0 { -1 } else { 1 },
        alpha: -r.abs().log2(),
        limit,
        predicted,
        error,
        residual,
    })
}
