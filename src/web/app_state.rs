use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) lock: Arc<Mutex<()>>,
}

impl AppState {
    pub(crate) fn new() -> Self {
        Self {
            lock: Arc::new(Mutex::new(())),
        }
    }
}
