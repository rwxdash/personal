# Mesh Link Failures

**Difficulty:** Medium–Hard
**ID:** `union-find-01-mesh-link-failures`

## Scenario

Your service mesh has `n` nodes, numbered `0 .. n-1`, wired together by a fixed
set of bidirectional links. Two nodes can talk to each other if there is any
chain of intact links between them; a maximal set of mutually reachable nodes
is a **partition**. A node with no intact links to anywhere is a partition of
one.

You are running a chaos-engineering experiment. You have a script that will
disable links one at a time, in a fixed order decided up front, and links are
never restored during the run. Before executing it for real, you want to know
how badly the mesh fragments at each step: **after each individual link is
disabled, how many partitions does the mesh have?**

Because the whole failure sequence is known before the experiment starts, you
may analyse it however you like — you are not required to answer step `k`
before you have looked at step `k+1`.

## Task

Implement:

```python
partitions_after_failures(n: int, links: List[Tuple[int, int]], failures: List[int]) -> List[int]
```

- `links[i] = (u, v)` — a bidirectional link between nodes `u` and `v`.
- `failures` — **indices into `links`**, in the order those links go down.

Return a list of the same length as `failures`, where element `k` is the number
of partitions in the mesh after the first `k + 1` listed failures have been
applied.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `n` | `int` | `1 <= n <= 200_000` |
| `links` | `List[Tuple[int, int]]` | `0 <= len(links) <= 400_000`; endpoints in `0 .. n-1` |
| `failures` | `List[int]` | distinct valid indices into `links`; `0 <= len(failures) <= len(links)` |

Two important input details:

- **`links` may list the same pair more than once** — a redundant second cable
  between the same two nodes. These are separate links with separate indices.
  Disabling one of them does not disable the other.
- **`links` may contain self-loops** `(u, u)` — a misconfigured entry pointing a
  node at itself. It connects nothing.

Failures are identified by **link index**, never by endpoint pair, so
duplicates are never ambiguous.

## Output

`List[int]` of length `len(failures)`. An empty `failures` list yields an empty
result.

## Required complexity

- **Time:** near-linear in `n + m`, where `m = len(links)`. Treat
  `O((n + m) log n)` as the ceiling; a solution that touches each link a
  constant number of times overall is what the bounds are set for.
- **Space:** `O(n + m)`.

At `n = 200_000` with 100 000 failures, recomputing connectivity from scratch
after each failure performs on the order of 2 × 10^10 operations.

## Examples

### Example 1

```
n = 4
links = [(0,1), (1,2), (2,3)]
failures = [1]
```

**Answer:** `[2]`

The mesh starts as one chain `0—1—2—3`: a single partition. Link index 1 is the
`1—2` cable; cutting it splits the chain into `{0,1}` and `{2,3}`, so 2
partitions.

### Example 2 — progressive fragmentation

```
n = 4
links = [(0,1), (1,2), (2,3)]
failures = [1, 0, 2]
```

**Answer:** `[2, 3, 4]`

- After failure 1 (cut `1—2`): `{0,1}`, `{2,3}` → 2
- After also cutting index 0 (`0—1`): `{0}`, `{1}`, `{2,3}` → 3
- After also cutting index 2 (`2—3`): every node alone → 4

### Example 3 — redundant cable

```
n = 2
links = [(0,1), (0,1)]
failures = [0, 1]
```

**Answer:** `[1, 2]`

Two separate cables run between nodes 0 and 1. Cutting the first one changes
nothing — the second still carries the traffic — so the mesh remains a single
partition. Cutting the second one too finally separates them, giving 2.
Answering by endpoint pair rather than by index would wrongly report `[2, 2]`.

### Example 4 — self-loop and an isolated node

```
n = 3
links = [(0,0), (1,2)]
failures = [0, 1]
```

**Answer:** `[2, 3]`

Node 0 is wired to itself, which connects nothing, so the mesh starts as
`{0}` and `{1,2}` — 2 partitions. Cutting the self-loop (index 0) changes
nothing, so the count stays at 2. Then cutting `1—2` gives `{0}`, `{1}`, `{2}`
— 3.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/union-find/01-mesh-link-failures/EDITORIAL.md`.
