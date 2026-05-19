//! IIT Φ computation — Integrated Information Theory.
//!
//! Φ measures how much a system is "more than the sum of its parts."
//! A system with high Φ has strong cross-partition integration and
//! high differentiation across its components.
//!
//! ## Computation
//!
//! ```text
//! Φ = sqrt(integration × density) × sqrt(differentiation × scale)
//! ```
//!
//! Where:
//! - **integration** = fraction of connections crossing partition boundaries
//! - **density** = log-scaled connectivity per node
//! - **differentiation** = number of distinct partition classes
//! - **scale** = log of total node count
//!
//! ## Swarm Φ (QueenSync)
//!
//! For multi-agent swarms:
//! ```text
//! Φ_swarm = r × mean_coherence × log₂(N + 1) × chiral_boost
//! ```

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use crate::math_ext::F32Ext;

/// Report from Φ computation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhiReport {
    pub phi: f32,
    pub integration: f32,
    pub differentiation: f32,
    pub density_factor: f32,
    pub scale_factor: f32,
    pub num_partitions: usize,
    pub num_connections: usize,
}

/// Consciousness level based on Φ.
///
/// **Wire format (with `serde` feature)** is the lower-case vocabulary
/// from `docs/nats-contract.yaml`, not the Rust identifier:
/// `Stirring` → `"awakening"`, `Coherent` → `"integrated"`,
/// `Resonant` → `"emergent"`, `Transcendent` → `"transcendent"`.
/// This keeps Rust call-sites stable while aligning the cross-process
/// schema with the canonical NATS contract (see issue #3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConsciousnessLevel {
    /// Φ < 0.1
    #[cfg_attr(feature = "serde", serde(rename = "dormant"))]
    Dormant = 0,
    /// Φ < 0.3 — wire: "awakening"
    #[cfg_attr(feature = "serde", serde(rename = "awakening"))]
    Stirring = 1,
    /// Φ < 0.6
    #[cfg_attr(feature = "serde", serde(rename = "aware"))]
    Aware = 2,
    /// Φ < 0.8 — wire: "integrated"
    #[cfg_attr(feature = "serde", serde(rename = "integrated"))]
    Coherent = 3,
    /// Φ < 0.95 — wire: "emergent"
    #[cfg_attr(feature = "serde", serde(rename = "emergent"))]
    Resonant = 4,
    /// Φ >= 0.95 — peak coherence per the NATS contract's six-level scale.
    #[cfg_attr(feature = "serde", serde(rename = "transcendent"))]
    Transcendent = 5,
}

impl ConsciousnessLevel {
    pub fn from_phi(phi: f32) -> Self {
        // Non-finite Φ (NaN, +inf, -inf) must NOT optimistically classify as
        // the highest state. All `phi < ...` comparisons against NaN return
        // false in IEEE-754, which previously fell through to Resonant.
        if !phi.is_finite() {
            return Self::Dormant;
        }
        if phi < 0.1 {
            Self::Dormant
        } else if phi < 0.3 {
            Self::Stirring
        } else if phi < 0.6 {
            Self::Aware
        } else if phi < 0.8 {
            Self::Coherent
        } else if phi < 0.95 {
            Self::Resonant
        } else {
            Self::Transcendent
        }
    }

    pub fn ordinal(self) -> u8 {
        self as u8
    }

