# Hints — Mesh Link Failures

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Start with an undirected graph. Delete a given sequence of edges one at a
> time. After each deletion, report the number of connected components.

Before reaching for any structure, notice which parts of the input are doing
real work:

- **Links are identified by index, not by endpoints.** That is not decoration.
  The same pair may appear twice, and Example 3 shows that answering by
  endpoint pair gives a different — wrong — answer. Every place you handle a
  link, ask yourself whether you are handling *this cable* or *this pair of
  nodes*.
- **A self-loop connects nothing.** It should be inert everywhere: in the
  starting count and in every step.
- **`failures` may be much shorter than `links`.** Many links are never cut at
  all, and those are the ones that hold the mesh together throughout.

And then the sentence the statement goes out of its way to include:

> "Because the whole failure sequence is known before the experiment starts,
> you may analyse it however you like — you are not required to answer step `k`
> before you have looked at step `k+1`."

Statements do not say that by accident. It is telling you the *order in which
you produce the answers* is yours to choose. Sit with that before opening the
next hint — it is most of the problem.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

This is a **connectivity-maintenance problem**, and the crucial thing to
realise is that connectivity is *deeply* asymmetric between the two directions:

- **Adding an edge is easy.** Two groups become one. A tiny amount of
  bookkeeping merges them, and the component count drops by exactly one — or
  by zero, if the endpoints were already together.
- **Removing an edge is hard.** When you cut a link, did the component split?
  You cannot tell without searching for an alternative route, and if it *did*
  split you must somehow tear one group into two — an operation the natural
  data structures for this simply do not support. (Maintaining connectivity
  under arbitrary online deletions is a genuinely hard research problem, and it
  is not what this question wants.)

So you have a problem posed entirely in the hard direction, plus explicit
permission to process the timeline in any order you like. That is the whole
puzzle: **turn the hard direction into the easy one.**

Also think about how to count components without ever enumerating them. If you
start from `n` isolated nodes, what exactly does each *successful* merge do to
the count, and what does an unsuccessful one do?

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Disjoint-set union (union-find) with path compression and union by
size/rank, applied to the failure timeline in reverse.**

Run time backwards. The final state — after every listed failure has happened —
is the *most* fragmented the mesh ever gets, and you can build it directly:
take the links whose indices never appear in `failures` and union them. From
there, replaying the failures in reverse order means **adding** links back one
at a time, which is exactly the direction union-find handles in near-constant
amortised time.

For the count, maintain a single integer rather than ever enumerating
components: initialise it to `n` (all nodes isolated) and decrement it by one
on every union that actually merges two distinct sets. A union whose endpoints
already share a root — a redundant cable, or a self-loop, whose endpoints are
trivially the same root — changes nothing, which is precisely the behaviour
Examples 3 and 4 demand. You get the duplicate and self-loop handling for free
if you write the union honestly and only decrement on a real merge.

The remaining work is purely index bookkeeping: you are producing answers in
the opposite order from the one you must return them in, and the state you hold
after re-adding a link corresponds to a *different* step number than the one
you just processed. Get that mapping wrong by one and every answer shifts.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

Let `q = len(failures)`. Write `S_k` for the mesh state after the first `k`
failures, so the answer you must return is
`[components(S_1), components(S_2), ..., components(S_q)]`.

**Step 1 — build the final state `S_q`.** Mark every index in `failures` as
doomed. Initialise the DSU over `n` nodes with `count = n`. Union every link
whose index is *not* doomed, decrementing `count` only when the union actually
merges two different sets. When this finishes, `count == components(S_q)`.

**Step 2 — walk backwards, adding links back.** Record `answer[q-1] = count`,
since that is `components(S_q)`. Then for `k` from `q-1` down to `1`:

- Union `links[failures[k]]` — this is the link that *caused* the transition
  from `S_k` to `S_{k+1}`, so putting it back moves you from `S_{k+1}` to
  `S_k`. Decrement `count` if it merged.
- Record `answer[k-1] = count`, which is now `components(S_k)`.

Return `answer`.

The mapping in one line: **after re-adding `failures[k]`, the DSU represents
`S_k`, whose count belongs at `answer[k-1]`.** Write that down and check it
against Example 2 by hand before you trust your loop bounds — this is where
almost all bugs in this problem live.

**The DSU itself.** `parent` and `size` arrays, `find` with path compression,
`union` by size returning a boolean "did this merge two distinct sets". Write
`find` **iteratively** — with 200 000 nodes a recursive `find` can nest deeply
enough to exhaust the stack, and a union-by-size implementation that forgets
the size heuristic can build chains long enough for that to happen.

Complexity: one pass to build (`O(m α(n))`), one pass backwards
(`O(q α(n))`), so `O((n + m) α(n))` overall with `O(n + m)` space.

Edge cases the loop bounds must survive: `failures` empty (return an empty
list immediately — step 2 must not underflow), `q == 1` (the backwards loop
body never runs and `answer[0]` comes entirely from step 1), and self-loops
(inert, because `find(u) == find(u)`).

</details>
