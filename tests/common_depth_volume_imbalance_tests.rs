use std::collections::BTreeMap;

// 类型别名
pub type Price = u64;
pub type Quantity = u64;
pub type TradingSymbol = String;
pub type Exchange = String;

// 你的实盘数据结构
#[derive(Debug, Clone)]
pub struct CommonDepth {
    pub bid_list: BTreeMap<Price, Quantity>,
    pub ask_list: BTreeMap<Price, Quantity>,
    pub symbol: TradingSymbol,
    pub timestamp: i64,
    pub exchange: Exchange,
}

impl CommonDepth {
    pub fn new(symbol: TradingSymbol, exchange: Exchange, timestamp: i64) -> Self {
        Self {
            bid_list: BTreeMap::new(),
            ask_list: BTreeMap::new(),
            symbol,
            timestamp,
            exchange,
        }
    }

    // 添加买入订单
    pub fn add_bid(&mut self, price: Price, quantity: Quantity) {
        if quantity > 0 {
            self.bid_list.insert(price, quantity);
        } else {
            self.bid_list.remove(&price);
        }
    }

    // 添加卖出订单
    pub fn add_ask(&mut self, price: Price, quantity: Quantity) {
        if quantity > 0 {
            self.ask_list.insert(price, quantity);
        } else {
            self.ask_list.remove(&price);
        }
    }

    // 获取最佳买入价
    pub fn best_bid(&self) -> Option<(Price, Quantity)> {
        self.bid_list.iter().next_back().map(|(&price, &qty)| (price, qty))
    }

    // 获取最佳卖出价
    pub fn best_ask(&self) -> Option<(Price, Quantity)> {
        self.ask_list.iter().next().map(|(&price, &qty)| (price, qty))
    }

    // 更新订单簿（Binance全量更新）
    pub fn update_from_binance(&mut self, bids: Vec<(Price, Quantity)>, asks: Vec<(Price, Quantity)>) {
        self.bid_list.clear();
        self.ask_list.clear();

        for (price, qty) in bids {
            if qty > 0 {
                self.bid_list.insert(price, qty);
            }
        }

        for (price, qty) in asks {
            if qty > 0 {
                self.ask_list.insert(price, qty);
            }
        }
    }

    // 增量更新订单簿（MEXC增量更新）
    pub fn update_from_mexc(&mut self, updates: Vec<(Price, Quantity)>) {
        for (price, qty) in updates {
            if qty == 0 {
                self.bid_list.remove(&price);
                self.ask_list.remove(&price);
            } else {
                // 改进的判断逻辑：根据价格范围判断买入/卖出
                // 买入订单：价格低于当前最低卖出价
                // 卖出订单：价格高于当前最高买入价
                let should_be_bid = if let Some((best_ask_price, _)) = self.ask_list.iter().next() {
                    price < *best_ask_price
                } else if let Some((best_bid_price, _)) = self.bid_list.iter().next_back() {
                    price <= *best_bid_price
                } else {
                    // 如果订单簿为空，根据价格判断（简单策略：低于50000为买入）
                    price < 50000
                };

                if should_be_bid {
                    self.bid_list.insert(price, qty);
                } else {
                    self.ask_list.insert(price, qty);
                }
            }
        }
    }
}

// 实现 Orderbook trait 以支持 VolumeImbalance 指标
impl ta::Orderbook for CommonDepth {
    fn get_bids_btm(&self) -> &BTreeMap<Price, Quantity> {
        &self.bid_list
    }

    fn get_asks_btm(&self) -> &BTreeMap<Price, Quantity> {
        &self.ask_list
    }
}

#[cfg(test)]
mod volume_imbalance_integration_tests {
    use super::*;
    use ta::ob_indicators::VolumeImbalance;
    use ta::Next;

    #[test]
    fn test_volume_imbalance_with_common_depth() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // 添加订单数据
        depth.add_bid(50000, 100);  // bid volume = 5,000,000
        depth.add_bid(49900, 200);  // bid volume = 9,980,000
        depth.add_ask(50100, 80);   // ask volume = 4,008,000
        depth.add_ask(50200, 120);  // ask volume = 6,024,000

