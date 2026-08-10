# Hints — Conveyor Pick Window

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Strip the warehouse framing away and the problem is:

> Given an array `stream` and a required multiset of values, find the shortest
> contiguous subarray whose value counts dominate the required counts,
> tie-broken by smallest start index.

Two properties of the problem are worth writing down before you code anything:

1. **Surplus is free.** You need *at least* `order[sku]` of each SKU, never
   exactly. That means "does this stretch fill the order?" is a question with a
   very cheap yes/no answer if you are tracking counts.
2. **The answer's length has a floor.** It can never be shorter than
   `sum(order.values())`. That is not the answer, but it tells you the quantity
   totals — not the number of distinct SKUs — are what drive the size.

The constraint that should be dictating your design is `n = 200_000`. That
rules out doing independent work for each candidate start index. Whatever you
compute for one stretch has to be *reused* when you consider the next one.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

This is a **contiguous-range scanning problem**, not a search problem and not a
sorting problem. Sorting the stream would destroy the very thing being asked
about (adjacency), so leave the order alone.

The family of technique you want is the one where you maintain a *range* over
the array and move its two ends forward, never backward, keeping a small piece
of running state that answers "is the current range good?" in O(1). Each end
travels the length of the array exactly once, giving a linear scan overall.

The property that makes that legal here: **goodness is monotone under
growth**. If a stretch fills the order, then any stretch containing it also
fills the order (extra items never hurt). Equivalently, for each right end
there is a single threshold position for the left end — left of it every
stretch is good, right of it every stretch is bad. That threshold only ever
moves forward as the right end advances.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Sliding window (two pointers) with a deficit counter.**

The data structure is a hash map of counts for the SKUs you currently hold,
plus one integer. The integer is the trick that keeps the "is the window good?"
check O(1) instead of O(k): rather than comparing your count map against the
order map on every step, maintain a single scalar measuring how far you still
are from filling the order, and update it incrementally as items enter and
leave the window.

Choose that scalar carefully. Counting *distinct unsatisfied SKUs* and counting
*total missing units* both work, but they update under different conditions —
one of them changes on every relevant item, the other only when a SKU crosses
its required quantity. Pick one and be precise about when it changes.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

Maintain `left`, a count map `have`, and a scalar `missing` initialised to the
total number of units the order requires (`sum(order.values())`).

Sweep `right` from `0` to `n-1`:

1. **Admit** `stream[right]`. If that SKU is required *and* your current count
   for it is still below its required quantity, this item covers a genuinely
   missing unit, so decrement `missing`. Then increment its count
   unconditionally. The guard matters: the 5th copy of a SKU you only need 2 of
   must not decrement `missing`.
2. **Contract** while `missing == 0` — the window is currently good, so record
   it as a candidate and then try to shrink it. Compare its length against the
   best recorded so far and replace the best only on a *strictly* shorter
   length; because `right` advances outward and you record windows in
   left-to-right order, strict comparison automatically keeps the earliest of
   any tie. Then evict `stream[left]`: decrement its count, and if that
   decrement drops the count *below* its required quantity, the window has just
   become deficient, so increment `missing`. Advance `left`.

When the sweep ends, the best recorded window is the answer; if none was ever
recorded, return `None`.

Each index is admitted once and evicted at most once, so both pointers travel
`n` steps total — O(n) time, O(k) space for the count map.

Watch the ordering inside the contract step: you must record the window
*before* evicting, and the eviction's effect on `missing` is what terminates
the loop. Getting the guard conditions backwards (decrementing `missing` for
every required-SKU item, or incrementing it whenever any count drops) is the
classic way this goes wrong, and it fails on inputs with surplus.

</details>
