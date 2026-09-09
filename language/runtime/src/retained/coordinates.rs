// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Depth–phase–index coordinates; floating-point geometry is an explicit observation.
//!
//! The exact storage is q = |z|², a rational phase ray, and an integer index.
//! This represents X = ln(q)/2 and (Y,Z) = (k+1) ray / |ray| without rounding
//! logarithms or square roots. A point is one sample, not a replacement for
//! the retained program. Multi-index states require an explicit index direction.

use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};
use serde_json::{Value, json};

use super::{Depth, Scalar, ScalarData};
use crate::core::{NativeScalar, NativeState, Rational, rational_text};

/// An exact sample with depth, phase, and a retained integer index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Point {
    scalar: Scalar,
    index: BigUint,
}

impl Point {
    /// Place a native scalar at an exact nonnegative sample index.
    #[must_use]
    pub const fn new(scalar: Scalar, index: BigUint) -> Self {
        Self { scalar, index }
    }

    /// Read the exact sample index, independently of phase wrapping.
    #[must_use]
    pub const fn index(&self) -> &BigUint {
        &self.index
    }

    /// Read the native scalar, including any defined zero-boundary phase.
    #[must_use]
    pub const fn scalar(&self) -> &Scalar {
        &self.scalar
    }

    /// Multiply the sample without advancing its index.
    #[must_use]
    pub fn multiply(&self, factor: &Scalar) -> Self {
        Self::new(self.scalar.multiply(factor), self.index.clone())
    }

    /// Double depth and phase without changing the sample index.
    #[must_use]
    pub fn square(&self) -> Self {
        self.multiply(&self.scalar)
    }

    /// Reverse depth and phase at the same sample index.
    ///
    /// # Errors
    /// Exact zero has no reciprocal.
    pub fn reciprocal(&self) -> Result<Self, String> {
        if self.scalar.depth().is_zero_boundary() {
            return Err("the zero boundary has no multiplicative reciprocal".into());
        }
        let direction = self.scalar.direction().map(|ray| NativeScalar {
            real: ray.real.clone(),
            imag: -&ray.imag,
        });
        Ok(Self::new(
            Scalar::from_coordinates(
                Depth {
                    squared_magnitude: self.scalar.depth().squared_magnitude().recip(),
                },
                direction,
            )?,
            self.index.clone(),
        ))
    }

    /// Advance the retained index without changing depth or phase.
    #[must_use]
    pub fn advance(&self, steps: &BigUint) -> Self {
        Self::new(self.scalar.clone(), &self.index + steps)
    }

    /// Observe exact Cartesian coordinates as rational, logarithmic, and radical expressions.
    #[must_use]
    pub fn vector(&self) -> Value {
        let transverse = self.scalar.direction().map(|ray| {
            let norm_squared = &ray.real * &ray.real + &ray.imag * &ray.imag;
            let radius = Rational::from_integer((&self.index + BigUint::one()).into());
            [
                radical(&radius * &ray.real, &norm_squared),
                radical(radius * &ray.imag, &norm_squared),
            ]
        });
        json!([
            self.scalar.depth().to_data(),
            transverse.as_ref().map(|yz| &yz[0]),
            transverse.as_ref().map(|yz| &yz[1])
        ])
    }

    /// Serialize exact storage and its recomputable three-coordinate observation.
    #[must_use]
    pub fn to_data(&self) -> Value {
        json!({"model": "depth-phase-index", "index": self.index.to_string(),
            "scalar": self.scalar.to_data(), "vector": self.vector()})
    }

    /// Restore and validate an exact point, rejecting altered observations.
    ///
    /// # Errors
    /// Rejects malformed storage, invalid scalar coordinates, and noncanonical observations.
    pub fn from_data(value: &Value) -> Result<Self, String> {
        let index = value["index"]
            .as_str()
            .ok_or("point index must be an integer string")?
            .parse::<BigUint>()
            .map_err(|error| error.to_string())?;
        let scalar: ScalarData =
            serde_json::from_value(value["scalar"].clone()).map_err(|error| error.to_string())?;
        let point = Self::new(Scalar::from_data(&scalar)?, index);
        if point.to_data() != *value {
            return Err("point storage or its coordinates are noncanonical".into());
        }
        Ok(point)
    }

    /// Numerically observe this point without changing exact storage.
    ///
    /// # Errors
    /// Rejects an undefined boundary phase or a radius outside finite f64 range.
    pub fn to_f64(&self) -> Result<[f64; 3], String> {
        let ray = self
            .scalar
            .direction()
            .ok_or("zero has no defined phase; retain a boundary ray to locate it")?;
        let radius = (&self.index + BigUint::one())
            .to_f64()
            .filter(|r| r.is_finite())
            .ok_or("index radius exceeds f64 range")?;
        // Normalize by the larger component before converting: a rational ray
        // can have an arbitrarily large slope without having an infinite angle.
        let unit = unit_ray(ray)?;
        Ok([
            natural_depth(self.scalar.depth()),
            radius * unit[0],
            radius * unit[1],
        ])
    }

