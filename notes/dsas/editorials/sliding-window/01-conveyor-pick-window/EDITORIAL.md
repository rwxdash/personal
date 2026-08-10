# Editorial — Conveyor Pick Window

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

Three signals in the statement point the same direction, and an experienced
engineer reads them together rather than one at a time:

1. **The answer is a contiguous range.** The belt "cannot be paused, reversed,
   or restarted" is flavour text for *the answer is a subarray*. Any framing
   that forbids reordering is protecting adjacency, which means adjacency is
   load-bearing.
2. **The predicate is monotone under growth.** "Collecting more than the
   required quantity is fine" is not a convenience — it is the load-bearing
   sentence. It guarantees that if a stretch fills the order, every stretch
   containing it also fills the order. Without that, surplus could break a
   window, and the technique below would be invalid.
3. **`n = 200_000` with an O(n) target.** Linear time over a subarray problem
   leaves very little room: you get one pass, and whatever you compute for one
   window has to be repaired into the next window rather than recomputed.

Monotone predicate + contiguous range + linear budget is the sliding-window
signature. The moment you notice the predicate is monotone, you know the left
boundary is a *function* of the right boundary and that it only moves forward,
which is exactly what makes two pointers legal.

## The approach

Maintain a window `[left, right]`, a count map `have`, and one integer.

The integer is where most implementations either shine or fall apart. Comparing
`have` against `order` to test the window costs O(k) per step, which drags the
whole algorithm to O(n·k) — at k = 10 000 that is as bad as the quadratic scan
you were trying to avoid. Instead track a scalar:

```
missing = number of required units not yet covered by the window
```

initialised to `sum(order.values())`. The window fills the order **iff**
`missing == 0`, an O(1) test.

The invariant to keep straight is what changes `missing`:

- On **admitting** `stream[right]`: if it is required *and* `have[sku] <
  order[sku]` before the increment, this copy covers a previously-missing unit
  → `missing -= 1`. Otherwise it is surplus and changes nothing. Increment
  `have[sku]` either way.
- On **evicting** `stream[left]`: decrement `have[sku]`, then if `have[sku] <
  order[sku]` after the decrement, a unit just became missing → `missing += 1`.

Note the asymmetry: the admit check reads the count *before* the update and the
evict check reads it *after*. Both are asking the same question — "did this
item cross the requirement boundary?" — and writing them symmetrically is the
single most common source of off-by-one bugs here.

The sweep:

```
for right in 0..n:
    admit(stream[right])
    while missing == 0:
        record (left, right) if strictly shorter than best
        evict(stream[left]); left += 1
```

Two details worth naming:

- **Record before evicting.** The window is valid at the top of the loop body;
  once you evict you may have destroyed it.
- **Strict `<` gives the tie-break for free.** Candidate windows are recorded
  in non-decreasing order of `left`, so refusing to replace on equal length
  keeps the earliest one. No extra tie-breaking logic is needed — and adding
  `<=` silently breaks Example 3.

## Complexity

- **Time:** O(n). `right` advances n times; `left` advances at most n times
  across the entire run, because it never moves backward. The inner `while` is
  therefore amortised, not nested — this is the point people miss when they
  eyeball the code and call it quadratic.
- **Space:** O(k) for `have`, where k is the number of distinct SKUs the order
  mentions. Filler SKUs are never inserted into the map.

Why the naive approach fails: fixing each `left` and extending `right` until
the order is filled is O(n²·k) worst case, or O(n²) with careful counting. At
n = 200 000 the measured cost of the pure-Python checker in the test file
extrapolates to roughly **50 minutes** (it is 5 s at n = 8 000 and scales 4×
per doubling). The `large_*` tests exist to make that failure mode unmissable.

## Common wrong turns

- **Comparing the whole count map every step.** Correct, but O(n·k). Passes the
  examples, dies on inputs with many distinct ordered SKUs.
- **Counting distinct satisfied SKUs instead of missing units.** This *can*
  work, but only if you increment the "satisfied" counter exactly when a SKU
  reaches its requirement and decrement exactly when it falls below. People who
  choose this variable usually forget that quantities are > 1 and end up
  treating one occurrence as full satisfaction — `_test_edges` catches this
  with `["A"] * 5` against `{"A": 2, "B": 1}`.
- **Letting surplus decrement `missing`.** Drop the `count < need` guard on
  admit and `missing` goes negative; `missing == 0` then never fires again and
  you return `None` on perfectly solvable inputs.
- **`<=` when recording the best window.** Returns the *latest* shortest window.
  Example 3 and the `examples` test target exactly this.
- **Shrinking with `if` instead of `while`.** Leaves the window larger than
  necessary; fails on inputs with long runs of a single SKU
  (`["A"]*10 + ["B"]`).
- **Not handling required SKUs absent from the stream.** `missing` never
  reaches 0, the `while` never runs, `best` stays `None` — which is correct, so
  this one is usually fine by construction. It only breaks if you initialise
  `best` to something non-`None`.

## Interview follow-ups

A Staff-level interviewer would keep going:

1. **Streaming.** The belt is infinite and you cannot store it. Report the
   shortest filling window seen so far, online. What state must you keep, and
   why is the window's own contents no longer discardable?
2. **Weighted cost.** Each belt position has a cost (grabbing during peak hours
   is more expensive) and you want the minimum-cost filling window rather than
   the shortest. Does the two-pointer argument survive? (It does for
   non-negative costs — prefix sums plus the same window — and breaks the
   moment costs may be negative, because the monotone-length argument no longer
   implies monotone cost.)
3. **k orders at once.** Given m orders, find each one's shortest window in
   better than m independent passes. Discuss what can be shared: a single pass
   maintaining per-order deficits is O(n·m) but cache-friendly; suffix
   structures do better when the orders overlap heavily.
4. **Two arms.** You may engage the arm twice, giving two disjoint stretches
   whose union fills the order. Minimise the total engaged length. This is a
   genuine step up: prefix/suffix best-window arrays computed in two passes,
   then a split point scan.
5. **Why not binary search on the answer length?** Length-feasibility *is*
   monotone here, so binary search + a fixed-size window check gives
   O(n log n) — a legitimate alternative worth being able to derive on the
   spot, and a good way to demonstrate you know why the linear version is
   strictly better.
