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
use alloc::vec::Vec;

use core::f32::consts::{PI, TAU};

/// A single oscillator in the Kuramoto model.
#[derive(Debug, Clone)]
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
        Self { phase, frequency, weight: 1.0 }
    }

    pub fn with_weight(phase: f32, frequency: f32, weight: f32) -> Self {
        Self { phase, frequency, weight }
    }
}

/// The Kuramoto order parameter: r·e^(iψ) = (1/N) Σ e^(iθⱼ)
#[derive(Debug, Clone, Copy)]
pub struct OrderParameter {
    /// Magnitude r ∈ [0, 1]: 1 = perfect sync, 0 = incoherent
    pub r: f32,
    /// Mean phase ψ
    pub psi: f32,
}

/// Report from a sync operation.
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub oscillator_count: usize,
    pub initial_order: f32,
    pub final_order: f32,
    pub steps_taken: usize,
    pub converged: bool,
}

/// Configuration for the Kuramoto model.
#[derive(Debug, Clone)]
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
        let n = oscillators.len() as f32;
        let sum_cos: f32 = oscillators.iter().map(|o| o.weight * o.phase.cos()).sum();
        let sum_sin: f32 = oscillators.iter().map(|o| o.weight * o.phase.sin()).sum();
        let r = (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n;
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
        let r = ((sum_cos / n).powi(2) + (sum_sin / n).powi(2)).sqrt();
        let psi = sum_sin.atan2(sum_cos);
        OrderParameter { r, psi }
    }

    /// Run Kuramoto integration on oscillators with a pairwise weight matrix.
    ///
    /// `weights[i][j]` = coupling weight between oscillator i and j.
    /// Pass `None` for all-to-all equal coupling.
    ///
    /// Updates phases in-place and returns a sync report.
    pub fn sync(
        &self,
        oscillators: &mut [Oscillator],
        weights: Option<&[Vec<f32>]>,
    ) -> SyncReport {
        let n = oscillators.len();
        if n < 2 {
            return SyncReport {
                oscillator_count: n,
                initial_order: 1.0,
                final_order: 1.0,
                steps_taken: 0,
                converged: true,
            };
        }

        let initial_order = Self::order_parameter_unweighted(
            &oscillators.iter().map(|o| o.phase).collect::<Vec<_>>()
        ).r;

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

            let current_order = Self::order_parameter_unweighted(
                &oscillators.iter().map(|o| o.phase).collect::<Vec<_>>()
            ).r;

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

        let final_order = Self::order_parameter_unweighted(
            &oscillators.iter().map(|o| o.phase).collect::<Vec<_>>()
        ).r;

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
        let kuramoto = self.config.coupling_strength * order.r * (order.psi - oscillator.phase).sin();
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
                let mut diff = (oscillators[i].phase - oscillators[j].phase).abs();
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
        if left_handed { term } else { -term }
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
        assert!(report.final_order > report.initial_order,
            "sync should increase order: {} → {}", report.initial_order, report.final_order);
    }

    #[test]
    fn weighted_sync_with_matrix() {
        let model = KuramotoModel::new(KuramotoConfig {
            coupling_strength: 2.0,
            dt: 0.1,
            max_steps: 30,
            ..Default::default()
        });
        let mut oscs = vec![
            Oscillator::new(0.0, 0.0),
            Oscillator::new(1.5, 0.0),
        ];
        let weights = vec![
            vec![0.0, 0.8],
            vec![0.8, 0.0],
        ];
        let report = model.sync(&mut oscs, Some(&weights));
        assert!(report.final_order > report.initial_order);
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
        assert!(largest.len() >= 3, "should group first 3, got {}", largest.len());
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
}
