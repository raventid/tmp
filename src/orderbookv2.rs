use std::{
    cell::RefCell,
    collections::{btree_map, HashMap, VecDeque},
    rc::Rc,
};

#[derive(Debug, PartialEq, Eq)]
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

    fn is_filled(&self) -> bool {
        self.remaining_quantity == 0
    }
}

type OrderPointer = Rc<RefCell<Order>>;
type OrderList = VecDeque<OrderPointer>;

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
    orders: HashMap<Price, OrderPointer>,
}

impl OrderBook {
    fn new() -> OrderBook {
        OrderBook {
            bids: btree_map::BTreeMap::new(),
            asks: btree_map::BTreeMap::new(),
            orders: HashMap::new(),
        }
    }

    fn cancel_order(&self, order_id: OrderId) {
        // TODO: cancel the order logic
    }

    fn can_match(&self, price: Price, side: Side) -> bool {
        match side {
            Side::Buy => {
                if self.asks.is_empty() {
                    return false;
                }

                let best_ask = self
                    .asks
                    .iter()
                    .next()
                    .expect("No ask found | unreachable state");
                price >= *best_ask.0
            }
            Side::Sell => {
                if self.bids.is_empty() {
                    return false;
                }

                let best_bid = self
                    .bids
                    .iter()
                    .next()
                    .expect("No bid found | unreachable state");
                price <= best_bid.0 .0
            }
        }
    }

    fn match_orders(&mut self) -> Vec<Trade> {
        let mut trades = Vec::new();

        loop {
            if self.bids.is_empty() || self.asks.is_empty() {
                break;
            }

            let (bids_level_to_remove, asks_level_to_remove) = {
                let bids = self
                    .bids
                    .iter_mut()
                    .next()
                    .expect("No bid found | unreachable state");
                let asks = self
                    .asks
                    .iter_mut()
                    .next()
                    .expect("No ask found | unreachable state");

                // Nothing to match in orderbook
                if bids.0 .0 < *asks.0 {
                    break;
                }

                // internal loop to match orders, will be stopped when bids or asks are empty
                while !bids.1.is_empty() && !asks.1.is_empty() {
                    let (bid_is_filled, ask_is_filled, quantity) = {
                        let mut bid = bids.1.front().unwrap().borrow_mut();
                        let mut ask = asks.1.front().unwrap().borrow_mut();
                        let quantity =
                            std::cmp::min(bid.remaining_quantity, ask.remaining_quantity);

                        bid.fill(quantity);
                        ask.fill(quantity);

                        (bid.is_filled(), ask.is_filled(), quantity)
                    };

                    if bid_is_filled {
                        bids.1.pop_front();
                        self.orders.remove(&bids.0 .0);
                    }

                    if ask_is_filled {
                        asks.1.pop_front();
                        self.orders.remove(&asks.0);
                    }

                    trades.push(Trade {
                        bid_trade: TradeInfo {
                            order_id: bids.1.front().unwrap().borrow().order_id,
                            price: bids.0 .0,
                            quantity,
                        },
                        ask_trade: TradeInfo {
                            order_id: asks.1.front().unwrap().borrow().order_id,
                            price: *asks.0,
                            quantity,
                        },
                    });
                }

                // remove the level if it is empty
                let bids_level_to_remove = if bids.1.is_empty() {
                    Some(bids.0 .0)
                } else {
                    None
                };

                let asks_level_to_remove = if asks.1.is_empty() {
                    Some(*asks.0)
                } else {
                    None
                };

                (bids_level_to_remove, asks_level_to_remove)
            };

            if let Some(price) = bids_level_to_remove {
                self.bids.remove(&std::cmp::Reverse(price));
            }

            if let Some(price) = asks_level_to_remove {
                self.asks.remove(&price);
            }

            if !self.bids.is_empty() {
                let need_cancelation = {
                    let (_, bids) = self.bids.iter_mut().next().unwrap();
                    let first_order = bids.front().unwrap().borrow();
                    if first_order.order_type == OrderType::FillAndKill {
                        Some(first_order.order_id)
                    } else {
                        None
                    }
                };

                if let Some(order_id) = need_cancelation {
                    self.cancel_order(order_id);
                }
            }

            if !self.asks.is_empty() {
                let need_cancelation = {
                    let (_, asks) = self.asks.iter_mut().next().unwrap();
                    let first_order = asks.front().unwrap().borrow();
                    if first_order.order_type == OrderType::FillAndKill {
                        Some(first_order.order_id)
                    } else {
                        None
                    }
                };

                if let Some(order_id) = need_cancelation {
                    self.cancel_order(order_id);
                }
            }
        }

        // just a dummy implementation to make the code compile
        vec![]
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
        orderlist.push_back(Rc::new(RefCell::new(Order::new(
            1,
            10,
            100,
            OrderType::GoodToCancel,
            Side::Buy,
        ))));
        orderlist.push_back(Rc::new(RefCell::new(Order::new(
            2,
            20,
            200,
            OrderType::GoodToCancel,
            Side::Buy,
        ))));

        assert_eq!(orderlist.len(), 2);
    }

    #[test]
    fn test_can_match() {
        let mut orderbook = OrderBook::new();

        orderbook
            .bids
            .insert(std::cmp::Reverse(10), OrderList::new());
        orderbook.asks.insert(20, OrderList::new());

        assert_eq!(orderbook.can_match(10, Side::Buy), false);
        assert_eq!(orderbook.can_match(20, Side::Buy), true);
        assert_eq!(orderbook.can_match(10, Side::Sell), true);
        assert_eq!(orderbook.can_match(20, Side::Sell), false);
    }
}
