// use binance_spot_connector_rust::{
//     market_stream::diff_depth::DiffDepthStream,
//     tungstenite::BinanceWebSocketClient,
// };

const BINANCE_WSS_BASE_URL: &str = "wss://stream.binance.com:9443/ws";
const INSTRUMENT: &str = "BTCUSDT";

// fn main() {
//     // Establish connection
//     let mut conn =
//         BinanceWebSocketClient::connect_with_url(BINANCE_WSS_BASE_URL).expect("Failed to connect");

//     // Subscribe to streams
//     conn.subscribe(vec![&DiffDepthStream::from_100ms(INSTRUMENT).into()]);

//     // Read messages
//     while let Ok(message) = conn.as_mut().read() {
//         let data = message.into_data();
//         let string_data = String::from_utf8(data).expect("Found invalid UTF-8 chars");
//         dbg!(string_data);
//         log::info!("{}", &string_data);
//     }

//     // Disconnect
//     conn.close().expect("Failed to disconnect");
// }












use tokio;
use binance_spot_connector_rust::{
    market::klines::KlineInterval, market_stream::kline::KlineStream,
    tokio_tungstenite::BinanceWebSocketClient,
    market_stream::diff_depth::DiffDepthStream,
};
use env_logger::Builder;
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();

    // Establish connection
    let (mut conn, _) = BinanceWebSocketClient::connect_async_default()
        .await
        .expect("Failed to connect");

    // Subscribe to streams
    conn.subscribe(vec![
        &DiffDepthStream::from_100ms(INSTRUMENT).into(),
        &KlineStream::new("BTCUSDT", KlineInterval::Minutes1).into()
    ])
    .await;

    // Read messages
    while let Some(message) = conn.as_mut().next().await {
        match message {
            Ok(message) => {
                let binary_data = message.into_data();
                let data = std::str::from_utf8(&binary_data).expect("Failed to parse message");
                log::info!("{:?}", data);
            }
            Err(_) => break,
        }
    }

    // Disconnect
    conn.close().await.expect("Failed to disconnect");
}
