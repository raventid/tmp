use binance_spot_connector_rust::{
    tokio_tungstenite::BinanceWebSocketClient,
    market_stream::diff_depth::DiffDepthStream,
    market_stream::ticker::TickerStream,
};
use env_logger::Builder;
use futures_util::StreamExt;

const INSTRUMENT: &str = "BTCUSDT";

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
        &TickerStream::from_symbol(INSTRUMENT).into(),
    ])
    .await;

    // Read messages
    while let Some(message) = conn.as_mut().next().await {
        match message {
            Ok(message) => {
                let binary_data = message.into_data();
                let data = std::str::from_utf8(&binary_data).expect("Failed to parse message");
                log::debug!("{:?}", data);
            }
            Err(_) => break,
        }
    }

    // Disconnect
    conn.close().await.expect("Failed to disconnect");
}
