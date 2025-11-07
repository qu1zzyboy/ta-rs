use std::fmt;

use crate::errors::Result;
use crate::indicators::NewEma;
use crate::{Close, Next, Period, Reset, Update};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Moving average convergence divergence (MACD) with Update support.
/// This implementation maintains historical data and allows updating the last value.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct NewMacd {
    fast_ema: NewEma,
    slow_ema: NewEma,
    signal_ema: NewEma,
    last_macd: f64,  // 存储最后一个MACD值，用于Update
}

impl NewMacd {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Result<Self> {
        Ok(Self {
            fast_ema: NewEma::new(fast_period)?,
            slow_ema: NewEma::new(slow_period)?,
            signal_ema: NewEma::new(signal_period)?,
            last_macd: 0.0,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewMacdOutput {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

impl From<NewMacdOutput> for (f64, f64, f64) {
    fn from(mo: NewMacdOutput) -> Self {
        (mo.macd, mo.signal, mo.histogram)
    }
}

impl Next<f64> for NewMacd {
    type Output = NewMacdOutput;

    fn next(&mut self, input: f64) -> Self::Output {
        let fast_val = self.fast_ema.next(input);
        let slow_val = self.slow_ema.next(input);

        let macd = fast_val - slow_val;
        self.last_macd = macd;  // 保存最后的MACD值
        let signal = self.signal_ema.next(macd);
        let histogram = macd - signal;

        NewMacdOutput {
            macd,
            signal,
            histogram,
        }
    }
}

impl Update<f64> for NewMacd {
    type Output = NewMacdOutput;

    fn update(&mut self, input: f64) -> Self::Output {
        let fast_val = self.fast_ema.update(input);
        let slow_val = self.slow_ema.update(input);

        let macd = fast_val - slow_val;
        let signal = self.signal_ema.update(macd);
        let histogram = macd - signal;

        self.last_macd = macd;

        NewMacdOutput {
            macd,
            signal,
            histogram,
        }
    }
}

impl<T: Close> Next<&T> for NewMacd {
    type Output = NewMacdOutput;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for NewMacd {
    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.last_macd = 0.0;
    }
}

impl Default for NewMacd {
    fn default() -> Self {
        Self::new(12, 26, 9).unwrap()
    }
}

impl fmt::Display for NewMacd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "NewMACD({}, {}, {})",
            self.fast_ema.period(),
            self.slow_ema.period(),
            self.signal_ema.period()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round(nums: (f64, f64, f64)) -> (f64, f64, f64) {
        let n0 = (nums.0 * 100.0).round() / 100.0;
        let n1 = (nums.1 * 100.0).round() / 100.0;
        let n2 = (nums.2 * 100.0).round() / 100.0;
        (n0, n1, n2)
    }

    #[test]
    fn test_new() {
        assert!(NewMacd::new(0, 1, 1).is_err());
        assert!(NewMacd::new(1, 0, 1).is_err());
        assert!(NewMacd::new(1, 1, 0).is_err());
        assert!(NewMacd::new(1, 1, 1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut macd = NewMacd::new(3, 6, 4).unwrap();

        // 这些值应该和原始MACD实现产生相同的结果
        assert_eq!(round(macd.next(2.0).into()), (0.0, 0.0, 0.0));
        assert_eq!(round(macd.next(3.0).into()), (0.21, 0.09, 0.13));
        assert_eq!(round(macd.next(4.2).into()), (0.52, 0.26, 0.26));
        assert_eq!(round(macd.next(7.0).into()), (1.15, 0.62, 0.54));
        assert_eq!(round(macd.next(6.7).into()), (1.15, 0.83, 0.32));
        assert_eq!(round(macd.next(6.5).into()), (0.94, 0.87, 0.07));
    }

    #[test]
    fn test_update() {
        let mut macd = NewMacd::new(3, 6, 4).unwrap();

        // 先添加一些初始值
        assert_eq!(round(macd.next(2.0).into()), (0.0, 0.0, 0.0));
        assert_eq!(round(macd.next(3.0).into()), (0.21, 0.09, 0.13));
        assert_eq!(round(macd.next(4.2).into()), (0.52, 0.26, 0.26));

        // 测试更新最后一个值
        let result = macd.update(7.0);
        assert_eq!(round(result.into()), (1.15, 0.62, 0.54));

        // 再次更新同一个值
        let result = macd.update(6.7);
        assert_eq!(round(result.into()), (1.15, 0.83, 0.32));
    }

    #[test]
    fn test_reset() {
        let mut macd = NewMacd::new(3, 6, 4).unwrap();

        assert_eq!(round(macd.next(2.0).into()), (0.0, 0.0, 0.0));
        assert_eq!(round(macd.next(3.0).into()), (0.21, 0.09, 0.13));

        macd.reset();

        assert_eq!(round(macd.next(2.0).into()), (0.0, 0.0, 0.0));
        assert_eq!(round(macd.next(3.0).into()), (0.21, 0.09, 0.13));
    }

    #[test]
    fn test_default() {
        NewMacd::default();
    }

    #[test]
    fn test_display() {
        let indicator = NewMacd::new(13, 30, 10).unwrap();
        assert_eq!(format!("{}", indicator), "NewMACD(13, 30, 10)");
    }
}
