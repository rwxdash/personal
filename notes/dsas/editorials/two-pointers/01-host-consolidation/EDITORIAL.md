# Editorial — Host Consolidation

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

Three signals, and the third is the one that picks the technique:

1. **Input order is meaningless.** The VMs are an unordered multiset; nothing
   about position matters. Whenever the given order is irrelevant *and* the
   question is about which values combine with which, sorting is the first move
   and the real question becomes what sortedness buys you.
2. **"At most two per host" is a hard cap, not a budget.** This is what keeps
   the problem out of bin-packing territory. General bin packing is NP-hard;
   with at most two items per bin it collapses to a matching problem that a
   greedy solves exactly. If the licence tier had allowed three VMs per host,
   this would be a genuinely different — and much harder — question.
3. **Minimising hosts is maximising pairs.** With `u` unpinned VMs and `p`
   pairs formed, you use `u - p` hosts. Restating the objective this way is what
   makes a greedy argument tractable, because "grab as many pairs as possible"
   has an obvious candidate rule and "use few hosts" does not.

Sorted array + resolve from both ends + each step permanently retires an
element = **converging two pointers**. Note this is *not* a sliding window:
there is no contiguous range, no predicate being maintained, and the pointers
move toward each other rather than both rightward.

## The approach

**Split off the pinned VMs first.** They never interact with anything — one
host each, and then they are irrelevant. Every solution that keeps them mixed
into the main loop ends up with a special case inside the hot path for no
reason.

**Sort the unpinned footprints ascending.** Then converge:

```
lo, hi = 0, len(free) - 1
while lo <= hi:
    if free[lo] + free[hi] <= cap:
        lo += 1        # smallest rides along with the largest
    hi -= 1            # largest is placed either way
    hosts += 1         # exactly one host per iteration
```

**Why largest-with-smallest is optimal** (the exchange argument, which is the
part an interviewer will actually push on):

**Claim.** If `s + L <= cap`, some optimal arrangement pairs `L` with `s`.

Take any optimal arrangement and look at what `L` and `s` are doing. Every case
either already has them together or can be rewritten to have them together
without adding a host:

| In the optimum | Rewrite | Hosts used |
| --- | --- | --- |
| `{L, s}` | nothing to do | — |
| `{L}`, `{s}` | merge to `{L, s}` — legal, and *saves* a host | contradicts optimality, so this never occurs |
| `{L}`, `{s, y}` | → `{L, s}`, `{y}` | 2 → 2 |
| `{L, x}`, `{s}` | → `{L, s}`, `{x}` | 2 → 2 |
| `{L, x}`, `{s, y}` | → `{L, s}`, `{x, y}` | 2 → 2 |

The only row needing an argument is the last: is `{x, y}` legal? Yes —
`y <= L` because `L` is the largest, so `x + y <= x + L <= cap`, and
`x + L <= cap` is exactly what made `{L, x}` legal in the original
arrangement. Every rewrite keeps the host count identical, so an optimal
arrangement pairing `L` with `s` always exists and taking it costs nothing.

The contrapositive is the half people forget: **if `s + L > cap`, then `L` is
unpairable**, because `s` was its best possible partner. Place it alone
immediately rather than deferring the decision.

**The `lo == hi` case needs no special handling.** With one VM left the test
becomes `2 * free[lo] <= cap`. If it passes, `lo` advances and `hi` retreats,
the pointers cross, and one host was counted. If it fails, `hi` retreats and
one host was counted. Either way the last VM gets exactly one host and is never
paired with itself. Writing an explicit guard here is harmless but unnecessary,
and the guard is usually where the off-by-one gets introduced.

## Complexity

- **Time:** O(n log n), entirely the sort. The sweep is a single O(n) pass in
  which every VM is visited once.
- **Space:** O(n) for the filtered array (O(1) beyond that).

Why the naive approaches fail, measured:

- **Exhaustive matching** (what the test oracle does): choose the best partner
  for each VM over all subsets. Exponential; fine at n ≤ 10, hopeless at any
  real size.
- **"Repeatedly take the largest, scan for a partner"** — the plausible O(n²)
  greedy. It is *correct*, just slow. Measured on its worst case (nothing ever
  pairs, so every inner scan runs to completion) at 1.68 s for n = 8 000 with
  clean 4×-per-doubling scaling → roughly **17 minutes** at n = 200 000. That
  case is `_test_large_nothing_pairs`, which exists precisely to catch it.

The insight the two-pointer version adds over that O(n²) greedy is that once
sorted, the best partner is *always* at the `lo` end — so the inner scan is
not needed at all.

## Common wrong turns

- **Pairing largest with largest-that-fits** (or with the closest-fitting
  partner). Intuitive, and wrong: it burns a large VM that a smaller one could
  have accommodated. Random cross-checking against exhaustive matching kills
  this quickly.
- **Sorting descending and pairing adjacent elements.** Fails whenever the
  distribution is uneven; `[5, 4, 3, 2]` with `cap = 7` needs 2 hosts, and
  adjacent pairing gives 3.
- **Letting pinned VMs into the two-pointer sweep.** Either they get paired
  (wrong answer) or they need a per-iteration skip that breaks the pointer
  invariant.
- **Forgetting pinned VMs still count as hosts.** Filtering them out and never
  adding them back gives an answer that is too small by exactly the pinned
  count.
- **Special-casing `lo == hi` wrongly**, e.g. by breaking out of the loop
  without counting that VM's host, or by pairing it with itself. Both show up
  on odd-length inputs — `[1, 1, 1]` with `cap = 2` must give 2.
- **Assuming a VM can always pair with something.** A VM whose footprint equals
  `cap` never shares; `[3, 3, 3]` with `cap = 3` needs 3 hosts.
- **Treating this as bin packing** and reaching for first-fit-decreasing. It
  gives the right answer here by accident on many inputs but is not justified,
  and it obscures that the two-per-host cap is what makes an exact greedy
  possible at all.

## Interview follow-ups

1. **Three VMs per host.** The natural next question, and the honest answer is
   that the problem becomes NP-hard — it contains 3-partition. Recognising the
   complexity-class jump is the point; the follow-up is which approximation you
   would ship (first-fit-decreasing, with its 11/9 bound).
2. **Return the actual placement, not just the count.** Trivial bookkeeping on
   top of the same sweep, but worth being asked because it forces you to be
   precise about which VM went with which.
3. **Hosts with differing capacities.** Now sorting hosts *and* VMs matters, and
   the greedy needs re-justifying — a good test of whether the candidate
   understood the exchange argument or just memorised the loop.
4. **Minimise wasted memory instead of host count.** The objective changes and
   the greedy no longer applies; this is a matching problem with weights, which
   opens the door to a discussion of assignment algorithms.
5. **VMs arrive online.** You must place each VM on arrival without seeing the
   future. Competitive-ratio conversation; the offline optimum computed here
   becomes the benchmark.
6. **Why not binary search the answer?** Ask the candidate whether "can we do
   it in `k` hosts?" is monotone in `k` and cheaper to test than to solve
   directly. It is monotone, but the feasibility test is no easier than the
   original problem — a useful reminder that binary-search-on-answer needs a
   *cheap* predicate, not merely a monotone one.
