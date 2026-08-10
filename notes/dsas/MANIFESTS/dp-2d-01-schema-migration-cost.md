# Manifest — dp-2d/01-schema-migration-cost

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `dp-2d-01-schema-migration-cost` |
| Pattern | 2D dynamic programming over two sequence prefixes |
| Difficulty | Medium |
| Theme | Database schema migration downtime |
| Generated | 2026-08-09 |
| Batch | C |

## Intended approach

`dp[i][j]` = cheapest way to turn the first `i` columns of `current` into the
first `j` of `target`. Base cases `dp[i][0] = i·drop`, `dp[0][j] = j·insert`.
Transition takes the min of drop / insert / handle-both, where handle-both is
free when the names match and `retype_cost` otherwise.

Rolled to two rows for the space bound, with the shorter sequence in the inner
loop — and `insert_cost` swapped with `drop_cost` whenever the sequences are
swapped, since that reverses the direction of the migration.

- **Optimal complexity:** O(n·m) time, O(min(n,m)) space. A full n×m table is
  explicitly ruled out by the stated space bound.
- **Naive it must beat:** unmemoised three-way recursion. Measured 0.393 s at
  n = m = 9, growing ~5.7× per added column — hours by n = 20, and the bound is
  2000.

## Design note: asymmetric costs

Distinct insert / drop / retype costs are what keep this from being textbook
Levenshtein. They force the candidate to reason about which route through the
grid wins rather than counting edits, and they make the row-swap bug (below)
observable. Example 2 and Example 3 are the same shape with the retype cost
moved either side of `drop + insert`, giving different answers.

## Correction during generation

The hints and reference comments originally claimed the unconditional `min` was
*necessary* — that short-circuiting to the free diagonal on a name match could
be wrong with arbitrary costs. That claim was false, and it was caught by
mutation testing: the short-circuit version passed all 11 tests, so it was
checked directly rather than assumed to be a test gap.

The short-circuit is provably correct. When the names match:

- `dp[i-1][j-1] <= dp[i-1][j] + drop_cost` (take that solution, drop its last
  produced column)
- `dp[i-1][j-1] <= dp[i][j-1] + insert_cost` (insert the extra source column,
  then apply that solution)

Confirmed empirically on 300 000 random inputs with independently varied costs:
zero disagreements. `HINTS.md` and both reference files were corrected to say
both versions are valid and to give the proof. The reference keeps the
unconditional `min` for simplicity.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Sequences swapped for the space bound without swapping insert/drop | **verified**: mutation fails 6 of 11 Rust tests; `random_lopsided` runs every instance in both orientations to force it |
| Assuming an alignment, then pricing it | Example 5: `["a","b"] → ["b"]` → 1, not 2 |
| Special-casing `retype > drop + insert` | Example 2 → 2; `cost_asymmetry` |
| Missing base cases (row 0 / column 0) | `edges`: empty-side cases; `large_one_empty` |
| Crossed rolling-row indices | asymmetric costs throughout, e.g. `["a","b","c"] → ["a"]` with drop 100 → 200 |
| 32-bit overflow | `large_all_different`: 2 × 10^9 |
| Assuming non-empty inputs | `migration_cost([], [], …)` → 0 |
| Assuming distinct column names | `["a","a","a"] → ["a"]` → 2 |
| Zero costs | `cost_asymmetry`: free inserts, free drops, all-zero |
| Reordering assumed free | `["a","b"] → ["b","a"]` → 2 |
| Exponential recursion | six `_test_large_*` cases at 2000 × 2000 |
| General correctness | 600 randomised instances (300 general + 300 lopsided, each run in both orientations) vs unmemoised recursion; identical LCG in Python and Rust |

All 24 hand-written expectations were confirmed against the recursive oracle
before the Rust mirror was written.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py dp-2d/01-schema-migration-cost
=== python: dp-2d/01-schema-migration-cost ===
All tests passed.
=== rust: dp-2d/01-schema-migration-cost ===
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
stubs intact: python=True rust=True
RESULT dp-2d/01-schema-migration-cost: PASS
```

- `reference.py` / `reference.rs` — both verified, both using two rolling rows
  over the shorter sequence.
- Row-swap trap confirmed by deliberate mutation.
- Diagonal-shortcut question resolved by direct experiment (see correction
  above) rather than left as an assumption.
- Exponential blow-up confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
