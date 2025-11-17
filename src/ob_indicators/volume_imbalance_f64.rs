use std::fmt;

use crate::traits::{Next, Orderbookf64, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Volume Imbalance indicator for orderbook data (f64 version).
///
/// This indicator calculates the ratio of bid volume to ask volume in the orderbook.
/// Volume is calculated as price * quantity for each order level.
/// The result is returned as a floating-point ratio.
///
/// # Formula
///
/// Volume Imbalance = Bid Volume / Ask Volume
///
/// Where:
/// - Bid Volume = Σ(price * quantity) for all bid orders
/// - Ask Volume = Σ(price * quantity) for all ask orders
/// - Result is a ratio (e.g., 1.5 means bid volume is 1.5x ask volume)
///
/// # Parameters
///
/// None - this is a stateless indicator that calculates imbalance from current orderbook state.
///
/// # Example
///
/// ```
/// use ta::ob_indicators::VolumeImbalanceF64;
/// use ta::Next;
/// use ta::Orderbookf64;
/// 
/// // Assuming you have an orderbook that implements Orderbookf64 trait
/// let mut vi = VolumeImbalanceF64::new();
/// let imbalance = vi.next(&orderbook);
/// // imbalance is now an f64 ratio
/// // - 1.0 means balanced (bid volume == ask volume)
/// // - > 1.0 means more bid volume (buying pressure)
/// // - < 1.0 means more ask volume (selling pressure)
/// ```
///
#[doc(alias = "VI_F64")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct VolumeImbalanceF64;

impl VolumeImbalanceF64 {
    /// Creates a new Volume Imbalance indicator.
    pub fn new() -> Self {
        Self
    }
}

impl<T: Orderbookf64> Next<&T> for VolumeImbalanceF64 {
    type Output = f64;

    fn next(&mut self, orderbook: &T) -> Self::Output {
        // Calculate bid volume (sum of price * quantity for all bids)
        let bids_btm = orderbook.get_bids_btm();
        let bid_volume: f64 = bids_btm.iter()
            .map(|(price, quantity)| price.into_inner() * quantity)
            .sum();

        // Calculate ask volume (sum of price * quantity for all asks)
        let asks_btm = orderbook.get_asks_btm();
        let ask_volume: f64 = asks_btm.iter()
            .map(|(price, quantity)| price.into_inner() * quantity)
            .sum();

        // Calculate imbalance ratio
        if ask_volume == 0.0 {
            if bid_volume == 0.0 {
                1.0 // Both are zero, return neutral value
            } else {
                f64::INFINITY // Only bids exist, return infinity
            }
        } else {
            bid_volume / ask_volume
        }
    }
}

impl Reset for VolumeImbalanceF64 {
    fn reset(&mut self) {
        // No state to reset for this stateless indicator
    }
}

impl Default for VolumeImbalanceF64 {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VolumeImbalanceF64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VolumeImbalanceF64")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordered_float::OrderedFloat;
    use std::collections::BTreeMap;

    // Mock orderbook for testing using BTreeMap
    struct MockOrderbook {
        bids: BTreeMap<OrderedFloat<f64>, f64>,
        asks: BTreeMap<OrderedFloat<f64>, f64>,
    }

    impl MockOrderbook {
        fn new() -> Self {
            Self {
                bids: BTreeMap::new(),
                asks: BTreeMap::new(),
            }
        }

        fn add_bid(&mut self, price: f64, quantity: f64) {
            if quantity > 0.0 {
                self.bids.insert(OrderedFloat(price), quantity);
            } else {
                self.bids.remove(&OrderedFloat(price));
            }
        }

        fn add_ask(&mut self, price: f64, quantity: f64) {
            if quantity > 0.0 {
                self.asks.insert(OrderedFloat(price), quantity);
            } else {
                self.asks.remove(&OrderedFloat(price));
            }
        }
    }

    impl Orderbookf64 for MockOrderbook {
        fn get_bids_btm(&self) -> &BTreeMap<OrderedFloat<f64>, f64> {
            &self.bids
        }

        fn get_asks_btm(&self) -> &BTreeMap<OrderedFloat<f64>, f64> {
            &self.asks
        }
    }

    #[test]
    fn test_volume_imbalance_equal_volumes() {
        let mut orderbook = MockOrderbook::new();
        orderbook.add_bid(100.0, 10.0); // bid volume = 1000.0
        orderbook.add_ask(100.0, 10.0); // ask volume = 1000.0

        let mut vi = VolumeImbalanceF64::new();
        let imbalance = vi.next(&orderbook);
        
        // 1000.0 / 1000.0 = 1.0
        assert_eq!(imbalance, 1.0);
    }

    #[test]
    fn test_volume_imbalance_more_bids() {
        let mut orderbook = MockOrderbook::new();
        orderbook.add_bid(100.0, 20.0); // bid volume = 2000.0
        orderbook.add_ask(100.0, 10.0); // ask volume = 1000.0

        let mut vi = VolumeImbalanceF64::new();
        let imbalance = vi.next(&orderbook);
        
        // 2000.0 / 1000.0 = 2.0
        assert_eq!(imbalance, 2.0);
    }

    #[test]
    fn test_volume_imbalance_more_asks() {
        let mut orderbook = MockOrderbook::new();
        orderbook.add_bid(100.0, 10.0); // bid volume = 1000.0
        orderbook.add_ask(100.0, 20.0); // ask volume = 2000.0

        let mut vi = VolumeImbalanceF64::new();
        let imbalance = vi.next(&orderbook);
        
        // 1000.0 / 2000.0 = 0.5
        assert_eq!(imbalance, 0.5);
    }

    #[test]
    fn test_volume_imbalance_no_asks() {
        let mut orderbook = MockOrderbook::new();
        orderbook.add_bid(100.0, 10.0); // bid volume = 1000.0
        // No asks

        let mut vi = VolumeImbalanceF64::new();
        let imbalance = vi.next(&orderbook);
        
        // When ask_volume = 0 and bid_volume > 0, return infinity
        assert_eq!(imbalance, f64::INFINITY);
    }

    #[test]
    fn test_volume_imbalance_no_bids() {
        let mut orderbook = MockOrderbook::new();
        // No bids
        orderbook.add_ask(100.0, 10.0); // ask volume = 1000.0

        let mut vi = VolumeImbalanceF64::new();
        let imbalance = vi.next(&orderbook);
        
        // 0.0 / 1000.0 = 0.0
        assert_eq!(imbalance, 0.0);
    }

    #[test]
    fn test_volume_imbalance_empty_orderbook() {
        let orderbook = MockOrderbook::new();

        let mut vi = VolumeImbalanceF64::new();
        let imbalance = vi.next(&orderbook);
        
        // When both volumes are 0, return neutral value 1.0
        assert_eq!(imbalance, 1.0);
    }

    #[test]
    fn test_volume_imbalance_multiple_levels() {
        let mut orderbook = MockOrderbook::new();
        // Multiple bid levels
        orderbook.add_bid(100.0, 5.0);  // bid volume = 500.0
        orderbook.add_bid(99.0, 10.0);  // bid volume = 990.0
        // Total bid volume = 1490.0
        
        // Multiple ask levels
        orderbook.add_ask(101.0, 8.0);  // ask volume = 808.0
        orderbook.add_ask(102.0, 6.0);  // ask volume = 612.0
        // Total ask volume = 1420.0

        let mut vi = VolumeImbalanceF64::new();
        let imbalance = vi.next(&orderbook);
        
        // 1490.0 / 1420.0 ≈ 1.0493
        let expected = 1490.0 / 1420.0;
        assert!((imbalance - expected).abs() < 0.0001);
    }

    #[test]
    fn test_reset() {
        let mut vi = VolumeImbalanceF64::new();
        vi.reset(); // Should not panic
    }

    #[test]
    fn test_display() {
        let vi = VolumeImbalanceF64::new();
        assert_eq!(format!("{}", vi), "VolumeImbalanceF64");
    }
}


