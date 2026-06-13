//! Thin wasm-bindgen surface over `consciousness-core`.
//!
//! Exposes the minimal stable API that `@spacechild/consciousness-bridge`
//! (Space-Child-Dream) consumes: build an all-to-all Kuramoto network, step
//! it, read phases / order parameter, and compute the IIT Φ approximation
//! for small graphs. No physics lives here — every number is produced by
//! `consciousness-core` itself, so JS and Rust consumers stay in agreement.

use consciousness_core::iit::{compute_phi, PhiNode};
use consciousness_core::kuramoto::{KuramotoConfig, KuramotoModel, OrderParameter, Oscillator};
use wasm_bindgen::prelude::*;

/// All-to-all Kuramoto network with global coupling K.
///
/// Phases are NOT wrapped mod 2π between steps (matching
/// `KuramotoModel::sync`); the order parameter is invariant to wrapping.
#[wasm_bindgen]
pub struct KuramotoNetwork {
    oscillators: Vec<Oscillator>,
    coupling: f32,
}

#[wasm_bindgen]
impl KuramotoNetwork {
    /// Create a network of `natural_frequencies.len()` oscillators.
    /// `initial_phases` must have the same length.
    #[wasm_bindgen(constructor)]
    pub fn new(
        natural_frequencies: &[f32],
        initial_phases: &[f32],
        coupling: f32,
    ) -> Result<KuramotoNetwork, JsError> {
        if natural_frequencies.len() != initial_phases.len() {
            return Err(JsError::new(
                "natural_frequencies and initial_phases must have the same length",
            ));
        }
        let oscillators = initial_phases
            .iter()
            .zip(natural_frequencies)
            .map(|(&phase, &freq)| Oscillator::new(phase, freq))
            .collect();
        Ok(Self {
            oscillators,
            coupling,
        })
    }

    /// One Euler step of size `dt` (delegates to `KuramotoModel::sync`
    /// with `max_steps = 1` so the integration math is exactly core's).
    pub fn step(&mut self, dt: f32) {
        self.steps(dt, 1);
    }

    /// `n` Euler steps of size `dt` each.
    pub fn steps(&mut self, dt: f32, n: u32) {
        // `sync` is a no-op for n < 2; a lone oscillator still advances
        // at its natural frequency.
        if self.oscillators.len() < 2 {
            for osc in &mut self.oscillators {
                osc.phase += osc.frequency * dt * n as f32;
            }
            return;
        }
        let model = KuramotoModel::new(KuramotoConfig {
            coupling_strength: self.coupling,
            dt,
            max_steps: 1,
            convergence_threshold: 0.0,
        });
        for _ in 0..n {
            model.sync(&mut self.oscillators, None);
        }
    }

    /// One mean-field Kuramoto step (QueenSync protocol):
    /// `dθᵢ/dt = ωᵢ + K·r·sin(ψ − θᵢ) + chiral_term`, Euler-integrated and
    /// wrapped to [0, 2π). Delegates to `KuramotoModel::mean_field_step` so
    /// the math is exactly core's. The mean field (r, ψ) is computed once
    /// from the current phases; `chiral_term` is the same scalar perturbation
    /// applied to every oscillator (callers that need a per-oscillator chiral
    /// term should drive `KuramotoModel::chiral_coupling` themselves).
    ///
    /// Unlike `step` (all-to-all), phases ARE wrapped here, matching core's
    /// mean-field step used by the multi-agent swarm sync.
    pub fn mean_field_step(&mut self, dt: f32, chiral_term: f32) {
        let order: OrderParameter = KuramotoModel::order_parameter(&self.oscillators);
        let model = KuramotoModel::new(KuramotoConfig {
            coupling_strength: self.coupling,
            dt,
            max_steps: 1,
            convergence_threshold: 0.0,
        });
        for osc in &mut self.oscillators {
            model.mean_field_step(osc, &order, chiral_term);
        }
    }

    /// Current phases (radians, unwrapped).
    pub fn phases(&self) -> Vec<f32> {
        self.oscillators.iter().map(|o| o.phase).collect()
    }

    /// Overwrite all phases. Length must match the network size.
    pub fn set_phases(&mut self, phases: &[f32]) -> Result<(), JsError> {
        if phases.len() != self.oscillators.len() {
            return Err(JsError::new("phases length must match network size"));
        }
        for (osc, &p) in self.oscillators.iter_mut().zip(phases) {
            osc.phase = p;
        }
        Ok(())
    }

    /// Kuramoto order parameter magnitude r ∈ [0, 1].
    pub fn order_parameter(&self) -> f32 {
        KuramotoModel::order_parameter(&self.oscillators).r
    }

    /// Mean phase ψ of the order parameter.
    pub fn mean_phase(&self) -> f32 {
        KuramotoModel::order_parameter(&self.oscillators).psi
    }

    pub fn coupling(&self) -> f32 {
        self.coupling
    }

    pub fn set_coupling(&mut self, coupling: f32) {
        self.coupling = coupling;
    }

