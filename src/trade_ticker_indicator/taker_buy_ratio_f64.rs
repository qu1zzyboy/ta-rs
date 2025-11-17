use std::fmt;

use crate::indicators::ExponentialMovingAverage;
use crate::traits::{BatchTradeTickerf64, Next, Period, Reset, TradeTickerf64};
use crate::trade_ticker_indicator::VolumeF64;
use crate::trade_ticker_indicator::TakerBuyVolumeF64;
use crate::errors::Result;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Taker Buy Ratio indicator for BatchTradeTicker (f64 version).
///
/// This indicator calculates the ratio of taker buy volume to total volume, with smoothing
/// to handle sparse or low-volume periods (e.g., 100ms windows with no trades or small volume).
///
/// The ratio indicates buying pressure: higher values suggest higher probability of upward price movement.
///
/// # Formula
///
/// Raw Ratio = taker_buy_volume / total_volume
///
/// Smoothed Ratio = EMA(Raw Ratio) with configurable period
///
/// # Smoothing Strategy
///
/// 1. **Minimum Volume Threshold**: If total_volume < min_volume_threshold, use the previous
///    smoothed value (or neutral value 0.5 if no previous value exists) to avoid noise from
///    low-volume periods.
///
/// 2. **EMA Smoothing**: Apply exponential moving average to the raw ratios to reduce volatility
///    and provide a more stable signal.
///
/// 3. **Window-based Smoothing**: Optionally maintain a sliding window of recent ratios for
///    additional stability.
///
/// # Parameters
///
/// * `ema_period` - Period for exponential moving average smoothing (default: 10)
/// * `min_volume_threshold` - Minimum total volume to consider a valid ratio (default: 0.0, meaning no threshold)
///
/// # Example
///
/// ```
/// use ta::trade_ticker_indicator::TakerBuyRatioF64;
/// use ta::{BatchTradeTickerf64, TradeTickerf64};
///
/// let mut ratio = TakerBuyRatioF64::new(10, 1000.0);
/// let smoothed_ratio = ratio.next(&batch_trade_ticker);
/// // Returns a value between 0.0 and 1.0, where higher values indicate stronger buying pressure
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct TakerBuyRatioF64 {
    /// EMA for smoothing the ratio
    ema: ExponentialMovingAverage,
    /// Volume indicator for calculating total volume
    volume: VolumeF64,
    /// Taker buy volume indicator for calculating taker buy volume
    taker_buy_volume: TakerBuyVolumeF64,
    /// Minimum volume threshold to consider a valid ratio
    min_volume_threshold: f64,
    /// Last valid EMA value (used when volume is below threshold)
    last_valid_ema: Option<f64>,
}

impl TakerBuyRatioF64 {
    /// Creates a new TakerBuyRatioF64 indicator.
    ///
    /// # Parameters
    ///
    /// * `ema_period` - Period for EMA smoothing (must be > 0)
    /// * `min_volume_threshold` - Minimum total volume to consider valid (>= 0.0)
    ///
    /// # Returns
    ///
    /// Returns `Err` if ema_period is 0.
    pub fn new(ema_period: usize, min_volume_threshold: f64) -> Result<Self> {
        Ok(Self {
            ema: ExponentialMovingAverage::new(ema_period)?,
            volume: VolumeF64::new(),
            taker_buy_volume: TakerBuyVolumeF64::new(),
            min_volume_threshold: min_volume_threshold.max(0.0),
            last_valid_ema: None,
        })
    }

    /// Creates a new TakerBuyRatioF64 with default parameters (EMA period: 10, no volume threshold).
    pub fn default() -> Self {
        Self::new(10, 0.0).unwrap()
    }


    /// Process a BatchTradeTicker and return the smoothed taker buy ratio.
    ///
    /// Returns a value between 0.0 and 1.0, where:
    /// - 0.0 means all volume is from maker buy (no taker buy pressure)
    /// - 1.0 means all volume is from taker buy (maximum buying pressure)
    /// - 0.5 is neutral (balanced)
    pub fn next<T, B>(&mut self, input: &B) -> f64
    where
        T: TradeTickerf64,
        B: BatchTradeTickerf64<T>,
    {
        // Calculate total volume using VolumeF64
        let total_volume = self.volume.extract(input);

        // Check if volume meets threshold
        if total_volume < self.min_volume_threshold {
            // Use previous EMA value or neutral value
            return self.last_valid_ema.unwrap_or(0.5);
        }

        // Calculate taker buy volume using TakerBuyVolumeF64
        let taker_buy_volume = self.taker_buy_volume.extract(input);

        // Calculate raw ratio
        if total_volume == 0.0 {
            // No valid ratio, use previous EMA value or neutral
            return self.last_valid_ema.unwrap_or(0.5);
        }

        let raw_ratio = taker_buy_volume / total_volume;

        // Apply EMA smoothing using the existing EMA indicator
        let smoothed_ratio = self.ema.next(raw_ratio);
        self.last_valid_ema = Some(smoothed_ratio);
        smoothed_ratio
    }
}

impl Default for TakerBuyRatioF64 {
    fn default() -> Self {
        Self::default()
    }
}

impl Reset for TakerBuyRatioF64 {
    fn reset(&mut self) {
        self.ema.reset();
        self.volume.reset();
        self.taker_buy_volume.reset();
        self.last_valid_ema = None;
    }
}

