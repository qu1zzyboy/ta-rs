use std::fmt;

use crate::errors::{Result, TaError};
use crate::{High, Next, Period, Reset, Update};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Returns the highest value in a given time frame.
///
/// # Parameters
///
/// * _period_ - size of the time frame (integer greater than 0). Default value is 14.
///
/// # Example
///
/// ```
/// use ta::indicators::Maximum;
/// use ta::Next;
///
/// let mut max = Maximum::new(3).unwrap();
/// assert_eq!(max.next(7.0), 7.0);
/// assert_eq!(max.next(5.0), 7.0);
/// assert_eq!(max.next(4.0), 7.0);
/// assert_eq!(max.next(4.0), 5.0);
/// assert_eq!(max.next(8.0), 8.0);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Maximum {
    period: usize,
    max_index: usize,
    cur_index: usize,
    deque: Box<[f64]>,
}

impl Maximum {
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0 => Err(TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                max_index: 0,
                cur_index: 0,
                deque: vec![f64::NEG_INFINITY; period].into_boxed_slice(),
            }),
        }
    }

    fn find_max_index(&self) -> usize {
        let mut max = f64::NEG_INFINITY;
        let mut index: usize = 0;

        for (i, &val) in self.deque.iter().enumerate() {
            if val > max {
                max = val;
                index = i;
            }
        }

        index
    }
}

impl Period for Maximum {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for Maximum {
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        self.deque[self.cur_index] = input;

        if input > self.deque[self.max_index] {
            self.max_index = self.cur_index;
        } else if self.max_index == self.cur_index {
            self.max_index = self.find_max_index();
        }

        self.cur_index = if self.cur_index + 1 < self.period {
            self.cur_index + 1
        } else {
            0
        };

        self.deque[self.max_index]
    }
}

impl Update<f64> for Maximum {
    type Output = f64;

    fn update(&mut self, input: f64) -> Self::Output {
        // 计算当前要更新的索引（cur_index指向下一个要写入的位置）
        let update_index = if self.cur_index == 0 {
            self.period - 1
        } else {
            self.cur_index - 1
        };

        // 更新指定位置的值
        self.deque[update_index] = input;

        // 重新计算最大值索引
        if update_index == self.max_index {
            // 如果更新的是当前最大值的位置，需要重新查找最大值
            self.max_index = self.find_max_index();
        } else if input > self.deque[self.max_index] {
            // 如果更新值比当前最大值还大，更新最大值索引
            self.max_index = update_index;
        }
        // 如果更新的是其他位置且不是新的最大值，则不需要改变max_index

        self.deque[self.max_index]
    }
}

impl<T: High> Next<&T> for Maximum {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.high())
    }
}

impl Reset for Maximum {
    fn reset(&mut self) {
        for i in 0..self.period {
            self.deque[i] = f64::NEG_INFINITY;
        }
    }
}

impl Default for Maximum {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl fmt::Display for Maximum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MAX({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(Maximum);

    #[test]
    fn test_new() {
        assert!(Maximum::new(0).is_err());
        assert!(Maximum::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut max = Maximum::new(3).unwrap();

        assert_eq!(max.next(4.0), 4.0);
        assert_eq!(max.next(1.2), 4.0);
        assert_eq!(max.next(5.0), 5.0);
        assert_eq!(max.next(3.0), 5.0);
        assert_eq!(max.next(4.0), 5.0);
        assert_eq!(max.next(0.0), 4.0);
        assert_eq!(max.next(-1.0), 4.0);
        assert_eq!(max.next(-2.0), 0.0);
        assert_eq!(max.next(-1.5), -1.0);
    }

    #[test]
    fn test_next_with_bars() {
        fn bar(high: f64) -> Bar {
            Bar::new().high(high)
        }

        let mut max = Maximum::new(2).unwrap();

        assert_eq!(max.next(&bar(1.1)), 1.1);
        assert_eq!(max.next(&bar(4.0)), 4.0);
        assert_eq!(max.next(&bar(3.5)), 4.0);
        assert_eq!(max.next(&bar(2.0)), 3.5);
    }

    #[test]
    fn test_reset() {
        let mut max = Maximum::new(100).unwrap();
        assert_eq!(max.next(4.0), 4.0);
        assert_eq!(max.next(10.0), 10.0);
        assert_eq!(max.next(4.0), 10.0);

        max.reset();
        assert_eq!(max.next(4.0), 4.0);
    }

    #[test]
    fn test_default() {
        Maximum::default();
    }

    #[test]
    fn test_display() {
        let indicator = Maximum::new(7).unwrap();
        assert_eq!(format!("{}", indicator), "MAX(7)");
    }

    #[test]
    fn test_update() {
        let mut max = Maximum::new(3).unwrap();
        
        // 先添加一些值
        assert_eq!(max.next(4.0), 4.0);
        assert_eq!(max.next(1.2), 4.0);
        assert_eq!(max.next(5.0), 5.0);
        
        // 更新最后一个值
        assert_eq!(max.update(6.25), 6.25);
        
        // 再次更新同一个值
        assert_eq!(max.update(3.0), 4.0);
    }
}
