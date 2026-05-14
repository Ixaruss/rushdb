# RushDB

> A fast, lightweight, persistent in-memory key-value store with a TCP server, binary protocol, and interactive CLI — version `0.1.0`

---

## Overview

`RushDB` is a Redis-inspired key-value store built from scratch. It features a custom binary TCP protocol, async multi-client server, snapshot-based persistence, and a full-featured interactive shell — all split across two independent binaries: a **server** and a **CLI client**.

This is a systems-level project focused on understanding how in-memory stores work at a low level — protocol design, client-server architecture, shared mutable state, and persistence. Rust is the implementation language.

---

## Features

- ⚡ **In-memory storage** via `HashMap` for O(1) lookups
- 🔌 **TCP server** with async multi-client support via `tokio`
- 📦 **Custom binary protocol** with length-prefixed strings
- 💾 **Snapshot persistence** — periodic saves to disk, replayed on startup
- 🖥️ **Interactive REPL** with tab completion and command hinting
- 🧰 **CLI interface** via `clap` for one-shot commands
- 🐳 **Docker support** — server and client as separate containers

---

## Architecture

```
RushDB/
├── /                  
│        src/                # db binary
│       ├── main.rs          # Entry point
│       ├── engine.rs        # KvStore — in-memory HashMap
│       ├── persistence.rs   # Snapshot save/load
│       ├── server.rs        # TCP listener, request dispatch
│       └── libr.rs          # Server-side protocol (read request, send response)
│
├── Rushcli/                 # cli binary
│   └── src/
│       ├── main.rs          # Entry point
│       ├── client.rs        # Query abstraction
│       ├── cmd.rs           # clap CLI + command dispatch
│       ├── shell.rs         # Interactive REPL
│       ├── util.rs          # Tab completion, hinting
│       └── libc.rs          # Client-side protocol (send request, read response)
│
├── config.toml              # Configuration file
├── docker-compose.yml
├── Dockerfile
└── README.md
```

---

## Protocol

`RushDB` uses a custom binary protocol over TCP — not JSON, not HTTP.

### Request Format

```
[1 byte: op] [4 bytes: key_len] [N bytes: key] [4 bytes: val_len?] [N bytes: val?]
```

### Response Format

```
[1 byte: status] [4 bytes: val_len] [N bytes: val]
```

### Opcodes

| Op | Byte | Description |
|---|---|---|
| `GET` | `0x00` | Fetch value by key |
| `SET` | `0x01` | Insert or overwrite key |
| `DEL` | `0x02` | Delete key |
| `EXISTS` | `0x03` | Check if key exists |
| `TOTAL` | `0x04` | Return total key count |

### Status Codes

| Status | Byte |
|---|---|
| `Ok` | `0x00` |
| `NotFound` | `0x01` |
| `Error` | `0x02` |

---

## Getting Started

### Prerequisites

- Rust `1.75+`
- Cargo
- Docker + Docker Compose (optional)

### Build & Run (local)

```bash
# Clone the repo
cd RushDB

# Start the server
cargo run

# In another terminal, start the client shell
cd cli
cargo run -p rushcli
```

### Run with Docker

```bash
docker compose up
```

---

## Usage

### Interactive Shell

```bash
rushcli

# or to build it from source
cargo run -p rushcli
```

```
Type 'help' for commands or 'exit' to quit.
version: 0.1.0
cli> set name Alice
DONE
cli> get name
Alice
cli> exists name
true
cli> total
1
cli> del name
DONE
cli> get name
DONT EXISTS
cli> exit
```

### One-shot CLI

```bash
# Set a key
rushcli set name Alice

# Get a key
rushcli get name

# Check existence
rushcli exists name

# Delete
rushcli del name

# Total keys
rushcli total
```

---

## Configuration

All server settings are in `config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080
save_interval = 10        # seconds between snapshots
save_path = "./data/snapshot.bin"
save_dir = "./data"
```

Override config path via environment variable:

```bash
CONFIG_PATH=/etc/RushDB/config.toml ./rushdb
```

---

## Persistence

`RushDB` uses **snapshot-based persistence**:

- Every `save_interval` seconds, the full in-memory store is serialized with `bincode` and written to `snapshot.bin`
- On server startup, the snapshot is deserialized and loaded into memory
- If no snapshot exists or it is corrupt, the server starts with an empty store

> ⚠️ Data written between the last snapshot and a crash is lost. WAL (write-ahead log) based persistence is planned for `v0.2.0`.

---

## Roadmap

| Version | Features |
|---|---|
| `v0.1.0` | Core CRUD, TCP server, binary protocol, snapshot persistence, REPL |
| `v0.2.0` | WAL persistence, TTL / key expiry, `KEYS` pattern matching |
| `v0.3.0` | Pub/Sub, IO multiplexing with `mio` |
| `v0.4.0` | Shared `protocol` crate, workspace restructure |

---

## Dependencies

### Server
| Crate | Purpose |
|---|---|
| `tokio` | Async runtime, TCP, timers |
| `serde` + `bincode` | Snapshot serialization |

### Client
| Crate | Purpose |
|---|---|
| `tokio` | Async TCP |
| `clap` | CLI argument parsing |
| `rustyline` | REPL with readline support |

---

## License

MIT — see [`LICENSE`](LICENSE)

---

## Author

Built in a day. Bugs fixed the same day.
