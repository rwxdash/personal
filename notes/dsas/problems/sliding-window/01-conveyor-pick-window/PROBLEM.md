# Conveyor Pick Window

**Difficulty:** Medium
**ID:** `sliding-window-01-conveyor-pick-window`

## Scenario

In a fulfilment warehouse, items travel past a picking station on a single
conveyor belt. The belt cannot be paused, reversed, or restarted: the robot arm
is engaged once, and while it is engaged it grabs **every** item that passes.
So the set of items the arm collects is always a **contiguous stretch** of the
belt.

You are given the exact sequence of SKUs that will pass the station during this
shift, and one customer order that must be filled from this shift. The order
specifies a required quantity per SKU. Collecting **more** than the required
quantity of a SKU is fine — surplus is returned to stock afterwards — but
collecting fewer of any required SKU means the order cannot ship.

Every second the arm stays engaged costs throughput, so you want the **shortest
contiguous stretch of the belt that fills the whole order**.

## Task

Implement:

```python
shortest_fulfilling_run(stream: List[str], order: Dict[str, int]) -> Optional[Tuple[int, int]]
```

Return the inclusive `(start, end)` 0-based index pair of the shortest
contiguous stretch of `stream` that contains at least `order[sku]` occurrences
of every `sku` in `order`.

- If several stretches tie for the shortest length, return the one with the
  **smallest `start`**.
- If no stretch can fill the order, return `None`.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `stream` | `List[str]` | `1 <= len(stream) <= 200_000`; each SKU is a short string (<= 16 chars) |
| `order` | `Dict[str, int]` | non-empty; at most `10_000` distinct SKUs; every quantity is `>= 1`; the sum of all quantities is at most `200_000` |

SKUs appearing in `stream` but not in `order` are irrelevant filler — they can
be collected freely, they just do not help fill the order. SKUs appearing in
`order` may be entirely absent from `stream`.

## Output

`Optional[Tuple[int, int]]` — inclusive index pair, or `None`.

## Required complexity

- **Time:** `O(n)` expected, where `n = len(stream)`. `O(n log n)` is accepted.
- **Space:** `O(k)` extra, where `k` is the number of distinct SKUs in `order`.

At `n = 200_000`, any solution that independently re-examines the belt from
every possible starting index will not finish in reasonable time.

## Examples

### Example 1

```
stream = ["A", "B", "A", "C", "A", "B"]
order  = {"A": 2, "B": 1}
```

**Answer:** `(0, 2)`

The order needs 2 × A and 1 × B, so any stretch that fills it holds at least 3
items — length 3 is the floor. The stretch at indices `0..2` is `A, B, A`,
which supplies A×2 and B×1 exactly. Because no length-3 stretch starts earlier
and no shorter stretch can exist, `(0, 2)` is the answer. (Indices `1..3` are
`B, A, C` — only one A, so it fails. Indices `2..5` are `A, C, A, B`, which
does fill the order but is length 4.)

### Example 2

```
stream = ["A", "A", "B", "B"]
order  = {"A": 1, "B": 1}
```

**Answer:** `(1, 2)`

One of each is needed, so length 2 is the floor. Indices `1..2` are `A, B` —
the only length-2 stretch holding both. The stretch `0..2` (`A, A, B`) also
fills the order but is longer.

### Example 3 — tie broken by earliest start

```
stream = ["A", "B", "Z", "A", "B"]
order  = {"A": 1, "B": 1}
```

**Answer:** `(0, 1)`

Both `0..1` (`A, B`) and `3..4` (`A, B`) fill the order with length 2. The tie
rule selects the smaller start index, so `(0, 1)`. The filler SKU `Z` is never
required and never helps.

### Example 4 — impossible

```
stream = ["X", "Y"]
order  = {"X": 2}
```

**Answer:** `None`

The belt carries only one `X` during the entire shift, so no stretch — not even
the whole belt — can supply the two that the order requires.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/sliding-window/01-conveyor-pick-window/EDITORIAL.md`.
