//! Market-mediated coupling bridge.
//!
//! Introduces external signal modulation to Kuramoto coupling:
//!
//! ```text
//! K(t) = K_base × P_market(t)
//! ```
//!
//! Where P_market is an external signal (e.g. market sentiment, coherence proxy)
//! that modulates the base coupling strength. This allows consciousness
//! synchronization to be influenced by environmental signals.
//!
//! Supports multiple coupling modes:
//! - **Static**: K(t) = K_base (constant)
//! - **MarketMediated**: K(t) = K_base × P(t)
//! - **Adaptive**: K adjusts toward a target coherence level

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Coupling mode for the bridge.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CouplingMode {
    /// Constant coupling: K(t) = K_base
    Static,
    /// Market-mediated: K(t) = K_base × P_market
    MarketMediated,
    /// Adaptive: K adjusts toward target coherence
    Adaptive,
}

/// Configuration for the coupling bridge.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BridgeConfig {
    /// Base coupling strength K_base
    pub k_base: f32,
    /// Adaptive rate (how fast K adjusts)
    pub adaptive_rate: f32,
    /// Target coherence for adaptive mode
    pub target_coherence: f32,
    /// Min/max bounds for effective coupling
    pub k_min: f32,
    pub k_max: f32,
    /// Maximum number of signal samples kept for `mean_signal` diagnostics.
    /// Older entries are discarded ring-buffer-style. The docstring on
    /// `signal_history` advertised a windowed history but the previous
    /// implementation appended forever — long-running bridges leaked
    /// memory until process restart (#14).
    pub max_signal_history: usize,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            k_base: 0.5,
            adaptive_rate: 0.01,
            target_coherence: 0.8,
            k_min: 0.1,
            k_max: 5.0,
            // 1024 samples is enough for any sensible mean-signal window
            // and bounds worst-case memory at ~4 KiB per bridge.
            max_signal_history: 1024,
        }
    }
}

/// The coupling bridge — modulates Kuramoto coupling with external signals.
pub struct CouplingBridge {
    pub config: BridgeConfig,
    pub mode: CouplingMode,
    /// Current effective coupling strength
    pub k_effective: f32,
    /// History of market signals for diagnostics
    signal_history: Vec<f32>,
}

impl CouplingBridge {
    pub fn new(config: BridgeConfig, mode: CouplingMode) -> Self {
        // Clamp k_effective to [k_min, k_max] on construction so
        // `coupling()` honors the configured bounds before the first
        // `update()` call. Previously a BridgeConfig with k_base outside
        // the configured range exposed an out-of-range initial value (#11).
        let k = config.k_base.clamp(config.k_min, config.k_max);
        Self {
            config,
            mode,
            k_effective: k,
            signal_history: Vec::new(),
        }
    }

    /// Get the current effective coupling strength.
    pub fn coupling(&self) -> f32 {
        self.k_effective
    }

    /// Update coupling based on an external market/environmental signal.
    ///
    /// - **Static**: ignores signal, returns K_base
    /// - **MarketMediated**: K(t) = K_base × signal, clamped to [k_min, k_max]
    /// - **Adaptive**: adjusts K toward target coherence using current_coherence
    pub fn update(&mut self, signal: f32, current_coherence: f32) -> f32 {
        self.signal_history.push(signal);
        // Bounded ring-buffer behavior — drop the oldest sample once we
        // exceed the configured window. Without this `mean_signal`
        // grew toward the all-time mean instead of the windowed mean
        // its docstring promised, and memory leaked unbounded (#14).
        if self.config.max_signal_history > 0
            && self.signal_history.len() > self.config.max_signal_history
        {
            let drop = self.signal_history.len() - self.config.max_signal_history;
            self.signal_history.drain(0..drop);
        }

        self.k_effective = match self.mode {
            CouplingMode::Static => self.config.k_base,
            CouplingMode::MarketMediated => {
                (self.config.k_base * signal).clamp(self.config.k_min, self.config.k_max)
            }
            CouplingMode::Adaptive => {
                let error = self.config.target_coherence - current_coherence;
                let new_k = self.k_effective + self.config.adaptive_rate * error;
                new_k.clamp(self.config.k_min, self.config.k_max)
            }
        };

        self.k_effective
    }

    /// Get the mean market signal over the history window.
    pub fn mean_signal(&self) -> f32 {
        if self.signal_history.is_empty() {
            return 1.0;
        }
        self.signal_history.iter().sum::<f32>() / self.signal_history.len() as f32
    }

