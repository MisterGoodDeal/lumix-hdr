//! FFI vers Lmxptpif via le shim C++ `extern "C"` (Windows uniquement).
//!
//! Les constantes et signatures sont lues depuis `vendor/lumix-sdk/Library/`.
//! Ne jamais inventer de symboles SDK.

#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

#[cfg(windows)]
mod windows_ffi;

#[cfg(windows)]
pub use windows_ffi::*;

#[cfg(not(windows))]
mod stub;

#[cfg(not(windows))]
pub use stub::*;
