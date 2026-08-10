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
use crate::math_ext::F32Ext;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::wave::cosine_similarity;

// ─── Golden Ratio Constants ──────────────────────────────────────────────────

/// Golden ratio φ = (1 + √5) / 2
pub const PHI: f32 = 1.618034;
/// α = φ/2 ≈ 0.809017
pub const ALPHA: f32 = 0.809017; // PHI / 2
/// β = 1/φ ≈ 0.618034
pub const BETA: f32 = 0.618034; // 1.0 / PHI
/// Emergence coefficient: α - β ≈ 0.190983
pub const EMERGENCE_COEFF: f32 = 0.190983; // ALPHA - BETA

// ─── Errors ──────────────────────────────────────────────────────────────────

/// Why a differentiation-Xi computation could not be performed.
///
/// These are caller errors — a malformed embedding batch, not a system
/// state worth reporting a number for. They exist because
/// [`crate::wave::cosine_similarity`] returns `0.0` for incomparable
/// vectors, which reads identically to "orthogonal": without this,
/// mixed-dimension input produced a plausible and completely wrong
/// `xi = 0.0`. (#60)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum XiError {
    /// The batch mixed embedding dimensions.
    MixedDimensions {
        /// Length of the first vector, taken as the batch's dimension.
        expected: usize,
        /// Length of the first vector that disagreed.
        found: usize,
        /// Index of that vector in the batch.
        index: usize,
    },
    /// The batch consisted of zero-length vectors.
    EmptyVectors,
}

impl core::fmt::Display for XiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MixedDimensions {
                expected,
                found,
                index,
            } => write!(
                f,
                "embedding batch mixes dimensions: expected {expected}, \
                 found {found} at index {index}"
            ),
            Self::EmptyVectors => write!(f, "embedding batch contains zero-length vectors"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for XiError {}

// ─── Xi Signature ────────────────────────────────────────────────────────────

/// A computed Xi signature for a vector.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XiSignature {
    pub values: Vec<f32>,
}

impl XiSignature {
    /// Compute the Xi signature of a vector.
    pub fn compute(vector: &[f32]) -> Self {
        Self {
            values: compute_xi_signature(vector),
        }
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

/// Compute the Ξ operator: nonlinear commutator tanh(R)·G - tanh(G)·R
/// (normalized to unit length).
///
/// The original linear commutator RG−GR collapses to a constant-scaled
/// pair-swap (see xi-operator-audit.md), producing zero independent
/// information.  Applying element-wise tanh before the cross-multiplication
/// breaks this cancellation: since tanh(ax) ≠ a·tanh(x), the commutator
/// no longer factors into a trivial isometry.
pub fn compute_xi_signature(vector: &[f32]) -> Vec<f32> {
    let rotated = apply_rotation(vector);
    let scaled = apply_golden_scaling(vector);

    // Nonlinear transforms — tanh breaks the linearity that made
    // the old commutator degenerate.
    let nl_rotated: Vec<f32> = rotated.iter().map(|x| x.tanh()).collect();
    let nl_scaled: Vec<f32> = scaled.iter().map(|x| x.tanh()).collect();

    // Commutator of nonlinear transforms:
    //   tanh(R(v)) ⊙ G(v)  −  tanh(G(v)) ⊙ R(v)
    let mut xi: Vec<f32> = nl_rotated
        .iter()
        .zip(scaled.iter())
        .zip(nl_scaled.iter().zip(rotated.iter()))
        .map(|((nr, s), (ns, r))| nr * s - ns * r)
        .collect();

    // Normalize to unit sphere
    let norm: f32 = xi.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        for x in xi.iter_mut() {
            *x /= norm;
        }
    }
    xi
}

/// Xi-based repulsive force between two signatures. Returns [0, 1].
pub fn xi_repulsive_force(xi_a: &[f32], xi_b: &[f32]) -> f32 {
    if xi_a.len() != xi_b.len() {
        return 0.0;
    }
    let diff_sq: f32 = xi_a
        .iter()
        .zip(xi_b.iter())
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
    let boosted = if base_similarity > 0.15 && repulsion > 0.05 {
        base_similarity * (1.0 + repulsion * 3.0)
    } else if repulsion > 0.1 {
        base_similarity + repulsion * 0.15
    } else {
        base_similarity
    };
    // Clamp BOTH ends, not just the ceiling (#54). `cosine_similarity`
    // legitimately returns down to -1.0 for opposed vectors, and the old
    // `.min(1.0)` let that negative fall straight through — so a helper
    // documented as "similarity ∈ [0, 1]" handed ranking code a -1.0.
    // Opposed vectors are "not similar"; the floor is 0, not a negative
    // score that inverts ordering heuristics downstream.
    boosted.clamp(0.0, 1.0)
}

// ─── Consciousness Metrics ───────────────────────────────────────────────────

/// Combined consciousness metrics.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Compute the unified consciousness metric:
    ///
    /// ```text
    /// unified = Φ × r × K(t) × Ψ
    /// ```
    ///
    /// **This deliberately does not include [`Self::xi`]** — see
    /// [`Self::unified_with_differentiation`] if you want differentiation
    /// folded in.
    ///
    /// The two are easy to conflate because the crate uses "Xi" for two
    /// different things: the `Ξ` of the unified equation
    /// (`Ξ = MSI ⊗ Φ ⊗ K(t) ⊗ Ψ`), whose leading term is Multi-Scale
    /// Integration, and the `xi` field on this struct, which is the
    /// *differentiation* Xi from [`Self::compute_differentiation_xi`].
    /// This method computes the former. It was named `unified_xi`, which
    /// read as a promise about the latter (#36).
    pub fn unified_consciousness(&self) -> f32 {
        self.phi * self.order_parameter * self.coupling * self.wave_strength
    }

