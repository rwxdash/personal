# Manifest — topological-sort/01-pipeline-critical-path

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `topological-sort-01-pipeline-critical-path` |
| Pattern | Topological sort (Kahn) + DP on a DAG |
| Difficulty | Medium–Hard |
| Theme | CI/CD build pipeline scheduling |
| Generated | 2026-08-08 |

## Intended approach

Kahn's algorithm with the DP relaxation fused into the traversal. Build
successor lists and in-degrees counting every `deps` entry once in both
structures (no deduplication). Initialise `start[i] = ready[i]`, seed the queue
with zero-in-degree nodes. On popping `u`, its `start` is final — a node is
enqueued only after every incoming edge has been relaxed — so
`finish_u = start[u] + durations[u]` is final; fold it into a running max over
*all* nodes, then relax each successor with
`start[v] = max(start[v], finish_u)` and decrement its in-degree. If fewer than
`n` nodes are processed, the graph is cyclic → `None`.

- **Optimal complexity:** O(n + m) time and space.
- **Naive it must beat:**
  - Enumerate all routes: ~2^499 paths on the layered test graph. Infeasible,
    not merely slow.
  - Recursive memoised DFS: correct and linear, but dies on the 200 000-deep
    chain — verified `RecursionError` even with the limit raised to 10 000;
    raising it to 200 000 crashes CPython.
  - Bellman-Ford-style relaxation to fixpoint: round count depends on the edge
    listing order. Measured 4.34 s at n = 8 000 on a back-to-front chain with
    4×-per-doubling scaling → ~45 minutes at n = 200 000.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Summing durations instead of maximising over routes | `_test_examples`: `deps=[]` → 4, not 9 |
| Answer taken from sinks only | `_test_large_disconnected_late_job` → 10^9 + 1 |
| `ready` applied only to source nodes | `_test_large_chain_with_late_artifact` → 10^9 + 50 000 |
| `max(ready, preds) + dur` vs `max(ready, preds + dur)` | `[10,1],[0,5],[(0,1)]` → 11 |
| Duplicate edges corrupting in-degree bookkeeping | `[(0,1),(0,1),(0,1)]` → 5, not `None` |
| Self-loop filtered out during graph build | `[(0,0)]` → `None` |
| Cycle in a component with no zero-in-degree entry | `[(0,1),(1,2),(2,1)]` → `None` |
| Cycle detection via "queue emptied" rather than `processed < n` | all four `_test_cycles` cases |
| Recursion depth | `_test_large_deep_chain`, 200 000-deep |
| Reliance on edge-list ordering / relaxation to fixpoint | `_test_large_reversed_chain` |
| 32-bit overflow | `[10^9, 10^9]` with `ready=[10^9,0]` → 3 × 10^9 |
| Zero-duration gate jobs treated as skippable | `[0,0,3]` chain → 3 |
| Exponential path enumeration | `_test_large_wide_layers`, 399 200 edges |
| General correctness | 300 randomised graphs (≈⅓ cyclic) + 300 randomised DAGs, cross-checked against a fixpoint oracle guarded by an independent O(n(n+m)) reachability cycle check; identical LCG in Python and Rust |

Note on the oracle: the cycle check *cannot* be folded into the fixpoint. A
cycle whose jobs all have duration 0 reaches a fixpoint immediately, so
"values stopped moving" does not imply "schedulable". The random generator
produces zero durations, so this case is actually exercised.

## Verification record

**Date:** 2026-08-08

```
$ python3 verify.py topological-sort/01-pipeline-critical-path
=== python: topological-sort/01-pipeline-critical-path ===
All tests passed.
=== rust: topological-sort/01-pipeline-critical-path ===
running 12 tests
............
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
stubs intact: python=True rust=True
RESULT topological-sort/01-pipeline-critical-path: PASS
```

- `reference.py` — verified, all Python asserts pass.
- `reference.rs` — verified, all 12 `cargo test` cases pass.
- Naive infeasibility confirmed by direct measurement (recursion crash and
  reversed-chain fixpoint timing, both recorded above).
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
