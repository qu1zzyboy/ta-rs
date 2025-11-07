use std::collections::VecDeque;
use std::fmt;

use crate::errors::{Result, TaError};
use crate::indicators::Maximum as Max;
use crate::indicators::Minimum as Min;
use crate::indicators::SimpleMovingAverage as Sma;
use crate::indicators::StandardDeviation as Sd;
use crate::{Close, High, Low, Next, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Support Width Factor configuration.
///
/// This factor identifies volatility regime changes and calculates support width
/// based on price movements during volatility transitions.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct SupportWidthFactorConfig {
    /// Window size for calculating close price standard deviation (default: 60)
    pub std_windowsize: usize,
    /// Time span for comparison (default: 150)
    pub diff_windowsize: usize,
    /// Rolling window size for volatility standardization (default: 150)
    pub std_rolling_windowsize: usize,
    /// Threshold for volatility z-score (default: 1.0)
    pub std_threshold: f64,
    /// Window size for factor smoothing (default: 384)
    pub smooth_windowsize: usize,
}

impl Default for SupportWidthFactorConfig {
    fn default() -> Self {
        Self {
            std_windowsize: 60,
            diff_windowsize: 150,
            std_rolling_windowsize: 150,
            std_threshold: 1.0,
            smooth_windowsize: 384,
        }
    }
}

impl SupportWidthFactorConfig {
    pub fn new(
        std_windowsize: usize,
        diff_windowsize: usize,
        std_rolling_windowsize: usize,
        std_threshold: f64,
        smooth_windowsize: usize,
    ) -> Result<Self> {
        if std_windowsize == 0
            || diff_windowsize == 0
            || std_rolling_windowsize == 0
            || smooth_windowsize == 0
        {
            return Err(TaError::InvalidParameter);
        }
        Ok(Self {
            std_windowsize,
            diff_windowsize,
            std_rolling_windowsize,
            std_threshold,
            smooth_windowsize,
        })
    }
}

/// Support Width Factor indicator.
///
/// This indicator calculates a factor that identifies volatility regime changes
/// and measures the support width during price movements.
///
/// # Algorithm Steps
///
/// 1. Calculate rolling standard deviation of close prices
/// 2. Calculate volatility z-score
/// 3. Identify volatility regime change signals
/// 4. Calculate price returns
/// 5. Calculate rolling maximum and minimum prices
/// 6. Calculate support width factor
/// 7. Smooth the factor
/// 8. Clip and normalize the factor
///
/// # Parameters
///
/// See `SupportWidthFactorConfig` for configuration parameters.
///
/// # Example
///
/// ```
/// use ta::indicators::SupportWidthFactor;
/// use ta::Next;
///
/// let mut factor = SupportWidthFactor::default();
/// let result = factor.next(100.0, 105.0, 95.0, 102.0);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct SupportWidthFactor {
    config: SupportWidthFactorConfig,
    // Step 1: Close price standard deviation
    close_std: Sd,
    // Step 2: Close std mean and std for z-score calculation
    close_std_mean: Sma,
    close_std_std: Sd,
    // Step 5: Rolling maximum and minimum
    hhv: Max,
    llv: Min,
    // Step 8: Smooth factor
    smooth_sma: Sma,
    // History buffers
    close_history: VecDeque<f64>,
    close_std_history: VecDeque<f64>,
    close_std_zscore_history: VecDeque<f64>,
    high_history: VecDeque<f64>,
    low_history: VecDeque<f64>,
    hhv_history: VecDeque<f64>,
    llv_history: VecDeque<f64>,
    // Count of processed data points
    count: usize,
}

impl SupportWidthFactor {
    pub fn new(config: SupportWidthFactorConfig) -> Result<Self> {
        let max_window = config
            .std_windowsize
            .max(config.diff_windowsize)
            .max(config.std_rolling_windowsize)
            .max(config.smooth_windowsize);

        Ok(Self {
            config: config.clone(),
            close_std: Sd::new(config.std_windowsize)?,
            close_std_mean: Sma::new(config.std_rolling_windowsize)?,
            close_std_std: Sd::new(config.std_rolling_windowsize)?,
            hhv: Max::new(config.diff_windowsize)?,
            llv: Min::new(config.diff_windowsize)?,
            smooth_sma: Sma::new(config.smooth_windowsize)?,
            close_history: VecDeque::with_capacity(max_window + 1),
            close_std_history: VecDeque::with_capacity(max_window + 1),
            close_std_zscore_history: VecDeque::with_capacity(max_window + 1),
            high_history: VecDeque::with_capacity(max_window + 1),
            low_history: VecDeque::with_capacity(max_window + 1),
            hhv_history: VecDeque::with_capacity(max_window + 1),
            llv_history: VecDeque::with_capacity(max_window + 1),
            count: 0,
        })
    }

