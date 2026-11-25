# Native CRDT benchmark

| Tasks | automerge v0.12.0 (2026-09-16) | loro v1.19.1 (2026-11-24) | diamond-types v1.0.0 (2022-08-25) | y-octo v0.4.1 (`05e0611`) | yrs v0.28.2 (2026-11-19) | crdt(pion) v0.2.0 (`f66e02f`) |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| automerge - apply | 2775.57 ± 159.19 ms | 87.34 ± 1.14 ms | 17.62 ± 0.29 ms | 125.36 ± 0.45 ms | 5058.18 ± 102.19 ms | 1122.35 ± 38.52 ms |
| automerge - decode time | 521.23 ± 13.89 ms | 96.62 ± 4.50 us | 1.77 ± 0.00 ms | 4.16 ± 0.11 ms | 4.03 ± 0.13 ms | 17.73 ± 1.31 ms |
| automerge - encode time | 312.38 ± 10.19 us | 32.12 ± 0.48 us | 851.80 ± 11.58 us | 605.99 ± 8.68 us | 890.70 ± 72.03 us | 30.75 ± 1.44 ms |
| concurrent list inserts | 266.16 ± 9.14 ms | 119.81 ± 4.51 ms | 58.24 ± 0.32 ms | 9.38 ± 0.11 ms | 12.86 ± 0.27 ms | 19.67 ± 0.42 ms |
| list_random_insert_1k | 522.56 ± 10.72 ms | 8.11 ± 0.21 ms | 4.36 ± 0.05 ms | 2.42 ± 0.03 ms | 6.62 ± 0.52 ms | 3.80 ± 0.13 ms |
| parallel applying automerge edits | 14962.21 ± 1909.45 ms | 460.08 ± 12.25 ms | 343.71 ± 5.85 ms | 540.04 ± 5.99 ms | 11751.18 ± 486.09 ms | 4873.87 ± 152.87 ms |
| text - append 50k | 247.07 ± 8.50 ms | 9.31 ± 0.01 ms | 2.30 ± 0.02 ms | 10.29 ± 0.44 ms | 15.50 ± 0.14 ms | 13.96 ± 0.45 ms |
| text - backspace 50k | 206.85 ± 5.90 ms | 1289.92 ± 10.73 ms | 4.34 ± 0.11 ms | 40.99 ± 3.41 ms | 1802.71 ± 11.47 ms | 1879.28 ± 42.15 ms |
| text - random edits 20k | 375.77 ± 13.73 ms | 10.63 ± 0.43 ms | 2.17 ± 0.02 ms | 18.14 ± 0.40 ms | 781.49 ± 42.02 ms | 113.98 ± 6.95 ms |
| text - tombstone 50k | 458.37 ± 8.93 ms | 189.85 ± 1.04 ms | 5.37 ± 0.03 ms | 48.33 ± 1.00 ms | 5007.84 ± 260.72 ms | 14072.26 ± 1602.63 ms |

Measured on AMD Ryzen 7 PRO 8845HS (Linux x86_64) with `rustc 1.100.0 (0ef1bc386 2026-11-09)` and `pionc 0.1.0 (amd64_sysv, abi pionc-v0.19, self-hosted)`. Each value is the mean ± standard error across 10 samples (Criterion `measurement_time(1s).sample_size(10)` for the five Rust crates; matching 10-sample monotonic-clock runner in `pion-bench/` for `crdt(pion)`).

## Workloads & Notes

- **Adapter settings**: `y-octo` uses Update V1 with `large_refs`; `yrs` uses Update V2; `crdt(pion)` uses Update V1 (`AFK-surf/pion-packages` `packages/crdt`). `AutomergeDoc::text_insert` uses `splice_text` so multi-character text inserts behave identically across all CRDTs (verified by `cargo test --release --lib` and `export_inputs --hashes`).
- **`parallel applying automerge edits`**: Applies the full Automerge trace to two replicas in random batches of 1–1000 local edits, synchronizes after each pair of batches, and verifies replica convergence.
- **`text - append 50k`**: 50,000 sequential single-character ASCII tail inserts.
- **`text - backspace 50k`**: Insert a 50,000-character ASCII block, then delete 1 character at a time from the tail 50,000 times.
- **`text - random edits 20k`**: 20,000 deterministic ASCII text ops (60% cursor typing, 25% random insert, 15% random delete of 1–4 chars).
- **`text - tombstone 50k`**: 50,000 single-character inserts, delete every odd index (25,000 tombstones), then 10,000 random single-character inserts.

## Running

```bash
# Rust CRDTs
cargo test --release --lib
cargo bench

# crdt(pion) (requires pionc)
./scripts/run-pion-bench.sh

# Generate Markdown table
deno run --allow-read --allow-net --allow-env scripts/gen-report.ts
```
