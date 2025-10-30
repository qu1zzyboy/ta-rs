//! Orderbook-based indicators
//!
//! This module contains indicators that operate on orderbook data rather than OHLCV data.
//! These indicators analyze the depth of market information including bid/ask spreads,
//! volume imbalances, and other microstructure metrics.

pub mod volume_imbalance;

pub use volume_imbalance::VolumeImbalance;
