use std::fmt;

use crate::traits::{Next, OrderbookU64, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Volume Imbalance indicator for orderbook data.
///
/// This indicator calculates the ratio of bid volume to ask volume in the orderbook.
/// Volume is calculated as price * quantity for each order level.
/// The result is returned as an integer scaled by 10000 (basis points).
///
/// # Formula
///
/// Volume Imbalance = (Bid Volume * 10000) / Ask Volume
///
/// Where:
/// - Bid Volume = Σ(price * quantity) for all bid orders
/// - Ask Volume = Σ(price * quantity) for all ask orders
/// - Result is scaled by 10000 (e.g., 1.5 becomes 15000, 0.5 becomes 5000)
///
/// # Precision and Overflow Handling
///
/// This implementation uses u64 arithmetic throughout to maintain consistency with u64-based systems.
/// Overflow is handled gracefully:
/// - If volume calculation overflows, the maximum u64 value is used
/// - If ratio calculation overflows, the maximum u64 value is returned
/// - This sacrifices some precision for system consistency
///
/// # Parameters
///
/// None - this is a stateless indicator that calculates imbalance from current orderbook state.
///
/// # Example
///
/// ```
/// use ta::ob_indicators::VolumeImbalance;
/// use ta::Next;
/// use ta::OrderbookU64;
/// 
/// // Assuming you have an orderbook that implements Orderbook trait
/// let mut vi = VolumeImbalance::new();
/// let imbalance = vi.next(&orderbook);
/// // imbalance is now a u64 value scaled by 10000
/// // To get the actual ratio: imbalance as f64 / 10000.0
/// ```
///
#[doc(alias = "VI")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct VolumeImbalance;

impl VolumeImbalance {
    /// Creates a new Volume Imbalance indicator.
    pub fn new() -> Self {
        Self
    }
}

impl<T: OrderbookU64> Next<&T> for VolumeImbalance {
    type Output = u64;

    fn next(&mut self, orderbook: &T) -> Self::Output {
        // Calculate bid volume (sum of price * quantity for all bids)
        // Prefer BTreeMap for better performance, fallback to iterator
        let mut bid_volume: u64 = 0;
        
        // Use BTreeMap for better performance
        let bids_btm = orderbook.get_bids_btm();
        for (&price, &quantity) in bids_btm.iter() {
            // Check for overflow before multiplication
            if let Some(volume) = price.checked_mul(quantity) {
                if let Some(new_total) = bid_volume.checked_add(volume) {
                    bid_volume = new_total;
                } else {
                    // Overflow occurred, use maximum value
                    bid_volume = u64::MAX;
                    break;
                }
            } else {
                // Overflow occurred, use maximum value
                bid_volume = u64::MAX;
                break;
            }
        }

        // Calculate ask volume (sum of price * quantity for all asks)
        // Prefer BTreeMap for better performance, fallback to iterator
        let mut ask_volume: u64 = 0;
        
        // Use BTreeMap for better performance
        let asks_btm = orderbook.get_asks_btm();
        for (&price, &quantity) in asks_btm.iter() {
            // Check for overflow before multiplication
            if let Some(volume) = price.checked_mul(quantity) {
                if let Some(new_total) = ask_volume.checked_add(volume) {
                    ask_volume = new_total;
                } else {
                    // Overflow occurred, use maximum value
                    ask_volume = u64::MAX;
                    break;
                }
            } else {
                // Overflow occurred, use maximum value
                ask_volume = u64::MAX;
                break;
            }
        }

        // Calculate imbalance ratio using integer arithmetic
        // We'll return the ratio scaled by 10000 (basis points)
        // e.g., 1.5 becomes 15000, 0.5 becomes 5000
        if ask_volume == 0 {
            if bid_volume == 0 {
                10000 // Both are zero, return neutral value (1.0 * 10000)
            } else {
                u64::MAX // Only bids exist, return maximum value
            }
        } else {
            // Calculate (bid_volume * 10000) / ask_volume
            // Check for overflow in multiplication
            if let Some(scaled_bid) = bid_volume.checked_mul(10000) {
                scaled_bid / ask_volume
            } else {
                // Overflow occurred, return maximum value
                u64::MAX
            }
        }
    }
}

impl Reset for VolumeImbalance {
    fn reset(&mut self) {
        // No state to reset for this stateless indicator
    }
}

impl Default for VolumeImbalance {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VolumeImbalance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VolumeImbalance")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{U64Price, U64Quantity};

    // Mock orderbook for testing using BTreeMap
    struct MockOrderbook {
        bids: std::collections::BTreeMap<U64Price, U64Quantity>,
        asks: std::collections::BTreeMap<U64Price, U64Quantity>,
    }

    impl MockOrderbook {
        fn new() -> Self {
            Self {
                bids: std::collections::BTreeMap::new(),
                asks: std::collections::BTreeMap::new(),
            }
        }

        fn add_bid(&mut self, price: U64Price, quantity: U64Quantity) {
            if quantity > 0 {
                self.bids.insert(price, quantity);
            } else {
                self.bids.remove(&price);
            }
        }

        fn add_ask(&mut self, price: U64Price, quantity: U64Quantity) {
            if quantity > 0 {
                self.asks.insert(price, quantity);
            } else {
                self.asks.remove(&price);
            }
        }
    }

    impl OrderbookU64 for MockOrderbook {
        fn get_bids_btm(&self) -> &std::collections::BTreeMap<U64Price, U64Quantity> {
            &self.bids
        }

        fn get_asks_btm(&self) -> &std::collections::BTreeMap<U64Price, U64Quantity> {
            &self.asks
        }
    }

    #[test]
    fn test_volume_imbalance_equal_volumes() {
        let mut orderbook = MockOrderbook::new();
        orderbook.add_bid(100, 10); // bid volume = 1000
        orderbook.add_ask(100, 10); // ask volume = 1000

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // (1000 * 10000) / 1000 = 10000 (scaled ratio of 1.0)
        assert_eq!(imbalance, 10000);
    }

    #[test]
    fn test_volume_imbalance_more_bids() {
        let mut orderbook = MockOrderbook::new();
        orderbook.add_bid(100, 20); // bid volume = 2000
        orderbook.add_ask(100, 10); // ask volume = 1000

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // (2000 * 10000) / 1000 = 20000 (scaled ratio of 2.0)
        assert_eq!(imbalance, 20000);
    }

    #[test]
    fn test_volume_imbalance_more_asks() {
        let mut orderbook = MockOrderbook::new();
        orderbook.add_bid(100, 10); // bid volume = 1000
        orderbook.add_ask(100, 20); // ask volume = 2000

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // (1000 * 10000) / 2000 = 5000 (scaled ratio of 0.5)
        assert_eq!(imbalance, 5000);
    }

    #[test]
    fn test_volume_imbalance_no_asks() {
        let mut orderbook = MockOrderbook::new();
        orderbook.add_bid(100, 10); // bid volume = 1000
        // No asks

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // When ask_volume = 0 and bid_volume > 0, return u64::MAX
        assert_eq!(imbalance, u64::MAX);
    }

    #[test]
    fn test_volume_imbalance_no_bids() {
        let mut orderbook = MockOrderbook::new();
        // No bids
        orderbook.add_ask(100, 10); // ask volume = 1000

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // (0 * 10000) / 1000 = 0 (scaled ratio of 0.0)
        assert_eq!(imbalance, 0);
    }

    #[test]
    fn test_volume_imbalance_empty_orderbook() {
        let orderbook = MockOrderbook::new();

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // When both volumes are 0, return neutral value 10000 (scaled ratio of 1.0)
        assert_eq!(imbalance, 10000);
    }

    #[test]
    fn test_volume_imbalance_multiple_levels() {
        let mut orderbook = MockOrderbook::new();
        // Multiple bid levels
        orderbook.add_bid(100, 5);  // bid volume = 500
        orderbook.add_bid(99, 10);  // bid volume = 990
        // Total bid volume = 1490
        
        // Multiple ask levels
        orderbook.add_ask(101, 8);  // ask volume = 808
        orderbook.add_ask(102, 6);  // ask volume = 612
        // Total ask volume = 1420

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // (1490 * 10000) / 1420 = 10492 (scaled ratio of 1.0492)
        let expected = (1490 * 10000) / 1420;
        assert_eq!(imbalance, expected);
    }

    #[test]
    fn test_reset() {
        let mut vi = VolumeImbalance::new();
        vi.reset(); // Should not panic
    }

    #[test]
    fn test_display() {
        let vi = VolumeImbalance::new();
        assert_eq!(format!("{}", vi), "VolumeImbalance");
    }

    #[test]
    fn test_overflow_handling() {
        let mut orderbook = MockOrderbook::new();
        
        // Test multiplication overflow: u64::MAX * 2 would overflow
        orderbook.add_bid(u64::MAX, 2);
        orderbook.add_ask(1, 1);

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // Should handle overflow gracefully by returning u64::MAX
        assert_eq!(imbalance, u64::MAX);
    }

    #[test]
    fn test_addition_overflow() {
        let mut orderbook = MockOrderbook::new();
        
        // Test addition overflow: multiple large values that sum to overflow
        orderbook.add_bid(u64::MAX / 2, 1);
        orderbook.add_bid(u64::MAX / 2, 1);
        orderbook.add_bid(1, 1); // This should cause overflow
        orderbook.add_ask(1, 1);

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // Should handle overflow gracefully
        assert_eq!(imbalance, u64::MAX);
    }

    #[test]
    fn test_scaling_overflow() {
        let mut orderbook = MockOrderbook::new();
        
        // Test scaling overflow: bid_volume * 10000 would overflow
        // Use a value that when multiplied by 10000 would exceed u64::MAX
        let large_volume = u64::MAX / 10000 + 1;
        orderbook.add_bid(large_volume, 1);
        orderbook.add_ask(1, 1);

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&orderbook);
        
        // Should handle overflow gracefully
        assert_eq!(imbalance, u64::MAX);
    }
}
