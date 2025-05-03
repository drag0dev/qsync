# qsync
A fast and lightweight tool for local file synchronization—ideal when you do not need encryption.
Qsync is inspired by rsync and leverages the performance of QUIC for efficient data transfer.

## Features
- Delta sync
- Sync files, directories and symlinks
- Symlinks are synced as-is (target path validity is not verified)
- Optionally preserve timestamps and permissions
- Naive sync mode: only transfers files if size or modified time differs
- Powered by QUIC
- Certificate verification is disabled (therefore intended for local use only)

## Usage

### Client
Usage: 
```
client [OPTIONS] --server-address <SERVER_ADDRESS> --remote-target <REMOTE_TARGET> --local-target <LOCAL_TARGET>  
```
Options:
```
  -s, --server-address <SERVER_ADDRESS>      Server address
  -p, --port <PORT>                          Server port [default: 4433]
  -r, --remote-target <REMOTE_TARGET>        Remote directory path
  -l, --local-target <LOCAL_TARGET>          Local directory path to sync with remote
  -c, --concurrent-streams <CONCURRENT_STREAMS>  Number of QUIC streams to use [default: 25]
  -t, --timestamp                            Preserve timestamps from remote
      --permissions                          Preserve file permissions
  -n, --naive                                Skip files if size and modified time match
  -b, --block-size <BLOCK_SIZE>              Block size in bytes for diffing [default: 16384]
  -v, --verbose                              Enable verbose output
  -h, --help                                 Show help
  -V, --version                              Show version
```

### Server
Options:
```
  -p, --port <PORT>  [default: 4433]
  -h, --help         Print help
```

## Installation

Ensure you have Rust installed.

```
git clone https://github.com/drag0dev/qsync.git
cd qsync
```

To install the server (binary name: qsync_server):
```
cargo install --path server
```
To install the client (binary name: qsync_client):
```
cargo install --path client
```
