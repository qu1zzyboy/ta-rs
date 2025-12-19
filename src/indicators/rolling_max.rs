use std::fmt;

use crate::errors::{Result, TaError};
use crate::{High, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Rolling maximum (highest value) over a given window.
///
/// It maintains a sliding window of values and returns the maximum value within that window.
/// Unlike a simple maximum, it supports min_samples=1, meaning it returns the current maximum
/// even when the window is not full.
///
/// # Parameters
///
/// * _period_ - size of the rolling window (integer greater than 0)
/// * _min_samples_ - minimum samples before returning valid result (default: 1)
///
/// # Example
///
/// ```
/// use ta::indicators::RollingMax;
/// use ta::Next;
///
/// let mut rm = RollingMax::new(3).unwrap();
/// assert_eq!(rm.next(7.0), 7.0);
/// assert_eq!(rm.next(5.0), 7.0);
/// assert_eq!(rm.next(4.0), 7.0);
/// assert_eq!(rm.next(4.0), 5.0);
/// assert_eq!(rm.next(8.0), 8.0);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RollingMax {
    period: usize,
    min_samples: usize,
    values: Vec<f64>,
    max_value: f64,
    max_index: usize,
    cur_index: usize,
}

impl RollingMax {
    pub fn new(period: usize) -> Result<Self> {
        Self::with_min_samples(period, 1)
    }

    pub fn with_min_samples(period: usize, min_samples: usize) -> Result<Self> {
        match period {
            0 => Err(TaError::InvalidParameter),
            _ => {
                if min_samples == 0 || min_samples > period {
                    Err(TaError::InvalidParameter)
                } else {
                    Ok(Self {
                        period,
                        min_samples,
                        values: Vec::with_capacity(period),
                        max_value: f64::NEG_INFINITY,
                        max_index: 0,
                        cur_index: 0,
                    })
                }
            }
        }
    }

    fn recalculate_max(&mut self) {
        self.max_value = f64::NEG_INFINITY;
        self.max_index = 0;
        for (i, &val) in self.values.iter().enumerate() {
            if val > self.max_value {
                self.max_value = val;
                self.max_index = i;
            }
        }
    }
}

impl Period for RollingMax {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for RollingMax {
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        // Add new value
        if self.values.len() < self.period {
            self.values.push(input);
        } else {
            // Replace oldest value
            let old_index = self.cur_index;
            self.values[old_index] = input;
            self.cur_index = (self.cur_index + 1) % self.period;

            // If we replaced the old max, need to recalculate
            if self.max_index == old_index {
                self.recalculate_max();
            } else if input > self.max_value {
                // New value is greater than current max
                self.max_value = input;
                self.max_index = old_index;
            }
            return self.max_value;
        }

        // Update maximum for growing window
        if self.values.len() == 1 {
            self.max_value = input;
            self.max_index = 0;
        } else {
            // Check if new value is greater than current max
            if input > self.max_value {
                self.max_value = input;
                self.max_index = self.values.len() - 1;
            }
        }

        // Return max if we have enough samples
        if self.values.len() >= self.min_samples {
            self.max_value
        } else {
            // If not enough samples, still return current max (min_samples=1 behavior)
            self.max_value
        }
    }
}

impl<T: High> Next<&T> for RollingMax {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.high())
    }
}

impl Reset for RollingMax {
    fn reset(&mut self) {
        self.values.clear();
        self.max_value = f64::NEG_INFINITY;
        self.max_index = 0;
        self.cur_index = 0;
    }
}

impl fmt::Display for RollingMax {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RollingMax({}, min_samples={})", self.period, self.min_samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(RollingMax::new(0).is_err());
        assert!(RollingMax::new(1).is_ok());
        assert!(RollingMax::with_min_samples(3, 0).is_err());
        assert!(RollingMax::with_min_samples(3, 4).is_err());
        assert!(RollingMax::with_min_samples(3, 1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut rm = RollingMax::new(3).unwrap();

        assert_eq!(rm.next(4.0), 4.0);
        assert_eq!(rm.next(1.2), 4.0);
        assert_eq!(rm.next(5.0), 5.0);
        assert_eq!(rm.next(3.0), 5.0);
        assert_eq!(rm.next(4.0), 5.0);
        assert_eq!(rm.next(0.0), 4.0);
    }

    #[test]
    fn test_reset() {
        let mut rm = RollingMax::new(3).unwrap();
        assert_eq!(rm.next(4.0), 4.0);
        assert_eq!(rm.next(10.0), 10.0);
        assert_eq!(rm.next(4.0), 10.0);

        rm.reset();
        assert_eq!(rm.next(4.0), 4.0);
    }
}

