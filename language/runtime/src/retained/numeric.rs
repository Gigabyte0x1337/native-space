// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Explicit f64 execution of a retained graph, never a proof or a replacement state.
//!
//! Source elaboration and index routing remain exact. Each scalar ADD/MULTIPLY
//! rounds in f64. Numerical zeros are approximate observations; the original
//! exact graph is kept, including operands lost to rounding. Overflow and
//! underflow-to-zero are errors, not fabricated finite coordinates.

use num_traits::{ToPrimitive, Zero};
use serde_json::{Value, json};

use super::{
    Scalar, ScalarData, State,
    coordinates::Point,
    field::{self, Coefficient},
};
use crate::core::{MultiIndex, NativeScalar, NativeState, OutputKind, Rational};

#[derive(Clone, Debug)]
struct Complex {
    real: f64,
    imag: f64,
    boundary_ray: Option<NativeScalar>,
}

impl Complex {
    fn native(&self) -> NativeScalar {
        NativeScalar {
            real: Rational::from_float(self.real).expect("validated finite f64"),
            imag: Rational::from_float(self.imag).expect("validated finite f64"),
        }
    }

    fn is_zero(&self) -> bool {
        self.real == 0.0 && self.imag == 0.0
    }

    fn native_scalar(&self) -> Scalar {
        if self.is_zero() {
            Scalar::from_coordinates(
                super::Depth::of(&NativeScalar::zero()),
                self.boundary_ray.clone(),
            )
            .expect("boundary rays remain nonzero")
        } else {
            Scalar::from_classical(&self.native())
        }
    }

    fn ray(&self) -> Option<NativeScalar> {
        self.native_scalar().direction().cloned()
    }
}

impl Coefficient for Complex {
    fn scalar(data: &ScalarData) -> Result<Self, String> {
        let scalar = Scalar::from_data(data)?;
        let value = scalar.project();
        Ok(Self {
            real: convert(&value.real)?,
            imag: convert(&value.imag)?,
            boundary_ray: if value.is_zero() {
                scalar.direction().cloned()
            } else {
                None
            },
        })
    }
    fn add(&self, right: &Self) -> Result<Self, String> {
        Ok(Self {
            real: finite(self.real + right.real)?,
            imag: finite(self.imag + right.imag)?,
            boundary_ray: None,
        })
    }
    fn multiply(&self, right: &Self) -> Result<Self, String> {
        Ok(Self {
            real: finite(product(self.real, right.real)? - product(self.imag, right.imag)?)?,
            imag: finite(product(self.real, right.imag)? + product(self.imag, right.real)?)?,
            // An explicit zero-boundary phase is provenance, not zero's argument.
            // A zero created only by rounded cancellation has no invented phase.
            boundary_ray: if self.is_zero() || right.is_zero() {
                self.ray()
                    .zip(right.ray())
                    .map(|(left, right)| left.multiply(&right))
            } else {
                None
            },
        })
    }
    fn phase(&self, turns: i64) -> Result<Self, String> {
        // Sign/swap is exact for f64. No trigonometric approximation of i.
        let (real, imag) = match turns {
            0 => (self.real, self.imag),
            1 => (-self.imag, self.real),
            2 => (-self.real, -self.imag),
            3 => (self.imag, -self.real),
            _ => return Err("invalid quarter-turn in retained graph".into()),
        };
        Ok(Self {
            real,
            imag,
            boundary_ray: self.boundary_ray.as_ref().map(|ray| ray.phase(turns)),
        })
    }
}

fn convert(value: &Rational) -> Result<f64, String> {
    let rounded = finite(value.to_f64().ok_or("literal exceeds f64 range")?)?;
    if rounded == 0.0 && !value.is_zero() {
        return Err("f64 underflow would turn a nonzero literal into zero; use exact mode".into());
    }
    Ok(rounded)
}

fn finite(value: f64) -> Result<f64, String> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err("f64 overflow or invalid arithmetic; use exact mode".into())
    }
}

fn product(left: f64, right: f64) -> Result<f64, String> {
    let result = finite(left * right)?;
    if result == 0.0 && left != 0.0 && right != 0.0 {
        return Err("f64 multiplication underflow; use exact mode".into());
    }
    Ok(result)
}

