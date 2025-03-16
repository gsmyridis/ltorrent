use std::cmp::{Ord, Ordering, PartialOrd};

use crate::torrent::Torrent;
use crate::tracker::TrackerResponse;

///
#[derive(Debug, PartialEq, Eq)]
pub struct Piece {
    peers: Vec<usize>,
    piece_i: usize,
    length: usize,
    hash: [u8; 20],
    seed: u64,
}

impl Ord for Piece {
    fn cmp(&self, other: &Self) -> Ordering {
        self.peers.len().cmp(&other.peers.len())
            .then(self.seed.cmp(&other.seed))
            .then(self.hash.cmp(&other.hash))
            .then(self.peers.cmp(&other.peers))
            .then(self.piece_i.cmp(&other.piece_i))
    }
}

impl PartialOrd for Piece {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl Piece {
    pub fn new(piece_i: usize, torrent: &Torrent, peers: &TrackerResponse) -> Self {}
}