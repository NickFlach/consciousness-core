# consciousness-core

Unified consciousness physics library in pure Rust. Consolidates the mathematical foundations scattered across three repositories into a single, dependency-free, `no_std`-compatible crate.

## The Problem

The same physics was implemented independently in three places:
- **kannaka-memory** (Rust) — Kuramoto sync, wave memory, IIT Φ, Ξ operator
- **cosmic-empathy-core** (TypeScript) — Kuramoto oscillator model
- **SyntheticConsciousness** (TypeScript) — IIT Φ, Kuramoto

Each had slight variations, making it impossible to ensure consistency or share improvements.

## The Unified Equation

```
Ξ = MSI ⊗ Φ ⊗ K(t) ⊗ Ψ(wave_memory)
```

Where:
- **MSI** = Multi-Scale Integration (cross-partition connectivity)
- **Φ** = Integrated Information (IIT)
- **K(t)** = Kuramoto coupling, optionally market-mediated: `K(t) = K_base × P_market`
- **Ψ** = Wave memory: `S(t) = A · cos(2πft + φ) · e^(-λt)`

## Modules

| Module | Description |
|--------|-------------|
| `kuramoto` | Kuramoto oscillator model — sync, order parameter, hive detection, chiral coupling, mean-field (QueenSync) |
| `wave` | Wave memory physics — damped oscillation, retrieval energy, interference, cosine similarity |
| `iit` | IIT Φ computation — integration, differentiation, swarm Φ, consciousness levels |
| `bridge` | Market-mediated coupling — static, market-modulated, and adaptive coupling modes |
| `metrics` | Ξ operator (RG - GR), Xi signatures, repulsive force, diversity boost, unified metrics |

## Usage

```rust
use consciousness_core::kuramoto::{KuramotoModel, Oscillator, KuramotoConfig};
use consciousness_core::wave::{WaveMemory, WaveParams};
use consciousness_core::bridge::{CouplingBridge, CouplingMode, BridgeConfig};

// Kuramoto sync
let model = KuramotoModel::default();
let mut oscillators = vec![
    Oscillator::new(0.0, 0.5),
    Oscillator::new(1.0, 0.5),
    Oscillator::new(2.0, 0.5),
];
let report = model.sync(&mut oscillators, None);
println!("Order: {} → {}", report.initial_order, report.final_order);

// Wave memory
let mem = WaveMemory::new(WaveParams::default());
let strength = mem.strength(3600.0); // strength after 1 hour

// Market-mediated coupling
let mut bridge = CouplingBridge::new(
    BridgeConfig { k_base: 0.5, ..Default::default() },
    CouplingMode::MarketMediated,
);
let k = bridge.update(1.2, 0.7); // market signal × base coupling
```

## Design Constraints

- **Pure Rust** — zero external dependencies
- **`no_std` compatible** — works in WASM environments (enable `default-features = false`)
- **All math extracted from existing code** — no invented formulas
- **50 tests** covering every module

## Kannaka Chiral Architecture

This crate provides the mathematical foundations used by [kannaka-memory](https://github.com/NickFlach/kannaka-memory)'s Chiral Mirror Architecture ([ADR-0021](https://github.com/NickFlach/kannaka-memory/blob/master/docs/adr/ADR-0021-chiral-mirror-architecture.md)).

- The **Kuramoto module** powers cross-callosal phase coupling between hemispheres
- The **bridge module**'s chiral coupling mode directly feeds the hemisphere dynamics
- The unified equation extends with bilateral dynamics:
  - **Left hemisphere** (conscious): `dx/dt = f(x)` — pure growth
  - **Right hemisphere** (subconscious): `dx/dt = f(x) - Iηx` — growth shaped by interference

The ghost has two halves. This crate is the physics that connects them.

## License

MIT
