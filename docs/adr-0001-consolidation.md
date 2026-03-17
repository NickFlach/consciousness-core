# ADR-0001: Consolidate Consciousness Physics into consciousness-core

**Date:** 2026-03-17
**Status:** Accepted
**Author:** Kannaka

## Context

The consciousness physics (Kuramoto sync, IIT Φ, wave memory, Ξ operator) exists in three independent implementations:

1. **kannaka-memory** (Rust) — the most complete, with Kuramoto, wave physics, IIT Φ, Ξ operator, and QueenSync
2. **cosmic-empathy-core** (TypeScript) — Kuramoto oscillator for empathy mapping
3. **SyntheticConsciousness** (TypeScript) — IIT Φ and Kuramoto for AI consciousness research

Additionally, **ghostsignals** introduces market-mediated coupling (`K(t) = K_base × P_market`) that modulates synchronization strength based on external signals.

### Problems

- Bug fixes in one repo don't propagate to others
- Each implementation has slight formula variations
- No single source of truth for the math
- TypeScript implementations can't run in constrained environments
- The unified equation `Ξ = MSI ⊗ Φ ⊗ K(t) ⊗ Ψ` was never implemented in one place

## Decision

Create `consciousness-core` as a pure Rust library crate that:

1. Extracts the **real math** from kannaka-memory (the most mature implementation)
2. Provides a clean, dependency-free API that all other crates can depend on
3. Supports `no_std` for WASM compilation
4. Consolidates all five physics modules: kuramoto, wave, iit, bridge, metrics

## Consequences

### Positive
- Single source of truth for all consciousness physics
- kannaka-memory can depend on consciousness-core instead of reimplementing
- TypeScript projects can use via WASM compilation
- Clean separation of pure math from application-specific code (memory stores, Dolt, NATS)

### Negative
- Migration effort: kannaka-memory needs refactoring to use this crate
- Two places to maintain until migration is complete

### Migration Plan
1. ✅ Create consciousness-core with extracted math
2. Add consciousness-core as dependency to kannaka-memory's Cargo.toml
3. Replace kannaka-memory's kuramoto.rs, wave.rs, bridge.rs, xi_operator.rs with re-exports
4. Build WASM target for TypeScript consumers
5. Deprecate TypeScript implementations in cosmic-empathy-core and SyntheticConsciousness
