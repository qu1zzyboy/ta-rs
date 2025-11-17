use std::fmt;

use crate::traits::{BatchOrderTickerf64, Next2, Reset, OrderTickerf64};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Order Ticker Quantity indicator for BatchOrderTicker (f64 version).
///
/// This indicator returns the number of order tickers in a batch window.
/// It simply counts the number of tickers returned by `get_batch_order_ticker()`.
///
/// # Example
///
/// ```
/// use ta::order_ticker_indicators::OtQuantityF64;
/// use ta::{BatchOrderTickerf64, OrderTickerf64};
///
/// let mut quantity = OtQuantityF64::new();
/// let count = quantity.extract(&batch_order_ticker);
/// // Returns the number of order tickers in the batch
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct OtQuantityF64;

impl OtQuantityF64 {
    pub fn new() -> Self {
        Self
    }

    /// Extract the quantity (count) of order tickers from a BatchOrderTicker.
    ///
    /// Returns 0 if the window is empty or None.
    pub fn extract<T, B>(&mut self, input: &B) -> usize
    where
        T: OrderTickerf64,
        B: BatchOrderTickerf64<T>,
    {
        match input.get_batch_order_ticker() {
            Some(ticks) => ticks.len(),
            None => 0, // Empty window
        }
    }
}

impl<T, B> Next2<&B, T> for OtQuantityF64
where
    T: OrderTickerf64,
    B: BatchOrderTickerf64<T>,
{
    type Output = usize;

    fn next(&mut self, input: &B) -> Self::Output {
        self.extract(input)
    }
}

impl Default for OtQuantityF64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Reset for OtQuantityF64 {
    fn reset(&mut self) {
        // No state to reset
    }
}

impl fmt::Display for OtQuantityF64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OtQuantityF64")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Timestamp;

    // Mock OrderTicker implementation
    #[derive(Debug, Clone)]
    struct MockOrderTicker {
        best_bid_price: f64,
        best_ask_price: f64,
        best_bid_quantity: f64,
        best_ask_quantity: f64,
        time: u64,
    }

    impl Timestamp for MockOrderTicker {
        fn timestamp(&self) -> u64 {
            self.time
        }
    }

    impl OrderTickerf64 for MockOrderTicker {
        fn get_best_bid_price(&self) -> f64 {
            self.best_bid_price
        }

        fn get_best_ask_price(&self) -> f64 {
            self.best_ask_price
        }

        fn get_best_bid_quantity(&self) -> f64 {
            self.best_bid_quantity
        }

        fn get_best_ask_quantity(&self) -> f64 {
            self.best_ask_quantity
        }
    }

    // Mock BatchOrderTicker implementation
    struct MockBatchWindow {
        ticks: Option<Vec<MockOrderTicker>>,
    }

    impl BatchOrderTickerf64<MockOrderTicker> for MockBatchWindow {
        fn get_batch_order_ticker(&self) -> Option<&[MockOrderTicker]> {
            self.ticks.as_deref()
        }
    }

    #[test]
    fn test_empty_window() {
        let mut quantity = OtQuantityF64::new();
        let window = MockBatchWindow { ticks: None };
        let result = quantity.extract(&window);

        assert_eq!(result, 0);
    }

    #[test]
    fn test_single_ticker() {
        let mut quantity = OtQuantityF64::new();
        let window = MockBatchWindow {
            ticks: Some(vec![MockOrderTicker {
                best_bid_price: 10000.0,
                best_ask_price: 10050.0,
                best_bid_quantity: 10.0,
                best_ask_quantity: 10.0,
                time: 1000,
            }]),
        };
        let result = quantity.extract(&window);

        assert_eq!(result, 1);
    }

    #[test]
    fn test_multiple_ticks() {
        let mut quantity = OtQuantityF64::new();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockOrderTicker {
                    best_bid_price: 10000.0,
                    best_ask_price: 10050.0,
                    best_bid_quantity: 10.0,
                    best_ask_quantity: 10.0,
                    time: 1000,
                },
                MockOrderTicker {
                    best_bid_price: 10010.0,
                    best_ask_price: 10060.0,
                    best_bid_quantity: 15.0,
                    best_ask_quantity: 12.0,
                    time: 1100,
                },
                MockOrderTicker {
                    best_bid_price: 10020.0,
                    best_ask_price: 10070.0,
                    best_bid_quantity: 20.0,
                    best_ask_quantity: 18.0,
                    time: 1200,
                },
            ]),
        };
        let result = quantity.extract(&window);

        assert_eq!(result, 3);
    }

    #[test]
    fn test_reset() {
        let mut quantity = OtQuantityF64::new();
        quantity.reset(); // Should not panic
    }

    #[test]
    fn test_display() {
        let quantity = OtQuantityF64::new();
        assert_eq!(format!("{}", quantity), "OtQuantityF64");
    }

    #[test]
    fn test_next_trait() {
        let mut quantity = OtQuantityF64::new();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockOrderTicker {
                    best_bid_price: 10000.0,
                    best_ask_price: 10050.0,
                    best_bid_quantity: 10.0,
                    best_ask_quantity: 10.0,
                    time: 1000,
                },
                MockOrderTicker {
                    best_bid_price: 10010.0,
                    best_ask_price: 10060.0,
                    best_bid_quantity: 15.0,
                    best_ask_quantity: 12.0,
                    time: 1100,
                },
            ]),
        };
        
        // Test using Next2 trait - need to specify type parameters
        let result = <OtQuantityF64 as Next2<&MockBatchWindow, MockOrderTicker>>::next(&mut quantity, &window);
        assert_eq!(result, 2);
        
        // Test with empty window
        let empty_window = MockBatchWindow { ticks: None };
        let result = <OtQuantityF64 as Next2<&MockBatchWindow, MockOrderTicker>>::next(&mut quantity, &empty_window);
        assert_eq!(result, 0);
    }
}

