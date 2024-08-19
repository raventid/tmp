use crate::binance_payloads;
use std::collections::BTreeMap;

// Additional types and traits
type Price = u64;
type Quantity = u64;

const CONVERSION_FACTOR: f64 = 10000.0;

trait ToU64 {
    fn to_u64(self) -> u64;
}

impl ToU64 for f64 {
    #[inline]
    fn to_u64(self) -> u64 {
        (self * CONVERSION_FACTOR).round() as u64
    }
}

// Binance orderbook implementation
#[derive(Debug)]
pub struct OrderBook {
    #[allow(dead_code)]
    symbol: String,
    bids: BTreeMap<Price, Quantity>,
    asks: BTreeMap<Price, Quantity>,
    last_update_id: u64,
}

impl OrderBook {
    pub fn new(symbol: String) -> OrderBook {
        OrderBook {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_id: 0,
        }
    }

    pub fn update_book_ticker(&mut self, data: &binance_payloads::BookTickerUpdate) {
        self.bids.insert(
            data.best_bid_price.to_u64() as Price,
            data.best_bid_quantity.to_u64() as Quantity,
        );
        self.asks.insert(
            data.best_ask_price.to_u64() as Price,
            data.best_ask_quantity.to_u64() as Quantity,
        );
    }

    pub fn update_depth(&mut self, data: &binance_payloads::DepthUpdate) {
        if data.last_update_id <= self.last_update_id {
            return;
        }

        for (price, qty) in &data.bids {
            let price_u64 = price.to_u64() as Price;
            let qty_u64 = qty.to_u64() as Quantity;
            if qty_u64 == 0 {
                self.bids.remove(&price_u64);
            } else {
                self.bids.insert(price_u64, qty_u64);
            }
        }

        for (price, qty) in &data.asks {
            let price_u64 = price.to_u64() as Price;
            let qty_u64 = qty.to_u64() as Quantity;
            if qty_u64 == 0 {
                self.asks.remove(&price_u64);
            } else {
                self.asks.insert(price_u64, qty_u64);
            }
        }

        self.last_update_id = data.last_update_id;
    }

