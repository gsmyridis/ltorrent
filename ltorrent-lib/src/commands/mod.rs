use std::path::Path;

use anyhow::Context;

use super::config::Configuration;
use super::torrent;
use super::tracker::{PeersAddresses, Tracker, TrackerRequest};

/// Fetches the list of peer addresses from the tracker for a given torrent file.
///
/// # Errors
///
/// If the torrent file cannot be read, or the info dictionary cannot be hashed, or the tracker
/// request fails, an error is returned.
///
/// # Example
///
/// ```no_run
/// use ltorrent::commands;
///
/// #[tokio::main]
/// async fn main() {
///    let path = "path/to/torrent/file.torrent";
///    let peers = commands::peers(path).await.unwrap();
///    println!("Peers: {:?}", peers);
/// }
/// ```
pub async fn peers(path: impl AsRef<Path>) -> anyhow::Result<PeersAddresses> {
    /// Load the torrent file, and hash the info dictionary.
    let torrent = torrent::Torrent::from_file(&path)
        .await
        .context("Failed to read torrent file.")?;

    let info_hash = torrent
        .info_hash()
        .context("Failed to hash info dictionary")?;

    /// Create a tracker and send a tracker request.
    let config = Configuration::default();
    let tracker = Tracker::new(torrent.announce())?;
    let request = TrackerRequest::new(
        &info_hash,
        config.peer_id(),
        config.port(),
        0,
        0,
        torrent.length(),
        1,
    );
    let response = tracker.query(request).await?;
    Ok(response.peers().to_owned())
}


