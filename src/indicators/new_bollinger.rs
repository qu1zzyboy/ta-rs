use std::fmt;

use crate::errors::Result;
use crate::{Close, Next, Period, Reset, Update};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Bollinger Bands (BB) implementation with Update support.
/// This implementation maintains historical data in a circular buffer and allows updating the last value.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct NewBollinger {
    period: usize,
    multiplier: f64,
    index: usize,
    count: usize,
    sum: f64,
    sum_sq: f64,  // 平方和，用于计算标准差
    deque: Box<[f64]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewBollingerOutput {
    pub average: f64,
    pub upper: f64,
    pub lower: f64,
}

impl From<NewBollingerOutput> for (f64, f64, f64) {
    fn from(bo: NewBollingerOutput) -> Self {
        (bo.average, bo.upper, bo.lower)
    }
}

impl NewBollinger {
    pub fn new(period: usize, multiplier: f64) -> Result<Self> {
        match period {
            0 => Err(crate::errors::TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                multiplier,
                index: 0,
                count: 0,
                sum: 0.0,
                sum_sq: 0.0,
                deque: vec![0.0; period].into_boxed_slice(),
            }),
        }
    }

    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    // 辅助函数：计算标准差
    fn calculate_sd(&self) -> f64 {
        if self.count <= 1 {
            return 0.0;
        }
        let mean = self.sum / (self.count as f64);
        let variance = (self.sum_sq / (self.count as f64)) - (mean * mean);
        variance.sqrt()
    }

    // 辅助函数：计算布林带值
    fn calculate_bands(&self) -> NewBollingerOutput {
        let mean = self.sum / (self.count as f64);
        let sd = self.calculate_sd();
        NewBollingerOutput {
            average: mean,
            upper: mean + sd * self.multiplier,
            lower: mean - sd * self.multiplier,
        }
    }
}

impl Period for NewBollinger {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for NewBollinger {
    type Output = NewBollingerOutput;

    fn next(&mut self, input: f64) -> Self::Output {
        // 获取旧值
        let old_val = self.deque[self.index];
        
        // 更新队列
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

        // 更新总和和平方和
        self.sum = self.sum - old_val + input;
        self.sum_sq = self.sum_sq - (old_val * old_val) + (input * input);

        self.calculate_bands()
    }
}

impl Update<f64> for NewBollinger {
    type Output = NewBollingerOutput;

    fn update(&mut self, input: f64) -> Self::Output {
        if self.count == 0 {
            return self.next(input);
        }

        // 计算前一个索引
        let prev_index = if self.index == 0 {
            self.period - 1
        } else {
            self.index - 1
        };

        // 获取旧值
        let old_val = self.deque[prev_index];
        
        // 更新队列
        self.deque[prev_index] = input;

        // 更新总和和平方和
        self.sum = self.sum - old_val + input;
        self.sum_sq = self.sum_sq - (old_val * old_val) + (input * input);

        self.calculate_bands()
    }
}

impl<T: Close> Next<&T> for NewBollinger {
    type Output = NewBollingerOutput;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for NewBollinger {
    fn reset(&mut self) {
        self.index = 0;
        self.count = 0;
        self.sum = 0.0;
        self.sum_sq = 0.0;
        for i in 0..self.period {
            self.deque[i] = 0.0;
        }
    }
}

impl Default for NewBollinger {
    fn default() -> Self {
        Self::new(20, 2.0).unwrap()
    }
}

impl fmt::Display for NewBollinger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NewBB({}, {})", self.period, self.multiplier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert!(NewBollinger::new(0, 2.0).is_err());
        assert!(NewBollinger::new(1, 2.0).is_ok());
        assert!(NewBollinger::new(2, 2.0).is_ok());
    }

    #[test]
    fn test_next() {
        let mut bb = NewBollinger::new(3, 2.0).unwrap();

        let a = bb.next(2.0);
        let b = bb.next(5.0);
        let c = bb.next(1.0);
        let d = bb.next(6.25);

        assert_eq!(round(a.average), 2.0);
        assert_eq!(round(b.average), 3.5);
        assert_eq!(round(c.average), 2.667);
        assert_eq!(round(d.average), 4.083);

        assert_eq!(round(a.upper), 2.0);
        assert_eq!(round(b.upper), 6.5);
        assert_eq!(round(c.upper), 6.066);
        assert_eq!(round(d.upper), 8.562);

        assert_eq!(round(a.lower), 2.0);
        assert_eq!(round(b.lower), 0.5);
        assert_eq!(round(c.lower), -0.733);
        assert_eq!(round(d.lower), -0.395);
    }

    #[test]
    fn test_update() {
        let mut bb = NewBollinger::new(3, 2.0).unwrap();

        // 先添加一些初始值
        let a = bb.next(2.0);
        let b = bb.next(5.0);
        let c = bb.next(1.0);
        
        assert_eq!(round(a.average), 2.0);
        assert_eq!(round(b.average), 3.5);
        assert_eq!(round(c.average), 2.667);

        // 测试更新最后一个值
        let update1 = bb.update(6.25);
        assert_eq!(round(update1.average), 4.417);
        assert_eq!(round(update1.upper), 7.983);
        assert_eq!(round(update1.lower), 0.85);

        // 再次更新同一个值
        let update2 = bb.update(4.0);
        assert_eq!(round(update2.average), 3.667);
        assert_eq!(round(update2.upper), 6.161);
        assert_eq!(round(update2.lower), 1.172);

        // 继续正常的next操作，确保update没有破坏状态
        let next1 = bb.next(3.0);
        assert_eq!(round(next1.average), 4.0);
        assert_eq!(round(next1.upper), 5.633);
        assert_eq!(round(next1.lower), 2.367);
    }

    #[test]
    fn test_reset() {
        let mut bb = NewBollinger::new(3, 2.0).unwrap();

        let out = bb.next(2.0);
        assert_eq!(round(out.average), 2.0);
        assert_eq!(round(out.upper), 2.0);
        assert_eq!(round(out.lower), 2.0);

        bb.next(5.0);
        bb.next(1.0);
        let out = bb.next(6.25);
        assert_eq!(round(out.average), 4.083);

        bb.reset();
        let out = bb.next(2.0);
        assert_eq!(round(out.average), 2.0);
        assert_eq!(round(out.upper), 2.0);
        assert_eq!(round(out.lower), 2.0);
    }

    #[test]
    fn test_default() {
        NewBollinger::default();
    }

    #[test]
    fn test_display() {
        let bb = NewBollinger::new(10, 3.0).unwrap();
        assert_eq!(format!("{}", bb), "NewBB(10, 3)");
    }

    // 辅助函数：四舍五入到3位小数
    fn round(x: f64) -> f64 {
        (x * 1000.0).round() / 1000.0
    }
}
