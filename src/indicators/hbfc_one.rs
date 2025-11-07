use std::sync::Arc;

use crate::{Close, Next, Open, Tbbav};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct HbfcOne {
    open: Option<f64>,
    close: Option<f64>,
    tbqav: Option<f64>,
}

impl HbfcOne {
    pub fn new() -> Self {
        Self {
            open: None,
            close: None,
            tbqav: None,
        }
    }
}

impl Default for HbfcOne {
    fn default() -> Self {
        Self::new()
    }
}
impl Next<[f64; 3]> for HbfcOne {
    type Output = Option<f64>;

    fn next(&mut self, input: [f64; 3]) -> Self::Output {
        if input[1] < input[0] {
            None
        } else {
            let result = input[2] / (input[1] - input[0]);
            Some(result)
        }
    }
}
impl<T: Close + Open + Tbbav> Next<&T> for HbfcOne {
    type Output = Option<f64>;

    fn next(&mut self, input: &T) -> Self::Output {
        match input.tbbav() {
            Some(tbqav) => {
                let result = self.next([input.close(), input.open(), tbqav]);
                result
            }
            None => None,
        }
    }
}

impl<T: Close + Open + Tbbav> Next<Arc<T>> for HbfcOne {
    type Output = Option<f64>;

    fn next(&mut self, input: Arc<T>) -> Self::Output {
        match input.tbbav() {
            Some(tbqav) => {
                let result = self.next([input.close(), input.open(), tbqav]);
                result
            }
            None => None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use crate::DataItem;
    use std::sync::Arc;

    #[test]
    fn test_next_f64_arr() {
        let mut hbfc_one = HbfcOne::new();
        let test_array: [f64; 3] = [10.0, 20.0, 300.0];
        assert_eq!(round(hbfc_one.next(test_array).unwrap()), 30.0);
    }

    #[test]
    fn test_next_with_arc() {
        let mut hbfc_one = HbfcOne::new();

        // 创建一个 DataItem 实例 - 确保 open > close
        let data_item = DataItem::builder()
            .open(20.0) // open 改为 20.0
            .high(25.0)
            .low(5.0)
            .close(10.0) // close 改为 10.0
            .volume(1000.0)
            .tbbav(300.0)
            .build()
            .unwrap();

        // 包装在 Arc 中
        let arc_data = Arc::new(data_item);

        // 测试 Arc<T> 支持
        let result = hbfc_one.next(arc_data);
        // 计算: 300.0 / (20.0 - 10.0) = 300.0 / 10.0 = 30.0
        assert_eq!(round(result.unwrap()), 30.0);
    }
}
