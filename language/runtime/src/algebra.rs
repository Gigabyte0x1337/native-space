// SPDX-License-Identifier: AGPL-3.0-or-later
//! Authoritative rational triples. Raw equality deliberately retains common scale.
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
pub type Rational = BigRational;

pub fn rational(text: &str) -> Result<Rational, String> {
    let (n, d) = text.split_once('/').unwrap_or((text, "1"));
    let n = BigInt::from_str(n).map_err(|_| format!("invalid rational {text:?}"))?;
    let d = BigInt::from_str(d).map_err(|_| format!("invalid rational {text:?}"))?;
    if d.is_zero() {
        return Err("rational denominator is zero".into());
    }
    Ok(Rational::new(n, d))
}

/// L/A/M components are independent of graph metadata and observation indices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub l: Rational,
    pub a: Rational,
    pub m: Rational,
}

impl State {
    pub fn new(l: Rational, a: Rational, m: Rational) -> Self {
        Self { l, a, m }
    }
    /// Literal n embeds the diagonal decoded pair (n,n); no hidden order is guessed.
    pub fn number(n: Rational) -> Self {
        Self::new(Rational::one(), Rational::zero(), n)
    }
    pub fn zero() -> Self {
        Self::number(Rational::zero())
    }
    pub fn one() -> Self {
        Self::number(Rational::one())
    }
    pub fn total(&self) -> Rational {
        &self.l + &self.a + &self.m
    }
    pub fn decode(&self) -> Result<(Rational, Rational), String> {
        if self.l.is_zero() {
            return Err("L = 0: finite classical readout is undefined".into());
        }
        Ok(((&self.a + &self.m) / &self.l, &self.m / &self.l))
    }
    pub fn add(&self, q: &Self) -> Self {
        Self::new(
            &self.l * &q.l,
            &self.a * &q.l + &q.a * &self.l,
            &self.m * &q.l + &q.m * &self.l,
        )
    }
    pub fn multiply(&self, q: &Self) -> Self {
        Self::new(
            &self.l * &q.l,
            &self.a * &q.a + &self.a * &q.m + &self.m * &q.a,
            &self.m * &q.m,
        )
    }
    pub fn negate(&self) -> Self {
        Self::new(self.l.clone(), -&self.a, -&self.m)
    }
    pub fn inverse(&self) -> Result<Self, String> {
        self.decode()?;
        let sum = &self.a + &self.m;
        if self.m.is_zero() || sum.is_zero() {
            return Err("inverse requires M != 0 and A + M != 0".into());
        }
        Ok(Self::new(
            &self.m * &sum,
            -(&self.a * &self.l),
            &self.l * &sum,
        ))
    }
    pub fn split(&self, route: Split) -> Self {
        let half = &self.l / rational("2").expect("constant");
        match route {
            Split::Add => Self::new(half.clone(), &self.a + half, self.m.clone()),
            Split::Multiply => Self::new(half.clone(), self.a.clone(), &self.m + half),
        }
    }
    pub fn rescale(&self, c: &Rational) -> Result<Self, String> {
        if c.is_zero() {
            return Err("reversible scale must be nonzero".into());
        }
        Ok(Self::new(&self.l * c, &self.a * c, &self.m * c))
    }
    pub fn simplex(&self) -> Result<[Rational; 3], String> {
        let t = self.total();
        if t.is_zero() {
            return Err("T = 0: simplex camera is undefined".into());
        }
        Ok([&self.l / &t, &self.a / &t, &self.m / &t])
    }
    /// Algebraic equality checks both decoded coordinates, not only classical x.
    pub fn equivalent(&self, q: &Self) -> Result<bool, String> {
        Ok(self.decode()? == q.decode()?)
    }
    pub fn coordinates(&self) -> [Rational; 3] {
        [self.l.clone(), self.a.clone(), self.m.clone()]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Split {
    Add,
    Multiply,
}

/// Validated invertible rational linear camera; it never asserts an inverse on samples.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Transform {
    forward: [[Rational; 3]; 3],
    inverse: [[Rational; 3]; 3],
}
impl Transform {
    pub fn new(matrix: [[Rational; 3]; 3]) -> Result<Self, String> {
        let mut rows: Vec<Vec<Rational>> = (0..3)
            .map(|i| {
                (0..6)
                    .map(|j| {
                        if j < 3 {
                            matrix[i][j].clone()
                        } else if j - 3 == i {
                            Rational::one()
                        } else {
                            Rational::zero()
                        }
                    })
                    .collect()
            })
            .collect();
        for col in 0..3 {
            let pivot = (col..3)
                .find(|&r| !rows[r][col].is_zero())
                .ok_or("transform matrix is singular")?;
            rows.swap(col, pivot);
            let scale = rows[col][col].clone();
            for entry in &mut rows[col] {
                *entry /= &scale;
            }
            let pivot_row = rows[col].clone();
            for (r, row) in rows.iter_mut().enumerate() {
                if r != col {
                    let scale = row[col].clone();
                    for (v, p) in row.iter_mut().zip(&pivot_row) {
                        *v -= &scale * p;
                    }
                }
            }
        }
        let inverse = std::array::from_fn(|i| std::array::from_fn(|j| rows[i][j + 3].clone()));
        Ok(Self {
            forward: matrix,
            inverse,
        })
    }
    pub fn matrix(&self) -> &[[Rational; 3]; 3] {
        &self.forward
    }
    pub fn inverse_matrix(&self) -> &[[Rational; 3]; 3] {
        &self.inverse
    }
    pub fn encode(&self, p: &State) -> State {
        linear(&self.forward, p)
    }
    pub fn decode(&self, p: &State) -> State {
        linear(&self.inverse, p)
    }
    pub fn cost(&self) -> (usize, usize) {
        let nonzero = self
            .forward
            .iter()
            .flatten()
            .filter(|v| !v.is_zero())
            .count();
        let adds = self
            .forward
            .iter()
            .map(|r| r.iter().filter(|v| !v.is_zero()).count().saturating_sub(1))
            .sum();
        (adds, nonzero)
    }
}
fn linear(matrix: &[[Rational; 3]; 3], p: &State) -> State {
    let c = p.coordinates();
    let v: [Rational; 3] =
        std::array::from_fn(|i| matrix[i].iter().zip(&c).map(|(a, b)| a * b).sum());
    State::new(v[0].clone(), v[1].clone(), v[2].clone())
}
impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            forward: [[Rational; 3]; 3],
            inverse: [[Rational; 3]; 3],
        }
        let w = Wire::deserialize(d)?;
        let t = Self::new(w.forward).map_err(serde::de::Error::custom)?;
        if t.inverse != w.inverse {
            return Err(serde::de::Error::custom("incorrect transform inverse"));
        }
        Ok(t)
    }
}
