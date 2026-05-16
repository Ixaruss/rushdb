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

## Benchmarks

Benchmarked against Redis on the same machine using 1,000,000 requests, 200 concurrent clients, and 64-byte payloads. RushDB uses `RwLock` for concurrent reads — multiple GET operations run fully in parallel, which is the key architectural difference vs Redis's single-threaded execution model.

**Machine:** AMD Ryzen 5 5600H, 6 cores / 12 threads

### Throughput

<img width="1600" height="889" alt="Image" src="https://github.com/user-attachments/assets/5645f669-1484-49f3-a1cb-8385d62bc855" />

| | RushDB | Redis (single thread) | Redis (multi threaded) |
|---|---|---|---|
| SET | 136,934 ops/s | 97,373 ops/s | 158,035 ops/s |
| GET | **365,433 ops/s** | 97,442 ops/s | 168,548 ops/s |

### Latency

<img width="1600" height="800" alt="Image" src="https://github.com/user-attachments/assets/921a1f80-d1f0-43ce-9f59-075f8349e7a0" />

| | RushDB | Redis (single thread) | Redis (multi threaded) |
|---|---|---|---|
| p50 | **0.029ms** | 1.031ms | 1.151ms |
| p95 | **0.110ms** | 1.119ms | 1.431ms |
| p99 | **0.823ms** | 1.247ms | 2.439ms |
| max | **1.489ms** | 4.783ms | 8.087ms |

> Benchmarks run with snapshot persistence disabled. See `bench/` for the benchmark binary and full methodology.

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
cargo run -p db

# In another terminal, start the client shell
cargo run -p cli
```

### Run with Docker

```bash
docker build -t rushdb:0.1 .
docker run -p 6080:6080 --name=rushdb -d rushdb:0.1
```

---

### Benchmarking

```bash
cargo build -p Benchmark
benchmark

# via docker
docker exec -it rushdb /bin/bash
benchmark
```

---

## Usage

### Interactive Shell

```bash
cli
# or to build it from source
cargo run -p cli
# or via docker
docker exec -it rushdb /bin/bash
cli
```

### One-shot CLI

```bash
# Set a key
cli set name Alice

# Get a key
cli get name

# Check existence
cli exists name

# Delete
cli del name

# Total keys
cli total
```

---

## Configuration

All server settings are in `config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 6080
save_interval = 60        # seconds between snapshots
save_path = "./data/snapshot.bin"
save_dir = "./data"
```

Override config path via environment variable:

```bash
CONFIG_PATH=/etc/RushDB/config.toml ./rushdb
# or via docker
docker cp /path/to/config rushdb:/etc/
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

## Author

Built in a day. Bugs fixed the same day.
