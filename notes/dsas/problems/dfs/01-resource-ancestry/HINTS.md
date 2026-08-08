# Hints — Resource Ancestry

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Given a forest by parent pointers, answer many `is u an ancestor of v?`
> queries. Non-strict: `u == v` counts.

The shape of the constraints is the whole story:

- `n` up to 200 000, `q` up to 200 000, and the required bound is `O(n + q)`.
- That is **`O(1)` amortised per query after linear preprocessing** — there is
  no `log` factor in the budget at all.

So the obvious approach — walk from `v` up the parent chain looking for `u` — is
out. It is `O(depth)` per query, and the statement explicitly permits a chain
200 000 deep, giving 4 × 10^10 steps.

The interesting consequence: **you cannot afford to answer a query by
traversing anything.** Whatever a query does must be a fixed amount of work on
precomputed data. So the real question is not "how do I answer a query?" but
"what can I compute about each node, once, so that ancestry becomes a trivial
test between two nodes' values?"

Think about what a query really compares. Ancestry is about *containment* — the
set of descendants of `u` either contains `v` or does not.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

Containment is the word to hold onto. `u` is an ancestor of `v` exactly when
`v` belongs to `u`'s subtree. And subtrees have a very convenient property:

> **A node's subtree is contiguous in the order a depth-first traversal visits
> nodes.**

Walk the forest depth-first. Once you step into `u`, you visit every node of
`u`'s subtree — all of them, with nothing else mixed in — before you step back
out of `u`. Nothing outside the subtree can be visited in the middle, because
leaving the subtree requires returning through `u`.

That means if you stamp each node with a counter when you **enter** it and
again when you **leave** it, every subtree becomes an **interval**, and
ancestry becomes interval containment — two comparisons, no traversal.

This is why the traversal has to be depth-first specifically. A breadth-first
walk visits nodes in distance order, which scatters each subtree across many
non-contiguous positions; there is no interval to compare. Entry/exit
timestamps are an artefact of depth-first order and nothing else produces them.

Two implementation realities to plan for before you write it:

- The hierarchy is a **forest**, so the traversal has to start from every root.
- A chain can be 200 000 deep, which has consequences for *how* you write the
  traversal.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**DFS entry/exit times (an Euler tour), turning ancestry into interval
containment.**

Run one depth-first traversal over the whole forest with a single global
counter. On entering node `x` record `tin[x]`; after finishing all of its
children, record `tout[x]`. Then:

> `u` is an ancestor of `v` **iff** `tin[u] <= tin[v]` **and**
> `tout[v] <= tout[u]`.

Read that as: `v`'s interval is nested inside `u`'s.

Three things fall out for free, which is a good sign the encoding is right:

- **`u == v` returns True**, since a node's interval trivially contains itself.
  The non-strict definition needs no special case.
- **Cross-tree queries return False**, provided the counter is *global* across
  all roots rather than reset per tree. Different trees get disjoint intervals,
  so neither can contain the other. Resetting the counter per root breaks this
  silently and is a genuinely nasty bug — it only shows up on multi-root inputs.
- **Direction is handled**, since containment is asymmetric.

**Write the traversal iteratively with an explicit stack.** A recursive DFS on a
200 000-deep chain blows the interpreter's stack in Python and overflows the
thread stack in Rust. This is not a theoretical concern — it is a guaranteed
crash on the stated bounds, and it is the main reason this problem is harder to
implement than to design.

You will need children lists, which you can build in `O(n)` from the parent
array in a single pass.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

**Step 1 — build children lists and collect roots.**

```
children = [[] for _ in range(n)]
roots = []
for i, p in enumerate(parent):
    if p == -1: roots.append(i)
    else:       children[p].append(i)
```

**Step 2 — iterative DFS stamping in/out times**, one global counter across
every root:

```
tin  = [0] * n
tout = [0] * n
timer = 0

for r in roots:
    stack = [(r, False)]          # (node, are we returning to it?)
    while stack:
        node, returning = stack.pop()
        if returning:
            tout[node] = timer; timer += 1
            continue
        tin[node] = timer; timer += 1
        stack.append((node, True))          # schedule the exit stamp
        for c in children[node]:
            stack.append((c, False))
```

The `(node, returning)` flag is the standard way to give an iterative DFS the
"after all children are done" moment that recursion gets for free. Push the
exit marker **before** the children so it is popped **after** them.

**Step 3 — answer each query in O(1):**

```
return [tin[u] <= tin[v] and tout[v] <= tout[u] for u, v in queries]
```

Things to get right:

- **One global `timer`**, never reset between roots. This is what makes
  cross-tree queries False.
- **Iterative, not recursive.** A 200 000-deep chain is an explicit test case.
- **Both comparisons are non-strict** (`<=`), which is what makes `u == v`
  return True.
- Child visit order does not matter — any DFS order produces a valid nesting.
- `q` may be 0, and every node may be a root (a forest of singletons).

Complexity: building children is `O(n)`, the traversal pushes and pops each node
a constant number of times for `O(n)`, and each query is `O(1)` — giving
`O(n + q)` time and `O(n)` space.

</details>
