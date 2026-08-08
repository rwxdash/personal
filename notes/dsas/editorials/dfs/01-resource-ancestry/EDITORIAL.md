# Editorial — Resource Ancestry

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

The budget is the tell. `n` up to 200 000, `q` up to 200 000, required
`O(n + q)` — **no log factor at all**. That is `O(1)` amortised per query after
linear preprocessing, which forbids a query from traversing anything.

So the obvious approach dies immediately: walking from `v` up the parent chain
looking for `u` is `O(depth)` per query, and the statement explicitly permits a
200 000-deep chain. Measured at 0.56 s for n = q = 4 000 with quadratic scaling
→ roughly **23 minutes** at the stated bounds.

The reframing that unlocks it: **ancestry is containment.** `u` is an ancestor
of `v` exactly when `v` lies in `u`'s subtree. So instead of asking "can I walk
from `v` to `u`?", ask "what can I precompute per node so that subtree
membership is a comparison?"

And subtrees have a property tailor-made for this:

> A node's subtree is **contiguous** in the order a depth-first traversal
> visits nodes.

Once you step into `u`, you visit every node of `u`'s subtree — all of them,
nothing else interleaved — before stepping back out, because leaving the
subtree requires returning through `u`. Stamp a counter on entry and on exit,
and every subtree becomes an **interval**.

**This is why the traversal must be depth-first specifically.** A breadth-first
walk visits in distance order, scattering each subtree across non-contiguous
positions; there is no interval to compare. Entry/exit timestamps are an
artefact of depth-first order and nothing else produces them. If you can only
name one thing DFS gives you that BFS cannot, this is it.

## The approach

**Step 1.** Build children lists and collect roots in one `O(n)` pass over
`parent`.

**Step 2.** Run one depth-first traversal over the whole forest with a single
global counter, recording `tin[x]` on entry and `tout[x]` after all children
finish.

**Step 3.** Answer each query in `O(1)`:

```
u is an ancestor of v  ⟺  tin[u] <= tin[v]  and  tout[v] <= tout[u]
```

That is: `v`'s interval is nested inside `u`'s.

### Three properties that fall out for free

Good encodings make special cases disappear. All three of these are handled by
the two comparisons above, with no extra code — and each has a mutation test
proving it:

- **`u == v` returns True.** A node's interval trivially contains itself, which
  is exactly the non-strict definition the problem asks for. Changing both
  comparisons to strict `<` fails **8 of the 11** tests.
- **Cross-tree queries return False** — *provided the counter is global across
  all roots rather than reset per tree*. Different trees then occupy disjoint
  intervals, so neither can contain the other. Resetting the timer at each root
  makes different trees' intervals overlap arbitrarily, producing confident
  wrong answers. That mutation fails **6 of the 11** tests, and it is the
  nastiest bug in this problem because single-tree inputs never expose it.
- **Direction is respected**, since containment is asymmetric.

### Write the traversal iteratively

A recursive DFS on a 200 000-deep chain is a guaranteed crash, not a risk.
Verified: it raises `RecursionError` in CPython even with the limit lifted to
20 000, and lifting it to 200 000 segfaults the interpreter rather than
working. In Rust it overflows the thread stack.

The standard iterative shape uses a `(node, returning)` flag to recover the
"after all children are done" moment that recursion gives you for free:

```
stack = [(r, False)]
while stack:
    node, returning = stack.pop()
    if returning:
        tout[node] = timer; timer += 1
        continue
    tin[node] = timer; timer += 1
    stack.append((node, True))       # push exit BEFORE children
    for c in children[node]:
        stack.append((c, False))     # ...so it pops AFTER them
```

Child visit order is irrelevant — any DFS order produces a valid nesting.

## Complexity

- **Time:** `O(n + q)`. Building children is `O(n)`; each node is pushed and
  popped a constant number of times; each query is two comparisons.
- **Space:** `O(n)` for children lists, both timestamp arrays, and the stack.

## Common wrong turns

- **Walking the parent chain per query.** Correct, `O(q · depth)`, ~23 minutes.
- **Recursive DFS.** Crashes on the chain test.
- **Resetting the timer per root.** Silently wrong on forests only.
- **Strict `<` comparisons.** Breaks `u == v`.
- **Comparing only `tin`** (`tin[u] <= tin[v]`) without the `tout` check.
  Passes on chains, fails the moment `u` has a sibling subtree to its right —
  a "cousin to the right" looks like a descendant.
- **Using depth instead of timestamps.** Depth tells you nothing about *which*
  branch a node is on.
- **Binary lifting / LCA.** Correct, and `O(n log n)` preprocessing with
  `O(log n)` queries — strictly more machinery for a strictly worse bound than
  the interval trick. Worth knowing you *could*, and knowing you shouldn't.
- **Assuming node 0 is a root**, or that parents always have smaller indices.
  The test file permutes labels specifically to break this.
- **Forgetting the forest can be all roots** (every `parent[i] == -1`).

## Interview follow-ups

1. **Lowest common ancestor.** The natural escalation. Now the interval trick
   is not enough on its own and you need binary lifting or Euler tour + sparse
   table RMQ — a good place to explain why you *didn't* need it here.
2. **Distance between two resources.** `depth[u] + depth[v] - 2 * depth[lca]`.
3. **Subtree aggregate queries** — "how many buckets under this folder?"
   The Euler tour turns each subtree into a contiguous range, so subtree sums
   become range sums over the tour array. Same preprocessing, more payoff, and
   it shows the timestamps were a *linearisation*, not a trick.
4. **The hierarchy changes.** Reparenting a resource invalidates the intervals.
   Discuss recompute-versus-incremental, and where an Euler tour tree or
   link-cut tree would come in.
5. **Streaming queries with a memory cap.** Can you answer without storing two
   arrays of size `n`? (Not really — and articulating *why* is the point.)
6. **Why not just store each node's full ancestor set?** `O(n · depth)` memory,
   which is 4 × 10^10 entries in the worst case. A good sanity check on whether
   they cost their ideas before proposing them.
