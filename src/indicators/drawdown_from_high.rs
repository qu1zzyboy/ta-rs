use std::fmt;

use crate::errors::Result;
use crate::{High, Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::rolling_max::RollingMax;

/// Drawdown from the rolling maximum high price.
///
/// It calculates the percentage decline from the highest price in a rolling window.
/// Formula: (high_max - close) / (high_max + epsilon)
///
/// This indicator is composed of:
///   - RollingMax: to track the maximum high price over a window
///
/// # Parameters
///
/// * _window_size_ - size of the rolling window for maximum high (must be > 0)
/// * _epsilon_ - small value to avoid division by zero (default: 1e-10)
///
/// # Example
///
/// ```
/// use ta::indicators::DrawdownFromHigh;
/// use ta::Next;
///
/// let mut dd = DrawdownFromHigh::new(10).unwrap();
/// let drawdown = dd.next(110.0, 108.0); // high=110, close=108
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct DrawdownFromHigh {
    window_size: usize,
    epsilon: f64,
    rolling_max: RollingMax,
    current: f64,
}

impl DrawdownFromHigh {
    pub fn new(window_size: usize) -> Result<Self> {
        Self::with_epsilon(window_size, 1e-10)
    }

    pub fn with_epsilon(window_size: usize, epsilon: f64) -> Result<Self> {
        if window_size == 0 {
            return Err(crate::errors::TaError::InvalidParameter);
        }

        let rolling_max = RollingMax::new(window_size)?;

        Ok(Self {
            window_size,
            epsilon,
            rolling_max,
            current: 0.0,
        })
    }
}

impl Period for DrawdownFromHigh {
    fn period(&self) -> usize {
        self.window_size
    }
}

impl Next<(f64, f64)> for DrawdownFromHigh {
    type Output = f64;

    fn next(&mut self, input: (f64, f64)) -> Self::Output {
        let (high, close) = input;
        self.next_high_close(high, close)
    }
}

impl DrawdownFromHigh {
    /// Calculate drawdown using high and close prices
    pub fn next_high_close(&mut self, high: f64, close: f64) -> f64 {
        let high_max = self.rolling_max.next(high);
        if high_max <= 0.0 {
            self.current = 0.0;
            return 0.0;
        }
        self.current = (high_max - close) / (high_max + self.epsilon);
        self.current
    }
}

impl<T: High + Close> Next<&T> for DrawdownFromHigh {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next_high_close(input.high(), input.close())
    }
}

impl Reset for DrawdownFromHigh {
    fn reset(&mut self) {
        self.rolling_max.reset();
        self.current = 0.0;
    }
}

impl fmt::Display for DrawdownFromHigh {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DrawdownFromHigh(window_size={}, epsilon={})", self.window_size, self.epsilon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(DrawdownFromHigh::new(0).is_err());
        assert!(DrawdownFromHigh::new(10).is_ok());
    }

    #[test]
    fn test_next() {
        let mut dd = DrawdownFromHigh::new(3).unwrap();

        // First value: high=100, close=100, no drawdown yet
        assert_eq!(dd.next_high_close(100.0, 100.0), 0.0);

        // High increases to 110, close=108
        let drawdown1 = dd.next_high_close(110.0, 108.0);
        assert!(drawdown1 > 0.0 && drawdown1 < 0.02); // (110 - 108) / 110 ≈ 0.018

        // High stays at 110, close drops to 105
        let drawdown2 = dd.next_high_close(110.0, 105.0);
        assert!(drawdown2 > 0.04 && drawdown2 < 0.05); // (110 - 105) / 110 ≈ 0.045
    }

    #[test]
    fn test_reset() {
        let mut dd = DrawdownFromHigh::new(3).unwrap();
        dd.next_high_close(100.0, 100.0);
        dd.next_high_close(110.0, 108.0);

        dd.reset();
        assert_eq!(dd.next_high_close(100.0, 100.0), 0.0);
    }
}

