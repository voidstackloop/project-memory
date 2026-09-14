#!/usr/bin/env python3
"""Benchmark pmem's REST API: seed N memories, then time search/list/stats.

Usage:
    pmem init && pmem api --port 8799 &
    python3 scripts/perf_benchmark.py [N]
"""
import json
import sys
import time
import urllib.request

BASE = "http://127.0.0.1:8799"
N = int(sys.argv[1]) if len(sys.argv) > 1 else 1000
KINDS = ["convention", "pattern", "decision", "preference", "context"]
TOPICS = ["auth", "database", "caching", "routing", "logging", "testing", "deploy", "config", "search", "api"]


def post(path, body):
    data = json.dumps(body).encode()
    req = urllib.request.Request(BASE + path, data=data, headers={"content-type": "application/json"}, method="POST")
    with urllib.request.urlopen(req) as r:
        return json.loads(r.read())


def get(path):
    with urllib.request.urlopen(BASE + path) as r:
        return json.loads(r.read())


def main():
    print(f"=== seeding {N} memories via REST API ===")
    t0 = time.time()
    for i in range(N):
        kind = KINDS[i % len(KINDS)]
        topic = TOPICS[i % len(TOPICS)]
        post("/api/memories", {
            "kind": kind,
            "key": f"{topic}-{i}",
            "content": f"Decision #{i} about {topic} handling in the {kind} layer, chosen after evaluating alternatives.",
            "tags": [topic, kind],
            "related_ids": [],
        })
    elapsed = time.time() - t0
    print(f"{N} inserts in {elapsed:.2f}s  ({elapsed / N * 1000:.2f} ms/insert avg, {N / elapsed:.0f} inserts/sec)")

    print(f"\n=== fuzzy search across {N} memories (5 runs) ===")
    times = []
    result = []
    for _ in range(5):
        t0 = time.time()
        result = get("/api/search?q=database&limit=10")
        times.append((time.time() - t0) * 1000)
    print(f"query='database' matches={len(result)}  latency: {[f'{t:.1f}ms' for t in times]}")

    print(f"\n=== list all {N} memories (1 run) ===")
    t0 = time.time()
    result = get(f"/api/memories?limit={N}")
    elapsed = (time.time() - t0) * 1000
    print(f"returned {len(result)} memories in {elapsed:.1f}ms")

    print("\n=== /api/stats ===")
    t0 = time.time()
    stats = get("/api/stats")
    elapsed = (time.time() - t0) * 1000
    print(f"{stats} in {elapsed:.1f}ms")


if __name__ == "__main__":
    main()
