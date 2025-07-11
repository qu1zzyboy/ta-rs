use std::fmt;

use crate::errors::{Result, TaError};
use crate::{Close, Next, Period, Reset};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct HbfcOneSd {
    period: usize,
    index: usize,
    count: usize,
    sum: f64,
    st: f64,
    deque: Box<[f64]>,
}

impl HbfcOneSd {
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0 => Err(),
        }
    }
}
