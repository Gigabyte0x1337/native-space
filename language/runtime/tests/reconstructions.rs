// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

use native_space_language::core::{NativeScalar, NativeState};

fn scalar(real: &str, imag: &str) -> NativeScalar {
    NativeScalar::from_text(real, imag).unwrap()
}
fn squared_size(value: &NativeScalar) -> NativeScalar {
    value.multiply(&NativeScalar {
        real: value.real.clone(),
        imag: -&value.imag,
    })
}

#[test]
fn sum_of_two_squares_size_is_multiplicative() {
    let left = scalar("3", "4");
    let right = scalar("5", "-2");
    assert_eq!(
        squared_size(&left.multiply(&right)),
        squared_size(&left).multiply(&squared_size(&right))
    );
}

#[test]
fn finite_geometric_identity_is_exact() {
    let x = NativeState::one().index_power(1, 1).unwrap();
    let minus_one = NativeState::scalar(scalar("-1", "0"));
    let x2 = x.multiply(&x);
    let x3 = x2.multiply(&x);
    let x4 = x3.multiply(&x);
    let sum = NativeState::one().add(&x).add(&x2).add(&x3);
    assert_eq!(
        NativeState::one()
            .add(&minus_one.multiply(&x))
            .multiply(&sum),
        NativeState::one().add(&minus_one.multiply(&x4))
    );
}

#[test]
fn pythagorean_parameterization_lies_on_the_cone() {
    let (major, minor) = (5_i64, 2_i64);
    let first_leg = major * major - minor * minor;
    let second_leg = 2 * major * minor;
    let hypotenuse = major * major + minor * minor;
    assert_eq!(
        first_leg * first_leg + second_leg * second_leg,
        hypotenuse * hypotenuse
    );
}

#[test]
fn native_binomial_identity_is_exact() {
    let a = NativeState::one().index_power(2, 1).unwrap();
    let b = NativeState::one().index_power(3, 1).unwrap();
    let left = a.add(&b).multiply(&a.add(&b));
    let right = a
        .multiply(&a)
        .add(
            &NativeState::scalar(scalar("2", "0"))
                .multiply(&a)
                .multiply(&b),
        )
        .add(&b.multiply(&b));
    assert_eq!(left, right);
}

fn dft(values: &[NativeScalar]) -> Vec<NativeScalar> {
    let n = values.len();
    (0..n)
        .map(|frequency| {
            values
                .iter()
                .enumerate()
                .fold(NativeScalar::zero(), |sum, (position, value)| {
                    let turns = -i64::try_from(frequency * position * 4 / n).unwrap();
                    sum.add(&value.orient(turns))
                })
        })
        .collect()
}
fn cyclic_convolution(left: &[NativeScalar], right: &[NativeScalar]) -> Vec<NativeScalar> {
    let n = left.len();
    (0..n)
        .map(|index| {
            (0..n).fold(NativeScalar::zero(), |sum, k| {
                sum.add(&left[k].multiply(&right[(index + n - k) % n]))
            })
        })
        .collect()
}

#[test]
fn quarter_turn_dft_preserves_cyclic_multiplication() {
    let left = [
        scalar("1", "0"),
        scalar("2", "0"),
        scalar("-1", "0"),
        scalar("3", "0"),
    ];
    let right = [
        scalar("0", "1"),
        scalar("1", "0"),
        scalar("2", "0"),
        scalar("-2", "0"),
    ];
    let transformed = dft(&cyclic_convolution(&left, &right));
    let pointwise = dft(&left)
        .iter()
        .zip(dft(&right))
        .map(|(a, b)| a.multiply(&b))
        .collect::<Vec<_>>();
    assert_eq!(transformed, pointwise);
}
