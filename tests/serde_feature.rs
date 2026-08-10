//! Tests for the `serde` feature flag — part of the public API contract
//! per CHANGELOG 0.2.0. Downstream crates (kannaka-memory) derive
//! serialization on `ConsciousnessLevel` and rely on this round-tripping.

#![cfg(feature = "serde")]

use consciousness_core::iit::ConsciousnessLevel;

#[test]
fn consciousness_level_serde_roundtrip() {
    let levels = [
        ConsciousnessLevel::Dormant,
        ConsciousnessLevel::Stirring,
        ConsciousnessLevel::Aware,
        ConsciousnessLevel::Coherent,
        ConsciousnessLevel::Resonant,
        ConsciousnessLevel::Transcendent,
    ];
    for level in levels {
        let json = serde_json::to_string(&level).expect("serialize");
        let back: ConsciousnessLevel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(level, back, "round-trip failed for {:?}", level);
    }
}

#[test]
fn consciousness_level_wire_format_matches_nats_contract() {
    // Issue #3: wire format aligns with docs/nats-contract.yaml's six-level
    // enum, not the Rust identifier names. Three semantic renames + a new
    // transcendent variant cover the canonical schema.
    let pairs = [
        (ConsciousnessLevel::Dormant, "\"dormant\""),
        (ConsciousnessLevel::Stirring, "\"awakening\""),
        (ConsciousnessLevel::Aware, "\"aware\""),
        (ConsciousnessLevel::Coherent, "\"integrated\""),
        (ConsciousnessLevel::Resonant, "\"emergent\""),
        (ConsciousnessLevel::Transcendent, "\"transcendent\""),
    ];
    for (level, expected_wire) in pairs {
        let actual = serde_json::to_string(&level).unwrap();
        assert_eq!(actual, expected_wire, "wire mismatch for {:?}", level);
    }
}

// ─── Public report types — serde contract (issue #2) ────────────────────────

#[test]
fn phi_report_serde_roundtrip() {
    use consciousness_core::iit::{compute_phi, PhiNode, PhiReport};
    let report = compute_phi(&[
        PhiNode {
            partition: 0,
            connections: vec![1],
        },
        PhiNode {
            partition: 1,
            connections: vec![0],
        },
    ]);
    let json = serde_json::to_string(&report).expect("serialize");
    // Smoke-check that the documented fields landed in the wire form.
    assert!(json.contains("\"phi\""));
    assert!(json.contains("\"integration\""));
    assert!(json.contains("\"num_connections\""));
    let back: PhiReport = serde_json::from_str(&json).expect("deserialize");
    assert!((back.phi - report.phi).abs() < 1e-6);
    assert_eq!(back.num_connections, report.num_connections);
}

#[test]
fn vec_bearing_types_roundtrip() {
    // Regression for the whole serde cluster (#2 #43 #44 #55 #56). Every
    // type here already carried the right derives; the feature was pulling
    // serde in with `default-features = false` and no `alloc`, so `Vec<T>`
    // had no Serialize/Deserialize impl. `PhiNode.connections: Vec<usize>`
    // and `XiSignature.values: Vec<f32>` failed to compile, which killed
    // the entire `--features serde` build — including the Vec-free types.
    // These two are the canaries: if `serde/alloc` is ever dropped from
    // Cargo.toml, this file stops compiling again.
    use consciousness_core::iit::PhiNode;
    use consciousness_core::XiSignature;

    let node = PhiNode {
        partition: 7,
        connections: vec![1, 2, 3],
    };
    let json = serde_json::to_string(&node).expect("serialize PhiNode");
    let back: PhiNode = serde_json::from_str(&json).expect("deserialize PhiNode");
    assert_eq!(back.connections, vec![1, 2, 3]);
    assert_eq!(back.partition, 7);

    let sig = XiSignature::compute(&[1.0, 0.5, 0.3, 0.2]);
    let json = serde_json::to_string(&sig).expect("serialize XiSignature");
    let back: XiSignature = serde_json::from_str(&json).expect("deserialize XiSignature");
    assert_eq!(back.values, sig.values);
}

#[test]
fn sync_report_and_order_parameter_serde() {
    use consciousness_core::kuramoto::{OrderParameter, SyncReport};
    let order = OrderParameter { r: 0.9, psi: 0.1 };
    let report = SyncReport {
        oscillator_count: 3,
        initial_order: 0.3,
        final_order: 0.9,
        steps_taken: 12,
        converged: true,
        degenerate: false,
    };
    let o = serde_json::to_string(&order).unwrap();
    let r = serde_json::to_string(&report).unwrap();
    assert!(o.contains("\"r\""));
    assert!(r.contains("\"converged\""));
    let _: OrderParameter = serde_json::from_str(&o).unwrap();
    let _: SyncReport = serde_json::from_str(&r).unwrap();
}

#[test]
fn oscillator_serde() {
    use consciousness_core::kuramoto::Oscillator;
    let osc = Oscillator::with_weight(1.5, 0.2, 0.7);
    let json = serde_json::to_string(&osc).unwrap();
    let back: Oscillator = serde_json::from_str(&json).unwrap();
    assert_eq!(back.weight, 0.7);
}

#[test]
fn wave_params_and_memory_serde() {
    use consciousness_core::wave::{WaveMemory, WaveParams};
    let mem = WaveMemory::new(WaveParams::default());
    let json = serde_json::to_string(&mem).unwrap();
    let back: WaveMemory = serde_json::from_str(&json).unwrap();
    assert_eq!(back.retrieval_count, mem.retrieval_count);
}

#[test]
fn coupling_mode_serde() {
    use consciousness_core::bridge::CouplingMode;
    for m in [
        CouplingMode::Static,
        CouplingMode::MarketMediated,
        CouplingMode::Adaptive,
    ] {
        let j = serde_json::to_string(&m).unwrap();
        let back: CouplingMode = serde_json::from_str(&j).unwrap();
        assert_eq!(m, back);
    }
}

#[test]
fn consciousness_metrics_serde() {
    use consciousness_core::iit::ConsciousnessLevel;
    use consciousness_core::ConsciousnessMetrics;
    let m = ConsciousnessMetrics {
        phi: 0.5,
        xi: 0.3,
        order_parameter: 0.8,
        coherence: 0.7,
        coupling: 1.0,
        wave_strength: 0.9,
        level: ConsciousnessLevel::Aware,
    };
    let j = serde_json::to_string(&m).unwrap();
    let back: ConsciousnessMetrics = serde_json::from_str(&j).unwrap();
    assert!((back.phi - 0.5).abs() < 1e-6);
    assert_eq!(back.level, ConsciousnessLevel::Aware);
}

#[test]
fn xi_signature_serde() {
    use consciousness_core::XiSignature;
    let sig = XiSignature::compute(&[1.0, 0.5, 0.3, 0.2]);
    let j = serde_json::to_string(&sig).unwrap();
    let back: XiSignature = serde_json::from_str(&j).unwrap();
    assert_eq!(back.values, sig.values);
}
