use std::{collections::HashMap, sync::Arc};

use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::mpsc,
};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};
use tracing::{error, warn};

use crate::{
    connection::Connection,
    error::LiRpcError,
    handler::Handler,
    lirpc_message::{LiRpcMessage, LiRpcResponse},
    service::{HandlerService, Service},
};

#[derive(Default)]
pub struct ServerBuilder<S: Clone> {
    handlers: HashMap<String, Box<dyn Service<S>>>,
}

impl<S> ServerBuilder<S>
where
    S: Send + Sync + Clone + 'static,
{
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<F, T>(
        mut self,
        name: String,
        handler: impl Handler<F, T, S> + 'static,
    ) -> Self
    where
        F: 'static,
        T: 'static,
    {
        self.handlers
            .insert(name, Box::new(HandlerService(Box::new(handler))));
        self
    }

    pub fn build_with_state(self, state: S) -> Server<S> {
        Server {
            state,
            handlers: Arc::new(self.handlers),
        }
    }
}

pub struct Server<S: Clone> {
    state: S,
    handlers: Arc<HashMap<String, Box<dyn Service<S>>>>,
}

impl<S> Server<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(state: S) -> Self {
        Self {
            state,
            handlers: Arc::new(HashMap::new()),
        }
    }

    async fn handle_message(
        handlers: Arc<HashMap<String, Box<dyn Service<S>>>>,
        websocket_message: Message,
        state: S,
        output: mpsc::Sender<LiRpcResponse>,
    ) -> Result<(), LiRpcError> {
        let message = LiRpcMessage::try_from(websocket_message)?;

        let handler = handlers
            .get(&message.headers.method)
            .ok_or(LiRpcError::HandlerNotFound(
                message.headers.method.to_string(),
            ))?;

        handler.call(Connection {}, message, state, output).await?;

        Ok(())
    }

    async fn handle_connection(
        socket: WebSocketStream<TcpStream>,
        state: S,
        handlers: Arc<HashMap<String, Box<dyn Service<S>>>>,
    ) {
        let (mut ws_sender, mut ws_receiver) = socket.split();
        let (tx, mut rx) = mpsc::channel(10);

        loop {
            tokio::select! {
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(message)) => {
                            if message.is_close() || message.is_ping() || message.is_pong() {
                                break;
                            }
                            let handlers_clone = handlers.clone();
                            let tx_clone = tx.clone();
                            let state_clone = state.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_message(handlers_clone, message, state_clone, tx_clone).await {
                                    error!("Error handling message: {e}");
                                }
                            });
                        }
                        Some(Err(e)) => {
                            error!("Error receiving message: {e}");
                            break;
                        }
                        None => break,
                    }
                }

                Some(response) = rx.recv() => {
                    let serialized_response = match response.try_into() {
                        Ok(r) => r,
                        Err(e) => {
                            error!("Error serializing response: {e}");
                            break;
                        }
                    };

                    if let Err(e) = ws_sender.send(serialized_response).await {
                        error!("Error sending response: {e}");
                        break;
                    }
                }
            }
        }
    }

    pub async fn serve<A>(&self, address: A) -> Result<(), LiRpcError>
    where
        A: ToSocketAddrs,
    {
        let server = TcpListener::bind(address).await?;

        while let Ok((stream, _)) = server.accept().await {
            let handlers_clone = self.handlers.clone();
            let state_clone = self.state.clone();

            let accepted_stream = match accept_async(stream).await {
                Ok(s) => s,
                Err(e) => {
                    warn!("establishing connection with client failed: {e}");
                    continue;
                }
            };

            tokio::spawn(async move {
                Self::handle_connection(accepted_stream, state_clone, handlers_clone).await;
            });
        }

        Ok(())
    }
}
