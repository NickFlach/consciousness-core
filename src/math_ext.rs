//! Float-math extension trait for `no_std` builds.
//!
//! `core` does not provide `f32::ln`, `f32::sqrt`, `f32::cos`, `f32::sin`,
//! `f32::tanh`, `f32::exp`, `f32::atan2`, or `f32::log2` — those live in
//! `std`. This module re-exposes them under the same names by routing
//! through `libm`, but only when the `std` feature is disabled.
//!
//! Under `std` this module is empty; the native f32 methods win.
//!
//! Call-sites in `kuramoto.rs`, `iit.rs`, `wave.rs`, and `metrics.rs` add
//! `#[cfg(not(feature = "std"))] use crate::math_ext::F32Ext;` so the
//! existing method-style syntax keeps working in both build modes.

#![cfg(not(feature = "std"))]
#![allow(dead_code)] // Some helpers are forward-looking (sin, abs, exp on f64).

pub trait F32Ext {
    fn ln(self) -> f32;
    fn log2(self) -> f32;
    fn sqrt(self) -> f32;
    fn cos(self) -> f32;
    fn sin(self) -> f32;
    fn tanh(self) -> f32;
    fn exp(self) -> f32;
    fn abs(self) -> f32;
    fn atan2(self, other: f32) -> f32;
    fn powi(self, n: i32) -> f32;
}

impl F32Ext for f32 {
    #[inline] fn ln(self)   -> f32 { libm::logf(self) }
    #[inline] fn log2(self) -> f32 { libm::log2f(self) }
    #[inline] fn sqrt(self) -> f32 { libm::sqrtf(self) }
    #[inline] fn cos(self)  -> f32 { libm::cosf(self) }
    #[inline] fn sin(self)  -> f32 { libm::sinf(self) }
    #[inline] fn tanh(self) -> f32 { libm::tanhf(self) }
    #[inline] fn exp(self)  -> f32 { libm::expf(self) }
    #[inline] fn abs(self)  -> f32 { libm::fabsf(self) }
    #[inline] fn atan2(self, other: f32) -> f32 { libm::atan2f(self, other) }
    #[inline] fn powi(self, n: i32) -> f32 { libm::powf(self, n as f32) }
}

/// f64 helpers used by wave.rs decay + interference math.
pub trait F64Ext {
    fn ln(self) -> f64;
    fn exp(self) -> f64;
    fn cos(self) -> f64;
    fn sin(self) -> f64;
    fn sqrt(self) -> f64;
    fn abs(self) -> f64;
}

impl F64Ext for f64 {
    #[inline] fn ln(self)   -> f64 { libm::log(self) }
    #[inline] fn exp(self)  -> f64 { libm::exp(self) }
    #[inline] fn cos(self)  -> f64 { libm::cos(self) }
    #[inline] fn sin(self)  -> f64 { libm::sin(self) }
    #[inline] fn sqrt(self) -> f64 { libm::sqrt(self) }
    #[inline] fn abs(self)  -> f64 { libm::fabs(self) }
}
