use std::fmt;

use crate::traits::{BatchTradeTickerf64, Reset, TradeTickerf64};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Volume indicator for BatchTradeTicker (f64 version).
///
/// This indicator calculates the total volume (sum of price * quantity for all trades) from a batch of trade ticks.
/// Volume represents the total trading value (成交额) rather than just the quantity.
///
/// # Example
///
/// ```
/// use ta::trade_ticker_indicator::VolumeF64;
/// use ta::{BatchTradeTickerf64, TradeTickerf64};
///
/// let mut volume = VolumeF64::new();
/// let total_volume = volume.extract(&batch_trade_ticker);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct VolumeF64;

impl VolumeF64 {
    pub fn new() -> Self {
        Self
    }

    /// Extract total volume from a BatchTradeTicker.
    ///
    /// Returns 0.0 if the window is empty.
    /// Volume is calculated as the sum of price * quantity for all trades.
    pub fn extract<T, B>(&mut self, input: &B) -> f64
    where
        T: TradeTickerf64,
        B: BatchTradeTickerf64<T>,
    {
        match input.get_batch_trade_ticker() {
            Some(ticks) => {
                ticks.iter()
                    .map(|tick| tick.get_trade_price() * tick.get_trade_quantity())
                    .sum()
            }
            None => 0.0, // Empty window
        }
    }
}

impl Default for VolumeF64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Reset for VolumeF64 {
    fn reset(&mut self) {
        // No state to reset
    }
}

impl fmt::Display for VolumeF64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VolumeF64")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Timestamp;

    // Mock TradeTicker implementation
    #[derive(Debug, Clone)]
    struct MockTradeTick {
        price: f64,
        quantity: f64,
        time: u64,
    }

    impl Timestamp for MockTradeTick {
        fn timestamp(&self) -> u64 {
            self.time
        }
    }

    impl TradeTickerf64 for MockTradeTick {
        fn get_trade_price(&self) -> f64 {
            self.price
        }

        fn get_trade_quantity(&self) -> f64 {
            self.quantity
        }

        fn is_mm_buyer(&self) -> bool {
            false // Default value for testing
        }
    }

    // Mock BatchTradeTicker implementation
    struct MockBatchWindow {
        ticks: Option<Vec<MockTradeTick>>,
    }

    impl BatchTradeTickerf64<MockTradeTick> for MockBatchWindow {
        fn get_batch_trade_ticker(&self) -> Option<&[MockTradeTick]> {
            self.ticks.as_deref()
        }
    }

    #[test]
    fn test_empty_window() {
        let mut volume = VolumeF64::new();
        let window = MockBatchWindow { ticks: None };
        let result = volume.extract(&window);

        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_single_trade() {
        let mut volume = VolumeF64::new();
        let window = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000.0,
                quantity: 10.0,
                time: 1000,
            }]),
        };
        let result = volume.extract(&window);

        // Volume = price * quantity = 10000.0 * 10.0 = 100000.0
        assert_eq!(result, 100000.0);
    }

    #[test]
    fn test_multiple_trades() {
        let mut volume = VolumeF64::new();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000.0,
                    quantity: 5.0,
                    time: 1000,
                },
                MockTradeTick {
                    price: 10050.0,
                    quantity: 10.0,
                    time: 1050,
                },
                MockTradeTick {
                    price: 10020.0,
                    quantity: 3.0,
                    time: 1100,
                },
            ]),
        };
        let result = volume.extract(&window);

        // Volume = (10000.0 * 5.0) + (10050.0 * 10.0) + (10020.0 * 3.0)
        //        = 50000.0 + 100500.0 + 30060.0
        //        = 180560.0
        assert_eq!(result, 180560.0);
    }

    #[test]
    fn test_reset() {
        let mut volume = VolumeF64::new();
        volume.reset(); // Should not panic
    }

    #[test]
    fn test_display() {
        let volume = VolumeF64::new();
        assert_eq!(format!("{}", volume), "VolumeF64");
    }
}

