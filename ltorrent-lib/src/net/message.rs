use anyhow;
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

const MAX_MESSAGE_LENGTH: usize = 1 << 16;

/// Structured message exchanged between peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    tag: MessageTag,
    payload: Vec<u8>,
}

impl Message {
    /// Creates a new message from message tag and payload.
    ///
    /// # Errors
    ///
    /// If the message tag does not require a payload, and it is provided, an error is returned.
    /// The messages that do not require a payload are: `Choke`, `UnChoke`, `Interested`, and
    /// `NotInterested`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ltorrent::net::message::{Message, MessageTag};
    ///
    /// let message = Message::new(MessageTag::Bitfield, vec![0b10101010]).unwrap();
    /// assert_eq!(message.tag(), &MessageTag::Bitfield);
    /// assert_eq!(message.payload(), &[0b10101010]);
    /// ```
    pub fn new(tag: MessageTag, payload: Vec<u8>) -> anyhow::Result<Self> {
        match tag {
            // If the message has a tag that does not require a payload, the payload must be empty.
            MessageTag::Choke
            | MessageTag::UnChoke
            | MessageTag::Interested
            | MessageTag::NotInterested => {
                if !payload.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Message tag {:?} does not require a payload.",
                        tag
                    ));
                }
                Ok(Self {
                    tag,
                    payload: Vec::new(),
                })
            }
            // TODO: Add more checks for other message tags.
            _ => Ok(Self { tag, payload }),
        }
    }
    /// Creates a new message from message tag, without payload.
    ///
    /// The messages that do not require a payload are: `Choke`, `UnChoke`, `Interested`, and
    /// `NotInterested`.
    ///
    /// # Errors
    ///
    /// If the message tag requires a payload, an error is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use ltorrent::net::message::{Message, MessageTag};
    ///
    /// let message = Message::without_payload(MessageTag::Choke).unwrap();
    /// assert_eq!(message.tag(), &MessageTag::Choke);
    /// assert_eq!(message.payload(), &[]);
    /// ```
    pub fn without_payload(tag: MessageTag) -> anyhow::Result<Self> {
        match tag {
            MessageTag::Choke
            | MessageTag::UnChoke
            | MessageTag::Interested
            | MessageTag::NotInterested => Ok(Self {
                tag,
                payload: Vec::new(),
            }),
            _ => Err(anyhow::anyhow!("Message tag {:?} requires a payload.", tag)),
        }
    }

    /// Returns the tag of the message.
    pub fn tag(&self) -> &MessageTag {
        &self.tag
    }

    /// Returns the payload of the message.
    pub fn payload(&self) -> &[u8] {
        self.payload.as_slice()
    }
}

/// Represents the different types of messages exchanged between peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageTag {
    /// No payload. Indicates that the sender will not send any more data.
    Choke = 0,
    /// No payload. Indicates that the sender is ready to send data.
    UnChoke = 1,
    /// No payload. Indicates that the sender is interested in receiving data.
    Interested = 2,
    /// No payload. Indicates that the sender is not interested in receiving data.
    NotInterested = 3,
    /// The `Have` message's payload is a single number, the index which that downloader just
    /// completed and checked the hash of.
    Have = 4,
    /// `Bitfield` is only ever sent as the first message. Its payload is a bitfield with each
    /// index that downloader has sent set to one and the rest set to zero. Downloaders which don't
    /// have anything yet may skip the 'bitfield' message. The first byte of the bitfield
    /// corresponds to indices 0 - 7 from high bit to low bit, respectively. The next one 8-15, etc.
    /// Spare bits at the end are set to zero.
    Bitfield = 5,
    /// `Request` messages contain an index, begin, and length. The last two are byte offsets.
    /// Length is generally a power of two unless it gets truncated by the end of the file.
    /// All current implementations use 2^14 (16 kiB), and close connections which request an
    /// amount greater than that.
    Request = 6,
    /// `Piece` messages contain an index, begin, and block. Begin is the byte offset within the
    /// piece, and the block is the raw data. The maximum length is 2^14 (16 kiB). The one exception
    /// is the piece which ends the file, which may be shorter. Files are padded to a multiple of
    /// 2^14 bytes, but the file is not sent in the clear. It is hashed and then the hash is
    /// compared to the hash in the .torrent file. If it doesn't match, the downloader will close
    /// the connection.
    Piece = 7,
    /// `Cancel` messages have the same payload as request messages. They are generally only sent
    /// towards the end of a download, during what's called 'endgame mode'. When a download is
    /// almost complete, there's a tendency for the last few pieces to all be downloaded off a
    /// single hosed modem line, taking a very long time. To make sure the last few pieces come in
    /// quickly, once requests for all pieces a given downloader doesn't have yet are currently
    /// pending, it sends requests for everything to everyone it's downloading from. To keep this
    /// from becoming horribly inefficient, it sends cancels to everyone else every time a piece
    /// arrives.
    Cancel = 8,
}

impl TryFrom<u8> for MessageTag {
    type Error = std::io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Choke),
            1 => Ok(Self::UnChoke),
            2 => Ok(Self::Interested),
            3 => Ok(Self::NotInterested),
            4 => Ok(Self::Have),
            5 => Ok(Self::Bitfield),
            6 => Ok(Self::Request),
            7 => Ok(Self::Piece),
            8 => Ok(Self::Cancel),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Could not turn {} to MessageTag.", value),
            )),
        }
    }
}

pub struct MessageFramer;

impl Decoder for MessageFramer {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // The peer message consists of a message length prefix (4 bytes), a message id (1 byte),
        // and a message payload (variable length).
        if src.len() < 4 {
            // Not enough bytes to read the length marker.
            return Ok(None);
        }

        // Read length marker
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        if length == 0 {
            // This is a heartbeat message. Discard it (for now).
            // We advance till after the heartbeat message and try to decode again.
            src.advance(4);
            return self.decode(src);
        }

        if src.len() < 5 {
            // Not enough bytes to read the message tag.
            return Ok(None);
        }

        // Check that the length is not too large to avoid a denial of service attack where the
        // server runs out of memory.
        if length > MAX_MESSAGE_LENGTH {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Decoding: Frame of length {} is too large.", length),
            ));
        }

        if src.len() < 4 + length {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(4 + length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance to modify src such that it no longer contains
        // this frame.
        let tag = MessageTag::try_from(src[4])?;
        let data = if src.len() > 5 {
            src[5..4 + length].to_vec()
        } else {
            Vec::new()
        };
        src.advance(4 + length);

        Ok(Some(Message { tag, payload: data }))
    }
}

impl Encoder<Message> for MessageFramer {
    type Error = std::io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Don't send too long a message.
        if item.payload.len() > MAX_MESSAGE_LENGTH {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Encoding: Frame of length {} is too large.",
                    item.payload.len()
                ),
            ));
        }

        // Convert the length into a byte array.
        // The cast to u32 cannot overflow due to the length check above.
        let length_bytes = u32::to_be_bytes(item.payload.len() as u32 + 1);

        // Reserve space in the buffer.
        dst.reserve(
            4 /* length */ + 1 /* tag */ + item.payload.len(), /* payload */
        );

        // Write the length and string to the buffer.
        dst.extend_from_slice(&length_bytes);
        dst.put_u8(item.tag as u8);
        dst.extend_from_slice(item.payload.as_slice());
        Ok(())
    }
}
