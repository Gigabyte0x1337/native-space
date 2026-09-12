// SPDX-License-Identifier: AGPL-3.0-or-later
//! Progressive numerical exponential fits; never a convergence proof or exact rewrite.
//!
//! Samples are F(N), F(2N), ... . Each rank fits only the training prefix.
//! Validation selects among ranks, so it is held out from fitting, not an
//! independent final test set. Every attempted fit and unexplained residual
//! remains available. A joint refit prevents an approximate first layer from
//! irrevocably biasing subsequent residual extraction.
use nalgebra::{
    DMatrix, DVector,
    linalg::{SVD, Schur},
};

// Bound numerical work rather than let a decomposition iterate indefinitely.
const MAX_ITERATIONS: usize = 10_000;
const MAX_LAYERS: usize = 16;
const MAX_SAMPLES: usize = 4096;

/// Controls finite-window numerical validation, not mathematical convergence.
#[derive(Clone, Debug)]
pub struct ProbeOptions {
    pub max_layers: usize,
    pub held_out: usize,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
}
impl Default for ProbeOptions {
    fn default() -> Self {
        Self {
            max_layers: 4,
            held_out: 2,
            absolute_tolerance: 1e-10,
            relative_tolerance: 1e-8,
        }
    }
}
/// One real decaying contribution amplitude * ratio^sample_index.
#[derive(Clone, Debug)]
pub struct Layer {
    pub ratio: f64,
    pub alpha: f64,
    pub sign: i8,
    pub amplitude: f64,
}
/// One complete hypothesis, including data it does not explain.
#[derive(Clone, Debug)]
pub struct Fit {
    pub limit: f64,
    pub layers: Vec<Layer>,
    pub predictions: Vec<f64>,
    pub held_out_predictions: Vec<f64>,
    pub held_out_errors: Vec<f64>,
    pub held_out_error: f64,
    /// Residual after subtracting the limit, then each successive layer.
    pub residual_stages: Vec<Vec<f64>>,
    pub residual: Vec<f64>,
    /// Training and every held-out value meet the caller's tolerance.
    pub validated: bool,
}
/// Retains attempted models, selection, and the final unexplained residual.
#[derive(Clone, Debug)]
pub struct ScaleReport {
    pub training_samples: usize,
    pub attempts: Vec<Fit>,
    /// Best improving, validated fit; None means no candidate passed validation.
    pub selected: Option<usize>,
    /// Best improving fit, which may still be unvalidated.
    pub best: Option<usize>,
    pub unexplained_residual: Vec<f64>,
    pub stopped: String,
}

/// Fits progressively richer real scale laws using a held-out suffix.
///
/// # Errors
/// Rejects nonfinite samples, invalid tolerances, insufficient samples, or
/// work above 4096 samples / 16 layers. Numerical fit failures are reported
/// in the returned report, preserving all observations as residual.
pub fn progressive(samples: &[f64], options: &ProbeOptions) -> Result<ScaleReport, String> {
    if samples.len() > MAX_SAMPLES || !samples.iter().all(|v| v.is_finite()) {
        return Err("probe requires at most 4096 finite samples".into());
    }
    if options.max_layers == 0
        || options.max_layers > MAX_LAYERS
        || options.held_out == 0
        || options.held_out > samples.len()
        || samples.len() - options.held_out < 3
    {
        return Err(
            "probe needs 1..16 layers, a held-out suffix and at least three training samples"
                .into(),
        );
    }
    if [options.absolute_tolerance, options.relative_tolerance]
        .iter()
        .any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err("probe tolerances must be finite and nonnegative".into());
    }
    let training = samples.len() - options.held_out;
    let mut report = ScaleReport {
        training_samples: training,
        attempts: Vec::new(),
        selected: None,
        best: None,
        unexplained_residual: samples.to_vec(),
        stopped: "layer/data limit reached".into(),
    };
    // The first fitted rank establishes the baseline. A constant predictor can
    // outperform a misspecified one-layer mixture and would prevent inspecting
    // its missing second layer. Validation remains required before selection.
    let mut best_error = f64::INFINITY;
    for rank in 1..=options.max_layers.min((training - 1) / 2) {
        let fit = match fit(samples, training, rank, options) {
            Ok(fit) => fit,
            Err(reason) => {
                report.stopped = reason;
                break;
            }
        };
        let improved = fit.held_out_error < best_error;
        report.attempts.push(fit);
        if !improved {
            report.stopped = "held-out error no longer improves".into();
            break;
        }
        let index = report.attempts.len() - 1;
        let fit = &report.attempts[index];
        best_error = fit.held_out_error;
        report.best = Some(index);
        report.unexplained_residual = fit.residual.clone();
        if fit.validated {
            report.selected = Some(index);
        }
    }
    Ok(report)
}

