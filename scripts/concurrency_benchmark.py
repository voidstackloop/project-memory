#!/usr/bin/env python3
"""Fire concurrent requests at the REST API and measure throughput/latency at
increasing concurrency levels. Exists to verify api.rs/dashboard.rs actually
benefit from running store access on a blocking-pool thread (spawn_blocking)
instead of stalling the async executor — a regression here would show up as
throughput flattening or latency spiking non-linearly as concurrency rises.

Usage:
    pmem init && pmem api --port 8799 &
    python3 scripts/concurrency_benchmark.py
"""
import json
import statistics
import sys
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor

BASE = "http://127.0.0.1:8799"
REQUESTS_PER_LEVEL = 300
CONCURRENCY_LEVELS = [1, 10, 50, 100]


def post(path, body):
    data = json.dumps(body).encode()
    req = urllib.request.Request(BASE + path, data=data, headers={"content-type": "application/json"}, method="POST")
    urllib.request.urlopen(req).read()


def get(path):
    with urllib.request.urlopen(BASE + path) as r:
        return r.read()


def timed_search():
    t0 = time.time()
    get("/api/search?q=database&limit=10")
    return time.time() - t0


def seed(n=500):
    for i in range(n):
        post("/api/memories", {
            "kind": "context", "key": f"seed-{i}",
            "content": f"Seed memory #{i} about database handling for the concurrency benchmark.",
            "tags": ["bench"], "related_ids": [],
        })


def percentile(sorted_vals, p):
    idx = int(len(sorted_vals) * p) - 1
    return sorted_vals[max(0, min(idx, len(sorted_vals) - 1))]


def main():
    print(f"=== seeding {500} memories ===")
    seed(500)

    print(f"\n=== concurrent GET /api/search, {REQUESTS_PER_LEVEL} requests per level ===")
    print(f"{'concurrency':>11} | {'req/sec':>9} | {'p50 (ms)':>9} | {'p95 (ms)':>9} | {'p99 (ms)':>9} | {'max (ms)':>9}")
    print("-" * 70)
    for c in CONCURRENCY_LEVELS:
        t0 = time.time()
        with ThreadPoolExecutor(max_workers=c) as pool:
            latencies = list(pool.map(lambda _: timed_search(), range(REQUESTS_PER_LEVEL)))
        elapsed = time.time() - t0
        latencies_ms = sorted(l * 1000 for l in latencies)
        rps = REQUESTS_PER_LEVEL / elapsed
        print(f"{c:>11} | {rps:>9.1f} | {percentile(latencies_ms, 0.50):>9.1f} | "
              f"{percentile(latencies_ms, 0.95):>9.1f} | {percentile(latencies_ms, 0.99):>9.1f} | {max(latencies_ms):>9.1f}")

    print("\n=== does /api/health stay responsive under 100 concurrent /api/search? ===")
    print("(this is the real test of spawn_blocking: a single Mutex<MemoryStore>/SQLite")
    print(" connection means DB throughput itself is always going to be serialized — the")
    print(" question is whether that serialization stalls the whole async executor, which")
    print(" would make even a trivial, store-free endpoint like /api/health slow too)")
    with ThreadPoolExecutor(max_workers=100) as pool:
        futures = [pool.submit(timed_search) for _ in range(300)]
        time.sleep(0.05)  # let the search flood get going first
        health_latencies = []
        for _ in range(20):
            t0 = time.time()
            get("/api/health")
            health_latencies.append((time.time() - t0) * 1000)
        for f in futures:
            f.result()
    print(f"/api/health under load: mean={statistics.mean(health_latencies):.1f}ms  "
          f"max={max(health_latencies):.1f}ms  (idle baseline is sub-millisecond)")

    print("\nA healthy result: /api/health stays fast (low single-digit ms) even while 100")
    print("concurrent searches are in flight — proving spawn_blocking keeps the async")
    print("executor free for unrelated work, even though total search throughput is")
    print("still capped by the single SQLite connection behind Mutex<MemoryStore>.")


if __name__ == "__main__":
    main()
