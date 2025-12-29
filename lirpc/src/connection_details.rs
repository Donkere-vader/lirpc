#[derive(Default)]
pub struct ConnectionDetails<S: Clone> {
    pub(crate) connection_state: S,
}

impl<S: Clone> ConnectionDetails<S> {
    pub fn new(connection_state: S) -> Self {
        Self { connection_state }
    }
}
