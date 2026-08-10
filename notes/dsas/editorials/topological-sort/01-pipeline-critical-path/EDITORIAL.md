# Editorial — Pipeline Critical Path

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

This problem is unusual in that the statement hands you the recurrence
outright:

```
finish[i] = max(ready[i], max over predecessors a of finish[a]) + durations[i]
```

That is deliberate. The mathematics is not the test — the *evaluation strategy*
is. `finish[i]` depends on `finish[a]` for every predecessor, so you cannot
evaluate in index order, and there is no formula that lets you skip ahead. The
data itself dictates a valid order, and the question is how to find one.

The three signals that fix the approach:

1. **The input is an edge list wearing a costume.** `deps` is a list of pairs
   over `0..n-1`. That is a directed graph, and Example 3's "diamond" is
   literally the shape of the graph.
2. **The `None` case is a structural guarantee, not an afterthought.** Longest
   path in a general directed graph is NP-hard. The only reason this problem is
   solvable in linear time is that you are permitted — required — to reject
   cyclic inputs. The moment you reject cycles you are on a DAG, and on a DAG
   there is a linear order in which every edge points forward.
3. **`n = 200_000`, possibly a single chain that deep.** This kills recursive
   DFS: CPython's default limit is ~1000, and raising it to 200 000 segfaults
   rather than working. Whatever you write must be iterative.

Signals 1 + 2 + 3 point at exactly one tool: an **iterative topological sort
with the DP fused into the same pass**.

## The approach

Kahn's algorithm, with the relaxation done at pop time.

**Build.** `adj[a]` = successor list, `indeg[b]` = number of incoming edges.
Process every entry of `deps` exactly as given. Do **not** deduplicate: the
correctness argument depends only on `indeg[v]` equalling the number of times
`v` will be relaxed, so duplicates are harmless *as long as they are counted
consistently in both structures*. Deduplicating the adjacency list but not the
in-degree (or vice versa) strands nodes at a permanently positive in-degree,
and you will report a phantom cycle.

**Initialise.** `start[i] = ready[i]`. Read this as "the earliest instant job
`i` is currently known to be able to begin"; it monotonically increases as
predecessors report in. Seed the queue with every zero-in-degree node.

**Sweep.**

```
while queue:
    u = queue.pop()
    processed += 1
    finish_u = start[u] + durations[u]
    answer = max(answer, finish_u)
    for v in adj[u]:
        start[v] = max(start[v], finish_u)
        indeg[v] -= 1
        if indeg[v] == 0: queue.push(v)
```

**The invariant that makes this correct:** *when a node is popped, its `start`
is final.* A node is enqueued only when its in-degree reaches zero; the
in-degree reaches zero only after every incoming edge has been relaxed; each
edge is relaxed only when its source is popped. So by the time `u` is popped,
every predecessor has already contributed its finish time to `start[u]`. This
is why the DP can live inside the traversal rather than needing a completed
topological order first — and it is the sentence to say out loud in an
interview.

**Cycle detection is free.** If `processed < n`, the unreached nodes are
exactly those in or downstream of a cycle: every node in a cycle has an
in-degree contribution from inside the cycle that nothing outside can ever
retire. A self-loop `(a, a)` gives `a` an in-degree it can never shed and falls
out of the same check — unless you "helpfully" filtered self-loops during the
build, which converts an unschedulable pipeline into a schedulable one.

Note the check is `processed < n`, **not** "the queue emptied early". The queue
always empties; that tells you nothing.

## Complexity

- **Time:** O(n + m). Each node is enqueued and popped exactly once; each edge
  is relaxed exactly once.
- **Space:** O(n + m) for the adjacency lists, in-degrees, and queue.

Why the naive approaches fail, measured:

- **Enumerate all paths and take the heaviest.** The layered test graph has 400
  nodes per layer across 500 layers with 2 forward edges each — on the order of
  2^499 routes. Not slow; impossible.
- **Recursive memoised DFS.** Correct in principle, O(n + m), and it dies on
  the 200 000-deep chain. Verified: it raises `RecursionError` even with the
  limit lifted to 10 000, and lifting it to 200 000 crashes the interpreter
  instead of working. This is the single most common way a correct algorithm
  fails this problem.
- **Repeated relaxation until fixpoint** (Bellman-Ford style, what the test
  file's oracle does). Its round count depends on the *order edges happen to be
  listed in*. With the chain listed front to back it converges in two passes
  and looks fine; with it listed back to front it needs one pass per edge.
  Measured at 4.3 s for n = 8 000 with clean 4×-per-doubling scaling —
  roughly **45 minutes** at n = 200 000. The `_test_large_reversed_chain` case
  exists precisely to remove that illusion.

## Common wrong turns

- **Summing durations instead of maximising.** Example 2 kills this
  immediately: unlimited workers means the makespan is a max over routes, not a
  total.
- **Taking the answer from sink nodes only.** A disconnected job with a late
  `ready` and no edges can be the last thing to finish.
  `_test_large_disconnected_late_job` targets this.
- **Ignoring `ready` on internal nodes.** Applying availability only to sources
  is a common shortcut; a late artifact halfway down a chain shifts everything
  after it (`_test_large_chain_with_late_artifact`).
- **Applying `ready[i]` after adding the duration** — i.e.
  `max(ready[i], preds) + dur` vs `max(ready[i], preds + dur)`. The second is
  wrong and only shows up when `ready` dominates.
- **Deduplicating edges in one structure but not the other.** Reports a
  phantom cycle. `[(0,1), (0,1), (0,1)]` targets it.
- **Filtering out self-loops during the build.** Turns `None` into a number.
- **Checking for cycles only among reachable nodes.** A cycle in a component
  with no zero-in-degree entry point (`[(0,1),(1,2),(2,1)]`) is never visited
  by a traversal seeded from sources; only the `processed < n` count catches it.
- **32-bit arithmetic.** `ready` up to 10^9 plus a 200 000-long chain of 10^9
  durations reaches ~2 × 10^14.
- **Treating zero-duration jobs as skippable.** They are ordinary nodes; their
  edges still order the graph.

## Interview follow-ups

1. **Report the critical path itself, not just its length.** Store a parent
   pointer at each relaxation that actually raised `start[v]`, then walk back
   from the argmax. Ask what happens with ties and whether the path is unique.
2. **Bounded workers.** Now only `k` jobs may run at once. This is the real
   jump: the problem becomes NP-hard for `k >= 2` (it contains multiprocessor
   scheduling), so the conversation turns to list scheduling with a priority
   heap and the classic 2 − 1/k approximation bound. Recognising that the
   complexity class changes is the point.
3. **Which job should we optimise?** Compute each job's *slack* — how much it
   could be delayed without moving the makespan — via a second, reverse pass
   computing latest permissible start times. Zero-slack jobs are the critical
   path. This is the standard forward/backward CPM pass and is the most likely
   immediate follow-up.
4. **Incremental updates.** A job's duration changes; recompute the makespan
   without redoing the whole graph. Discuss what must be invalidated
   (everything reachable downstream) and when a full recompute is cheaper.
5. **Explain the cycle.** Returning `None` is unhelpful to a user — return the
   actual cycle. Kahn's leftovers give you the set of implicated nodes; one DFS
   restricted to that set extracts a concrete cycle.
6. **Why not DFS-based topological sort?** It is equally linear and perfectly
   valid — as long as it is written with an explicit stack. Worth having the
   iterative-DFS version in your pocket, plus the reason you did not reach for
   it here.