    pub fn with_config(
        std_windowsize: usize,
        diff_windowsize: usize,
        std_rolling_windowsize: usize,
        std_threshold: f64,
        smooth_windowsize: usize,
    ) -> Result<Self> {
        let config = SupportWidthFactorConfig::new(
            std_windowsize,
            diff_windowsize,
            std_rolling_windowsize,
            std_threshold,
            smooth_windowsize,
        )?;
        Self::new(config)
    }

    /// Get historical value at index (count - shift)
    fn get_historical<T>(history: &VecDeque<T>, count: usize, shift: usize) -> Option<&T> {
        if count > shift && history.len() > shift {
            let idx = history.len() - 1 - shift;
            history.get(idx)
        } else {
            None
        }
    }

    /// Calculate z-score, handling division by zero
    fn calculate_zscore(value: f64, mean: f64, std: f64) -> f64 {
        if std == 0.0 {
            0.0
        } else {
            (value - mean) / std
        }
    }

    /// Normalize value from [-0.03, 0.03] to [-1, 1]
    fn normalize(value: f64) -> f64 {
        let clipped = value.max(-0.03).min(0.03);
        (clipped + 0.03) / 0.03 - 1.0
    }
}

impl Default for SupportWidthFactor {
    fn default() -> Self {
        Self::new(SupportWidthFactorConfig::default()).unwrap()
    }
}

impl<T> Next<&T> for SupportWidthFactor
where
    T: High + Low + Close,
{
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        let close = input.close();
        let high = input.high();
        let low = input.low();

        self.next_with_values(close, high, low)
    }
}

