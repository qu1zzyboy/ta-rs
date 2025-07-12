use crate::errors::*;
use crate::traits::{Close, High, Low, Not, Open, Qav, Tbbav, Tbqav, Volume};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Data item is used as an input for indicators.
///
/// # Example
///
/// ```
/// use ta::DataItem;
/// use ta::{Open, High, Low, Close, Volume, Qav, Tbbav, Tbqav, Not};
///
/// let item = DataItem::builder()
///     .open(20.0)
///     .high(25.0)
///     .low(15.0)
///     .close(21.0)
///     .volume(7500.0)
///     .qav(1000.0)
///     .tbbav(500.0)
///     .tbqav(100.0)
///     .not(150)
///     .timestamp(1672531200)
///     .build()
///     .unwrap();
///
/// assert_eq!(item.open(), 20.0);
/// assert_eq!(item.high(), 25.0);
/// assert_eq!(item.low(), 15.0);
/// assert_eq!(item.close(), 21.0);
/// assert_eq!(item.volume(), 7500.0);
/// assert_eq!(item.qav(), Some(1000.0));
/// assert_eq!(item.tbbav(), Some(500.0));
/// assert_eq!(item.tbqav(), Some(100.0));
/// assert_eq!(item.not(), Some(150));
/// ```
///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DataItem {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    qav: Option<f64>,       // Quote asset volume
    tbbav: Option<f64>,     // Taker buy base asset volume
    tbqav: Option<f64>,     // Taker buy quote asset volume
    not: Option<u64>,       // Number of trades
    timestamp: Option<u64>, // Unix timestamp
}

impl DataItem {
    pub fn builder() -> DataItemBuilder {
        DataItemBuilder::new()
    }

    // 便利方法：检查是否有时间戳
    pub fn has_timestamp(&self) -> bool {
        self.timestamp.is_some()
    }

    // 便利方法：获取时间戳
    pub fn timestamp(&self) -> Option<u64> {
        self.timestamp
    }
}

// 实现基本的 OHLCV traits
impl Open for DataItem {
    fn open(&self) -> f64 {
        self.open
    }
}

impl High for DataItem {
    fn high(&self) -> f64 {
        self.high
    }
}

impl Low for DataItem {
    fn low(&self) -> f64 {
        self.low
    }
}

impl Close for DataItem {
    fn close(&self) -> f64 {
        self.close
    }
}

impl Volume for DataItem {
    fn volume(&self) -> f64 {
        self.volume
    }
}

// 实现新添加的可选字段 traits
impl Qav for DataItem {
    fn qav(&self) -> Option<f64> {
        self.qav
    }
}

impl Tbbav for DataItem {
    fn tbbav(&self) -> Option<f64> {
        self.tbbav
    }
}

impl Tbqav for DataItem {
    fn tbqav(&self) -> Option<f64> {
        self.tbqav
    }
}

impl Not for DataItem {
    fn not(&self) -> Option<u64> {
        self.not
    }
}

// Builder 模式实现
pub struct DataItemBuilder {
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: Option<f64>,
    volume: Option<f64>,
    qav: Option<f64>,
    tbbav: Option<f64>,
    tbqav: Option<f64>,
    not: Option<u64>,
    timestamp: Option<u64>,
}

impl DataItemBuilder {
    pub fn new() -> Self {
        Self {
            open: None,
            high: None,
            low: None,
            close: None,
            volume: None,
            qav: None,
            tbbav: None,
            tbqav: None,
            not: None,
            timestamp: None,
        }
    }

    // 必需字段的设置方法
    pub fn open(mut self, val: f64) -> Self {
        self.open = Some(val);
        self
    }

    pub fn high(mut self, val: f64) -> Self {
        self.high = Some(val);
        self
    }

    pub fn low(mut self, val: f64) -> Self {
        self.low = Some(val);
        self
    }

    pub fn close(mut self, val: f64) -> Self {
        self.close = Some(val);
        self
    }

    pub fn volume(mut self, val: f64) -> Self {
        self.volume = Some(val);
        self
    }

    // 可选字段的设置方法
    pub fn qav(mut self, val: f64) -> Self {
        self.qav = Some(val);
        self
    }

    pub fn tbbav(mut self, val: f64) -> Self {
        self.tbbav = Some(val);
        self
    }

    pub fn tbqav(mut self, val: f64) -> Self {
        self.tbqav = Some(val);
        self
    }

    pub fn not(mut self, val: u64) -> Self {
        self.not = Some(val);
        self
    }

    pub fn timestamp(mut self, val: u64) -> Self {
        self.timestamp = Some(val);
        self
    }

