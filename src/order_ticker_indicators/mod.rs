//! Order Ticker Indicators
//!
//! This module contains indicators that operate on order ticker data (best bid/ask prices).
//! These indicators analyze the spread, mid-price, and other microstructure metrics
//! based on the best bid and ask prices from order ticker data.

pub mod spread_indicator;

pub use spread_indicator::SpreadIndicator;
