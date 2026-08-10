# Hints — Config Propagation

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> In an undirected graph, compute each node's distance to the **nearest** node
> in a given start set, where certain nodes may be entered but not exited.
> Report `-1` for unreachable nodes.

Three things to pin down before writing anything, because each is a place the
obvious implementation goes wrong:

1. **"Round received" is a shortest distance.** The config spreads one hop per
   round from everything that already has it, so the round a node receives it
   is exactly its hop-distance to the nearest seed. Not *a* distance — the
   *minimum* one.
2. **Quarantined nodes are entered but not exited.** They get a real round
   number recorded; only their outgoing propagation is suppressed. The single
   most common error here is skipping them entirely, which loses their round
   number as well as their (correctly suppressed) neighbours.
3. **A quarantined seed is still a seed.** It holds the config at round 0 and
   forwards nothing. Example 4 exists solely to make this concrete.

Also note `seeds` may contain duplicates, and `links` may contain duplicate
pairs and self-loops. None of those should change any answer — but they will
break a fragile implementation.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

This is a **shortest-path problem on an unweighted graph**, and that phrase
should immediately narrow your options:

- Every edge costs exactly one round, so you do not need a priority queue. A
  weighted-graph algorithm would be correct but is more machinery than the
  problem requires.
- Exploring depth-first would visit nodes in the wrong order entirely. A
  depth-first walk can reach a node by a long path before a short one exists,
  and would record the wrong round unless you revisit nodes repeatedly — which
  destroys the linear bound.

What you want is a traversal that visits nodes in **non-decreasing distance
order**, so that the first time you see a node, you have already found its
shortest distance and can commit to it and never look again.

Now the second question, and the one that actually determines the complexity.
There is not one start node — there may be 100 000 of them. Running the
traversal once per seed and taking the per-node minimum is `O(S · (n + m))`,
roughly 6 × 10^10 at the bounds. Far too slow.

But think about what "distance to the nearest seed" means. Is there a way to
make *all* the seeds behave as if they were a single starting point?

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Multi-source BFS.**

The trick is to seed the queue with **every** seed node at once, all at
distance 0, before the traversal begins. Then run one ordinary BFS.

Why that computes the minimum over all seeds: it is equivalent to adding a
virtual node connected to every seed and running a single BFS from it, which
would give every node `1 + min-distance-to-a-seed`. Initialising the queue with
all seeds at distance 0 is that same computation with the virtual node's hop
subtracted. Because BFS pops in non-decreasing distance order, the first time
any node is dequeued its recorded distance is already the minimum across all
seeds — no per-seed passes, no post-hoc minimum.

One pass, `O(n + m)`, regardless of how many seeds there are.

**Where the quarantine goes.** Not on the enqueue side and not on the visited
check — those would suppress the node's own round number. It belongs at the
*expansion* step: when you dequeue a node, record its distance as normal, then
**only iterate its neighbours if it is not quarantined**. A quarantined node is
dequeued, counted, and then contributes nothing further.

Seeding with a quarantined seed follows automatically: it enters the queue at
distance 0, gets dequeued, records 0, and expands nothing. No special case
needed — provided you put the check in the right place.

Mark nodes as visited when you **enqueue** them, not when you dequeue them.
Marking on dequeue lets a node enter the queue several times via different
neighbours, which is still correct but can blow the queue up to `O(m)`.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

Build an undirected adjacency list, then run one BFS seeded with everything.

```
adj = [[] for _ in range(n)]
for u, v in links:
    adj[u].append(v)
    adj[v].append(u)          # self-loops add u twice; harmless

round_of = [-1] * n
queue = deque()

for s in seeds:
    if round_of[s] == -1:     # guards against duplicate seeds
        round_of[s] = 0
        queue.append(s)

while queue:
    u = queue.popleft()
    if quarantined[u]:
        continue              # dequeued and recorded, but never expands
    for v in adj[u]:
        if round_of[v] == -1:
            round_of[v] = round_of[u] + 1
            queue.append(v)

return round_of
```

Note how compact the quarantine rule is: a single `continue` **after** the node
has been dequeued. Its round was written when it was *enqueued*, so it is
already recorded by the time the check fires.

Details worth being deliberate about:

- **`round_of` doubles as the visited marker.** `-1` means "not yet reached",
  which is also the required output for unreachable nodes — so no separate
  `visited` array is needed and the two can never disagree.
- **The duplicate-seed guard.** Without the `if round_of[s] == -1`, a repeated
  seed is enqueued twice. Harmless for correctness here, but it is free to
  handle properly.
- **Self-loops and duplicate edges** need no special handling at all: the
  `round_of[v] == -1` check rejects any node already reached, and a self-loop
  just tests the node against itself.
- **Write the BFS iteratively** with a queue. This is naturally iterative, but
  do not be tempted to swap in recursion — a 200 000-node chain would exhaust
  the stack, and depth-first order would give wrong answers anyway.
- **`seeds` may be empty**, in which case the queue never starts and every
  entry stays `-1`. The general code handles it.

Complexity: each node is enqueued at most once and each edge is examined at
most twice (once from each endpoint), giving `O(n + m)` time and `O(n + m)`
space for the adjacency list.

</details>
