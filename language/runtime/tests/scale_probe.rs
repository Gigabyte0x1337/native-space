// SPDX-License-Identifier: AGPL-3.0-or-later
use native_space_language::optimize::{
    probe,
    scale::{ProbeOptions, progressive},
};

fn samples(limit: f64, layers: &[(f64, f64)], n: usize) -> Vec<f64> {
    (0..n)
        .map(|k| {
            limit
                + layers
                    .iter()
                    .map(|(amplitude, ratio)| amplitude * ratio.powi(k as i32))
                    .sum::<f64>()
        })
        .collect()
}
#[test]
fn single_positive_and_alternating_laws_predict_held_out_samples() {
    for ratio in [0.5, -0.5, 0.8, -0.8] {
        let data = samples(3.0, &[(2.0, ratio)], 8);
        let quick = probe(data[..4].try_into().unwrap()).unwrap();
        assert!((quick.ratio - ratio).abs() < 1e-12);
        assert!((quick.alpha + ratio.abs().log2()).abs() < 1e-12);
        assert_eq!(quick.sign, if ratio < 0.0 { -1 } else { 1 });
        assert!(quick.error < 1e-12);
        let report = progressive(&data, &ProbeOptions::default()).unwrap();
        let fit = &report.attempts[report.selected.expect("validated fit")];
        assert_eq!(fit.layers.len(), 1);
        assert!(fit.held_out_error < 1e-10);
        assert_eq!(fit.held_out_predictions.len(), 2);
    }
}
#[test]
fn progressive_layers_retain_every_residual_and_improve_prediction() {
    for layers in [
        vec![(2.0, 0.8), (1.0, 0.3)],
        vec![(2.0, 0.8), (1.0, -0.3)],
        vec![(2.0, 0.8), (1.0, -0.5), (0.7, 0.2)],
    ] {
        let data = samples(3.0, &layers, 12);
        let report = progressive(&data, &ProbeOptions::default()).unwrap();
        let chosen = report
            .selected
            .unwrap_or_else(|| panic!("mixture {layers:?}: {report:?}"));
        let fit = &report.attempts[chosen];
        assert_eq!(fit.layers.len(), layers.len());
        assert!(fit.held_out_error < 1e-8);
        assert_eq!(fit.residual_stages.len(), layers.len() + 1);
        for i in 1..=chosen {
            assert!(report.attempts[i].held_out_error < report.attempts[i - 1].held_out_error);
        }
        for (k, observed) in data.iter().enumerate() {
            let explained = fit.limit
                + fit
                    .layers
                    .iter()
                    .map(|l| l.amplitude * l.ratio.powi(k as i32))
                    .sum::<f64>();
            assert!((explained + fit.residual[k] - observed).abs() < 1e-12);
            for (j, layer) in fit.layers.iter().enumerate() {
                assert!(
                    (fit.residual_stages[j][k]
                        - layer.amplitude * layer.ratio.powi(k as i32)
                        - fit.residual_stages[j + 1][k])
                        .abs()
                        < 1e-12
                );
            }
        }
        assert_eq!(
            report.unexplained_residual,
            report.attempts[report.best.unwrap()].residual
        );
    }
}
#[test]
fn held_out_corruption_cannot_change_fitted_parameters_or_become_a_proof() {
    let data = samples(1.0, &[(2.0, 0.5)], 8);
    let mut bad = data.clone();
    bad[7] += 9.0;
    let options = ProbeOptions {
        max_layers: 1,
        ..ProbeOptions::default()
    };
    let good_report = progressive(&data, &options).unwrap();
    let bad_report = progressive(&bad, &options).unwrap();
    assert!(good_report.selected.is_some());
    assert!(bad_report.selected.is_none());
    let a = &good_report.attempts[0];
    let b = &bad_report.attempts[0];
    assert_eq!(a.limit, b.limit);
    assert_eq!(a.layers[0].ratio, b.layers[0].ratio);
    assert_eq!(a.layers[0].amplitude, b.layers[0].amplitude);
    assert_eq!(a.held_out_predictions, b.held_out_predictions);
    assert!(b.residual[7].abs() > 8.0);
}
#[test]
fn failures_are_explicit_and_leave_data_available() {
    for data in [vec![2.0; 8], samples(0.0, &[(1.0, 2.0)], 8)] {
        let report = progressive(&data, &ProbeOptions::default()).unwrap();
        assert!(report.selected.is_none());
        assert_eq!(report.unexplained_residual, data);
        assert!(!report.stopped.is_empty());
    }
    assert!(progressive(&[1.0, f64::NAN, 2.0, 3.0], &ProbeOptions::default()).is_err());
    assert!(progressive(&[1.0, 2.0], &ProbeOptions::default()).is_err());
    assert!(
        progressive(
            &[1.0; 8],
            &ProbeOptions {
                max_layers: 0,
                ..ProbeOptions::default()
            }
        )
        .is_err()
    );
    assert!(
        progressive(
            &[1.0; 8],
            &ProbeOptions {
                absolute_tolerance: f64::INFINITY,
                ..ProbeOptions::default()
            }
        )
        .is_err()
    );
}
