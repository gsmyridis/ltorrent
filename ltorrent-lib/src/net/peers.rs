// use std::net::SocketAddrV4;
//
// use anyhow::Context;
// use tokio::net::TcpStream;
// use tokio_util::codec::Framed;
//
// use super::bitfield::BitField;
// use super::msg::{Message, MessageFramer};
//
// /// It holds the state of the connection with a specific peer. Specifically, it
// /// stores the socket address of the peer, the framed stream, and the bitfield
// /// of the peer.
// pub struct Peer {
//     address: SocketAddrV4,
//     stream: Framed<TcpStream, MessageFramer>,
//     pub bitfield: BitField,
// }
//
// impl Peer {
//     /// Creates a new peer connection.
//     ///
//     /// First, it connects to the peer with a TCP stream. Subsequently, it performs
//     /// the handshake with the peer. If the handshake is successful, it receives the
//     /// bitfield message from the peer, which contains the pieces that the peer has.
//     /// Finally, it returns a new peer connection.
//     ///
//     /// # Arguments
//     ///
//     /// * `address` - The address of the peer.
//     /// * `info_hash` - The info hash of the torrent.
//     ///
//     /// # Returns
//     ///
//     /// A new peer connection.
//     pub async fn new(
//         address: SocketAddrV4,
//         info_hash: [u8; 20],
//         peer_id: [u8; 20],
//     ) -> anyhow::Result<Self> {
//         let mut stream = TcpStream::connect(address)
//             .await
//             .context("Failed to connect to peer.")?;
//
//         let handshake = HandShake::new(info_hash, peer_id);
//         handshake.perform_handshake(&mut stream).await?;
//
//         let mut framed_stream = Framed::new(stream, MessageFramer);
//         let bitfield = framed_stream
//             .next()
//             .await
//             .expect("Peer always sends Bitfield message.")?;
//         anyhow::ensure!(
//             bitfield.tag == MessageTag::Bitfield,
//             "Peer did not send Bitfield message."
//         );
//
//         Ok(Self {
//             address,
//             stream: framed_stream,
//             bitfield: BitField::from_payload(bitfield.payload),
//         })
//     }
//
//     /// Checks if the peer has a specific piece.
//     fn has_piece(&self, piece_i: usize) -> bool {
//         self.bitfield.contains_piece(piece_i)
//     }
//
//     /// Sends a message to the peer.
//     pub async fn send(&mut self, message: Message) -> std::io::Result<()> {
//         self.stream.send(message).await
//     }
//
//     /// Receives a message from the peer.
//     pub async fn next(&mut self) -> Option<std::io::Result<Message>> {
//         self.stream.next().await
//     }
// }
