use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    Mutex,
    watch::{self, Receiver, Sender},
};

use crate::error::LiRpcError;

#[derive(Debug)]
pub enum StreamManagementMessage {
    CloseStream,
}

/// The StreamManager holds all the open streams from a connection
///
/// If an RPC method is called and it uses and `OutputStream<M>` the
/// stream will be registered here.
///
/// To keep track of the open streams and close once that the connection's client
/// no longer requires or "unsubscribes" from.
#[derive(Clone, Debug, Default)]
pub struct StreamManager {
    streams: Arc<Mutex<HashMap<u32, Sender<bool>>>>,
}

impl StreamManager {
    pub async fn register_stream(&self, stream_id: u32) -> Receiver<bool> {
        let (tx, rx) = watch::channel(true);

        let mut stream_lock = self.streams.lock().await;
        (*stream_lock).insert(stream_id, tx);

        rx
    }

    /// Send signal to producer (some handler) to close the stream
    ///
    /// the stream will automatically be removed from the registered streams.
    ///
    /// # Error
    ///
    /// fails if the closing signal fails to send over the `tokio::watch` channel.
    pub async fn close_stream(&self, stream_id: u32) -> Result<(), LiRpcError> {
        let mut stream_lock = self.streams.lock().await;

        if let Some(tx) = (*stream_lock).remove(&stream_id) {
            tx.send(false)?;
        }

        Ok(())
    }
}
