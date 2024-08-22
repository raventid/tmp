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
}