/// The `BitField` struct is used to track which pieces of the torrent a peer has.
/// It stores this information as a vector of bytes (`Vec<u8>`), where each bit
/// represents the presence (1) or absence (0) of a corresponding piece.
///
/// # Examples
///
/// ```
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
}