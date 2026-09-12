// SPDX-License-Identifier: AGPL-3.0-or-later
//! Numerical cameras are fallible readouts, never authoritative state.
use crate::algebra::{Rational, State};
use num_traits::{Signed, ToPrimitive};
pub fn finite(v: &Rational) -> Result<f64, String> {
    let n = v.to_f64().ok_or("coordinate is outside f64 range")?;
    if !n.is_finite() || (n == 0.0 && v != &Rational::from_integer(0.into())) {
        return Err("coordinate overflow or underflow".into());
    }
    Ok(n)
}
pub fn raw(p: &State) -> Result<[f64; 3], String> {
    Ok([finite(&p.l)?, finite(&p.a)?, finite(&p.m)?])
}
pub fn classical(p: &State) -> Result<[f64; 3], String> {
    let (r, x) = p.decode()?;
    Ok([finite(&x)?, finite(&r)?, 0.0])
}
pub fn orthogonal(p: &State) -> Result<[f64; 3], String> {
    let [l, a, m] = raw(p)?;
    let v = [
        (a - m) / 2_f64.sqrt(),
        (a + m - 2.0 * l) / 6_f64.sqrt(),
        (l + a + m) / 3_f64.sqrt(),
    ];
    if v.iter().all(|v| v.is_finite()) {
        Ok(v)
    } else {
        Err("orthogonal camera overflow".into())
    }
}
pub fn orthogonal_inverse([x, y, z]: [f64; 3]) -> Result<[f64; 3], String> {
    let v = [
        -2.0 * y / 6_f64.sqrt() + z / 3_f64.sqrt(),
        x / 2_f64.sqrt() + y / 6_f64.sqrt() + z / 3_f64.sqrt(),
        -x / 2_f64.sqrt() + y / 6_f64.sqrt() + z / 3_f64.sqrt(),
    ];
    if v.iter().all(|v| v.is_finite()) {
        Ok(v)
    } else {
        Err("inverse camera overflow".into())
    }
}
fn log2(v: &Rational) -> Result<f64, String> {
    if !v.is_positive() {
        return Err("log camera requires L, A, M > 0".into());
    }
    // Scale each exact integer before conversion, avoiding overflow at 2^1000 and beyond.
    fn log_integer(n: &num_bigint::BigInt) -> f64 {
        let shift = n.bits().saturating_sub(52);
        (n >> shift)
            .to_f64()
            .expect("52-bit positive integer")
            .log2()
            + shift as f64
    }
    Ok(log_integer(v.numer()) - log_integer(v.denom()))
}
pub fn log_ratio(p: &State) -> Result<[f64; 3], String> {
    let l = log2(&p.l)?;
    let a = log2(&p.a)?;
    let m = log2(&p.m)?;
    Ok([
        (a - m) / 2_f64.sqrt(),
        (2.0 * l - a - m) / 6_f64.sqrt(),
        log2(&p.total())?,
    ])
}
pub fn log_ratio_inverse([x, y, q]: [f64; 3]) -> Result<[f64; 3], String> {
    if ![x, y, q].iter().all(|v| v.is_finite()) {
        return Err("nonfinite log coordinates".into());
    }
    let w = [
        2.0 * y / 6_f64.sqrt(),
        x / 2_f64.sqrt() - y / 6_f64.sqrt(),
        -x / 2_f64.sqrt() - y / 6_f64.sqrt(),
    ];
    let max = w.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let w = w.map(|v| (v - max).exp2());
    let sum: f64 = w.iter().sum();
    let v = w.map(|v| (q + (v / sum).log2()).exp2());
    if v.iter().all(|v| v.is_finite() && *v > 0.0) {
        Ok(v)
    } else {
        Err("inverse log camera exceeds f64 range".into())
    }
}
