use std::io::Write;
use std::path::Path;

use ltorrent::commands;

/// Invokes the command to fetch and print the list of peer addresses from the tracker for a given torrent file.
///
/// # Arguments
///
/// * `path` - A path to the torrent file.
///
/// # Returns
///
/// * `anyhow::Result<()>` - An empty result indicating success or an error.
///
/// # Errors
///
/// This function will return an error if:
/// - The torrent file cannot be read.
/// - The tracker request fails.
/// - Writing to stdout fails.
pub async fn invoke(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let peers = commands::peers(path).await?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    for peer_addr in peers.0 {
        writeln!(stdout, "{}", peer_addr)?;
    }

    Ok(())
}