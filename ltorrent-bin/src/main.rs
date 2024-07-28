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
            commands::peers::invoke(torrent_path)
                .await
                .context("Failed to fetch peers")?;
        }
        _ => unimplemented!(),
    }
    Ok(())
}
