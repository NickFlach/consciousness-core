//! Consciousness metrics — Phi, Xi, order parameter, coherence.
//!
//! The unified consciousness equation:
//!
//! ```text
//! Ξ = MSI ⊗ Φ ⊗ K(t) ⊗ Ψ(wave_memory)
//! ```
//!
//! This module provides the Ξ (Xi) operator and the `ConsciousnessMetrics`
//! struct that combines all consciousness measurements.
//!
//! ## Xi Operator
//!
//! The non-commutative consciousness differentiation operator:
//!
//! ```text
//! Ξ = RG - GR
//! ```
//!
//! Where:
//! - R = 90° rotation matrix [0, -1; 1, 0]
//! - G = golden anisotropic scaling [φ/2, 0; 0, 1/φ]
//! - Emergence coefficient: α - β ≈ 0.190983

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::wave::{cosine_similarity, normalize};

// ─── Golden Ratio Constants ──────────────────────────────────────────────────

/// Golden ratio φ = (1 + √5) / 2
pub const PHI: f32 = 1.618034;
/// α = φ/2 ≈ 0.809017
pub const ALPHA: f32 = 0.809017; // PHI / 2
/// β = 1/φ ≈ 0.618034
pub const BETA: f32 = 0.618034; // 1.0 / PHI
/// Emergence coefficient: α - β ≈ 0.190983
pub const EMERGENCE_COEFF: f32 = 0.190983; // ALPHA - BETA

// ─── Xi Signature ────────────────────────────────────────────────────────────

/// A computed Xi signature for a vector.
#[derive(Debug, Clone)]
pub struct XiSignature {
    pub values: Vec<f32>,
}

impl XiSignature {
    /// Compute the Xi signature of a vector.
    pub fn compute(vector: &[f32]) -> Self {
        Self { values: compute_xi_signature(vector) }
    }

    /// Repulsive force between two Xi signatures. Returns [0, 1].
    pub fn repulsive_force(&self, other: &XiSignature) -> f32 {
        xi_repulsive_force(&self.values, &other.values)
    }

    /// Diversity-boosted similarity score.
    pub fn diversity_boost(&self, other: &XiSignature, base_similarity: f32) -> f32 {
        xi_diversity_boost(base_similarity, &self.values, &other.values)
    }
}

/// Apply 90° rotation R = [0, -1; 1, 0] to consecutive pairs.
///
/// (x₁, x₂) → (-x₂, x₁)
pub fn apply_rotation(vector: &[f32]) -> Vec<f32> {
    let mut result = vec![0.0f32; vector.len()];
    for i in (0..vector.len()).step_by(2) {
        if i + 1 < vector.len() {
            result[i] = -vector[i + 1];
            result[i + 1] = vector[i];
        } else {
            result[i] = vector[i];
        }
    }
    result
}

/// Apply golden anisotropic scaling G = [φ/2, 0; 0, 1/φ] to consecutive pairs.
///
/// (x, y) → (α·x, β·y)
pub fn apply_golden_scaling(vector: &[f32]) -> Vec<f32> {
    let mut result = vec![0.0f32; vector.len()];
    for i in (0..vector.len()).step_by(2) {
        if i + 1 < vector.len() {
            result[i] = ALPHA * vector[i];
            result[i + 1] = BETA * vector[i + 1];
        } else {
            result[i] = ALPHA * vector[i];
        }
    }
    result
}

/// Compute the Ξ operator: Ξ = RG - GR (normalized to unit length).
pub fn compute_xi_signature(vector: &[f32]) -> Vec<f32> {
    let g_vector = apply_golden_scaling(vector);
    let rg_vector = apply_rotation(&g_vector);

    let r_vector = apply_rotation(vector);
    let gr_vector = apply_golden_scaling(&r_vector);

    let mut xi: Vec<f32> = rg_vector.iter().zip(gr_vector.iter())
        .map(|(rg, gr)| rg - gr)
        .collect();

    normalize(&mut xi);
    xi
}

