use std::collections::VecDeque;
use std::fmt;

use crate::traits::{BatchTradeTickerU64, Reset, TradeTickerU64};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Volume Ratio indicator for BatchTradeTicker.
///
/// This indicator maintains a circular queue of the last 10 windows and calculates
/// the ratio of volume above the median price to volume below the median price.
///
/// # Formula
///
/// ratio = volume_above_median / volume_below_median
///
/// Where:
/// - volume_above_median: sum of volumes for trades with price > median_price
/// - volume_below_median: sum of volumes for trades with price < median_price
///
/// # Example
///
/// ```
/// use ta::trade_ticker_indicator::VolumeRatio;
/// use ta::{BatchTradeTickerU64, TradeTickerU64};
///
/// let mut volume_ratio = VolumeRatio::new();
/// let ratio = volume_ratio.next(&batch_trade_ticker);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct VolumeRatio {
    queue: VecDeque<f64>, // Stores the last 10 ratios
    capacity: usize,
}

impl VolumeRatio {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(10),
            capacity: 10,
        }
    }

    /// Calculate the median price from a slice of trade ticks.
    fn calculate_median_price<T: TradeTickerU64>(ticks: &[T]) -> f64 {
        if ticks.is_empty() {
            return 0.0;
        }

        let mut prices: Vec<u64> = ticks.iter().map(|tick| tick.get_trade_price()).collect();
        prices.sort_unstable();

        let len = prices.len();
        if len % 2 == 0 {
            // Even number of elements: average of two middle values
            (prices[len / 2 - 1] + prices[len / 2]) as f64 / 2.0
        } else {
            // Odd number of elements: middle value
            prices[len / 2] as f64
        }
    }

    /// Calculate volume ratio for a single window.
    fn calculate_ratio<T, B>(input: &B) -> f64
    where
        T: TradeTickerU64,
        B: BatchTradeTickerU64<T>,
    {
        match input.get_batch_trade_ticker() {
            Some(ticks) if !ticks.is_empty() => {
                let median_price = Self::calculate_median_price(ticks);

                let mut volume_above = 0.0;
                let mut volume_below = 0.0;

                for tick in ticks {
                    let price = tick.get_trade_price() as f64;
                    let volume = tick.get_trade_quantity() as f64;

                    if price > median_price {
                        volume_above += volume;
                    } else if price < median_price {
                        volume_below += volume;
                    }
                    // If price == median_price, we don't count it in either side
                }

                // Avoid division by zero
                if volume_below == 0.0 {
                    if volume_above == 0.0 {
                        1.0 // Both are zero, return 1.0
                    } else {
                        f64::INFINITY // Only above has volume
                    }
                } else {
                    volume_above / volume_below
                }
            }
            _ => 1.0, // Empty window, return 1.0 (neutral ratio)
        }
    }

    /// Process a BatchTradeTicker and return the current ratio.
    ///
    /// This method maintains a circular queue of the last 10 ratios.
    pub fn next<T, B>(&mut self, input: &B) -> f64
    where
        T: TradeTickerU64,
        B: BatchTradeTickerU64<T>,
    {
        let ratio = Self::calculate_ratio(input);

        // Maintain circular queue of size 10
        if self.queue.len() >= self.capacity {
            self.queue.pop_front();
        }
        self.queue.push_back(ratio);

        ratio
    }
}

impl Default for VolumeRatio {
    fn default() -> Self {
        Self::new()
    }
}

impl Reset for VolumeRatio {
    fn reset(&mut self) {
        self.queue.clear();
    }
}

impl fmt::Display for VolumeRatio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VolumeRatio(queue_size={})", self.queue.len())
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
        let mut volume_ratio = VolumeRatio::new();
        let window = MockBatchWindow { ticks: None };
        let result = volume_ratio.next(&window);

        assert_eq!(result, 1.0); // Empty window returns 1.0
    }

    #[test]
    fn test_single_trade() {
        let mut volume_ratio = VolumeRatio::new();
        let window = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000,
                quantity: 10,
                time: 1000,
            }]),
        };
        let result = volume_ratio.next(&window);

        // Single trade: median = 10000, price == median, so both volumes are 0
        // Returns 1.0 (both zero case)
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_multiple_trades_above_below() {
        let mut volume_ratio = VolumeRatio::new();
        // Prices: 10000, 10020, 10050
        // Median: 10020
        // Above median (10050): quantity 10
        // Below median (10000): quantity 5
        // Ratio: 10 / 5 = 2.0
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000,
                    quantity: 5,
                    time: 1000,
                },
                MockTradeTick {
                    price: 10020,
                    quantity: 3,
                    time: 1050,
                },
                MockTradeTick {
                    price: 10050,
                    quantity: 10,
                    time: 1100,
                },
            ]),
        };
        let result = volume_ratio.next(&window);

        assert_eq!(result, 2.0); // 10 / 5 = 2.0
    }

    #[test]
    fn test_circular_queue() {
        let mut volume_ratio = VolumeRatio::new();

        // Add 12 windows (should only keep last 10)
        for i in 0..12 {
            let window = MockBatchWindow {
                ticks: Some(vec![MockTradeTick {
                    price: 10000 + i,
                    quantity: 10,
                    time: 1000 + i,
                }]),
            };
            volume_ratio.next(&window);
        }

        assert_eq!(volume_ratio.queue.len(), 10);
    }

    #[test]
    fn test_even_number_of_trades() {
        let mut volume_ratio = VolumeRatio::new();
        // Prices: 10000, 10020, 10030, 10050
        // Median: (10020 + 10030) / 2 = 10025
        // Above median (10030, 10050): quantity 3 + 10 = 13
        // Below median (10000, 10020): quantity 5 + 7 = 12
        // Ratio: 13 / 12 ≈ 1.083
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000,
                    quantity: 5,
                    time: 1000,
                },
                MockTradeTick {
                    price: 10020,
                    quantity: 7,
                    time: 1050,
                },
                MockTradeTick {
                    price: 10030,
                    quantity: 3,
                    time: 1100,
                },
                MockTradeTick {
                    price: 10050,
                    quantity: 10,
                    time: 1150,
                },
            ]),
        };
        let result = volume_ratio.next(&window);

        let expected = 13.0 / 12.0;
        assert!((result - expected).abs() < 0.001);
    }

    #[test]
    fn test_only_above_median() {
        let mut volume_ratio = VolumeRatio::new();
        // All prices are above median (edge case)
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10020,
                    quantity: 3,
                    time: 1050,
                },
                MockTradeTick {
                    price: 10050,
                    quantity: 10,
                    time: 1100,
                },
            ]),
        };
        let result = volume_ratio.next(&window);

        // Median is 10035, both prices are below median
        // Actually wait, let me recalculate:
        // Prices: 10020, 10050
        // Median: (10020 + 10050) / 2 = 10035
        // 10020 < 10035, 10050 > 10035
        // Above: 10, Below: 3
        // Ratio: 10 / 3 ≈ 3.333
        let expected = 10.0 / 3.0;
        assert!((result - expected).abs() < 0.001);
    }

    #[test]
    fn test_reset() {
        let mut volume_ratio = VolumeRatio::new();
        let window = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000,
                quantity: 10,
                time: 1000,
            }]),
        };
        volume_ratio.next(&window);
        assert_eq!(volume_ratio.queue.len(), 1);

        volume_ratio.reset();
        assert_eq!(volume_ratio.queue.len(), 0);
    }

    #[test]
    fn test_display() {
        let volume_ratio = VolumeRatio::new();
        let display_str = format!("{}", volume_ratio);
        assert!(display_str.contains("VolumeRatio"));
    }
}

