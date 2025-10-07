use std::fmt;

use crate::errors::{Result, TaError};
use crate::{Low, Next, Period, Reset, Update};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Returns the lowest value in a given time frame.
///
/// # Parameters
///
/// * _period_ - size of the time frame (integer greater than 0). Default value is 14.
///
/// # Example
///
/// ```
/// use ta::indicators::Minimum;
/// use ta::Next;
///
/// let mut min = Minimum::new(3).unwrap();
/// assert_eq!(min.next(10.0), 10.0);
/// assert_eq!(min.next(11.0), 10.0);
/// assert_eq!(min.next(12.0), 10.0);
/// assert_eq!(min.next(13.0), 11.0);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Minimum {
    period: usize,
    min_index: usize,
    cur_index: usize,
    deque: Box<[f64]>,
}

impl Minimum {
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0 => Err(TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                min_index: 0,
                cur_index: 0,
                deque: vec![f64::INFINITY; period].into_boxed_slice(),
            }),
        }
    }

    fn find_min_index(&self) -> usize {
        let mut min = f64::INFINITY;
        let mut index: usize = 0;

        for (i, &val) in self.deque.iter().enumerate() {
            if val < min {
                min = val;
                index = i;
            }
        }

        index
    }
}

impl Period for Minimum {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for Minimum {
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        self.deque[self.cur_index] = input;

        if input < self.deque[self.min_index] {
            self.min_index = self.cur_index;
        } else if self.min_index == self.cur_index {
            self.min_index = self.find_min_index();
        }

        self.cur_index = if self.cur_index + 1 < self.period {
            self.cur_index + 1
        } else {
            0
        };

        self.deque[self.min_index]
    }
}

impl Update<f64> for Minimum {
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

        // 重新计算最小值索引
        if update_index == self.min_index {
            // 如果更新的是当前最小值的位置，需要重新查找最小值
            self.min_index = self.find_min_index();
        } else if input < self.deque[self.min_index] {
            // 如果更新值比当前最小值还小，更新最小值索引
            self.min_index = update_index;
        }
        // 如果更新的是其他位置且不是新的最小值，则不需要改变min_index

        self.deque[self.min_index]
    }
}

impl<T: Low> Next<&T> for Minimum {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.low())
    }
}

impl Reset for Minimum {
    fn reset(&mut self) {
        for i in 0..self.period {
            self.deque[i] = f64::INFINITY;
        }
    }
}

impl Default for Minimum {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl fmt::Display for Minimum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MIN({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(Minimum);

    #[test]
    fn test_new() {
        assert!(Minimum::new(0).is_err());
        assert!(Minimum::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut min = Minimum::new(3).unwrap();

        assert_eq!(min.next(4.0), 4.0);
        assert_eq!(min.next(1.2), 1.2);
        assert_eq!(min.next(5.0), 1.2);
        assert_eq!(min.next(3.0), 1.2);
        assert_eq!(min.next(4.0), 3.0);
        assert_eq!(min.next(6.0), 3.0);
        assert_eq!(min.next(7.0), 4.0);
        assert_eq!(min.next(8.0), 6.0);
        assert_eq!(min.next(-9.0), -9.0);
        assert_eq!(min.next(0.0), -9.0);
    }

    #[test]
    fn test_next_with_bars() {
        fn bar(low: f64) -> Bar {
            Bar::new().low(low)
        }

        let mut min = Minimum::new(3).unwrap();

        assert_eq!(min.next(&bar(4.0)), 4.0);
        assert_eq!(min.next(&bar(4.0)), 4.0);
        assert_eq!(min.next(&bar(1.2)), 1.2);
        assert_eq!(min.next(&bar(5.0)), 1.2);
    }

    #[test]
    fn test_reset() {
        let mut min = Minimum::new(10).unwrap();

        assert_eq!(min.next(5.0), 5.0);
        assert_eq!(min.next(7.0), 5.0);

        min.reset();
        assert_eq!(min.next(8.0), 8.0);
    }

    #[test]
    fn test_default() {
        Minimum::default();
    }

    #[test]
    fn test_display() {
        let indicator = Minimum::new(10).unwrap();
        assert_eq!(format!("{}", indicator), "MIN(10)");
    }

    #[test]
    fn test_update() {
        let mut min = Minimum::new(3).unwrap();
        
        // 先添加一些值
        assert_eq!(min.next(4.0), 4.0);
        assert_eq!(min.next(1.2), 1.2);
        assert_eq!(min.next(5.0), 1.2);
        
        // 更新最后一个值
        assert_eq!(min.update(0.5), 0.5);
        // 再次更新同一个值
        assert_eq!(min.update(2.0), 1.2);
    }
}
