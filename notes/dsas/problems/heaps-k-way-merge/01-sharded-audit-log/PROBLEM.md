# Sharded Audit Log

**Difficulty:** Medium
**ID:** `heaps-k-way-merge-01-sharded-audit-log`

## Scenario

Your audit log is sharded across many storage nodes. Each shard holds the
events it captured, already sorted by timestamp — shards are append-only and
events arrive in order, so every shard is individually sorted.

The compliance console pages through the **globally merged** view: all events
from all shards, ordered by timestamp. When two events share a timestamp, the
one from the **lower-numbered shard** comes first — an arbitrary but fixed rule
so that pagination is stable across requests.

A page request arrives as an offset. You must answer it **without materialising
the merged view**, because the shards together hold far more events than the
console will ever scroll through.

## Task

Implement:

```python
nth_merged_event(shards: List[List[int]], offset: int) -> Optional[Tuple[int, int]]
```

Return the event at position `offset` (0-indexed) in the globally merged order,
as a `(timestamp, shard_index)` pair. If the shards hold `offset` or fewer
events in total, return `None`.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `shards` | `List[List[int]]` | `0 <= k <= 100_000` shards |
| each shard | `List[int]` | sorted non-decreasing; may be empty; timestamps in `0 ..= 10^18` |
| total events | | `0 <= N <= 2_000_000` across all shards |
| `offset` | `int` | `0 <= offset <= 200_000` |

Timestamps may repeat, both within a shard and across shards. Shards may be
empty, and the whole list may be empty.

Note that `offset` is bounded far below `N`: the console never scrolls deep, so
your cost should scale with how far you page, not with how much data exists.

## Output

`Optional[Tuple[int, int]]` — `(timestamp, shard_index)`, or `None` when the
offset is past the end.

## Required complexity

- **Time:** `O(k + offset · log k)`.
- **Space:** `O(k)`.

Re-scanning every shard's current head on each step is `O(offset · k)` — about
2 × 10^10 operations at the stated bounds. Concatenating everything and sorting
is `O(N log N)` and needs `O(N)` memory, which the "without materialising"
requirement rules out.

## Examples

### Example 1

```
shards = [[10, 40], [20, 30], [50]]
offset = 2
```

**Answer:** `(30, 1)`

The merged order is:

| Position | Timestamp | Shard |
| --- | --- | --- |
| 0 | 10 | 0 |
| 1 | 20 | 1 |
| 2 | **30** | **1** |
| 3 | 40 | 0 |
| 4 | 50 | 2 |

Position 2 holds timestamp 30, which came from shard 1.

### Example 2 — ties break by shard index

```
shards = [[5, 5], [5], [5]]
offset = 1
```

**Answer:** `(5, 0)`

Every event has timestamp 5, so the tie rule decides the whole ordering: both
of shard 0's events come first, then shard 1's, then shard 2's. The merged
order is `(5,0), (5,0), (5,1), (5,2)`, so position 1 is shard 0's second event.

Note that shard 0 contributes *both* its events before shard 1 contributes any.
The rule orders by shard index, not by round-robin across shards.

### Example 3 — empty shards are skipped

```
shards = [[], [7], [], [3, 9]]
offset = 0
```

**Answer:** `(3, 3)`

Two shards are empty and contribute nothing. The merged order is
`(3,3), (7,1), (9,3)`, so position 0 is timestamp 3 from shard 3.

### Example 4 — past the end

```
shards = [[1], [2]]
offset = 5
```

**Answer:** `None`

There are only 2 events in total, so positions 0 and 1 exist and everything
beyond is out of range.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/heaps-k-way-merge/01-sharded-audit-log/EDITORIAL.md`.
