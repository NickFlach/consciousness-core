//! Kuramoto oscillator model with configurable coupling.
//!
//! The Kuramoto model describes synchronization of coupled oscillators:
//!
//! ```text
//! dθᵢ/dt = ωᵢ + (K/N) Σⱼ wᵢⱼ sin(θⱼ - θᵢ)
//! ```
//!
//! Where:
//! - θᵢ = phase of oscillator i
//! - ωᵢ = natural frequency of oscillator i
//! - K = global coupling strength
//! - wᵢⱼ = pairwise coupling weight
//! - N = number of oscillators
//!
//! Supports static, market-mediated, and adaptive coupling modes.

#[cfg(not(feature = "std"))]
use crate::math_ext::F32Ext;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::f32::consts::{PI, TAU};

/// A single oscillator in the Kuramoto model.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Oscillator {
    /// Current phase θ ∈ [0, 2π)
    pub phase: f32,
    /// Natural frequency ω
    pub frequency: f32,
    /// Optional trust/weight for mean-field computation
    pub weight: f32,
}

impl Oscillator {
    pub fn new(phase: f32, frequency: f32) -> Self {
        Self {
            phase,
            frequency,
            weight: 1.0,
        }
    }

    pub fn with_weight(phase: f32, frequency: f32, weight: f32) -> Self {
        Self {
            phase,
            frequency,
            weight,
        }
    }
}

/// The Kuramoto order parameter: r·e^(iψ) = (1/N) Σ e^(iθⱼ)
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderParameter {
    /// Magnitude r ∈ [0, 1]: 1 = perfect sync, 0 = incoherent
    pub r: f32,
    /// Mean phase ψ
    pub psi: f32,
}

/// Report from a sync operation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SyncReport {
    pub oscillator_count: usize,
    pub initial_order: f32,
    pub final_order: f32,
    pub steps_taken: usize,
    pub converged: bool,
}

/// Configuration for the Kuramoto model.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KuramotoConfig {
    /// Base coupling strength K
    pub coupling_strength: f32,
    /// Time step for Euler integration
    pub dt: f32,
    /// Maximum integration steps per sync round
    pub max_steps: usize,
    /// Convergence threshold for order parameter change
    pub convergence_threshold: f32,
}

impl Default for KuramotoConfig {
    fn default() -> Self {
        Self {
            coupling_strength: 0.5,
            dt: 0.1,
            max_steps: 50,
            convergence_threshold: 1e-6,
        }
    }
}

/// The Kuramoto synchronization model.
pub struct KuramotoModel {
    pub config: KuramotoConfig,
}

impl KuramotoModel {
    pub fn new(config: KuramotoConfig) -> Self {
        Self { config }
    }

