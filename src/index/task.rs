use crate::index::{
    find::find, list::list, make_install_script::make_install_script, request::IndexRequest,
    upload::upload, write_gpg_key::write_gpg_key,
};
use anyhow::{Context as _, Result, anyhow};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub(crate) struct IndexTask;

impl IndexTask {
    pub(crate) fn spawn(dir: &str) -> (JoinHandle<()>, Sender<IndexRequest>) {
        let dir = dir.to_string();
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let handle = tokio::spawn(async move {
            if let Err(err) = r#loop(rx, dir).await {
                log::error!("Index task has crashes: {err:?}");
            }
        });
        (handle, tx)
    }
}

async fn r#loop(mut rx: Receiver<IndexRequest>, dir: String) -> Result<()> {
    tokio::fs::create_dir_all(&dir)
        .await
        .context("failed to create data dir")?;

    while let Some(req) = rx.recv().await {
        if let IndexRequest::Stop { ack } = req {
            if ack.send(()).is_err() {
                log::error!("failed to ACK");
            }
            break;
        }

        if let Err(err) = dispatch(req, &dir).await {
            log::error!("failed to process request: {err:?}");
        }
    }

    Ok(())
}

async fn dispatch(req: IndexRequest, dir: &str) -> Result<()> {
    fn send<T>(ack: tokio::sync::oneshot::Sender<T>, reply: T) -> Result<()> {
        ack.send(reply).map_err(|_| anyhow!("failed to ACK"))
    }

    match req {
        IndexRequest::Upload {
            filename,
            data,
            ack,
        } => {
            let reply = upload(dir, filename, data).await;
            send(ack, reply)?;
        }
        IndexRequest::List { ack } => {
            let reply = list(dir, None).await;
            send(ack, reply)?;
        }
        IndexRequest::Find { name, ack } => {
            let reply = find(dir, name).await;
            send(ack, reply)?;
        }
        IndexRequest::MakeInstallScript { base_url, ack } => {
            let reply = make_install_script(base_url);
            send(ack, reply)?;
        }
        IndexRequest::WriteGpgKey { key, ack } => {
            let reply = write_gpg_key(dir, key).await;
            send(ack, reply)?;
        }
        IndexRequest::Stop { .. } => {
            // handled by the caller
        }
    }

    Ok(())
}
