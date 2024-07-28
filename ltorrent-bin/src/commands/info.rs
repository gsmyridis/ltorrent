use std::io::Write;
use std::path::Path;

use anyhow::Context;

use ltorrent::torrent::Torrent;

/// Prints detailed information about a torrent file to the standard output.
///
/// # Errors
///
/// This function will return an error if:
/// - The torrent file cannot be read.
/// - The info dictionary cannot be hashed.
/// - Writing to stdout fails.
pub async fn invoke(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let torrent = Torrent::from_file(path).await?;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    writeln!(stdout, "Tracker URL: {}", torrent.announce())?;
    writeln!(stdout, "Length: {}", torrent.length())?;
    let info_hash = torrent
        .info_hash()
        .context("Failed to hash info dictionary")?;
    let info_hash = hex::encode(info_hash);
    writeln!(stdout, "Info Hash: {}", info_hash)?;
    writeln!(stdout, "Piece Length: {}", torrent.piece_length())?;
    writeln!(stdout, "Piece Hashes:")?;
    let hashes = torrent.pieces_hex();
    for sha1 in hashes {
        writeln!(stdout, "{}", sha1)?;
    }
    Ok(())
}