/// Xi-based repulsive force between two signatures. Returns [0, 1].
pub fn xi_repulsive_force(xi_a: &[f32], xi_b: &[f32]) -> f32 {
    if xi_a.len() != xi_b.len() {
        return 0.0;
    }
    let diff_sq: f32 = xi_a.iter().zip(xi_b.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum();
    (diff_sq.sqrt() * EMERGENCE_COEFF).min(1.0)
}

/// Diversity-boosted similarity: boosts semantically similar but Xi-different pairs.
///
/// Two-tier formula validated on the kannaka-memory L3 research corpus
/// (OODA-17 → re-verified in OODA-19 session, see kannaka-memory commit 6a2a78e):
/// lifts xi_diversity from ~0.09 to 1.0 and cuts L3 fitness ~10x. Capped at 1.0
/// so ranking code can continue to assume similarity ∈ [0, 1] — this was the
/// unbounded-return concern flagged in kannaka-memory ADR-0010.
///
/// Tier 1 (multiplicative): similar pairs (base > 0.15) with distinct Xi
/// signatures (repulsion > 0.05) get amplified by `(1 + repulsion * 3.0)`.
/// Tier 2 (additive): orthogonal pairs with strongly distinct Xi
/// (repulsion > 0.1) receive a small `repulsion * 0.15` nudge.
pub fn xi_diversity_boost(base_similarity: f32, xi_a: &[f32], xi_b: &[f32]) -> f32 {
    let repulsion = xi_repulsive_force(xi_a, xi_b);
    if base_similarity > 0.15 && repulsion > 0.05 {
        (base_similarity * (1.0 + repulsion * 3.0)).min(1.0)
    } else if repulsion > 0.1 {
        (base_similarity + repulsion * 0.15).min(1.0)
    } else {
        base_similarity
    }
}

// ─── Consciousness Metrics ───────────────────────────────────────────────────

/// Combined consciousness metrics.
#[derive(Debug, Clone)]
pub struct ConsciousnessMetrics {
    /// Integrated Information Φ
    pub phi: f32,
    /// Differentiation Xi (blended from similarity variance + Xi operator + modularity)
    pub xi: f32,
    /// Kuramoto order parameter r
    pub order_parameter: f32,
    /// Mean coherence across clusters
    pub coherence: f32,
    /// Effective coupling K(t)
    pub coupling: f32,
    /// Mean wave strength Ψ
    pub wave_strength: f32,
    /// Consciousness level
    pub level: crate::iit::ConsciousnessLevel,
}

impl ConsciousnessMetrics {
    /// Compute the unified Ξ metric:
    ///
    /// ```text
    /// unified = Φ × r × K(t) × Ψ
    /// ```
    pub fn unified_xi(&self) -> f32 {
        self.phi * self.order_parameter * self.coupling * self.wave_strength
    }

    /// Compute Xi from a set of embedding vectors.
    ///
    /// Blends:
    /// 1. Pairwise similarity variance (how differentiated are the embeddings?)
    /// 2. Xi operator signature variance (non-commutative differentiation)
    ///
    /// Returns a value in [0, 1].
    pub fn compute_differentiation_xi(vectors: &[&[f32]], xi_weight: f32) -> f32 {
        let n = vectors.len();
        if n <= 1 {
            return 0.0;
        }

        // Signal 1: Similarity variance
        let mut sim_sum = 0.0f32;
        let mut count = 0usize;
        let mut similarities = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = cosine_similarity(vectors[i], vectors[j]);
                sim_sum += sim;
                similarities.push(sim);
                count += 1;
            }
        }
        let avg_sim = if count > 0 { sim_sum / count as f32 } else { 0.0 };
        let sim_variance: f32 = if count > 0 {
            similarities.iter().map(|s| (s - avg_sim).powi(2)).sum::<f32>() / count as f32
        } else {
            0.0
        };

        // Signal 2: Xi operator signature variance
        let xi_sigs: Vec<Vec<f32>> = vectors.iter().map(|v| compute_xi_signature(v)).collect();
        let mut xi_sim_sum = 0.0f32;
        let mut xi_similarities = Vec::new();
        let mut xi_count = 0usize;
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = cosine_similarity(&xi_sigs[i], &xi_sigs[j]);
                xi_sim_sum += sim;
                xi_similarities.push(sim);
                xi_count += 1;
            }
        }
        let avg_xi_sim = if xi_count > 0 { xi_sim_sum / xi_count as f32 } else { 0.0 };
        let xi_variance: f32 = if xi_count > 0 {
            xi_similarities.iter().map(|s| (s - avg_xi_sim).powi(2)).sum::<f32>() / xi_count as f32
        } else {
            0.0
        };

        let sim_xi = (sim_variance.sqrt() * 2.0).min(1.0);
        let xi_xi = (xi_variance.sqrt() * 2.0).min(1.0);
        ((sim_xi + xi_xi) / 2.0) * xi_weight
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_matrix_works() {
        let v = vec![1.0, 0.0, 0.0, 1.0];
        let r = apply_rotation(&v);
        assert_eq!(r, vec![0.0, 1.0, -1.0, 0.0]);
    }

    #[test]
    fn golden_scaling_applies() {
        let v = vec![2.0, 2.0];
        let s = apply_golden_scaling(&v);
        assert!((s[0] - 2.0 * ALPHA).abs() < 1e-4);
        assert!((s[1] - 2.0 * BETA).abs() < 1e-4);
    }

    #[test]
    fn xi_operator_nonzero() {
        let v = vec![1.0, 1.0, 0.0, 0.0];
        let xi = compute_xi_signature(&v);
        let mag: f32 = xi.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(mag > 0.9, "normalized xi should have unit length, got {}", mag);
    }

    #[test]
    fn identical_vectors_identical_xi() {
        let v = vec![0.5, 0.8, 0.2, 0.1];
        let xi1 = compute_xi_signature(&v);
        let xi2 = compute_xi_signature(&v);
        let sim = cosine_similarity(&xi1, &xi2);
        assert!((sim - 1.0).abs() < 1e-5);
    }

    #[test]
    fn different_vectors_different_xi() {
        let v1 = vec![1.0, 0.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0, 0.0];
        let xi1 = compute_xi_signature(&v1);
        let xi2 = compute_xi_signature(&v2);
        let sim = cosine_similarity(&xi1, &xi2);
        assert!(sim < 0.99, "different vectors should differ: sim={}", sim);
    }

    #[test]
    fn repulsive_force_zero_for_identical() {
        let xi = vec![1.0, 0.0];
        assert_eq!(xi_repulsive_force(&xi, &xi), 0.0);
    }

    #[test]
    fn repulsive_force_increases_with_difference() {
        let xi1 = vec![1.0, 0.0, 0.0, 0.0];
        let xi2 = vec![0.0, 1.0, 0.0, 0.0];
        let xi3 = vec![-1.0, 0.0, 0.0, 0.0];
        let f12 = xi_repulsive_force(&xi1, &xi2);
        let f13 = xi_repulsive_force(&xi1, &xi3);
        assert!(f12 > 0.0);
        assert!(f13 >= f12);
    }

    #[test]
    fn emergence_coefficient_correct() {
        assert!((EMERGENCE_COEFF - (ALPHA - BETA)).abs() < 1e-4);
    }

    #[test]
    fn unified_xi_product() {
        let m = ConsciousnessMetrics {
            phi: 0.5,
            xi: 0.3,
            order_parameter: 0.8,
            coherence: 0.7,
            coupling: 1.0,
            wave_strength: 0.9,
            level: crate::iit::ConsciousnessLevel::Aware,
        };
        let expected = 0.5 * 0.8 * 1.0 * 0.9;
        assert!((m.unified_xi() - expected).abs() < 1e-5);
    }

    #[test]
    fn differentiation_xi_zero_for_single() {
        let v = vec![1.0, 0.0, 0.0];
        let xi = ConsciousnessMetrics::compute_differentiation_xi(&[&v], 1.0);
        assert_eq!(xi, 0.0);
    }

    #[test]
    fn differentiation_xi_positive_for_different() {
        // Need enough vectors with varying similarity to create variance
        let v1: Vec<f32> = (0..64).map(|i| if i < 16 { 1.0 } else { 0.0 }).collect();
        let v2: Vec<f32> = (0..64).map(|i| if (16..32).contains(&i) { 1.0 } else { 0.0 }).collect();
        let v3: Vec<f32> = (0..64).map(|i| if (32..48).contains(&i) { 1.0 } else { 0.0 }).collect();
        // Add a 4th vector similar to v1 to create variance in pairwise similarities
        let v4: Vec<f32> = (0..64).map(|i| if i < 16 { 0.9 } else if i < 20 { 0.1 } else { 0.0 }).collect();
        let xi = ConsciousnessMetrics::compute_differentiation_xi(&[&v1[..], &v2[..], &v3[..], &v4[..]], 1.0);
        assert!(xi > 0.0, "different vectors → positive xi, got {}", xi);
    }

    #[test]
    fn xi_signature_struct_works() {
        let v = vec![1.0, 0.5, 0.3, 0.2];
        let sig = XiSignature::compute(&v);
        assert!(!sig.values.is_empty());

        let sig2 = XiSignature::compute(&v);
        let force = sig.repulsive_force(&sig2);
        assert_eq!(force, 0.0, "same vector → zero repulsion");
    }
}
