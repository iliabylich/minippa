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
use system_deps::SystemDeps;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    SystemDeps::ensure_installed().await?;
    let Config { port, token, dir } = Config::read().await?;

    let (index, index_ctl) = index::spawn(&dir);

    match Args::parse() {
        Args::StartServer => {
            let web = web::spawn(port, index_ctl.clone(), &token, &dir).await?;
            tokio::try_join!(index, web)?;
        }
        Args::GenerateKey => {
            GPG::generate_key().await?;
            let key = GPG::export_key().await?;

            index_ctl.write_gpg_key(key).await?;
            index_ctl.stop().await?;

            tokio::try_join!(index)?;
        }
    }

    Ok(())
}