    /// Compute the order parameter: r·e^(iψ) = (1/N) Σ wⱼ·e^(iθⱼ)
    ///
    /// When all weights are equal, this reduces to the standard Kuramoto order parameter.
    /// Trust-weighted version: weight = oscillator.weight (e.g. trust × coherence).
    pub fn order_parameter(oscillators: &[Oscillator]) -> OrderParameter {
        if oscillators.is_empty() {
            return OrderParameter { r: 0.0, psi: 0.0 };
        }
        // Treat any negative weight as zero. The Kuramoto order parameter
        // is defined for non-negative trust/coherence weights; allowing
        // negatives lets `r = |Σ wⱼ e^(iθⱼ)| / Σ wⱼ` blow past 1.0 because
        // the denominator can be small or even negative while the
        // numerator's magnitude grows. Floor-at-zero is the conservative
        // reading of "active distrust" (#17).
        let effective_weight = |o: &Oscillator| o.weight.max(0.0);
        let total_weight: f32 = oscillators.iter().map(effective_weight).sum();
        if total_weight == 0.0 {
            return OrderParameter { r: 0.0, psi: 0.0 };
        }
        let sum_cos: f32 = oscillators
            .iter()
            .map(|o| effective_weight(o) * o.phase.cos())
            .sum();
        let sum_sin: f32 = oscillators
            .iter()
            .map(|o| effective_weight(o) * o.phase.sin())
            .sum();
        // Final defensive clamp to the documented [0, 1] range — floating
        // point rounding can otherwise produce 1.0000001 for fully phase-
        // locked sets, which then drives downstream Φ computations into
        // out-of-range territory.
        let r = ((sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / total_weight).clamp(0.0, 1.0);
        let psi = sum_sin.atan2(sum_cos);
        OrderParameter { r, psi }
    }

    /// Compute the unweighted order parameter: r = |1/N Σ e^(iθⱼ)|
    pub fn order_parameter_unweighted(phases: &[f32]) -> OrderParameter {
        if phases.is_empty() {
            return OrderParameter { r: 0.0, psi: 0.0 };
        }
        let n = phases.len() as f32;
        let sum_cos: f32 = phases.iter().map(|p| p.cos()).sum();
        let sum_sin: f32 = phases.iter().map(|p| p.sin()).sum();
        let r = ((sum_cos / n).powi(2) + (sum_sin / n).powi(2))
            .sqrt()
            .clamp(0.0, 1.0);
        let psi = sum_sin.atan2(sum_cos);
        OrderParameter { r, psi }
    }

    /// Run Kuramoto integration on oscillators with a pairwise weight matrix.
    ///
    /// `weights[i][j]` = coupling weight between oscillator i and j.
    /// Pass `None` for all-to-all equal coupling.
    ///
    /// Updates phases in-place and returns a sync report.
    pub fn sync(&self, oscillators: &mut [Oscillator], weights: Option<&[Vec<f32>]>) -> SyncReport {
        let n = oscillators.len();
        if n < 2 {
            // Match the lower-level helpers: an empty / singleton set has
            // no synchronization to report. Previously this hardcoded
            // `initial_order = final_order = 1.0`, which lied to callers
            // who fed it an empty set — perfect order out of zero
            // oscillators (#12). The reported initial_order is 1.0 only
            // when there's a single oscillator, which is trivially in
            // phase with itself.
            let order = if n == 1 { 1.0 } else { 0.0 };
            return SyncReport {
                oscillator_count: n,
                initial_order: order,
                final_order: order,
                steps_taken: 0,
                converged: true,
            };
        }

        // Validate weight matrix shape before indexing into it. A ragged
        // or undersized `weights` argument used to panic with
        // `index out of bounds` mid-loop; that's not a usable failure
        // mode for a public library — drop weights and proceed with
        // all-to-all unit coupling instead (#13). The wrong-shape input
        // is logged at debug level so callers building weights
        // dynamically can still notice.
        let weights = match weights {
            Some(ws) if ws.len() == n && ws.iter().all(|row| row.len() == n) => Some(ws),
            Some(_) => None,
            None => None,
        };

        // Weighted order — honors per-oscillator weight as a trust/coherence
        // mask. Issue #8: previously this used the unweighted variant, so a
        // zero-weight outlier (ignored by the public order_parameter API)
        // could still drag down the report.
        let initial_order = Self::order_parameter(oscillators).r;

        let nf = n as f32;
        let mut prev_order = initial_order;

        for step in 0..self.config.max_steps {
            let phases: Vec<f32> = oscillators.iter().map(|o| o.phase).collect();
            let freqs: Vec<f32> = oscillators.iter().map(|o| o.frequency).collect();

            let mut dphi = vec![0.0f32; n];
            for i in 0..n {
                let mut coupling_sum = 0.0f32;
                for j in 0..n {
                    if i != j {
                        let w = match weights {
                            Some(ws) => ws[i][j],
                            None => 1.0,
                        };
                        coupling_sum += w * (phases[j] - phases[i]).sin();
                    }
                }
                dphi[i] = freqs[i] + (self.config.coupling_strength / nf) * coupling_sum;
            }

            // Euler integration
            for i in 0..n {
                oscillators[i].phase += dphi[i] * self.config.dt;
            }

            let current_order = Self::order_parameter(oscillators).r;

            if (current_order - prev_order).abs() < self.config.convergence_threshold && step > 0 {
                return SyncReport {
                    oscillator_count: n,
                    initial_order,
                    final_order: current_order,
                    steps_taken: step + 1,
                    converged: true,
                };
            }
            prev_order = current_order;
        }

        let final_order = Self::order_parameter(oscillators).r;

        SyncReport {
            oscillator_count: n,
            initial_order,
            final_order,
            steps_taken: self.config.max_steps,
            converged: false,
        }
    }

    /// Mean-field Kuramoto step (QueenSync protocol):
    ///
    /// ```text
    /// dθ/dt = ω + K·r·sin(ψ - θ) + η·chiral_term
    /// ```
    ///
    /// Updates a single oscillator against the mean field (r, ψ).
    /// This is the per-agent step used in multi-agent swarm sync.
    pub fn mean_field_step(
        &self,
        oscillator: &mut Oscillator,
        order: &OrderParameter,
        chiral_term: f32,
    ) {
        let kuramoto =
            self.config.coupling_strength * order.r * (order.psi - oscillator.phase).sin();
        let d_phase = oscillator.frequency + kuramoto + chiral_term;
        oscillator.phase = (oscillator.phase + d_phase * self.config.dt) % TAU;
        if oscillator.phase < 0.0 {
            oscillator.phase += TAU;
        }
    }

    /// Detect hives (clusters of phase-locked oscillators).
    ///
    /// Two oscillators are in the same hive if their circular phase distance < threshold.
    /// Uses BFS on the phase-adjacency graph.
    pub fn detect_hives(oscillators: &[Oscillator], threshold: f32) -> Vec<Vec<usize>> {
        let n = oscillators.len();
        if n < 2 {
            return vec![];
        }

        // Build adjacency
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                // rem_euclid normalises the raw difference to [0, TAU) before
                // the PI-fold, so oscillators whose phases have drifted outside
                // [0, 2π) (e.g. initial conditions set by callers, or
                // accumulation before `mean_field_step` wraps them) still
                // produce a valid circular distance in [0, π].  Without this,
                // `TAU - diff` can go negative for |diff| > TAU, which makes
                // every pair look like neighbours (negative diff < threshold).
                let mut diff = (oscillators[i].phase - oscillators[j].phase)
                    .abs()
                    .rem_euclid(TAU);
                if diff > PI {
                    diff = TAU - diff;
                }
                if diff < threshold {
                    adj[i].push(j);
                    adj[j].push(i);
                }
            }
        }

