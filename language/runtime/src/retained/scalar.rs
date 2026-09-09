// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Exact scalar storage in multiplicative-depth and phase coordinates.

use num_traits::{Signed, Zero};
use serde::{Deserialize, Serialize};

use super::Depth;
use crate::core::{NativeScalar, Rational, rational, rational_text};

/// An exact scalar whose primary coordinates are depth and phase.
///
/// Squared magnitude is the argument of twice the natural-log depth. The ray
/// is rational and canonically scaled; it is not an approximate angle. Zero
/// may retain a boundary ray. Projection is exact over the Gaussian-rational domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scalar {
    depth: Depth,
    direction: Option<NativeScalar>,
}

impl Scalar {
    /// Lift an exact classical scalar into multiplicative coordinates.
    #[must_use]
    pub fn from_classical(value: &NativeScalar) -> Self {
        Self {
            depth: Depth::of(value),
            direction: ray(value),
        }
    }

    /// Construct a scalar from validated exact native coordinates.
    ///
    /// # Errors
    /// Rejects nonzero depth without a direction,
    /// a zero direction, or coordinates outside the exact rational scalar domain.
    pub fn from_coordinates(depth: Depth, direction: Option<NativeScalar>) -> Result<Self, String> {
        let direction = match direction {
            Some(value) => Some(ray(&value).ok_or("phase ray must be nonzero")?),
            None => None,
        };
        let result = Self { depth, direction };
        result.checked_projection()?;
        Ok(result)
    }

    /// Read the multiplicative-depth coordinate.
    #[must_use]
    pub fn depth(&self) -> &Depth {
        &self.depth
    }

    /// Read the exact phase ray, including explicitly supplied boundary provenance.
    #[must_use]
    pub fn direction(&self) -> Option<&NativeScalar> {
        self.direction.as_ref()
    }

    /// Project into exact classical additive coordinates.
    ///
    /// # Panics
    /// Panics only if an internal constructor violates the validated scalar domain.
    #[must_use]
    pub fn project(&self) -> NativeScalar {
        self.checked_projection()
            .expect("native scalar coordinates remain Gaussian-rational")
    }

    /// Multiply by adding depths and composing phase rays.
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        let direction = self
            .direction
            .as_ref()
            .zip(other.direction.as_ref())
            .and_then(|(left, right)| ray(&left.multiply(right)));
        Self {
            depth: self.depth.compose(&other.depth),
            direction,
        }
    }

    /// Add through the exact additive chart and lift the result back.
    ///
    /// General addition is not linear in logarithmic coordinates. Retaining
    /// cancellation operands belongs to the enclosing operation state.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self::from_classical(&self.project().add(&other.project()))
    }

    fn checked_projection(&self) -> Result<NativeScalar, String> {
        match (&self.direction, self.depth.is_zero_boundary()) {
            (_, true) => Ok(NativeScalar::zero()),
            (Some(direction), false) => {
                let ray_size =
                    &direction.real * &direction.real + &direction.imag * &direction.imag;
                let squared_scale = &self.depth.squared_magnitude / ray_size;
                let scale = exact_sqrt(&squared_scale)
                    .ok_or("native coordinates require irrational classical coefficients")?;
                Ok(NativeScalar {
                    real: &direction.real * &scale,
                    imag: &direction.imag * &scale,
                })
            }
            _ => Err("zero boundary and phase are inconsistent".into()),
        }
    }

    pub(super) fn to_data(&self) -> ScalarData {
        ScalarData {
            squared_magnitude: rational_text(&self.depth.squared_magnitude),
            direction: self.direction.as_ref().map(|direction| RayData {
                real: rational_text(&direction.real),
                imag: rational_text(&direction.imag),
            }),
        }
    }

    pub(crate) fn from_data(data: &ScalarData) -> Result<Self, String> {
        let squared_magnitude = rational(&data.squared_magnitude)?;
        if squared_magnitude.is_negative() {
            return Err("squared magnitude cannot be negative".into());
        }
        Self::from_coordinates(
            Depth { squared_magnitude },
            data.direction
                .as_ref()
                .map(|direction| NativeScalar::from_text(&direction.real, &direction.imag))
                .transpose()?,
        )
    }
}

fn ray(value: &NativeScalar) -> Option<NativeScalar> {
    if value.is_zero() {
        return None;
    }
    let scale = if value.real.is_zero() {
        value.imag.abs()
    } else {
        value.real.abs()
    };
    Some(NativeScalar {
        real: &value.real / &scale,
        imag: &value.imag / &scale,
    })
}

pub(super) fn exact_sqrt(value: &Rational) -> Option<Rational> {
    let numerator = value.numer().sqrt();
    let denominator = value.denom().sqrt();
    if &numerator * &numerator != *value.numer() || &denominator * &denominator != *value.denom() {
        return None;
    }
    Some(Rational::new(numerator, denominator))
}

/// Exact wire coordinates, validated before becoming a native scalar.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalarData {
    pub squared_magnitude: String,
    pub direction: Option<RayData>,
}

/// Rational coordinates of a phase ray, not a rounded angle.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RayData {
    pub real: String,
    pub imag: String,
}
