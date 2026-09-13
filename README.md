# project-memory

Persistent, structured memory for AI coding agents. `pmem` stores a project's
conventions, decisions, patterns, and context in a local SQLite database and
serves them to any MCP-compatible agent (Claude Code, Cursor, and others) —
so an agent stops re-deriving the same context every session.

## Why

AI coding assistants re-read a codebase from scratch on every conversation.
Decisions explained once get explained again next week. `pmem` gives a
project a small, queryable memory: short, tagged entries an agent can search,
link, and pull in as context — checked into version control alongside the
code they describe.

## Features

- **Structured memories** — typed entries (convention, pattern, decision,
  preference, context) with tags and cross-references
- **Fuzzy and full-text search** — Jaro-Winkler ranking plus SQLite FTS5,
  with faceted and date-range filters
- **MCP integration** — stdio transport (Claude Code, Cursor) and an HTTP
  transport, both speaking the same JSON-RPC protocol
- **REST API and web dashboard** — browse and query memories outside the CLI
- **Snapshots** — save, diff, and restore the full memory state
- **Duplicate detection** — similarity-based merge suggestions
- **Encryption at rest** — AES-256-GCM with PBKDF2 key derivation for
  sensitive entries
- **Git hooks** — auto-export memories on commit
- **Code and markdown import** — pull in `TODO`/`FIXME` comments or existing
  docs as memories

## Installation

```bash
cargo install --path .
```

Requires a Rust toolchain (2021 edition).

## Quick start

```bash
pmem init                                              # create .memory/ in the current project
pmem add --kind convention -k naming -c "Use snake_case" --tags rust
pmem search naming
pmem stdio                                             # serve over MCP stdio
```

Point an MCP client at it:

```json
{
  "mcpServers": {
    "project-memory": {
      "command": "pmem",
      "args": ["stdio"]
    }
  }
}
```

## Usage

Every command has its own `--help`; run `pmem help` for the full list. The
most commonly used ones:

| Command | Description |
|---|---|
| `pmem add` | Add a new memory entry |
| `pmem list` / `pmem get <id>` | List or show memories |
| `pmem search <query>` | Fuzzy search |
| `pmem update` / `pmem delete` | Modify or remove an entry |
| `pmem link` / `pmem related <id>` | Cross-reference entries |
| `pmem snapshot save\|restore\|diff` | Save and restore memory state |
| `pmem dedupe find\|merge` | Find and merge duplicate entries |
| `pmem validate --fix` | Check memory quality and auto-fix issues |
| `pmem encryption set-key\|encrypt\|decrypt` | Encrypt sensitive entries |
| `pmem stdio` / `pmem serve` | Run the MCP server (stdio or HTTP) |
| `pmem dashboard` / `pmem api` | Run the web dashboard or REST API |

The CLI also covers tagging, groups, templates, batch operations, git-hook
integration, backups, and analytics — see `pmem help` for the complete
reference.

## MCP integration

`pmem stdio` speaks MCP over stdio; `pmem serve --port <p>` exposes the same
protocol over HTTP at `POST /mcp`. Both share one request handler, so
behavior is identical across transports. Available tools: `memory_add`,
`memory_search`, `memory_list`, `memory_get`, `memory_delete`,
`memory_context`, `memory_link`.

## REST API

```bash
pmem api --port 3779
```

Exposes memory CRUD and search over HTTP for integrations that don't speak
MCP.

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for how the CLI, storage
layer, and server transports fit together.

## Development

```bash
cargo build
cargo test
```

## License

MIT
