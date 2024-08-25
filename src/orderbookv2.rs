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
}