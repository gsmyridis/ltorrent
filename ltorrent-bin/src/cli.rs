use std::path::PathBuf;

use clap;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(clap::Subcommand)]
#[clap(rename_all = "snake_case")]
pub(crate) enum Command {
    Info {
        torrent_path: PathBuf,
    },
    Peers {
        torrent_path: PathBuf,
    },
}
