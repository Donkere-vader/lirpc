use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::mpsc,
};
use tokio_tungstenite::accept_async;
use tokio_util::codec::LengthDelimitedCodec;
use tracing::{debug, error, warn};

/// Maximum size (in bytes) of a single length-prefixed TCP frame, guarding
/// against a malformed/oversized length prefix causing an unbounded allocation.
const MAX_TCP_FRAME_LENGTH: usize = 8 * 1024 * 1024;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    handler::Handler,
    lirpc_message::{LiRpcRequest, LiRpcResponse},
    service::{HandlerService, Service},
    translatable::Translatable,
};

#[derive(Default)]
pub struct ServerBuilder<S: Clone, C> {
    handlers: HashMap<String, Box<dyn Service<S, C>>>,
}

impl<S, C> ServerBuilder<S, C>
where
    S: Send + Sync + Clone + 'static,
    C: Clone,
{
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a method/handler to this service
    /// The `name` given to the handler here is what a client
    /// will use to end up calling this handler/method
    pub fn register_handler<F, T, R>(
        mut self,
        name: String,
        handler: impl Handler<F, T, S, C, R> + 'static,
    ) -> Self
    where
        F: 'static,
        T: 'static,
        R: Translatable + 'static,
        C: Default + Send + Sync + 'static,
    {
        self.handlers
            .insert(name, Box::new(HandlerService(Box::new(handler))));
        self
    }

    pub fn build_with_state_and_connection_state(
        self,
        state: S,
        default_connection_state: impl Fn() -> C + 'static,
    ) -> Server<S, C> {
        Server {
            state,
            handlers: Arc::new(self.handlers),
            connection_state_initializer: Box::new(default_connection_state),
        }
    }
}

impl<S> ServerBuilder<S, ()>
where
    S: Clone,
{
    pub fn build_with_state(self, state: S) -> Server<S, ()> {
        Server {
            state,
            handlers: Arc::new(self.handlers),
            connection_state_initializer: Box::new(|| ()),
        }
    }
}

impl ServerBuilder<(), ()> {
    pub fn build(self) -> Server<(), ()> {
        Server {
            state: (),
            handlers: Arc::new(self.handlers),
            connection_state_initializer: Box::new(|| ()),
        }
    }
}

impl<C> ServerBuilder<(), C> {
    pub fn build_with_connection_state(
        self,
        default_connection_state: impl Fn() -> C + 'static,
    ) -> Server<(), C> {
        Server {
            state: (),
            handlers: Arc::new(self.handlers),
            connection_state_initializer: Box::new(default_connection_state),
        }
    }
}

pub struct Server<S: Clone, C> {
    state: S,
    handlers: Arc<HashMap<String, Box<dyn Service<S, C>>>>,
    connection_state_initializer: Box<dyn Fn() -> C>,
}

