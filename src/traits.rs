use std::collections::BTreeMap;
use ordered_float::OrderedFloat;
use crate::types::{U64Price, U64Quantity};
// Indicator traits
//

/// Resets an indicator to the initial state.
pub trait Reset {
    fn reset(&mut self);
}

/// Return the period used by the indicator.
pub trait Period {
    fn period(&self) -> usize;
}

/// Consumes a data item of type `T` and returns `Output`.
///
/// Typically `T` can be `f64` or a struct similar to [DataItem](struct.DataItem.html), that implements
/// traits necessary to calculate value of a particular indicator.
///
/// In most cases `Output` is `f64`, but sometimes it can be different. For example for
/// [MACD](indicators/struct.MovingAverageConvergenceDivergence.html) it is `(f64, f64, f64)` since
/// MACD returns 3 values.
///
pub trait Next<T> {
    type Output;
    fn next(&mut self, input: T) -> Self::Output;
}

pub trait Update<T> {
    type Output;
    fn update(&mut self, input: T) -> Self::Output;
}
/// Open price of a particular period.
pub trait Open {
    fn open(&self) -> f64;
}

/// Close price of a particular period.
pub trait Close {
    fn close(&self) -> f64;
}

/// Lowest price of a particular period.
pub trait Low {
    fn low(&self) -> f64;
}

/// Highest price of a particular period.
pub trait High {
    fn high(&self) -> f64;
}

/// Trading volume of a particular trading period.
pub trait Volume {
    fn volume(&self) -> f64;
}
//Quote asset volume
pub trait Qav {
    fn qav(&self) -> Option<f64>;
}
//Taker buy base asset volume
pub trait Tbbav {
    fn tbbav(&self) -> Option<f64>;
}

//Taker buy quote asset volume
pub trait Tbqav {
    fn tbqav(&self) -> Option<f64>;
}

//Number of Trades
pub trait Not {
    fn not(&self) -> Option<u64>;
}

pub trait IsClosed {
    fn is_closed(&self) -> Option<bool>;
}
pub trait Timestamp{
    fn timestamp(&self) -> u64;
}

pub trait Orderbookf64{
    fn get_bids_btm(&self) -> &BTreeMap<OrderedFloat<f64>, f64>;
    fn get_asks_btm(&self) -> &BTreeMap<OrderedFloat<f64>, f64>;
}

pub trait TradeTickerf64: Timestamp {
    fn get_trade_price(&self) -> f64;
    fn get_trade_quantity(&self) -> f64;
}

pub trait OrderTickerf64:Timestamp{
    fn get_best_bid_price(&self) -> f64;
    fn get_best_ask_price(&self) -> f64;
    fn get_best_bid_quantity(&self) -> f64;
    fn get_best_ask_quantity(&self) -> f64;
}

pub trait OrderbookU64 {
    fn get_bids_btm(&self) -> &BTreeMap<U64Price, U64Quantity>;
    fn get_asks_btm(&self) -> &BTreeMap<U64Price, U64Quantity>;
}
/// OrderTicker trait for accessing best bid/ask price data
pub trait OrderTickerU64 {
    fn get_best_bid_price(&self) -> u64;
    fn get_best_ask_price(&self) -> u64;
    fn get_best_bid_quantity(&self) -> u64;
    fn get_best_ask_quantity(&self) -> u64;
}
pub trait TradeTickerU64: Timestamp {
    fn get_trade_price(&self) -> u64;
    fn get_trade_quantity(&self) -> u64;
}

/// BatchTradeTicker trait for accessing a batch of trade ticks within a time window.
///
/// Returns `None` if the time window is empty (no trades), or `Some(&[T])` with the slice
/// of trade ticks in the window.
pub trait BatchTradeTickerU64<T: TradeTickerU64> {
    fn get_batch_trade_ticker(&self) -> Option<&[T]>;
}

pub trait BatchOrderTickerU64<T: OrderTickerU64> {
    fn get_batch_order_ticker(&self) -> Option<&[T]>;
}

pub trait BatchTradeTickerf64<T: TradeTickerf64> {
    fn get_batch_trade_ticker(&self) -> Option<&[T]>;
}

pub trait BatchOrderTickerf64<T: OrderTickerf64> {
    fn get_batch_order_ticker(&self) -> Option<&[T]>;
}