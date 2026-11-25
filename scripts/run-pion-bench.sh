#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
INPUTS_DIR="$TARGET_DIR/pion-inputs"
BIN_PATH="$TARGET_DIR/crdt_bench_pion"
CRITERION_DIR="$TARGET_DIR/criterion/crdt(pion)"

mkdir -p "$INPUTS_DIR" "$CRITERION_DIR"

cargo run --release --manifest-path "$ROOT_DIR/Cargo.toml" --bin export_inputs -- "$INPUTS_DIR"
pionc build --release -o "$BIN_PATH" "$ROOT_DIR/pion-bench"

"$BIN_PATH" "$INPUTS_DIR" "$@" | python3 -c '
import json, math, os, re, sys

out_root = sys.argv[1]
for raw in sys.stdin:
    line = raw.strip()
    if not line:
        continue
    print(line, flush=True)
    parts = line.split("\t")
    if parts[0] != "SAMPLES":
        continue
    _, task, iters_s, samples_s, length_s, fnv_s = parts
    iters = int(iters_s)
    vals_ns = [(float(x) * 1000.0) / iters for x in samples_s.split(",")]
    n = len(vals_ns)
    mean = sum(vals_ns) / n
    var = sum((x - mean) ** 2 for x in vals_ns) / max(1, n - 1)
    stderr = math.sqrt(var) / math.sqrt(n)

    dir_name = re.sub(r"[^A-Za-z0-9._-]+", "_", task)
    task_dir = os.path.join(out_root, dir_name, "new")
    os.makedirs(task_dir, exist_ok=True)
    with open(os.path.join(task_dir, "benchmark.json"), "w") as f:
        json.dump({"group_id": "crdt(pion)", "function_id": task, "full_id": f"crdt(pion)/{task}"}, f, indent=2)
    with open(os.path.join(task_dir, "estimates.json"), "w") as f:
        json.dump(
            {
                "mean": {
                    "point_estimate": mean,
                    "standard_error": stderr,
                }
            },
            f,
            indent=2,
        )
' "$CRITERION_DIR"
