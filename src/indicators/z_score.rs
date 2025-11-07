use std::fmt;

use crate::errors::Result;
use crate::indicators::SimpleMovingAverage as Sma;
use crate::indicators::StandardDeviation as Sd;
use crate::{Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Z-Score (Standard Score).
///
/// Z-Score measures how many standard deviations a value is from the mean.
///
/// # Formula
///
/// z = (x - μ) / σ
///
/// Where:
///
/// * _z_ - z-score value
/// * _x_ - current input value
/// * _μ_ - mean (average) of the period
/// * _σ_ - standard deviation of the period
///
/// # Parameters
///
/// * _period_ - number of periods (integer greater than 0)
///
/// # Example
///
/// ```
/// use ta::indicators::ZScore;
/// use ta::Next;
///
/// let mut z_score = ZScore::new(3).unwrap();
/// // First value: mean = 10, std = 0, z-score = 0 (or NaN if division by zero)
/// assert_eq!(z_score.next(10.0).is_nan() || z_score.next(10.0) == 0.0, true);
/// ```
///
/// # Links
///
/// * [Z-Score, Wikipedia](https://en.wikipedia.org/wiki/Standard_score)
///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ZScore {
    sma: Sma,
    sd: Sd,
}

impl ZScore {
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            sma: Sma::new(period)?,
            sd: Sd::new(period)?,
        })
    }
}

impl Period for ZScore {
    fn period(&self) -> usize {
        self.sma.period()
    }
}

impl Next<f64> for ZScore {
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        let mean = self.sma.next(input);
        let std_dev = self.sd.next(input);

        if std_dev == 0.0 {
            // If standard deviation is zero, all values are the same, z-score is 0
            return 0.0;
        }

        (input - mean) / std_dev
    }
}

impl<T: Close> Next<&T> for ZScore {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for ZScore {
    fn reset(&mut self) {
        self.sma.reset();
        self.sd.reset();
    }
}

impl Default for ZScore {
    fn default() -> Self {
        Self::new(20).unwrap()
    }
}

impl fmt::Display for ZScore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ZScore({})", self.period())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(ZScore);

    #[test]
    fn test_new() {
        assert!(ZScore::new(0).is_err());
        assert!(ZScore::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut z_score = ZScore::new(3).unwrap();
        
        // First value: mean = 10, std = 0, z-score = 0
        assert_eq!(z_score.next(10.0), 0.0);
        
        // Second value: mean = 10.5, std = 0.5, z-score = (11 - 10.5) / 0.5 = 1.0
        assert_eq!(z_score.next(11.0), 1.0);
        
        // Third value: mean = 11, std = 0.816..., z-score = (12 - 11) / 0.816...
        let result = z_score.next(12.0);
        assert!((result - 1.225).abs() < 0.01);
    }

    #[test]
    fn test_next_same_values() {
        let mut z_score = ZScore::new(3).unwrap();
        
        // All values are the same, std = 0, z-score should be 0
        assert_eq!(z_score.next(10.0), 0.0);
        assert_eq!(z_score.next(10.0), 0.0);
        assert_eq!(z_score.next(10.0), 0.0);
        assert_eq!(z_score.next(10.0), 0.0);
    }

    #[test]
    fn test_next_with_bars() {
        fn bar(close: f64) -> Bar {
            Bar::new().close(close)
        }

        let mut z_score = ZScore::new(3).unwrap();
        assert_eq!(z_score.next(&bar(10.0)), 0.0);
        assert_eq!(z_score.next(&bar(11.0)), 1.0);
        let result = z_score.next(&bar(12.0));
        assert!((result - 1.225).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let mut z_score = ZScore::new(3).unwrap();
        assert_eq!(z_score.next(10.0), 0.0);
        assert_eq!(z_score.next(11.0), 1.0);

        z_score.reset();
        assert_eq!(z_score.next(10.0), 0.0);
    }

    #[test]
    fn test_default() {
        ZScore::default();
    }

    #[test]
    fn test_display() {
        let z_score = ZScore::new(5).unwrap();
        assert_eq!(format!("{}", z_score), "ZScore(5)");
    }
}

