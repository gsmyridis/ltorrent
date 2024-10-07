use std::net::SocketAddrV4;

use anyhow::Context;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use super::bitfield::BitField;
use super::message::*;

/// Represents a peer connection in the BitTorrent network.
///
/// This struct holds the state of the connection with a specific peer, including:
/// - The socket address of the peer.
/// - The framed stream for sending and receiving messages.
/// - The bitfield representing the pieces that the peer has.
///
/// The `Peer` struct implements the `PeerConnection` trait, which allows the user to
/// interact with the peer connection in a structured manner.
pub struct Peer<S> {
    address: SocketAddrV4,
    peer_id: [u8; 20],
    stream: Framed<S, MessageFramer>,
    bitfield: BitField,
}

impl<S> Peer<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn peer_id(&self) -> &[u8; 20] {
        &self.peer_id
    }

    pub fn has_piece(&self, piece_i: usize) -> bool {
        self.bitfield.contains_piece(piece_i)
    }

    pub async fn send(&mut self, message: Message) -> std::io::Result<()> {
        self.stream.send(message).await
    }

    pub async fn next(&mut self) -> Option<std::io::Result<Message>> {
        self.stream.next().await
    }
}


/// A builder for the `Peer` struct, with a `TcpStream` as stream.
pub struct PeerConnectionBuilder {
    address: SocketAddrV4,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

impl PeerConnectionBuilder {
    /// Creates a new `PeerConnectionBuilder`.
    pub fn new(address: SocketAddrV4, info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self { address, info_hash, peer_id }
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
    pub async fn build(&self) -> anyhow::Result<Peer<TcpStream>> {

        // Connect to peer with TCP stream.
        let mut stream = TcpStream::connect(self.address)
            .await
            .context("Failed to connect to peer via TCP stream.")?;

        // Perform handshake with peer.
        let handshake = HandShakeMessage::new(self.info_hash, self.peer_id);
        let mut handshake_bytes = [0u8; size_of::<HandShakeMessage>()];
        stream.write_all(&handshake.to_bytes()).await.context("Failed to send handshake.")?;
        stream.read_exact(&mut handshake_bytes).await.context("Failed to receive handshake.")?;
        anyhow::ensure!(
            handshake_bytes[1..20] == *b"BitTorrent protocol",
            "Peer did not send BitTorrent protocol."
        );
        let peer_id: [u8; 20] = handshake_bytes[48..].try_into()?;

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
            address: self.address,
            peer_id,
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
pub(crate) struct HandShakeMessage {
    length: u8,
    protocol: [u8; 19],
    reserved: [u8; 8],
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

impl HandShakeMessage {
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
    pub(crate) fn to_bytes(&self) -> [u8; 68] {
        let mut bytes = [0; 68];
        bytes[0] = self.length;
        bytes[1..20].copy_from_slice(&self.protocol);
        bytes[20..28].copy_from_slice(&self.reserved);
        bytes[28..48].copy_from_slice(&self.info_hash);
        bytes[48..68].copy_from_slice(&self.peer_id);
        bytes
    }
}
