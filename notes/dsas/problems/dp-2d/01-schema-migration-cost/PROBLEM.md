# Schema Migration Cost

**Difficulty:** Medium
**ID:** `dp-2d-01-schema-migration-cost`

## Scenario

You are migrating a database table from its current column layout to a target
layout. Column **order matters** — the table's on-disk format is positional, so
the migration tool rewrites columns left to right and cannot reorder them.

The tool supports exactly three operations, each with its own cost in minutes
of downtime:

| Operation | Cost | Effect |
| --- | --- | --- |
| **Insert** | `insert_cost` | add a new column at the current position |
| **Drop** | `drop_cost` | remove the column at the current position |
| **Retype** | `retype_cost` | change the column at the current position into a different one |

A column that already matches what the target wants at that position costs
nothing — the tool leaves it alone.

The three costs are set independently by your storage engine's characteristics,
and they are frequently very different from each other. In particular, do not
assume retyping is cheaper than dropping and re-inserting.

Find the cheapest sequence of operations that turns the current layout into the
target layout.

## Task

Implement:

```python
migration_cost(current: List[str], target: List[str], insert_cost: int, drop_cost: int, retype_cost: int) -> int
```

Return the minimum total downtime in minutes.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `current` | `List[str]` | `0 <= len(current) <= 2_000`; column names, up to 32 chars |
| `target` | `List[str]` | `0 <= len(target) <= 2_000` |
| `insert_cost` | `int` | `0 ..= 10^6` |
| `drop_cost` | `int` | `0 ..= 10^6` |
| `retype_cost` | `int` | `0 ..= 10^6` |

Column names may repeat within a layout. Either list may be empty.

The total can reach about `4 * 10^9`, so 64-bit arithmetic is required.

## Output

`int` — the minimum total cost.

## Required complexity

- **Time:** `O(n · m)` where `n = len(current)` and `m = len(target)`.
- **Space:** `O(min(n, m))`.

Note the space bound: a full `n × m` table is **not** acceptable. At the stated
sizes that is 4 million entries, and the bound asks you to keep only what you
actually need.

## Examples

### Example 1 — equal costs

```
current     = ["id", "name", "email"]
target      = ["id", "email"]
insert_cost = 1, drop_cost = 1, retype_cost = 1
```

**Answer:** `1`

Keep `id` (it matches at position 0, free). Drop `name` for 1. Then `email`
matches. Total 1.

### Example 2 — retyping is expensive

```
current     = ["a"]
target      = ["b"]
insert_cost = 1, drop_cost = 1, retype_cost = 10
```

**Answer:** `2`

The obvious move is to retype `a` into `b` for 10. But dropping `a` (cost 1)
and inserting `b` (cost 1) achieves the same thing for 2. When
`retype_cost > drop_cost + insert_cost`, retyping is never worth doing.

### Example 3 — retyping is cheap

```
current     = ["a", "b"]
target      = ["x", "y"]
insert_cost = 5, drop_cost = 5, retype_cost = 1
```

**Answer:** `2`

Retype `a` into `x` and `b` into `y`, for 1 each. Dropping and re-inserting
both columns would cost 20.

### Example 4 — one side empty

```
current     = ["a", "b", "c"]
target      = []
insert_cost = 7, drop_cost = 2, retype_cost = 3
```

**Answer:** `6`

Every column must go, and dropping is the only way to remove one, so the cost
is 3 drops at 2 each. Symmetrically, migrating from an empty layout to a
3-column one would cost 3 inserts.

### Example 5 — matches are free but position-dependent

```
current     = ["a", "b"]
target      = ["b"]
insert_cost = 1, drop_cost = 1, retype_cost = 1
```

**Answer:** `1`

Drop `a` (cost 1), and then `b` lines up with the target's `b` for free. The
alternative — retype `a` into `b` (1) and drop `b` (1) — costs 2. A column
matching by name is only free if it also ends up in the right *position*.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/dp-2d/01-schema-migration-cost/EDITORIAL.md`.
