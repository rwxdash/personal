# Manifest — dfs/01-resource-ancestry

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `dfs-01-resource-ancestry` |
| Pattern | DFS entry/exit times (Euler tour) → interval containment |
| Difficulty | Medium |
| Theme | Cloud IAM resource hierarchy / permission inheritance |
| Generated | 2026-08-09 |
| Batch | B |

## Intended approach

Build children lists and roots in one O(n) pass. Run a single **iterative**
depth-first traversal over the whole forest with one **global** counter,
stamping `tin[x]` on entry and `tout[x]` after all children finish. Then each
query is two comparisons:

```
u is an ancestor of v  ⟺  tin[u] <= tin[v] && tout[v] <= tout[u]
```

- **Optimal complexity:** O(n + q) time, O(n) space — O(1) per query after
  linear preprocessing.
- **Naive it must beat:** walk the parent chain per query, O(q · depth).
  Measured 0.558 s at n = q = 4 000 with 4×-per-doubling scaling → ~23 minutes
  at n = q = 200 000.
- **Correct but over-engineered:** binary lifting / LCA, O(n log n) preprocess
  and O(log n) queries — strictly worse than the interval test here.

## Why this cell is not "plain DFS"

Recorded because `INDEX.md` flags `dfs` as a pattern whose textbook form is
below the Medium floor. What carries it: the recognition that **ancestry is
interval containment**, which only works because subtrees are contiguous in
depth-first visit order. This is the one thing DFS provides that BFS
structurally cannot — a breadth-first walk scatters each subtree across
non-contiguous positions, so there is no interval to compare. The problem is
therefore genuinely about *which* traversal, not merely about traversing.

Deliberately distinct from `topological-sort/01` (which also touches DFS
territory): that problem uses Kahn's algorithm and only detects cycle
existence; this one is about linearising a forest.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Timer reset per root, so cross-tree intervals overlap | `cross_tree`, `large_forest` — **verified**: resetting per root fails 6 of 11; single-tree inputs never expose it |
| Strict `<` instead of `<=`, breaking `u == v` | `self_and_direction` — **verified**: strict comparisons fail 8 of 11 |
| Recursive DFS on a 200 000-deep chain | `large_deep_chain` — **verified**: `RecursionError` in CPython even with the limit raised to 20 000; 200 000 segfaults |
| Comparing only `tin` without `tout` | `edges` (star, siblings), `large_binary_tree` — a right-hand cousin looks like a descendant |
| Direction ignored (treating the relation as symmetric) | every `(u,v)` / `(v,u)` pair in `self_and_direction` |
| Assuming node 0 is a root | `edges`: `parent = [1, -1, 1]` |
| Assuming parents have smaller indices than children | `random_shuffled_labels`, which permutes all labels |
| Forest of all-roots | `edges`: `[-1] * 6`, full n×n query matrix |
| Empty query list | `ancestor_queries([-1, 0], [])` → `[]` |
| Per-query chain walk | `large_chain_many_queries` (200 000 queries on a 200 000-deep chain) |
| General correctness | 700 randomised forests (400 index-ordered + 300 label-permuted) vs a per-query upward-walk oracle; identical LCG in Python and Rust |

Large-case answers are closed-form, not reference-derived: chain ancestry is
`u <= v`, star ancestry is `u == 0`, and complete-binary-tree ancestry follows
the heap index rule.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py dfs/01-resource-ancestry
=== python: dfs/01-resource-ancestry ===
All tests passed.
=== rust: dfs/01-resource-ancestry ===
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
stubs intact: python=True rust=True
RESULT dfs/01-resource-ancestry: PASS
```

- `reference.py` / `reference.rs` — both verified; both iterative with an
  explicit `(node, returning)` stack.
- Both headline traps confirmed by deliberate mutation (recorded above).
- Recursion crash and naive infeasibility both confirmed by direct measurement.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
