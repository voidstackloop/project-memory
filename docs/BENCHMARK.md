# Benchmark

Four questions this answers: does `pmem` work as an MCP server for a real
agent, does it actually save tokens over the alternative, is it fast, and
what does it cost in disk/memory/startup time. All measured live against the
`v0.2.0` release build — nothing here is estimated or simulated.

## 1. Real client integration

[OpenCode](https://opencode.ai) — an independent, third-party coding agent —
was pointed at `pmem stdio` via a standard `opencode.json` MCP config:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "project-memory": {
      "type": "local",
      "command": ["/path/to/pmem", "stdio"],
      "enabled": true
    }
  }
}
```

```
$ opencode mcp list

┌  MCP Servers
│
●  ✓ project-memory  connected
│      /path/to/pmem stdio
│
└  1 server(s)
```

OpenCode discovers and connects to the server on its own, using nothing but
the MCP protocol — no custom integration code on either side. This is
protocol-level proof, not a self-test: the client implementation is entirely
independent of `pmem`'s.

Driving raw JSON-RPC at that same process confirms the tools work, not just
the handshake:

```
tools/list -> memory_add, memory_search, memory_list, memory_get,
              memory_delete, memory_context, memory_link

tools/call memory_search {"query": "database"}
-> Found 3 memories:
   [decision] database: Use SQLite via rusqlite bundled feature... (score: 1.20)
   [convention] naming: Use snake_case for all Rust identifiers... (score: 0.43)
   [pattern] error-handling: Use anyhow::Result for fallible functions... (score: 0.39)

tools/call memory_add {"kind": "context", "key": "opencode-verified", ...}
-> Stored memory [context] opencode-verified: 3527c81c-0f98-4578-8757-6affd0ba46df

tools/call memory_context {}
-> # Project Memory Context
   ## Conventions
   - naming [rust, style]: Use snake_case for all Rust identifiers...
   ## Context
   - opencode-verified [opencode, benchmark]: This memory was written by
     OpenCode's MCP client via pmem stdio.
```

Fuzzy search correctly ranks the exact-key match above partial content
matches; a memory written by one tool call is immediately visible to another
in the same session.

## 2. Token usage: memory vs. re-reading the source

This is the actual value proposition, measured rather than asserted: 12 real
facts about *this* codebase — conventions and decisions established while
building it (encryption design, mutex-poisoning recovery, the atomic-merge
transaction pattern, the Docker bind-address fix, and so on) — stored as
`pmem` memories, versus the token cost of an agent reading the 8 source
files that actually contain those same facts.

Reproduce: `python3 scripts/token_benchmark.py` (needs `pip install
tiktoken`; falls back to a chars/4 estimate without it).

| | Tokens | vs. reading source |
|---|---:|---:|
| `memory_context` (all 12 facts, one call) | 508 | **67x cheaper** |
| `memory_search` (one targeted query) | 156 | **219x cheaper** |
| Reading `main.rs` + `store.rs` + `encryption.rs` + `mcp.rs` + `mcp_stdio.rs` + `api.rs` + `dashboard.rs` + `backup.rs` | 34,177 | — |

Per-file breakdown of the "reading source" side:

| File | Tokens | Lines |
|---|---:|---:|
| `src/main.rs` | 19,966 | 2,571 |
| `src/store.rs` | 5,070 | 251 |
| `src/mcp_stdio.rs` | 3,632 | 482 |
| `src/dashboard.rs` | 1,994 | 177 |
| `src/api.rs` | 1,127 | 123 |
| `src/backup.rs` | 1,142 | 125 |
| `src/encryption.rs` | 810 | 61 |
| `src/mcp.rs` | 436 | 42 |

This isn't 12 facts vs. 12 lines of grep — an agent without stored memory
doesn't know in advance which 12 lines matter, so the honest comparison is
against the files those facts actually live in. The gap only widens as a
project grows: the memory set stays roughly linear in the number of facts
you choose to record, while the source you'd otherwise have to read grows
with the codebase.

## 3. Performance at scale

REST API, local SQLite, end-to-end including HTTP round trip:

| Memories | Insert | Fuzzy search | List all | Stats |
|---:|---:|---:|---:|---:|
| 100 | 7.9ms/insert (126/sec) | 1.1ms | 1.1ms | 0.9ms |
| 1,000 | 7.8ms/insert (128/sec) | 3.9ms | 5.4ms | 2.2ms |
| 5,000 | 9.7ms/insert (103/sec) | 26.4ms | 29.5ms | 16.7ms |

Reproduce: `pmem init && pmem api --port 8799 &` then
`python3 scripts/perf_benchmark.py [N]`.

## 4. Footprint

| Metric | Value |
|---|---|
| Release binary size | 9.9 MB |
| Cold start (process spawn -> first MCP response) | 2.1ms avg (1.9–2.4ms over 5 runs) |
| Storage: 100 memories | 84 KB (860 bytes/memory) |
| Storage: 1,000 memories | 472 KB (483 bytes/memory) |
| Storage: 5,000 memories | 2,208 KB (452 bytes/memory) |
| Encryption (AES-256-GCM + PBKDF2, 100k iterations, fresh process per call) | 18.4ms/call avg |

Storage cost per memory drops as the store grows — SQLite's fixed per-table
overhead amortizes over more rows.

Cold start matters specifically for MCP stdio: a client spawns a fresh
`pmem stdio` process per session, so 2ms is effectively free. The
encryption number is dominated by PBKDF2's 100,000 iterations, which is the
point of PBKDF2 — it's deliberately slow to resist brute-forcing a weak
passphrase. It only runs on `pmem encryption encrypt/decrypt`, never on the
read path most commands use.

### Reading these numbers honestly

The scaling table isn't flat, and that's the point of showing three sizes
instead of one. Search/list/stats stay under 10ms at 1,000 memories, then
grow roughly linearly (5x the memories, ~5-8x the latency) by 5,000 — because
`fuzzy_search`, `list_tags`, and a few other `store.rs` paths load the full
table into memory before scoring/aggregating client-side rather than pushing
the work into SQL (a known efficiency gap flagged in code review, not a
mystery uncovered here). Fine at the size most single-project memory stores
will ever reach; the numbers above are the actual evidence for exactly where
that stops being true, not a claim that it never will.
