use std::net::SocketAddrV4;

use anyhow::Context;
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use super::bitfield::BitField;
use super::message::*;

/// It holds the state of the connection with a specific peer. Specifically, it
/// stores the socket address of the peer, the framed stream, and the bitfield
/// of the peer.
pub struct Peer {
    address: SocketAddrV4,
    stream: Framed<TcpStream, MessageFramer>,
    bitfield: BitField,
}

impl Peer {
    /// Checks if the peer has a specific piece.
    fn has_piece(&self, piece_i: usize) -> bool {
        self.bitfield.contains_piece(piece_i)
    }

    /// Sends a message to the peer.
    pub async fn send(&mut self, message: Message) -> std::io::Result<()> {
        self.stream.send(message).await
    }

    /// Receives a message from the peer.
    pub async fn next(&mut self) -> Option<std::io::Result<Message>> {
        self.stream.next().await
    }
}

/// A builder for the `Peer` struct.
pub struct PeerBuilder {
    address: Option<SocketAddrV4>,
    info_hash: Option<[u8; 20]>,
    peer_id: Option<[u8; 20]>,
}

impl PeerBuilder {
    /// Creates a new `PeerConnectionBuilder`, with all fields set to `None`.
    pub fn new() -> Self {
        Self { address: None, info_hash: None, peer_id: None }
    }

    /// Sets the address of the peer.
    pub fn with_address(&mut self, address: SocketAddrV4) -> &mut Self {
        self.address = Some(address);
        self
    }

    /// Sets the info hash of the torrent.
    pub fn with_info_hash(&mut self, info_hash: &[u8; 20]) -> &mut Self {
        self.info_hash = Some(info_hash.to_owned());
        self
    }

    /// Sets the peer ID of the client.
    pub fn with_peer_id(&mut self, peer_id: &[u8; 20]) -> &mut Self {
        self.peer_id = Some(peer_id.to_owned());
        self
    }


    /// Creates a new peer connection.
    ///
    /// First, it connects to the peer with a TCP stream. Subsequently, it performs
    /// the handshake with the peer. If the handshake is successful, it receives the
    /// bitfield message from the peer, which contains the pieces that the peer has.
    /// Finally, it returns a new peer connection.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The address is not set.
    /// - The info hash is not set.
    /// - The peer ID is not set.
    /// - The handshake message cannot be sent.
    /// - The handshake message cannot be received.
    /// - The received handshake message does not follow the BitTorrent protocol.
    pub async fn build(&self) -> anyhow::Result<Peer> {
        // Check every field is set.
        let address = self.address.context("Address is not set.")?;
        let info_hash = self.info_hash.context("Info hash is not set.")?;
        let peer_id = self.peer_id.context("Peer ID is not set.")?;

        // Connect to peer with TCP stream.
        let mut stream = TcpStream::connect(address)
            .await
            .context("Failed to connect to peer.")?;

        // Perform handshake with peer.
        let handshake = HandShake::new(info_hash, peer_id);
        let mut handshake_bytes = [0; 68];
        stream.write_all(&handshake.to_bytes()).await.context("Failed to send handshake.")?;
        stream.read_exact(&mut handshake_bytes).await.context("Failed to receive handshake.")?;
        anyhow::ensure!(
            handshake_bytes[1..20] == *b"BitTorrent protocol",
            "Peer did not send BitTorrent protocol."
        );


        // Frame stream so that messages can be sent and received in a structured manner.
        let mut framed_stream = Framed::new(stream, MessageFramer);
        let bitfield = framed_stream
            .next()
            .await
            .expect("Peer always sends Bitfield message.")?;
        anyhow::ensure!(
            *bitfield.tag() == MessageTag::Bitfield,
            "Peer did not send Bitfield message."
        );

        Ok(Peer {
            address,
            stream: framed_stream,
            bitfield: BitField::from_payload(bitfield.payload()),
        })
    }
}

/// It represents the handshake message that is exchanged between peers.
///
/// The handshake message requires:
/// * length: 1 byte, which is always 19.
/// * protocol: 19 bytes, which is always "BitTorrent protocol".
/// * reserved: 8 bytes, which is always 0.
/// * info hash: 20 bytes, which is the SHA1 hash of the info dictionary in the torrent file.
/// * peer ID: 20 bytes, which is the peer ID of the client.
pub(crate) struct HandShake {
    length: u8,
    protocol: [u8; 19],
    reserved: [u8; 8],
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

impl HandShake {
    /// Creates a new HandShake struct from the SHA1 hash of the info dictionary in the
    /// torrent file and the peer ID.
    pub(crate) fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            length: 19,
            protocol: *b"BitTorrent protocol",
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }

    /// Returns the `HandShake` struct as a byte array.
    ///
    /// The handshake message contains:
    /// * length: 1 byte, which is always 19.
    /// * protocol: 19 bytes, which is always "BitTorrent protocol".
    /// * reserved: 8 bytes, which is always 0.
    /// * info hash: 20 bytes, which is the SHA1 hash of the info dictionary in the torrent file.
    /// * peer ID: 20 bytes, which is the peer ID of the client.
    pub(crate) fn to_bytes(&self) -> [u8; std::mem::size_of::<HandShake>()] {
        let mut bytes = [0; std::mem::size_of::<HandShake>()];
        bytes[0] = self.length;
        bytes[1..20].copy_from_slice(&self.protocol);
        bytes[20..28].copy_from_slice(&self.reserved);
        bytes[28..48].copy_from_slice(&self.info_hash);
        bytes[48..68].copy_from_slice(&self.peer_id);
        bytes
    }
}
