// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Synthesizes finite programs for the classical complex projection.
//!
//! This module deliberately operates after projection. It reads one finite
//! INDEX window from an exact native state, converts its coefficients to
//! classical `f64` complex coordinates, selects discrete frequency modes, and
//! verifies the reconstructed coordinates against an explicit absolute error
//! bound. Acceptance establishes camera-relative agreement only.

use num_traits::{One as _, ToPrimitive as _};
use serde_json::{Value, json};

use crate::core::{Diagnostic, LanguageError, NativeState};

/// Maximum window accepted by the direct finite transform.
///
/// The initial implementation uses a deterministic quadratic transform. This
/// bound prevents accidental unbounded work until a separately verified fast
/// transform replaces it.
const MAX_SAMPLES: usize = 4_096;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ComplexCoordinate {
    real: f64,
    imag: f64,
}

impl ComplexCoordinate {
    const ZERO: Self = Self {
        real: 0.0,
        imag: 0.0,
    };

    fn add(self, other: Self) -> Self {
        Self {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        }
    }

    fn multiply(self, other: Self) -> Self {
        Self {
            real: self.real.mul_add(other.real, -(self.imag * other.imag)),
            imag: self.real.mul_add(other.imag, self.imag * other.real),
        }
    }

    fn scale(self, factor: f64) -> Self {
        Self {
            real: self.real * factor,
            imag: self.imag * factor,
        }
    }

    fn magnitude_squared(self) -> f64 {
        self.real.mul_add(self.real, self.imag * self.imag)
    }

    fn distance(self, other: Self) -> f64 {
        (self.real - other.real).hypot(self.imag - other.imag)
    }

    fn is_finite(self) -> bool {
        self.real.is_finite() && self.imag.is_finite()
    }
}

/// One retained classical frequency coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrequencyMode {
    bin: usize,
    coefficient: ComplexCoordinate,
}

impl FrequencyMode {
    /// Return the finite frequency bin.
    #[must_use]
    pub const fn bin(&self) -> usize {
        self.bin
    }

    /// Return the classical real coefficient.
    #[must_use]
    pub const fn real(&self) -> f64 {
        self.coefficient.real
    }

    /// Return the classical imaginary coefficient.
    #[must_use]
    pub const fn imag(&self) -> f64 {
        self.coefficient.imag
    }
}

/// A verified finite classical-frequency replay program.
#[derive(Clone, Debug, PartialEq)]
pub struct FrequencyProgram {
    first_index: u64,
    sample_count: usize,
    modes: Vec<FrequencyMode>,
    maximum_error: f64,
    observed_error: f64,
}

impl FrequencyProgram {
    /// Return the first projected INDEX direction.
    #[must_use]
    pub const fn first_index(&self) -> u64 {
        self.first_index
    }

    /// Return the finite projected window length.
    #[must_use]
    pub const fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// Return the retained frequency modes.
    #[must_use]
    pub fn modes(&self) -> &[FrequencyMode] {
        &self.modes
    }

    /// Return the declared absolute reconstruction bound.
    #[must_use]
    pub const fn maximum_error(&self) -> f64 {
        self.maximum_error
    }

    /// Return the largest observed classical-coordinate error.
    #[must_use]
    pub const fn observed_error(&self) -> f64 {
        self.observed_error
    }

    /// Replay every projected coordinate in the finite window.
    #[must_use]
    pub fn reconstruct(&self) -> Vec<(f64, f64)> {
        reconstruct(self.sample_count, &self.modes)
            .into_iter()
            .map(|coordinate| (coordinate.real, coordinate.imag))
            .collect()
    }

    /// Return the stable report representation.
    #[must_use]
    pub fn to_data(&self) -> Value {
        json!({
            "schema": "native-space-classical-frequency-program",
            "version": 1,
            "camera": "classical-complex-f64",
            "first_index": self.first_index,
            "sample_count": self.sample_count,
            "mode_count": self.modes.len(),
            "maximum_absolute_error": self.maximum_error,
            "observed_maximum_error": self.observed_error,
            "verified": self.observed_error <= self.maximum_error,
            "modes": self.modes.iter().map(|mode| json!({
                "bin": mode.bin,
                "real": mode.coefficient.real,
                "imag": mode.coefficient.imag,
            })).collect::<Vec<_>>(),
        })
    }
}

