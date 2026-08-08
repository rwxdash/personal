# Audit Sampling

**Difficulty:** Medium
**ID:** `dp-on-trees-01-audit-sampling`

## Scenario

Your compliance team audits services in a dependency tree. Service 0 is the
root; every other service depends on exactly one parent, so the whole estate
forms a single tree.

Auditing a service produces `evidence[i]` points of audit coverage. But an
audit of a service also inspects its **immediate dependency relationships**, so
auditing a service and auditing its direct parent produces almost entirely
duplicated paperwork — the compliance framework forbids it outright.

So you may audit any set of services, as long as **no audited service is the
direct parent of another audited service**. Services two levels apart are fine;
siblings are fine. Only a direct parent-child pair is forbidden.

Maximise the total evidence collected.

## Task

Implement:

```python
max_audit_evidence(parent: List[int], evidence: List[int]) -> int
```

- `parent[i]` — the parent of service `i`. `parent[0]` is always `-1`; every
  other entry is a valid service index.
- `evidence[i]` — audit coverage points from auditing service `i`.

Return the maximum total evidence.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `parent` | `List[int]` | `1 <= n <= 200_000`; `parent[0] == -1`, others valid indices |
| `evidence` | `List[int]` | same length; each value in `0 ..= 10^9` |

**Guaranteed a valid tree** rooted at 0: following parent pointers from any
service reaches service 0, and there are no cycles. A service may have any
number of children, and the tree may be a single chain 200 000 deep.

Note that a parent's index is **not** guaranteed to be smaller than its
children's.

The total can reach `2 * 10^14`, so 64-bit arithmetic is required.

## Output

`int` — the maximum total evidence. Auditing nothing is allowed, so the answer
is never negative.

## Required complexity

- **Time:** `O(n)`.
- **Space:** `O(n)`.

Trying every subset of services is `2^n`.

## Examples

### Example 1 — a small tree

```
parent   = [-1, 0, 0]
evidence = [10, 6, 6]
```

The tree is a root with two children:

```
    0 (10)
   /  \
 1(6) 2(6)
```

**Answer:** `12`

Auditing the root alone gives 10. Auditing both children gives 6 + 6 = 12, and
they are siblings, not a parent-child pair, so that is allowed. Auditing all
three is forbidden — the root is the direct parent of both.

Note the greedy instinct ("audit the biggest one") gives 10 and is wrong.

### Example 2 — the root wins

```
parent   = [-1, 0, 0]
evidence = [10, 3, 3]
```

Same shape, smaller children.

**Answer:** `10`

Now the two children total only 6, so auditing the root alone is better.
Whether to take a service depends on what its children are worth, which is why
you cannot decide in isolation.

### Example 3 — a chain

```
parent   = [-1, 0, 1, 2]
evidence = [4, 1, 1, 4]
```

A chain `0 → 1 → 2 → 3`.

**Answer:** `8`

Audit services 0 and 3. They are two apart, so the constraint is satisfied.
Total 4 + 4 = 8. Taking service 0 forbids service 1 but says nothing about
services 2 and 3.

### Example 4 — grandchildren are allowed

```
parent   = [-1, 0, 1]
evidence = [5, 100, 5]
```

A chain `0 → 1 → 2`.

**Answer:** `100`

Service 1 alone is worth more than services 0 and 2 combined (10). Even though
0 and 2 are both allowed together — they are grandparent and grandchild, not
parent and child — their total is smaller.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/dp-on-trees/01-audit-sampling/EDITORIAL.md`.
