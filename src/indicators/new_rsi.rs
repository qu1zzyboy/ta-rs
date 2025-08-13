use std::fmt;

use crate::errors::Result;
use crate::indicators::NewEma;
use crate::{Close, Next, Period, Reset, Update};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Relative Strength Index (RSI) with Update support.
/// This implementation maintains historical data and allows updating the last value.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct NewRsi {
    period: usize,
    up_ema: NewEma,
    down_ema: NewEma,
    prev_val: f64,
    is_new: bool,
    last_up: f64,   // 存储最后一个上涨值
    last_down: f64, // 存储最后一个下跌值
}

impl NewRsi {
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            period,
            up_ema: NewEma::new(period)?,
            down_ema: NewEma::new(period)?,
            prev_val: 0.0,
            is_new: true,
            last_up: 0.0,
            last_down: 0.0,
        })
    }

    // 辅助函数：计算RSI值
    fn calculate_rsi(&self, up_ema: f64, down_ema: f64) -> f64 {
        100.0 * up_ema / (up_ema + down_ema)
    }
}

impl Period for NewRsi {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for NewRsi {
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        let mut up = 0.0;
        let mut down = 0.0;

        if self.is_new {
            self.is_new = false;
            // 初始化时使用小数避免除以零
            up = 0.1;
            down = 0.1;
        } else {
            if input > self.prev_val {
                up = input - self.prev_val;
            } else {
                down = self.prev_val - input;
            }
        }

        self.prev_val = input;
        self.last_up = up;
        self.last_down = down;

        let up_ema = self.up_ema.next(up);
        let down_ema = self.down_ema.next(down);

        self.calculate_rsi(up_ema, down_ema)
    }
}

impl Update<f64> for NewRsi {
    type Output = f64;

    fn update(&mut self, input: f64) -> Self::Output {
        if self.is_new {
            return self.next(input);
        }

        let mut up = 0.0;
        let mut down = 0.0;

        if input > self.prev_val {
            up = input - self.prev_val;
        } else {
            down = self.prev_val - input;
        }

        self.prev_val = input;
        self.last_up = up;
        self.last_down = down;

        let up_ema = self.up_ema.update(up);
        let down_ema = self.down_ema.update(down);

        self.calculate_rsi(up_ema, down_ema)
    }
}

impl<T: Close> Next<&T> for NewRsi {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for NewRsi {
    fn reset(&mut self) {
        self.is_new = true;
        self.prev_val = 0.0;
        self.last_up = 0.0;
        self.last_down = 0.0;
        self.up_ema.reset();
        self.down_ema.reset();
    }
}

impl Default for NewRsi {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl fmt::Display for NewRsi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NewRSI({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(NewRsi::new(0).is_err());
        assert!(NewRsi::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut rsi = NewRsi::new(3).unwrap();
        
        // 这些值应该和原始RSI实现产生相同的结果
        assert_eq!(rsi.next(10.0).round(), 50.0);
        assert_eq!(rsi.next(10.5).round(), 86.0);
        assert_eq!(rsi.next(10.0).round(), 35.0);
        assert_eq!(rsi.next(9.5).round(), 16.0);
    }

    #[test]
    fn test_update() {
        let mut rsi = NewRsi::new(3).unwrap();
        
        // 先添加一些初始值
        assert_eq!(rsi.next(10.0).round(), 50.0);
        assert_eq!(rsi.next(10.5).round(), 86.0);
        assert_eq!(rsi.next(10.0).round(), 35.0);
        
        // 测试更新最后一个值
        assert_eq!(rsi.update(10.5).round(), 70.0);
        
        // 再次更新同一个值
        assert_eq!(rsi.update(10.8).round(), 82.0);
        
        // 继续正常的next操作，确保update没有破坏状态
        assert_eq!(rsi.next(10.3).round(), 35.0);
    }

    #[test]
    fn test_reset() {
        let mut rsi = NewRsi::new(3).unwrap();
        
        assert_eq!(rsi.next(10.0).round(), 50.0);
        assert_eq!(rsi.next(10.5).round(), 86.0);
        
        rsi.reset();
        
        // 重置后应该和新创建的实例行为一样
        assert_eq!(rsi.next(10.0).round(), 50.0);
        assert_eq!(rsi.next(10.5).round(), 86.0);
    }

    #[test]
    fn test_default() {
        NewRsi::default();
    }

    #[test]
    fn test_display() {
        let rsi = NewRsi::new(14).unwrap();
        assert_eq!(format!("{}", rsi), "NewRSI(14)");
    }
}
