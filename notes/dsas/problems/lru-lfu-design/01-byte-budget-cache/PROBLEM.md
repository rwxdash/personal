# Byte-Budget Cache

**Difficulty:** Medium
**ID:** `lru-lfu-design-01-byte-budget-cache`

## Scenario

You are implementing the in-memory response cache in front of an object store.
Unlike a textbook cache that holds a fixed *number* of entries, this one has a
fixed **byte budget** — entries vary enormously in size, and what matters is
total memory, not entry count.

The eviction policy is least-recently-used. When an insert pushes the cache
over budget, the cache evicts the least recently used entry, then checks again,
and keeps evicting until it fits. A single large insert can therefore evict
several small entries at once.

Both reading and writing an entry count as *using* it: a successful read makes
that entry the most recently used, and so does a write.

One operational rule: an entry larger than the entire budget is **rejected**.
It is not stored, and — importantly — it does not evict anything on its way
out. A 10 GB object must not flush a 1 GB cache to make room for something
that was never going to fit.

## Task

Implement:

```python
cache_simulate(budget: int, ops: List[Tuple[int, int, int, int]]) -> List[int]
```

Each operation is a 4-tuple `(kind, key, value, size)`:

| `kind` | Operation | Fields used |
| --- | --- | --- |
| `0` | **GET** `key` | `key` (`value` and `size` are 0 and ignored) |
| `1` | **PUT** `key`, `value`, `size` | all four |

Return a list holding one entry per **GET** operation, in order: the stored
value, or `-1` if the key is not cached.

### PUT semantics

- If `size > budget`, the operation is rejected: the cache is left exactly as
  it was, including any existing entry under that key.
- Otherwise the key is stored with the new value and new size, becoming the
  most recently used entry. If the key was already present, its old size stops
  counting toward the total.
- Then, while the total size exceeds the budget, evict the least recently used
  entry.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `budget` | `int` | `0 <= budget <= 10^12` |
| `ops` | `List[Tuple[int, int, int, int]]` | `0 <= len(ops) <= 200_000` |
| `key`, `value` | `int` | `0 ..= 10^9` |
| `size` | `int` | `1 ..= 10^9` (only on PUT) |

A budget of `0` rejects every PUT, since every size is at least 1.

## Output

`List[int]` — one entry per GET, in the order the GETs appear.

## Required complexity

- **Time:** `O(len(ops))` overall — amortised constant work per operation.
- **Space:** `O(len(ops))`.

Scanning the cache to find the least recently used entry on each eviction is
`O(len(ops)^2)`, about 4 × 10^10 steps at the bounds.

## Examples

### Example 1 — eviction by bytes, not by count

```
budget = 10
ops = [(1, 1, 100, 6),    # PUT key 1, value 100, size 6
       (1, 2, 200, 3),    # PUT key 2, value 200, size 3   (total 9)
       (0, 1, 0, 0),      # GET key 1  -> 100, and key 1 becomes most recent
       (1, 3, 300, 4),    # PUT key 3, value 300, size 4   (total 13, over budget)
       (0, 2, 0, 0),      # GET key 2
       (0, 1, 0, 0)]      # GET key 1
```

**Answer:** `[100, -1, 100]`

After the first two puts the cache holds keys 1 and 2, totalling 9 bytes. The
GET of key 1 returns 100 **and makes key 1 the most recently used**, so key 2
is now the least recent.

Putting key 3 brings the total to 13, over the budget of 10. The least recently
used entry is key 2, so it is evicted, leaving 10 bytes — which fits. Key 2 is
gone (`-1`) and key 1 survives, because the earlier GET protected it.

### Example 2 — one insert evicting several entries

```
budget = 10
ops = [(1, 1, 10, 3), (1, 2, 20, 3), (1, 3, 30, 3),   # total 9
       (1, 4, 40, 9),                                  # total 18, over budget
       (0, 1, 0, 0), (0, 2, 0, 0), (0, 3, 0, 0), (0, 4, 0, 0)]
```

**Answer:** `[-1, -1, -1, 40]`

The size-9 insert takes the total to 18. Evicting key 1 leaves 15, still over.
Evicting key 2 leaves 12, still over. Evicting key 3 leaves 9, which fits. One
insert evicted three entries — eviction is a loop, not a single step.

### Example 3 — an oversized entry is rejected without evicting

```
budget = 10
ops = [(1, 1, 10, 8),
       (1, 2, 20, 11),    # size 11 > budget 10: rejected
       (0, 1, 0, 0), (0, 2, 0, 0)]
```

**Answer:** `[10, -1]`

Key 2's size exceeds the whole budget, so the PUT is rejected outright. Key 1
is untouched — it must **not** be evicted to make room for an entry that can
never be stored.

### Example 4 — overwriting a key releases its old size

```
budget = 10
ops = [(1, 1, 10, 8), (1, 2, 20, 2),   # total 10, exactly full
       (1, 1, 99, 3),                   # overwrite key 1: 8 bytes freed, 3 used
       (0, 1, 0, 0), (0, 2, 0, 0)]
```

**Answer:** `[99, 20]`

Overwriting key 1 replaces its old size of 8 with 3, so the total becomes 5 and
nothing needs evicting. Both keys survive. A replacement must release the old
size, not add to it.

### Example 5 — a miss does not change recency

```
budget = 6
ops = [(1, 1, 10, 3), (1, 2, 20, 3),
       (0, 9, 0, 0),               # GET a key that is not cached
       (1, 3, 30, 3),              # total 9, over budget
       (0, 1, 0, 0)]
```

**Answer:** `[-1, -1]`

The GET of key 9 misses and changes nothing. Key 1 is still the least recently
used, so the insert of key 3 evicts it.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/lru-lfu-design/01-byte-budget-cache/EDITORIAL.md`.
