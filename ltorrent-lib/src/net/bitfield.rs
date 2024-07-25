/// Represents the bitfield of a peer in a torrent swarm.
///
/// The `BitField` struct is used to track which pieces of the torrent a peer has.
/// It stores this information as a vector of bytes (`Vec<u8>`), where each bit
/// represents the presence (1) or absence (0) of a corresponding piece.
///
/// # Examples
///
/// ```
/// use ltorrent::net::bitfield::BitField;
///
/// let bitfield = BitField::from_payload(vec![0b10101010, 0b01010101]);
/// assert!(bitfield.contains_piece(0)); // The peer has the first piece.
/// assert!(!bitfield.contains_piece(7)); // The peer does not have the eighth piece.
/// assert!(bitfield.contains_piece(9)); // The peer has the ninth piece.
/// ```
pub struct BitField {
    pub payload: Vec<u8>,
}

impl BitField {
    /// Creates a new `BitField` instance from a given payload.
    ///
    /// This function takes a vector of bytes (`Vec<u8>`) as input, where each byte represents
    /// 8 pieces of a torrent. Each bit in a byte indicates the presence (1) or absence (0)
    /// of a corresponding piece.
    ///
    /// # Examples
    ///
    /// ```
    /// use ltorrent::net::bitfield::BitField;
    ///
    /// let payload = vec![0b10101010, 0b01010101];
    /// let bitfield = BitField::from_payload(payload);
    /// assert!(bitfield.contains_piece(0)); // The peer has the first piece.
    /// assert!(!bitfield.contains_piece(1)); // The peer does not have the second piece.
    /// ```
    pub fn from_payload(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    /// Checks if a specific piece is present in the bitfield.
    ///
    /// This method determines whether a particular piece, identified by its index (`piece_i`),
    /// is present in the bitfield. It calculates the corresponding byte and bit positions
    /// within the `payload` vector to check the presence of the piece. If the piece is present,
    /// the method returns `true`; otherwise, it returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ltorrent::net::bitfield::BitField;
    ///
    /// let bitfield = BitField::from_payload(vec![0b10101010, 0b01010101]);
    /// assert!(bitfield.contains_piece(0)); // The peer has the first piece.
    /// assert!(!bitfield.contains_piece(7)); // The peer does not have the eighth piece.
    /// ```
    pub fn contains_piece(&self, piece_i: usize) -> bool {
        let byte_i = piece_i / 8;
        let bit_i = piece_i % 8;
        let Some(byte) = self.payload.get(byte_i) else {
            return false;
        };
        byte & 1_u8.rotate_right((bit_i + 1) as u32) != 0
    }
}

impl<'a> IntoIterator for &'a BitField {
    type Item = usize;
    type IntoIter = BitFieldIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BitFieldIter {
            bitfield: self,
            byte_i: 0,
            bit_i: 0,
        }
    }
}

/// Represents an iterator over the pieces in a `BitField`.
///
/// This struct is used to iterate over each piece in the `BitField`, providing the index
/// of each piece that is present. It iterates over the bytes of the `BitField`'s payload,
/// and for each byte, it checks each bit to determine if a piece is present.
///
/// # Examples
///
/// ```
/// use ltorrent::net::bitfield::BitField;
///
/// let bitfield = BitField::from_payload(vec![0b10101010, 0b01010101]);
/// let mut iterator = bitfield.into_iter();
///
/// assert_eq!(iterator.next(), Some(0)); // The first piece is present.
/// assert_eq!(iterator.next(), Some(2)); // The third piece is present.
/// assert_eq!(iterator.next(), Some(4)); // The fifth piece is present.
/// ```
pub struct BitFieldIter<'a> {
    bitfield: &'a BitField,
    byte_i: usize,
    bit_i: usize,
}

impl<'a> Iterator for BitFieldIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.byte_i < self.bitfield.payload.len() {
            let byte = self.bitfield.payload[self.byte_i];
            while self.bit_i < u8::BITS as usize {
                let piece_i = self.byte_i * (u8::BITS as usize) + self.bit_i;
                let mask = 1_u8.rotate_right((self.bit_i + 1) as u32) as usize;
                self.bit_i += 1;
                if (byte as usize) & mask != 0 {
                    return Some(piece_i);
                }
            }
            self.byte_i += 1;
            self.bit_i = 0;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_piece() {
        let bitfield = BitField::from_payload(vec![0b10101010, 0b01010101]);
        assert!(bitfield.contains_piece(0));
        assert!(!bitfield.contains_piece(1));
        assert!(!bitfield.contains_piece(7));
        assert!(!bitfield.contains_piece(8));
        assert!(bitfield.contains_piece(15));
    }

    #[test]
    fn test_bitfield_iterator() {
        let bitfield = BitField::from_payload(vec![0b10101010, 0b01010101]);
        let mut iter = bitfield.into_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next(), Some(9));
        assert_eq!(iter.next(), Some(11));
        assert_eq!(iter.next(), Some(13));
        assert_eq!(iter.next(), Some(15));
        assert_eq!(iter.next(), None);
    }
}