    // TODO: Use better types ((BID_PRICE, BID_QUANTITY), (ASK_PRICE, ASK_QUANTITY))
    #[allow(dead_code)]
    fn get_best_bid_ask(&self) -> Option<((f64, f64), (f64, f64))> {
        match (self.bids.iter().next_back(), self.asks.iter().next()) {
            (Some(best_bid), Some(best_ask)) => Some((
                (
                    *best_bid.0 as f64 / CONVERSION_FACTOR,
                    *best_bid.1 as f64 / CONVERSION_FACTOR,
                ),
                (
                    *best_ask.0 as f64 / CONVERSION_FACTOR,
                    *best_ask.1 as f64 / CONVERSION_FACTOR,
                ),
            )),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn get_volume_at_price(&self, price: f64) -> f64 {
        let price_u64 = price.to_u64() as Price;
        (self.bids.get(&price_u64).unwrap_or(&0) + self.asks.get(&price_u64).unwrap_or(&0)) as f64
            / CONVERSION_FACTOR
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binance_payloads;

    #[test]
    fn test_new_order_book() {
        let orderbook = OrderBook::new("BNBUSDT".to_string());
        assert_eq!(orderbook.symbol, "BNBUSDT");
        assert!(orderbook.bids.is_empty());
        assert!(orderbook.asks.is_empty());
        assert_eq!(orderbook.last_update_id, 0);
    }

    #[test]
    fn test_update_book_ticker() {
        let mut orderbook = OrderBook::new("BNBUSDT".to_string());
        let book_ticker_update = binance_payloads::BookTickerUpdate {
            update_id: 400900217,
            symbol: "BNBUSDT".to_string(),
            best_bid_price: 25.3519,
            best_bid_quantity: 31.21,
            best_ask_price: 25.3652,
            best_ask_quantity: 40.66,
        };
        orderbook.update_book_ticker(&book_ticker_update);
        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.asks.len(), 1);
        assert_eq!(*orderbook.bids.get(&253519).unwrap(), 312100);
        assert_eq!(*orderbook.asks.get(&253652).unwrap(), 406600);
    }

    #[test]
    fn test_update_depth() {
        let mut orderbook = OrderBook::new("BNBUSDT".to_string());
        let depth_update = binance_payloads::DepthUpdate {
            last_update_id: 160,
            bids: vec![(0.0024, 10.0), (0.0025, 20.0)],
            asks: vec![(0.0026, 100.0), (0.0027, 200.0)],
        };
        orderbook.update_depth(&depth_update);
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert_eq!(*orderbook.bids.get(&24).unwrap(), 100000);
        assert_eq!(*orderbook.bids.get(&25).unwrap(), 200000);
        assert_eq!(*orderbook.asks.get(&26).unwrap(), 1000000);
        assert_eq!(*orderbook.asks.get(&27).unwrap(), 2000000);
        assert_eq!(orderbook.last_update_id, 160);
    }

    #[test]
    fn test_update_depth_with_older_update_id() {
        let mut orderbook = OrderBook::new("BNBUSDT".to_string());
        orderbook.last_update_id = 200;
        let depth_update = binance_payloads::DepthUpdate {
            last_update_id: 150,
            bids: vec![(0.0024, 10.0)],
            asks: vec![(0.0026, 100.0)],
        };
        orderbook.update_depth(&depth_update);
        assert!(orderbook.bids.is_empty());
        assert!(orderbook.asks.is_empty());
        assert_eq!(orderbook.last_update_id, 200);
    }

    #[test]
    fn test_update_depth_with_zero_quantity() {
        let mut orderbook = OrderBook::new("BNBUSDT".to_string());
        let depth_update = binance_payloads::DepthUpdate {
            last_update_id: 160,
            bids: vec![(0.0024, 10.0), (0.0025, 0.0)],
            asks: vec![(0.0026, 0.0), (0.0027, 200.0)],
        };
        orderbook.update_depth(&depth_update);

        let ((bid_price, _bid_amount), (ask_price, _ask_amount)) =
            orderbook.get_best_bid_ask().unwrap();

        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.asks.len(), 1);
        assert_eq!(bid_price, 0.0024);
        assert_eq!(ask_price, 0.0027);
    }

    #[test]
    fn test_get_best_bid_ask() {
        let mut orderbook = OrderBook::new("BNBUSDT".to_string());
        let depth_update = binance_payloads::DepthUpdate {
            last_update_id: 160,
            bids: vec![(0.0024, 10.0), (0.0025, 20.0)],
            asks: vec![(0.0026, 100.0), (0.0027, 200.0)],
        };
        orderbook.update_depth(&depth_update);
        let best_bid_ask = orderbook.get_best_bid_ask();
        assert_eq!(best_bid_ask, Some(((0.0025, 20.0), (0.0026, 100.0))));
    }

    #[test]
    fn test_get_best_bid_ask_with_empty_book() {
        let orderbook = OrderBook::new("BNBUSDT".to_string());
        let best_bid_ask = orderbook.get_best_bid_ask();
        assert_eq!(best_bid_ask, None);
    }

    #[test]
    fn test_get_volume_at_price() {
        let mut orderbook = OrderBook::new("BNBUSDT".to_string());
        let depth_update = binance_payloads::DepthUpdate {
            last_update_id: 160,
            bids: vec![(0.0024, 10.0), (0.0025, 20.0)],
            asks: vec![(0.0024, 100.0), (0.0027, 200.0)],
        };
        orderbook.update_depth(&depth_update);
        assert_eq!(orderbook.get_volume_at_price(0.0024), 110.0);
        assert_eq!(orderbook.get_volume_at_price(0.0025), 20.0);
        assert_eq!(orderbook.get_volume_at_price(0.0026), 0.0);
        assert_eq!(orderbook.get_volume_at_price(0.0027), 200.0);
        assert_eq!(orderbook.get_volume_at_price(0.0028), 0.0);
    }

    #[test]
    fn test_get_volume_at_price_with_empty_orderbook() {
        let orderbook = OrderBook::new("BNBUSDT".to_string());
        assert_eq!(orderbook.get_volume_at_price(0.0024), 0.0);
    }

    // If you want to see an extra output here:
    // add display feature when running `cargo test`
    #[test]
    fn flow_test() {
        let mut orderbook = OrderBook::new("BNBUSDT".to_string());

        // Update with Book Ticker data
        let book_ticker_update = binance_payloads::BookTickerUpdate {
            update_id: 400900217,
            symbol: "BNBUSDT".to_string(),
            best_bid_price: 25.3519,
            best_bid_quantity: 31.21,
            best_ask_price: 25.3652,
            best_ask_quantity: 40.66,
        };
        orderbook.update_book_ticker(&book_ticker_update);

        // Update with Partial Book Depth data
        let depth_update = binance_payloads::DepthUpdate {
            last_update_id: 160,
            bids: vec![(0.0024, 10.0)],
            asks: vec![(0.0026, 100.0)],
        };
        orderbook.update_depth(&depth_update);

        // Get the best bid and ask prices and quantities
        if let Some(((bid_price, bid_qty), (ask_price, ask_qty))) = orderbook.get_best_bid_ask() {
            println!("Best Bid: Price - {}, Quantity - {}", bid_price, bid_qty);
            println!("Best Ask: Price - {}, Quantity - {}", ask_price, ask_qty);
        }

        // Get the total volume at a given price level
        let price = 0.0024;
        let volume = orderbook.get_volume_at_price(price);
        println!("Volume at price {}: {}", price, volume);
    }
}
