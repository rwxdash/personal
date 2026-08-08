# Autoscaler Stable Windows

**Difficulty:** Medium–Hard
**ID:** `monotonic-queue-01-autoscaler-stable-windows`

## Scenario

Your autoscaler samples a service's CPU utilisation once per tick and appends
the reading to a time series. Before it is allowed to change the replica count,
it must first convince itself that the service is *stable*: it looks at a
contiguous run of consecutive samples and declares that run **stable** when the
spread between the highest and lowest reading in the run is at most `delta`.
The autoscaler also refuses to trust very short runs, so a run only counts if
it holds at least `min_len` samples.

You are auditing the autoscaler's sensitivity. Given a full shift of readings,
you want to know **how many distinct stable runs exist** — that is, how many
contiguous index ranges of the series satisfy both conditions. Runs are counted
by their position, so `samples[3..7]` and `samples[4..8]` are two different
runs even if they contain the same values.

## Task

Implement:

```python
count_stable_windows(usage: List[int], delta: int, min_len: int) -> int
```

Return the number of contiguous ranges `usage[i..j]` (inclusive, `i <= j`)
satisfying both:

- `max(usage[i..j]) - min(usage[i..j]) <= delta`
- `j - i + 1 >= min_len`

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `usage` | `List[int]` | `0 <= len(usage) <= 200_000`; each reading in `0 ..= 10^9` |
| `delta` | `int` | `0 <= delta <= 10^9` |
| `min_len` | `int` | `1 <= min_len <= 200_000` (may exceed `len(usage)`) |

## Output

`int` — the count. It can be as large as about `2 * 10^10`, so a 64-bit
accumulator is required (in Rust the return type is `u64`; in Python integers
are unbounded).

## Required complexity

- **Time:** `O(n)` expected. `O(n log n)` is accepted.
- **Space:** `O(n)` extra.

At `n = 200_000` there are ~2 × 10^10 candidate ranges, so enumerating them —
even at O(1) per range — is out of reach. The answer must be assembled without
ever visiting individual ranges.

## Examples

### Example 1

```
usage = [5, 7, 6, 9], delta = 2, min_len = 1
```

**Answer:** `7`

Every range and its spread:

| Range | Values | max − min | Stable? |
| --- | --- | --- | --- |
| `0..0` | 5 | 0 | ✅ |
| `1..1` | 7 | 0 | ✅ |
| `2..2` | 6 | 0 | ✅ |
| `3..3` | 9 | 0 | ✅ |
| `0..1` | 5, 7 | 2 | ✅ |
| `1..2` | 7, 6 | 1 | ✅ |
| `2..3` | 6, 9 | 3 | ❌ |
| `0..2` | 5, 7, 6 | 2 | ✅ |
| `1..3` | 7, 6, 9 | 3 | ❌ |
| `0..3` | 5, 7, 6, 9 | 4 | ❌ |

Four single-sample runs plus `0..1`, `1..2` and `0..2` gives 7. Note that every
single sample is trivially stable when `min_len = 1`, since its spread is 0.

### Example 2 — the same series with a length floor

```
usage = [5, 7, 6, 9], delta = 2, min_len = 2
```

**Answer:** `3`

The same table as above, minus the four length-1 runs: only `0..1`, `1..2` and
`0..2` survive. Raising `min_len` never makes a run stable that was not stable
before — it only removes short ones from the tally.

### Example 3 — zero tolerance

```
usage = [4, 4, 4, 1], delta = 0, min_len = 1
```

**Answer:** `7`

With `delta = 0` a run is stable only if every reading in it is identical. The
leading block of three 4s contributes its 3 single runs, its 2 adjacent pairs
(`0..1`, `1..2`), and the triple `0..2` — that is 6. The trailing `1` at index
3 contributes its own single run. No range spanning index 2 and index 3 is
stable, because it holds both a 4 and a 1. Total 7.

### Example 4 — nothing qualifies

```
usage = [1, 2, 3], delta = 10, min_len = 5
```

**Answer:** `0`

Every range is well within the tolerance, but the series is only 3 samples
long, so no range can reach the required length of 5.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/monotonic-queue/01-autoscaler-stable-windows/EDITORIAL.md`.
