use std::io::Write;
use std::net::SocketAddrV4;
use std::path::Path;
use std::str::FromStr;

use anyhow::Context;
use tokio::net::TcpStream;

use ltorrent::config::Configuration;
use ltorrent::net::peers::Peer;
use ltorrent::torrent::Torrent;
use ltorrent::tracker::{Tracker, TrackerRequest};

/// Invokes the command to fetch and print the list of peer addresses from the tracker for a given torrent file.
pub async fn search(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let torrent = Torrent::from_file(&path)
        .await
        .context("Failed to read torrent file.")?;

    let info_hash = torrent
        .info_hash()
        .context("Failed to hash info dictionary")?;

    // Create a tracker and send a tracker request.
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

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    for peer_addr in &response.peers().0 {
        writeln!(stdout, "{}", peer_addr)?;
    }

    Ok(())
}

/// Performs a handshake with a specified peer.
pub async fn handshake(path: impl AsRef<Path>, address: &str) -> anyhow::Result<()> {
    let address = SocketAddrV4::from_str(address).context("Not a valid peer address")?;

    let torrent = Torrent::from_file(&path)
        .await
        .context("Failed to read torrent file.")?;
    let info_hash = torrent
        .info_hash()
        .context("Failed to hash info dictionary")?;

    let config = Configuration::default();
    let peer_id: [u8; 20] = config.peer_id().as_bytes().try_into()?;

    let peer = Peer::<TcpStream>::new(address, peer_id, info_hash).await?;
    let peer_id = hex::encode(peer.peer_id());

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    writeln!(stdout, "Peer ID: {peer_id}")?;
    Ok(())
}
