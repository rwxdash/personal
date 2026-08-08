# Resource Ancestry

**Difficulty:** Medium
**ID:** `dfs-01-resource-ancestry`

## Scenario

Your cloud platform organises every resource — organisations, folders,
projects, buckets — into a hierarchy. Each resource has exactly one parent,
except top-level organisations, which have none. There may be several
organisations, so the hierarchy is a **forest**, not a single tree.

Permissions are inherited downward: a policy attached to a resource applies to
that resource and to everything beneath it. So the authorisation service
constantly asks the same question:

> Does a policy attached to resource `u` apply to resource `v`?

which is exactly: **is `u` an ancestor of `v`?** A resource counts as its own
ancestor, since a policy attached to a resource certainly applies to it.

The service answers this question hundreds of thousands of times per second
against a hierarchy that changes rarely. You are writing the batch evaluator:
one preprocessing pass over the hierarchy, then all the queries.

## Task

Implement:

```python
ancestor_queries(parent: List[int], queries: List[Tuple[int, int]]) -> List[bool]
```

- `parent[i]` — the parent of resource `i`, or `-1` if `i` is a root.
- `queries[j] = (u, v)` — is `u` an ancestor of `v`?

Return a list of booleans, one per query, in the same order.

`u` is an ancestor of `v` when `u == v`, or when `u` lies on the path from `v`
upward to its root. Resources in different trees are never ancestors of one
another.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `parent` | `List[int]` | `1 <= n <= 200_000`; each entry is `-1` or a valid index |
| `queries` | `List[Tuple[int, int]]` | `0 <= q <= 200_000`; both endpoints valid indices |

**Guaranteed a valid forest:** following `parent` pointers from any resource
always terminates at a root. There are no cycles, and no resource is its own
ancestor via a chain. A resource may have any number of children, and the
hierarchy may be a single chain up to 200 000 deep.

## Output

`List[bool]` of length `q`.

## Required complexity

- **Time:** `O(n + q)`.
- **Space:** `O(n)`.

Walking upward from `v` to check each query is `O(depth)` per query — about
4 × 10^10 steps for a deep hierarchy with 200 000 queries.

## Examples

### Example 1

```
parent  = [-1, 0, 0, 1]
queries = [(0, 3), (1, 3), (2, 3), (3, 0), (1, 1)]
```

The hierarchy is one tree:

```
      0
     / \
    1   2
    |
    3
```

**Answer:** `[True, True, False, False, True]`

- `(0, 3)` — 0 is the root, and 3 is beneath it. **True**
- `(1, 3)` — 3's parent is 1. **True**
- `(2, 3)` — 2 is 3's *uncle*, not an ancestor; the path from 3 upward is
  3 → 1 → 0 and never visits 2. **False**
- `(3, 0)` — the question is directional. 3 is a *descendant* of 0, not an
  ancestor. **False**
- `(1, 1)` — a resource is its own ancestor. **True**

### Example 2 — separate trees

```
parent  = [-1, 0, -1, 2]
queries = [(0, 1), (0, 3), (2, 3), (2, 1)]
```

Two trees: `0 → 1` and `2 → 3`.

**Answer:** `[True, False, True, False]`

Resources 0 and 1 are in one tree, 2 and 3 in another. Cross-tree queries are
always False, no matter how the indices are numbered.

### Example 3 — a chain

```
parent  = [-1, 0, 1, 2]
queries = [(0, 3), (1, 2), (2, 1), (3, 3)]
```

A single chain `0 → 1 → 2 → 3`.

**Answer:** `[True, True, False, True]`

Every resource is an ancestor of everything below it in the chain, and of
itself. `(2, 1)` is False because 1 sits *above* 2.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/dfs/01-resource-ancestry/EDITORIAL.md`.
