# Native CRDT benchmark

| Tasks | automerge v0.10.0 (2026-06-05) | loro v1.13.7 (2026-07-15) | diamond-types v1.0.0 (2022-08-25) | y-octo v0.1.0 (2026-07-27) | yrs v0.27.3 (2026-07-13) |
| :---- | :---- | :---- | :---- | :---- | :---- |
| automerge - apply | 3862.98 ± 3.78 ms | 45.66 ± 0.06 ms | 9.28 ± 0.02 ms | 203.74 ± 0.43 ms | 2387.79 ± 12.86 ms |
| automerge - decode time | 206.94 ± 1.17 ms | 73.09 ± 0.95 us | 1.49 ± 0.00 ms | 37.51 ± 0.05 ms | 2.32 ± 0.01 ms |
| automerge - encode time | 1.71 ± 0.05 ms | 219.52 ± 3.05 us | 672.47 ± 7.71 us | 9.70 ± 0.02 ms | 459.53 ± 0.91 us |
| concurrent list inserts | 166.52 ± 0.71 ms | 164.31 ± 0.41 ms | 34.10 ± 0.06 ms | 7.88 ± 0.02 ms | 7.55 ± 0.03 ms |
| list_random_insert_1k | 248.13 ± 0.25 ms | 10.22 ± 0.10 ms | 2.61 ± 0.00 ms | 2.06 ± 0.01 ms | 3.16 ± 0.00 ms |
| parallel applying automerge edits | - | 10216.50 ± 120.33 ms | 96.83 ± 0.37 ms | 1157.47 ± 14.51 ms | - |

The parallel workload applies the full Automerge trace to two replicas in random batches of 1–1000 local edits, synchronizes after each pair of batches, and verifies replica convergence. Earlier parallel results used an invalid length calculation and are not comparable. With the corrected workload and the complete dependency family pinned, Loro 1.8.1 measures 231.41 ms versus 10216.50 ms for 1.13.7.
