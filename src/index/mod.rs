mod ctl;
mod find;
mod list;
mod make_install_script;
mod package;
mod request;
mod task;
mod upload;
mod write_gpg_key;

pub(crate) use ctl::IndexCtl;
pub(crate) use package::Package;
use task::IndexTask;
use tokio::task::JoinHandle;

pub(crate) fn spawn(dir: &str) -> (JoinHandle<()>, IndexCtl) {
    let (handle, tx) = IndexTask::spawn(dir);
    let ctl = IndexCtl::new(tx);
    (handle, ctl)
}