/// Synthesize and verify one finite classical frequency program.
///
/// Modes are considered in descending projected power. The first prefix whose
/// replay satisfies `maximum_error` is retained. This is deterministic sparse
/// approximation, not a globally minimal mode search.
///
/// # Errors
///
/// Returns an `NSQ` diagnostic for an invalid window or error bound, an
/// incompatible native INDEX layout, a non-finite classical projection, or a
/// reconstruction that cannot meet the bound.
pub fn synthesize(
    state: &NativeState,
    first_index: u64,
    sample_count: usize,
    maximum_error: &str,
    source_name: &str,
) -> Result<FrequencyProgram, LanguageError> {
    let maximum_error = parse_maximum_error(maximum_error, source_name)?;
    let samples = project(state, first_index, sample_count, source_name)?;
    let coefficients = transform(&samples, source_name)?;
    let mut ranked = coefficients
        .into_iter()
        .enumerate()
        .map(|(bin, coefficient)| FrequencyMode { bin, coefficient })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .coefficient
            .magnitude_squared()
            .total_cmp(&left.coefficient.magnitude_squared())
            .then_with(|| left.bin.cmp(&right.bin))
    });

    let mut selected = Vec::new();
    let mut replay = vec![ComplexCoordinate::ZERO; sample_count];
    let mut observed_error = maximum_distance(&samples, &replay);
    for mode in ranked {
        if observed_error <= maximum_error {
            break;
        }
        add_mode(&mut replay, mode);
        selected.push(mode);
        observed_error = maximum_distance(&samples, &replay);
    }
    if !observed_error.is_finite() || observed_error > maximum_error {
        return Err(frequency_error(
            "NSQ005",
            format!(
                "classical frequency replay error {observed_error:e} exceeds the declared bound {maximum_error:e}"
            ),
            source_name,
        ));
    }
    selected.sort_by_key(|mode| mode.bin);
    let program = FrequencyProgram {
        first_index,
        sample_count,
        modes: selected,
        maximum_error,
        observed_error,
    };
    let verified_error = maximum_distance(&samples, &reconstruct(sample_count, &program.modes));
    if !verified_error.is_finite() || verified_error > maximum_error {
        return Err(frequency_error(
            "NSQ006",
            "final frequency replay verification failed",
            source_name,
        ));
    }
    Ok(FrequencyProgram {
        observed_error: verified_error,
        ..program
    })
}

fn parse_maximum_error(source: &str, source_name: &str) -> Result<f64, LanguageError> {
    source
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .ok_or_else(|| {
            frequency_error(
                "NSQ001",
                "maximum error must be one finite nonnegative classical number",
                source_name,
            )
        })
}

fn project(
    state: &NativeState,
    first_index: u64,
    sample_count: usize,
    source_name: &str,
) -> Result<Vec<ComplexCoordinate>, LanguageError> {
    if first_index == 0 {
        return Err(frequency_error(
            "NSQ002",
            "the first INDEX direction must be positive",
            source_name,
        ));
    }
    if sample_count == 0 || sample_count > MAX_SAMPLES {
        return Err(frequency_error(
            "NSQ002",
            format!("sample count must be from 1 through {MAX_SAMPLES}"),
            source_name,
        ));
    }
    let last_offset = u64::try_from(sample_count - 1).map_err(|_capacity_error| {
        frequency_error("NSQ002", "sample count exceeds u64", source_name)
    })?;
    let last_index = first_index.checked_add(last_offset).ok_or_else(|| {
        frequency_error(
            "NSQ002",
            "the projected INDEX window overflows",
            source_name,
        )
    })?;
    let mut samples = vec![ComplexCoordinate::ZERO; sample_count];
    for (index, scalar) in &state.0 {
        if index.0.len() != 1 {
            return Err(frequency_error(
                "NSQ003",
                "frequency synthesis expects one INDEX direction per sample",
                source_name,
            ));
        }
        let (&direction, depth) = index
            .0
            .first_key_value()
            .expect("one checked INDEX direction exists");
        if !depth.is_one() || direction < first_index || direction > last_index {
            return Err(frequency_error(
                "NSQ003",
                format!(
                    "INDEX direction {direction} is outside the requested one-depth sample window"
                ),
                source_name,
            ));
        }
        let offset = direction
            .checked_sub(first_index)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| frequency_error("NSQ002", "sample offset exceeds usize", source_name))?;
        let coordinate = ComplexCoordinate {
            real: scalar.real.to_f64().ok_or_else(|| {
                frequency_error(
                    "NSQ004",
                    format!("INDEX direction {direction} cannot be projected to finite f64"),
                    source_name,
                )
            })?,
            imag: scalar.imag.to_f64().ok_or_else(|| {
                frequency_error(
                    "NSQ004",
                    format!("INDEX direction {direction} cannot be projected to finite f64"),
                    source_name,
                )
            })?,
        };
        if !coordinate.is_finite() {
            return Err(frequency_error(
                "NSQ004",
                format!("INDEX direction {direction} has a non-finite classical projection"),
                source_name,
            ));
        }
        samples[offset] = coordinate;
    }
    Ok(samples)
}

fn transform(
    samples: &[ComplexCoordinate],
    source_name: &str,
) -> Result<Vec<ComplexCoordinate>, LanguageError> {
    let count = bounded_f64(samples.len(), source_name)?;
    let positions = (0..samples.len())
        .map(|position| bounded_f64(position, source_name))
        .collect::<Result<Vec<_>, _>>()?;
    (0..samples.len())
        .map(|bin| {
            let bin = bounded_f64(bin, source_name)?;
            let coefficient = samples
                .iter()
                .zip(&positions)
                .fold(ComplexCoordinate::ZERO, |sum, (sample, position)| {
                    let angle = -std::f64::consts::TAU * bin * position / count;
                    sum.add(sample.multiply(unit(angle)))
                })
                .scale(count.recip());
            coefficient
                .is_finite()
                .then_some(coefficient)
                .ok_or_else(|| {
                    frequency_error(
                        "NSQ004",
                        "the classical frequency transform produced a non-finite coordinate",
                        source_name,
                    )
                })
        })
        .collect()
}

