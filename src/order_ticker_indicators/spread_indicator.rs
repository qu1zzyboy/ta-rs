use std::fmt;
use crate::traits::{Next, OrderTicker, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Spread indicator for OrderTicker data.
///
/// This indicator calculates the average spread over a specified period
/// from OrderTicker data. Spread is calculated as ask_price - bid_price.
///
/// # Formula
///
/// Average Spread = Σ(spread_i) / period
///
/// Where:
/// - spread_i = ask_price_i - bid_price_i for each tick
/// - period = number of ticks to average over
///
/// # Parameters
///
/// - `period`: Number of ticks to include in the average calculation
///
/// # Example
///
/// ```
/// use ta::order_ticker_indicators::SpreadIndicator;
/// use ta::Next;
/// use ta::OrderTicker;
/// 
/// // Assuming you have an OrderTickBuffer that implements OrderTicker trait
/// let mut spread = SpreadIndicator::new(10);
/// let avg_spread = spread.next(&order_tick_buffer);
/// // avg_spread is now the average spread over the last 10 ticks
/// ```
///
#[doc(alias = "SPREAD")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct SpreadIndicator {
    period: usize,
    spreads: std::collections::VecDeque<u64>,
}

impl SpreadIndicator {
    /// Creates a new Spread indicator with the specified period.
    pub fn new(period: usize) -> Self {
        Self {
            period,
            spreads: std::collections::VecDeque::new(),
        }
    }
}

impl<T: OrderTicker> Next<&T> for SpreadIndicator {
    type Output = u64;

    fn next(&mut self, ticker: &T) -> Self::Output {
        // Calculate current spread
        let current_spread = if ticker.get_best_ask_price() > ticker.get_best_bid_price() {
            ticker.get_best_ask_price() - ticker.get_best_bid_price()
        } else {
            0
        };

        // Add current spread to the buffer
        self.spreads.push_back(current_spread);

        // Remove old spreads if we exceed the period
        while self.spreads.len() > self.period {
            self.spreads.pop_front();
        }

        // Calculate average spread
        if self.spreads.is_empty() {
            0
        } else {
            let sum: u64 = self.spreads.iter().sum();
            sum / self.spreads.len() as u64
        }
    }
}

impl Reset for SpreadIndicator {
    fn reset(&mut self) {
        self.spreads.clear();
    }
}

impl Default for SpreadIndicator {
    fn default() -> Self {
        Self::new(10)
    }
}

impl fmt::Display for SpreadIndicator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SpreadIndicator(period={})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock OrderTicker for testing
    #[allow(dead_code)]
    struct MockOrderTicker {
        best_bid_price: u64,
        best_ask_price: u64,
        best_bid_quantity: u64,
        best_ask_quantity: u64,
        timestamp: u64,
        exchange: String,
        symbol: String,
    }

    impl MockOrderTicker {
        fn new(bid_price: u64, ask_price: u64, bid_qty: u64, ask_qty: u64) -> Self {
            Self {
                best_bid_price: bid_price,
                best_ask_price: ask_price,
                best_bid_quantity: bid_qty,
                best_ask_quantity: ask_qty,
                timestamp: 1000,
                exchange: "TEST".to_string(),
                symbol: "BTCUSDT".to_string(),
            }
        }
    }

    impl OrderTicker for MockOrderTicker {
        fn get_best_bid_price(&self) -> u64 {
            self.best_bid_price
        }

        fn get_best_ask_price(&self) -> u64 {
            self.best_ask_price
        }

        fn get_best_bid_quantity(&self) -> u64 {
            self.best_bid_quantity
        }

        fn get_best_ask_quantity(&self) -> u64 {
            self.best_ask_quantity
        }
    }

    impl MockOrderTicker {
        #[allow(dead_code)]
        fn get_timestamp(&self) -> u64 {
            self.timestamp
        }

        #[allow(dead_code)]
        fn get_exchange(&self) -> &str {
            &self.exchange
        }

        #[allow(dead_code)]
        fn get_symbol(&self) -> &str {
            &self.symbol
        }
    }

    #[test]
    fn test_spread_indicator_single_tick() {
        let ticker = MockOrderTicker::new(10000, 10050, 10, 10);
        let mut spread = SpreadIndicator::new(1);
        let result = spread.next(&ticker);
        
        // Spread = 10050 - 10000 = 50
        assert_eq!(result, 50);
    }

    #[test]
    fn test_spread_indicator_multiple_ticks() {
        let mut spread = SpreadIndicator::new(3);
        
        // First tick: spread = 50
        let ticker1 = MockOrderTicker::new(10000, 10050, 10, 10);
        let result1 = spread.next(&ticker1);
        assert_eq!(result1, 50);
        
        // Second tick: spread = 60
        let ticker2 = MockOrderTicker::new(10010, 10070, 10, 10);
        let result2 = spread.next(&ticker2);
        assert_eq!(result2, 55); // (50 + 60) / 2 = 55
        
        // Third tick: spread = 40
        let ticker3 = MockOrderTicker::new(10020, 10060, 10, 10);
        let result3 = spread.next(&ticker3);
        assert_eq!(result3, 50); // (50 + 60 + 40) / 3 = 50
    }

    #[test]
    fn test_spread_indicator_zero_spread() {
        let ticker = MockOrderTicker::new(10000, 10000, 10, 10);
        let mut spread = SpreadIndicator::new(1);
        let result = spread.next(&ticker);
        
        // Spread = 10000 - 10000 = 0
        assert_eq!(result, 0);
    }

    #[test]
    fn test_spread_indicator_negative_spread() {
        let ticker = MockOrderTicker::new(10050, 10000, 10, 10);
        let mut spread = SpreadIndicator::new(1);
        let result = spread.next(&ticker);
        
        // When ask < bid, spread = 0
        assert_eq!(result, 0);
    }

    #[test]
    fn test_reset() {
        let mut spread = SpreadIndicator::new(3);
        
        // Add some data
        let ticker = MockOrderTicker::new(10000, 10050, 10, 10);
        spread.next(&ticker);
        
        // Reset should clear the buffer
        spread.reset();
        let result = spread.next(&ticker);
        assert_eq!(result, 50); // Should work as if it's the first tick
    }

    #[test]
    fn test_display() {
        let spread = SpreadIndicator::new(5);
        assert_eq!(format!("{}", spread), "SpreadIndicator(period=5)");
    }
}
