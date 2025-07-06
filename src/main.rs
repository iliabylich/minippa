mod args;
mod bash;
mod config;
mod gpg;
mod index;
mod system_deps;
mod web;

use anyhow::Result;
use args::Args;
use config::Config;
use gpg::GPG;
use index::Index;
use system_deps::SystemDeps;
use web::Web;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    SystemDeps::ensure_installed().await?;
    Config::read().await?;
    Index::mkdir_p().await?;

    match Args::parse() {
        Args::StartServer => {
            Web::spawn().await?;
        }
        Args::GenerateKey => {
            GPG::generate_key().await?;
            GPG::export_key().await?;
        }
    }

    Ok(())
}
