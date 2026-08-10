# Manifest — two-pointers/01-host-consolidation

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `two-pointers-01-host-consolidation` |
| Pattern | Two pointers (converging) + greedy with exchange argument |
| Difficulty | Medium |
| Theme | VM fleet consolidation onto physical hosts |
| Generated | 2026-08-09 |
| Batch | A |

## Intended approach

Split off pinned VMs (one host each, then irrelevant). Sort the unpinned
footprints ascending and converge two pointers: pair the largest with the
smallest if they fit, otherwise the largest goes alone; either way the largest
is placed and exactly one host is consumed per iteration. Return
`pinned_count + hosts`.

Key reframing: minimising hosts == maximising pairs, since `u` unpinned VMs
forming `p` pairs use `u - p` hosts.

Exchange argument (in full in the editorial): if `s + L <= cap`, any optimal
arrangement can be rewritten to pair them without adding a host. The only
non-trivial case is `{L,x}, {s,y}` → `{L,s}, {x,y}`, legal because `y <= L`
gives `x + y <= x + L <= cap`.

- **Optimal complexity:** O(n log n) time (sort-dominated), O(n) space.
- **Naive it must beat:** "take the largest, scan for a partner" — correct but
  O(n²). Measured on its worst case (nothing pairs, so every inner scan runs
  to completion) at 1.68 s for n = 8 000, clean 4×-per-doubling → ~17 minutes
  at n = 200 000.
- **Also infeasible:** exhaustive matching over subsets (what the oracle does),
  exponential.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Pairing largest with largest-that-fits instead of smallest | `_test_random_against_brute` vs exhaustive matching |
| Sorting descending and pairing adjacent elements | `[5,4,3,2]`, cap 7 → 2 (adjacent pairing gives 3) |
| Pinned VMs allowed into the pairing sweep | `[1,1]` pinned `[True,False]` → 2 vs `[False,False]` → 1 |
| Pinned VMs filtered out and never re-added | `_test_large_all_pinned` → 200 000 |
| `lo == hi` mishandled (VM dropped or paired with itself) | `[1,1,1]` cap 2 → 2; `[5]` → 1 |
| Assuming every VM can pair with something | `[3,3,3]` cap 3 → 3 |
| Off-by-one at the capacity boundary | `[60]*n` cap 119 → n vs cap 120 → n/2 |
| Empty input | `min_hosts([], [], 5)` → 0 |
| Quadratic partner search | `_test_large_nothing_pairs`, worst case for the inner scan |
| Greedy optimality itself | 300 randomised instances cross-checked against exhaustive bitmask matching — this oracle proves the greedy rule, it does not assume it |

Note on the oracle: it computes `pinned + (unpinned - max_matching)` by
bitmask DP over subsets, sharing no reasoning with the two-pointer greedy. A
passing run is therefore evidence that the greedy is *optimal*, not merely
that it is self-consistent.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py two-pointers/01-host-consolidation
=== python: two-pointers/01-host-consolidation ===
All tests passed.
=== rust: two-pointers/01-host-consolidation ===
running 8 tests
........
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
stubs intact: python=True rust=True
RESULT two-pointers/01-host-consolidation: PASS
```

- `reference.py` — verified, all Python asserts pass.
- `reference.rs` — verified, all 8 `cargo test` cases pass. Uses signed `isize`
  pointers so `hi` may fall to −1, mirroring the Python loop exactly.
- Naive infeasibility confirmed by direct timing (see above).
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
