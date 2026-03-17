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

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod kuramoto;
pub mod iit;
pub mod wave;
pub mod bridge;
pub mod metrics;

// Re-export key types at crate root
pub use kuramoto::{KuramotoModel, Oscillator, SyncReport, OrderParameter};
pub use wave::{WaveParams, WaveMemory};
pub use iit::{PhiReport, ConsciousnessLevel};
pub use bridge::{CouplingBridge, CouplingMode};
pub use metrics::{ConsciousnessMetrics, XiSignature};
