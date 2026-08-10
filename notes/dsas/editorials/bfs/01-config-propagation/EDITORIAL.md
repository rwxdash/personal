# Editorial — Config Propagation

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

Two recognitions are needed here, and they are independent. Getting only the
first gives a correct-but-far-too-slow solution.

**Recognition 1: "round received" is a shortest distance.** The config spreads
one hop per round from everything that already holds it, so the round a node
receives it is exactly its hop-distance to the nearest seed. Every edge costs
the same, so this is *unweighted* shortest path — no priority queue needed, and
a weighted-graph algorithm would be more machinery than the problem warrants.

This also rules out depth-first exploration. A depth-first walk can reach a node
by a long path before a short one exists, recording the wrong round unless you
revisit nodes repeatedly — which destroys the linear bound. You need a traversal
that visits in **non-decreasing distance order**, so the first sighting of a node
is already its shortest distance and can be committed to permanently.

**Recognition 2: there are up to 100 000 start points.** Running one search per
seed and taking the per-node minimum is `O(S · (n + m))` — measured at 8.3 s for
n = 8 000 with 4 000 seeds, scaling quadratically, which extrapolates to roughly
**86 minutes** at the stated bounds.

The fix is the pattern's actual content:

> **Seed the queue with every start node at once, all at distance 0.**

This is equivalent to adding a virtual node joined to every seed and running one
ordinary BFS from it, which yields `1 + min-distance-to-a-seed` everywhere;
initialising with all seeds at distance 0 is that same computation with the
virtual hop subtracted. Because BFS dequeues in non-decreasing distance order,
the first time a node is reached its round is *already* the minimum over all
seeds. No per-seed passes, no post-hoc minimum, one `O(n + m)` sweep regardless
of seed count.

Mutating the reference to use only the first seed fails 7 of the 12 tests.

## The approach

```
adj = undirected adjacency list from links

round_of = [-1] * n
queue = deque()

for s in seeds:
    if round_of[s] == -1:          # duplicate-seed guard
        round_of[s] = 0
        queue.append(s)

while queue:
    u = queue.popleft()
    if quarantined[u]:
        continue                   # recorded, but expands nothing
    for v in adj[u]:
        if round_of[v] == -1:
            round_of[v] = round_of[u] + 1
            queue.append(v)

return round_of
```

### Where the quarantine check goes

This is the part the problem is really testing, and it is a one-line decision
with three consequences.

The check belongs at the **expansion** step — *after* dequeuing — not at enqueue
time and not in the visited test. The reason: a node's round is written when it
is **enqueued**. By the time the `continue` fires, its round is already
recorded, so suppressing the expansion suppresses exactly the propagation and
nothing else.

Put the check on the enqueue side instead (`if round_of[v] == -1 and not
quarantined[v]`) and the quarantined node never gets a round at all — it becomes
`-1` when the answer should be a real number. That mutation fails 4 of the 12
tests, including all four worked examples.

**A quarantined seed needs no special case.** It enters the queue at distance 0,
is dequeued, has its 0 already recorded, and expands nothing. Example 4 exists
to make that behaviour explicit before you code, because it is the one place
where "accepts but never forwards" and "is a seed" interact.

### `round_of` as the visited marker

`-1` means "not yet reached", which is also the required output for unreachable
nodes. Using one array for both means they can never disagree, and it removes
the separate `visited` array that is usually where the off-by-one lives.

**Mark on enqueue, not on dequeue.** Marking on dequeue lets a node enter the
queue several times via different neighbours — still correct, but the queue can
swell to `O(m)`.

### Free handling of messy input

- **Self-loops** add `u` to its own adjacency list; the `round_of[v] == -1`
  check rejects it immediately.
- **Duplicate edges** are examined twice and rejected the second time.
- **Duplicate seeds** are caught by the enqueue guard.
- **Empty seeds** means the queue never starts and everything stays `-1`.

None of these need special cases, which is the sign the structure is right.

## Complexity

- **Time:** `O(n + m)`. Each node is enqueued at most once; each edge is
  examined at most twice, once from each endpoint.
- **Space:** `O(n + m)` for the adjacency list.

## Common wrong turns

- **One search per seed.** Correct, `O(S · (n + m))`, ~86 minutes at the bounds.
- **Checking quarantine at enqueue time.** Loses the node's own round.
- **Skipping quarantined nodes entirely** (never enqueuing them). Same failure,
  more directly.
- **Special-casing quarantined seeds.** Unnecessary, and the special case is
  usually wrong.
- **Using DFS.** Wrong visit order; records non-minimal rounds.
- **Recursion.** A 200 000-node chain exhausts the stack — and depth-first order
  would be wrong anyway, so this fails twice over.
- **Reaching for Dijkstra.** Correct but unnecessary: all edges cost 1, so the
  `log` factor buys nothing.
- **A separate `visited` array that drifts from the distance array.**
- **Marking visited on dequeue**, letting the queue grow to `O(m)`.
- **Returning `0` instead of `-1`** for unreachable nodes, or conflating "seed"
  with "distance 0 from something".

## Interview follow-ups

1. **Which seed reached each node?** Carry the source along in the queue. Then
   ask what to do about ties — two seeds equidistant — and watch whether they
   notice the tie rule must be decided, not discovered.
2. **Quarantined nodes forward with a one-round delay** instead of never. Now
   edges have weights 1 and 2, plain BFS breaks, and the right answer is 0-1
   BFS with a deque (or Dijkstra). A good escalation because it shows exactly
   which property of BFS was being relied on.
3. **How many rounds until everything is covered?** `max(round_of)`, but only
   if nothing is `-1` — worth asking so they handle the unreachable case.
4. **Minimum seeds to cover the whole graph within `r` rounds.** NP-hard
   (dominating set); recognising the complexity jump is the point.
5. **Dynamic quarantine.** Nodes enter and leave maintenance between rollouts;
   answer many queries efficiently. Opens up preprocessing-versus-query
   trade-offs.
6. **Why not bidirectional search?** It helps for a single source–target pair
   and does nothing here, since you need distances for *all* nodes. A good test
   of whether they apply optimisations by reflex or by analysis.
