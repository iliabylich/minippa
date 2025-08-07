use crate::index::{Package, request::IndexRequest};
use anyhow::{Context as _, Result};
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub(crate) struct IndexCtl(Sender<IndexRequest>);

impl IndexCtl {
    pub(crate) fn new(tx: Sender<IndexRequest>) -> Self {
        Self(tx)
    }

    pub(crate) async fn upload(&self, filename: String, data: Vec<u8>) -> Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let req = IndexRequest::Upload {
            filename,
            data,
            ack: tx,
        };
        self.roundtrip(req, rx).await?
    }

    pub(crate) async fn list(&self) -> Result<Vec<Package>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let req = IndexRequest::List { ack: tx };
        self.roundtrip(req, rx).await?
    }

    pub(crate) async fn find(&self, name: String) -> Result<Option<Package>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let req = IndexRequest::Find { name, ack: tx };
        self.roundtrip(req, rx).await?
    }

    pub(crate) async fn make_install_script(&self, base_url: String) -> Result<String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let req = IndexRequest::MakeInstallScript { base_url, ack: tx };
        self.roundtrip(req, rx).await
    }

    pub(crate) async fn write_gpg_key(&self, key: String) -> Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let req = IndexRequest::WriteGpgKey { key, ack: tx };
        self.roundtrip(req, rx).await?
    }

    pub(crate) async fn stop(&self) -> Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let req = IndexRequest::Stop { ack: tx };
        self.roundtrip(req, rx).await
    }

    async fn roundtrip<T>(
        &self,
        req: IndexRequest,
        rx: tokio::sync::oneshot::Receiver<T>,
    ) -> Result<T> {
        self.0
            .send(req)
            .await
            .context("failed to send request: channel is closed")?;
        rx.await
            .context("failed to receive response: channel is closed")
    }
}
