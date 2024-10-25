use std::{collections::{btree_map, HashMap, VecDeque}, rc::Rc};

enum OrderType {
    GoodToCancel,
    FillAndKill,
}

enum Side {
    Buy,
    Sell,
}

type Price = i32;
type Quantity = u32;
type OrderId = u64;

struct LevelInfo {
    price: Price,
    quantity: Quantity,
}

struct OrderBookLevelInfos {
    bids: Vec<LevelInfo>,
    asks: Vec<LevelInfo>,
}

impl OrderBookLevelInfos {
    fn new(bids: Vec<LevelInfo>, asks: Vec<LevelInfo>) -> OrderBookLevelInfos {
        OrderBookLevelInfos { bids, asks }
    }

    fn from_existing() -> OrderBookLevelInfos {
        OrderBookLevelInfos {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    fn get_bids(&self) -> &Vec<LevelInfo> {
        &self.bids
    }

    fn get_asks(&self) -> &Vec<LevelInfo> {
        &self.asks
    }
}

struct Order {
    order_id: OrderId,
    price: Price,
    remaining_quantity: Quantity,
    initial_quantity: Quantity,
    order_type: OrderType,
    side: Side,
}

impl Order {
    fn new(
        order_id: OrderId,
        price: Price,
        quantity: Quantity,
        order_type: OrderType,
        side: Side,
    ) -> Order {
        Order {
            order_id,
            price,
            remaining_quantity: quantity,
            initial_quantity: quantity,
            order_type,
            side,
        }
    }

    fn get_fill_quantity(&self) -> Quantity {
        self.initial_quantity - self.remaining_quantity
    }

    fn fill(&mut self, quantity: Quantity) {
        if quantity > self.remaining_quantity {
            panic!("Cannot fill more than the order quantity");
        }

        self.remaining_quantity -= quantity;
    }
}

type OrderList = VecDeque<Rc<Order>>;

struct OrderModify {
    order_id: OrderId,
    side: Side,
    price: Price,
    quantity: Quantity,
}

impl OrderModify {
    fn new(order_id: OrderId, side: Side, price: Price, quantity: Quantity) -> OrderModify {
        OrderModify {
            order_id,
            side,
            price,
            quantity,
        }
    }
}

struct TradeInfo {
    order_id: OrderId,
    price: Price,
    quantity: Quantity,
}

struct Trade {
    bid_trade: TradeInfo,
    ask_trade: TradeInfo,
}


struct OrderBook {
    bids: btree_map::BTreeMap<std::cmp::Reverse<Price>, OrderList>,
    asks: btree_map::BTreeMap<Price, OrderList>,
    orders: HashMap<Price, Rc<Order>>
}

impl OrderBook {
    fn new() -> OrderBook {
        OrderBook {
            bids: btree_map::BTreeMap::new(),
            asks: btree_map::BTreeMap::new(),
            orders: HashMap::new()
        }
    }
    
    fn can_match(&self, price: Price, side: Side) -> bool {
        match side {
            Side::Buy => { 
                if self.asks.is_empty() {
                    return false;
                }

                let best_ask = self.asks.iter().next().expect("No ask found | unreachable state");
                price >= *best_ask.0
            }
            Side::Sell => {
                if self.bids.is_empty() {
                    return false;
                }

                let best_bid = self.bids.iter().next().expect("No bid found | unreachable state");
                price <= best_bid.0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook() {
        let Price = 10;

        assert_eq!(Price, 10);
    }

    #[test]
    fn test_orderbooklevelinfos() {
        let orderbooklevelinfos = OrderBookLevelInfos::from_existing();

        assert_eq!(orderbooklevelinfos.bids.len(), 0);
        assert_eq!(orderbooklevelinfos.asks.len(), 0);
    }

    #[test]
    fn test_filling_an_order() {
        let initial_quantity = 100;
        let mut order = Order::new(1, 10, initial_quantity, OrderType::GoodToCancel, Side::Buy);

        order.fill(50);

        assert_eq!(order.get_fill_quantity(), 50);
    }

    #[test]
    fn test_orderlist_creation() {     
        let mut orderlist = OrderList::new();
        orderlist.push_back(Rc::new(Order::new(1, 10, 100, OrderType::GoodToCancel, Side::Buy)));
        orderlist.push_back(Rc::new(Order::new(2, 20, 200, OrderType::GoodToCancel, Side::Buy)));

        assert_eq!(orderlist.len(), 2);
    }

    #[test]
    fn test_can_match() {
        let mut orderbook = OrderBook::new();

        orderbook.bids.insert(std::cmp::Reverse(10), OrderList::new());
        orderbook.asks.insert(20, OrderList::new());

        assert_eq!(orderbook.can_match(10, Side::Buy), false);
        assert_eq!(orderbook.can_match(20, Side::Buy), true);
        assert_eq!(orderbook.can_match(10, Side::Sell), true);
        assert_eq!(orderbook.can_match(20, Side::Sell), false);
    }
}