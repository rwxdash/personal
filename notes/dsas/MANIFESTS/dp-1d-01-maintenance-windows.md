# Manifest — dp-1d/01-maintenance-windows

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `dp-1d-01-maintenance-windows` |
| Pattern | 1D dynamic programming over index |
| Difficulty | Medium |
| Theme | Fleet maintenance scheduling with cooldown |
| Generated | 2026-08-09 |
| Batch | C |

## Intended approach

`best[i]` = maximum total value using slots `0..i`:

```
best[i] = max(best[i-1], value[i] + best[i - cooldown - 1])
```

Take branch only when slot `i` is not blacked out; `best[j] = 0` for `j < 0`.
Answer is `best[n-1]`, or 0 when empty.

The lookup needs no backward scan because `best` is **non-decreasing** —
skipping is always legal, so `best[i - cooldown - 1]` is already the maximum
over everything at or before that cut-off. That single property is what makes
it O(n) instead of O(n²), and it is exactly what breaks if values are allowed
to go negative (recorded as an interview follow-up).

- **Optimal complexity:** O(n) time, O(n) space.
- **Naive it must beat:** enumerate all subsets, O(2^n · n). Measured 41.9 s at
  **n = 24**, doubling per added slot — unusable by n = 40, and n reaches
  200 000.
- **Also too slow:** the same recurrence with a backward scan for the max,
  O(n²).

## Traps the tests target

| Trap | Test |
| --- | --- |
| Lookback index `i - cooldown` instead of `i - cooldown - 1` | **verified**: mutation fails 9 of 12 Rust tests, including all 4 examples |
| Greedy "take the largest legal slot" | `greedy_is_wrong`: `[4,5,4]` cd 1 → 8 (greedy gives 5); `[5,6,5]` → 10; `[3,10,3,3,10,3]` cd 2 → 20 |
| `j < 0` treated as "cannot take" rather than base 0 | `edges`: `[7]` with cooldown 100 → 7 |
| Blackout slots not written into the array | `[4,100,4]` with the 100 blacked out → 8 |
| Distance exactly `cooldown` accepted as legal | `[5,0,5]` cd 2 → 5 vs `[5,0,0,5]` cd 2 → 10 |
| `cooldown = 0` special-cased | Example 4 → 18; `large_no_cooldown` |
| `cooldown >= n` special-cased | `[3,9,4]` cd 1000 → 9; `large_single_choice` |
| 32-bit overflow | `large_no_cooldown`: 2 × 10^14 |
| Empty input | `max_maintenance_value([], [], 0)` → 0 |
| All blackout / all zero values | → 0 |
| Exponential enumeration | seven `_test_large_*` cases at n = 200 000 |
| General correctness | 700 randomised inputs (400 with small cooldowns + 300 where cooldown often exceeds n) vs full subset enumeration; identical LCG in Python and Rust |

Large-case answers are closed-form: sum for cooldown 0, `n/2` picks for the
uniform alternating case, peak counts for the spaced-peaks cases, and an
arithmetic series for the ramp.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py dp-1d/01-maintenance-windows
=== python: dp-1d/01-maintenance-windows ===
All tests passed.
=== rust: dp-1d/01-maintenance-windows ===
running 12 tests
............
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
stubs intact: python=True rust=True
RESULT dp-1d/01-maintenance-windows: PASS
```

- All 19 hand-written expectations independently confirmed against subset
  enumeration before the Rust mirror was written.
- `reference.py` / `reference.rs` — both verified. The Rust version computes
  the lookback in signed arithmetic so `i - cooldown - 1` cannot wrap a `usize`.
- Off-by-one trap confirmed by deliberate mutation (recorded above).
- Exponential blow-up confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