/// Execute rounded arithmetic while preserving the original exact graph.
///
/// # Errors
/// Rejects overflow, underflow-to-zero, and output types requiring exact discrete values.
pub fn output(state: &State, kind: OutputKind) -> Result<Value, String> {
    if matches!(kind, OutputKind::Boolean | OutputKind::String) {
        return Err("boolean and string outputs require exact mode".into());
    }
    let fields = field::evaluate::<Complex>(state, false)?;
    let field = fields.last().ok_or("a state must contain a root")?;
    let projected = NativeState::from_terms(
        field
            .iter()
            .map(|(index, value)| (index.clone(), value.native())),
    );
    match kind {
        OutputKind::Auto | OutputKind::Pattern => Ok(json!({"kind":"pattern", "value":{
            "evaluation":"f64", "approximate":true, "state":state.native_data(),
            "observation":field.iter().map(|(index, value)| json!({
                "index":index.to_data(), "real":value.real, "imag":value.imag,
            })).collect::<Vec<_>>()
        }})),
        OutputKind::Number => {
            if projected
                .0
                .keys()
                .any(|index| *index != MultiIndex::default())
            {
                return Err("number output requires one unindexed real value".into());
            }
            let value = field
                .get(&MultiIndex::default())
                .cloned()
                .unwrap_or(Complex {
                    real: 0.0,
                    imag: 0.0,
                    boundary_ray: None,
                });
            if value.imag != 0.0 {
                return Err("number output requires a real value".into());
            }
            Ok(
                json!({"kind":"number", "value":value.real.to_string(), "evaluation":"f64", "approximate":true}),
            )
        }
        OutputKind::Vector => {
            if field.keys().any(|index| *index != MultiIndex::default()) {
                return Err("vector output requires one unindexed scalar; use view with --index-direction for indexed states".into());
            }
            let scalar = field.get(&MultiIndex::default()).map_or_else(
                || Scalar::from_classical(&NativeScalar::zero()),
                Complex::native_scalar,
            );
            let point = Point::new(scalar, 0_u8.into());
            Ok(
                json!({"kind":"vector", "value":vector(&point)?, "evaluation":"f64", "approximate":true}),
            )
        }
        OutputKind::Boolean | OutputKind::String => unreachable!("discrete outputs rejected above"),
    }
}

/// Observe each branch in the cylindrical frame with an explicit precision choice.
///
/// Exact mode keeps logarithms/radicals symbolic. Numerical mode executes all
/// scalar steps in f64 before observing coordinates. Neither mode changes INDEX.
///
/// # Errors
/// Rejects an invalid index direction, quarter-turn, or unrepresentable numeric execution.
pub fn view(
    state: &State,
    index_direction: u64,
    turns: i64,
    approximate: bool,
) -> Result<Value, String> {
    if index_direction == 0 || !crate::core::is_canonical_phase(turns) {
        return Err("view requires a positive index direction and turns from 0 through 3".into());
    }
    let fields = if approximate {
        field::evaluate::<Complex>(state, true)?
            .into_iter()
            .map(|field| {
                field
                    .into_iter()
                    .map(|(index, value)| (index, value.native_scalar()))
                    .collect()
            })
            .collect()
    } else {
        field::evaluate::<Scalar>(state, true)?
    };
    let mut locations = Vec::new();
    for (address, (step, field)) in state.plan().iter().zip(fields).enumerate() {
        let mut points = Vec::new();
        for (index, value) in field {
            let value = Coefficient::phase(&value, turns)?;
            let point = Point::new(value, index.depth(index_direction));
            points.push(json!({"index":index.to_data(), "point":if approximate {
                json!({"index":point.index().to_string(), "vector":vector(&point)?})
            } else { point.to_data() }}));
        }
        locations.push(json!({"address":address, "inputs":step.inputs,
            "retained":step.retained, "points":points}));
    }
    Ok(
        json!({"state":state.native_data(), "model":"depth-phase-index",
        "evaluation":if approximate {"f64"} else {"exact"}, "approximate":approximate,
        "index_direction":index_direction, "turns":turns, "locations":locations}),
    )
}

fn vector(point: &Point) -> Result<Value, String> {
    if point.scalar().direction().is_none() {
        return Ok(json!(["-infinity", null, null]));
    }
    let [x, y, z] = point.to_f64()?;
    Ok(json!([
        if x == f64::NEG_INFINITY {
            json!("-infinity")
        } else {
            json!(x)
        },
        y,
        z
    ]))
}