impl<S, C> Server<S, C>
where
    S: Clone + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
{
    async fn handle_message(
        handlers: Arc<HashMap<String, Box<dyn Service<S, C>>>>,
        message: LiRpcRequest,
        state: S,
        connection: Arc<ConnectionDetails<C>>,
        output: mpsc::Sender<LiRpcResponse>,
    ) -> Result<(), LiRpcError> {
        debug!("Received message: {message:?}");

        let handler =
            handlers
                .get(&message.headers.function)
                .ok_or(LiRpcError::HandlerNotFound(
                    message.headers.function.to_string(),
                ))?;

        let message_id = message.headers.id;

        let response = handler.call(connection, message, state).await;

        if let Err(e) = output.send(response).await {
            error!("Error sending response for message ({message_id}): {e}");
        };

        Ok(())
    }

    async fn handle_tcp_connection(
        stream: TcpStream,
        state: S,
        new_connection_details: ConnectionDetails<C>,
        handlers: Arc<HashMap<String, Box<dyn Service<S, C>>>>,
    ) {
        let framed = LengthDelimitedCodec::builder()
            .max_frame_length(MAX_TCP_FRAME_LENGTH)
            .new_framed(stream);

        let (mut frame_sender, mut frame_receiver) = framed.split();
        let (tx, mut rx) = mpsc::channel(10);

        let connection_details = Arc::new(new_connection_details);

        loop {
            tokio::select! {
                frame = frame_receiver.next() => {
                    match frame {
                        Some(Ok(bytes)) => {
                            let handlers_clone = handlers.clone();
                            let tx_clone = tx.clone();
                            let state_clone = state.clone();
                            let connection_clone = connection_details.clone();

                            tokio::spawn(async move {
                                let message = match serde_json::from_slice::<LiRpcRequest>(&bytes) {
                                    Ok(m) => m,
                                    Err(e) => {
                                        error!("Error deserializing message: {e}");
                                        return;
                                    }
                                };

                                if let Err(e) = Self::handle_message(handlers_clone, message, state_clone, connection_clone, tx_clone).await {
                                    match e {
                                        LiRpcError::OutputStreamClosed => {},
                                        _ => error!("Error during handling of message: {e}"),
                                    };
                                }
                            });
                        }
                        Some(Err(e)) => {
                            debug!("Error receiving TCP frame: {e}");
                            break;
                        }
                        None => break,
                    }
                }

                Some(response) = rx.recv() => {
                    let serialized_response = match serde_json::to_vec(&response) {
                        Ok(bytes) => Bytes::from(bytes),
                        Err(e) => {
                            error!("Error serializing response: {e}");
                            break;
                        }
                    };

                    if let Err(e) = frame_sender.send(serialized_response).await {
                        error!("Error sending TCP response: {e}");
                        break;
                    }
                }
            }
        }
    }

    async fn handle_ws_connection(
        stream: TcpStream,
        state: S,
        new_connection_details: ConnectionDetails<C>,
        handlers: Arc<HashMap<String, Box<dyn Service<S, C>>>>,
    ) {
        let socket = match accept_async(stream).await {
            Ok(s) => s,
            Err(e) => {
                warn!("establishing ws connection with client failed: {e}");
                return;
            }
        };

        let (mut ws_sender, mut ws_receiver) = socket.split();
        let (tx, mut rx) = mpsc::channel(10);

        let connection_details = Arc::new(new_connection_details);

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
                            let connection_clone = connection_details.clone();

                            tokio::spawn(async move {
                                let message = match LiRpcRequest::try_from(message) {
                                    Ok(m) => m,
                                    Err(e) => {
                                        error!("Error deserializing message: {e}");
                                        return;
                                    }
                                };

                                if let Err(e) = Self::handle_message(handlers_clone, message, state_clone, connection_clone, tx_clone).await {
                                    match e {
                                        LiRpcError::OutputStreamClosed => {},
                                        _ => error!("Error during handling of message: {e}"),
                                    };
                                }
                            });
                        }
                        Some(Err(e)) => {
                            debug!("Error receiving message: {e}");
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

    async fn classify_connection(stream: &TcpStream) -> ConnectionKind {
        let mut buf = [0u8; 1024];
        let n = match stream.peek(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                warn!(
                    "Failed to classify connection type. Falling back to TCP. Original error: {e}"
                );
                return ConnectionKind::Tcp;
            }
        };

        let data = &buf[..n];

        if Self::is_websocket_upgrade(data) {
            ConnectionKind::WebSocket
        } else {
            ConnectionKind::Tcp
        }
    }

    fn is_websocket_upgrade(data: &[u8]) -> bool {
        let Ok(text) = std::str::from_utf8(data) else {
            return false;
        };

        text.to_ascii_lowercase().contains("upgrade: websocket")
    }

    pub async fn serve<A>(&self, address: A) -> Result<(), LiRpcError>
    where
        A: ToSocketAddrs,
    {
        let server = TcpListener::bind(address).await?;

        while let Ok((stream, _)) = server.accept().await {
            let handlers_clone = self.handlers.clone();
            let state_clone = self.state.clone();
            let new_connection_details =
                ConnectionDetails::new((*self.connection_state_initializer)());

            match Self::classify_connection(&stream).await {
                ConnectionKind::Tcp => {
                    tokio::spawn(async move {
                        Self::handle_tcp_connection(
                            stream,
                            state_clone,
                            new_connection_details,
                            handlers_clone,
                        )
                        .await;
                    });
                }
                ConnectionKind::WebSocket => {
                    tokio::spawn(async move {
                        Self::handle_ws_connection(
                            stream,
                            state_clone,
                            new_connection_details,
                            handlers_clone,
                        )
                        .await;
                    });
                }
            }
        }

        Ok(())
    }
}

enum ConnectionKind {
    Tcp,
    WebSocket,
}
