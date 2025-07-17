mod entry;
mod package;
mod raw_index;
mod upload;

use anyhow::Result;
pub(crate) use package::Package;
use raw_index::RawIndex;
use std::sync::Arc;
use tokio::sync::Mutex;
pub(crate) use upload::Upload;

#[derive(Clone, Debug)]
pub(crate) struct Index {
    inner: Arc<Mutex<RawIndex>>,
}

impl Index {
    pub(crate) async fn new() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(Mutex::new(RawIndex::new().await?)),
        })
    }

    pub(crate) async fn write(&self, filename: String, upload: Upload) -> Result<()> {
        let inner = self.inner.lock().await;
        inner.write(filename, upload).await?;
        Ok(())
    }
}