        // 使用 VolumeImbalance 指标
        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&depth);

        // 计算期望值
        // bid_volume = 5,000,000 + 9,980,000 = 14,980,000
        // ask_volume = 4,008,000 + 6,024,000 = 10,032,000
        // imbalance = (14,980,000 * 10000) / 10,032,000 = 14,932
        let expected = (14_980_000 * 10000) / 10_032_000;
        assert_eq!(imbalance, expected);
    }

    #[test]
    fn test_volume_imbalance_binance_update() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // Binance 全量更新
        let bids = vec![(51000, 150), (50900, 200), (50800, 100)];
        let asks = vec![(51100, 120), (51200, 180), (51300, 90)];

        depth.update_from_binance(bids, asks);

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&depth);

        // 计算期望值
        // bid_volume = 51000*150 + 50900*200 + 50800*100 = 7,650,000 + 10,180,000 + 5,080,000 = 22,910,000
        // ask_volume = 51100*120 + 51200*180 + 51300*90 = 6,132,000 + 9,216,000 + 4,617,000 = 19,965,000
        // imbalance = (22,910,000 * 10000) / 19,965,000 = 11,473
        let expected = (22_910_000 * 10000) / 19_965_000;
        assert_eq!(imbalance, expected);
    }

    #[test]
    fn test_volume_imbalance_mexc_incremental() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "mexc".to_string(),
            1640995200000,
        );

        // 初始状态
        depth.add_bid(50000, 100);
        depth.add_ask(50100, 80);

        // MEXC 增量更新
        let updates = vec![
            (50000, 0),    // 删除买入订单
            (50100, 0),    // 删除卖出订单
            (49900, 200),  // 添加新的买入订单
            (50200, 120),  // 添加新的卖出订单
        ];

        depth.update_from_mexc(updates);

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&depth);

        // 计算期望值
        // bid_volume = 49900 * 200 = 9,980,000
        // ask_volume = 50200 * 120 = 6,024,000
        // imbalance = (9,980,000 * 10000) / 6,024,000 = 16,568
        let expected = (9_980_000 * 10000) / 6_024_000;
        assert_eq!(imbalance, expected);
    }

    #[test]
    fn test_volume_imbalance_edge_cases() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        let mut vi = VolumeImbalance::new();

        // 测试空订单簿
        let imbalance = vi.next(&depth);
        assert_eq!(imbalance, 10000); // 中性值

        // 测试只有买入订单
        depth.add_bid(50000, 100);
        let imbalance = vi.next(&depth);
        assert_eq!(imbalance, u64::MAX); // 最大值

        // 测试只有卖出订单
        depth.bid_list.clear();
        depth.add_ask(50100, 80);
        let imbalance = vi.next(&depth);
        assert_eq!(imbalance, 0); // 最小值
    }

    #[test]
    fn test_volume_imbalance_overflow_handling() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // 添加会导致溢出的订单
        depth.add_bid(u64::MAX, 2);  // 这会导致乘法溢出
        depth.add_ask(1, 1);

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&depth);

        // 应该处理溢出并返回最大值
        assert_eq!(imbalance, u64::MAX);
    }

    #[test]
    fn test_volume_imbalance_realistic_data() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // 模拟真实的订单簿数据
        let bids = vec![
            (50000, 1000),  // 50,000,000
            (49950, 2000),  // 99,900,000
            (49900, 1500),  // 74,850,000
            (49850, 3000),  // 149,550,000
            (49800, 1200),  // 59,760,000
        ];

        let asks = vec![
            (50100, 800),   // 40,080,000
            (50150, 1200),  // 60,180,000
            (50200, 900),   // 45,180,000
            (50250, 1500),  // 75,375,000
            (50300, 1100),  // 55,330,000
        ];

        depth.update_from_binance(bids, asks);

        let mut vi = VolumeImbalance::new();
        let imbalance = vi.next(&depth);

        // 计算期望值
        let bid_volume: u64 = depth.bid_list.iter()
            .map(|(&price, &qty)| price * qty)
            .sum();
        let ask_volume: u64 = depth.ask_list.iter()
            .map(|(&price, &qty)| price * qty)
            .sum();

        let expected = (bid_volume * 10000) / ask_volume;
        assert_eq!(imbalance, expected);

        println!("Bid volume: {}", bid_volume);
        println!("Ask volume: {}", ask_volume);
        println!("Volume imbalance: {} ({:.4})", imbalance, imbalance as f64 / 10000.0);
    }

    #[test]
    fn test_volume_imbalance_performance() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // 创建大量订单
        for i in 0..100 {
            depth.add_bid(50000 + i, 100 + i);
            depth.add_ask(51000 + i, 80 + i);
        }

        let mut vi = VolumeImbalance::new();

        // 性能测试
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _imbalance = vi.next(&depth);
        }
        let duration = start.elapsed();

        println!("1000 VolumeImbalance calculations in {:?}", duration);
        println!("Average time per calculation: {:?}", duration / 1000);

        // 验证结果一致性
        let imbalance1 = vi.next(&depth);
        let imbalance2 = vi.next(&depth);
        assert_eq!(imbalance1, imbalance2);
    }

    #[test]
    fn test_volume_imbalance_multiple_exchanges() {
        // 测试多个交易所的订单簿
        let mut binance_depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        let mut mexc_depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "mexc".to_string(),
            1640995200000,
        );

        // 设置不同的订单簿数据
        binance_depth.add_bid(50000, 100);
        binance_depth.add_ask(50100, 80);

        mexc_depth.add_bid(49900, 200);
        mexc_depth.add_ask(50200, 120);

        let mut vi = VolumeImbalance::new();

        let binance_imbalance = vi.next(&binance_depth);
        let mexc_imbalance = vi.next(&mexc_depth);

        // 验证不同交易所的订单簿产生不同的 imbalance
        assert_ne!(binance_imbalance, mexc_imbalance);

        println!("Binance imbalance: {}", binance_imbalance);
        println!("MEXC imbalance: {}", mexc_imbalance);
    }
}
