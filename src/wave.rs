//! Wave memory physics — amplitude, frequency, phase, interference, decay.
//!
//! Each memory is modeled as a damped oscillator:
//!
//! ```text
//! S(t) = (A + E_retrieval) · cos(2πf·t + φ) · e^(-λt)
//! ```
//!
//! Where:
//! - A = base amplitude
//! - f = oscillation frequency
//! - φ = initial phase
//! - λ = decay rate
//! - E_retrieval = 0.05 · ln(1 + retrieval_count) — diminishing retrieval energy
//!
//! This models the Ebbinghaus forgetting curve with wave interference.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::f64::consts::PI;

/// Wave parameters governing memory strength over time.
#[derive(Debug, Clone, Copy)]
pub struct WaveParams {
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
    pub decay_rate: f32,
}

impl Default for WaveParams {
    fn default() -> Self {
        Self {
            amplitude: 1.0,
            frequency: 0.1,   // slow oscillation
            phase: 0.0,
            decay_rate: 1e-6, // very slow decay
        }
    }
}

/// A wave memory combining parameters with retrieval state.
#[derive(Debug, Clone)]
pub struct WaveMemory {
    pub params: WaveParams,
    pub retrieval_count: u32,
}

impl WaveMemory {
    pub fn new(params: WaveParams) -> Self {
        Self { params, retrieval_count: 0 }
    }

    /// Compute effective strength at a given age in seconds.
    pub fn strength(&self, age_seconds: f64) -> f32 {
        compute_strength_with_retrieval(&self.params, age_seconds, self.retrieval_count)
    }

    /// Record a retrieval, boosting future strength.
    pub fn record_retrieval(&mut self) {
        self.retrieval_count += 1;
    }
}

impl Default for WaveMemory {
    fn default() -> Self {
        Self::new(WaveParams::default())
    }
}

/// Compute effective strength: S(t) = A · cos(2πf·t + φ) · e^(-λt)
pub fn compute_strength(params: &WaveParams, age_seconds: f64) -> f32 {
    compute_strength_with_retrieval(params, age_seconds, 0)
}

/// Compute effective strength with retrieval energy:
///
/// S(t) = (A + 0.05·ln(1 + retrieval_count)) · cos(2πf·t + φ) · e^(-λt)
///
/// Each retrieval adds diminishing energy, making retrieval a generative
/// term in the dx/dt = f(x) - λx dynamical system.
pub fn compute_strength_with_retrieval(
    params: &WaveParams,
    age_seconds: f64,
    retrieval_count: u32,
) -> f32 {
    let retrieval_energy = 0.05 * (1.0 + retrieval_count as f64).ln();
    let a = params.amplitude as f64 + retrieval_energy;
    let f = params.frequency as f64;
    let phi = params.phase as f64;
    let lambda = params.decay_rate as f64;

    let wave = (2.0 * PI * f * age_seconds + phi).cos();
    let decay = (-lambda * age_seconds).exp();
    (a * wave * decay) as f32
}

/// Compute wave interference between two memories.
///
/// Constructive interference when phases align, destructive when opposed.
/// Returns a factor in [-1, 1].
pub fn interference(a: &WaveParams, b: &WaveParams, t: f64) -> f32 {
    let phase_a = 2.0 * PI * a.frequency as f64 * t + a.phase as f64;
    let phase_b = 2.0 * PI * b.frequency as f64 * t + b.phase as f64;
    (phase_a - phase_b).cos() as f32
}

/// Cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na * nb)
}

/// Normalize a vector to unit length in-place.
pub fn normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

/// Dot product of two vectors.
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strength_decays_over_time() {
        let params = WaveParams {
            amplitude: 1.0,
            frequency: 0.0, // no oscillation, pure decay
            phase: 0.0,
            decay_rate: 0.01,
        };
        let s0 = compute_strength(&params, 0.0);
        let s1 = compute_strength(&params, 100.0);
        let s2 = compute_strength(&params, 1000.0);
        assert!(s0 > s1);
        assert!(s1 > s2);
        assert!((s0 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn retrieval_boosts_strength() {
        let params = WaveParams {
            amplitude: 1.0,
            frequency: 0.0,
            phase: 0.0,
            decay_rate: 0.001,
        };
        let s0 = compute_strength_with_retrieval(&params, 100.0, 0);
        let s10 = compute_strength_with_retrieval(&params, 100.0, 10);
        assert!(s10 > s0, "retrieval should boost: {} vs {}", s10, s0);
    }

    #[test]
    fn retrieval_diminishing_returns() {
        let params = WaveParams { frequency: 0.0, decay_rate: 0.0, ..Default::default() };
        let boost_low = compute_strength_with_retrieval(&params, 0.0, 10)
            - compute_strength_with_retrieval(&params, 0.0, 1);
        let boost_high = compute_strength_with_retrieval(&params, 0.0, 110)
            - compute_strength_with_retrieval(&params, 0.0, 100);
        assert!(boost_low > boost_high, "diminishing returns: {} vs {}", boost_low, boost_high);
    }

    #[test]
    fn zero_retrieval_matches_original() {
        let params = WaveParams { frequency: 0.1, phase: 0.5, decay_rate: 0.001, ..Default::default() };
        let s = compute_strength(&params, 500.0);
        let s0 = compute_strength_with_retrieval(&params, 500.0, 0);
        assert!((s - s0).abs() < 1e-6);
    }

    #[test]
    fn wave_memory_tracks_retrievals() {
        let mut wm = WaveMemory::default();
        let s0 = wm.strength(0.0);
        wm.record_retrieval();
        wm.record_retrieval();
        let s1 = wm.strength(0.0);
        assert!(s1 > s0);
    }

    #[test]
    fn interference_constructive_when_aligned() {
        let a = WaveParams { phase: 0.0, frequency: 1.0, ..Default::default() };
        let b = WaveParams { phase: 0.0, frequency: 1.0, ..Default::default() };
        let i = interference(&a, &b, 0.0);
        assert!((i - 1.0).abs() < 1e-5, "aligned → constructive, got {}", i);
    }

    #[test]
    fn interference_destructive_when_opposed() {
        let a = WaveParams { phase: 0.0, frequency: 1.0, ..Default::default() };
        let b = WaveParams { phase: core::f32::consts::PI, frequency: 1.0, ..Default::default() };
        let i = interference(&a, &b, 0.0);
        assert!((i - (-1.0)).abs() < 1e-5, "opposed → destructive, got {}", i);
    }

    #[test]
    fn cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn normalize_produces_unit_vector() {
        let mut v = vec![3.0, 4.0];
        normalize(&mut v);
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn empty_vectors_similarity_zero() {
        assert_eq!(cosine_similarity(&[], &[1.0]), 0.0);
        assert_eq!(cosine_similarity(&[1.0], &[]), 0.0);
    }
}
