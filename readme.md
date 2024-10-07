# Leech Torrent

## Parse torrent file

Aa torrent file, also known as metainfo file, contains the information about what the torrent contains, and where we
retrieve the torrent from. Specifically, the metainfo file contains a dictionary with the following keys and values (all
strings in a .torrent file that contains text must be UTF-8 encoded):

- `announce`: URL to a "tracker", which is a central server that keeps track of peers participating in the sharing of
  a torrent.
- `info`: A dictionary with keys:
    - `length`: size of the file in bytes, for single-file torrents.
    - `name`: suggested name to save the file or directory.
    - `piece length`: the number of bytes in each piece the file is split into. For the purposes of transfer, files
      are split into fixed-size pieces which are all the same length except for possibly the last one, which may be
      truncated. `piece length` is almost always a power of two, most commonly 2^18 = 256k.
    - `pieces`: maps to a string whose length is a multiple of 20. It is to be subdivided into strings of length 20,
      each of which is the SHA1 hash of the piece at the corresponding index.

  The `info` dictionary looks slightly different for multi-file torrents. The dictionary contains a key `length` or
  a key `files`, but not both or neither. If `length` is present then the download represents a single file, or
  otherwise it represents a set of files which go in a directory structure.
    - `length`: In the single file case, `length` maps to the length of the file in bytes. In this case, the
      key `name` is the name of the file.
    - `files`: For the purposes of the previous keys, the multi-file case is treated as only having a single file by
      concatenating the fields in the order they appear in the files list. The files list is the value `files` maps
      to, and is a list of dictionaries containing the following keys:
        - `length`: The length of the file, in bytes.
        - `path`: A list of strings corresponding to subdirectory names, the last of which is the actual file name (
          a zero length list is an error case). The key `name` is the name of the top directory.

To parse a torrent file, simply run:

```shell
ltorrent info <PATH>
```

## Get peers from tracker

Trackers are central servers that maintain information about peers participating in the sharing and downloading of a
torrent. To discover peers to download the file from, we make a GET request to an HTTP tracker. The get request has to
include the following query parameters:

- `info_hash`: The info hash of the torrent.
- `peer_id`: A unique identifier for your client. A string of length 20 that you get to pick.
- `port`: The port your client is listening on.
-

# Roadmap

- [ ] Torrent File
    - [x] Parse torrent file
    - [ ] Builder
    - [ ] From Magnet
    - [ ] Tests
- [ ] Messages
    - [ ] Checks when creating a message for each message type.
    - [ ] Tests
- [ ] Tracker
    - [ ] HTTP Tracker
    - [ ] DHT Tracker
    - [ ] Tests
- [ ] Peer
    - [ ] PeerConnection async Trait
    - [ ] Peer Builder
    - [ ] Tests
- [ ] Piece
    - [ ] Download piece: Download concurrently blocks from different peers.
    - [ ] Tests
- [ ] Commands
- [ ] UI