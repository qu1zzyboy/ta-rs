use std::fmt;

use crate::errors::{Result, TaError};
use crate::{Close, Next, Period, Reset, Update};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An exponential moving average (EMA) implementation using a circular buffer.
/// This implementation maintains historical data and provides both Next and Update traits.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct NewEma {
    /// Number of periods
    period: usize,
    /// Current position in the circular buffer
    index: usize,
    /// Number of values seen so far
    count: usize,
    /// The smoothing factor: 2/(period + 1)
    k: f64,
    /// Current EMA value
    current: f64,
    /// Circular buffer to store historical data
    deque: Box<[f64]>,
}

impl NewEma {
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0 => Err(TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                index: 0,
                count: 0,
                k: 2.0 / (period + 1) as f64,
                current: 0.0,
                deque: vec![0.0; period].into_boxed_slice(),
            }),
        }
    }
}

impl Period for NewEma {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for NewEma {
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        // 存储新值
        self.deque[self.index] = input;

        // 更新索引
        self.index = if self.index + 1 < self.period {
            self.index + 1
        } else {
            0
        };

        // 更新计数
        if self.count < self.period {
            self.count += 1;
        }

        // 计算 EMA
        if self.count == 1 {
            // 第一个值直接作为 EMA
            self.current = input;
        } else {
            // EMA = α * price + (1 - α) * EMA_prev
            self.current = self.k * input + (1.0 - self.k) * self.current;
        }

        self.current
    }
}

impl Update<f64> for NewEma {
    type Output = f64;

    fn update(&mut self, input: f64) -> Self::Output {
        if self.count == 0 {
            self.next(input)
        } else {
            // 计算前一个索引
            let prev_index = if self.index == 0 {
                self.period - 1
            } else {
                self.index - 1
            };

            // 更新前一个值
            self.deque[prev_index] = input;

            // 重新计算 EMA
            self.current = self.k * input + (1.0 - self.k) * self.current;
            self.current
        }
    }
}

impl<T: Close> Next<&T> for NewEma {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for NewEma {
    fn reset(&mut self) {
        self.index = 0;
        self.count = 0;
        self.current = 0.0;
        for i in 0..self.period {
            self.deque[i] = 0.0;
        }
    }
}

impl Default for NewEma {
    fn default() -> Self {
        Self::new(9).unwrap()
    }
}

impl fmt::Display for NewEma {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NewEMA({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(NewEma::new(0).is_err());
        assert!(NewEma::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut ema = NewEma::new(3).unwrap();

        // 这些值应该和原始 EMA 实现产生相同的结果
        assert_eq!(ema.next(2.0), 2.0);
        assert_eq!(ema.next(5.0), 3.5);
        assert_eq!(ema.next(1.0), 2.25);
        assert_eq!(ema.next(6.25), 4.25);
    }

    #[test]
    fn test_update() {
        let mut ema = NewEma::new(3).unwrap();
        
        assert_eq!(ema.next(2.0), 2.0);
        assert_eq!(ema.next(5.0), 3.5);
        assert_eq!(ema.next(1.0), 2.25);
        
        // 更新最后一个值
        assert_eq!(ema.update(6.25), 4.25);
        // 再次更新同一个值
        assert_eq!(ema.update(3.0), 3.625);
    }

    #[test]
    fn test_reset() {
        let mut ema = NewEma::new(3).unwrap();
        
        assert_eq!(ema.next(2.0), 2.0);
        assert_eq!(ema.next(5.0), 3.5);
        
        ema.reset();
        
        // 重置后应该和新创建的实例行为一样
        assert_eq!(ema.next(2.0), 2.0);
        assert_eq!(ema.next(5.0), 3.5);
    }

    #[test]
    fn test_default() {
        NewEma::default();
    }

    #[test]
    fn test_display() {
        let ema = NewEma::new(7).unwrap();
        assert_eq!(format!("{}", ema), "NewEMA(7)");
    }
}
