# Manifest — monotonic-stack/01-peak-dominance

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `monotonic-stack-01-peak-dominance` |
| Pattern | Monotonic stack (previous/next strictly greater) |
| Difficulty | Medium |
| Theme | CDN edge bandwidth capacity dashboard |
| Generated | 2026-08-09 |
| Batch | A |

## Intended approach

Two symmetric monotonic-stack passes. Left-to-right yields `left[i]` = nearest
strictly greater index to the left (sentinel −1); right-to-left yields
`right[i]` = nearest strictly greater index to the right (sentinel `n`). Both
pop on `<=`. Answer is `right[i] - left[i] - 1`.

Shadowing argument: at index `i`, any stacked `x` with `load[x] <= load[i]` is
permanently useless, because if `x` would qualify for some future `j` then `i`
qualifies too and is nearer. Hence each index is pushed once, popped at most
once, and the nested `while` is amortised O(1).

- **Optimal complexity:** O(n) time, O(n) space.
- **Naive it must beat:** expand outward per index, O(n²). Its worst case is a
  *flat* series, which is the common case for this metric. Measured 5.60 s at
  n = 8 000, clean 4×-per-doubling → ~58 minutes at n = 200 000.

## Why this is not a window problem

Deliberate separation from `monotonic-queue/01-autoscaler-stable-windows`. That
problem maintains a contiguous range and needs both ends of a deque; this one
produces a per-index answer using only the top of a stack, and there is no
range being maintained at all. Filed under `monotonic-stack` for that reason —
see the overlap note in `INDEX.md`.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Popping on `<` instead of `<=` (plateaus collapse to 1) | `ties_matter`, `large_flat`, `large_plateaus`, `large_sawtooth` — **verified**: mutating `<=` → `<` fails 7 of 11 Rust tests |
| `<=` in one pass, `<` in the other (breaks only on ties) | `random_against_brute` uses a 1–4 value alphabet specifically to force constant ties |
| Joint maxima wrongly blocking each other | `[1,5,1,5,1]` → `[1,5,1,5,1]`; `large_sawtooth` → every peak spans all 200 000 |
| Storing values instead of indices | duplicate-heavy cases throughout |
| Wrong sentinels (`0`/`n-1` instead of `-1`/`n`) | `[1,2,3]` → `[1,2,3]`; `[3,2,1]` → `[3,2,1]`; `large_increasing`/`large_decreasing` |
| Off-by-one in `right - left - 1` | global-maximum cases where both bounds are sentinels |
| Empty and singleton input | `dominance_spans([])` → `[]`; `[42]` → `[1]` |
| Value-range boundaries | `[0, 10^9]` → `[1,2]`; `[10^9, 0]` → `[2,1]` |
| Stack depth / recursion | `large_decreasing`, a 200 000-deep monotone run |
| Quadratic expansion | six `_test_large_*` cases at n = 200 000 |
| General correctness | 700 randomised series (400 tiny-alphabet + 300 wide-alphabet) vs an O(n²) oracle; identical LCG in Python and Rust |

## Corrections during generation

Two hardcoded asserts were wrong, both from the same mistaken assumption — that
tied maxima block each other. Caught against the brute-force oracle before
verification and fixed per the repo rule (fix the test, not the problem):

| Case | Written | Correct |
| --- | --- | --- |
| `[1,5,1,5,1]` | `[1,3,1,3,1]` | `[1,5,1,5,1]` — both 5s are joint maxima, so neither bounds the other |
| sawtooth `0,1,0,1,…` (n = 200 000) | peaks span 3, ends span 2 | peaks span **n**; valleys span 1 |

Both cases were kept (with corrected expectations and explanatory comments)
because they turned out to be the sharpest tie tests in the file. A false start
in `PROBLEM.md` Example 1 and a garbled shadowing argument in `HINTS.md`
Hint 2 were also rewritten before verification.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py monotonic-stack/01-peak-dominance
=== python: monotonic-stack/01-peak-dominance ===
All tests passed.
=== rust: monotonic-stack/01-peak-dominance ===
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
stubs intact: python=True rust=True
RESULT monotonic-stack/01-peak-dominance: PASS
```

- `reference.py` — verified, all Python asserts pass.
- `reference.rs` — verified, all 11 `cargo test` cases pass. Uses `isize`
  boundary arrays so the −1 sentinel is expressible.
- Tie trap confirmed by deliberate mutation (recorded above).
- Naive infeasibility confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