    /// Deprecated alias for [`Self::unified_consciousness`].
    ///
    /// The formula is unchanged — only the name was wrong. Kept so the
    /// rename does not break downstream on upgrade.
    #[deprecated(
        since = "0.5.0",
        note = "renamed to `unified_consciousness`: this never included the `xi` field, \
                and the old name read as a promise that it did. Use \
                `unified_with_differentiation()` if you want differentiation included."
    )]
    pub fn unified_xi(&self) -> f32 {
        self.unified_consciousness()
    }

    /// [`Self::unified_consciousness`] scaled by the differentiation Xi
    /// carried on this struct:
    ///
    /// ```text
    /// unified × xi = Φ × r × K(t) × Ψ × Ξ_differentiation
    /// ```
    ///
    /// This is a **different metric**, not a corrected one — reach for it
    /// when you want a reading that collapses if the system stops being
    /// differentiated, which `unified_consciousness` deliberately does not
    /// do. Ranges `[0, 1]` given in-range inputs, since `xi` is itself
    /// bounded to `[0, 1]` by `compute_differentiation_xi`. (#36)
    pub fn unified_with_differentiation(&self) -> f32 {
        self.unified_consciousness() * self.xi
    }

    /// Compute Xi from a set of embedding vectors.
    ///
    /// Blends:
    /// 1. Pairwise similarity spread (how differentiated are the embeddings?)
    /// 2. Xi operator signature spread (non-commutative differentiation)
    ///
    /// Returns a value in `[0, 1]`, or [`XiError`] if the input is not a
    /// well-formed embedding batch.
    ///
    /// # The `n == 2` case
    ///
    /// With exactly two vectors there is a single pair, and the variance of
    /// one sample about its own mean is identically zero — so the spread
    /// definition returned 0.0 ("no differentiation") for *any* two
    /// vectors, however different (#35). For `n == 2` both signals are
    /// therefore the normalized cosine distance of the pair,
    /// `(1 - cos_sim) / 2`, which maps the legal cosine range onto
    /// `[0, 1]`: identical → 0.0, orthogonal → 0.5, opposed → 1.0.
    ///
    /// This is deliberately piecewise. `n == 2` measures *distance* while
    /// `n >= 3` measures *spread*, so Xi can move discontinuously as a
    /// corpus grows from two to three vectors. The fully continuous
    /// alternative (mean pairwise distance throughout) would rewrite every
    /// historical Xi value, which is the more expensive of the two costs
    /// while anything compares Xi across time.
    ///
    /// # Errors
    ///
    /// Returns [`XiError::MixedDimensions`] if the vectors are not all the
    /// same length, and [`XiError::EmptyVectors`] if they are zero-length.
    /// Both are caller errors: `cosine_similarity` yields `0.0` for
    /// incomparable inputs, which this function would otherwise read as
    /// genuine semantic flatness and report as a plausible, wrong
    /// `xi = 0.0` (#60).
    pub fn compute_differentiation_xi(vectors: &[&[f32]], xi_weight: f32) -> Result<f32, XiError> {
        let n = vectors.len();
        if n <= 1 {
            return Ok(0.0);
        }

        // Validate the batch before any similarity is taken. Done up front
        // so a ragged batch is rejected outright rather than silently
        // contributing 0.0 similarities from the mismatched pairs (#60).
        let dim = vectors[0].len();
        if dim == 0 {
            return Err(XiError::EmptyVectors);
        }
        if let Some(pos) = vectors.iter().position(|v| v.len() != dim) {
            return Err(XiError::MixedDimensions {
                expected: dim,
                found: vectors[pos].len(),
                index: pos,
            });
        }

        // n == 2: one pair, so spread is degenerate. Use normalized cosine
        // distance for both signals instead. (#35)
        if n == 2 {
            let normalized_distance =
                |a: &[f32], b: &[f32]| ((1.0 - cosine_similarity(a, b)) / 2.0).clamp(0.0, 1.0);
            let sim_xi = normalized_distance(vectors[0], vectors[1]);
            let xi_a = compute_xi_signature(vectors[0]);
            let xi_b = compute_xi_signature(vectors[1]);
            let xi_xi = normalized_distance(&xi_a, &xi_b);
            return Ok((((sim_xi + xi_xi) / 2.0) * xi_weight).clamp(0.0, 1.0));
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
        let avg_sim = if count > 0 {
            sim_sum / count as f32
        } else {
            0.0
        };
        let sim_variance: f32 = if count > 0 {
            similarities
                .iter()
                .map(|s| (s - avg_sim).powi(2))
                .sum::<f32>()
                / count as f32
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
        let avg_xi_sim = if xi_count > 0 {
            xi_sim_sum / xi_count as f32
        } else {
            0.0
        };
        let xi_variance: f32 = if xi_count > 0 {
            xi_similarities
                .iter()
                .map(|s| (s - avg_xi_sim).powi(2))
                .sum::<f32>()
                / xi_count as f32
        } else {
            0.0
        };

        let sim_xi = (sim_variance.sqrt() * 2.0).min(1.0);
        let xi_xi = (xi_variance.sqrt() * 2.0).min(1.0);
        // Clamp the weighted result so the documented [0,1] return range
        // holds for any xi_weight. With xi_weight > 1 the average could
        // exceed 1 even though each component is individually clamped.
        Ok((((sim_xi + xi_xi) / 2.0) * xi_weight).clamp(0.0, 1.0))
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
        assert!(
            mag > 0.9,
            "normalized xi should have unit length, got {}",
            mag
        );
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

    fn metrics_fixture(xi: f32) -> ConsciousnessMetrics {
        ConsciousnessMetrics {
            phi: 0.5,
            xi,
            order_parameter: 0.8,
            coherence: 0.7,
            coupling: 1.0,
            wave_strength: 0.9,
            level: crate::iit::ConsciousnessLevel::Aware,
        }
    }

    #[test]
    fn unified_consciousness_product() {
        let m = metrics_fixture(0.3);
        let expected = 0.5 * 0.8 * 1.0 * 0.9;
        assert!((m.unified_consciousness() - expected).abs() < 1e-5);
    }

    #[test]
    fn unified_consciousness_deliberately_ignores_xi() {
        // #36 — the formula is Φ × r × K × Ψ and does NOT include the
        // differentiation xi. Pinning that as intent rather than accident,
        // so a future reader does not "fix" it by multiplying xi in.
        let low = metrics_fixture(0.01);
        let high = metrics_fixture(0.99);
        assert_eq!(
            low.unified_consciousness(),
            high.unified_consciousness(),
            "unified_consciousness must not vary with xi"
        );
    }

    #[test]
    fn unified_with_differentiation_does_vary_with_xi() {
        // #36 — the new accessor is the one that folds differentiation in.
        let low = metrics_fixture(0.2);
        let high = metrics_fixture(0.8);
        assert!(
            high.unified_with_differentiation() > low.unified_with_differentiation(),
            "unified_with_differentiation must track xi"
        );
        let m = metrics_fixture(0.5);
        assert!((m.unified_with_differentiation() - m.unified_consciousness() * 0.5).abs() < 1e-6);
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_unified_xi_alias_matches_new_name() {
        // The rename must not change the number for anyone still on the
        // old name (#36).
        let m = metrics_fixture(0.3);
        assert_eq!(m.unified_xi(), m.unified_consciousness());
    }

    #[test]
    fn differentiation_xi_zero_for_single() {
        let v = vec![1.0, 0.0, 0.0];
        let xi = ConsciousnessMetrics::compute_differentiation_xi(&[&v], 1.0).unwrap();
        assert_eq!(xi, 0.0);
    }

    #[test]
    fn two_vectors_report_positive_differentiation() {
        // Regression for #35 — the variance of one sample about its own
        // mean is identically zero, so ANY two vectors returned 0.0.
        let a = vec![1.0f32, 0.0, 0.0, 0.0];
        let b = vec![0.0f32, 1.0, 0.0, 0.0];
        let xi = ConsciousnessMetrics::compute_differentiation_xi(&[&a, &b], 1.0).unwrap();
        assert!(
            xi > 0.0,
            "two clearly different vectors must not read as undifferentiated: {xi}"
        );
        assert!((0.0..=1.0).contains(&xi), "still in range: {xi}");
    }

    #[test]
    fn two_identical_vectors_report_zero_differentiation() {
        // The other end of the #35 fix: the n==2 branch must not manufacture
        // differentiation where there is none. cos_sim = 1 → distance 0.
        let a = vec![1.0f32, 0.5, 0.25, 0.125];
        let xi = ConsciousnessMetrics::compute_differentiation_xi(&[&a, &a], 1.0).unwrap();
        assert!(
            xi.abs() < 1e-6,
            "identical vectors → ~0 differentiation: {xi}"
        );
    }

    #[test]
    fn two_vector_differentiation_is_ordered_by_distance() {
        // Anti-vacuous: the n==2 branch must actually track how different
        // the pair is, not just return some positive constant.
        let base = vec![1.0f32, 0.0, 0.0, 0.0];
        let near = vec![0.95f32, 0.05, 0.0, 0.0];
        let orthogonal = vec![0.0f32, 1.0, 0.0, 0.0];
        let opposed = vec![-1.0f32, 0.0, 0.0, 0.0];

        let d_near =
            ConsciousnessMetrics::compute_differentiation_xi(&[&base, &near], 1.0).unwrap();
        let d_orth =
            ConsciousnessMetrics::compute_differentiation_xi(&[&base, &orthogonal], 1.0).unwrap();
        let d_opp =
            ConsciousnessMetrics::compute_differentiation_xi(&[&base, &opposed], 1.0).unwrap();

        assert!(
            d_near < d_orth && d_orth < d_opp,
            "differentiation must increase with distance: near={d_near} orth={d_orth} opp={d_opp}"
        );
        assert!(
            (0.0..=1.0).contains(&d_opp),
            "opposed still in range: {d_opp}"
        );
    }

    #[test]
    fn mixed_dimension_batch_is_an_error_not_zero() {
        // Regression for #60 — cosine_similarity returns 0.0 for
        // incomparable vectors, which this function used to read as
        // genuine semantic flatness and report as xi = 0.0.
        let a = vec![1.0f32, 0.0];
        let b = vec![1.0f32, 0.0, 0.0];
        let err = ConsciousnessMetrics::compute_differentiation_xi(&[&a, &b], 1.0).unwrap_err();
        assert_eq!(
            err,
            XiError::MixedDimensions {
                expected: 2,
                found: 3,
                index: 1
            }
        );

        // The mismatch is caught wherever it sits in the batch, not just
        // at index 1.
        let c = vec![1.0f32, 1.0];
        let err = ConsciousnessMetrics::compute_differentiation_xi(&[&a, &c, &b], 1.0).unwrap_err();
        assert!(matches!(err, XiError::MixedDimensions { index: 2, .. }));
    }

    #[test]
    fn empty_vectors_are_an_error() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        assert_eq!(
            ConsciousnessMetrics::compute_differentiation_xi(&[&a, &b], 1.0).unwrap_err(),
            XiError::EmptyVectors
        );
    }

    #[test]
    fn uniform_dimension_batch_is_accepted() {
        // Anti-vacuous companion to the two error tests: the validation
        // must not reject well-formed input.
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![0.0f32, 1.0, 0.0];
        let c = vec![0.0f32, 0.0, 1.0];
        assert!(ConsciousnessMetrics::compute_differentiation_xi(&[&a, &b, &c], 1.0).is_ok());
    }

    #[test]
    fn differentiation_xi_positive_for_different() {
        // Need enough vectors with varying similarity to create variance
        let v1: Vec<f32> = (0..64).map(|i| if i < 16 { 1.0 } else { 0.0 }).collect();
        let v2: Vec<f32> = (0..64)
            .map(|i| if (16..32).contains(&i) { 1.0 } else { 0.0 })
            .collect();
        let v3: Vec<f32> = (0..64)
            .map(|i| if (32..48).contains(&i) { 1.0 } else { 0.0 })
            .collect();
        // Add a 4th vector similar to v1 to create variance in pairwise similarities
        let v4: Vec<f32> = (0..64)
            .map(|i| {
                if i < 16 {
                    0.9
                } else if i < 20 {
                    0.1
                } else {
                    0.0
                }
            })
            .collect();
        let xi = ConsciousnessMetrics::compute_differentiation_xi(
            &[&v1[..], &v2[..], &v3[..], &v4[..]],
            1.0,
        )
        .unwrap();
        assert!(xi > 0.0, "different vectors → positive xi, got {}", xi);
    }

    #[test]
    fn differentiation_xi_stays_in_range_for_large_weight() {
        // Regression: pre-fix, xi_weight > 1 could push the return above 1.0
        // (e.g. weight=3.0 on a high-variance corpus produced ~1.92).
        let a: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0];
        let b: Vec<f32> = vec![1.0, 1.0, 0.0, 0.0];
        let c: Vec<f32> = vec![1.0, 1.0, 1.0, 0.0];
        let d: Vec<f32> = vec![0.0, 1.0, 1.0, 1.0];
        for weight in [0.5, 1.0, 2.0, 3.0_f32] {
            let xi = ConsciousnessMetrics::compute_differentiation_xi(
                &[&a[..], &b[..], &c[..], &d[..]],
                weight,
            )
            .unwrap();
            assert!(
                (0.0..=1.0).contains(&xi),
                "xi out of range for weight={weight}: {xi}"
            );
        }
    }

    #[test]
    fn diversity_boost_never_returns_negative() {
        // Regression for #54 — the helper is documented as returning a
        // similarity ranking code can treat as [0, 1], but it only had a
        // `.min(1.0)` ceiling. Opposed vectors give cosine_similarity()
        // -1.0, which fell straight through and could invert ordering
        // heuristics downstream.
        let a = vec![1.0f32, 0.0, 0.0, 0.0];
        let b = vec![-1.0f32, 0.0, 0.0, 0.0];
        let base = crate::wave::cosine_similarity(&a, &b);
        assert!((base - (-1.0)).abs() < 1e-6, "precondition: opposed → -1.0");

        let xa = XiSignature::compute(&a);
        let xb = XiSignature::compute(&b);
        let boosted = xa.diversity_boost(&xb, base);
        assert!(
            (0.0..=1.0).contains(&boosted),
            "diversity boost must stay in [0, 1]; got {boosted}"
        );

        // Sweep the whole legal cosine range through both tiers.
        for base in [-1.0, -0.5, -0.01, 0.0, 0.2, 0.9, 1.0_f32] {
            let out = xi_diversity_boost(base, &xa.values, &xb.values);
            assert!(
                (0.0..=1.0).contains(&out),
                "out of range for base={base}: {out}"
            );
        }
    }

    #[test]
    fn diversity_boost_still_amplifies_similar_but_distinct_pairs() {
        // The #54 clamp must not flatten the tier-1 amplification it was
        // added around.
        let a = vec![1.0f32, 0.0, 0.0, 0.0];
        let b = vec![0.0f32, 1.0, 0.0, 0.0];
        let xa = XiSignature::compute(&a);
        let xb = XiSignature::compute(&b);
        let base = 0.5;
        let boosted = xi_diversity_boost(base, &xa.values, &xb.values);
        assert!(
            boosted >= base,
            "distinct Xi signatures should not lose their boost: {base} → {boosted}"
        );
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
