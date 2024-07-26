use anyhow::Context;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_bencode;

/// Represents a BitTorrent tracker.
///
/// Trackers are central servers that maintain information about peers participating in the sharing
/// and downloading of a torrent.
pub struct Tracker {
    url: String,
}

impl Tracker {
    /// Creates a new Tracker with the specified URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid.
    pub fn new(url: String) -> anyhow::Result<Self> {
        let _ = Url::parse(&url).context("Failed to pars URL.")?;
        Ok(Tracker { url })
    }

    /// Sends a query to the tracker and returns the response.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn query(&self, request: TrackerRequest) -> anyhow::Result<TrackerResponse> {
        let query = request.serialize();
        let mut url = Url::parse(&self.url).context("Failed to parse URL.")?;
        url.set_query(Some(query.as_str()));

        let response = reqwest::get(url).await.context("Failed to fetch tracker response.")?;
        let response = response.bytes().await.context("Failed to read tracker response.")?;
        let response: TrackerResponse = serde_bencode::from_bytes(&response).context("Failed to deserialize tracker response.")?;
        Ok(response)
    }
}

/// Represents a request to a tracker.
///
/// The request contains information about the client and the torrent.
/// * info hash: The SHA1 hash of the info dictionary in the torrent file.
/// * peer ID: The peer ID of the client.
/// * port: The port number the client is listening on. Common behavior is for a
///   downloader to try to listen on port 6881 and if that port is taken try 6882,
///   then 6883, etc. and give up after 6889.
/// * uploaded: The total number of bytes uploaded so far.
/// * downloaded: The total number of bytes downloaded so far.
/// * left: The number of bytes the client still has to download.
/// * compact: Whether the peer list should use the compact representation.
///   The compact representation is more commonly used in the wild, the non-compact
///   representation is mostly supported for backward-compatibility.
#[derive(Debug, Clone)]
pub struct TrackerRequest {
    /// Note that this is a substring of the metainfo file. The info-hash must be the hash of the
    /// encoded form as found in the .torrent file, which is identical to bencoding the metainfo
    /// file, extracting the info dictionary and encoding it if and only if the bencoder fully
    /// validated the input (e.g. key ordering, absence of leading zeros). Conversely, that means
    /// clients must either reject invalid metainfo files or extract the substring directly.
    /// They must not perform a decode-encode round-trip on invalid data.
    info_hash: [u8; 20],
    peer_id: String,
    port: u16,
    uploaded: usize,
    downloaded: usize,
    /// The number of bytes this peer still has to download, encoded in base ten ascii. Note that
    /// this can't be computed from downloaded and the file length since it might be a resume, and
    /// there's a chance that some of the downloaded data failed an integrity check and had to be
    /// re-downloaded.
    left: usize,
    /// Whether the peer list should use the compact representation. Boolean encoded as integer.
    /// Ref: https://www.bittorrent.org/beps/bep_0023.html.
    compact: u8,
}

impl TrackerRequest {
    /// Creates a new TrackerRequest.
    pub fn new(
        info_hash: [u8; 20],
        peer_id: String,
        port: u16,
        uploaded: usize,
        downloaded: usize,
        left: usize,
        compact: u8,
    ) -> Self {
        TrackerRequest {
            info_hash,
            peer_id,
            port,
            uploaded,
            downloaded,
            left,
            compact,
        }
    }

    /// Serializes the request into a URL-encoded string.
    ///
    /// The method is needed because of the difficulty of serializing the info_hash field.
    pub(crate) fn serialize(&self) -> String {
        let hex_str = hex::encode(&self.info_hash);
        let encoded_str = hex_str
            .chars()
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|chunk| format!("%{}", chunk.iter().collect::<String>()))
            .collect::<String>();
        format!(
            "info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
            encoded_str,
            self.peer_id,
            self.port,
            self.uploaded,
            self.downloaded,
            self.left,
            self.compact
        )
    }
}

/// Represents the response from a tracker.
///
/// The response contains a list of peers' addresses and the interval between requests.
#[derive(Debug, Clone, Deserialize)]
pub struct TrackerResponse {
    interval: usize,
    peers: PeersAddresses,
}

impl Serialize for PeersAddresses {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut bytes = Vec::with_capacity(self.0.len() * 6);
        for peer_addr in &self.0 {
            bytes.extend_from_slice(&peer_addr.ip().octets());
            bytes.extend_from_slice(&peer_addr.port().to_be_bytes());
        }
        serializer.serialize_bytes(&bytes)
    }
}

#[derive(Debug, Clone)]
pub struct PeersAddresses(pub Vec<std::net::SocketAddrV4>);

/// Implementation of Deserialization for Peers.
impl<'de> Deserialize<'de> for PeersAddresses {
    fn deserialize<D>(deserializer: D) -> Result<PeersAddresses, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}

struct PeersVisitor;

impl<'de> serde::de::Visitor<'de> for PeersVisitor {
    type Value = PeersAddresses;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "6 bytes. The first 4 bytes are a peer's IP address, the last 2 bytes are the port number.")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() % 6 != 0 {
            return Err(E::invalid_length(v.len(), &self));
        }
        Ok(PeersAddresses(
            v.chunks_exact(6)
                .map(|chunk| {
                    let ip = std::net::Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                    let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                    std::net::SocketAddrV4::new(ip, port)
                })
                .collect(),
        ))
    }
}
