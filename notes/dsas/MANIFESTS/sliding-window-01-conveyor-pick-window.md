# Manifest — sliding-window/01-conveyor-pick-window

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `sliding-window-01-conveyor-pick-window` |
| Pattern | Sliding window (two pointers) |
| Difficulty | Medium |
| Theme | Warehouse fulfilment / conveyor picking |
| Generated | 2026-08-08 |

## Intended approach

Two-pointer window over the belt with a hash map of held counts plus a single
scalar `missing` = number of required *units* not yet covered. Admitting an
item decrements `missing` only when its count is still below the requirement
(surplus is inert); evicting increments `missing` only when the count drops
below the requirement. While `missing == 0` the window is recorded and shrunk
from the left. Strict `<` on the length comparison yields the earliest-start
tie-break for free, since candidates are recorded in increasing order of
`left`.

- **Optimal complexity:** O(n) time, O(k) space (k = distinct ordered SKUs).
- **Naive it must beat:** O(n²) / O(n²·k) — restart the scan from every start
  index. Measured at 5.0 s for n = 8 000 in CPython with clean 4×-per-doubling
  scaling, extrapolating to ~52 minutes at n = 200 000.
- **Also acceptable:** O(n log n) binary search on window length with a
  fixed-width sweep per candidate length.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Surplus items wrongly counted as progress (`missing` goes negative) | `_test_edges`: `["A"]*5` vs `{"A": 2, "B": 1}` → `None` |
| One occurrence treated as satisfying a quantity > 1 | same, plus `{"A": 4}` case |
| Tie-break returning the latest instead of earliest shortest window | `_test_examples`: `["A","B","Z","A","B"]` → `(0, 1)` |
| `if` instead of `while` when contracting | `["A"]*10 + ["B"]` vs `{"A":3,"B":1}` → `(7, 10)` |
| Required SKU absent from the stream | `["A","A","A"]` vs `{"B": 1}` → `None` |
| Answer spanning the entire input | `_test_large_whole_belt` → `(0, 199_999)` |
| Off-by-one at the window's right edge | `_test_large_planted` → exact `(100_000, 100_003)` |
| O(n·k) full-map comparison per step | large tests with multi-SKU orders |
| Quadratic rescan | all four `_test_large_*` cases at n = 200 000 |
| General correctness | 400 randomised inputs cross-checked against a brute force, identical LCG sequence in Python and Rust |

## Verification record

**Date:** 2026-08-08

Reference spliced into a scratch copy of the candidate files ahead of the
untouched test block, then run:

```
$ python3 verify.py sliding-window/01-conveyor-pick-window
=== python: sliding-window/01-conveyor-pick-window ===
All tests passed.
=== rust: sliding-window/01-conveyor-pick-window ===
running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
stubs intact: python=True rust=True
RESULT sliding-window/01-conveyor-pick-window: PASS
```

- `reference.py` — verified, all Python asserts pass.
- `reference.rs` — verified, all 7 `cargo test` cases pass.
- Naive infeasibility confirmed by direct timing (see above).
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
