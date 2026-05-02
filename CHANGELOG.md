# Changelog

All notable changes to `consciousness-core`. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
from 0.2.0 forward.

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
