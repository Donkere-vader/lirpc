use std::{collections::HashMap, sync::Arc};

use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::mpsc,
};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};
use tracing::{debug, error, warn};

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    handler::Handler,
    lirpc_message::{LiRpcRequest, LiRpcStreamOutput},
    service::{HandlerService, Service},
    stream_manager::StreamManager,
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
    pub fn register_handler<F, T>(
        mut self,
        name: String,
        handler: impl Handler<F, T, S, C> + 'static,
    ) -> Self
    where
        F: 'static,
        T: 'static,
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
        websocket_message: Message,
        state: S,
        connection: Arc<ConnectionDetails<C>>,
        output: mpsc::Sender<LiRpcStreamOutput>,
        stream_manager: StreamManager,
    ) -> Result<(), LiRpcError> {
        let message = LiRpcRequest::try_from(websocket_message)?;

        debug!("Received message: {message:?}");

        match message {
            LiRpcRequest::FunctionCall(fc) => {
                let handler = handlers
                    .get(&fc.headers.method)
                    .ok_or(LiRpcError::HandlerNotFound(fc.headers.method.to_string()))?;

                handler
                    .call(connection, fc, state, output, stream_manager)
                    .await?;
            }
            LiRpcRequest::CloseStream(cs) => stream_manager.close_stream(cs.stream_id).await?,
        }

        Ok(())
    }

    async fn handle_connection(
        socket: WebSocketStream<TcpStream>,
        state: S,
        new_connection_details: ConnectionDetails<C>,
        handlers: Arc<HashMap<String, Box<dyn Service<S, C>>>>,
    ) {
        let (mut ws_sender, mut ws_receiver) = socket.split();
        let (tx, mut rx) = mpsc::channel(10);

        let connection_details = Arc::new(new_connection_details);
        let stream_manager = StreamManager::default();

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
                            let stream_manager_clone = stream_manager.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_message(handlers_clone, message, state_clone, connection_clone, tx_clone, stream_manager_clone).await {
                                    match e {
                                        LiRpcError::OutputStreamClosed => {},
                                        _ => error!("Error handling message: {e}"),
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
                            debug!("Error serializing response: {e}");
                            break;
                        }
                    };

                    if let Err(e) = ws_sender.send(serialized_response).await {
                        debug!("Error sending response: {e}");
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

            let new_connection_details =
                ConnectionDetails::new((*self.connection_state_initializer)());

            tokio::spawn(async move {
                Self::handle_connection(
                    accepted_stream,
                    state_clone,
                    new_connection_details,
                    handlers_clone,
                )
                .await;
            });
        }

        Ok(())
    }
}
