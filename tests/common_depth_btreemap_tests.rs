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

    // 获取买卖价差
    pub fn spread(&self) -> Option<Price> {
        if let (Some((bid_price, _)), Some((ask_price, _))) = (self.best_bid(), self.best_ask()) {
            Some(ask_price - bid_price)
        } else {
            None
        }
    }

    // 计算总买入量
    pub fn total_bid_volume(&self) -> u128 {
        self.bid_list.values().map(|&qty| qty as u128).sum()
    }

    // 计算总卖出量
    pub fn total_ask_volume(&self) -> u128 {
        self.ask_list.values().map(|&qty| qty as u128).sum()
    }

    // 计算买入总价值
    pub fn total_bid_value(&self) -> u128 {
        self.bid_list.iter().map(|(&price, &qty)| price as u128 * qty as u128).sum()
    }

    // 计算卖出总价值
    pub fn total_ask_value(&self) -> u128 {
        self.ask_list.iter().map(|(&price, &qty)| price as u128 * qty as u128).sum()
    }

    // 获取前N档买入订单
    pub fn top_bids(&self, n: usize) -> Vec<(Price, Quantity)> {
        self.bid_list.iter()
            .rev()
            .take(n)
            .map(|(&price, &qty)| (price, qty))
            .collect()
    }

    // 获取前N档卖出订单
    pub fn top_asks(&self, n: usize) -> Vec<(Price, Quantity)> {
        self.ask_list.iter()
            .take(n)
            .map(|(&price, &qty)| (price, qty))
            .collect()
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
                // 删除订单
                self.bid_list.remove(&price);
                self.ask_list.remove(&price);
            } else {
                // 判断是买入还是卖出（简单策略：根据价格范围）
                if let Some((best_bid_price, _)) = self.best_bid() {
                    if price >= best_bid_price {
                        self.bid_list.insert(price, qty);
                    } else {
                        self.ask_list.insert(price, qty);
                    }
                } else {
                    // 没有买入订单，默认为卖出
                    self.ask_list.insert(price, qty);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_depth_creation() {
        let depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        assert_eq!(depth.symbol, "BTCUSDT");
        assert_eq!(depth.exchange, "binance");
        assert_eq!(depth.timestamp, 1640995200000);
        assert!(depth.bid_list.is_empty());
        assert!(depth.ask_list.is_empty());
    }

    #[test]
    fn test_add_bid_and_ask() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // 添加买入订单
        depth.add_bid(50000, 100);
        depth.add_bid(49900, 200);
        depth.add_bid(49800, 150);

        // 添加卖出订单
        depth.add_ask(50100, 80);
        depth.add_ask(50200, 120);
        depth.add_ask(50300, 90);

        assert_eq!(depth.bid_list.len(), 3);
        assert_eq!(depth.ask_list.len(), 3);

        // 验证订单按价格排序
        let bid_prices: Vec<Price> = depth.bid_list.keys().cloned().collect();
        assert_eq!(bid_prices, vec![49800, 49900, 50000]);

        let ask_prices: Vec<Price> = depth.ask_list.keys().cloned().collect();
        assert_eq!(ask_prices, vec![50100, 50200, 50300]);
    }

    #[test]
    fn test_best_bid_and_ask() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        depth.add_bid(50000, 100);
        depth.add_bid(49900, 200);
        depth.add_ask(50100, 80);
        depth.add_ask(50200, 120);

        assert_eq!(depth.best_bid(), Some((50000, 100)));
        assert_eq!(depth.best_ask(), Some((50100, 80)));
    }

    #[test]
    fn test_spread_calculation() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        depth.add_bid(50000, 100);
        depth.add_ask(50100, 80);

        assert_eq!(depth.spread(), Some(100));

        // 测试没有买卖订单的情况
        let empty_depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );
        assert_eq!(empty_depth.spread(), None);
    }

    #[test]
    fn test_volume_calculations() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        depth.add_bid(50000, 100);
        depth.add_bid(49900, 200);
        depth.add_ask(50100, 80);
        depth.add_ask(50200, 120);

        assert_eq!(depth.total_bid_volume(), 300);
        assert_eq!(depth.total_ask_volume(), 200);

        assert_eq!(depth.total_bid_value(), 50000 * 100 + 49900 * 200);
        assert_eq!(depth.total_ask_value(), 50100 * 80 + 50200 * 120);
    }

    #[test]
    fn test_top_levels() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        depth.add_bid(50000, 100);
        depth.add_bid(49900, 200);
        depth.add_bid(49800, 150);
        depth.add_bid(49700, 300);

        depth.add_ask(50100, 80);
        depth.add_ask(50200, 120);
        depth.add_ask(50300, 90);
        depth.add_ask(50400, 110);

        // 获取前3档买入订单（按价格从高到低）
        let top_bids = depth.top_bids(3);
        assert_eq!(top_bids, vec![(50000, 100), (49900, 200), (49800, 150)]);

        // 获取前3档卖出订单（按价格从低到高）
        let top_asks = depth.top_asks(3);
        assert_eq!(top_asks, vec![(50100, 80), (50200, 120), (50300, 90)]);
    }

    #[test]
    fn test_binance_full_update() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // 先添加一些旧数据
        depth.add_bid(50000, 100);
        depth.add_ask(50100, 80);

        // Binance全量更新
        let new_bids = vec![(51000, 150), (50900, 200), (50800, 100)];
        let new_asks = vec![(51100, 120), (51200, 180), (51300, 90)];

        depth.update_from_binance(new_bids, new_asks);

        // 验证旧数据被清除，新数据被添加
        assert_eq!(depth.bid_list.len(), 3);
        assert_eq!(depth.ask_list.len(), 3);

        assert_eq!(depth.best_bid(), Some((51000, 150)));
        assert_eq!(depth.best_ask(), Some((51100, 120)));

        // 验证价格排序
        let bid_prices: Vec<Price> = depth.bid_list.keys().cloned().collect();
        assert_eq!(bid_prices, vec![50800, 50900, 51000]);
    }

    #[test]
    fn test_mexc_incremental_update() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "mexc".to_string(),
            1640995200000,
        );

        // 初始状态
        depth.add_bid(50000, 100);
        depth.add_ask(50100, 80);

        // MEXC增量更新
        let updates = vec![
            (50000, 0),    // 删除买入订单
            (50100, 0),    // 删除卖出订单
            (49900, 200),  // 添加新的买入订单
            (50200, 120),  // 添加新的卖出订单
        ];

        depth.update_from_mexc(updates);

        // 验证更新结果
        assert_eq!(depth.bid_list.len(), 1);
        assert_eq!(depth.ask_list.len(), 1);

        assert_eq!(depth.best_bid(), Some((49900, 200)));
        assert_eq!(depth.best_ask(), Some((50200, 120)));

        // 验证旧订单被删除
        assert!(depth.bid_list.get(&50000).is_none());
        assert!(depth.ask_list.get(&50100).is_none());
    }

    #[test]
    fn test_zero_quantity_handling() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // 添加订单
        depth.add_bid(50000, 100);
        depth.add_ask(50100, 80);

        // 设置数量为0应该删除订单
        depth.add_bid(50000, 0);
        depth.add_ask(50100, 0);

        assert!(depth.bid_list.is_empty());
        assert!(depth.ask_list.is_empty());
    }

    #[test]
    fn test_large_orderbook_performance() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // 添加大量订单测试性能
        let start = std::time::Instant::now();

        for i in 0..1000 {
            depth.add_bid(50000 + i, 100 + i);
            depth.add_ask(51000 + i, 80 + i);
        }

        let duration = start.elapsed();
        println!("Added 2000 orders in {:?}", duration);

        assert_eq!(depth.bid_list.len(), 1000);
        assert_eq!(depth.ask_list.len(), 1000);

        // 测试查找性能
        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = depth.best_bid();
            let _ = depth.best_ask();
        }
        let duration = start.elapsed();
        println!("10000 lookups in {:?}", duration);
    }

    #[test]
    fn test_orderbook_consistency() {
        let mut depth = CommonDepth::new(
            "BTCUSDT".to_string(),
            "binance".to_string(),
            1640995200000,
        );

        // 添加一些订单
        depth.add_bid(50000, 100);
        depth.add_bid(49900, 200);
        depth.add_ask(50100, 80);
        depth.add_ask(50200, 120);

        // 验证订单簿一致性
        assert!(depth.best_bid().unwrap().0 < depth.best_ask().unwrap().0);
        assert!(depth.spread().unwrap() > 0);

        // 验证总价值计算
        let bid_value = depth.total_bid_value();
        let ask_value = depth.total_ask_value();
        assert!(bid_value > 0);
        assert!(ask_value > 0);

        // 验证价格排序
        let bid_prices: Vec<Price> = depth.bid_list.keys().cloned().collect();
        let ask_prices: Vec<Price> = depth.ask_list.keys().cloned().collect();

        for i in 1..bid_prices.len() {
            assert!(bid_prices[i-1] < bid_prices[i]);
        }

        for i in 1..ask_prices.len() {
            assert!(ask_prices[i-1] < ask_prices[i]);
        }
    }
}
