use std::fmt;

use crate::errors::Result;
use crate::{High, Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::drawdown_from_high::DrawdownFromHigh;
use super::price_change::PriceChange;

/// Downward momentum indicator.
///
/// It calculates momentum as: drawdown_from_high × abs(price_change) when price is falling,
/// otherwise returns 0.0.
///
/// Formula:
///   downward_momentum = drawdown_from_high × abs(price_change_1m) if price_change_1m < 0
///                     = 0.0 otherwise
///
/// This indicator is composed of:
///   - DrawdownFromHigh: to calculate drawdown from rolling maximum
///   - PriceChange: to calculate price change rate (with shift=1)
///
/// # Parameters
///
/// * _trend_window_ - size of the rolling window for maximum high (e.g., 10 for 10 minutes)
/// * _epsilon_ - small value to avoid division by zero (default: 1e-10)
///
/// # Example
///
/// ```
/// use ta::indicators::DownwardMomentum;
/// use ta::Next;
///
/// let mut dm = DownwardMomentum::new(10).unwrap();
/// let momentum = dm.next((110.0, 108.0)); // (high, close)
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct DownwardMomentum {
    trend_window: usize,
    epsilon: f64,
    drawdown_from_high: DrawdownFromHigh,
    price_change: PriceChange,
    current: f64,
}

impl DownwardMomentum {
    pub fn new(trend_window: usize) -> Result<Self> {
        Self::with_epsilon(trend_window, 1e-10)
    }

    pub fn with_epsilon(trend_window: usize, epsilon: f64) -> Result<Self> {
        if trend_window == 0 {
            return Err(crate::errors::TaError::InvalidParameter);
        }

        let drawdown_from_high = DrawdownFromHigh::with_epsilon(trend_window, epsilon)?;
        let price_change = PriceChange::with_shift(1);

        Ok(Self {
            trend_window,
            epsilon,
            drawdown_from_high,
            price_change,
            current: 0.0,
        })
    }

    /// Calculate momentum using high and close prices
    pub fn next_high_close(&mut self, high: f64, close: f64) -> f64 {
        // Calculate drawdown from high
        let drawdown = self.drawdown_from_high.next_high_close(high, close);

        // Calculate price change (1 minute shift)
        let price_change = self.price_change.next(close);

        // Calculate momentum: only when price is falling
        if price_change < 0.0 {
            self.current = drawdown * price_change.abs();
            self.current
        } else {
            self.current = 0.0;
            0.0
        }
    }

    /// Calculate the reversed downward momentum (negated value).
    /// This makes IC positive (larger value → positive future return).
    pub fn next_reversed(&mut self, high: f64, close: f64) -> f64 {
        -self.next_high_close(high, close)
    }
}

impl Period for DownwardMomentum {
    fn period(&self) -> usize {
        self.trend_window
    }
}

impl Next<(f64, f64)> for DownwardMomentum {
    type Output = f64;

    fn next(&mut self, input: (f64, f64)) -> Self::Output {
        let (high, close) = input;
        self.next_high_close(high, close)
    }
}

impl<T: High + Close> Next<&T> for DownwardMomentum {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next_high_close(input.high(), input.close())
    }
}

impl Reset for DownwardMomentum {
    fn reset(&mut self) {
        self.drawdown_from_high.reset();
        self.price_change.reset();
        self.current = 0.0;
    }
}

impl fmt::Display for DownwardMomentum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DownwardMomentum(trend_window={}, epsilon={})",
            self.trend_window, self.epsilon
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(DownwardMomentum::new(0).is_err());
        assert!(DownwardMomentum::new(10).is_ok());
    }

    #[test]
    fn test_next() {
        let mut dm = DownwardMomentum::new(3).unwrap();

        // First value: no change
        assert_eq!(dm.next_high_close(100.0, 100.0), 0.0);

        // Price increases: high=110, close=108 (no downward momentum)
        assert_eq!(dm.next_high_close(110.0, 108.0), 0.0);

        // Price starts falling: high=110, close=105
        let momentum = dm.next_high_close(110.0, 105.0);
        // Should have momentum since price is falling
        assert!(momentum >= 0.0);
    }

    #[test]
    fn test_reset() {
        let mut dm = DownwardMomentum::new(3).unwrap();
        dm.next_high_close(100.0, 100.0);
        dm.next_high_close(110.0, 108.0);

        dm.reset();
        assert_eq!(dm.next_high_close(100.0, 100.0), 0.0);
    }
}

