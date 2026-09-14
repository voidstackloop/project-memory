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

This is the actual value proposition, measured rather than asserted with one
cherry-picked example: 50 real facts about *this* codebase — bugs found and
fixed, decisions made, benchmark results — recorded over the course of
actually building and fixing it this session. Not synthetic filler; every
one of them maps to real source or doc files an agent would otherwise have
to read.

Reproduce: `python3 scripts/token_benchmark.py` (needs `pip install
tiktoken`; falls back to a chars/4 estimate without it). It runs
`memory_context` once, then **50 separate `memory_search` calls**, one
plausible query per fact, and checks each response actually contains the
fact it was searching for — a correctness check riding along with the token
count. All 50 found their target.

| | Tokens | vs. reading source |
|---|---:|---:|
| `memory_context` (all 50 facts, one call) | 2,022 | **19x cheaper** |
| `memory_search`, mean of 50 queries | 168.3 | **231x cheaper** |
| `memory_search`, min / median / max | 149 / 164.5 / 200 | — |
| `memory_search`, stdev | 13.6 | tight — this isn't one lucky query |
| Reading the 16 source/doc files that contain these facts | 38,921 | — |

Per-file breakdown of the "reading source" side:

| File | Tokens | Lines |
|---|---:|---:|
| `src/main.rs` | 19,966 | 2,571 |
| `src/store.rs` | 5,070 | 251 |
| `src/mcp_stdio.rs` | 3,632 | 482 |
| `src/dashboard.rs` | 1,994 | 177 |
| `docs/BENCHMARK.md` | 1,706 | 153 |
| `src/backup.rs` | 1,142 | 125 |
| `src/api.rs` | 1,127 | 123 |
| `README.md` | 1,068 | 129 |
| `src/encryption.rs` | 810 | 61 |
| `docs/DEVELOPMENT.md` | 635 | 66 |
| `docs/ARCHITECTURE.md` | 454 | 47 |
| `src/validation.rs` | 461 | 50 |
| `src/mcp.rs` | 436 | 42 |
| `Dockerfile` | 188 | 20 |
| `docker-compose.yml` | 177 | 30 |
| `docker-entrypoint.sh` | 55 | 5 |

This isn't 50 facts vs. 50 lines of grep — an agent without stored memory
doesn't know in advance which lines matter, so the honest comparison is
against the files those facts actually live in (source *and* docs, since
several facts here are about Docker/README/architecture decisions, not
Rust code). The `memory_context` multiplier dropped from an earlier
12-fact/8-file run (67x) to 19x here, not because memory got more
expensive but because the file set it's compared against grew alongside
the fact count — a fairer comparison, not a better-looking one. The
`memory_search` multiplier, which doesn't depend on how many facts happen
to be stored, held steady (219x on one query earlier, 231x averaged over
50 here) — that's the number that matters for "ask a specific question,"
which is most of what an agent actually does.

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
