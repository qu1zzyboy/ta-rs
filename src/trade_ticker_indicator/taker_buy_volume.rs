use std::fmt;

use crate::traits::{BatchTradeTickerU64, Reset, TradeTickerU64};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Taker Buy Volume indicator for BatchTradeTicker.
///
/// This indicator calculates the total taker buy volume (sum of price * quantity for trades where
/// the buyer is a taker, not a market maker) from a batch of trade ticks.
///
/// Taker buy volume represents the trading value where buyers actively take liquidity from the market
/// (i.e., `is_mm_buyer() == false`).
///
/// # Formula
///
/// Taker Buy Volume = Σ(price * quantity) for all trades where `is_mm_buyer() == false`
///
/// # Example
///
/// ```
/// use ta::trade_ticker_indicator::TakerBuyVolume;
/// use ta::{BatchTradeTickerU64, TradeTickerU64};
///
/// let mut taker_buy_volume = TakerBuyVolume::new();
/// let total_volume = taker_buy_volume.extract(&batch_trade_ticker);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct TakerBuyVolume;

impl TakerBuyVolume {
    pub fn new() -> Self {
        Self
    }

    /// Extract total taker buy volume from a BatchTradeTicker.
    ///
    /// Returns 0.0 if the window is empty.
    /// Only includes trades where `is_mm_buyer() == false` (taker buy orders).
    pub fn extract<T, B>(&mut self, input: &B) -> f64
    where
        T: TradeTickerU64,
        B: BatchTradeTickerU64<T>,
    {
        match input.get_batch_trade_ticker() {
            Some(ticks) => {
                ticks.iter()
                    .filter(|tick| !tick.is_mm_buyer()) // Only taker buy orders
                    .map(|tick| (tick.get_trade_price() as f64) * (tick.get_trade_quantity() as f64))
                    .sum()
            }
            None => 0.0, // Empty window
        }
    }
}

impl Default for TakerBuyVolume {
    fn default() -> Self {
        Self::new()
    }
}

impl Reset for TakerBuyVolume {
    fn reset(&mut self) {
        // No state to reset
    }
}

impl fmt::Display for TakerBuyVolume {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TakerBuyVolume")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Timestamp;

    // Mock TradeTicker implementation
    #[derive(Debug, Clone)]
    struct MockTradeTick {
        price: u64,
        quantity: u64,
        time: u64,
        is_mm_buyer: bool,
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

        fn is_mm_buyer(&self) -> bool {
            self.is_mm_buyer
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
        let mut taker_buy_volume = TakerBuyVolume::new();
        let window = MockBatchWindow { ticks: None };
        let result = taker_buy_volume.extract(&window);

        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_single_taker_buy_trade() {
        let mut taker_buy_volume = TakerBuyVolume::new();
        let window = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000,
                quantity: 10,
                time: 1000,
                is_mm_buyer: false, // Taker buy
            }]),
        };
        let result = taker_buy_volume.extract(&window);

        // Volume = price * quantity = 10000 * 10 = 100000.0
        assert_eq!(result, 100000.0);
    }

    #[test]
    fn test_single_maker_buy_trade() {
        let mut taker_buy_volume = TakerBuyVolume::new();
        let window = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000,
                quantity: 10,
                time: 1000,
                is_mm_buyer: true, // Maker buy (should be excluded)
            }]),
        };
        let result = taker_buy_volume.extract(&window);

        // Should be 0.0 because maker buy trades are excluded
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_mixed_trades() {
        let mut taker_buy_volume = TakerBuyVolume::new();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000,
                    quantity: 5,
                    time: 1000,
                    is_mm_buyer: false, // Taker buy
                },
                MockTradeTick {
                    price: 10050,
                    quantity: 10,
                    time: 1050,
                    is_mm_buyer: true, // Maker buy (excluded)
                },
                MockTradeTick {
                    price: 10020,
                    quantity: 3,
                    time: 1100,
                    is_mm_buyer: false, // Taker buy
                },
                MockTradeTick {
                    price: 10030,
                    quantity: 8,
                    time: 1150,
                    is_mm_buyer: true, // Maker buy (excluded)
                },
            ]),
        };
        let result = taker_buy_volume.extract(&window);

        // Only taker buy trades: (10000 * 5) + (10020 * 3) = 50000 + 30060 = 80060.0
        assert_eq!(result, 80060.0);
    }

    #[test]
    fn test_all_taker_buy_trades() {
        let mut taker_buy_volume = TakerBuyVolume::new();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000,
                    quantity: 5,
                    time: 1000,
                    is_mm_buyer: false,
                },
                MockTradeTick {
                    price: 10050,
                    quantity: 10,
                    time: 1050,
                    is_mm_buyer: false,
                },
                MockTradeTick {
                    price: 10020,
                    quantity: 3,
                    time: 1100,
                    is_mm_buyer: false,
                },
            ]),
        };
        let result = taker_buy_volume.extract(&window);

        // All are taker buy: (10000 * 5) + (10050 * 10) + (10020 * 3) = 50000 + 100500 + 30060 = 180560.0
        assert_eq!(result, 180560.0);
    }

    #[test]
    fn test_all_maker_buy_trades() {
        let mut taker_buy_volume = TakerBuyVolume::new();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000,
                    quantity: 5,
                    time: 1000,
                    is_mm_buyer: true,
                },
                MockTradeTick {
                    price: 10050,
                    quantity: 10,
                    time: 1050,
                    is_mm_buyer: true,
                },
            ]),
        };
        let result = taker_buy_volume.extract(&window);

        // All are maker buy, should be 0.0
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_reset() {
        let mut taker_buy_volume = TakerBuyVolume::new();
        taker_buy_volume.reset(); // Should not panic
    }

    #[test]
    fn test_display() {
        let taker_buy_volume = TakerBuyVolume::new();
        assert_eq!(format!("{}", taker_buy_volume), "TakerBuyVolume");
    }
}

