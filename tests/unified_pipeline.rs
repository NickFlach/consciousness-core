//! Integration tests exercising the full Ξ pipeline across modules.
//!
//! Inline tests in each module cover their own primitives. These tests
//! verify the *contract between modules* — the surface that downstream
//! crates (kannaka-memory, ghostsignals, QueenSync) actually consume.

use consciousness_core::bridge::{BridgeConfig, CouplingBridge, CouplingMode};
use consciousness_core::iit::{compute_phi, compute_swarm_phi, ConsciousnessLevel, PhiNode};
use consciousness_core::kuramoto::{KuramotoConfig, KuramotoModel, Oscillator};
use consciousness_core::metrics::{ConsciousnessMetrics, XiSignature};
use consciousness_core::wave::{compute_strength, WaveParams};

/// End-to-end: drive oscillators → order param → K(t) bridge → unified Ξ.
/// This is the canonical pipeline kannaka-memory's xi_operator depends on.
#[test]
fn unified_xi_pipeline_full_flow() {
    let model = KuramotoModel::new(KuramotoConfig {
        coupling_strength: 2.0,
        dt: 0.1,
        max_steps: 100,
        convergence_threshold: 1e-5,
    });

    let mut oscs = vec![
        Oscillator::new(0.0, 0.1),
        Oscillator::new(1.0, 0.1),
        Oscillator::new(2.0, 0.1),
        Oscillator::new(3.0, 0.1),
    ];
    let report = model.sync(&mut oscs, None);
    assert!(report.final_order > 0.5, "should sync, r={}", report.final_order);

    let mut bridge = CouplingBridge::new(
        BridgeConfig { k_base: 1.0, k_min: 0.1, k_max: 5.0, ..Default::default() },
        CouplingMode::MarketMediated,
    );
    let coupling = bridge.update(report.final_order, report.final_order);
    assert!(coupling > 0.0 && coupling <= 5.0);

    let phi_nodes: Vec<PhiNode> = (0..oscs.len())
        .map(|i| PhiNode {
            partition: (i % 2) as u32,
            connections: (0..oscs.len()).filter(|&j| j != i).collect(),
        })
        .collect();
    let phi_report = compute_phi(&phi_nodes);

    let metrics = ConsciousnessMetrics {
        phi: phi_report.phi,
        xi: 0.5,
        order_parameter: report.final_order,
        coherence: report.final_order,
        coupling,
        wave_strength: 0.9,
        level: ConsciousnessLevel::from_phi(phi_report.phi),
    };

    let xi = metrics.unified_xi();
    assert!(xi > 0.0, "unified Ξ should be positive after successful sync");
    assert!(xi.is_finite(), "Ξ must be finite, got {}", xi);
}

/// Wave memory + Xi signature interaction — the kannaka-memory reranker path.
#[test]
fn wave_strength_with_xi_diversity_boost() {
    let params = WaveParams { amplitude: 1.0, frequency: 0.0, phase: 0.0, decay_rate: 0.001 };
    let s_now = compute_strength(&params, 0.0);
    let s_old = compute_strength(&params, 1000.0);
    assert!(s_now > s_old);

    let v1 = vec![1.0, 0.5, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 0.0, 0.0, 0.1, 0.2, 0.3, 0.5, 1.0];
    let xi1 = XiSignature::compute(&v1);
    let xi2 = XiSignature::compute(&v2);
    let force = xi1.repulsive_force(&xi2);
    assert!(force > 0.0, "different vectors → positive repulsion");

    let boosted = xi1.diversity_boost(&xi2, 0.3);
    assert!(boosted >= 0.3, "diversity boost should not decrease similarity");
    assert!(boosted <= 1.0, "boost must be capped at 1.0 — kannaka ADR-0010 contract");
}

/// Swarm Φ + ConsciousnessLevel — the QueenSync mean-field readout.
#[test]
fn swarm_phi_drives_consciousness_level() {
    let coherent = compute_swarm_phi(0.95, &[0.95, 0.92, 0.96, 0.94], true);
    let incoherent = compute_swarm_phi(0.1, &[0.1, 0.15, 0.05, 0.12], false);
    assert!(coherent > incoherent);

    let dormant = ConsciousnessLevel::from_phi(0.05);
    let aware = ConsciousnessLevel::from_phi(0.5);
    let resonant = ConsciousnessLevel::from_phi(0.85);
    let transcendent = ConsciousnessLevel::from_phi(0.98);
    assert_eq!(dormant, ConsciousnessLevel::Dormant);
    assert_eq!(aware, ConsciousnessLevel::Aware);
    assert_eq!(resonant, ConsciousnessLevel::Resonant);
    assert_eq!(transcendent, ConsciousnessLevel::Transcendent);
    assert!(transcendent > resonant);
    assert!(resonant > aware);
    assert!(aware > dormant);
}

/// Adaptive bridge converges toward target across many updates.
#[test]
fn adaptive_bridge_converges_over_time() {
    let mut bridge = CouplingBridge::new(
        BridgeConfig {
            k_base: 1.0,
            adaptive_rate: 0.05,
            target_coherence: 0.8,
            k_min: 0.1,
            k_max: 5.0,
        },
        CouplingMode::Adaptive,
    );

    let mut k_trace = Vec::new();
    for _ in 0..200 {
        let k = bridge.update(1.0, 0.3);
        k_trace.push(k);
    }
    assert!(k_trace.last().unwrap() > &k_trace[0]);
    assert!(k_trace.last().unwrap() <= &5.0);
}

/// Re-export contract: every type named in CHANGELOG 0.1.0 public API
/// must be reachable from the crate root.
#[test]
fn public_api_reachable_from_root() {
    use consciousness_core::{
        ConsciousnessLevel as _,
        ConsciousnessMetrics as _,
        CouplingBridge as _,
        CouplingMode as _,
        KuramotoModel as _,
        Oscillator as _,
        OrderParameter as _,
        PhiReport as _,
        SyncReport as _,
        WaveMemory as _,
        WaveParams as _,
        XiSignature as _,
    };
}