    // 便利方法：设置当前时间戳
    pub fn timestamp_now(mut self) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.timestamp = Some(now);
        self
    }

    // 构建 DataItem
    pub fn build(self) -> Result<DataItem> {
        // 检查所有必需字段是否都有值
        if let (Some(open), Some(high), Some(low), Some(close), Some(volume)) =
            (self.open, self.high, self.low, self.close, self.volume)
        {
            // 验证 OHLCV 数据的合法性
            if low <= open
                && low <= close
                && low <= high
                && high >= open
                && high >= close
                && volume >= 0.0
            {
                // 验证可选字段的合法性
                if let Some(qav) = self.qav {
                    if qav < 0.0 {
                        return Err(TaError::DataItemInvalid);
                    }
                }

                if let Some(tbbav) = self.tbbav {
                    if tbbav < 0.0 {
                        return Err(TaError::DataItemInvalid);
                    }
                }

                if let Some(tbqav) = self.tbqav {
                    if tbqav < 0.0 {
                        return Err(TaError::DataItemInvalid);
                    }
                }

                // 创建 DataItem
                let item = DataItem {
                    open,
                    high,
                    low,
                    close,
                    volume,
                    qav: self.qav,
                    tbbav: self.tbbav,
                    tbqav: self.tbqav,
                    not: self.not,
                    timestamp: self.timestamp,
                };
                Ok(item)
            } else {
                Err(TaError::DataItemInvalid)
            }
        } else {
            Err(TaError::DataItemIncomplete)
        }
    }
}

// 为 Builder 实现 Default trait
impl Default for DataItemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_builder() {
        let item = DataItem::builder()
            .open(20.0)
            .high(25.0)
            .low(15.0)
            .close(21.0)
            .volume(7500.0)
            .build()
            .unwrap();

        assert_eq!(item.open(), 20.0);
        assert_eq!(item.high(), 25.0);
        assert_eq!(item.low(), 15.0);
        assert_eq!(item.close(), 21.0);
        assert_eq!(item.volume(), 7500.0);

        // 可选字段应该是 None
        assert_eq!(item.qav(), None);
        assert_eq!(item.tbbav(), None);
        assert_eq!(item.tbqav(), None);
        assert_eq!(item.not(), None);
        assert_eq!(item.timestamp(), None);
    }

    #[test]
    fn test_full_builder() {
        let item = DataItem::builder()
            .open(20.0)
            .high(25.0)
            .low(15.0)
            .close(21.0)
            .volume(7500.0)
            .qav(1000.0)
            .tbbav(500.0)
            .tbqav(100.0)
            .not(150)
            .timestamp(1672531200)
            .build()
            .unwrap();

        assert_eq!(item.open(), 20.0);
        assert_eq!(item.high(), 25.0);
        assert_eq!(item.low(), 15.0);
        assert_eq!(item.close(), 21.0);
        assert_eq!(item.volume(), 7500.0);
        assert_eq!(item.qav(), Some(1000.0));
        assert_eq!(item.tbbav(), Some(500.0));
        assert_eq!(item.tbqav(), Some(100.0));
        assert_eq!(item.not(), Some(150));
        assert_eq!(item.timestamp(), Some(1672531200));
    }

    #[test]
    fn test_invalid_data() {
        // 测试无效的 OHLCV 数据
        let result = DataItem::builder()
            .open(20.0)
            .high(15.0) // high < low，无效
            .low(25.0)
            .close(21.0)
            .volume(7500.0)
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TaError::DataItemInvalid);
    }

    #[test]
    fn test_incomplete_data() {
        // 测试不完整的数据
        let result = DataItem::builder()
            .open(20.0)
            .high(25.0)
            // 缺少 low, close, volume
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TaError::DataItemIncomplete);
    }

    #[test]
    fn test_invalid_optional_fields() {
        // 测试无效的可选字段
        let result = DataItem::builder()
            .open(20.0)
            .high(25.0)
            .low(15.0)
            .close(21.0)
            .volume(7500.0)
            .qav(-100.0) // 负数，无效
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TaError::DataItemInvalid);
    }

    #[test]
    fn test_timestamp_methods() {
        let item_without_timestamp = DataItem::builder()
            .open(20.0)
            .high(25.0)
            .low(15.0)
            .close(21.0)
            .volume(7500.0)
            .build()
            .unwrap();

        assert!(!item_without_timestamp.has_timestamp());
        assert_eq!(item_without_timestamp.timestamp(), None);

        let item_with_timestamp = DataItem::builder()
            .open(20.0)
            .high(25.0)
            .low(15.0)
            .close(21.0)
            .volume(7500.0)
            .timestamp(1672531200)
            .build()
            .unwrap();

        assert!(item_with_timestamp.has_timestamp());
        assert_eq!(item_with_timestamp.timestamp(), Some(1672531200));
    }

    #[test]
    fn test_timestamp_now() {
        let item = DataItem::builder()
            .open(20.0)
            .high(25.0)
            .low(15.0)
            .close(21.0)
            .volume(7500.0)
            .timestamp_now()
            .build()
            .unwrap();

        assert!(item.has_timestamp());
        assert!(item.timestamp().is_some());
        // 时间戳应该是最近的
        let ts = item.timestamp().unwrap();
        assert!(ts > 1600000000); // 大于 2020 年
    }
}
