// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coefficients compiled from the frozen maps, not an alternate scalar algebra.
use super::{Rational, Split, State, Transform, linear};
use num_traits::{One, Zero};

type Tensor = [[[Rational; 3]; 3]; 3];
type Matrix = [[Rational; 3]; 3];

/// Exact local-coordinate bilinear tensors and linear maps for one frame.
#[derive(Clone, Debug)]
pub struct Transported {
    add: Tensor,
    multiply: Tensor,
    negate: Matrix,
    split_add: Matrix,
    split_multiply: Matrix,
}
impl Transported {
    pub(super) fn new(t: &Transform) -> Self {
        // A bilinear map is determined by its values on all pairs of basis vectors.
        // Evaluate the unchanged canonical equations only during coefficient compilation.
        let basis: [State; 3] = std::array::from_fn(|i| {
            let c = std::array::from_fn(|j| {
                if i == j {
                    Rational::one()
                } else {
                    Rational::zero()
                }
            });
            let [l, a, m] = c;
            t.decode(&State::new(l, a, m))
        });
        let bilinear = |op: fn(&State, &State) -> State| {
            let images: [[State; 3]; 3] = std::array::from_fn(|i| {
                std::array::from_fn(|j| t.encode(&op(&basis[i], &basis[j])))
            });
            std::array::from_fn(|o| {
                std::array::from_fn(|i| {
                    std::array::from_fn(|j| images[i][j].coordinates()[o].clone())
                })
            })
        };
        let unary = |op: fn(&State) -> State| {
            let images: [State; 3] = std::array::from_fn(|i| t.encode(&op(&basis[i])));
            std::array::from_fn(|o| std::array::from_fn(|i| images[i].coordinates()[o].clone()))
        };
        Self {
            add: bilinear(State::add),
            multiply: bilinear(State::multiply),
            negate: unary(State::negate),
            split_add: unary(|p| p.split(Split::Add)),
            split_multiply: unary(|p| p.split(Split::Multiply)),
        }
    }
    /// Applies ADD directly to local states.
    pub fn add(&self, u: &State, v: &State) -> State {
        contract(&self.add, u, v)
    }
    /// Applies MULTIPLY directly to local states.
    pub fn multiply(&self, u: &State, v: &State) -> State {
        contract(&self.multiply, u, v)
    }
    /// Applies negation directly to a local state.
    pub fn negate(&self, u: &State) -> State {
        linear(&self.negate, u)
    }
    /// Applies half-refinement directly to a local state.
    pub fn split(&self, u: &State, route: Split) -> State {
        linear(
            match route {
                Split::Add => &self.split_add,
                Split::Multiply => &self.split_multiply,
            },
            u,
        )
    }
}
fn contract(tensor: &Tensor, u: &State, v: &State) -> State {
    let a = u.coordinates();
    let b = v.coordinates();
    let values: [Rational; 3] = std::array::from_fn(|o| {
        let mut sum = Rational::zero();
        for (i, x) in a.iter().enumerate() {
            for (j, y) in b.iter().enumerate() {
                let coefficient = &tensor[o][i][j];
                if !coefficient.is_zero() {
                    sum += coefficient * x * y;
                }
            }
        }
        sum
    });
    let [l, a, m] = values;
    State::new(l, a, m)
}
