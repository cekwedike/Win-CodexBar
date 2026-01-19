//! Browser detection and cookie extraction for Windows

pub mod cookies;
pub mod detection;
pub mod watchdog;

// Re-exports for future UI integration
#[allow(unused_imports)]
pub use watchdog::{WebProbeWatchdog, WatchdogConfig, WatchdogError, global_watchdog};
