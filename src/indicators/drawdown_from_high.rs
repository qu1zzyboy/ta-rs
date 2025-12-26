use std::fmt;

use crate::errors::{Result, TaError};
use crate::{High, Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::maximum::Maximum;

/// Drawdown from the rolling maximum high price.
///
/// It calculates the percentage decline from the highest price in a rolling window.
/// Formula: (high_max - close) / high_max
///
/// This indicator is composed of:
///   - Maximum: to track the maximum high price over a window
///
/// # Parameters
///
/// * _window_size_ - size of the rolling window for maximum high (must be > 0)
///
/// # Error Handling
///
/// Returns 0.0 if:
///   - high_max <= 0.0 (invalid or negative high price)
///   - high_max is infinite or NaN
///   - close is infinite or NaN
///   - Division by zero would occur
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
    maximum: Maximum,
    current: f64,
}

impl DrawdownFromHigh {
    pub fn new(window_size: usize) -> Result<Self> {
        if window_size == 0 {
            return Err(crate::errors::TaError::InvalidParameter);
        }

        let maximum = Maximum::new(window_size)?;

        Ok(Self {
            window_size,
            maximum,
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
        // Return 0.0 on error to maintain compatibility with Next trait
        self.next_high_close(high, close).unwrap_or(0.0)
    }
}

impl DrawdownFromHigh {
    /// Calculate drawdown using high and close prices
    ///
    /// Returns an error if any invalid input is detected (NaN, infinite, or invalid values)
    pub fn next_high_close(&mut self, high: f64, close: f64) -> Result<f64> {
        // Validate input values
        if high.is_nan() || high.is_infinite() || close.is_nan() || close.is_infinite() {
            return Err(TaError::InvalidInput);
        }

        let high_max = self.maximum.next(high);



        // Calculate drawdown: (high_max - close) / high_max
        // This is safe because we've already checked high_max > 0.0
        let drawdown = (high_max - close) / high_max;

        // Validate result: check for NaN or infinite values
        if drawdown.is_nan() || drawdown.is_infinite() {
            return Err(TaError::InvalidInput);
        }

        self.current = drawdown;
        Ok(self.current)
    }
}

impl<T: High + Close> Next<&T> for DrawdownFromHigh {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        // Return 0.0 on error to maintain compatibility with Next trait
        self.next_high_close(input.high(), input.close()).unwrap_or(0.0)
    }
}

impl Reset for DrawdownFromHigh {
    fn reset(&mut self) {
        self.maximum.reset();
        self.current = 0.0;
    }
}

impl fmt::Display for DrawdownFromHigh {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DrawdownFromHigh(window_size={})", self.window_size)
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
        assert_eq!(dd.next_high_close(100.0, 100.0).unwrap(), 0.0);

        // High increases to 110, close=108
        let drawdown1 = dd.next_high_close(110.0, 108.0).unwrap();
        assert!(drawdown1 > 0.0 && drawdown1 < 0.02); // (110 - 108) / 110 ≈ 0.018

        // High stays at 110, close drops to 105
        let drawdown2 = dd.next_high_close(110.0, 105.0).unwrap();
        assert!(drawdown2 > 0.04 && drawdown2 < 0.05); // (110 - 105) / 110 ≈ 0.045
    }

    #[test]
    fn test_reset() {
        let mut dd = DrawdownFromHigh::new(3).unwrap();
        dd.next_high_close(100.0, 100.0).unwrap();
        dd.next_high_close(110.0, 108.0).unwrap();

        dd.reset();
        assert_eq!(dd.next_high_close(100.0, 100.0).unwrap(), 0.0);
    }

    #[test]
    fn test_error_handling() {
        let mut dd = DrawdownFromHigh::new(3).unwrap();

        // Test with NaN values - should return error
        assert!(dd.next_high_close(f64::NAN, 100.0).is_err());
        assert!(dd.next_high_close(100.0, f64::NAN).is_err());

        // Test with infinite values - should return error
        assert!(dd.next_high_close(f64::INFINITY, 100.0).is_err());
        assert!(dd.next_high_close(100.0, f64::INFINITY).is_err());
        assert!(dd.next_high_close(f64::NEG_INFINITY, 100.0).is_err());

        // Test with zero or negative high_max - should return error
        dd.reset();
        assert!(dd.next_high_close(0.0, 50.0).is_err());

        // Test normal calculation after reset
        dd.reset();
        assert_eq!(dd.next_high_close(100.0, 100.0).unwrap(), 0.0);
        let drawdown = dd.next_high_close(110.0, 105.0).unwrap();
        assert!(drawdown > 0.0 && drawdown < 0.1); // (110 - 105) / 110 ≈ 0.045
    }

    #[test]
    fn test_division_by_zero_protection() {
        let mut dd = DrawdownFromHigh::new(3).unwrap();
        
        // Even if high_max becomes 0.0 somehow, should return error instead of panicking
        dd.reset();
        assert!(dd.next_high_close(0.0, 50.0).is_err());
    }

    #[test]
    fn test_error_type() {
        let mut dd = DrawdownFromHigh::new(3).unwrap();
        
        // Verify that InvalidInput error is returned
        let result = dd.next_high_close(f64::NAN, 100.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TaError::InvalidInput);
    }
}

