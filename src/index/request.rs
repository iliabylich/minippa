use crate::index::Package;
use anyhow::Result;
use tokio::sync::oneshot::Sender;

pub(crate) enum IndexRequest {
    Upload {
        filename: String,
        data: Vec<u8>,
        ack: Sender<Result<()>>,
    },

    List {
        ack: Sender<Result<Vec<Package>>>,
    },

    Find {
        name: String,
        ack: Sender<Result<Option<Package>>>,
    },

    MakeInstallScript {
        base_url: String,
        ack: Sender<String>,
    },

    WriteGpgKey {
        key: String,
        ack: Sender<Result<()>>,
    },

    Stop {
        ack: Sender<()>,
    },
}