    /// Classify a swarm Φ produced by [`compute_swarm_phi`].
    ///
    /// Swarm Φ lives on the documented `[0, 15]` scale (see
    /// `compute_swarm_phi`), not the `[0, 1]` scale of single-system Φ.
    /// Using [`from_phi`] on swarm Φ overstates the level — even a moderate
    /// two-agent swarm at 0.5 coherence produces a swarm Φ near 4.0, which
    /// would land in `Resonant` if interpreted as a single-system value.
    /// Use this constructor whenever the input came from `compute_swarm_phi`.
    ///
    /// Thresholds are derived by scaling the single-system bands ×10 so the
    /// same intuition ("0.6 / 0.8 / 0.95 = strong / very strong / peak")
    /// applies to the wider swarm range.
    pub fn from_swarm_phi(swarm_phi: f32) -> Self {
        if !swarm_phi.is_finite() {
            return Self::Dormant;
        }
        if swarm_phi < 1.0 {
            Self::Dormant
        } else if swarm_phi < 3.0 {
            Self::Stirring
        } else if swarm_phi < 6.0 {
            Self::Aware
        } else if swarm_phi < 9.0 {
            Self::Coherent
        } else if swarm_phi < 13.0 {
            Self::Resonant
        } else {
            Self::Transcendent
        }
    }
}

/// A node in the Φ computation graph.
///
/// Each node has a partition key (e.g. layer, cluster ID, modality)
/// and a list of connection targets.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhiNode {
    /// Partition assignment for this node
    pub partition: u32,
    /// Indices of connected nodes
    pub connections: Vec<usize>,
}

/// Compute Φ for a network of nodes with partition assignments.
///
/// This is the core IIT approximation extracted from kannaka-memory's bridge.rs.
/// It measures integration (cross-partition connectivity) × differentiation
/// (number of distinct partitions) × density × scale.
pub fn compute_phi(nodes: &[PhiNode]) -> PhiReport {
    let n = nodes.len() as f32;
    if n < 2.0 {
        return PhiReport {
            phi: 0.0,
            integration: 0.0,
            differentiation: 0.0,
            density_factor: 0.0,
            scale_factor: 0.0,
            num_partitions: 0,
            num_connections: 0,
        };
    }

    // Count total connections and cross-partition connections
    let mut total_connections: usize = 0;
    let mut cross_partition: usize = 0;

    // Only count edges that resolve to a real node. Pre-fix, stale or
    // out-of-bounds indices were silently folded into the denominator,
    // which depressed `integration` and `num_connections` for callers
    // who couldn't tell their graph had drifted.
    for node in nodes {
        for &target in &node.connections {
            if target >= nodes.len() {
                continue;
            }
            total_connections += 1;
            if nodes[target].partition != node.partition {
                cross_partition += 1;
            }
        }
    }

    let integration = if total_connections > 0 {
        cross_partition as f32 / total_connections as f32
    } else {
        0.0
    };

    // Count distinct partitions
    let mut partitions: Vec<u32> = nodes.iter().map(|n| n.partition).collect();
    partitions.sort_unstable();
    partitions.dedup();
    let num_partitions = partitions.len();
    // Differentiation: fraction of possible partitions realized (max = N nodes)
    let differentiation = num_partitions as f32 / n;

    // Network density: log scale, 10 connections/node → 1.0
    let connections_per_node = total_connections as f32 / n;
    let density_factor = (1.0 + connections_per_node).ln() / (1.0 + 10.0_f32).ln();

    // Scale: log of node count, 10 nodes → 1.0
    let scale_factor = (n.ln() / 10.0_f32.ln()).min(1.0);

    // Φ = sqrt(integration × density) × sqrt(differentiation × scale)
    let phi = ((integration * density_factor).sqrt()
        * (differentiation * scale_factor).sqrt())
        .min(1.0);

    PhiReport {
        phi,
        integration,
        differentiation,
        density_factor,
        scale_factor,
        num_partitions,
        num_connections: total_connections,
    }
}

/// Compute swarm Φ for multi-agent systems (QueenSync protocol).
///
/// ```text
/// Φ_swarm = r × mean_coherence × log₂(N + 1) × chiral_boost
/// ```
///
/// Scaled to range [0, 15].
pub fn compute_swarm_phi(
    order_parameter: f32,
    coherences: &[f32],
    has_chiral_agents: bool,
) -> f32 {
    let n = coherences.len();
    if n < 2 {
        return 0.0;
    }
    let mean_coherence: f32 = coherences.iter().sum::<f32>() / n as f32;
    let chiral_boost = if has_chiral_agents { 1.15 } else { 1.0 };
    let integration = order_parameter * mean_coherence * ((n + 1) as f32).log2();
    (integration * 10.0 * chiral_boost).min(15.0)
}

