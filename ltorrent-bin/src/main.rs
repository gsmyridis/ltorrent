use anyhow::Context;
use clap::Parser;

use crate::cli::*;

mod cli;
mod commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Info { torrent_path } => {
            commands::info::invoke(torrent_path)
                .await
                .context("Failed to fetch info")?;
        }
        Command::Peers { torrent_path } => {
            commands::peers::search(torrent_path)
                .await
                .context("Failed to fetch peers")?;
        }
        Command::Handshake { torrent_path, peer_address } => {
            commands::peers::handshake(torrent_path, peer_address.as_str()).await?;
        }
    }
    Ok(())
}
