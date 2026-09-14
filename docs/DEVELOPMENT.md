# Local development

## Without Docker

```bash
cargo build
cargo test
cargo run -- init
cargo run -- dashboard --port 3778
```

## With Docker

`docker-compose.yml` builds one image and runs `pmem`'s three network-facing
servers against the repo checked out on your host, bind-mounted at
`/workspace`:

| Service | Command | Port |
|---|---|---|
| `dashboard` | `pmem dashboard` | 3778 |
| `api` | `pmem api` | 3779 |
| `mcp` | `pmem serve` (MCP over HTTP) | 3777 |

```bash
docker compose up --build
```

- Dashboard: http://localhost:3778
- REST API: http://localhost:3779/api/health
- MCP HTTP: http://localhost:3777/mcp

Each container's entrypoint runs `pmem init` against `/workspace` on
startup (a no-op if `.memory/` already exists), so all three share one
`.memory/store.db` that lands directly in your checked-out repo — already
covered by `.gitignore`, inspectable with the CLI on your host, and
resettable with `rm -rf .memory`.

Run a one-off CLI command against the same data without starting a server:

```bash
docker compose run --rm dashboard add --kind convention -k naming -c "Use snake_case"
```

To point the containers at a different project instead of this repo, change
the bind mount in `docker-compose.yml` from `.:/workspace` to the path you
want.

### Notes

- The servers bind to `127.0.0.1` by default (safe for plain local use —
  nothing on your LAN can reach them). `docker-compose.yml` sets
  `PMEM_HOST=0.0.0.0` for each service so Docker's port mapping can
  actually reach the process inside the container; a bare `127.0.0.1`
  bind is only visible from loopback *inside* that container's own
  network namespace, not from the host via `-p`, even though the process
  starts up looking fine. Set `PMEM_HOST` yourself if you need these
  servers reachable from another machine outside Docker.
- Containers run as root — this setup is for local development, not
  production deployment.
- The image does a full `cargo build --release` with no dependency-layer
  caching, so the first build takes a few minutes (native compilation of
  SQLite and the crypto stack). `add when`: swap in `cargo-chef` if
  rebuild-on-every-source-change starts to hurt.
- `pmem stdio` isn't a Docker service — it's a one-shot process an MCP
  client (Claude Code, Cursor) spawns and talks to over stdin/stdout, not a
  long-running network server. Run it directly: `pmem stdio`.
