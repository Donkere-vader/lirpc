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
    api_spec::ApiSpec,
    connection_details::ConnectionDetails,
    error::LiRpcError,
    handler::Handler,
    lirpc_message::{LiRpcRequest, LiRpcResponse},
    service::{HandlerService, Service},
    translatable::Translatable,
    type_definition::TypeDefinition,
};

pub struct NamedHandler<S, C> {
    name: String,
    handler: Box<dyn Service<S, C>>,
}

impl<S, C> NamedHandler<S, C> {
    pub fn new<F, T, R>(name: String, handler: impl Handler<F, T, S, C, R> + 'static) -> Self
    where
        F: 'static,
        T: 'static,
        R: Translatable + 'static,
        S: Send + Sync + Clone + 'static,
        C: Clone + Send + Sync + 'static,
    {
        Self {
            name,
            handler: Box::new(HandlerService(Box::new(handler))),
        }
    }
}

#[derive(Default)]
pub struct ServerBuilder<S: Clone, C> {
    handlers: HashMap<String, Box<dyn Service<S, C>>>,
    type_definitions: HashMap<String, TypeDefinition>,
}

impl<S, C> ServerBuilder<S, C>
where
    S: Send + Sync + Clone + 'static,
    C: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            type_definitions: HashMap::new(),
        }
    }

    /// Register a method/handler to this service
    /// The `name` given to the handler here is what a client
    /// will use to end up calling this handler/method
    ///
    /// Check out `.with_handlers` for setting up all handlers for the
    /// server in one go.
    pub fn register_handler<F, T, R>(
        mut self,
        name: String,
        handler: impl Handler<F, T, S, C, R> + 'static,
    ) -> Self
    where
        F: 'static,
        T: 'static,
        R: Translatable + 'static,
    {
        if name.starts_with("lirpc_") {
            // Using `eprintln` instead of something from tracing,
            // as tracing might not have been initialized at this point
            // and we also want to print this if a user is not using tracing.
            eprintln!(
                "Handlers prefixed with 'lirpc_' are reserved. You should name your methods differently."
            )
        }
        self.handlers
            .insert(name, Box::new(HandlerService(Box::new(handler))));
        self
    }

    /// Overwrites the set of handlers with the supplied the collection.
    /// Recommended usage is in combination with the `handlers!` macro.
    ///
    /// # Example
    /// ```rs
    /// ServerBuilder::new()
    ///     .with_handlers(handlers!(greet, ping))
    ///     .build()
    /// ```
    pub fn with_handlers(mut self, handlers: Vec<NamedHandler<S, C>>) -> Self {
        for handler in handlers.iter() {
            if handler.name.starts_with("lirpc_") {
                // Using `eprintln` instead of something from tracing,
                // as tracing might not have been initialized at this point
                // and we also want to print this if a user is not using tracing.
                eprintln!(
                    "Handlers prefixed with 'lirpc_' are reserved. You should name your methods differently."
                )
            }
        }
        self.handlers = handlers.into_iter().map(|h| (h.name, h.handler)).collect();

        self
    }

    /// Registers the types that the server uses, for api spec
    /// generation.
    /// Recommended usage is in combination with the `types!` macro.
    ///
    /// # Example
    /// ```rs
    /// ServerBuilder::new()
    ///     .with_types(types!(GreetRequest, GreetResponse))
    ///     .build()
    /// ```
    pub fn with_types(mut self, types: Vec<(String, TypeDefinition)>) -> Self {
        self.type_definitions = types.into_iter().collect();

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
            type_definitions: Arc::new(self.type_definitions),
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
            type_definitions: Arc::new(self.type_definitions),
            connection_state_initializer: Box::new(|| ()),
        }
    }
}

impl ServerBuilder<(), ()> {
    pub fn build(self) -> Server<(), ()> {
        Server {
            state: (),
            handlers: Arc::new(self.handlers),
            type_definitions: Arc::new(self.type_definitions),
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
            type_definitions: Arc::new(self.type_definitions),
            connection_state_initializer: Box::new(default_connection_state),
        }
    }
}

pub struct Server<S: Clone, C> {
    state: S,
    handlers: Arc<HashMap<String, Box<dyn Service<S, C>>>>,
    type_definitions: Arc<HashMap<String, TypeDefinition>>,
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

    pub async fn compile_api_spec(
        &self,
        name: String,
        version: String,
    ) -> Result<ApiSpec, ApiSpecCompilationError> {
        ApiSpec::new(
            name,
            version,
            self.handlers
                .iter()
                .map(|h| (h.0.to_string(), h.1.get_spec()))
                .collect(),
            (*self.type_definitions).clone(),
        )
        .map_err(ApiSpecCompilationError::UnknownTypesReferenced)
    }

    pub async fn compile_json_api_spec(
        &self,
        name: String,
        version: String,
    ) -> Result<String, ApiSpecCompilationError> {
        Ok(serde_json::to_string(
            &self.compile_api_spec(name, version).await?,
        )?)
    }
}

enum ConnectionKind {
    Tcp,
    WebSocket,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiSpecCompilationError {
    #[error(
        "The type(s) {0:?} ware mentioned in a handler, but they were not registered with the server."
    )]
    UnknownTypesReferenced(Vec<String>),
    #[error("Error serializing api spec: {0:?}")]
    SerdeError(#[from] serde_json::Error),
}
