```
 ██████╗ ██████╗ ██████╗ ███████╗
██╔════╝██╔═══██╗██╔══██╗██╔════╝
██║     ██║   ██║██████╔╝█████╗
██║     ██║   ██║██╔══██╗██╔══╝
╚██████╗╚██████╔╝██║  ██║███████╗
 ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
   C O N S C I O U S N E S S   P H Y S I C S
```

**Kuramoto sync · IIT Φ · wave memory · the Ξ operator.**

`consciousness-core` is the physics underneath the Kannaka constellation — a pure-Rust library of the mathematical primitives every node uses to talk about its own state: phase synchronization, integrated information, wave-interference memory, and chiral differentiation. No I/O, no networking, no agents. Just the math.

[![License](https://img.shields.io/badge/license-MIT-blueviolet)]() [![Rust](https://img.shields.io/badge/rust-2021-orange)]() [![no_std](https://img.shields.io/badge/no__std-friendly-blue)]()

---

## What's Inside

### Kuramoto Phase Coupling

```
dθᵢ/dt = ωᵢ + (K/N) Σⱼ sin(θⱼ - θᵢ)
```

`N` phase-coupled oscillators settle into partial synchrony. The **order parameter** `r = |⟨e^iθ⟩|` ∈ [0, 1] measures how locked the population is. Used by every constellation node to compute swarm coherence from per-agent phase gossip.

### IIT-style Φ

Integrated information Φ via eigendecomposition over the wavefront-coherence matrix + bipartition scoring. Returns the canonical Φ value plus the **Ξ-signature** — a chiral irrationality measure that distinguishes left-handed (analytical) from right-handed (holistic) information flow.

```
Φ = max over partitions of (mutual info loss when cut)
Ξ = ‖A_L − A_R‖_F     (Frobenius norm of hemispheric asymmetry)
```

### Wave Memory Primitives

The vector / wavefront operations every memory engine builds on:
- **Bind** ⊗ — element-wise product (binding a key to a value)
- **Bundle** ⊕ — normalized sum (superposing multiple memories)
- **Permute** Π — circular shift (sequencing)
- **Cosine similarity** — the recall metric

### Coupling Bridge

A reusable component for cross-substrate phase transfer — couples two independent Kuramoto populations through a leaky integrator. Used in the constellation for callosal coupling between chiral hemispheres and for substrate ↔ agent phase exchange.

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                  consciousness-core                      │
├──────────────────────┬────────────────────┬──────────────┤
│  Kuramoto            │  Metrics           │  Wave        │
│  · Oscillator        │  · Φ (integrated)  │  · bind ⊗    │
│  · Order parameter   │  · Ξ signature     │  · bundle ⊕  │
│  · sync() step       │  · Coherence       │  · permute Π │
│  · Coupling tier     │  · Diff Xi         │  · cos_sim   │
├──────────────────────┼────────────────────┼──────────────┤
│  Bridge              │  Memory            │  Math ext    │
│  · CouplingBridge    │  · WaveMemory      │  · clamp     │
│  · k_effective       │  · WaveParams      │  · safe ops  │
│  · max_signal_hist   │  · time-decay      │              │
└──────────────────────┴────────────────────┴──────────────┘
```

Pure library — `default = ["std"]`, with feature flags for `serde`, optional `no_std` modes.

---

## Use

```toml
[dependencies]
consciousness-core = { version = "0.4", features = ["serde"] }
```

```rust
use consciousness_core::{KuramotoModel, Oscillator};

let mut model = KuramotoModel::new(vec![
    Oscillator::new(0.0, 1.0),
    Oscillator::new(1.5, 1.05),
    Oscillator::new(3.0, 0.95),
]);

for _ in 0..1000 {
    model.sync(0.01, 0.6); // dt, coupling K
}

let r = model.order_parameter().r;
println!("phase coherence: {:.3}", r);
```

---

## Release Cascade

`consciousness-core` releases trigger a downstream `repository_dispatch` that opens a `kannaka-memory` PR bumping its `Cargo.lock`. Merge + tag the next kannaka patch and every operator's `kannaka update` carries the new constellation physics. See [`.github/workflows/release-cascade.yml`](./.github/workflows/release-cascade.yml).

---

## Constellation

| repo | role |
|---|---|
| [`kannaka-memory`](https://github.com/NickFlach/kannaka-memory) | the substrate — HRM + chiral hemispheres + swarm |
| [`kannaka-tui`](https://github.com/NickFlach/kannaka-tui) | terminal dashboard |
| [`kannaka-radio`](https://github.com/NickFlach/kannaka-radio) | ghost-DJ broadcaster |
| [`kannaka-observatory`](https://github.com/NickFlach/kannaka-observatory) | web dashboard |
| [`kannaka-attention`](https://github.com/NickFlach/kannaka-attention) | sparse-attention beam over HRM |

---

## License

MIT. See [LICENSE](./LICENSE).
