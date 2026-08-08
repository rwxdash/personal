# Hints — Host Consolidation

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Given a multiset of sizes and a capacity, group them into as few groups as
> possible, where each group holds at most two items and sums to at most `cap`.
> Some items are forced into groups of one.

Two structural facts to establish before writing anything:

1. **The pinned VMs are a separate, trivial problem.** They never interact with
   anything else. Split them off immediately, answer that part with a count,
   and you are left with a cleaner problem over the unpinned VMs only. Any
   solution that keeps them mixed in will be fighting a self-inflicted special
   case.
2. **Minimising hosts is the same as maximising pairs.** With `u` unpinned VMs,
   every host either holds one VM or two. If you form `p` pairs, you use
   `u - p` hosts. So "use as few hosts as possible" and "put as many VMs as
   possible into pairs" are the same objective, restated. That reframing is
   worth making explicit, because "maximise pairs" is a much easier thing to
   reason about greedily.

Also notice what is *not* being asked: you never need to report which VM went
where, only how many hosts. That frees you from tracking the arrangement.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

Nothing about the input order matters — the VMs are an unordered multiset, and
you are free to rearrange them. Whenever the input's given order is irrelevant
and the problem is about *which values combine with which*, **sorting is
almost always the first move**, and the real question becomes what a sorted
array lets you do that an unsorted one does not.

Once sorted, think about the largest remaining VM. It is the most constrained
item in the whole instance: it has the fewest possible partners, and every step
you delay dealing with it, its options only shrink. So the decision to make
first is "what happens to the largest one?", and there are only two outcomes —
it pairs with something, or it goes alone.

If it pairs, which partner should it take? Consider what you lose by giving it
a large partner versus a small one. The partner it takes is removed from the
pool for everyone else, so you want to spend the *cheapest* item that gets the
job done.

That line of reasoning — repeatedly resolving the most constrained item, from
one end of a sorted array, against candidates from the other end — points at a
specific traversal shape.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Sort, then two pointers converging from opposite ends.**

This is *not* a sliding window. There is no contiguous range being maintained
and no predicate being repaired; you have one pointer at the smallest unplaced
VM and one at the largest, and they walk toward each other until they cross.
Each step permanently resolves at least one VM.

The greedy rule, and the exchange argument that justifies it:

> Pair the largest remaining VM with the **smallest** remaining VM if they fit.
> If even the smallest does not fit alongside the largest, then nothing does,
> so the largest must occupy a host alone.

Why pairing largest-with-smallest is safe: suppose an optimal arrangement pairs
the largest VM `L` with some VM `x`, while the smallest VM `s` is placed
elsewhere. Since `s <= x`, swapping them keeps `L + s <= L + x <= cap` valid,
and `x` takes over whatever slot `s` had — which still fits, because `x` was
already sharing with something at least as large. So there is always an optimal
arrangement that pairs `L` with `s`, and taking that pairing never costs you
anything.

The second half of the rule is the part people forget: when `s + L > cap`, you
have *proved* `L` is unpairable, because `s` was its best possible partner.
Give it a host and move on — do not leave it for later.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

**Step 1 — split off the pinned VMs.** Count them; that count is a fixed number
of hosts. Collect the unpinned footprints into their own array. The pinned
footprints and capacity never matter again (every VM is guaranteed to fit
alone).

**Step 2 — sort the unpinned footprints ascending.**

**Step 3 — converge.** With `lo = 0`, `hi = len - 1`, and `hosts = 0`:

```
while lo <= hi:
    if a[lo] + a[hi] <= cap:
        lo += 1          # the smallest joins the largest on this host
    hi -= 1              # the largest is placed either way
    hosts += 1           # exactly one host is consumed per iteration
```

Return `pinned_count + hosts`.

The loop is worth reading carefully, because its economy hides the case
analysis. Every iteration places the current largest VM on exactly one new
host — hence the unconditional `hi -= 1` and `hosts += 1`. The only decision is
whether the smallest gets to ride along, which is the conditional `lo += 1`.

Two things to convince yourself of rather than take on trust:

- **The `lo == hi` case is already correct.** When one VM remains, the test
  becomes `2 * a[lo] <= cap`. If it passes, `lo` advances and `hi` retreats, the
  pointers cross, and one host was counted — right answer. If it fails, `hi`
  retreats, one host was counted — also right. The single VM is never
  double-counted and never paired with itself, so no special case is needed.
- **Every iteration terminates**, because `hi` always decreases.

Complexity: O(n log n) for the sort, then a single O(n) pass in which each VM is
visited once. Space O(n) for the filtered array.

Edge cases: no VMs at all (answer 0), every VM pinned (answer `n`), and a
capacity so small that nothing pairs (answer `n`).

</details>
