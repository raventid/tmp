use crate::binance_payloads;
use std::collections::BTreeMap;

// Binance orderbook implementation
type Price = u64;
type Quantity = u64;

trait ToU64 {
    fn to_u64(self) -> u64;
}

impl ToU64 for f64 {
    #[inline]
    fn to_u64(self) -> u64 {
        (self * 10000.0).round() as u64
    }
}

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
        if self.bids.is_empty() || self.asks.is_empty() {
            None
        } else {
            let best_bid = self
                .bids
                .iter()
                .next_back()
                .expect("bids should not be empty");
            let best_ask = self.asks.iter().next().expect("asks should not be empty");

            Some((
                (*best_bid.0 as f64 / 10000.0, *best_bid.1 as f64 / 10000.0),
                (*best_ask.0 as f64 / 10000.0, *best_ask.1 as f64 / 10000.0),
            ))
        }
    }

    #[allow(dead_code)]
    fn get_volume_at_price(&self, price: f64) -> f64 {
        let price_u64 = price.to_u64() as Price;
        (self.bids.get(&price_u64).unwrap_or(&0) + self.asks.get(&price_u64).unwrap_or(&0)) as f64
            / 10000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binance_payloads;

    #[test]
    fn test_book_ticker_update_serde() {
        let update = binance_payloads::BookTickerUpdate {
            update_id: 123456789,
            symbol: "BTCUSDT".to_string(),
            best_bid_price: 50000.0,
            best_bid_quantity: 0.5,
            best_ask_price: 50100.0,
            best_ask_quantity: 0.3,
        };

        // Serialize the update to JSON
        let json = serde_json::to_string(&update).unwrap();

        // Deserialize the JSON back into a BookTickerUpdate
        let deserialized_update: binance_payloads::BookTickerUpdate =
            serde_json::from_str(&json).unwrap();

        // Assert that the deserialized update matches the original update
        assert_eq!(update.update_id, deserialized_update.update_id);
        assert_eq!(update.symbol, deserialized_update.symbol);
        assert_eq!(update.best_bid_price, deserialized_update.best_bid_price);
        assert_eq!(
            update.best_bid_quantity,
            deserialized_update.best_bid_quantity
        );
        assert_eq!(update.best_ask_price, deserialized_update.best_ask_price);
        assert_eq!(
            update.best_ask_quantity,
            deserialized_update.best_ask_quantity
        );
    }

    #[test]
    fn test_depth_update_serde() {
        let depth_update = binance_payloads::DepthUpdate {
            last_update_id: 987654321,
            bids: vec![(50000.0, 0.5), (49900.0, 1.2)],
            asks: vec![(50100.0, 0.3), (50200.0, 0.8)],
        };

        // Serialize the depth update to JSON
        let json = serde_json::to_string(&depth_update).unwrap();

        // Deserialize the JSON back into a DepthUpdate
        let deserialized_update: binance_payloads::DepthUpdate =
            serde_json::from_str(&json).unwrap();

        // Assert that the deserialized update matches the original update
        assert_eq!(
            depth_update.last_update_id,
            deserialized_update.last_update_id
        );
        assert_eq!(depth_update.bids, deserialized_update.bids);
        assert_eq!(depth_update.asks, deserialized_update.asks);
    }

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
