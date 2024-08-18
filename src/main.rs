use std::collections::BTreeMap;

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


struct OrderBook {
    symbol: String,
    bids: BTreeMap<Price, Quantity>,
    asks: BTreeMap<Price, Quantity>,
    last_update_id: u64,
}

struct BookTickerUpdate {
    u: u64,
    s: String,
    b: f64,
    B: f64,
    a: f64,
    A: f64,
}

struct DepthUpdate {
    last_update_id: u64,
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
}

impl OrderBook {
    fn new(symbol: String) -> OrderBook {
        OrderBook {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_id: 0,
        }
    }

    fn update_book_ticker(&mut self, data: &BookTickerUpdate) {
        self.bids
            .insert(data.b.to_u64() as Price, data.B.to_u64() as Quantity);
        self.asks
            .insert(data.a.to_u64() as Price, data.A.to_u64() as Quantity);
    }

    fn update_depth(&mut self, data: &DepthUpdate) {
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
    fn get_best_bid_ask(&self) -> Option<((f64, f64), (f64, f64))> {
        if self.bids.is_empty() || self.asks.is_empty() {
            None
        } else {
            let best_bid = self.bids.iter().next_back().unwrap();
            let best_ask = self.asks.iter().next().unwrap();

            Some((
                (*best_bid.0 as f64 / 10000.0, *best_bid.1 as f64 / 10000.0),
                (*best_ask.0 as f64 / 10000.0, *best_ask.1 as f64 / 10000.0),
            ))
        }
    }

    fn get_volume_at_price(&self, price: f64) -> f64 {
        let price_u64 = price.to_u64() as Price;
        (self.bids.get(&price_u64).unwrap_or(&0) + self.asks.get(&price_u64).unwrap_or(&0)) as f64
            / 10000.0
    }
}

fn main() {
    let mut orderbook = OrderBook::new("BNBUSDT".to_string());

    // Update with Book Ticker data
    let book_ticker_update = BookTickerUpdate {
        u: 400900217,
        s: "BNBUSDT".to_string(),
        b: 25.3519,
        B: 31.21,
        a: 25.3652,
        A: 40.66,
    };
    orderbook.update_book_ticker(&book_ticker_update);

    // Update with Partial Book Depth data
    let depth_update = DepthUpdate {
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let book_ticker_update = BookTickerUpdate {
            u: 400900217,
            s: "BNBUSDT".to_string(),
            b: 25.3519,
            B: 31.21,
            a: 25.3652,
            A: 40.66,
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
        let depth_update = DepthUpdate {
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
        let depth_update = DepthUpdate {
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
        let depth_update = DepthUpdate {
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
        let depth_update = DepthUpdate {
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
        let depth_update = DepthUpdate {
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
}
