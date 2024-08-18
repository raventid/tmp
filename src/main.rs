use binance_spot_connector_rust::{
    market_stream::partial_depth::PartialDepthStream,
    market_stream::book_ticker::BookTickerStream,
    tokio_tungstenite::BinanceWebSocketClient,
};
use env_logger::Builder;
use futures_util::StreamExt;

mod orderbook;

const INSTRUMENT: &str = "ETHUSDC";
const LEVELS: u16 = 20;

#[tokio::main]
async fn main() {
    Builder::from_default_env().init();

    // Establish connection
    let (mut conn, _) = BinanceWebSocketClient::connect_async_default()
        .await
        .expect("Failed to connect");

    // Subscribe to streams
    conn.subscribe(vec![
        &PartialDepthStream::from_100ms(INSTRUMENT, LEVELS).into(),
        &BookTickerStream::from_symbol(INSTRUMENT).into(),
    ])
    .await;

    // Read messages
    while let Some(message) = conn.as_mut().next().await {
        match message {
            Ok(message) => {
                let binary_data = message.into_data();
                let data = std::str::from_utf8(&binary_data).expect("Failed to parse message");
                log::debug!("{:?}", data);
                handle(data);
            }
            Err(_) => {
                log::error!("Broken message received from the socket, stopping execution");
                break
            },
        }
    }

    // Disconnect
    conn.close().await.expect("Failed to disconnect");
}

fn handle(message: &str) {
    match serde_json::from_str::<orderbook::DepthUpdateEnvelope>(message) {
        Ok(depth_update) => {
            log::debug!("{:?}", depth_update);
        },
        Err(_) =>  {
            match serde_json::from_str::<orderbook::BookTickerUpdateEnvelope>(message) {
                Ok(book_ticker_update) => {
                    log::debug!("{:?}", book_ticker_update);
                },
                Err(_) => log::error!("Unrecognized websocket message"),
            }
        }
    };
}
