use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Transport types to work with Binance API
#[derive(Debug, Serialize, Deserialize)]
pub struct BookTickerUpdateEnvelope {
    pub stream: String,
    pub data: BookTickerUpdate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookTickerUpdate {
    #[serde(rename = "u")]
    pub update_id: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(
        rename = "b",
        deserialize_with = "deserialize_string_to_f64",
        serialize_with = "serialize_f64_to_string"
    )]
    pub best_bid_price: f64,
    #[serde(
        rename = "B",
        deserialize_with = "deserialize_string_to_f64",
        serialize_with = "serialize_f64_to_string"
    )]
    pub best_bid_quantity: f64,
    #[serde(
        rename = "a",
        deserialize_with = "deserialize_string_to_f64",
        serialize_with = "serialize_f64_to_string"
    )]
    pub best_ask_price: f64,
    #[serde(
        rename = "A",
        deserialize_with = "deserialize_string_to_f64",
        serialize_with = "serialize_f64_to_string"
    )]
    pub best_ask_quantity: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepthUpdateEnvelope {
    pub stream: String,
    pub data: DepthUpdate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepthUpdate {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    #[serde(
        rename = "bids",
        deserialize_with = "deserialize_string_tuple_vec",
        serialize_with = "serialize_tuple_vec_to_string"
    )]
    pub bids: Vec<(f64, f64)>,
    #[serde(
        rename = "asks",
        deserialize_with = "deserialize_string_tuple_vec",
        serialize_with = "serialize_tuple_vec_to_string"
    )]
    pub asks: Vec<(f64, f64)>,
}

fn deserialize_string_tuple_vec<'de, D>(deserializer: D) -> Result<Vec<(f64, f64)>, D::Error>
where
    D: Deserializer<'de>,
{
    let string_tuple_vec: Vec<(String, String)> = Vec::deserialize(deserializer)?;
    string_tuple_vec
        .into_iter()
        .map(|(s1, s2)| {
            let v1 = s1.parse().map_err(serde::de::Error::custom)?;
            let v2 = s2.parse().map_err(serde::de::Error::custom)?;
            Ok((v1, v2))
        })
        .collect()
}

fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn serialize_f64_to_string<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = value.to_string();
    serializer.serialize_str(&s)
}

fn serialize_tuple_vec_to_string<S>(
    value: &Vec<(f64, f64)>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(value.len()))?;
    for (v1, v2) in value {
        let s1 = v1.to_string();
        let s2 = v2.to_string();
        seq.serialize_element(&(s1, s2))?;
    }
    seq.end()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_ticker_update_serde() {
        let update = BookTickerUpdate {
            update_id: 123456789,
            symbol: "BTCUSDT".to_string(),
            best_bid_price: 50000.0,
            best_bid_quantity: 0.5,
            best_ask_price: 50100.0,
            best_ask_quantity: 0.3,
        };

        // Serialize the update to JSON
        let json = match serde_json::to_string(&update) {
            Ok(json) => json,
            Err(_) => return,
        };

        // Deserialize the JSON back into a BookTickerUpdate
        let deserialized_update: BookTickerUpdate = match serde_json::from_str(&json) {
            Ok(deserialized) => deserialized,
            Err(_) => return,
        };

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
        let depth_update = DepthUpdate {
            last_update_id: 987654321,
            bids: vec![(50000.0, 0.5), (49900.0, 1.2)],
            asks: vec![(50100.0, 0.3), (50200.0, 0.8)],
        };

        // Serialize the depth update to JSON
        let json = match serde_json::to_string(&depth_update) {
            Ok(json) => json,
            Err(_) => return,
        };

        // Deserialize the JSON back into a DepthUpdate
        let deserialized_update: DepthUpdate = match serde_json::from_str(&json) {
            Ok(deserialized) => deserialized,
            Err(_) => return,
        };

        // Assert that the deserialized update matches the original update
        assert_eq!(
            depth_update.last_update_id,
            deserialized_update.last_update_id
        );
        assert_eq!(depth_update.bids, deserialized_update.bids);
        assert_eq!(depth_update.asks, deserialized_update.asks);
    }
}
