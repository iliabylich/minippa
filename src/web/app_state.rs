use crate::index::Index;

#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) index: Index,
}

impl AppState {
    pub(crate) fn new(index: Index) -> Self {
        Self { index }
    }
}
