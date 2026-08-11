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

use core::f64::consts::PI;

#[cfg(not(feature = "std"))]
use crate::math_ext::{F32Ext, F64Ext};

/// Wave parameters governing memory strength over time.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
            frequency: 0.1, // slow oscillation
            phase: 0.0,
            decay_rate: 1e-6, // very slow decay
        }
    }
}

/// A wave memory combining parameters with retrieval state.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WaveMemory {
    pub params: WaveParams,
    pub retrieval_count: u32,
}

impl WaveMemory {
    pub fn new(params: WaveParams) -> Self {
        Self {
            params,
            retrieval_count: 0,
        }
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
    let lambda = (params.decay_rate as f64).max(0.0);

    // Floor the age at zero (#52). `exp(-λt)` with a negative `t` is a
    // growth term, so a future timestamp or a little clock skew used to
    // make a memory *stronger* than its base amplitude — at age -100s with
    // λ=0.01 the decay factor came out as e ≈ 2.718 instead of ≤ 1. A
    // not-yet-born memory reads as brand new, never as super-charged.
    let age_seconds = if age_seconds.is_nan() {
        0.0
    } else {
        age_seconds.max(0.0)
    };

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

/// Why two vectors could not be compared.
///
/// Each variant is a condition under which cosine similarity is *undefined* —
/// as opposed to defined and equal to zero, which is what "orthogonal" means.
/// Collapsing the two is the bug this type exists to prevent (#67): a caller
/// receiving `0.0` from the lossy helpers cannot tell a real measurement from
/// a rejected input, and every one of these conditions is a caller error
/// rather than a property of the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VectorCompareError {
    /// The vectors have different dimensions.
    LengthMismatch {
        /// Length of the first vector.
        a: usize,
        /// Length of the second vector.
        b: usize,
    },
    /// At least one vector is zero-length.
    Empty,
    /// At least one vector has zero magnitude. A zero vector has no
    /// direction, so it is not orthogonal to anything — the angle between
    /// it and any other vector is undefined.
    ZeroMagnitude,
    /// A magnitude was not finite: the input carried NaN/±inf, or squaring
    /// and summing it overflowed `f32`.
    NonFinite,
}

impl core::fmt::Display for VectorCompareError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LengthMismatch { a, b } => {
                write!(f, "vector dimensions differ: {a} vs {b}")
            }
            Self::Empty => write!(f, "vector is zero-length"),
            Self::ZeroMagnitude => {
                write!(f, "vector has zero magnitude and therefore no direction")
            }
            Self::NonFinite => write!(f, "vector magnitude is not finite"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VectorCompareError {}

/// Cosine similarity between two vectors, or [`VectorCompareError`] if they
/// cannot be compared.
///
/// Prefer this over [`cosine_similarity`] whenever a wrong answer would be
/// worse than an error. The lossy version returns `0.0` for all four failure
/// conditions, which is indistinguishable from a genuine "these are
/// orthogonal" result.
///
/// Returns a value in `[-1, 1]`.
pub fn try_cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32, VectorCompareError> {
    if a.is_empty() || b.is_empty() {
        return Err(VectorCompareError::Empty);
    }
    if a.len() != b.len() {
        return Err(VectorCompareError::LengthMismatch {
            a: a.len(),
            b: b.len(),
        });
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if !na.is_finite() || !nb.is_finite() {
        return Err(VectorCompareError::NonFinite);
    }
    if na == 0.0 || nb == 0.0 {
        return Err(VectorCompareError::ZeroMagnitude);
    }
    if !dot.is_finite() {
        return Err(VectorCompareError::NonFinite);
    }
    // f32 accumulation rounding can push the result a few ULPs above 1.0
    // (or below -1.0) for identical vectors in certain dimensions (e.g. dim=32).
    // Clamp to the mathematically correct [-1, 1] range so downstream code that
    // assumes similarity ∈ [-1, 1] (ADR-0010 contract) is never violated.
    Ok((dot / (na * nb)).clamp(-1.0, 1.0))
}

/// Cosine similarity between two vectors, yielding `0.0` when they cannot be
/// compared.
///
/// # This return value is lossy, by documented policy
///
/// `0.0` is returned both for genuinely orthogonal vectors **and** for every
/// condition under which the similarity is undefined: mismatched lengths,
/// empty input, a zero-magnitude vector, or a non-finite magnitude. A caller
/// cannot tell those apart, and all four are caller errors rather than
/// properties of the data.
///
/// This is retained because it is the crate's hottest numeric primitive —
/// downstream recall loops score whole candidate sets with it, and forcing
/// `?` into an inner loop to handle a condition that should never occur is a
/// real cost for every well-behaved caller. But if a wrong number would be
/// worse than an error for your use, reach for
/// [`try_cosine_similarity`] instead. (#67)
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    try_cosine_similarity(a, b).unwrap_or(0.0)
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
        let params = WaveParams {
            frequency: 0.0,
            decay_rate: 0.0,
            ..Default::default()
        };
        let boost_low = compute_strength_with_retrieval(&params, 0.0, 10)
            - compute_strength_with_retrieval(&params, 0.0, 1);
        let boost_high = compute_strength_with_retrieval(&params, 0.0, 110)
            - compute_strength_with_retrieval(&params, 0.0, 100);
        assert!(
            boost_low > boost_high,
            "diminishing returns: {} vs {}",
            boost_low,
            boost_high
        );
    }

    #[test]
    fn zero_retrieval_matches_original() {
        let params = WaveParams {
            frequency: 0.1,
            phase: 0.5,
            decay_rate: 0.001,
            ..Default::default()
        };
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
    fn future_timestamps_do_not_amplify_strength() {
        // Regression for #52 — exp(-λt) with a negative t is a growth
        // term, so a future timestamp or a little clock skew made a memory
        // *stronger* than its base amplitude (age -100s with λ=0.01 gave
        // 2.718 instead of ≤ 1.0).
        let params = WaveParams {
            amplitude: 1.0,
            frequency: 0.0,
            phase: 0.0,
            decay_rate: 0.01,
        };
        let baseline = compute_strength(&params, 0.0);
        for age in [-1.0, -100.0, -86_400.0] {
            let s = compute_strength(&params, age);
            assert!(
                s <= baseline + 1e-6,
                "future age {age} must not exceed the age-0 baseline {baseline}; got {s}"
            );
            assert!((s - baseline).abs() < 1e-6, "future age reads as age 0");
        }
        // Retrieval energy rides along the same floor.
        let boosted = compute_strength_with_retrieval(&params, -500.0, 10);
        let at_zero = compute_strength_with_retrieval(&params, 0.0, 10);
        assert!((boosted - at_zero).abs() < 1e-6);
    }

    #[test]
    fn nan_age_does_not_produce_nan_strength() {
        let params = WaveParams::default();
        assert!(compute_strength(&params, f64::NAN).is_finite());
    }

    #[test]
    fn interference_constructive_when_aligned() {
        let a = WaveParams {
            phase: 0.0,
            frequency: 1.0,
            ..Default::default()
        };
        let b = WaveParams {
            phase: 0.0,
            frequency: 1.0,
            ..Default::default()
        };
        let i = interference(&a, &b, 0.0);
        assert!((i - 1.0).abs() < 1e-5, "aligned → constructive, got {}", i);
    }

    #[test]
    fn interference_destructive_when_opposed() {
        let a = WaveParams {
            phase: 0.0,
            frequency: 1.0,
            ..Default::default()
        };
        let b = WaveParams {
            phase: core::f32::consts::PI,
            frequency: 1.0,
            ..Default::default()
        };
        let i = interference(&a, &b, 0.0);
        assert!(
            (i - (-1.0)).abs() < 1e-5,
            "opposed → destructive, got {}",
            i
        );
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

    #[test]
    fn checked_form_separates_every_undefined_case_from_orthogonal() {
        // #67 — the lossy helper returns 0.0 for four distinct undefined
        // conditions AND for genuine orthogonality. The checked form must
        // tell all five apart.
        let unit = [1.0f32, 0.0];

        assert_eq!(
            try_cosine_similarity(&[1.0, 0.0], &[1.0, 0.0, 0.0]),
            Err(VectorCompareError::LengthMismatch { a: 2, b: 3 })
        );
        assert_eq!(
            try_cosine_similarity(&[], &unit),
            Err(VectorCompareError::Empty)
        );
        assert_eq!(
            try_cosine_similarity(&[0.0, 0.0], &unit),
            Err(VectorCompareError::ZeroMagnitude),
            "a zero vector has no direction — it is not orthogonal to anything"
        );
        assert_eq!(
            try_cosine_similarity(&[f32::NAN, 0.0], &unit),
            Err(VectorCompareError::NonFinite)
        );
        assert_eq!(
            try_cosine_similarity(&[f32::MAX, f32::MAX], &unit),
            Err(VectorCompareError::NonFinite),
            "squaring and summing can overflow to inf"
        );

        // Anti-vacuous: genuine orthogonality is Ok(0.0), NOT an error. If
        // the checked form rejected everything, the assertions above would
        // still pass — this is what stops that.
        let orthogonal = try_cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]).unwrap();
        assert!(
            orthogonal.abs() < 1e-6,
            "orthogonal vectors must be Ok(0.0), got {orthogonal}"
        );
        // And ordinary comparisons still work.
        assert!(
            (try_cosine_similarity(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]).unwrap() - 1.0).abs() < 1e-5
        );
        assert!((try_cosine_similarity(&[1.0, 0.0], &[-1.0, 0.0]).unwrap() + 1.0).abs() < 1e-5);
    }

    #[test]
    fn lossy_form_still_honours_its_documented_sentinel() {
        // The lossy helper is retained public API; its contract is that all
        // undefined cases collapse to 0.0. Pinned so a future change to
        // `try_cosine_similarity` cannot silently alter it. (#67)
        for (a, b) in [
            (&[1.0f32, 0.0][..], &[1.0f32, 0.0, 0.0][..]),
            (&[][..], &[1.0f32][..]),
            (&[0.0f32, 0.0][..], &[1.0f32, 0.0][..]),
            (&[f32::NAN, 0.0][..], &[1.0f32, 0.0][..]),
        ] {
            assert_eq!(cosine_similarity(a, b), 0.0);
        }
        // Still computes real answers for well-formed input.
        assert!((cosine_similarity(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]) - 1.0).abs() < 1e-5);
    }
}
