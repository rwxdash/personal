# Manifest — intervals-sweep-line/01-gateway-peak-concurrency

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `intervals-sweep-line-01-gateway-peak-concurrency` |
| Pattern | Sweep line over sorted events |
| Difficulty | Medium |
| Theme | API gateway connection-pool sizing |
| Generated | 2026-08-09 |
| Batch | A |

## Intended approach

Convert each session into two events, `(start, +1)` and `(end, -1)`, sort all
`2n` lexicographically — which puts releases before claims at equal timestamps,
since `-1 < +1` — then sweep with a running counter, recording a new best on a
**strict** `>`. Return `(best, best_time)`.

Concurrency is a step function changing only at the `2n` event instants, and it
only rises at a `start`, so the peak is always attained at some session's start
and the 10^9 coordinate range never enters the cost.

- **Optimal complexity:** O(n log n) time (sort-dominated), O(n) space.
- **Naive it must beat:** pairwise / per-candidate-instant evaluation, O(n²).
  Measured 0.53 s at n = 4 000, clean 4×-per-doubling → ~22 minutes at
  n = 200 000.
- **Also excluded by the constraints:** bucketing the timeline, which is
  O(max timestamp) = 10^9 time *and* memory. Unbounded in the wrong variable.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Opens ordered before closes at equal timestamps (breaks half-open semantics) | `half_open`, `large_handover_chain` — **verified**: mutating the sort so opens precede closes fails 8 of 13 Rust tests |
| `>=` instead of `>` (reports latest peak instant, not earliest) | `edges`: `[(0,5),(0,5),(100,105),(100,105)]` → `(2, 0)` — **verified**: mutation fails 11 of 13 tests |
| Unsigned counter underflowing on the zero-length transient dip | `zero_length`, `large_zero_length_flood` |
| Zero-length sessions counted as occupying a slot | `[(5,5)]` → `(0,0)`; `[(5,5),(5,10)]` → `(1,5)` |
| Returning a timestamp when the peak is 0 | `peak_concurrency([])` → `(0,0)`; all-zero-length → `(0,0)` |
| Assuming the input is sorted | `[(50,60),(10,20)]` → `(1, 10)`; `large_unsorted` (shuffled staircase) |
| Deduplicating identical sessions | `[(3,7)]×3` → `(3, 3)` |
| Off-by-one at overlap boundaries | `[(0,6),(5,10)]` → `(2,5)` vs `[(0,5),(5,10)]` → `(1,0)` |
| Timestamp range boundaries | `[(0,10^9),(10^9−1,10^9)]` → `(2, 10^9−1)` |
| Materialising the timeline | 10^9 coordinate range across all large cases |
| Quadratic pair comparison | seven `_test_large_*` cases at n = 100 000–200 000 |
| General correctness | 800 randomised instances (400 general + 400 clustered into a 4-value coordinate range to force shared timestamps) vs a direct per-instant oracle; identical LCG in Python and Rust |

Large-case answers are pinned by construction: all-overlapping → `(n, 0)`;
handover chain → `(1, 0)`; width-`k` staircase → `(k, k−1)`; nested → `(n, n−1)`;
disjoint → `(1, 0)`.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py intervals-sweep-line/01-gateway-peak-concurrency
=== python: intervals-sweep-line/01-gateway-peak-concurrency ===
All tests passed.
=== rust: intervals-sweep-line/01-gateway-peak-concurrency ===
running 13 tests
.............
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
stubs intact: python=True rust=True
RESULT intervals-sweep-line/01-gateway-peak-concurrency: PASS
```

- All 21 hand-written expectations independently confirmed against the
  brute-force oracle before the Rust mirror was written.
- `reference.py` — verified, all Python asserts pass.
- `reference.rs` — verified, all 13 `cargo test` cases pass. Signed counter so
  the zero-length transient dip cannot underflow.
- Both headline traps confirmed by deliberate mutation (recorded above).
- Naive infeasibility confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