fn fit(
    samples: &[f64],
    training: usize,
    rank: usize,
    options: &ProbeOptions,
) -> Result<Fit, String> {
    let train = &samples[..training];
    // Scale numerical work explicitly; it has no connection to authoritative L/A/M.
    let scale = train.iter().map(|v| v.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        return Err("constant training prefix has no identifiable decaying layer".into());
    }
    let normalized: Vec<_> = train.iter().map(|v| v / scale).collect();
    let differences: Vec<_> = normalized.windows(2).map(|p| p[1] - p[0]).collect();
    let size = differences.len() - rank;
    let a = DMatrix::from_fn(size, rank, |i, j| differences[i + j]);
    let b = DVector::from_fn(size, |i, _| differences[i + rank]);
    let recurrence = solve(a, b)?;
    // Characteristic roots of d[n+k] = sum_j c[j] d[n+j].
    let companion = DMatrix::from_fn(rank, rank, |i, j| {
        if j == rank - 1 {
            recurrence[i]
        } else if i == j + 1 {
            1.0
        } else {
            0.0
        }
    });
    let schur = Schur::try_new(companion, f64::EPSILON, MAX_ITERATIONS)
        .ok_or("scale eigenvalue iteration did not converge")?;
    let roots = schur
        .eigenvalues()
        .ok_or("candidate has nonreal ratios; no real-layer fit")?;
    let mut ratios: Vec<_> = roots.iter().copied().collect();
    if ratios
        .iter()
        .any(|r| !r.is_finite() || *r == 0.0 || r.abs() >= 1.0)
    {
        return Err("candidate ratios are not nonzero real decays".into());
    }
    ratios.sort_by(|a, b| b.abs().total_cmp(&a.abs()).then(a.total_cmp(b)));
    let design = DMatrix::from_fn(training, rank + 1, |i, j| {
        if j == 0 {
            1.0
        } else {
            ratios[j - 1].powi(i as i32)
        }
    });
    let coefficients = solve(design, DVector::from_vec(normalized))? * scale;
    let limit = coefficients[0];
    let layers: Vec<_> = ratios
        .into_iter()
        .enumerate()
        .map(|(i, ratio)| Layer {
            ratio,
            alpha: -ratio.abs().log2(),
            sign: if ratio < 0.0 { -1 } else { 1 },
            amplitude: coefficients[i + 1],
        })
        .collect();
    let mut predictions = vec![limit; samples.len()];
    let mut residual: Vec<_> = samples.iter().map(|v| v - limit).collect();
    let mut residual_stages = vec![residual.clone()];
    for layer in &layers {
        let mut contribution = layer.amplitude;
        for (prediction, residual) in predictions.iter_mut().zip(&mut residual) {
            *prediction += contribution;
            *residual -= contribution;
            contribution *= layer.ratio;
        }
        residual_stages.push(residual.clone());
    }
    if !predictions.iter().chain(&residual).all(|v| v.is_finite())
        || residual_stages.iter().flatten().any(|v| !v.is_finite())
    {
        return Err("scale prediction overflow".into());
    }
    let held_out_predictions = predictions[training..].to_vec();
    let held_out_errors: Vec<_> = samples[training..]
        .iter()
        .zip(&held_out_predictions)
        .map(|(a, b)| (a - b).abs())
        .collect();
    if held_out_errors.iter().any(|e| !e.is_finite()) {
        return Err("held-out error overflow".into());
    }
    let held_out_error = held_out_errors.iter().copied().fold(0.0, f64::max);
    let validated = samples.iter().zip(&predictions).all(|(a, b)| {
        let error = (a - b).abs();
        // Separate comparisons avoid a tolerance addition overflowing to infinity.
        error.is_finite()
            && (error <= options.absolute_tolerance
                || error / a.abs().max(b.abs()).max(f64::MIN_POSITIVE)
                    <= options.relative_tolerance)
    });
    Ok(Fit {
        limit,
        layers,
        predictions,
        held_out_predictions,
        held_out_errors,
        held_out_error,
        residual_stages,
        residual,
        validated,
    })
}

fn solve(a: DMatrix<f64>, b: DVector<f64>) -> Result<DVector<f64>, String> {
    let columns = a.ncols();
    let dimension = a.nrows().max(columns);
    let svd = SVD::try_new(a, true, true, f64::EPSILON, MAX_ITERATIONS)
        .ok_or("scale least-squares iteration did not converge")?;
    // Rank cutoff is relative to this matrix, not to the original sample units.
    let cutoff = 64.0 * f64::EPSILON * dimension as f64 * svd.singular_values[0];
    if svd.rank(cutoff) != columns {
        return Err("scale fit is rank-deficient".into());
    }
    let result = svd.solve(&b, cutoff).map_err(str::to_string)?;
    if result.iter().any(|v| !v.is_finite()) {
        return Err("nonfinite scale fit".into());
    }
    Ok(result)
}
