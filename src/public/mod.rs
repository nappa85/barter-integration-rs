use std::pin::Pin;
use std::task::{Context, Poll};
use crate::{
    socket::{
        ExchangeSocket, Transformer,
        protocol::websocket::{WebSocket, WebSocketParser, WsMessage},
        error::SocketError
    },
    public::model::MarketData
};
use async_trait::async_trait;
use futures::{Sink, SinkExt, Stream, StreamExt};
use serde::de::DeserializeOwned;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use crate::public::model::Subscription;
use crate::socket::protocol::websocket::connect;

/// Todo:
pub mod model;
pub mod binance;

/// Todo:
#[async_trait]
pub trait MarketEventStream<OutputIter>: Stream<Item = Result<OutputIter, SocketError>> + Sized + Unpin
where
    OutputIter: IntoIterator<Item = MarketData>,
{
    async fn init(subscriptions: &[String]) -> Result<Self, SocketError>;
}

/// Todo:
pub trait Exchange<ExMessage>
where
    Self: Transformer<ExMessage, MarketData>,
    ExMessage: DeserializeOwned,
{
    const EXCHANGE: &'static str;
    const BASE_URL: &'static str;
    fn new() -> Self;
    fn generate_subscriptions(&mut self, subscriptions: &[Subscription]) -> Vec<serde_json::Value>;
}

#[async_trait]
impl<ExTransformer, ExMessage, OutputIter> MarketEventStream<OutputIter>
    for ExchangeSocket<WebSocket, WsMessage, WebSocketParser, ExTransformer, ExMessage, MarketData>
where
    Self: Stream<Item = Result<OutputIter, SocketError>> + Sized + Unpin,
    ExTransformer: Exchange<ExMessage>,
    ExMessage: DeserializeOwned,
    OutputIter: IntoIterator<Item = MarketData>,
{
    async fn init(subscriptions: &[Subscription]) -> Result<Self, SocketError> {
        // Connect to exchange WebSocket server
        let mut websocket = connect(ExTransformer::BASE_URL).await?;

        // Construct Exchange capable of translating
        let mut exchange = ExTransformer::new();


        // Action Subscriptions over the socket
        for sub_payload in exchange.generate_subscriptions(subscriptions) {
            websocket
                .send(WsMessage::Text(sub_payload.to_string()))
                .await?;
        }

        Ok(Self {
            socket: websocket,
            parser: WebSocketParser,
            transformer: exchange,
            socket_item_marker: Default::default(),
            exchange_message_marker: Default::default(),
            output_marker: Default::default()
        })
    }
}



