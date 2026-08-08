# Editorial — Mesh Link Failures

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

The recognition here is not "this is a graph problem" — that is obvious from
the first line. It is noticing that the problem is posed in the **hard
direction**, and that the statement quietly grants you permission to flip it.

Connectivity under edge changes is violently asymmetric:

- **Adding an edge** merges two groups into one. Cheap, local, and the
  component count drops by exactly one — or by nothing at all if the endpoints
  were already together.
- **Removing an edge** asks "did this split the component?", which you cannot
  answer without searching for an alternative route. And if it did split, you
  must tear a group in two — an operation disjoint-set structures fundamentally
  do not support, because a compressed tree has thrown away the information
  about which node arrived via which edge.

Maintaining connectivity under arbitrary *online* deletions is a genuinely hard
problem (polylog-per-operation dynamic connectivity, deep research
machinery). If you found yourself heading there, you missed a sentence:

> "Because the whole failure sequence is known before the experiment starts,
> you may analyse it however you like — you are not required to answer step `k`
> before you have looked at step `k+1`."

That is the statement handing you **offline processing**. The entire timeline
is available up front, which means you may traverse it in whatever order suits
you — including backwards. And backwards, every deletion is an insertion.

The general lesson, which is what a Staff-level interviewer is actually
probing: *when a data structure supports one direction cheaply and the other
not at all, check whether the problem is offline. If it is, reverse time.*

## The approach

Let `q = len(failures)` and write `S_k` for the mesh state after the first `k`
failures. You must return `[|S_1|, |S_2|, ..., |S_q|]` where `|·|` is the
partition count.

**Phase 1 — build the endpoint, `S_q`.** Mark every index appearing in
`failures` as doomed. Initialise a DSU over `n` nodes with `count = n`, then
union every link whose index is *not* doomed. Those links survive the entire
experiment, so what you have built is exactly `S_q`, the most fragmented state
the mesh ever reaches. `count` is now `|S_q|`.

**Phase 2 — replay backwards.** Record `answer[q-1] = count`. Then for `k` from
`q-1` down to `1`, union `links[failures[k]]` and record `answer[k-1] = count`.

The index mapping is the part worth deriving rather than memorising:
`failures[k]` (0-based) is the link whose removal turns `S_k` into `S_{k+1}`.
So *putting it back* moves the structure from `S_{k+1}` to `S_k`. After that
union, `count` is `|S_k|`, and `|S_k|` is the answer for step `k`, which lives
at `answer[k-1]`. One sentence, and it is where essentially every bug in this
problem lives — check it against Example 2 by hand.

**Counting without enumerating.** Maintain a single integer initialised to `n`
and decrement it only when a union genuinely merges two distinct roots. This is
not just an optimisation; it is what makes the two awkward input features
disappear:

- A **redundant cable** between already-joined nodes hits `ra == rb` and
  changes nothing — exactly what Example 3 demands.
- A **self-loop** `(u, u)` also hits `ra == rb`, trivially. It needs no special
  case at all, provided you did not write one.

If you find yourself adding `if u == v: continue` or deduplicating `links`, you
are patching a symptom of counting components some other way.

**The DSU.** `parent` and `size` arrays, path compression in `find`, union by
size. Write `find` **iteratively** — 200 000 nodes is enough depth to blow the
stack if you skip the size heuristic and recurse.

## Complexity

- **Time:** O((n + m) α(n)). Phase 1 does `m` unions, phase 2 does `q ≤ m`
  more, each amortised inverse-Ackermann — constant for any input that fits in
  memory.
- **Space:** O(n + m) for the DSU arrays and the doomed flags.

Why the naive approach fails: rebuilding the adjacency lists and flood-filling
after every failure is O(q·(n + m)). Measured at 0.53 s for n = 2 000 with
q = 1 000 and clean quadratic scaling, which extrapolates to roughly **88
minutes** at n = 200 000 with q = 100 000. `_test_large_chain_scrambled` is
sized for exactly that.

An intermediate trap worth naming: "only re-run the flood fill on the component
that lost an edge" feels like a real optimisation, but on a path graph the
first cut still touches half the nodes, and the total is still quadratic.

## Common wrong turns

- **Trying to make union-find delete.** There is no `split`. Some candidates
  attempt rollback/undo union-find here; that works for a *stack* of
  operations (undo the most recent), not for removing an arbitrary edge added
  long ago. Recognising why rollback DSU does not apply is worth saying out
  loud.
- **Reaching for online dynamic connectivity.** Correct but wildly
  disproportionate, and it ignores the offline permission in the statement.
- **Getting the reverse-index mapping off by one.** Producing
  `[|S_0|, ..., |S_{q-1}|]` — an answer shifted by one step, which passes any
  test where the counts happen to be symmetric. Example 2's `[2, 3, 4]` catches
  it.
- **Treating failures as endpoint pairs.** Removing "the edge between u and v"
  rather than "link index i" silently kills every parallel cable at once.
  Example 3 and `_test_large_duplicate_cables` target this directly.
- **Special-casing self-loops during the build.** Harmless if done
  consistently, wrong if it also changes the initial `count`.
- **Recomputing the component count by scanning for roots** after each union.
  That is O(n) per step and drags you back to quadratic. Maintain the running
  counter.
- **Forgetting that links never listed in `failures` stay up forever.**
  Starting phase 1 from an empty graph rather than from the surviving links
  gives wildly high counts (`_test_large_untouched_backbone` returns all 1s
  precisely to catch this).
- **Empty `failures`.** Phase 2's `answer[q-1]` underflows if you do not return
  early.
- **Recursive `find` without union by size.** Stack overflow on the 200 000-node
  chain.

## Interview follow-ups

1. **Report the size of the largest partition after each failure**, not just
   the count. Easy forwards (the DSU already tracks sizes, and a running max
   only ever grows under union) — and worth noticing that the running max works
   *only* because you are going backwards, where sizes only increase.
2. **Interleave queries and failures.** "Is node a still reachable from node b
   at step k?" Answer offline by bucketing queries per step and evaluating them
   during the reverse sweep, at their correct point in time.
3. **Links come back up.** Now the timeline has both additions and removals,
   and reversal alone no longer suffices. This is the entry point to
   **offline dynamic connectivity**: a segment tree over the time axis with
   rollback-capable union-find (union by size, no path compression), giving
   O(m log m log n).
4. **Nodes fail, not links.** Node deletion also reverses cleanly — but ask
   what "the component count" means when nodes are removed from the universe,
   and watch whether the candidate notices the definition needs pinning down.
5. **Streaming / online for real.** Suppose you truly cannot look ahead. What
   is the best you can do? (Now the answer really is Holm–de Lichtenberg–Thorup;
   knowing it exists and that it is polylog amortised is enough.)
6. **Which single link is most critical?** Find the link whose removal
   increases the partition count — the bridges. Tarjan's bridge-finding in
   O(n + m), a natural pivot from this problem's shape.