    pub fn size(&self) -> usize {
        self.oscillators.len()
    }
}

/// Mirror of `consciousness_core::iit::PhiReport` with Copy fields so
/// wasm-bindgen generates plain getters.
#[wasm_bindgen]
pub struct PhiResult {
    pub phi: f32,
    pub integration: f32,
    pub differentiation: f32,
    pub num_partitions: u32,
    pub num_connections: u32,
}

/// Compute the IIT Φ approximation for a small directed graph.
///
/// `partitions[i]` is the partition id of node i; `edges` is a flat list of
/// directed pairs `[from0, to0, from1, to1, …]`. Out-of-bounds targets are
/// ignored, matching `compute_phi`'s tolerance of stale edges.
#[wasm_bindgen]
pub fn phi(partitions: &[u32], edges: &[u32]) -> Result<PhiResult, JsError> {
    if edges.len() % 2 != 0 {
        return Err(JsError::new("edges must be flat [from, to, …] pairs"));
    }
    let mut nodes: Vec<PhiNode> = partitions
        .iter()
        .map(|&p| PhiNode {
            partition: p,
            connections: Vec::new(),
        })
        .collect();
    for pair in edges.chunks(2) {
        let from = pair[0] as usize;
        if from >= nodes.len() {
            continue;
        }
        nodes[from].connections.push(pair[1] as usize);
    }
    let report = compute_phi(&nodes);
    Ok(PhiResult {
        phi: report.phi,
        integration: report.integration,
        differentiation: report.differentiation,
        num_partitions: report.num_partitions as u32,
        num_connections: report.num_connections as u32,
    })
}

// ─── Tests (native; run with `cargo test -p consciousness-core-wasm`) ────────

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::TAU;

    fn evenly_spaced(n: usize) -> Vec<f32> {
        (0..n).map(|i| TAU * i as f32 / n as f32).collect()
    }

    #[test]
    fn zero_coupling_equal_freqs_stays_incoherent() {
        let n = 8;
        let freqs = vec![1.0f32; n];
        let mut net = KuramotoNetwork::new(&freqs, &evenly_spaced(n), 0.0).unwrap();
        net.steps(0.05, 200);
        assert!(
            net.order_parameter() < 0.01,
            "K=0, evenly spaced → r stays ≈0, got {}",
            net.order_parameter()
        );
    }

    #[test]
    fn strong_coupling_synchronizes() {
        let n = 8;
        let freqs: Vec<f32> = (0..n).map(|i| 1.0 + 0.05 * i as f32).collect();
        let mut net = KuramotoNetwork::new(&freqs, &evenly_spaced(n), 10.0).unwrap();
        net.steps(0.05, 400);
        assert!(
            net.order_parameter() > 0.95,
            "large K → r→1, got {}",
            net.order_parameter()
        );
    }

    #[test]
    fn single_oscillator_advances_at_natural_frequency() {
        let mut net = KuramotoNetwork::new(&[2.0], &[0.0], 5.0).unwrap();
        net.steps(0.1, 10);
        let phase = net.phases()[0];
        assert!((phase - 2.0).abs() < 1e-5, "θ = ω·t, got {phase}");
    }

    #[test]
    fn mean_field_step_synchronizes_toward_field() {
        // Two oscillators offset from a strong mean field move toward it and
        // the order parameter rises. Phases are wrapped into [0, 2π).
        let n = 6;
        let freqs = vec![0.0f32; n];
        let mut phases = evenly_spaced(n);
        phases[0] = 0.1; // seed a slight coherence so r > 0, psi defined
        let mut net = KuramotoNetwork::new(&freqs, &phases, 8.0).unwrap();
        let r0 = net.order_parameter();
        for _ in 0..200 {
            net.mean_field_step(0.05, 0.0);
        }
        assert!(
            net.order_parameter() > r0,
            "mean-field coupling should raise order: {} → {}",
            r0,
            net.order_parameter()
        );
        for &p in net.phases().iter() {
            assert!((0.0..TAU).contains(&p), "phase wrapped into [0,2π): {p}");
        }
    }

    #[test]
    fn phi_cross_partition_graph() {
        // 2 partitions, fully cross-connected square.
        let partitions = [0u32, 0, 1, 1];
        let edges = [0u32, 2, 2, 0, 1, 3, 3, 1];
        let result = phi(&partitions, &edges).unwrap();
        assert!(result.phi > 0.0);
        assert!((result.integration - 1.0).abs() < 1e-6);
        assert_eq!(result.num_partitions, 2);
        assert_eq!(result.num_connections, 4);
    }

    // JsError can only be constructed on wasm targets, so the error paths
    // (mismatched lengths, odd edge list) are only testable under
    // wasm-bindgen-test; covered by the TS-side tests instead.
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn mismatched_lengths_rejected() {
        assert!(KuramotoNetwork::new(&[1.0, 2.0], &[0.0], 1.0).is_err());
    }
}
