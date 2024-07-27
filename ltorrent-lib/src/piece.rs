pub struct Piece {
    peers: Vec<usize>,
    piece_i: usize,
    length: usize,
    hash: [u8; 20],
    seed: u64,
}