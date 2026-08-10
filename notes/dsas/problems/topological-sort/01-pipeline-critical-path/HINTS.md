# Hints — Pipeline Critical Path

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

The statement already hands you the exact recurrence:

```
finish[i] = max(ready[i], max over predecessors a of finish[a]) + durations[i]
answer    = max over all i of finish[i]
```

So the mathematical content is settled. What is *not* settled — and what the
whole problem is actually about — is this:

> In what order do you evaluate that recurrence, and what happens when no valid
> order exists?

`finish[i]` depends on `finish[a]` for every predecessor `a`. You cannot
compute a value before its inputs. If you try to evaluate in index order, job 0
might depend on job 199 999 and you will read a value you have not computed
yet. So the ordering is forced on you by the data, not by the indices.

Two constraints worth pinning to the wall before you write anything:

- `n = 200_000` with a possible **single chain** that deep. Whatever you do
  must not have call depth proportional to the chain.
- Duplicate edges and self-loops are explicitly legal inputs. Decide early what
  each means for your bookkeeping — one of them is a cycle, the other must not
  corrupt your counters.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

This is a **graph problem**, and it is worth saying that out loud because the
input does not look like one. `deps` is an edge list; jobs are nodes; "a must
finish before b" is a directed edge `a → b`. Once you draw it, Example 3's
"diamond" is literally a diamond-shaped graph and the answer is the weight of
its heaviest route.

Now notice what kind of graph problem it is *not*. It is not shortest path —
you want the *latest* constraint, not the earliest. It is not general
longest-path either, because longest path in an arbitrary directed graph with
cycles is NP-hard. The problem is only tractable because of the acyclicity
requirement, and the statement's `None` case is what guarantees it: you either
reject the input as cyclic, or you are working on a DAG.

That is the key structural realisation. On a DAG there exists a linear ordering
of the nodes in which every edge points forward. Evaluate the recurrence in
that order and every dependency is already computed when you need it — one
pass, no recursion, no memo table lookups that miss.

So you need two things: a way to produce such an ordering, and a way to detect
that no such ordering exists. Look for an algorithm that gives you *both* from
the same machinery, rather than bolting a separate cycle check on the front.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Topological sort (Kahn's algorithm, BFS-style with in-degree counting), with
a DP relaxation fused into the same pass.**

Kahn's algorithm:

- Compute `indeg[v]` = number of incoming edges, counting duplicates as
  separate edges.
- Seed a queue with every node whose `indeg` is 0.
- Repeatedly pop a node, and for each outgoing edge decrement the target's
  `indeg`, pushing the target when it hits 0.

Two properties make it the right choice here over a DFS-based topological sort:

1. **It is iterative**, so a 200 000-deep chain is a non-event. A recursive DFS
   blows Python's default recursion limit on exactly that input.
2. **Cycle detection is free.** If the algorithm pops fewer than `n` nodes, the
   nodes it never reached are precisely those trapped in or behind a cycle. No
   separate colouring pass, no second traversal. A self-loop `(a, a)` gives `a`
   an in-degree it can never shed, so it falls out of this check automatically —
   provided you did not special-case it away while building the graph.

The DP goes *inside* the loop, not after it. When you pop a node, its
`finish` value is final; use it to relax its successors before moving on.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

Build the graph and run one fused pass.

**Build.** Adjacency lists `adj[a]` of successors, and `indeg[b]` counts.
Process every entry of `deps` exactly as given — do **not** deduplicate. If you
count a duplicate edge once in `indeg` but traverse it twice (or the reverse),
the counters desynchronise and you will either drop a node or push it twice.
Counting each occurrence consistently in both structures is what keeps
duplicates harmless.

**Initialise.** `start[i] = ready[i]` for every `i`. Think of `start[i]` as "the
earliest instant job `i` is currently known to be able to begin"; it only ever
increases as predecessors report in. Seed a queue with every `i` where
`indeg[i] == 0`.

**Sweep.** While the queue is non-empty:

1. Pop `u`, mark it processed (count it).
2. `finish_u = start[u] + durations[u]`. This is final: every predecessor of
   `u` has already been popped, so `start[u]` has absorbed all of them.
3. Fold it into the answer: `answer = max(answer, finish_u)`.
4. For each successor `v` in `adj[u]`: `start[v] = max(start[v], finish_u)`,
   then `indeg[v] -= 1`, and push `v` if `indeg[v]` reached 0.

**Finish.** If the number of processed nodes is less than `n`, the graph has a
cycle → return `None`. Otherwise return `answer`.

Why step 2 is sound: a node is only enqueued once its in-degree hits zero,
which happens only after every incoming edge has been relaxed, which happens
only after every predecessor was popped. That is the invariant the whole
algorithm rests on — **when a node is popped, its `start` is final** — and it
is worth convincing yourself of it explicitly rather than trusting the shape of
the code.

Complexity: each node is enqueued and popped once, each edge is relaxed once,
so O(n + m) time and O(n + m) space.

Watch these:

- The answer is the max of `finish` over **all** nodes, not just sinks. A
  disconnected job with a huge `ready` and no edges still sets the makespan.
- Use 64-bit arithmetic. `ready` up to 10^9 plus a 200 000-long chain of 10^9
  durations reaches ~2 × 10^14.
- Nodes with duration 0 are ordinary nodes, not nodes to skip.
- The cycle check must be "processed count < n", not "the queue emptied" — the
  queue always empties.

</details>
