use anyhow::Context;
use hex;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

/// Represents a meta-info file (.torrent file), which contains metadata about files to be
/// shared over a BitTorrent network. It includes the URL of the tracker and a detailed info
/// dictionary describing the files to be shared.
///
/// # Fields
///
/// * `announce`: The URL of the tracker that coordinates the distribution of file pieces
///   between peers. The tracker URL is essential for peers to find each other and share files.
///
/// * `info`: A structured dictionary containing UTF-8 formatted strings that provide detailed
///   information about the files to be shared. This includes the names of the files, their
///   lengths, and the SHA1 hashes of the pieces.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent {
    announce: String,
    info: Info,
}

impl Torrent {
    /// Creates a new `Torrent` instance from a .torrent file.
    ///
    /// # Errors
    ///
    /// This function can return an error in the following situations:
    /// - If the torrent file specified by `path` cannot be opened or read.
    /// - If the contents of the .torrent file cannot be parsed into a `Torrent` struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ltorrent::torrent::Torrent;
    ///
    /// # async fn run() -> anyhow::Result<()> {
    /// let torrent = Torrent::from_file("path/to/torrent/file.torrent").await?;
    /// println!("Tracker URL: {}", torrent.tracker_url());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_file(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let torrent_file = std::fs::read(path.as_ref()).context("Failed to open torrent file.")?;
        let torrent: Self =
            serde_bencode::from_bytes(&torrent_file).context("Failed to parse torrent file.")?;
        Ok(torrent)
    }

    /// This method provides access to the `announce` field of the `Torrent` struct,
    /// which contains the URL of the tracker. The tracker coordinates the distribution
    /// of file pieces between peers in the BitTorrent network.
    pub fn announce(&self) -> &str {
        &self.announce
    }

    /// Returns the name of the file or directory in the torrent.
    ///
    /// In the case of a single file, it is the name of the file. In the case of multiple
    /// files it's the name of the top-level directory.
    pub fn name(&self) -> &str {
        &self.info.name
    }

    /// Returns the keys of the file(s) in the torrent.
    ///
    /// If the torrent contains a single file, the name of the file is returned. If the
    /// torrent contains multiple files, then a `Vec` of the files' paths is returned.
    pub fn keys(&self) -> &Keys {
        &self.info.keys
    }

    /// Return the length of the file(s) in the torrent.
    ///
    /// If there is a single file, then the length of the file is returned. If there are
    /// multiple files, then the length of all the files is returned by summing the
    /// individual lengths.
    pub fn length(&self) -> usize {
        match &self.info.keys {
            Keys::SingleFile { length } => *length,
            Keys::MultiFile { files } => files.iter().map(|file| file.length).sum(),
        }
    }

    /// Print the file(s) in the torrent.
    /// # TODO: Fix the output format. Make it tree-like.
    pub fn print_tree(&self) {
        match &self.info.keys {
            Keys::SingleFile { length: _ } => {
                println!("{}", self.name());
            }
            Keys::MultiFile { files } => {
                for file in files {
                    println!("{}", file.path());
                }
            }
        }
    }

    /// Returns the total number of pieces that the file(s) in the torrent are divided into.
    pub fn n_pieces(&self) -> usize {
        self.info.pieces.0.len()
    }

    /// Returns the number of bytes in each piece the file is split into.
    ///
    /// For the purposes of transfer, files are split into fixed-size pieces which are all
    /// the same except for possibly the last one which may be truncated. `piece_length` is
    /// almost always a power of two, most commonly 2^18 = 256 KB.
    pub fn piece_length(&self) -> usize {
        self.info.piece_length
    }

    /// Returns the SHA1 hash of a specified piece in the torrent as a sequence of bytes.
    ///
    /// Each piece is assigned a SHA-1 hash value. On public networks, there may be
    /// malicious peers that send fake data. These hash values allow us to verify the
    /// integrity of each piece that we'll download.
    ///
    /// # Errors
    ///
    /// This function can return an error if the piece index is out of bounds.
    pub fn get_piece_hash(&self, piece_i: usize) -> anyhow::Result<&[u8; 20]> {
        self.info
            .pieces
            .0
            .get(piece_i)
            .context("Piece index out of bounds.")
    }

    /// Return a `Vec` of the SHA1 hashes of the pieces in the torrent encoded in hexadecimal.
    ///
    /// Each piece is assigned a SHA-1 hash value. On public networks, there may be
    /// malicious peers that send fake data. These hash values allow us to verify the
    /// integrity of each piece that we'll download.
    pub fn pieces_hex(&self) -> Vec<String> {
        self.info.pieces.0.iter().map(hex::encode).collect()
    }

    /// Return a `Vec` of the SHA1 hashes of the pieces in the torrent in bytes.
    ///
    /// Each piece is assigned a SHA-1 hash value. On public networks, there may be
    /// malicious peers that send fake data. These hash values allow us to verify the
    /// integrity of each piece that we'll download.
    pub fn pieces_sha1(&self) -> &Vec<[u8; 20]> {
        &self.info.pieces.0
    }

    /// Returns the SHA1 hash of the info dictionary.
    ///
    /// The SHA1 hash of the info dictionary is used to identify the torrent file. All
    /// keys are sorted alphabetically in bencoding. So the serialized and deserialized
    /// info dictionary will have the keys in the same order.
    ///
    /// # Errors
    ///
    /// This function can return an error if the info dictionary cannot be serialized.
    pub fn info_hash(&self) -> anyhow::Result<[u8; 20]> {
        let bytes =
            serde_bencode::to_bytes(&self.info).context("Failed to serialise info dictionary.")?;
        let mut hasher = Sha1::new();
        hasher.update(&bytes);
        Ok(hasher.finalize().into())
    }
}

