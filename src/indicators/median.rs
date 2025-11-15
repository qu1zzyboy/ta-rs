use std::fmt;
use std::sync::Arc;

use crate::errors::{Result, TaError};
use crate::{Close, Next, Period, Reset, Update};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Median (滑动窗口中位数).
///
/// 中位数是一组数据按大小顺序排列后，位于中间位置的数值。
/// 如果数据个数为奇数，中位数是中间的值；如果为偶数，中位数是中间两个值的平均值。
///
/// # Formula
///
/// 对于窗口内的数据 [x₁, x₂, ..., xₙ]，排序后：
/// - 如果 n 为奇数：median = x₍ₙ₊₁₎/₂
/// - 如果 n 为偶数：median = (xₙ/₂ + x₍ₙ/₂₊₁₎) / 2
///
/// # Parameters
///
/// * _period_ - 滑动窗口的大小（大于0的整数）
///
/// # Example
///
/// ```
/// use ta::indicators::Median;
/// use ta::Next;
///
/// let mut median = Median::new(5).unwrap();
/// assert_eq!(median.next(1.0), 1.0);
/// assert_eq!(median.next(3.0), 2.0);
/// assert_eq!(median.next(5.0), 3.0);
/// assert_eq!(median.next(7.0), 4.0);
/// assert_eq!(median.next(9.0), 5.0);
/// assert_eq!(median.next(2.0), 5.0);
/// ```
///
/// # Links
///
/// * [Median, Wikipedia](https://en.wikipedia.org/wiki/Median)
///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Median {
    period: usize,
    index: usize,
    count: usize,
    deque: Box<[f64]>,
}

impl Median {
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0 => Err(TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                index: 0,
                count: 0,
                deque: vec![0.0; period].into_boxed_slice(),
            }),
        }
    }

    fn calculate_median(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        // 创建一个临时向量来存储当前窗口内的值
        let mut values: Vec<f64> = self.deque[..self.count].to_vec();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mid = self.count / 2;
        if self.count % 2 == 1 {
            // 奇数个元素，返回中间值
            values[mid]
        } else {
            // 偶数个元素，返回中间两个值的平均值
            (values[mid - 1] + values[mid]) / 2.0
        }
    }
}

impl Period for Median {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for Median {
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        self.deque[self.index] = input;

        self.index = if self.index + 1 < self.period {
            self.index + 1
        } else {
            0
        };

        if self.count < self.period {
            self.count += 1;
        }

        self.calculate_median()
    }
}

impl Update<f64> for Median {
    type Output = f64;

    fn update(&mut self, input: f64) -> Self::Output {
        if self.count == 0 {
            self.next(input)
        } else {
            // 计算前一个索引，处理循环数组的边界情况
            let prev_index = if self.index == 0 {
                self.period - 1
            } else {
                self.index - 1
            };

            self.deque[prev_index] = input;
            self.calculate_median()
        }
    }
}

impl<T: Close> Next<&T> for Median {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl<T: Close> Next<Arc<T>> for Median {
    type Output = f64;

    fn next(&mut self, input: Arc<T>) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for Median {
    fn reset(&mut self) {
        self.index = 0;
        self.count = 0;
        for i in 0..self.period {
            self.deque[i] = 0.0;
        }
    }
}

impl Default for Median {
    fn default() -> Self {
        Self::new(9).unwrap()
    }
}

impl fmt::Display for Median {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MEDIAN({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(Median);

    #[test]
    fn test_new() {
        assert!(Median::new(0).is_err());
        assert!(Median::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut median = Median::new(5).unwrap();
        
        // 窗口大小小于 period 时
        assert_eq!(median.next(1.0), 1.0);  // [1] -> 1
        assert_eq!(median.next(3.0), 2.0);  // [1, 3] -> (1+3)/2 = 2
        assert_eq!(median.next(5.0), 3.0);  // [1, 3, 5] -> 3
        assert_eq!(median.next(7.0), 4.0);  // [1, 3, 5, 7] -> (3+5)/2 = 4
        assert_eq!(median.next(9.0), 5.0);  // [1, 3, 5, 7, 9] -> 5
        
        // 窗口已满，开始滑动
        assert_eq!(median.next(2.0), 5.0);  // [3, 5, 7, 9, 2] -> 排序后 [2, 3, 5, 7, 9] -> 5
        assert_eq!(median.next(4.0), 5.0);  // [5, 7, 9, 2, 4] -> 排序后 [2, 4, 5, 7, 9] -> 5
        assert_eq!(median.next(6.0), 6.0);  // [7, 9, 2, 4, 6] -> 排序后 [2, 4, 6, 7, 9] -> 6
    }

    #[test]
    fn test_next_even_period() {
        let mut median = Median::new(4).unwrap();
        
        assert_eq!(median.next(1.0), 1.0);  // [1] -> 1
        assert_eq!(median.next(3.0), 2.0);  // [1, 3] -> (1+3)/2 = 2
        assert_eq!(median.next(5.0), 3.0);  // [1, 3, 5] -> 3
        assert_eq!(median.next(7.0), 4.0);  // [1, 3, 5, 7] -> (3+5)/2 = 4
        assert_eq!(median.next(9.0), 6.0);  // [3, 5, 7, 9] -> (5+7)/2 = 6
    }

    #[test]
    fn test_update() {
        let mut median = Median::new(5).unwrap();
        
        assert_eq!(median.next(1.0), 1.0);
        assert_eq!(median.next(3.0), 2.0);
        assert_eq!(median.next(5.0), 3.0);
        assert_eq!(median.next(7.0), 4.0);
        assert_eq!(median.next(9.0), 5.0);
        
        // 更新最后一个值
        assert_eq!(median.update(2.0), 3.0);  // [1, 3, 5, 7, 2] -> 排序后 [1, 2, 3, 5, 7] -> 3
    }

    #[test]
    fn test_next_with_bars() {
        fn bar(close: f64) -> Bar {
            Bar::new().close(close)
        }

        let mut median = Median::new(3).unwrap();
        assert_eq!(median.next(&bar(4.0)), 4.0);
        assert_eq!(median.next(&bar(2.0)), 3.0);
        assert_eq!(median.next(&bar(6.0)), 4.0);
        assert_eq!(median.next(&bar(1.0)), 2.0);
    }

    #[test]
    fn test_next_with_arc() {
        use std::sync::Arc;
        
        fn bar(close: f64) -> Bar {
            Bar::new().close(close)
        }

        let mut median = Median::new(3).unwrap();
        
        let arc_bar1 = Arc::new(bar(4.0));
        let arc_bar2 = Arc::new(bar(2.0));
        let arc_bar3 = Arc::new(bar(6.0));
        
        assert_eq!(median.next(arc_bar1), 4.0);
        assert_eq!(median.next(arc_bar2), 3.0);
        assert_eq!(median.next(arc_bar3), 4.0);
    }

    #[test]
    fn test_reset() {
        let mut median = Median::new(5).unwrap();
        assert_eq!(median.next(1.0), 1.0);
        assert_eq!(median.next(3.0), 2.0);
        assert_eq!(median.next(5.0), 3.0);

        median.reset();
        assert_eq!(median.next(99.0), 99.0);
    }

    #[test]
    fn test_default() {
        Median::default();
    }

    #[test]
    fn test_display() {
        let median = Median::new(5).unwrap();
        assert_eq!(format!("{}", median), "MEDIAN(5)");
    }
}

