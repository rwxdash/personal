# Editorial — Audit Sampling

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

Start by killing the simple rules, because both are tempting and both are
wrong.

**Greedy by value.** Example 1 has values `10, 6, 6` with the 10 at the root.
Taking the biggest first gives 10 and locks out both children; taking the two
children gives 12.

**Take every node at even depth.** On Example 3 that gives 8, which happens to
be right. On Example 4 (`5, 100, 5` in a chain) it gives 10, while the single
middle node is worth 100. A rule that is right sometimes and wrong sometimes is
worse than no rule at all — it will pass your hand-checked example and fail the
tests.

So decisions genuinely interact. But look at *how far* they interact: auditing a
service constrains only its direct parent and direct children. Nothing further
away cares.

That locality is the whole problem. Consider a node `v` and everything below
it. The outside world needs to know only two things about that subtree:

- the best total achievable inside it, and
- whether that total involved auditing `v` itself.

The second bit matters because it is the only thing that constrains `v`'s
parent. Nothing deeper can affect anything above `v`. So an entire subtree —
possibly 100 000 nodes — collapses into **two numbers**.

## The approach

```
skip[v] = best total in v's subtree when v is NOT audited
take[v] = best total in v's subtree when v IS audited
```

with

```
skip[v] = sum over children c of max(skip[c], take[c])
take[v] = evidence[v] + sum over children c of skip[c]
```

and the answer is `max(skip[0], take[0])`.

Leaves need no special case: their sums are empty, giving `skip = 0` and
`take = evidence[v]`.

### The `max` that everyone gets wrong

In the `skip[v]` line it must be `max(skip[c], take[c])`, not `take[c]`.

When `v` is skipped, each child is **free to choose** — it is not obliged to be
audited. Writing `take[c]` quietly asserts that every child of an unaudited node
must itself be audited, which is not what the constraint says. Mutating the
reference this way fails 4 of the 12 tests, including all four examples.

### Return the max at the root too

The answer is `max(skip[0], take[0])`, not `take[0]`. The root is frequently
better left unaudited — Example 4 and the star tests are exactly that shape.
Returning `take[0]` fails 9 of the 12 tests.

### Getting the order right without recursion

The recurrence needs every node's children finished before the node itself, and
the tree can be a 200 000-deep chain. Recursion will not survive that: verified,
a recursive version raises `RecursionError` in CPython even with the limit
raised to 30 000, and would overflow the thread stack in Rust.

The clean way out is **two flat passes**:

1. Walk from the root with an explicit stack, recording the order nodes come
   off it. Every node lands in that order *after* its parent.
2. Process that order **in reverse**, which puts every node after all of its
   children.

```
order = []
stack = [0]
while stack:
    v = stack.pop()
    order.append(v)
    for c in children[v]:
        stack.append(c)

for v in reversed(order):
    ... compute skip[v] and take[v] from the children ...
```

This sidesteps the "return to a node after its children" bookkeeping that an
iterative post-order otherwise needs — you visit twice and let the second pass
inherit the ordering from the first.

Note also that a parent's index is **not** guaranteed to be smaller than its
children's, so simply looping over indices in reverse does not work.

## Complexity

- **Time:** `O(n)`. Each node is pushed and popped once; each child is examined
  once by its parent.
- **Space:** `O(n)` for the children lists, the order, and the two arrays.

Why brute force fails: trying every subset is `2^n`. Measured 3.03 s at just
**22 services**, quadrupling with every two added. At n = 40 that is centuries;
n goes to 200 000.

## Common wrong turns

- **Greedy by value.** Fails Example 1.
- **Alternating by depth.** Fails Example 4.
- **`take[c]` instead of `max(skip[c], take[c])`.** The dominant bug.
- **Returning `take[0]`** instead of the max at the root.
- **Recursion.** Crashes on the 200 000-deep chain.
- **Looping over indices in reverse** instead of a real traversal order.
  Works only when parents happen to be numbered before children, which the
  problem explicitly does not guarantee.
- **32-bit arithmetic.** 200 000 services at 10^9 reaches 2 × 10^14.
- **Special-casing leaves.** Unnecessary, and usually wrong when done.

## Interview follow-ups

1. **Return which services to audit**, not just the total. Walk back down from
   the root carrying "was my parent taken?" as you go.
2. **A cap on the number of audits.** At most `k` services. Adds a dimension:
   `skip[v][j]` and `take[v][j]` for `j` audits used in the subtree, and merging
   children becomes a small knapsack. Complexity subtleties here are a good
   Staff-level discussion — the naive bound looks like `O(nk²)` but is
   `O(nk)` amortised with careful merging.
3. **Forbid grandparent-grandchild too**, not just parent-child. The state has
   to grow to remember two levels; a good test of whether they can extend the
   idea rather than recall it.
4. **Minimum audits to cover every service** (each service must be audited or
   adjacent to one). Different problem, same machinery — this is domination
   rather than independence, and the state becomes three-valued.
5. **The tree changes.** A service is re-parented; recompute without redoing
   everything. Only the two paths to the root are invalidated — leads to a
   discussion of path updates and heavy-light decomposition.
6. **What if it were a general graph, not a tree?** Then this is maximum weight
   independent set, which is NP-hard. The tree structure is exactly what makes
   the linear-time solution possible, and knowing that boundary is the point.
