use binance_spot_connector_rust::{
    market_stream::book_ticker::BookTickerStream, market_stream::partial_depth::PartialDepthStream,
    tokio_tungstenite::BinanceWebSocketClient,
};
use env_logger::Builder;
use futures_util::StreamExt;

mod binance_payloads;
mod orderbook;

const INSTRUMENT: &str = "ETHUSDC";
const LEVELS: u16 = 20;

#[tokio::main]
async fn main() {
    Builder::from_default_env().init();

    let mut orderbook = orderbook::OrderBook::new(INSTRUMENT.to_string());

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
                let payload = std::str::from_utf8(&binary_data).expect("Failed to parse message");
                log::debug!("{:?}", payload);

                handle_payload(payload, &mut orderbook);
                log::info!("{:?}", orderbook);
            }
            Err(_) => {
                log::error!("Broken message received from the socket, stopping execution");
                break;
            }
        }
    }

    // Disconnect
    conn.close().await.expect("Failed to disconnect");
}

fn handle_payload(payload: &str, orderbook: &mut orderbook::OrderBook) {
    match serde_json::from_str::<binance_payloads::DepthUpdateEnvelope>(payload) {
        Ok(depth_update) => {
            log::debug!("{:?}", depth_update);
            orderbook.update_depth(&depth_update.data);
        }
        Err(_) => match serde_json::from_str::<binance_payloads::BookTickerUpdateEnvelope>(payload)
        {
            Ok(book_ticker_update) => {
                log::debug!("{:?}", book_ticker_update);
                orderbook.update_book_ticker(&book_ticker_update.data);
            }
            Err(_) => log::error!("Unrecognized websocket message"),
        },
    };
}
