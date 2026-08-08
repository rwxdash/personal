# Peak Dominance

**Difficulty:** Medium
**ID:** `monotonic-stack-01-peak-dominance`

## Scenario

A CDN edge node reports its egress bandwidth once per minute, producing a
series of readings. Your capacity dashboard wants to annotate every sample with
how significant that sample's spike was — specifically, **how wide a stretch of
time that sample dominated.**

A sample *dominates* a contiguous stretch of minutes if it lies inside that
stretch and no reading in the stretch is strictly higher than it. For each
sample, report the length of the **widest** stretch it dominates.

Ties count as dominated: a sample that equals its neighbour does not lose to
it. Only a strictly higher reading ends a sample's reign.

## Task

Implement:

```python
dominance_spans(load: List[int]) -> List[int]
```

Return a list of the same length as `load`, where element `i` is the number of
samples in the widest contiguous stretch containing index `i` in which
`load[i]` is greater than or equal to every reading.

Equivalently: find the nearest index to the left of `i` holding a **strictly
greater** value, and the nearest such index to the right. The answer for `i` is
the number of samples strictly between them. When no strictly greater reading
exists on a side, the stretch runs to that end of the series.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `load` | `List[int]` | `0 <= n <= 200_000`; each value in `0 ..= 10^9` |

Readings repeat frequently — flat plateaus are the normal case for this metric,
not an edge case.

## Output

`List[int]` of length `n`. Every entry is at least 1 (a sample always dominates
at least itself).

## Required complexity

- **Time:** `O(n)`.
- **Space:** `O(n)`.

At `n = 200_000`, expanding outward from each sample individually costs on the
order of 2 × 10^10 steps in the worst case.

## Examples

### Example 1

```
load = [2, 1, 3]
```

**Answer:** `[2, 1, 3]`

- Index 0 (value 2): nothing lies to its left. To the right, index 1 holds 1,
  which does not exceed 2, so the stretch extends over it; index 2 holds 3,
  which is strictly greater and ends the reign. Widest stretch `[0, 1]`,
  length 2.
- Index 1 (value 1): index 0 holds 2 > 1 and index 2 holds 3 > 1, so it is
  boxed in on both sides and dominates only itself. Length 1.
- Index 2 (value 3): it is the maximum of the whole series, so nothing strictly
  exceeds it anywhere and it dominates all 3 samples. Length 3.

### Example 2 — a plateau

```
load = [5, 5, 5]
```

**Answer:** `[3, 3, 3]`

No reading is strictly greater than any other, so every sample dominates the
entire series. This is the case that distinguishes "strictly greater" from
"greater or equal": if ties broke a reign, the answer would be `[1, 1, 1]`.

### Example 3 — a mountain

```
load = [1, 2, 5, 2, 1]
```

**Answer:** `[1, 2, 5, 2, 1]`

- Index 0 (value 1): index 1 holds 2 > 1, so the reign ends immediately to the
  right; nothing to the left. Length 1.
- Index 1 (value 2): index 2 holds 5 > 2 on the right. To the left, index 0
  holds 1, which does not exceed it, and the series ends. Stretch `[0, 1]`,
  length 2.
- Index 2 (value 5): the series maximum, dominates everything. Length 5.
- Index 3 (value 2): index 2 holds 5 > 2 on the left. To the right, index 4
  holds 1, then the series ends. Stretch `[3, 4]`, length 2.
- Index 4 (value 1): index 3 holds 2 > 1 on the left; nothing to the right.
  Length 1.

### Example 4 — a valley between equal peaks

```
load = [4, 1, 4]
```

**Answer:** `[3, 1, 3]`

Both 4s dominate the whole series: neither is strictly exceeded anywhere, and
the 1 between them does not break the stretch. The 1 is boxed in by strictly
greater values on both sides, so it dominates only itself.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/monotonic-stack/01-peak-dominance/EDITORIAL.md`.
