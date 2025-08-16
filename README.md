# data-cluster 🚀

A small, async, sharded key–value store written in Rust. It showcases consistent hashing, actor‑like shards, backpressure with bounded queues, and a simple line‑based TCP protocol.

- Language/runtime: Rust + Tokio ⚙️
- Protocol: plain text over TCP 🔌
- Storage: in‑memory per shard 🧠

Server binds to 127.0.0.1:8080 and supports: GET, SET, DELETE, LIST.

## ✨ Features

- Consistent hashing ring maps keys to shards
- Backpressure: requests return BUSY if queues are full
- Simple protocol for easy testing via netcat
- Clear separation of concerns: router, shard, storage

## 🧭 Quick start

Prerequisites: Rust toolchain (stable) with Cargo

Build and run:

- Debug: `cargo run`
- Release: `cargo build --release && target/release/data-cluster`

You should see:

```
Server listening on 127.0.0.1:8080 — commands: GET <key> | SET <key> <value> | DELETE <key> | LIST
```

Interact with the server (new terminal):

```
$ nc 127.0.0.1 8080
SET user1 alice
OK
GET user1
alice
LIST
user1
DELETE user1
OK
GET user1
ERR key not found
```

Notes:

- Values are single tokens (spaces are not supported in values) ✍️
- Under load you might get `BUSY` (backpressure) 🧯

## 🔤 Protocol (TCP, one command per line)

- GET <key> → `<value>` | `ERR key not found` | `BUSY`
- SET <key> <value> → `OK` | `BUSY`
- DELETE <key> → `OK` | `BUSY`
- LIST → comma‑separated list of keys (or empty line) | `BUSY`

Examples:

```
SET foo bar
GET foo
DELETE foo
LIST
```

## 🧱 Architecture

```
Client ↔ TCP Server (main.rs) → Router (router.rs) → Shards (shard.rs) → Storage (storage.rs)
                         ↑                                 ↑
                         └──────────── mpsc (pipe.rs) ─────┘
```

- Hash ring (`cluster.rs`): BTreeMap<u64, Shard> using DefaultHasher; N virtual replicas per node
- Router (`router.rs`): picks shard via ring, forwards requests with try_send, maps shard replies
- Shard (`shard.rs`): processes commands against its storage; responds via oneshot
- Storage (`storage.rs`): in‑memory `HashMap<String, String>` behind `RefCell`
- Channels (`pipe.rs`): typed request/response wrapper over Tokio mpsc + oneshot

Backpressure: both router and shards use `try_send`. If a queue is full, the client sees `BUSY`.

## ⚙️ Configuration

Defaults in `src/main.rs`:

- Replicas in hash ring: `HashRing::new(64)`
- Shard count: 3 (spawned in a loop)
- Listen address: `127.0.0.1:8080`
- Queue sizes: shard=1024, router=2048

Change those constants in code to tune topology and capacity.

## 🗂️ Project layout

```
src/
  main.rs      # TCP server; parses protocol; spawns router + shards
  lib.rs       # Module declarations
  cluster.rs   # Hash ring + ClusterTrait
  pipe.rs      # Request/response channel helpers
  router.rs    # Router and request fan‑out/aggregation
  shard.rs     # Shard actor + command handling
  storage.rs   # In‑memory key–value store (StorageTrait)
```

## 🛠️ Development

- Run: `cargo run`
- Format: `cargo fmt`
- Lint: `cargo clippy --all-targets -- -D warnings`

## 🔌 Extending

- New storage backend:
  1. Implement `StorageTrait` for your type
  2. In `main.rs`, replace `Storage::new()` with your implementation when spawning shards
- Change topology:
  - Adjust shard count in the spawn loop in `main.rs`
  - Tweak replicas via `HashRing::new(<replicas>)`
- Add a new command:
  1. Extend `ShardCmd` and its handling in `shard.rs`
  2. Extend `RouterRequest`/`RouterResponse` and routing in `router.rs`
  3. Extend command parsing and responses in `main.rs`

## 🧰 Troubleshooting

- BUSY: queues are full → retry later or increase buffer sizes in `main.rs`
- ERR key not found: GET for a missing key → ensure you SET first
- Port in use: change `addr` in `main.rs`

## 📜 Changelog (Git history)

A human‑friendly summary of notable commits. Dates and hashes come from `git log`.

- 2025-08-12 — 2e409eb — Seed: consistent hashing ring 🌱
  - Introduced `HashRing` and `ClusterTrait`; key→shard mapping via 64‑bit hash.
- 2025-08-12 — 0e971ac — Networking: TCP server 🔌
  - Basic line‑based protocol over a TCP listener and accept loop.
- 2025-08-14 — 4836efb — Refactor: ring and storage 🔧
  - Improved ring node handling, reduced cloning; expanded `StorageTrait` (get/set/delete/list).
- 2025-08-14 — cd2bbbc — Async: integrate Tokio ⚙️
  - Switched to async I/O and lightweight tasks; per‑connection handling.
- 2025-08-14 — d99693c — Router layer + backpressure 🚦
  - Added `RouterRequest`/`RouterResponse`; `try_send` with `BUSY` semantics.
- 2025-08-16 — f77c569 — Shards: command/response and per‑shard backpressure 📦
  - Implemented `ShardCmd`/`ShardResponse`, oneshot replies, and `LIST` aggregation.
- 2025-08-16 — 76ffa97 — Refactor: cluster and shard API 🧩
  - Cleaner node management, shard naming, and `Shard::send` ergonomics.
- 2025-08-16 — 84e1210 — Router: concurrency + BUSY mapping 🚀
  - Spawn per request, map shard `Busy` to router `ShardBusy`/`Busy`, better error propagation.

Tip:
```
# Full history in chronological order
git log --reverse --date=short --pretty=format:'%h %ad %s'

# Inspect a specific change
git show <hash>
```

— Happy hacking! ✨