fn reconstruct(sample_count: usize, modes: &[FrequencyMode]) -> Vec<ComplexCoordinate> {
    let count = valid_sample_f64(sample_count);
    (0..sample_count)
        .map(|position| {
            let position = valid_sample_f64(position);
            modes.iter().fold(ComplexCoordinate::ZERO, |sum, mode| {
                let bin = valid_sample_f64(mode.bin);
                let angle = std::f64::consts::TAU * bin * position / count;
                sum.add(mode.coefficient.multiply(unit(angle)))
            })
        })
        .collect()
}

fn add_mode(replay: &mut [ComplexCoordinate], mode: FrequencyMode) {
    let count = valid_sample_f64(replay.len());
    for (position, coordinate) in replay.iter_mut().enumerate() {
        let position = valid_sample_f64(position);
        let bin = valid_sample_f64(mode.bin);
        let angle = std::f64::consts::TAU * bin * position / count;
        *coordinate = coordinate.add(mode.coefficient.multiply(unit(angle)));
    }
}

fn unit(angle: f64) -> ComplexCoordinate {
    let (imag, real) = angle.sin_cos();
    ComplexCoordinate { real, imag }
}

fn maximum_distance(left: &[ComplexCoordinate], right: &[ComplexCoordinate]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left.distance(*right))
        .fold(0.0, f64::max)
}

fn bounded_f64(value: usize, source_name: &str) -> Result<f64, LanguageError> {
    value.to_f64().ok_or_else(|| {
        frequency_error(
            "NSQ004",
            "a bounded sample coordinate cannot be projected to f64",
            source_name,
        )
    })
}

fn valid_sample_f64(value: usize) -> f64 {
    f64::from(
        u32::try_from(value)
            .expect("the validated frequency window keeps sample coordinates within u32"),
    )
}

fn frequency_error(code: &str, message: impl Into<String>, source_name: &str) -> LanguageError {
    LanguageError(Diagnostic {
        code: code.into(),
        message: message.into(),
        source_name: source_name.into(),
        span: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{MultiIndex, NativeScalar};

    fn quarter_turn_sequence(sample_count: usize) -> NativeState {
        NativeState::from_terms((0..sample_count).map(|offset| {
            let direction = u64::try_from(offset + 1).expect("test sample index fits u64");
            let turns = i64::try_from(offset).expect("test phase fits i64");
            (
                MultiIndex::from_depths([(direction, 1)]).unwrap(),
                NativeScalar::one().orient(turns),
            )
        }))
    }

    #[test]
    fn quarter_turn_projection_synthesizes_one_mode() {
        let program = synthesize(&quarter_turn_sequence(16), 1, 16, "1e-12", "quarter.ns").unwrap();

        assert_eq!(program.modes().len(), 1);
        assert_eq!(program.modes()[0].bin(), 4);
        assert!(program.observed_error() <= program.maximum_error());
        assert_eq!(program.reconstruct().len(), 16);
    }

    #[test]
    fn constant_projection_synthesizes_the_zero_bin() {
        let state = NativeState::from_terms((1..=8).map(|direction| {
            (
                MultiIndex::from_depths([(direction, 1)]).unwrap(),
                NativeScalar::from_text("3", "0").unwrap(),
            )
        }));
        let program = synthesize(&state, 1, 8, "0", "constant.ns").unwrap();

        assert_eq!(program.modes().len(), 1);
        assert_eq!(program.modes()[0].bin(), 0);
        assert!(program.observed_error().abs() <= f64::EPSILON);
    }

    #[test]
    fn terms_outside_the_declared_window_are_rejected() {
        let error = synthesize(&quarter_turn_sequence(5), 1, 4, "1e-12", "outside.ns").unwrap_err();

        assert_eq!(error.0.code, "NSQ003");
    }

    #[test]
    fn nonperiodic_window_is_not_reported_as_sparse() {
        let values = [1, 1, 2, 3, 5, 8, 13];
        let state = NativeState::from_terms(values.iter().enumerate().map(|(offset, value)| {
            let direction = u64::try_from(offset + 1).expect("test sample index fits u64");
            (
                MultiIndex::from_depths([(direction, 1)]).unwrap(),
                NativeScalar::from_text(&value.to_string(), "0").unwrap(),
            )
        }));
        let program = synthesize(&state, 1, values.len(), "1e-12", "fibonacci.ns").unwrap();

        assert_eq!(program.modes().len(), values.len());
        assert!(program.observed_error() <= program.maximum_error());
    }
}
