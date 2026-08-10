# Manifest — prefix-sums/01-settlement-runs

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `prefix-sums-01-settlement-runs` |
| Pattern | Prefix sums + hash map on residues mod m |
| Difficulty | Medium |
| Theme | Payments ledger settlement batching |
| Generated | 2026-08-09 |
| Batch | A |

## Intended approach

A run's sum is `P[j] - P[i]` for running totals `P`, which is divisible by `m`
exactly when `P[i] ≡ P[j] (mod m)`. So the problem is counting pairs of equal
residues among the `n + 1` prefixes. One pass with two maps keyed by residue:
a tally (`count += seen_count[r]` *before* incrementing) for the count, and a
first-seen index (never overwritten) for the longest run. Both seeded with the
empty prefix `P[0] = 0`.

- **Optimal complexity:** O(n) time, O(min(n, m)) space — the maps are keyed by
  residue, so they stay small even at `m = 10^9`.
- **Naive it must beat:** O(n²) enumeration over `(i, j)`. Measured 3.69 s at
  n = 8 000, clean 4×-per-doubling → ~38 minutes at n = 200 000.

## Why this cannot be a sliding window

Deliberate design point, and the reason this problem sits under `prefix-sums`
rather than being a third window problem. Negative deltas mean a run's sum is
**not monotone** in its length, so there is no "extend until it qualifies, then
contract" structure. A two-pointer solution here is not slow — it is wrong.
The statement foregrounds refunds and chargebacks for exactly this reason, and
`_test_negative_modulo` plus the signed random generator make sure a window
cannot accidentally pass.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Rust `%` vs `rem_euclid` on negative running totals | `negative_modulo` — **verified**: swapping `rem_euclid` → `%` fails `negative_modulo` and `random_against_brute`, the other 9 tests still pass |
| Missing `P[0] = 0` seed (loses runs starting at index 0) | `[5,5,5,5]` m 5 → (10, 4); `_test_large_all_zero` |
| Tally incremented before being read (over-counts by n) | every case with repeated residues, e.g. `[0,0,0]` → (6, 3) |
| `first_index` overwritten instead of kept earliest | `[3,3,1,1,3]` m 3 → (4, 2) |
| `longest` tracking most-recent match rather than maximum | same case — the longest run appears early |
| 32-bit count overflow | `_test_large_all_zero` → 20 000 100 000, asserted `> 2**31` |
| 32-bit running-total overflow | `_test_large_unit_modulus`, deltas at ±10^9 over 200k elements |
| Zero treated as not-a-multiple | `[0]` m 7 → (1, 1); `[7,-7]` m 10^9 → (1, 2) |
| Negative sums not recognised as settleable | `[-6]` m 3 → (1, 1); `[-1,-2]` m 3 → (1, 2) |
| Special-casing `m = 1` | `_test_large_unit_modulus` → full triangular count |
| Residue map sized by `m` rather than by distinct residues | `random_large_modulus` and `[7,-7]` at m = 10^9 |
| Quadratic enumeration | six `_test_large_*` cases at n = 100 000–200 000 |
| General correctness | 600 randomised signed ledgers (400 small-modulus + 200 large-modulus) vs an O(n²) oracle; identical LCG in Python and Rust |

Large-case answers are closed-form, not reference-derived: triangular numbers
for the all-zero and unit-modulus ledgers, and sums of `C(k, 2)` over residue
classes for the periodic and alternating ones.

## Correction during generation

Three hardcoded asserts were computed wrong by hand and were fixed against the
brute-force oracle before verification:

| Case | Written | Correct |
| --- | --- | --- |
| `[3,1,1,1,3]` m 3 | `(2, 1)` | `(6, 5)` — case replaced by `[3,3,1,1,3]` → `(4, 2)`, which actually exercises the intended "longest appears early" trap |
| `[-1,4,-3]` m 3 | `(2, 3)` | `(3, 3)` |
| `[2,-5,3]` m 5 | `(1, 3)` | `(2, 3)` |

Per the repo rule, the tests were corrected — the problem was not bent to fit
them.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py prefix-sums/01-settlement-runs
=== python: prefix-sums/01-settlement-runs ===
All tests passed.
=== rust: prefix-sums/01-settlement-runs ===
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
stubs intact: python=True rust=True
RESULT prefix-sums/01-settlement-runs: PASS
```

- `reference.py` — verified, all Python asserts pass.
- `reference.rs` — verified, all 11 `cargo test` cases pass.
- Modulus trap confirmed by deliberate mutation (recorded above).
- Naive infeasibility confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
