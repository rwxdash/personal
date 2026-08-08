# Manifest — dp-on-trees/01-audit-sampling

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `dp-on-trees-01-audit-sampling` |
| Pattern | Dynamic programming on a tree (two states per node) |
| Difficulty | Medium |
| Theme | Compliance audit sampling over a service dependency tree |
| Generated | 2026-08-09 |
| Batch | C |

## Intended approach

Two values per node:

```
skip[v] = sum over children c of max(skip[c], take[c])
take[v] = evidence[v] + sum over children c of skip[c]
```

Answer is `max(skip[0], take[0])`. Leaves fall out with empty sums.

Computed with two flat passes: a stack walk from the root recording pop order
(every node lands after its parent), then that order processed in reverse
(every node after all of its children). This avoids both recursion and the
"return to a node after its children" bookkeeping of an iterative post-order.

- **Optimal complexity:** O(n) time, O(n) space.
- **Naive it must beat:** every subset, O(2^n · n). Measured 3.03 s at **n = 22**,
  quadrupling per two nodes added — centuries by n = 40, and n reaches 200 000.
- **Also fails:** recursion. Verified `RecursionError` in CPython at a 200 000-deep
  chain even with the limit raised to 30 000.

## Traps the tests target

| Trap | Test |
| --- | --- |
| `take[c]` instead of `max(skip[c], take[c])` when the parent is skipped | **verified**: mutation fails 4 of 12 Rust tests, including all 4 examples |
| Returning `take[0]` instead of `max(skip[0], take[0])` | **verified**: mutation fails 9 of 12 |
| Greedy by value | Example 1: `[10,6,6]` → 12, not 10 |
| Alternating by depth | Example 4: `[5,100,5]` → 100, not 10 |
| Recursion on a 200 000-deep chain | `large_chain`, `large_all_max`, `large_zero_root` |
| Looping over indices in reverse instead of a real traversal order | `edges`: `parent = [-1, 2, 0]`; `random_shuffled_labels` |
| 32-bit overflow | `large_all_max`: 10^14, asserted `> u32::MAX` |
| Leaves special-cased | star and wide-shallow cases |
| Exponential enumeration | seven `_test_large_*` cases at n = 131 071–200 000 |
| General correctness | 700 randomised trees (400 index-ordered + 300 label-permuted) vs full subset enumeration; identical LCG in Python and Rust |

Large-case answers are closed-form: alternate nodes on a uniform chain, even-depth
count on a perfect binary tree, hub-versus-leaves on stars.

## Correction during generation

`_test_edges` originally used `parent = [2, 2, -1]` to exercise "a parent's
index may exceed its child's". That input puts the root at index 2, which
**contradicts the problem's own contract** (`parent[0] == -1`, tree rooted at
0). Both language versions failed on it, correctly — the test was wrong, not the
solution.

Replaced with `parent = [-1, 2, 0]` (tree `0 → 2 → 1`), which keeps the root at
index 0 as the contract requires while still giving node 1 a parent with a
larger index. Expected values re-derived from the brute-force oracle: 20 and 14.

Per the repo rule, the test was fixed rather than the problem bent to fit it.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py dp-on-trees/01-audit-sampling
=== python: dp-on-trees/01-audit-sampling ===
All tests passed.
=== rust: dp-on-trees/01-audit-sampling ===
running 12 tests
............
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
stubs intact: python=True rust=True
RESULT dp-on-trees/01-audit-sampling: PASS
```

- All 17 hand-written expectations plus the 5 large-case formulas independently
  confirmed against subset enumeration.
- `reference.py` / `reference.rs` — both verified, both fully iterative.
- Both headline traps confirmed by deliberate mutation.
- Recursion crash and exponential blow-up both confirmed by direct measurement.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
