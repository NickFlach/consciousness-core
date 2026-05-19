# Changelog

All notable changes to `consciousness-core`. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
from 0.2.0 forward.

---

## [0.3.0] — 2026-05-19

Issue-triage sweep — fixes the eight open issues filed against 0.2.0.

### ⚠ Breaking

- **Wire format of `ConsciousnessLevel`** now matches the canonical NATS
  contract at `docs/nats-contract.yaml` (issue #3). Serialized values are
  the lower-case six-level vocabulary, not the Rust identifier names:
  `Dormant→"dormant"`, `Stirring→"awakening"`, `Aware→"aware"`,
  `Coherent→"integrated"`, `Resonant→"emergent"`,
  `Transcendent→"transcendent"`. Rust call-sites are unchanged (the
  identifiers `Stirring`/`Coherent`/`Resonant` keep working) — only the
  JSON/NATS payload form moves. Downstream HRM snapshots and any
  consumer keying off `consciousness_level` must update.
- **New `ConsciousnessLevel::Transcendent` variant** sits above `Resonant`
  at Φ ≥ 0.95. Existing `from_phi()` thresholds 0.0…0.8 are unchanged.

### Added

- `ConsciousnessLevel::from_swarm_phi(swarm_phi: f32)` — calibrated for
  `compute_swarm_phi()`'s documented 0..15 scale (issue #4). Use this
  instead of `from_phi()` whenever the input came from `compute_swarm_phi`.
- `serde` derives on every public report type: `PhiReport`, `PhiNode`,
  `SyncReport`, `OrderParameter`, `Oscillator`, `KuramotoConfig`,
  `WaveParams`, `WaveMemory`, `CouplingMode`, `BridgeConfig`,
  `ConsciousnessMetrics`, `XiSignature` (issue #2). `tests/serde_feature.rs`
  now exercises round-trip serialization for the full public surface.
- `libm` dependency (~80 KB, no_std-clean) routes float math through
  `math_ext::F32Ext` / `F64Ext` when the `std` feature is disabled —
  the documented `no_std` build now actually compiles and tests (issue #1).

### Fixed

- `ConsciousnessLevel::from_phi(NaN | ±inf)` now returns `Dormant` instead
  of `Resonant`. Non-finite Φ no longer flips an integration surface into
  a false "highest state" reading (issue #7).
- `compute_differentiation_xi()` clamps the final result to `[0, 1]`
  regardless of `xi_weight`, restoring the documented return range
  (issue #6).
- `compute_phi()` ignores out-of-bounds connection indices instead of
  silently inflating `total_connections` and depressing `integration`
  (issue #5).
- `KuramotoModel::sync()` now reports the **weighted** order parameter in
  `SyncReport.initial_order` / `final_order` and uses it for convergence
  detection, matching the public `order_parameter()` API (issue #8).
  Zero-weight oscillators no longer drag the report.

### Internal

- `src/math_ext.rs` — cfg-gated `F32Ext` / `F64Ext` extension traits
  routing through `libm` under `no_std`. Method-style call sites
  (`x.ln()`, `x.cos()`, etc.) work in both build modes without changes.
- `#[macro_use] extern crate alloc;` brings the `vec!` macro into the
  `no_std` build.
- README + `lib.rs` docs updated for the now-real `no_std` story.

---

## [0.2.0] — 2026-05-02

First release with documented downstream surface and a real
versioning policy. No breaking API changes from 0.1.0.

### Added

- `docs/dependency-map.md` — every constellation surface that consumes
  this crate, directly (Rust) or via NATS (cross-process). Indirect
  consumers in kannaka-radio, kannaka-observatory, kannaka-staff,
  OpenBotCity. Six hardening priorities listed in order of leverage.
- `docs/nats-contract.yaml` — canonical schema for every NATS subject
  the consciousness pipeline publishes. Required + optional fields,
  enum values, payload examples. Pinned at schema_version `1.0`.
- `Cargo.toml` keywords added for `cargo search` discoverability.
- `tests/unified_pipeline.rs` — integration tests covering the cross-module
  Ξ pipeline (Kuramoto sync → bridge K(t) → Φ → unified Ξ), wave + Xi
  diversity-boost interaction, swarm Φ → ConsciousnessLevel, adaptive bridge
  convergence, and a re-export contract test pinning every type named in
  the 0.1.0 public-API list.
- `tests/serde_feature.rs` — gated on `--features serde`, round-trips
  `ConsciousnessLevel` through serde_json. The serde feature is part of
  the public API contract; downstream crates (kannaka-memory) derive
  serialization on these types and rely on this round-tripping.
- `serde_json` added as `[dev-dependencies]` for the feature test.

### Test coverage

50 inline unit tests (across all 5 modules) + 7 integration tests = 57 total
on `cargo test --features serde`. All passing on 0.2.0.

### Shim policy (documented from 0.2.0)

Downstream crates (kannaka-memory) are organized so that direct
`use consciousness_core::*` imports are confined to a small set of
**shim files** that re-export the upstream API into a stable
crate-local namespace:

- `kannaka-memory/src/consciousness.rs` — re-exports `iit::*` types
- `kannaka-memory/src/wave.rs` — wraps `wave::*` math primitives
- `kannaka-memory/src/xi_operator.rs` — re-exports `metrics::*` constants

Every other module in kannaka-memory imports from `crate::consciousness::*`,
`crate::wave::*`, `crate::xi_operator::*` — never directly from
`consciousness_core::*`. This keeps the upstream crate's public API
behind a stable shim, so any future field rename here is a single-file
change.

**Rule:** if you find yourself adding `use consciousness_core::X` in
kannaka-memory outside of one of the three shim files, route through
the shim instead. Add the type to the shim's re-export list if it's
not there yet.

### Versioning policy

Starting from 0.2.0, public-API changes follow semver:

- **Patch (0.2.x)** — bug fixes, internal refactors, performance
  improvements that don't change the public API surface.
- **Minor (0.x.0)** — additive changes: new types, new methods,
  new optional NATS fields. Field renames go through a deprecation
  cycle and are kept as aliases for at least one minor version.
- **Major (x.0.0)** — breaking changes. Renamed/removed types,
  changed field semantics, removed deprecated aliases.

The `serde` feature flag is part of the public API. Downstream crates
that derive serialization on consciousness-core types depend on this
contract.

### Migration from 0.1.0

No code changes required. The 0.2.0 release is purely additive — same
types, same module structure, same public symbols. Bump your `Cargo.toml`:

```toml
[dependencies]
consciousness-core = { version = "0.2", features = ["serde"] }
```

---

## [0.1.0] — pre-2026-05-02

Initial release. No CHANGELOG existed before 0.2.0.

Public API as of 0.1.0 — preserved unchanged in 0.2.0:

- `kuramoto::{KuramotoModel, Oscillator, SyncReport, OrderParameter}`
- `iit::{PhiReport, ConsciousnessLevel}`
- `wave::{WaveParams, WaveMemory}`
- `bridge::{CouplingBridge, CouplingMode}`
- `metrics::{ConsciousnessMetrics, XiSignature}`
