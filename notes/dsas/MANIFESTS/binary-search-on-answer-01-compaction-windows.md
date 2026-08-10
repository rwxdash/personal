# Manifest — binary-search-on-answer/01-compaction-windows

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `binary-search-on-answer-01-compaction-windows` |
| Pattern | Binary search on the answer + greedy feasibility check |
| Difficulty | Medium |
| Theme | Log-structured store overnight compaction |
| Generated | 2026-08-09 |
| Batch | B |

## Intended approach

Binary search the integer range `[max(sizes), sum(sizes)]` for the smallest
ceiling `C` whose feasibility check passes. The check is one greedy left-to-
right pass counting windows, cutting for two distinct reasons: the ceiling
would be breached, or the segment is a checkpoint (unconditional, fires even
with room to spare). Feasible iff `windows <= k`.

Greedy is optimal for a fixed ceiling by an exchange argument: pushing a
segment into an earlier window that has room never breaks the ceiling and never
increases the window count, so any feasible arrangement reduces to the greedy
one.

- **Optimal complexity:** O(n log T) time (~48 passes at T = 2 × 10^14),
  O(1) extra space.
- **Naive it must beat:** DP over (segments, windows), O(n²k). Measured
  0.181 s at n = 240 / k = 30 with ~8×-per-doubling growth (n³ as k scales with
  n) → extrapolates to roughly **3 years** at n = 200 000.

## The checkpoint twist

Deliberate deviation from the textbook "split array largest sum". Forced cut
positions mean the feasibility check has two independent cut triggers, and the
unconditional one is easy to write conditionally by mistake. It also creates
the solvability precondition recorded in the statement (at most `k - 1`
checkpoints after index 0), which the random generator respects by choosing `k`
first and then spending a checkpoint budget.

## Traps the tests target

| Trap | Test |
| --- | --- |
| `lo` started at 0/1 without an `s > C` guard — greedy silently "fits" an oversized segment | **verified**: mutating `lo = max(sizes)` → `lo = 0` fails 6 of 10 Rust tests, including `random_against_brute` |
| Checkpoint cut treated as conditional, or dropped | `checkpoints_bind`, `large_checkpoint_every_other` — **verified**: deleting the branch fails 4 of 10 |
| `checkpoints[0]` applied as a real cut | `edges`: `min_window_bytes([5], [True], 1)` → 5 |
| Balancing segment counts instead of bytes | Example 1: `[7,2,5,10,8]`, k=2 → 18 |
| `hi = mid - 1` (skips the answer) or `lo = mid` (infinite loop) | every case with a unique answer, e.g. `large_binary_search_range` |
| Aiming for exactly `k` windows rather than at most `k` | `[3,3,3]` k=3 → 3; `large_uniform` with k = 1000 |
| 32-bit overflow | `large_all_max`: total 2 × 10^14 |
| Answer below `max(sizes)` | `[1,1,100,1,1]` k=3 → 100; `large_single_dominant` → 10^9 |
| Quadratic/cubic DP | six `_test_large_*` cases at n = 100 000–200 000 |
| General correctness | 400 randomised instances (k chosen first, checkpoints placed within budget) vs an O(n²k) DP oracle; identical LCG in Python and Rust |

All 19 hand-written expectations were independently confirmed against the DP
oracle before the Rust mirror was written.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py binary-search-on-answer/01-compaction-windows
=== python: binary-search-on-answer/01-compaction-windows ===
All tests passed.
=== rust: binary-search-on-answer/01-compaction-windows ===
running 10 tests
..........
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
stubs intact: python=True rust=True
RESULT binary-search-on-answer/01-compaction-windows: PASS
```

- `reference.py` / `reference.rs` — both verified.
- Both headline traps confirmed by deliberate mutation (recorded above).
- Naive infeasibility confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.

### Correction during generation

`_test_large_checkpoint_every_other` was first written with `k = n/2`, which
violates the problem's own solvability precondition — a checkpoint at every odd
index is 100 000 forced cuts and needs `k >= 100 001`. Fixed by deriving `k`
from the forced-cut count in the test itself, with an assertion pinning it at
100 001.
