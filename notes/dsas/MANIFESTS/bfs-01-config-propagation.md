# Manifest — bfs/01-config-propagation

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `bfs-01-config-propagation` |
| Pattern | Multi-source BFS with absorbing nodes |
| Difficulty | Medium |
| Theme | Datacentre gossip config rollout |
| Generated | 2026-08-09 |
| Batch | B |

## Intended approach

Seed the BFS queue with **every** seed node at distance 0, then run one
ordinary BFS. Equivalent to a virtual node joined to all seeds; since BFS
dequeues in non-decreasing distance order, the first sighting of a node already
carries the minimum round across all seeds.

The quarantine check goes at the **expansion** step, after dequeuing: a node's
round is written when it is *enqueued*, so a `continue` after the pop suppresses
propagation while preserving the node's own round. This also makes a
quarantined seed work with no special case.

`round_of` doubles as the visited marker (`-1` = unreached = the required
output for unreachable nodes), so the two can never disagree.

- **Optimal complexity:** O(n + m) time and space.
- **Naive it must beat:** one BFS per seed plus a per-node minimum,
  O(S · (n + m)). Measured 8.30 s at n = 8 000 / 4 000 seeds with
  4×-per-doubling scaling → ~86 minutes at n = 200 000 / 100 000 seeds.

## Why this cell is not "plain BFS"

Recorded because `INDEX.md` flags `bfs` as a pattern whose textbook form is
below the Medium floor. Two twists carry it: **multi-source** (which is what
makes it O(n+m) instead of O(S·(n+m))), and **absorbing nodes** (which forces
precision about *where* in the loop a condition is tested). Neither is
decoration — each is independently mutation-tested below.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Quarantine checked at enqueue time, losing the node's own round | `quarantine_semantics`, `large_quarantine_wall` — **verified**: moving the check to the enqueue side fails 4 of 12, including all 4 examples |
| Single-source instead of multi-source | `multi_source`, `large_many_seeds` — **verified**: using only the first seed fails 7 of 12 |
| Quarantined seed special-cased (or treated as a non-seed) | Example 4: `[0, -1, -1]`; `large_all_quarantined` |
| Quarantined node skipped entirely rather than absorbed | `[(0,1)]`, seed 0, quarantined 1 → `[0, 1]` not `[0, -1]` |
| DFS / wrong visit order | `large_chain_both_ends`, where every round is a true minimum |
| Recursion on a 200 000-node chain | `large_chain` |
| Duplicate seeds | `multi_source`: `[0,0,0]` → `[0,1,2]` |
| Self-loops and duplicate links | `edges`: `[(0,0),(0,1),(0,1),(1,2)]` → `[0,1,2]` |
| Empty seed list | `edges` → all `-1` |
| Disconnected components | `edges`, `large_disconnected` |
| Unreachable reported as 0 rather than −1 | every `-1` assertion |
| Per-seed quadratic blow-up | `large_many_seeds` (100 000 seeds), `large_disconnected` |
| General correctness | 400 randomised graphs with duplicate edges, self-loops, duplicate/empty seed lists and ~25% quarantine density, vs an order-agnostic fixpoint relaxation oracle; identical LCG in Python and Rust |

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py bfs/01-config-propagation
=== python: bfs/01-config-propagation ===
All tests passed.
=== rust: bfs/01-config-propagation ===
running 12 tests
............
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
stubs intact: python=True rust=True
RESULT bfs/01-config-propagation: PASS
```

- `reference.py` / `reference.rs` — both verified.
- Both headline traps confirmed by deliberate mutation (recorded above).
- Naive infeasibility confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
