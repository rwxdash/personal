# Config Propagation

**Difficulty:** Medium
**ID:** `bfs-01-config-propagation`

## Scenario

A new configuration is being rolled out across a datacentre using a gossip
protocol. Nodes are numbered `0 .. n-1` and connected by bidirectional peering
links.

The rollout proceeds in discrete **rounds**:

- At round 0, a set of **seed** nodes already hold the new config (an operator
  pushed it to them directly).
- In each subsequent round, every node that *already* holds the config forwards
  it to all of its direct peers simultaneously. A peer that does not yet hold
  the config receives it in that round.

One complication. Some nodes are **quarantined** — they are mid-maintenance, so
they will *accept* a config pushed to them but will **never forward it onward**
to their peers. A quarantined node still records the round in which it received
the config; it simply acts as a dead end for propagation.

Quarantine applies unconditionally. **A quarantined node that is also a seed
still never forwards** — it starts with the config at round 0 and propagates
nothing.

## Task

Implement:

```python
propagation_rounds(n: int, links: List[Tuple[int, int]], seeds: List[int], quarantined: List[bool]) -> List[int]
```

Return a list of length `n` where element `i` is the round at which node `i`
receives the config: `0` for a seed, and `-1` if it never receives it at all.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `n` | `int` | `1 <= n <= 200_000` |
| `links` | `List[Tuple[int, int]]` | `0 <= m <= 400_000`; endpoints in `0 .. n-1` |
| `seeds` | `List[int]` | `0 <= len(seeds) <= n`; valid node indices, may repeat |
| `quarantined` | `List[bool]` | length `n` |

`links` may contain **duplicate pairs** and **self-loops** `(u, u)`. The graph
may be disconnected. `seeds` may be empty, in which case nothing ever
propagates.

## Output

`List[int]` of length `n`. Seeds are `0`, unreachable nodes are `-1`.

## Required complexity

- **Time:** `O(n + m)`.
- **Space:** `O(n + m)`.

At `n = 200_000` with 100 000 seeds, running a separate search from each seed
and taking the minimum is on the order of 6 × 10^10 operations.

## Examples

### Example 1 — a simple chain

```
n = 4
links = [(0,1), (1,2), (2,3)]
seeds = [0]
quarantined = [False, False, False, False]
```

**Answer:** `[0, 1, 2, 3]`

Node 0 starts with the config. Round 1 reaches node 1, round 2 reaches node 2,
round 3 reaches node 3.

### Example 2 — two seeds meeting in the middle

```
n = 5
links = [(0,1), (1,2), (2,3), (3,4)]
seeds = [0, 4]
quarantined = [False] * 5
```

**Answer:** `[0, 1, 2, 1, 0]`

Both ends start at round 0 and the config spreads inward from both
simultaneously. Node 1 is reached from node 0 in round 1, node 3 from node 4 in
round 1, and node 2 is reached in round 2 — from *both* sides at once, but the
round it is first reached is what counts.

Note this is **not** the same as propagating from seed 0 and then separately
from seed 4: the answer for each node is the earliest round across all seeds.

### Example 3 — a quarantined node blocks the chain

```
n = 4
links = [(0,1), (1,2), (2,3)]
seeds = [0]
quarantined = [False, False, True, False]
```

**Answer:** `[0, 1, 2, -1]`

Rounds 1 and 2 proceed normally, so node 2 does receive the config at round 2
and that is recorded. But node 2 is quarantined, so it never forwards, and node
3 — reachable only through node 2 — never receives the config at all.

The key detail: node 2 is **not** skipped. It gets a real round number. It is
only its *outgoing* propagation that is suppressed.

### Example 4 — a quarantined seed

```
n = 3
links = [(0,1), (1,2)]
seeds = [0]
quarantined = [True, False, False]
```

**Answer:** `[0, -1, -1]`

Node 0 is a seed, so it holds the config from round 0. But it is quarantined,
so it forwards nothing, and the rollout never leaves it. Nodes 1 and 2 are
never reached even though the graph is fully connected.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/bfs/01-config-propagation/EDITORIAL.md`.
