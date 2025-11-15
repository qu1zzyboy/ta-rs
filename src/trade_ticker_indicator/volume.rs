use std::fmt;

use crate::traits::{BatchTradeTickerU64, Reset, TradeTickerU64};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Volume indicator for BatchTradeTicker.
///
/// This indicator calculates the total volume (sum of all trade quantities) from a batch of trade ticks.
///
/// # Example
///
/// ```
/// use ta::trade_ticker_indicator::Volume;
/// use ta::{BatchTradeTickerU64, TradeTickerU64};
///
/// let mut volume = Volume::new();
/// let total_volume = volume.extract(&batch_trade_ticker);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Volume;

impl Volume {
    pub fn new() -> Self {
        Self
    }

    /// Extract total volume from a BatchTradeTicker.
    ///
    /// Returns 0.0 if the window is empty.
    pub fn extract<T, B>(&mut self, input: &B) -> f64
    where
        T: TradeTickerU64,
        B: BatchTradeTickerU64<T>,
    {
        match input.get_batch_trade_ticker() {
            Some(ticks) => {
                ticks.iter().map(|tick| tick.get_trade_quantity() as f64).sum()
            }
            None => 0.0, // Empty window
        }
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::new()
    }
}

impl Reset for Volume {
    fn reset(&mut self) {
        // No state to reset
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Volume")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock TradeTicker implementation
    #[derive(Debug, Clone)]
    struct MockTradeTick {
        price: u64,
        quantity: u64,
        time: u64,
    }

    impl Timestamp for MockTradeTick {
        fn timestamp(&self) -> u64 {
            self.time
        }
    }

    impl TradeTickerU64 for MockTradeTick {
        fn get_trade_price(&self) -> u64 {
            self.price
        }

        fn get_trade_quantity(&self) -> u64 {
            self.quantity
        }
    }

    // Mock BatchTradeTicker implementation
    struct MockBatchWindow {
        ticks: Option<Vec<MockTradeTick>>,
    }

    impl BatchTradeTickerU64<MockTradeTick> for MockBatchWindow {
        fn get_batch_trade_ticker(&self) -> Option<&[MockTradeTick]> {
            self.ticks.as_deref()
        }
    }

    #[test]
    fn test_empty_window() {
        let mut volume = Volume::new();
        let window = MockBatchWindow { ticks: None };
        let result = volume.extract(&window);

        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_single_trade() {
        let mut volume = Volume::new();
        let window = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000,
                quantity: 10,
                time: 1000,
            }]),
        };
        let result = volume.extract(&window);

        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_multiple_trades() {
        let mut volume = Volume::new();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000,
                    quantity: 5,
                    time: 1000,
                },
                MockTradeTick {
                    price: 10050,
                    quantity: 10,
                    time: 1050,
                },
                MockTradeTick {
                    price: 10020,
                    quantity: 3,
                    time: 1100,
                },
            ]),
        };
        let result = volume.extract(&window);

        assert_eq!(result, 18.0); // 5 + 10 + 3
    }

    #[test]
    fn test_reset() {
        let mut volume = Volume::new();
        volume.reset(); // Should not panic
    }

    #[test]
    fn test_display() {
        let volume = Volume::new();
        assert_eq!(format!("{}", volume), "Volume");
    }
}

