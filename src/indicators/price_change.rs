use std::fmt;

use crate::errors::Result;
use crate::{Close, Next, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Price change rate over a given period.
///
/// It calculates the percentage change from the previous value (shift=1) or from N periods ago.
/// Formula: (current - previous) / (previous + epsilon)
///
/// # Parameters
///
/// * _shift_ - number of periods to look back (default: 1)
/// * _epsilon_ - small value to avoid division by zero (default: 1e-10)
///
/// # Example
///
/// ```
/// use ta::indicators::PriceChange;
/// use ta::Next;
///
/// let mut pc = PriceChange::new();
/// assert_eq!(pc.next(100.0), 0.0);  // First value, no change
/// assert_eq!(pc.next(102.0), 0.02); // (102 - 100) / 100 = 0.02
/// assert_eq!(pc.next(98.0), -0.0392); // (98 - 102) / 102 ≈ -0.0392
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct PriceChange {
    shift: usize,
    epsilon: f64,
    history: Vec<f64>,
    prev_value: f64,
    is_new: bool,
}

impl PriceChange {
    pub fn new() -> Self {
        Self::with_shift(1)
    }

    pub fn with_shift(shift: usize) -> Self {
        Self {
            shift: if shift == 0 { 1 } else { shift },
            epsilon: 1e-10,
            history: Vec::with_capacity(shift + 1),
            prev_value: 0.0,
            is_new: true,
        }
    }

    pub fn with_epsilon(shift: usize, epsilon: f64) -> Self {
        Self {
            shift: if shift == 0 { 1 } else { shift },
            epsilon,
            history: Vec::with_capacity(shift + 1),
            prev_value: 0.0,
            is_new: true,
        }
    }
}

impl Default for PriceChange {
    fn default() -> Self {
        Self::new()
    }
}

impl Next<f64> for PriceChange {
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        if self.is_new {
            self.is_new = false;
            self.prev_value = input;
            self.history.push(input);
            return 0.0; // No change on first value
        }

        // Add to history
        self.history.push(input);

        // Keep only the last (shift+1) values
        if self.history.len() > self.shift + 1 {
            self.history = self.history[self.history.len() - (self.shift + 1)..].to_vec();
        }

        // Calculate change if we have enough history
        if self.history.len() > self.shift {
            let prev_value = self.history[self.history.len() - self.shift - 1];
            let change = (input - prev_value) / (prev_value + self.epsilon);
            self.prev_value = input;
            change
        } else {
            self.prev_value = input;
            0.0
        }
    }
}

impl<T: Close> Next<&T> for PriceChange {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for PriceChange {
    fn reset(&mut self) {
        self.history.clear();
        self.prev_value = 0.0;
        self.is_new = true;
    }
}

impl fmt::Display for PriceChange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PriceChange(shift={}, epsilon={})", self.shift, self.epsilon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let pc = PriceChange::new();
        assert_eq!(pc.shift, 1);
        assert_eq!(pc.epsilon, 1e-10);
    }

    #[test]
    fn test_next() {
        let mut pc = PriceChange::new();

        assert_eq!(pc.next(100.0), 0.0); // First value
        let change1 = pc.next(102.0);
        assert!((change1 - 0.02).abs() < 1e-10); // (102 - 100) / 100 = 0.02

        let change2 = pc.next(98.0);
        let expected = (98.0 - 102.0) / (102.0 + 1e-10);
        assert!((change2 - expected).abs() < 1e-10);
    }

    #[test]
    fn test_reset() {
        let mut pc = PriceChange::new();
        pc.next(100.0);
        pc.next(102.0);

        pc.reset();
        assert_eq!(pc.next(100.0), 0.0);
    }
}

