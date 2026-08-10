# Manifest — monotonic-queue/01-autoscaler-stable-windows

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `monotonic-queue-01-autoscaler-stable-windows` |
| Pattern | Monotonic queue (two deques) + sliding window |
| Difficulty | Medium–Hard |
| Theme | Autoscaler / CPU time-series stability auditing |
| Generated | 2026-08-08 |

## Intended approach

Count ranges by right endpoint. For each `j`, the stable-range predicate is
monotone under shrinking, so valid starts form the suffix `[L(j), j]` and
`L(j)` is non-decreasing in `j` — a two-pointer scan. Maintaining the window's
max and min under left-eviction requires two monotonic deques of indices
(max-deque non-increasing, min-deque non-decreasing); the front of each is the
current extreme, and an index dominated by a newer, larger (resp. smaller)
value is discarded permanently from the back. After contracting, add the whole
interval of valid starts at once: `max(0, j - min_len + 2 - L(j))`.

- **Optimal complexity:** O(n) time, O(n) space.
- **Naive it must beat:** O(n²) enumeration over `(i, j)` pairs — ~2 × 10^10
  iterations at n = 200 000. Note that speeding up the range-max/min query does
  not help; the pair enumeration itself is the wall.
- **Also acceptable:** O(n log n) — sparse table for O(1) range extremes plus
  binary search for `L(j)`.

## Traps the tests target

| Trap | Test |
| --- | --- |
| 32-bit accumulator overflow | `_test_large_flat`: answer 20 000 100 000, asserted `> 2**31` |
| usize/unsigned underflow computing `right - min_len + 1` | `_test_examples`: `[1,2,3], delta=10, min_len=5` → 0; `_test_edges` `min_len` > n |
| `min_len` applied as a post-filter instead of in the arithmetic | `[5,7,6,9], delta=2, min_len=2` → 3 |
| Single contraction step (`if` not `while`) | `_test_large_sawtooth`: alternating 0 / 10^9 |
| Deques storing values instead of indices | duplicate-heavy cases: `[2,2,5,2,2]`, `[7]*10` |
| Blind front-popping after `left += 1` | `_test_random_against_brute`, small alphabet forces frequent duplicate fronts |
| Empty input | `count_stable_windows([], ·, ·)` → 0 |
| Boundary at exactly `delta` | `[0, 10^9]` with delta 10^9 → 3 vs delta 10^9−1 → 2 |
| `min_len` exactly n, and n+1 | `[3,3,3]` with min_len 3 → 1, min_len 4 → 0 |
| Quadratic enumeration | all five `_test_large_*` cases at n = 200 000 |
| General correctness | 400 randomised inputs vs brute force (identical LCG in Python and Rust), plus a 200k case checked by an *independent* binary-search recomputation on a non-decreasing series |

## Verification record

**Date:** 2026-08-08

```
$ python3 verify.py monotonic-queue/01-autoscaler-stable-windows
=== python: monotonic-queue/01-autoscaler-stable-windows ===
All tests passed.
=== rust: monotonic-queue/01-autoscaler-stable-windows ===
running 8 tests
........
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
stubs intact: python=True rust=True
RESULT monotonic-queue/01-autoscaler-stable-windows: PASS
```

- `reference.py` — verified, all Python asserts pass.
- `reference.rs` — verified, all 8 `cargo test` cases pass.
- Large-case answers independently confirmed: closed-form ramp count
  (`_ramp_expected`), triangular numbers for flat/block series, and a
  binary-search recomputation for the randomised series.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