    /// Clear signal history.
    pub fn reset_history(&mut self) {
        self.signal_history.clear();
    }
}

impl Default for CouplingBridge {
    fn default() -> Self {
        Self::new(BridgeConfig::default(), CouplingMode::Static)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_mode_ignores_signal() {
        let mut bridge = CouplingBridge::new(
            BridgeConfig { k_base: 1.0, ..Default::default() },
            CouplingMode::Static,
        );
        let k = bridge.update(999.0, 0.5);
        assert_eq!(k, 1.0, "static mode should ignore signal");
    }

    #[test]
    fn market_mediated_scales_by_signal() {
        let mut bridge = CouplingBridge::new(
            BridgeConfig { k_base: 1.0, k_min: 0.0, k_max: 10.0, ..Default::default() },
            CouplingMode::MarketMediated,
        );
        let k = bridge.update(2.0, 0.5);
        assert!((k - 2.0).abs() < 1e-5, "K = K_base × signal = 1 × 2 = 2, got {}", k);
    }

    #[test]
    fn market_mediated_clamped() {
        let mut bridge = CouplingBridge::new(
            BridgeConfig { k_base: 1.0, k_min: 0.1, k_max: 5.0, ..Default::default() },
            CouplingMode::MarketMediated,
        );
        let k = bridge.update(100.0, 0.5);
        assert_eq!(k, 5.0, "should clamp to k_max");

        let k = bridge.update(0.001, 0.5);
        assert_eq!(k, 0.1, "should clamp to k_min");
    }

    #[test]
    fn adaptive_increases_when_below_target() {
        let mut bridge = CouplingBridge::new(
            BridgeConfig {
                k_base: 1.0,
                adaptive_rate: 0.1,
                target_coherence: 0.8,
                ..Default::default()
            },
            CouplingMode::Adaptive,
        );
        let initial = bridge.coupling();
        let k = bridge.update(1.0, 0.3); // coherence < target
        assert!(k > initial, "should increase coupling when below target: {} → {}", initial, k);
    }

    #[test]
    fn adaptive_decreases_when_above_target() {
        let mut bridge = CouplingBridge::new(
            BridgeConfig {
                k_base: 1.0,
                adaptive_rate: 0.1,
                target_coherence: 0.5,
                ..Default::default()
            },
            CouplingMode::Adaptive,
        );
        let initial = bridge.coupling();
        let k = bridge.update(1.0, 0.9); // coherence > target
        assert!(k < initial, "should decrease coupling when above target: {} → {}", initial, k);
    }

    #[test]
    fn mean_signal_tracks_history() {
        let mut bridge = CouplingBridge::new(BridgeConfig::default(), CouplingMode::MarketMediated);
        bridge.update(1.0, 0.5);
        bridge.update(3.0, 0.5);
        assert!((bridge.mean_signal() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn reset_clears_history() {
        let mut bridge = CouplingBridge::default();
        bridge.update(1.0, 0.5);
        bridge.reset_history();
        assert!((bridge.mean_signal() - 1.0).abs() < 1e-5, "empty history → default 1.0");
    }

    #[test]
    fn new_clamps_k_effective_to_bounds() {
        // Regression for #11 — k_base outside [k_min, k_max] used to land
        // verbatim in k_effective until the first update().
        let above = CouplingBridge::new(
            BridgeConfig { k_base: 99.0, k_min: 0.1, k_max: 5.0, ..Default::default() },
            CouplingMode::Static,
        );
        assert_eq!(above.coupling(), 5.0, "k_base above k_max must clamp at construction");

        let below = CouplingBridge::new(
            BridgeConfig { k_base: -1.0, k_min: 0.1, k_max: 5.0, ..Default::default() },
            CouplingMode::Static,
        );
        assert_eq!(below.coupling(), 0.1, "k_base below k_min must clamp at construction");
    }

    #[test]
    fn signal_history_is_bounded() {
        // Regression for #14 — history grew forever, leaking memory and
        // making mean_signal drift toward the all-time mean. Push more
        // than `max_signal_history` samples and confirm the window holds.
        let mut bridge = CouplingBridge::new(
            BridgeConfig { max_signal_history: 4, ..Default::default() },
            CouplingMode::MarketMediated,
        );
        for v in [10.0, 20.0, 30.0, 40.0, 50.0, 60.0] {
            bridge.update(v, 0.5);
        }
        // The last 4 samples (30, 40, 50, 60) average to 45.
        assert!((bridge.mean_signal() - 45.0).abs() < 1e-5,
            "mean over the windowed last 4 samples, got {}", bridge.mean_signal());
    }
}
