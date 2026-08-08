# Gateway Peak Concurrency

**Difficulty:** Medium
**ID:** `intervals-sweep-line-01-gateway-peak-concurrency`

## Scenario

You are sizing the connection pool for an API gateway. From last week's access
log you have extracted, for every session the gateway handled, the millisecond
timestamp at which it opened and the timestamp at which it closed.

A session occupies a connection slot over the **half-open** interval
`[start, end)`: it holds the slot at instant `start`, and it has already
released it at instant `end`. So a session closing at `t` and another opening
at `t` **do not** overlap — the slot is handed straight over, and one
connection serves both.

You need two numbers to size the pool:

1. The **peak concurrency** — the largest number of sessions ever open
   simultaneously.
2. The **earliest instant** at which that peak was reached, so you can
   correlate it with the rest of your telemetry.

## Task

Implement:

```python
peak_concurrency(sessions: List[Tuple[int, int]]) -> Tuple[int, int]
```

Each entry of `sessions` is `(start, end)` with `start <= end`. Return
`(peak, earliest_time)`.

If no two sessions ever overlap and there is at least one session, the peak is
1. If there are no sessions at all — or every session is instantaneous, see
below — return `(0, 0)`.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `sessions` | `List[Tuple[int, int]]` | `0 <= n <= 200_000` |
| `start`, `end` | `int` | `0 <= start <= end <= 10^9` |

Two input details that are deliberate, not accidental:

- **Timestamps are not sorted**, and sessions may be listed in any order.
- **Zero-length sessions are legal.** A session with `start == end` occupies
  the interval `[t, t)`, which contains no instants at all — it never holds a
  slot and can never contribute to concurrency. These appear in real logs when
  a client disconnects inside the same millisecond it connected.

Timestamps range up to 10^9, so the timeline cannot be materialised as an
array.

## Output

`Tuple[int, int]` — `(peak, earliest_time)`. When the peak is 0, return
`(0, 0)`.

## Required complexity

- **Time:** `O(n log n)`.
- **Space:** `O(n)`.

Comparing every pair of sessions is `O(n²)` — about 2 × 10^10 comparisons at
`n = 200_000`. Bucketing by timestamp is also out: there are 10^9 possible
instants.

## Examples

### Example 1

```
sessions = [(1, 5), (2, 6), (8, 10)]
```

**Answer:** `(2, 2)`

Walking the timeline: at instant 1 only the first session is open (1 slot). At
instant 2 the second opens while the first is still running — 2 slots, and this
is the peak. At instant 5 the first closes (back to 1), at 6 the second closes
(0). The third session runs alone from 8 to 10, needing 1 slot. So the peak is
2, first reached at instant 2.

### Example 2 — handover at a shared timestamp

```
sessions = [(1, 5), (5, 9)]
```

**Answer:** `(1, 1)`

The first session releases its slot at instant 5 and the second claims it at
instant 5. Because intervals are half-open, they do **not** overlap: at instant
5 the first is already gone. One connection serves both sessions back to back,
so the peak is 1, first reached at instant 1 when the first session opens.

Treating the intervals as closed would wrongly report a peak of 2 here.

### Example 3 — nesting

```
sessions = [(0, 100), (10, 20), (12, 15)]
```

**Answer:** `(3, 12)`

A long session spans the whole window, a shorter one sits inside it, and a
shorter one still sits inside that. All three are open together from instant 12
(when the innermost opens) until instant 15 (when it closes). The peak is 3 and
it is first reached at 12.

### Example 4 — instantaneous sessions

```
sessions = [(5, 5), (5, 10)]
```

**Answer:** `(1, 5)`

The first session is zero-length: `[5, 5)` contains no instants, so it never
holds a slot. Only the second session ever occupies the pool, giving a peak of
1 from instant 5. Counting the zero-length session would wrongly report 2.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/intervals-sweep-line/01-gateway-peak-concurrency/EDITORIAL.md`.