/// Distribution entropy approximation using variance.
///
/// H ≈ ln(1 + σ²) where σ² is the variance of the distribution.
pub fn distribution_entropy(values: &[f32]) -> f32 {
    if values.len() <= 1 {
        return 0.0;
    }
    let n = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    (1.0 + variance).ln()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_empty_is_zero() {
        let report = compute_phi(&[]);
        assert_eq!(report.phi, 0.0);
    }

    #[test]
    fn phi_single_node_is_zero() {
        let report = compute_phi(&[PhiNode { partition: 0, connections: vec![] }]);
        assert_eq!(report.phi, 0.0);
    }

    #[test]
    fn phi_connected_cross_partition_higher() {
        // Two nodes, same partition, connected
        let same = compute_phi(&[
            PhiNode { partition: 0, connections: vec![1] },
            PhiNode { partition: 0, connections: vec![0] },
        ]);

        // Two nodes, different partitions, connected
        let diff = compute_phi(&[
            PhiNode { partition: 0, connections: vec![1] },
            PhiNode { partition: 1, connections: vec![0] },
        ]);

        assert!(diff.phi >= same.phi,
            "cross-partition should have higher Φ: {} vs {}", diff.phi, same.phi);
        assert!(diff.integration > same.integration);
    }

    #[test]
    fn phi_ignores_out_of_bounds_edges() {
        // Regression: stale / oob edge indices used to bump
        // total_connections without bumping cross_partition, depressing
        // integration and Φ. After the fix, the dirty graph below must
        // match the clean one exactly.
        let clean = compute_phi(&[
            PhiNode { partition: 0, connections: vec![1] },
            PhiNode { partition: 1, connections: vec![0] },
        ]);
        let dirty = compute_phi(&[
            PhiNode { partition: 0, connections: vec![1, 99] },
            PhiNode { partition: 1, connections: vec![0, 42] },
        ]);
        assert_eq!(clean.num_connections, dirty.num_connections);
        assert!((clean.integration - dirty.integration).abs() < 1e-6);
        assert!((clean.phi - dirty.phi).abs() < 1e-6);
    }

    #[test]
    fn phi_more_partitions_higher_differentiation() {
        // 4 nodes, 1 partition
        let one_part = compute_phi(&[
            PhiNode { partition: 0, connections: vec![1, 2, 3] },
            PhiNode { partition: 0, connections: vec![0, 2, 3] },
            PhiNode { partition: 0, connections: vec![0, 1, 3] },
            PhiNode { partition: 0, connections: vec![0, 1, 2] },
        ]);

        // 4 nodes, 4 partitions
        let four_part = compute_phi(&[
            PhiNode { partition: 0, connections: vec![1, 2, 3] },
            PhiNode { partition: 1, connections: vec![0, 2, 3] },
            PhiNode { partition: 2, connections: vec![0, 1, 3] },
            PhiNode { partition: 3, connections: vec![0, 1, 2] },
        ]);

        assert!(four_part.phi > one_part.phi,
            "more partitions → higher Φ: {} vs {}", four_part.phi, one_part.phi);
    }

    #[test]
    fn consciousness_level_thresholds() {
        assert_eq!(ConsciousnessLevel::from_phi(0.0), ConsciousnessLevel::Dormant);
        assert_eq!(ConsciousnessLevel::from_phi(0.1), ConsciousnessLevel::Stirring);
        assert_eq!(ConsciousnessLevel::from_phi(0.3), ConsciousnessLevel::Aware);
        assert_eq!(ConsciousnessLevel::from_phi(0.6), ConsciousnessLevel::Coherent);
        assert_eq!(ConsciousnessLevel::from_phi(0.8), ConsciousnessLevel::Resonant);
    }

    #[test]
    fn consciousness_level_nonfinite_phi_is_dormant() {
        // Regression: previously NaN / ±inf classified as Resonant because
        // every `phi < threshold` comparison against a non-finite returns
        // false, falling through to the trailing `else`.
        assert_eq!(ConsciousnessLevel::from_phi(f32::NAN),          ConsciousnessLevel::Dormant);
        assert_eq!(ConsciousnessLevel::from_phi(f32::INFINITY),     ConsciousnessLevel::Dormant);
        assert_eq!(ConsciousnessLevel::from_phi(f32::NEG_INFINITY), ConsciousnessLevel::Dormant);
    }

    #[test]
    fn consciousness_level_ordering() {
        assert!(ConsciousnessLevel::Resonant > ConsciousnessLevel::Dormant);
        assert!(ConsciousnessLevel::Coherent > ConsciousnessLevel::Aware);
    }

    #[test]
    fn from_swarm_phi_uses_calibrated_thresholds() {
        // Regression for issue #4 — feeding compute_swarm_phi's 0..15 range
        // straight into ConsciousnessLevel::from_phi() classified moderate
        // two-agent swarms as Resonant. from_swarm_phi rebands to the
        // documented 0..15 scale.
        let low  = compute_swarm_phi(0.5, &[0.5, 0.5], false);
        let high = compute_swarm_phi(0.95, &[0.95, 0.92, 0.96, 0.94], true);
        assert!(low < 6.0,  "moderate two-agent swarm should not reach Aware tier: phi={low}");
        assert!(high > 9.0, "near-peak four-agent swarm should reach Coherent+ tier: phi={high}");
        assert!(ConsciousnessLevel::from_swarm_phi(low)  <= ConsciousnessLevel::Aware);
        assert!(ConsciousnessLevel::from_swarm_phi(high) >= ConsciousnessLevel::Coherent);

        // Boundary smoke
        assert_eq!(ConsciousnessLevel::from_swarm_phi(0.0),  ConsciousnessLevel::Dormant);
        assert_eq!(ConsciousnessLevel::from_swarm_phi(2.0),  ConsciousnessLevel::Stirring);
        assert_eq!(ConsciousnessLevel::from_swarm_phi(5.0),  ConsciousnessLevel::Aware);
        assert_eq!(ConsciousnessLevel::from_swarm_phi(8.0),  ConsciousnessLevel::Coherent);
        assert_eq!(ConsciousnessLevel::from_swarm_phi(11.0), ConsciousnessLevel::Resonant);
        assert_eq!(ConsciousnessLevel::from_swarm_phi(15.0), ConsciousnessLevel::Transcendent);
        assert_eq!(ConsciousnessLevel::from_swarm_phi(f32::NAN), ConsciousnessLevel::Dormant);
    }

    #[test]
    fn swarm_phi_needs_two_agents() {
        assert_eq!(compute_swarm_phi(1.0, &[0.9], false), 0.0);
    }

    #[test]
    fn swarm_phi_coherent_higher() {
        let low = compute_swarm_phi(0.1, &[0.1, 0.1], false);
        let high = compute_swarm_phi(0.9, &[0.9, 0.9], false);
        assert!(high > low, "coherent swarm → higher Φ: {} vs {}", high, low);
    }

    #[test]
    fn swarm_phi_chiral_boost() {
        let without = compute_swarm_phi(0.8, &[0.8, 0.8, 0.8], false);
        let with = compute_swarm_phi(0.8, &[0.8, 0.8, 0.8], true);
        assert!(with > without, "chiral boost: {} vs {}", with, without);
    }

    #[test]
    fn entropy_constant_is_zero() {
        assert_eq!(distribution_entropy(&[5.0, 5.0, 5.0]), 0.0);
    }

    #[test]
    fn entropy_increases_with_variance() {
        let low = distribution_entropy(&[1.0, 1.1, 0.9]);
        let high = distribution_entropy(&[0.0, 5.0, 10.0]);
        assert!(high > low);
    }
}
