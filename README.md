# project-memory

Context-aware project memory for AI agents - CLI + MCP server

## Location

```
~/projects/project-memory/
```

## Build Status

| Check | Result |
|-------|--------|
| `cargo build --release` | Success (7s) |
| `cargo test` | 13/13 passed |
| `cargo clippy` | 2 minor warnings |
| Binary | 8.0 MB |

## Installation

```bash
cd ~/projects/project-memory
cargo install --path .
```

## Quick Start

```bash
pmem init
pmem template apply rust-lib
pmem add --kind convention -k "naming" -c "Use snake_case" --tags rust
pmem stdio
```

## Commands (55+)

### Core
- `pmem init` - Initialize
- `pmem add` - Add memory
- `pmem add-interactive` - Add with prompts
- `pmem list` - List memories
- `pmem search <query>` - Fuzzy search
- `pmem get <id>` - Show memory
- `pmem update <id>` - Update
- `pmem delete <id>` - Delete

### Organization
- `pmem pin/unpin/pinned` - Pin memories
- `pmem link/unlink/related` - Link memories
- `pmem group create/list/show/add/remove` - Groups
- `pmem tags list/rename/delete/add/stats` - Tags

### Templates
- `pmem template list/show/apply` - 6 templates

### Snapshots
- `pmem snapshot save/list/show/diff/restore`

### Duplicates
- `pmem dedupe find/merge/merge-group`

### Git Integration
- `pmem hooks install/uninstall/status`

### Code Import
- `pmem import-code` - Import TODO/FIXME
- `pmem import-md <file>` - Import markdown

### Encryption
- `pmem encryption set-key/remove-key/status/encrypt/decrypt`

### Sync
- `pmem sync <path> --direction push/pull/both`

### Analytics
- `pmem analytics` - Show analytics report
- `pmem analytics --format json` - JSON output

### Documentation
- `pmem docs --format markdown/html` - Generate docs

### Search
- `pmem faceted-search --kind convention --tags rust --pinned true`

### Versioning
- `pmem versions <id>` - Show version history
- `pmem restore <id> <version>` - Restore version

### Servers
- `pmem stdio` - MCP stdio
- `pmem serve` - MCP HTTP
- `pmem api` - REST API
- `pmem dashboard` - Web UI

### Other
- `pmem search <query> --exact` - Exact search
- `pmem date-search --after/--before` - Date range
- `pmem batch delete/tag/kind` - Batch ops
- `pmem merge <path>` - Merge from project
- `pmem validate` - Quality checks
- `pmem graph --format dot/json` - Export graph
- `pmem history` - Audit log
- `pmem stats` - Statistics
- `pmem browse` - TUI
- `pmem config show/set/reset` - Config
- `pmem completions <shell>` - Shell completions
- `pmem watch` - Watch mode

## MCP Integration

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

## REST API

```bash
pmem api --port 3779
```

15 endpoints at `/api/memories`, `/api/search`, `/api/stats`, etc.

## Source (22 modules, 6000+ lines)

```
src/
├── main.rs         CLI (1800+ lines)
├── store.rs        SQLite + search + versioning (700+ lines)
├── mcp.rs          MCP HTTP
├── mcp_stdio.rs    MCP stdio
├── api.rs          REST API (280+ lines)
├── tui.rs          TUI
├── dashboard.rs    Web UI
├── templates.rs    Templates
├── analytics.rs    Analytics
├── docs.rs         Documentation generator
├── snapshot.rs     Snapshots
├── duplicates.rs   Duplicates
├── hooks.rs        Git hooks
├── encryption.rs   Encryption
├── markdown.rs     Markdown import
├── code_import.rs  Code comments import
├── graph.rs        Graph export
├── audit.rs        Audit log
├── config.rs       Config
├── types.rs        Types
├── git.rs          Git detection
└── lib.rs          Module exports
```

## Features

| Feature | Description |
|---------|-------------|
| Fuzzy Search | Jaro-Winkler similarity scoring |
| Faceted Search | Filter by kind, tags, pinned, date |
| Versioning | Track changes per memory |
| Analytics | Health score, kind distribution, tag stats |
| Doc Generation | Markdown/HTML from memories |
| Encryption | XOR-based encryption for sensitive data |
| Git Hooks | Auto-export on commit |
| Code Import | Scan TODO/FIXME from source |
| Duplicate Detection | Find and merge similar memories |
| Groups | Organize memories into groups |
| Pinning | Pin important memories |
| Snapshots | Save/restore/diff states |
| MCP Integration | stdio + HTTP for AI agents |
| REST API | 15 HTTP endpoints |
| Web Dashboard | Browser-based UI |
| TUI | Interactive terminal UI |

## License

MIT
