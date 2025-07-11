use std::fmt;

use crate::{Close, DataItem, Next, Open, Reset, Tbbav};

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
impl<T: Close + Open + Tbbav> Next<T> for HbfcOne {
    type Output = Option<f64>;

    fn next(&mut self, input: T) -> Self::Output {
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
    #[test]
    fn test_next_f64_arr() {
        let mut hbfc_one = HbfcOne::new();
        let test_array: [f64; 3] = [10.0, 20.0, 300.0];
        assert_eq!(round(hbfc_one.next(test_array).unwrap()), 30.0);
    }
}
