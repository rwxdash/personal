# Compaction Windows

**Difficulty:** Medium
**ID:** `binary-search-on-answer-01-compaction-windows`

## Scenario

A log-structured store accumulates immutable segments on disk, one after
another in strict chronological order. Overnight, a compaction job rewrites
them: it walks the segments in order and groups **consecutive** segments into
compaction windows, rewriting each window as a single file. Segments can never
be reordered — the store's recovery protocol depends on chronological order —
so every window is a contiguous run of segments.

You may run at most `k` windows tonight. The job's wall-clock time is dominated
by its largest window, since windows run in parallel but the slowest one gates
completion. So you want to **minimise the number of bytes in the largest
window**.

One complication. Some segments are **checkpoints** — written just after a
schema migration. A checkpoint segment must be the *first* segment of whatever
window contains it: the compactor cannot merge a checkpoint into a window that
started under the old schema. (The very first segment always begins the first
window, so a checkpoint flag on segment 0 tells you nothing new and is ignored.)

## Task

Implement:

```python
min_window_bytes(sizes: List[int], checkpoints: List[bool], k: int) -> int
```

Split `sizes` into **at most `k`** contiguous, non-empty windows covering every
segment in order, such that no checkpoint segment appears anywhere except as
the first segment of its window. Return the smallest achievable value of the
largest window's total bytes.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `sizes` | `List[int]` | `1 <= n <= 200_000`; each value in `1 ..= 10^9` |
| `checkpoints` | `List[bool]` | same length as `sizes`; index 0 is ignored |
| `k` | `int` | `1 <= k <= n` |

**Guaranteed solvable:** the number of checkpoints at indices `1 .. n-1` is at
most `k - 1`, so a valid split always exists.

The total byte count can reach `2 * 10^14`, so 64-bit arithmetic is required.

## Output

`int` — the minimised largest-window byte total.

## Required complexity

- **Time:** `O(n log T)` where `T` is the total byte count.
- **Space:** `O(1)` beyond the input.

At `n = 200_000` a dynamic program over (segment, window count) is
`O(n² k)` — utterly out of reach.

## Examples

### Example 1

```
sizes       = [7, 2, 5, 10, 8]
checkpoints = [False, False, False, False, False]
k           = 2
```

**Answer:** `18`

The four ways to cut into 2 windows:

| Split | Window totals | Largest |
| --- | --- | --- |
| `[7] [2,5,10,8]` | 7, 25 | 25 |
| `[7,2] [5,10,8]` | 9, 23 | 23 |
| `[7,2,5] [10,8]` | 14, 18 | **18** |
| `[7,2,5,10] [8]` | 24, 8 | 24 |

The best is 18. Note that balancing the *counts* of segments is not the goal —
balancing the bytes is.

### Example 2 — more windows available

```
sizes       = [7, 2, 5, 10, 8]
checkpoints = [False, False, False, False, False]
k           = 3
```

**Answer:** `14`

Splitting `[7,2,5] [10] [8]` gives totals 14, 10, 8. No split can do better:
the segment of size 10 forces every answer to be at least 10, and no
arrangement into 3 windows achieves anything between 10 and 14 — trying to
cap the largest window at 13 forces 4 windows.

### Example 3 — a checkpoint forces a cut

```
sizes       = [7, 2, 5, 10, 8]
checkpoints = [False, False, True, False, False]
k           = 3
```

**Answer:** `15`

Segment 2 (size 5) is a checkpoint, so a window must begin exactly there. That
rules out Example 2's winning split `[7,2,5] [10] [8]`, because it buries
segment 2 in the middle of the first window. The best legal split is
`[7,2] [5,10] [8]` with totals 9, 15, 8 — largest 15. A single forced cut has
made the answer worse.

### Example 4 — a window per segment

```
sizes       = [4, 9, 2]
checkpoints = [False, False, False]
k           = 3
```

**Answer:** `9`

With as many windows as segments, each segment sits alone and the largest
window is just the largest segment. No answer can ever be below `max(sizes)`,
since some window must contain that segment.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/binary-search-on-answer/01-compaction-windows/EDITORIAL.md`.