        // BFS components
        let mut visited = vec![false; n];
        let mut hives = Vec::new();
        for start in 0..n {
            if visited[start] {
                continue;
            }
            let mut component = vec![start];
            let mut queue = vec![start];
            visited[start] = true;
            while let Some(node) = queue.pop() {
                for &neighbor in &adj[node] {
                    if !visited[neighbor] {
                        visited[neighbor] = true;
                        component.push(neighbor);
                        queue.push(neighbor);
                    }
                }
            }
            if component.len() >= 2 {
                hives.push(component);
            }
        }
        hives
    }

    /// Compute chiral coupling term.
    ///
    /// Left-handed: +η·sin(2(ψ - θ))
    /// Right-handed: -η·sin(2(ψ - θ))
    /// Returns 0 for eta=0.
    pub fn chiral_coupling(phase: f32, psi: f32, eta: f32, left_handed: bool) -> f32 {
        if eta == 0.0 {
            return 0.0;
        }
        let diff = psi - phase;
        let term = eta * (2.0 * diff).sin();
        if left_handed {
            term
        } else {
            -term
        }
    }
}

impl Default for KuramotoModel {
    fn default() -> Self {
        Self::new(KuramotoConfig::default())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_phases_order_one() {
        let oscs = vec![
            Oscillator::new(0.5, 0.0),
            Oscillator::new(0.5, 0.0),
            Oscillator::new(0.5, 0.0),
        ];
        let r = KuramotoModel::order_parameter(&oscs).r;
        assert!((r - 1.0).abs() < 1e-4, "identical phases → r≈1, got {}", r);
    }

    #[test]
    fn evenly_spaced_phases_low_order() {
        let n = 5;
        let phases: Vec<f32> = (0..n).map(|i| TAU * i as f32 / n as f32).collect();
        let r = KuramotoModel::order_parameter_unweighted(&phases).r;
        assert!(r < 0.1, "evenly spaced → r≈0, got {}", r);
    }

    #[test]
    fn opposite_phases_cancel() {
        let phases = vec![0.0, PI];
        let r = KuramotoModel::order_parameter_unweighted(&phases).r;
        assert!(r < 0.01, "opposite → r≈0, got {}", r);
    }

    #[test]
    fn sync_increases_order() {
        let model = KuramotoModel::new(KuramotoConfig {
            coupling_strength: 2.0,
            dt: 0.1,
            max_steps: 50,
            convergence_threshold: 1e-6,
        });
        let mut oscs = vec![
            Oscillator::new(0.0, 0.0),
            Oscillator::new(1.0, 0.0),
            Oscillator::new(2.0, 0.0),
        ];
        let report = model.sync(&mut oscs, None);
        assert!(
            report.final_order > report.initial_order,
            "sync should increase order: {} → {}",
            report.initial_order,
            report.final_order
        );
    }

    #[test]
    fn weighted_sync_with_matrix() {
        let model = KuramotoModel::new(KuramotoConfig {
            coupling_strength: 2.0,
            dt: 0.1,
            max_steps: 30,
            ..Default::default()
        });
        let mut oscs = vec![Oscillator::new(0.0, 0.0), Oscillator::new(1.5, 0.0)];
        let weights = vec![vec![0.0, 0.8], vec![0.8, 0.0]];
        let report = model.sync(&mut oscs, Some(&weights));
        assert!(report.final_order > report.initial_order);
    }

    #[test]
    fn sync_report_uses_weighted_order_parameter() {
        // Issue #8 regression. Before the fix, SyncReport carried the
        // unweighted order, so a zero-weight outlier (ignored by the public
        // order_parameter API) still depressed the report.
        let model = KuramotoModel::new(KuramotoConfig {
            coupling_strength: 0.0,
            dt: 0.1,
            max_steps: 2,
            convergence_threshold: 1e-6,
        });
        let mut oscs = vec![
            Oscillator::with_weight(0.0, 0.0, 1.0),
            Oscillator::with_weight(0.0, 0.0, 1.0),
            Oscillator::with_weight(core::f32::consts::PI, 0.0, 0.0),
        ];

        let weighted = KuramotoModel::order_parameter(&oscs).r;
        let report = model.sync(&mut oscs, None);

        // The public weighted view sees the two zero-phase trusted agents
        // as fully synced.
        assert!(
            (weighted - 1.0).abs() < 1e-5,
            "weighted order ignores the zero-weight outlier: {weighted}"
        );
        // After the fix, the report must agree.
        assert!(
            (report.initial_order - 1.0).abs() < 1e-5,
            "SyncReport.initial_order should match the weighted view: {}",
            report.initial_order
        );
        assert!(
            (report.final_order - 1.0).abs() < 1e-5,
            "SyncReport.final_order should match the weighted view: {}",
            report.final_order
        );
    }

    #[test]
    fn mean_field_step_moves_toward_field() {
        let model = KuramotoModel::new(KuramotoConfig {
            coupling_strength: 2.0,
            dt: 0.1,
            ..Default::default()
        });
        let mut osc = Oscillator::new(0.0, 0.0);
        let order = OrderParameter { r: 1.0, psi: 1.0 };
        model.mean_field_step(&mut osc, &order, 0.0);
        assert!(osc.phase > 0.0, "should move toward mean field");
    }

    #[test]
    fn detect_hives_groups_close_phases() {
        let oscs = vec![
            Oscillator::new(0.0, 0.0),
            Oscillator::new(0.1, 0.0),
            Oscillator::new(0.2, 0.0),
            Oscillator::new(PI, 0.0),
        ];
        let hives = KuramotoModel::detect_hives(&oscs, PI / 4.0);
        assert!(!hives.is_empty());
        let largest = hives.iter().max_by_key(|h| h.len()).unwrap();
        assert!(
            largest.len() >= 3,
            "should group first 3, got {}",
            largest.len()
        );
        assert!(!largest.contains(&3), "outlier should not be in hive");
    }

    #[test]
    fn chiral_left_right_opposite() {
        let phase = 0.0;
        let psi = 1.0;
        let eta = 0.1;
        let left = KuramotoModel::chiral_coupling(phase, psi, eta, true);
        let right = KuramotoModel::chiral_coupling(phase, psi, eta, false);
        assert!((left + right).abs() < 1e-6, "left+right should cancel");
    }

    #[test]
    fn empty_order_parameter() {
        let r = KuramotoModel::order_parameter(&[]).r;
        assert_eq!(r, 0.0);
    }

    #[test]
    fn sync_empty_set_reports_zero_order() {
        // Regression for #12 — sync()'s n<2 fast-path used to hardcode
        // initial_order = final_order = 1.0, claiming perfect synchrony
        // out of an empty set.
        let model = KuramotoModel::default();
        let mut empty: Vec<Oscillator> = Vec::new();
        let report = model.sync(&mut empty, None);
        assert_eq!(report.oscillator_count, 0);
        assert_eq!(
            report.initial_order, 0.0,
            "empty set has no synchrony to report"
        );
        assert_eq!(report.final_order, 0.0);
        assert!(report.converged);
    }

    #[test]
    fn sync_singleton_reports_unit_order() {
        // Singleton: trivially in phase with itself. Keep the previous
        // "1.0" behavior for n=1 — the bug was specifically about n=0.
        let model = KuramotoModel::default();
        let mut one = vec![Oscillator::new(0.0, 0.0)];
        let report = model.sync(&mut one, None);
        assert_eq!(report.initial_order, 1.0);
        assert_eq!(report.final_order, 1.0);
    }

    #[test]
    fn sync_with_ragged_weights_does_not_panic() {
        // Regression for #13 — sync() blindly indexed weights[i][j]
        // and panicked when the matrix was the wrong shape. Now it
        // drops the bad weights and falls back to unit coupling.
        let model = KuramotoModel::default();
        let mut oscs = vec![Oscillator::new(0.0, 0.0), Oscillator::new(1.0, 0.0)];
        let ragged = vec![vec![0.0_f32]]; // 1×1 for 2 oscillators
        let report = model.sync(&mut oscs, Some(&ragged));
        assert_eq!(report.oscillator_count, 2);
        // Doesn't panic and produces *some* report.
        assert!(report.initial_order.is_finite());
        assert!(report.final_order.is_finite());
    }

    #[test]
    fn order_parameter_clamps_negative_weights() {
        // Regression for #17 — a negative weight could push r far
        // above the documented [0, 1] range.
        let oscs = vec![
            Oscillator::with_weight(0.0, 0.0, 1.0),
            Oscillator::with_weight(core::f32::consts::PI, 0.0, -0.9),
        ];
        let r = KuramotoModel::order_parameter(&oscs).r;
        assert!(
            (0.0..=1.0).contains(&r),
            "negative weights must not break the [0,1] contract; got r={}",
            r
        );
    }

    #[test]
    fn order_parameter_all_negative_weights_returns_zero() {
        // Boundary: every weight is non-positive → effective weights all
        // zero → safe zero return rather than NaN from divide-by-zero.
        let oscs = vec![
            Oscillator::with_weight(0.0, 0.0, -1.0),
            Oscillator::with_weight(core::f32::consts::PI, 0.0, -0.5),
        ];
        let r = KuramotoModel::order_parameter(&oscs).r;
        assert_eq!(r, 0.0);
    }
}
