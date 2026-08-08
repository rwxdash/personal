# Maintenance Windows

**Difficulty:** Medium
**ID:** `dp-1d-01-maintenance-windows`

## Scenario

Your platform schedules disruptive maintenance — kernel patches, disk
replacements, firmware upgrades — in fixed hourly slots. The next `n` hours are
numbered `0 .. n-1`.

Running maintenance in slot `i` is worth `value[i]` to you (risk retired,
weighted by how overdue the work is). But maintenance destabilises the fleet,
so after running it in slot `i`, you must let the fleet settle: the next
maintenance may not start until slot `i + cooldown + 1`. In other words, at
least `cooldown` slots must sit empty between any two runs.

Some slots are **blackout** windows — a customer demo, a trading peak, a
release freeze — and maintenance can never run in them.

You may run maintenance as many times as the cooldown allows, or not at all.
Maximise the total value.

## Task

Implement:

```python
max_maintenance_value(value: List[int], blackout: List[bool], cooldown: int) -> int
```

Return the largest total value obtainable by choosing a set of slots such that:

- no chosen slot is a blackout slot, and
- any two chosen slots `i < j` satisfy `j - i > cooldown`.

Choosing nothing is allowed, so the answer is never negative.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `value` | `List[int]` | `0 <= n <= 200_000`; each value in `0 ..= 10^9` |
| `blackout` | `List[bool]` | same length as `value` |
| `cooldown` | `int` | `0 <= cooldown <= 200_000`; may exceed `n` |

A `cooldown` of `0` means there is no settling period at all and consecutive
slots may both be used. A `cooldown` of `n` or more means at most one slot can
ever be chosen.

The total can reach `2 * 10^14`, so 64-bit arithmetic is required.

## Output

`int` — the maximum total value. `0` if nothing can be chosen.

## Required complexity

- **Time:** `O(n)`.
- **Space:** `O(n)`.

Trying every subset of slots is `2^n`. Even at `n = 40` that is a trillion
combinations, and `n` goes to 200 000.

## Examples

### Example 1

```
value    = [5, 1, 8, 4]
blackout = [False, False, False, False]
cooldown = 1
```

**Answer:** `13`

A cooldown of 1 means no two chosen slots may be adjacent. The choices worth
considering:

| Chosen slots | Total |
| --- | --- |
| `{0, 2}` | 5 + 8 = **13** |
| `{0, 3}` | 5 + 4 = 9 |
| `{1, 3}` | 1 + 4 = 5 |
| `{2}` | 8 |

The best is slots 0 and 2, for 13. Note that slot 2 alone is worth more than
slot 0 and slot 3 together — taking the largest value first is not always
right, but here it happens to be part of the best answer.

### Example 2 — a blackout removes the best slot

```
value    = [5, 1, 8, 4]
blackout = [False, False, True, False]
cooldown = 1
```

**Answer:** `9`

Slot 2 is now unavailable, so the 8 cannot be collected at all. The best
remaining choice is slots 0 and 3, for 5 + 4 = 9.

### Example 3 — a longer cooldown

```
value    = [5, 1, 8, 4]
blackout = [False, False, False, False]
cooldown = 2
```

**Answer:** `9`

Choosing a slot now blocks the next two. Slots 0 and 3 are far enough apart
(`3 - 0 = 3 > 2`), giving 9. Slots 0 and 2 are no longer legal because
`2 - 0 = 2` is not greater than the cooldown of 2. Taking slot 2 alone gives
only 8.

### Example 4 — no cooldown

```
value    = [5, 1, 8, 4]
blackout = [False, False, False, False]
cooldown = 0
```

**Answer:** `18`

With no settling period, every non-blackout slot can be used, so the answer is
the sum of everything: 5 + 1 + 8 + 4 = 18.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/dp-1d/01-maintenance-windows/EDITORIAL.md`.
