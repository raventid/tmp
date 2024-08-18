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
