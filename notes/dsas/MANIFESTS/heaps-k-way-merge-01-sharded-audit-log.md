# Manifest — heaps-k-way-merge/01-sharded-audit-log

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `heaps-k-way-merge-01-sharded-audit-log` |
| Pattern | Min-heap k-way merge |
| Difficulty | Medium |
| Theme | Sharded audit log pagination |
| Generated | 2026-08-09 |
| Batch | B |

## Intended approach

Seed a min-heap with one `(timestamp, shard_index, position)` entry per
non-empty shard, then pop `offset + 1` times, pushing the popped shard's next
element after each pop. The answer is the last entry popped. The heap holds at
most `k` entries — one live candidate per shard — because the globally earliest
remaining event is always at some shard's front.

Ordering by the pair `(timestamp, shard_index)` implements the required tie
rule directly; `position` never participates in a tie since two entries from
one shard are never in the heap at once.

- **Optimal complexity:** O(k + offset · log k) time, O(k) space — independent
  of total event count `N`.
- **Naive it must beat:** rescan all `k` shard heads per step, O(offset · k).
  Measured 1.83 s at k = 8 000 / offset = 2 000, linear in both → ~38 minutes
  at k = 100 000 / offset = 200 000.
- **Also excluded by the statement:** concatenate-and-sort, O(N log N) time and
  O(N) memory. This is what the test oracle does, which is why it is an honest
  cross-check rather than a re-implementation.

## Bound design

The gap between `offset` (200 000) and `N` (2 000 000) is the whole recognition
signal — cost must scale with how far you page, not with how much data exists.
The bounds were chosen so that concatenate-and-sort is *feasible but wrong for
the stated contract*, while the per-step rescan is outright infeasible.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Returning the heap minimum after the loop instead of the last pop | **verified**: mutation fails **all 10** Rust tests |
| Heap ordered by timestamp alone, so ties resolve by heap layout | `ties`, `large_all_ties` — **verified**: reversing the tie order fails 4 of 10 |
| Round-robin tie handling instead of shard-index order | `ties`: `[[5,5],[5],[5]]` → `(5,0), (5,0), (5,1), (5,2)` |
| Empty shards not skipped when seeding (indexing `shard[0]`) | `edges`: `[[], [7], [], [3,9]]`, `[[],[],[]]` |
| Shard index reported after filtering empties | `[[],[],[8]]` → shard **2** |
| Offset past the end, and the empty-input cases | `[[1],[2]]` offset 5 → `None`; `[]` → `None`; `large_offset_past_end` |
| Pushing all of a shard's elements up front (O(N) memory) | `large_one_fat_shard`: one shard with 2M events |
| 32-bit timestamps | `edges`: timestamps at 10^18 |
| Per-step rescan of k heads | `large_many_shards` (k = 100 000), `large_all_ties` |
| Cost scaling with N rather than offset | `large_deep_offset` (N = 2 000 000), `large_one_fat_shard` |
| General correctness | 400 randomised shard sets, each probed at **every** valid offset plus one past the end, vs a materialise-and-sort oracle; identical LCG in Python and Rust |

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py heaps-k-way-merge/01-sharded-audit-log
=== python: heaps-k-way-merge/01-sharded-audit-log ===
All tests passed.
=== rust: heaps-k-way-merge/01-sharded-audit-log ===
running 10 tests
..........
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
stubs intact: python=True rust=True
RESULT heaps-k-way-merge/01-sharded-audit-log: PASS
```

- `reference.py` — verified; uses `heapq` with tuple ordering.
- `reference.rs` — verified; `BinaryHeap` with `Reverse` since it is a max-heap
  by default, and `collect()` to heapify in O(k).
- Both headline traps confirmed by deliberate mutation (recorded above).
- Naive infeasibility confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