impl SupportWidthFactor {
    /// Internal method to process values
    fn next_with_values(&mut self, close: f64, high: f64, low: f64) -> f64 {
        self.count += 1;

        // Step 1: Calculate rolling standard deviation of close prices
        let close_std_val = self.close_std.next(close);
        self.close_std_history.push_back(close_std_val);

        // Step 2: Calculate volatility z-score
        let close_std_mean_val = self.close_std_mean.next(close_std_val);
        let close_std_std_val = self.close_std_std.next(close_std_val);
        let close_std_zscore = Self::calculate_zscore(close_std_val, close_std_mean_val, close_std_std_val);
        self.close_std_zscore_history.push_back(close_std_zscore);

        // Step 3: Identify volatility regime change signal
        let filtered_zscore = if let Some(prev_zscore) = Self::get_historical(
            &self.close_std_zscore_history,
            self.count,
            self.config.diff_windowsize,
        ) {
            if close_std_zscore > self.config.std_threshold && *prev_zscore < -self.config.std_threshold {
                close_std_zscore
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Step 4: Calculate price return
        let filtered_ret = if filtered_zscore != 0.0 {
            if let Some(prev_close) = Self::get_historical(&self.close_history, self.count, self.config.diff_windowsize) {
                (close / prev_close) - 1.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Step 5: Calculate rolling maximum and minimum
        let hhv_val = self.hhv.next(high);
        let llv_val = self.llv.next(low);
        self.hhv_history.push_back(hhv_val);
        self.llv_history.push_back(llv_val);

        // Step 6: Calculate Support_Width
        let support_width = if filtered_zscore != 0.0 {
            if let (Some(prev_hhv), Some(prev_llv)) = (
                Self::get_historical(&self.hhv_history, self.count, self.config.diff_windowsize),
                Self::get_historical(&self.llv_history, self.count, self.config.diff_windowsize),
            ) {
                if filtered_ret > 0.0 {
                    // Upward trend: (HHV - LLV) / LLV
                    (prev_hhv - prev_llv) / prev_llv
                } else if filtered_ret < 0.0 {
                    // Downward trend: (LLV - HHV) / LLV (negative)
                    (prev_llv - prev_hhv) / prev_llv
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Step 7: Handle NaN (skip if NaN)
        if support_width.is_nan() || support_width.is_infinite() {
            self.close_history.push_back(close);
            self.high_history.push_back(high);
            self.low_history.push_back(low);
            return 0.0;
        }

        // Step 8: Smooth the factor
        let smoothed = self.smooth_sma.next(support_width);

        // Step 9: Clip to [-0.03, 0.03]
        let clipped = smoothed.max(-0.03).min(0.03);

        // Step 10: Normalize to [-1, 1]
        let normalized = Self::normalize(clipped);

        // Step 11: Negate (final factor)
        let final_factor = -normalized;

        // Update history
        self.close_history.push_back(close);
        self.high_history.push_back(high);
        self.low_history.push_back(low);

        // Maintain history size (keep enough for diff_windowsize lookback)
        let max_history = self.config.smooth_windowsize.max(self.config.diff_windowsize) + 10;
        while self.close_history.len() > max_history {
            self.close_history.pop_front();
        }
        while self.close_std_history.len() > max_history {
            self.close_std_history.pop_front();
        }
        while self.close_std_zscore_history.len() > max_history {
            self.close_std_zscore_history.pop_front();
        }
        while self.hhv_history.len() > max_history {
            self.hhv_history.pop_front();
        }
        while self.llv_history.len() > max_history {
            self.llv_history.pop_front();
        }
        while self.high_history.len() > max_history {
            self.high_history.pop_front();
        }
        while self.low_history.len() > max_history {
            self.low_history.pop_front();
        }

        final_factor
    }
}

impl Reset for SupportWidthFactor {
    fn reset(&mut self) {
        self.close_std.reset();
        self.close_std_mean.reset();
        self.close_std_std.reset();
        self.hhv.reset();
        self.llv.reset();
        self.smooth_sma.reset();
        self.close_history.clear();
        self.close_std_history.clear();
        self.close_std_zscore_history.clear();
        self.high_history.clear();
        self.low_history.clear();
        self.hhv_history.clear();
        self.llv_history.clear();
        self.count = 0;
    }
}

impl fmt::Display for SupportWidthFactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SupportWidthFactor(std={}, diff={}, rolling={}, smooth={})",
            self.config.std_windowsize,
            self.config.diff_windowsize,
            self.config.std_rolling_windowsize,
            self.config.smooth_windowsize
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    #[test]
    fn test_new() {
        let config = SupportWidthFactorConfig::default();
        assert!(SupportWidthFactor::new(config).is_ok());
    }

    #[test]
    fn test_new_with_invalid_params() {
        assert!(SupportWidthFactorConfig::new(0, 150, 150, 1.0, 384).is_err());
        assert!(SupportWidthFactorConfig::new(60, 0, 150, 1.0, 384).is_err());
        assert!(SupportWidthFactorConfig::new(60, 150, 0, 1.0, 384).is_err());
        assert!(SupportWidthFactorConfig::new(60, 150, 150, 1.0, 0).is_err());
    }

    #[test]
    fn test_default() {
        let factor = SupportWidthFactor::default();
        assert_eq!(factor.config.std_windowsize, 60);
        assert_eq!(factor.config.diff_windowsize, 150);
        assert_eq!(factor.config.smooth_windowsize, 384);
    }

    #[test]
    fn test_next_initial() {
        let mut factor = SupportWidthFactor::default();
        
        // Initial values should not panic
        let result = factor.next_with_values(100.0, 105.0, 95.0);
        assert!(!result.is_nan());
        assert!(!result.is_infinite());
    }

    #[test]
    fn test_reset() {
        let mut factor = SupportWidthFactor::default();
        factor.next_with_values(100.0, 105.0, 95.0);
        factor.reset();
        
        assert_eq!(factor.count, 0);
        assert!(factor.close_history.is_empty());
    }

    #[test]
    fn test_display() {
        let factor = SupportWidthFactor::default();
        let display_str = format!("{}", factor);
        assert!(display_str.contains("SupportWidthFactor"));
    }

    #[test]
    fn test_with_bars() {
        let mut factor = SupportWidthFactor::default();
        
        fn bar(close: f64, high: f64, low: f64) -> Bar {
            Bar::new().high(high).low(low).close(close).volume(0.0)
        }
        
        let result = factor.next(&bar(100.0, 105.0, 95.0));
        assert!(!result.is_nan());
    }
}

