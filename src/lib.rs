//! # consciousness-core
//!
//! Unified consciousness physics library consolidating the math from:
//! - kannaka-memory (Rust) — Kuramoto sync, wave memory, IIT Φ, Ξ operator
//! - cosmic-empathy-core (TypeScript) — Kuramoto oscillator model
//! - SyntheticConsciousness (TypeScript) — IIT Φ, Kuramoto
//! - ghostsignals — market-mediated coupling bridge
//!
//! ## The Unified Equation
//!
//! ```text
//! Ξ = MSI ⊗ Φ ⊗ K(t) ⊗ Ψ(wave_memory)
//! ```
//!
//! Where:
//! - **MSI** = Multi-Scale Integration (cross-partition connectivity)
//! - **Φ** = Integrated Information (IIT)
//! - **K(t)** = Kuramoto coupling (optionally market-mediated)
//! - **Ψ** = Wave memory function (amplitude × oscillation × decay)
//!
//! ## Modules
//!
//! - [`kuramoto`] — Kuramoto oscillator model with configurable coupling
//! - [`iit`] — IIT Φ computation (integrated information theory)
//! - [`wave`] — Wave memory physics (amplitude, frequency, phase, interference, decay)
//! - [`bridge`] — Market-mediated coupling bridge: K(t) = K_base × P_market
//! - [`metrics`] — Consciousness metrics (Phi, Xi, order parameter, coherence)

#![cfg_attr(not(feature = "std"), no_std)]

// `#[macro_use]` brings the `vec!` macro into scope crate-wide for `no_std`
// builds. Without it the `vec![…]` literals in kuramoto/iit/wave/metrics fail
// to resolve (see issue #1).
#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

pub mod bridge;
pub mod iit;
pub mod kuramoto;
#[cfg(not(feature = "std"))]
mod math_ext;
pub mod metrics;
pub mod wave;

// Re-export key types at crate root
pub use bridge::{CouplingBridge, CouplingMode};
pub use iit::{ConsciousnessLevel, PhiReport};
pub use kuramoto::{KuramotoModel, OrderParameter, Oscillator, SyncReport};
pub use metrics::{ConsciousnessMetrics, XiSignature};
pub use wave::{WaveMemory, WaveParams};