    /// Observe the derived compact sphere, deliberately discarding index radius.
    ///
    /// # Errors
    /// Rejects an undefined boundary phase.
    pub fn compact_f64(&self) -> Result<[f64; 3], String> {
        let ray = self
            .scalar
            .direction()
            .ok_or("compact view requires a defined phase")?;
        let unit = unit_ray(ray)?;
        let half_depth = natural_depth(self.scalar.depth()) / 2.0;
        // Stable sech: cosh would overflow for deep but valid inputs.
        let inward = (-half_depth.abs()).exp();
        let transverse = 2.0 * inward / (1.0 + inward * inward);
        Ok([
            half_depth.tanh(),
            transverse * unit[0],
            transverse * unit[1],
        ])
    }
}

// factor / sqrt(norm_squared); simplify only exact rational roots.
fn radical(factor: Rational, norm_squared: &Rational) -> Value {
    if factor.is_zero() {
        return json!({"kind":"finite", "value":"0"});
    }
    if let Some(norm) = super::scalar::exact_sqrt(norm_squared) {
        return json!({"kind":"finite", "value":rational_text(&(factor / norm))});
    }
    json!({"kind":"radical", "factor":rational_text(&factor),
        "radicand":rational_text(&norm_squared.recip())})
}

/// Observe two amplitudes through their derived quadratic balance/interference camera.
///
/// Returns (|b|²-|a|², 2 Re(conj(a)b), 2 Im(conj(a)b)). Its squared
/// length equals (|a|²+|b|²)². Global phase and both indices are discarded.
#[must_use]
pub fn quadratic_pair(left: &Point, right: &Point) -> [Rational; 3] {
    let a = left.scalar().project();
    let b = right.scalar().project();
    let two = Rational::from_integer(2.into());
    [
        right.scalar().depth().squared_magnitude() - left.scalar().depth().squared_magnitude(),
        &two * (&a.real * &b.real + &a.imag * &b.imag),
        two * (&a.real * &b.imag - &a.imag * &b.real),
    ]
}

fn unit_ray(ray: &NativeScalar) -> Result<[f64; 2], String> {
    use num_traits::Signed;
    let scale = ray.real.abs().max(ray.imag.abs());
    let real = (&ray.real / &scale)
        .to_f64()
        .ok_or("cannot evaluate phase ray")?;
    let imag = (&ray.imag / scale)
        .to_f64()
        .ok_or("cannot evaluate phase ray")?;
    let norm = real.hypot(imag);
    Ok([real / norm, imag / norm])
}

/// Numerically evaluate natural-log depth, including the explicit negative-infinity boundary.
#[must_use]
pub fn natural_depth(depth: &Depth) -> f64 {
    if depth.is_zero_boundary() {
        return f64::NEG_INFINITY;
    }
    let q = depth.squared_magnitude();
    // ln1p avoids subtracting large nearly equal integer logarithms near q=1.
    let offset = q - Rational::one();
    if let Some(offset) = offset.to_f64().filter(|v| v.abs() < 0.5) {
        return offset.ln_1p() / 2.0;
    }
    if let Some(q) = q.to_f64().filter(|q| q.is_finite() && *q > 0.0) {
        return q.ln() / 2.0;
    }
    (log_integer(q.numer().magnitude()) - log_integer(q.denom().magnitude())) / 2.0
}

#[expect(
    clippy::cast_precision_loss,
    reason = "f64 observation explicitly rounds logarithmic depth"
)]
fn log_integer(value: &BigUint) -> f64 {
    // Retain the leading 53 bits (the f64 significand); never convert a huge
    // integer directly to infinity before taking its logarithm.
    let shift = value.bits().saturating_sub(53);
    let leading = (value >> shift).to_f64().expect("53-bit integer fits f64");
    leading.ln() + (shift as f64) * std::f64::consts::LN_2
}

/// Locate projected terms using one named index direction as transverse radius.
#[must_use]
pub fn projection(state: &NativeState, direction: u64) -> Value {
    json!({"model":"depth-phase-index", "index_direction":direction,
        "terms":state.0.iter().map(|(index, value)| {
            json!({"index":index.to_data(),
                "point":Point::new(Scalar::from_classical(value), index.depth(direction)).to_data()})
        }).collect::<Vec<_>>()})
}
