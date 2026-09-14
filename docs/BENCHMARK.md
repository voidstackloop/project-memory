# Benchmark

Two questions this answers: does `pmem` actually work as an MCP server for a
real agent, and is it fast enough to use. Both were run live against the
`v0.2.0` release build, not mocked.

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

## 2. Tool calls actually work

Driving raw JSON-RPC at the exact `pmem stdio` process OpenCode connects to,
against a project seeded with three memories:

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
   ## Decisions
   - database [storage, architecture]: Use SQLite via rusqlite bundled...
   ## Context
   - opencode-verified [opencode, benchmark]: This memory was written by
     OpenCode's MCP client via pmem stdio.
```

Fuzzy search correctly ranks the exact-key match (`database`, score 1.20)
above partial content matches; the memory written by one tool call is
immediately visible to another (`memory_context`) in the same session.

## 3. Performance

REST API, 1,000 memories, local SQLite, measured end to end including HTTP
round trip (not just the query):

| Operation | Result |
|---|---|
| Insert (1,000 sequential) | 128/sec, 7.8ms avg |
| Fuzzy search | ~3.9ms |
| List all 1,000 | 5.4ms |
| Stats (count + group by kind/tag) | 2.2ms |

Reproduce it:

```bash
pmem init && pmem api --port 8799 &
python3 scripts/perf_benchmark.py       # defaults to 1,000 memories
python3 scripts/perf_benchmark.py 5000  # or pass a different count
```

### Reading these numbers honestly

Search/list/stats stay well under 10ms at 1,000 memories because SQLite
handles that scale trivially. They will not stay flat forever: `fuzzy_search`
and a few other paths currently load the full table into memory before
scoring (tracked as a known efficiency gap, not a mystery) rather than
pushing candidate selection into SQL — fine at the size most single-project
memory stores will ever reach, worth revisiting if someone runs this against
tens of thousands of entries.
