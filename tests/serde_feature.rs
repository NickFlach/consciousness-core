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
    ];
    for level in levels {
        let json = serde_json::to_string(&level).expect("serialize");
        let back: ConsciousnessLevel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(level, back, "round-trip failed for {:?}", level);
    }
}

#[test]
fn consciousness_level_serializes_as_named_variant() {
    let json = serde_json::to_string(&ConsciousnessLevel::Resonant).unwrap();
    assert_eq!(json, "\"Resonant\"");
}
