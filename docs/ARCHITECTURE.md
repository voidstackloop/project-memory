# Architecture

`pmem` is a single Rust binary with three layers: a CLI, a SQLite-backed
storage layer, and a set of transports that expose the same store to
different clients.

## Storage

`MemoryStore` (`src/store.rs`) wraps a single SQLite connection per project,
stored at `.memory/store.db`. Core tables:

- `memories` — the entries themselves (kind, key, content, tags,
  related_ids, timestamps)
- `memory_links` — explicit cross-references between memories
- `memory_versions` — version history for edited entries
- `audit_log` — an append-only record of add/update/delete/restore actions

Multi-row operations that must succeed or fail together (duplicate merges,
snapshot restore) run inside a single SQLite transaction rather than as a
loop of independent statements.

## CLI

`src/main.rs` parses arguments with `clap` and dispatches to the storage
layer and supporting modules (search, validation, analytics, export,
snapshots, backups, encryption, and others — one module per concern under
`src/`).

## Transports

The same `MemoryStore` is exposed three ways, all built on top of it rather
than duplicating its logic:

- **MCP stdio** (`src/mcp_stdio.rs`) — JSON-RPC over stdin/stdout, the
  primary integration path for Claude Code, Cursor, and similar agents
- **MCP HTTP** (`src/mcp.rs`) — the same JSON-RPC handler
  (`mcp_stdio::handle_request_line`) exposed over `POST /mcp` via `axum`,
  so both transports implement the protocol once
- **REST API** (`src/api.rs`) and **web dashboard** (`src/dashboard.rs`) —
  plain HTTP/JSON and an HTML UI for browsing memories outside an MCP client

## Security

Sensitive memory content can be encrypted at rest with AES-256-GCM
(`src/encryption.rs`); the key is derived from a user-supplied passphrase via
PBKDF2-HMAC-SHA256, with a random salt and nonce per encryption. Encrypted
content is stored as `ENC:<hex>`.
