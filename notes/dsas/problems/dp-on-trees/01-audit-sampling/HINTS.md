# Hints — Audit Sampling

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Given a rooted tree with a value on each node, choose a set of nodes with no
> two chosen nodes in a direct parent-child relationship, maximising the total
> value.

Rule out the easy answers first.

**Greedy by value fails.** Example 1: values `10, 6, 6` with the 10 at the root.
Taking the biggest first gives 10 and locks out both children. Taking the two
children gives 12.

**Level-by-level fails too.** "Take every node at even depth" gives 4 + 4 = 8
on Example 3, which happens to be right — but on Example 4 (`5, 100, 5` in a
chain) it gives 10, while taking the single middle node gives 100.

So the decision at each node genuinely depends on the values below it. But
notice how *locally* it depends on them: whether you audit a service constrains
only its direct parent and its direct children. Nothing further away cares.

That is a strong hint about how to decompose the problem. If you knew the best
answer for each of a node's subtrees, could you combine them?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

Think about a single node `v` and the subtree hanging below it. Whatever the
rest of the tree looks like, the only thing the outside world needs to know
about `v`'s subtree is:

- the best total achievable inside it, and
- whether that total involved auditing `v` itself.

That second bit matters because it is the only thing that constrains `v`'s
parent. Nothing deeper inside the subtree can affect anything above `v`.

So each subtree can be summarised by exactly **two numbers**:

- the best total for this subtree **if `v` is not audited**, and
- the best total for this subtree **if `v` is audited**.

And each of those can be computed from the same two numbers for each child:

- If `v` is **not** audited, each child is unconstrained — take whichever of
  its two numbers is larger.
- If `v` **is** audited, no child may be audited, so each child must contribute
  its "not audited" number. Add `evidence[v]` on top.

That is the whole recurrence. It computes a node's answer from its children's
answers, which means you need to process children before parents.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Dynamic programming over the tree, computed bottom-up.**

Keep two arrays:

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

A leaf falls out of the same formulas with no special case: it has no children,
so the sums are empty, giving `skip = 0` and `take = evidence[v]`.

The `max(skip[c], take[c])` in the first line is the part people get wrong.
When `v` is skipped, a child is **free to choose** — it is not forced to be
audited. Writing `take[c]` there instead of the max quietly assumes every
skipped node's children must be audited, which is not what the constraint says.

**The order of computation is the implementation problem.** Every node needs
all of its children finished first, and the tree can be a 200 000-deep chain.
Recursion will not survive that — it exhausts the stack in Python and overflows
it in Rust. You need a way to process nodes children-first without recursing.

Note also that a parent's index is not guaranteed to be smaller than its
children's, so you cannot simply loop over indices in reverse and hope.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

**Step 1 — build children lists** from the parent array in one pass.

**Step 2 — produce an order where every node appears after its parent.** The
simplest way is an iterative walk from the root, pushing children as you go and
recording the order you pop them:

```
order = []
stack = [0]
while stack:
    v = stack.pop()
    order.append(v)
    for c in children[v]:
        stack.append(c)
```

Every node lands in `order` after its parent.

**Step 3 — process that order in reverse**, which puts every node after all of
its children:

```
skip = [0] * n
take = [0] * n

for v in reversed(order):
    total_skip = 0
    total_free = 0
    for c in children[v]:
        total_skip += skip[c]                      # children forced to skip
        total_free += max(skip[c], take[c])        # children free to choose
    skip[v] = total_free
    take[v] = evidence[v] + total_skip

return max(skip[0], take[0])
```

This avoids the "returning to a node after its children" bookkeeping entirely —
you just visit twice, once to fix an order and once to compute in it.

Things to get right:

- **Iterative, both passes.** A 200 000-deep chain is an explicit test case.
- **`max(skip[c], take[c])` when the parent is skipped**, not `take[c]`.
- **Leaves need no special case** — the empty sums handle them.
- **The answer is `max(skip[0], take[0])`**, not `take[0]`. The root may well be
  better left unaudited.
- **Use 64-bit numbers.** 200 000 services at 10^9 reaches 2 × 10^14.
- **Do not assume parents have smaller indices than their children.**

Complexity: each node is pushed and popped once, and each child is examined
once by its parent, so both passes are `O(n)` — giving `O(n)` time and `O(n)`
space.

</details>
