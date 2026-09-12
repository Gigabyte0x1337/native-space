// SPDX-License-Identifier: AGPL-3.0-or-later
//! Optional frame policy. It never changes the raw authoritative triple.
use crate::algebra::{Rational, State, Transform};
use num_traits::{One, Signed, Zero};
pub fn balanced_frame(p: &State) -> Result<Transform, String> {
    let largest = p
        .coordinates()
        .into_iter()
        .map(|x| x.abs())
        .max()
        .expect("three coordinates");
    let scale = if largest.is_zero() {
        Rational::one()
    } else {
        largest.recip()
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
    pub alpha: f64,
    pub limit: f64,
    pub predicted: f64,
    pub error: f64,
}
pub fn probe(samples: [f64; 4]) -> Result<ScaleProbe, String> {
    if !samples.iter().all(|x| x.is_finite()) {
        return Err("samples must be finite".into());
    }
    let d = samples[1] - samples[0];
    let r = (samples[2] - samples[1]) / d;
    if !r.is_finite() || r <= 0.0 || r >= 1.0 {
        return Err("no positive decaying single-scale model".into());
    }
    let limit = (samples[1] - r * samples[0]) / (1.0 - r);
    let predicted = samples[2] + r * (samples[2] - samples[1]);
    if !limit.is_finite() || !predicted.is_finite() {
        return Err("scale inference overflow".into());
    }
    Ok(ScaleProbe {
        alpha: -r.log2(),
        limit,
        predicted,
        error: (samples[3] - predicted).abs(),
    })
}
