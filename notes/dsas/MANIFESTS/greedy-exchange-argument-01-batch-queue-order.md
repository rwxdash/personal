# Manifest — greedy-exchange-argument/01-batch-queue-order

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `greedy-exchange-argument-01-batch-queue-order` |
| Pattern | Greedy ordering justified by an exchange argument |
| Difficulty | Medium |
| Theme | Single-worker batch job queue, weighted completion time |
| Generated | 2026-08-09 |
| Batch | D |

## Intended approach

Sort ascending by `duration / rate` — expressed as the cross-multiplication
`dA * rB < dB * rA` — then sweep with a running clock accumulating
`rate * clock` after each job's duration is added.

The rule comes from an adjacent swap: two neighbouring jobs occupy the same
block of time in either order, so nothing outside the pair moves. A-first costs
an extra `rB·dA`, B-first an extra `rA·dB`. Since any ordering reaches the
sorted one by adjacent swaps and none of them increases cost, sorted is
optimal.

- **Optimal complexity:** O(n log n) time (sort-dominated), O(n) space.
- **Naive it must beat:** every permutation, O(n!·n). Measured **30.6 s at
  n = 11**, multiplying by `n` for each added job — over a year by n = 15, and
  the bound is 100 000.

## Bounds design

Durations and rates are capped at 10 000 and `n` at 100 000 so that:

- the cross-multiplication `d * r` stays at 10^8, well inside 64-bit, so the
  exact integer comparator has no overflow caveat, and
- the answer tops out near 5 × 10^17, which forces 64-bit accumulators while
  staying comfortably inside `i64`.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Charging the rate before advancing the clock | **verified**: mutation fails **10 of 11** Rust tests |
| Sorting by rate alone | `simple_rules_fail`, Example 1 — **verified**: fails 6 of 11 |
| Sorting by duration alone | `simple_rules_fail`, Example 2 — **verified**: fails 5 of 11 |
| Floating-point ratio comparison (inconsistent order on ties) | `random_ratio_collisions` (ratios deliberately collide), `large_all_ties` (100 000 jobs, all ratio 2) |
| Tie-breaking assumed necessary | `ties`: three permutations of equal-ratio jobs must agree |
| Dependence on input order | `large_sorted_input`: same jobs best-first and worst-first must match |
| 32-bit overflow | `large_all_max`: 5 × 10^17, asserted `> u32::MAX` |
| Empty input / single job | → 0 and `rate · duration` |
| Factorial enumeration | five `_test_large_*` cases at n = 50 000–100 000 |
| General correctness | 600 randomised instances (300 general + 300 with deliberately colliding ratios) vs full permutation search; identical LCG in Python and Rust |

The permutation oracle means a passing run is evidence the ordering rule is
**optimal**, not merely that the implementation is self-consistent.

## Correction during generation

Two hand-computed expectations were wrong and were caught against the
permutation oracle before the Rust mirror was written:

| Case | Written | Correct |
| --- | --- | --- |
| Example 3, `duration=[2,3] rate=[3,5]` | 27 | **30** — the table row `5·3 + 3·5` is 30, not 27 |
| `duration=[100,1,1] rate=[1,100,100]` | 20 602 | **402** — order is job1, job2, job0: 100 + 200 + 102 |

`PROBLEM.md` Example 3 and both asserts were corrected. Per the repo rule, the
tests were fixed rather than the problem bent to fit them.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py greedy-exchange-argument/01-batch-queue-order
=== python: greedy-exchange-argument/01-batch-queue-order ===
All tests passed.
=== rust: greedy-exchange-argument/01-batch-queue-order ===
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
stubs intact: python=True rust=True
RESULT greedy-exchange-argument/01-batch-queue-order: PASS
```

- All 19 hand-written expectations confirmed against the permutation oracle.
- `reference.py` uses `Fraction` as the sort key; `reference.rs` uses
  cross-multiplication in `sort_by`. Both are exact — neither uses floats.
- All three headline traps confirmed by deliberate mutation (recorded above).
- Factorial blow-up confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
