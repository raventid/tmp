use std::{
    cell::RefCell,
    collections::{btree_map, HashMap, VecDeque},
    rc::Rc,
};

// FOK type of order
// https://en.wikipedia.org/wiki/Fill_or_kill

// List of possible orders
// https://www.cmegroup.com/tools-information/webhelp/autocert-ilink-2x/Content/Definitions.html

// Fill and Kill (FAK) Order - FAK orders are immediately executed against resting orders. Any quantity that remains unfilled is cancelled.
// Fill or Kill (FOK) Order - FOK orders are cancelled if not immediately filled for the total quantity at the specified price or better.
// Give Up - An order to be given to another member firm in the clearing system, an allocation. An order executed by clearing firm A and given to clearing firm B where it will be cleared and processed. Give up order indicator is "GU" populated in the F-Ex field.
// Good Till Cancel (GTC) Order - GTC orders remain open until they are completely executed or cancelled.
// Good till Date (GTD) Order - GTD orders expire either at a specified date or when the security expires.

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum OrderType {
    GoodToCancel,
    FillAndKill,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, Clone)]
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
    orders: HashMap<OrderId, OrderPointer>,
}

impl OrderBook {
    fn new() -> OrderBook {
        OrderBook {
            bids: btree_map::BTreeMap::new(),
            asks: btree_map::BTreeMap::new(),
            orders: HashMap::new(),
        }
    }

    fn cancel_order(&mut self, order_id: OrderId) {
        // FIXME: This is very error prone impelmentation,
        // we should not do this conversion here and we should not panic!
        if !self.orders.contains_key(&order_id) {
            panic!("Order not found");
        }

        // Find the order first
        let order_price = self
            .orders
            .iter()
            .find(|(_, order)| order.borrow().order_id == order_id)
            .map(|(_, order)| order.borrow().price);

        if let Some(price) = order_price {
            let order_pointer = self.orders.remove(&order_id).unwrap();
            let order = order_pointer.borrow();

            match order.side {
                Side::Sell => {
                    if let Some(orders) = self.asks.get_mut(&price) {
                        orders.retain(|o| o.borrow().order_id != order_id);
                        // Remove the price level if no orders left
                        if orders.is_empty() {
                            self.asks.remove(&price);
                        }
                    }
                }
                Side::Buy => {
                    let reverse_price = std::cmp::Reverse(price);
                    if let Some(orders) = self.bids.get_mut(&reverse_price) {
                        orders.retain(|o| o.borrow().order_id != order_id);
                        // Remove the price level if no orders left
                        if orders.is_empty() {
                            self.bids.remove(&reverse_price);
                        }
                    }
                }
            }
        }
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

    fn match_order(&mut self, order_modify: OrderModify) -> Vec<Trades> {
        if (!self.orders.contains_key(&order_modify.order_id)) {
            return vec![];
        }

        let order = self.orders.get(&order_modify.order_id).unwrap();
        self.cancel_order(order.borrow().order_id);
        self.add_order(order.borrow().clone())
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
                    let ((bid_is_filled, bid_order_id), (ask_is_filled, ask_order_id), quantity) = {
                        let mut bid = bids.1.front().unwrap().borrow_mut();
                        let mut ask = asks.1.front().unwrap().borrow_mut();
                        let quantity =
                            std::cmp::min(bid.remaining_quantity, ask.remaining_quantity);

                        bid.fill(quantity);
                        ask.fill(quantity);

                        (
                            (bid.is_filled(), bid.order_id),
                            (ask.is_filled(), ask.order_id),
                            quantity,
                        )
                    };

                    if bid_is_filled {
                        bids.1.pop_front();
                        self.orders.remove(&bid_order_id);
                    }

                    if ask_is_filled {
                        asks.1.pop_front();
                        self.orders.remove(&ask_order_id);
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
        return trades;
    }

    pub fn add_order(&mut self, order: Order) -> Vec<Trade> {
        if self.orders.contains_key(&order.order_id) {
            // this is too much, but as an initial implementation, we can just panic
            println!("Order already exists");
            return vec![];
        }

        if order.order_type == OrderType::FillAndKill {
            if !self.can_match(order.price, order.side) {
                println!("Cannot match this Fill and Kill order");
                return vec![];
            }
        }

        let side = order.side;
        let price = order.price;
        let order_pointer = Rc::new(RefCell::new(order.clone()));

        match side {
            Side::Buy => {
                self.bids
                    .entry(std::cmp::Reverse(price))
                    .or_insert(OrderList::new())
                    .push_back(Rc::clone(&order_pointer));
            }
            Side::Sell => {
                self.asks
                    .entry(price)
                    .or_insert(OrderList::new())
                    .push_back(Rc::clone(&order_pointer));
            }
        }

        self.orders.insert(order.order_id, order_pointer);

        self.match_orders()
    }

    pub fn orderbook_size(&self) -> usize {
        self.orders.len()
    }

    pub fn get_orderbook_level_infos(&self) -> OrderBookLevelInfos {
        let bids = self
            .bids
            .iter()
            .map(|(price, orders)| LevelInfo {
                price: price.0,
                quantity: orders.iter().map(|o| o.borrow().remaining_quantity).sum(),
            })
            .collect();

        let asks = self
            .asks
            .iter()
            .map(|(price, orders)| LevelInfo {
                price: *price,
                quantity: orders.iter().map(|o| o.borrow().remaining_quantity).sum(),
            })
            .collect();

        OrderBookLevelInfos::new(bids, asks)
    }

    pub fn get_best_bid_ask(&self) -> Option<(Price, Price)> {
        let best_bid = self.bids.iter().next().map(|(price, _)| price.0);
        let best_ask = self.asks.iter().next().map(|(price, _)| *price);

        match (best_bid, best_ask) {
            (Some(best_bid), Some(best_ask)) => Some((best_bid, best_ask)),
            _ => None,
        }
    }

    // TODO: Not sure if we should only count bids here (maybe we should count asks too?)
    pub fn get_volume_at_price(&self, price: Price) -> Quantity {
        let bids = self.bids.get(&std::cmp::Reverse(price)).unwrap();
        bids.iter().fold(0, |total_quantity, bid| bid.borrow().remaining_quantity + total_quantity)
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

    #[test]
    fn test_add_order_to_orderbook() {
        let mut orderbook = OrderBook::new();
        let order = Order::new(1, 10, 100, OrderType::GoodToCancel, Side::Buy);

        orderbook.add_order(order);

        assert_eq!(orderbook.orders.len(), 1);
    }

    #[test]
    fn test_cancel_order() {
        let mut orderbook = OrderBook::new();
        let order = Order::new(1, 10, 100, OrderType::GoodToCancel, Side::Buy);

        orderbook.add_order(order);
        orderbook.cancel_order(1);

        assert_eq!(orderbook.orders.len(), 0);
    }
}
