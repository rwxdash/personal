# Manifest — union-find/01-mesh-link-failures

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `union-find-01-mesh-link-failures` |
| Pattern | Union-find (DSU) + offline reverse-time processing |
| Difficulty | Medium–Hard |
| Theme | Service mesh chaos engineering / link failure fragmentation |
| Generated | 2026-08-08 |

## Intended approach

Union-find supports merges but not splits, so process the failure timeline in
reverse. Phase 1: mark doomed link indices, then union every surviving link to
build `S_q` (the state after all failures), maintaining `count` initialised to
`n` and decremented only on a genuine merge. Phase 2: record
`answer[q-1] = count`, then for `k` from `q-1` down to `1`, union
`links[failures[k]]` — putting back the link that caused the `S_k → S_{k+1}`
transition — and record `answer[k-1] = count`.

Redundant cables and self-loops require no special handling: both hit
`find(u) == find(v)`, so they never decrement the counter.

- **Optimal complexity:** O((n + m) α(n)) time, O(n + m) space.
- **Naive it must beat:** rebuild + flood-fill after each failure,
  O(q·(n + m)). Measured 0.53 s at n = 2 000 / q = 1 000 with clean quadratic
  scaling → ~88 minutes at n = 200 000 / q = 100 000.
- **Also acceptable:** O((n + m) log n) — DSU without path compression, or with
  union by rank only.
- **Wrong-tool trap:** online dynamic connectivity (Holm–de Lichtenberg–Thorup)
  solves it but is disproportionate; the statement explicitly grants offline
  access to the whole failure sequence.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Failures treated as endpoint pairs instead of link indices | `_test_examples`: `[(0,1),(0,1)]` → `[1, 2]`, not `[2, 2]` |
| Redundant cables at scale | `_test_large_duplicate_cables` → all 1s |
| Off-by-one in the reverse index mapping | `[(0,1),(1,2),(2,3)]` with `failures=[1,0,2]` → `[2, 3, 4]` |
| Self-loops given special handling that also shifts the initial count | `[(0,0),(1,2)]` → `[2, 3]`; `_test_large_self_loops` → all `n` |
| Phase 1 started from an empty graph instead of surviving links | `_test_large_untouched_backbone` → all 1s |
| Empty `failures` (underflow on `answer[q-1]`) | `partitions_after_failures(5, …, [])` → `[]` |
| Isolated nodes not counted as partitions | `partitions_after_failures(6, [(0,1)], [0])` → `6` |
| Cutting a link inside a cycle assumed to fragment | triangle cases → `[1]`, `[1,2]`, `[1,2,3]`; `_test_large_cycle` |
| Failures given out of index order | `[3,0,2,1]` on a 5-node path → `[2,3,4,5]` |
| Recomputing the count by scanning roots (O(n) per step) | all six `_test_large_*` cases |
| Recursive `find` / missing union by size | `_test_large_star_shatter`, `_test_large_chain_scrambled` |
| Quadratic rebuild | `_test_large_chain_scrambled`: n = 200 000, q = 100 000 |
| General correctness | 300 randomised graphs with duplicate edges, self-loops and shuffled failure subsets, cross-checked against a flood-fill oracle; identical LCG in Python and Rust |

Note on the large-case expectations: they are pinned by graph theory, not by
the solution. Cutting `k` distinct edges of a path always yields `k + 1`
components regardless of cut order (so the scrambled failure order is safe);
cutting `k` edges of a cycle yields `max(1, k)`; a star peels one leaf per cut.

## Verification record

**Date:** 2026-08-08

```
$ python3 verify.py union-find/01-mesh-link-failures
=== python: union-find/01-mesh-link-failures ===
All tests passed.
=== rust: union-find/01-mesh-link-failures ===
running 9 tests
.........
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
stubs intact: python=True rust=True
RESULT union-find/01-mesh-link-failures: PASS
```

- `reference.py` — verified, all Python asserts pass.
- `reference.rs` — verified, all 9 `cargo test` cases pass.
- Naive infeasibility confirmed by direct timing (see above).
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
