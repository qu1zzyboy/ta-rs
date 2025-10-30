use std::collections::BTreeMap;
use ta::ob_indicators::VolumeImbalance;
use ta::{Next, Orderbook};

// 类型别名
pub type Price = u64;
pub type Quantity = u64;

// 测试用的订单簿结构
#[derive(Debug, Clone)]
pub struct TestOrderbook {
    pub bid_list: BTreeMap<Price, Quantity>,
    pub ask_list: BTreeMap<Price, Quantity>,
}

impl TestOrderbook {
    pub fn new() -> Self {
        Self {
            bid_list: BTreeMap::new(),
            ask_list: BTreeMap::new(),
        }
    }

    pub fn add_bid(&mut self, price: Price, quantity: Quantity) {
        if quantity > 0 {
            self.bid_list.insert(price, quantity);
        }
    }

    pub fn add_ask(&mut self, price: Price, quantity: Quantity) {
        if quantity > 0 {
            self.ask_list.insert(price, quantity);
        }
    }
}

// 实现 Orderbook trait
impl Orderbook for TestOrderbook {
    fn get_bids(&self) -> Option<Box<dyn Iterator<Item = (Price, Quantity)> + '_>> {
        Some(Box::new(self.bid_list.iter().map(|(&price, &qty)| (price, qty))))
    }

    fn get_asks(&self) -> Option<Box<dyn Iterator<Item = (Price, Quantity)> + '_>> {
        Some(Box::new(self.ask_list.iter().map(|(&price, &qty)| (price, qty))))
    }

    fn get_bids_btm(&self) -> Option<BTreeMap<Price, Quantity>> {
        Some(self.bid_list.clone())
    }

    fn get_asks_btm(&self) -> Option<BTreeMap<Price, Quantity>> {
        Some(self.ask_list.clone())
    }
}

fn main() {
    println!("VolumeImbalance BTreeMap Performance Test");
    println!("=========================================");

    // 创建测试订单簿
    let mut orderbook = TestOrderbook::new();
    
    // 添加大量订单数据
    for i in 0..1000 {
        orderbook.add_bid(50000 + i, 100 + i);
        orderbook.add_ask(51000 + i, 80 + i);
    }

    let mut vi = VolumeImbalance::new();

    // 性能测试
    let iterations = 10000;
    let start = std::time::Instant::now();
    
    for _ in 0..iterations {
        let _imbalance = vi.next(&orderbook);
    }
    
    let duration = start.elapsed();
    
    println!("Iterations: {}", iterations);
    println!("Total time: {:?}", duration);
    println!("Average time per calculation: {:?}", duration / iterations);
    println!("Orders in orderbook: {} bids, {} asks", 
             orderbook.bid_list.len(), 
             orderbook.ask_list.len());
    
    // 验证结果一致性
    let imbalance1 = vi.next(&orderbook);
    let imbalance2 = vi.next(&orderbook);
    println!("Result consistency check: {} == {} ? {}", 
             imbalance1, imbalance2, imbalance1 == imbalance2);
    
    // 显示实际的 imbalance 值
    println!("Volume imbalance: {} ({:.4})", 
             imbalance1, 
             imbalance1 as f64 / 10000.0);
}