impl fmt::Display for TakerBuyRatioF64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TakerBuyRatioF64(ema_period={}, min_volume={})", 
               self.ema.period(), self.min_volume_threshold)
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
        is_mm_buyer: bool,
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
            self.is_mm_buyer
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
    fn test_new() {
        assert!(TakerBuyRatioF64::new(0, 0.0).is_err());
        assert!(TakerBuyRatioF64::new(10, 0.0).is_ok());
    }

    #[test]
    fn test_empty_window() {
        let mut ratio = TakerBuyRatioF64::new(10, 0.0).unwrap();
        let window = MockBatchWindow { ticks: None };
        let result = ratio.next(&window);

        // Should return neutral value 0.5 for first empty window
        assert_eq!(result, 0.5);
    }

    #[test]
    fn test_all_taker_buy() {
        let mut ratio = TakerBuyRatioF64::new(10, 0.0).unwrap();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000.0,
                    quantity: 10.0,
                    time: 1000,
                    is_mm_buyer: false, // Taker buy
                },
                MockTradeTick {
                    price: 10050.0,
                    quantity: 5.0,
                    time: 1050,
                    is_mm_buyer: false, // Taker buy
                },
            ]),
        };
        let result = ratio.next(&window);

        // All are taker buy, ratio should be 1.0
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_all_maker_buy() {
        let mut ratio = TakerBuyRatioF64::new(10, 0.0).unwrap();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000.0,
                    quantity: 10.0,
                    time: 1000,
                    is_mm_buyer: true, // Maker buy
                },
            ]),
        };
        let result = ratio.next(&window);

        // All are maker buy, ratio should be 0.0
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_mixed_trades() {
        let mut ratio = TakerBuyRatioF64::new(10, 0.0).unwrap();
        let window = MockBatchWindow {
            ticks: Some(vec![
                MockTradeTick {
                    price: 10000.0,
                    quantity: 10.0, // Volume: 100000
                    time: 1000,
                    is_mm_buyer: false, // Taker buy
                },
                MockTradeTick {
                    price: 10000.0,
                    quantity: 10.0, // Volume: 100000
                    time: 1050,
                    is_mm_buyer: true, // Maker buy
                },
            ]),
        };
        let result = ratio.next(&window);

        // Taker: 100000, Total: 200000, Ratio: 0.5
        assert_eq!(result, 0.5);
    }

    #[test]
    fn test_volume_threshold() {
        let mut ratio = TakerBuyRatioF64::new(10, 100000.0).unwrap();
        
        // First window with low volume (below threshold)
        let window1 = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000.0,
                quantity: 5.0, // Volume: 50000 < threshold
                time: 1000,
                is_mm_buyer: false,
            }]),
        };
        let result1 = ratio.next(&window1);
        // Should return neutral value 0.5
        assert_eq!(result1, 0.5);

        // Second window with sufficient volume
        let window2 = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000.0,
                quantity: 20.0, // Volume: 200000 >= threshold
                time: 1100,
                is_mm_buyer: false,
            }]),
        };
        let result2 = ratio.next(&window2);
        // Should calculate actual ratio (1.0 for all taker buy)
        assert_eq!(result2, 1.0);

        // Third window with low volume again
        let window3 = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000.0,
                quantity: 5.0, // Volume: 50000 < threshold
                time: 1200,
                is_mm_buyer: true, // Maker buy
            }]),
        };
        let result3 = ratio.next(&window3);
        // Should use previous EMA value (close to 1.0, not 0.0)
        assert!(result3 > 0.9);
    }

    #[test]
    fn test_ema_smoothing() {
        let mut ratio = TakerBuyRatioF64::new(3, 0.0).unwrap(); // Short period for testing
        
        // First value: 1.0 (all taker buy)
        let window1 = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000.0,
                quantity: 10.0,
                time: 1000,
                is_mm_buyer: false,
            }]),
        };
        let result1 = ratio.next(&window1);
        assert_eq!(result1, 1.0);

        // Second value: 0.0 (all maker buy)
        let window2 = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000.0,
                quantity: 10.0,
                time: 1100,
                is_mm_buyer: true,
            }]),
        };
        let result2 = ratio.next(&window2);
        // EMA(3): k = 2/4 = 0.5, EMA = 0.5 * 0.0 + 0.5 * 1.0 = 0.5
        assert_eq!(result2, 0.5);

        // Third value: 1.0 again
        let window3 = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000.0,
                quantity: 10.0,
                time: 1200,
                is_mm_buyer: false,
            }]),
        };
        let result3 = ratio.next(&window3);
        // EMA = 0.5 * 1.0 + 0.5 * 0.5 = 0.75
        assert_eq!(result3, 0.75);
    }

    #[test]
    fn test_display() {
        let ratio = TakerBuyRatioF64::new(10, 1000.0).unwrap();
        let display_str = format!("{}", ratio);
        assert!(display_str.contains("TakerBuyRatioF64"));
        assert!(display_str.contains("ema_period=10"));
        assert!(display_str.contains("min_volume=1000"));
    }

    #[test]
    fn test_reset() {
        let mut ratio = TakerBuyRatioF64::new(10, 0.0).unwrap();
        
        // Process some data
        let window = MockBatchWindow {
            ticks: Some(vec![MockTradeTick {
                price: 10000.0,
                quantity: 10.0,
                time: 1000,
                is_mm_buyer: false,
            }]),
        };
        ratio.next(&window);
        
        // Reset
        ratio.reset();
        
        // Should return neutral value again for empty window
        let empty_window = MockBatchWindow { ticks: None };
        let result = ratio.next(&empty_window);
        assert_eq!(result, 0.5); // Neutral value after reset
    }
}