/// Torrent info dictionary.
///
/// - `name`: The name of the file or directory.
///
/// In the case of a single file, it is the name of the file. In the case of multiple files it's
/// the name of the top-level directory.
///
/// - `piece_length`: The number of bytes in each peace the file is split into.
///
/// For the purposes of transfer, files are split into fixed-size pieces which are all the same
/// except for possibly the last one which may be truncated. `piece length` is almost always a
/// power of two, most commonly 2^18 = 256 KB.
///
/// - `pieces`: The SHA1 hashes of the pieces in the file.
///
/// Each piece is assigned a SHA-1 hash value. On public networks, there may be malicious peers
/// that send fake data. These hash values allow us to verify the integrity of each piece
/// that we'll download.
///
/// - `keys`: The key of the file(s) in the torrent.
///
/// If there is a single file, then the key is `length`. If there are multiple files, then the key
/// is `files`.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Info {
    name: String,
    #[serde(rename = "piece length")]
    piece_length: usize,
    pieces: Hashes,
    #[serde(flatten)]
    keys: Keys,
}

/// There is a key `length` and a key `files`, but not both or neither.
///
/// If length is present then the download represents a single file. In the single file case,
/// `length` maps to the length of the file in bytes.
///
/// Otherwise, it represents a set of files which go in a directory structure. For the purposes of
/// the other keys in `Info`, the multi-file case is treated as only having a single file by
/// concatenating the files in the order they appear in the files list.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Keys {
    SingleFile { length: usize },
    MultiFile { files: Vec<File> },
}

/// Represents a file listed in the torrent.
///
/// This struct contains metadata about a file that is part of the torrent. It includes
/// the length of the file in bytes and the subdirectory names that form the path to the file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    length: usize,
    /// Subdirectory names for this file, the last of which is the actual file name
    /// (a zero length list is an error case).
    #[serde(rename = "path")]
    subdirectories: Vec<String>,
}

impl File {
    /// Return the path of the file.
    pub fn path(&self) -> String {
        self.subdirectories
            .join(std::path::MAIN_SEPARATOR.to_string().as_str())
    }

    /// Returns the length of the file in bytes.
    pub fn length(&self) -> usize {
        self.length
    }
}

/// A list of SHA1 hashes.
#[derive(Debug, Clone)]
pub struct Hashes(pub Vec<[u8; 20]>);

impl serde::Serialize for Hashes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0.concat())
    }
}

impl<'de> serde::Deserialize<'de> for Hashes {
    fn deserialize<D>(deserialiser: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserialiser.deserialize_bytes(HashesVisitor)
    }
}

struct HashesVisitor;

impl<'de> serde::de::Visitor<'de> for HashesVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a byte string whose length is a multiple of 20")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() % 20 != 0 {
            return Err(E::invalid_length(v.len(), &self));
        }
        Ok(Hashes(
            v.chunks_exact(20)
                .map(|chunk| chunk.try_into().expect("Guaranteed to be length 20"))
                .collect(),
        ))
    }
}
