# Editorial — Autoscaler Stable Windows

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

The tell is the **shape of the output**, not the shape of the input. You are
asked for a *count* of ranges, and the statement tells you that count can reach
2 × 10^10 on an input of 200 000 elements. Whenever the answer is far larger
than the input, you are being told something specific: the algorithm must
produce many units of answer per unit of work. Enumerating qualifying ranges is
not merely slow here, it is impossible — you could not write them down.

That immediately forces the standard decomposition for contiguous-range
counting:

> Fix the right endpoint `j`. Count how many valid ranges *end* at `j`. Sum
> over `j`.

Each range is counted exactly once (by its unique right endpoint), and the sum
has n terms. So the question becomes: can the per-`j` count be produced in
O(1)?

It can, because the predicate is **monotone under shrinking**: removing
elements from a range can only lower its max and raise its min, so a stable
range stays stable when you cut it down. Therefore, for a fixed `j`, the set of
valid start indices is a *suffix* of `[0, j]` — there is a single boundary
`L(j)` with everything at or after it valid and everything before it invalid.
One integer describes arbitrarily many ranges. And since growing `j` can only
tighten the constraint, `L(j)` never decreases: it is a two-pointer scan with
O(n) total motion.

Everything so far is the easy half. The hard half is the sub-problem hidden
inside the scan: **you must know the window's max and min at every step, under
insertions at the right and deletions at the left.** Insertions are trivial
(`max = max(max, new)`), deletions are not — when `left` advances past the
element that *was* the maximum, a running scalar has no idea what the new
maximum is, and rescanning the window is O(n) per step.

Sliding window + "I need the extreme of the window, with cheap retraction" is
the monotonic-deque signature.

## The approach

Keep two deques of **indices** (not values — you need the index to know when an
entry falls out of the window):

- `max_dq`: values non-increasing front to back. Front = window maximum.
- `min_dq`: values non-decreasing front to back. Front = window minimum.

**Why a deque is the right structure.** When a new value `v` arrives at index
`r`, consider any index `i < r` still in the window with `usage[i] <= v`. Index
`i` is both *older* (it leaves the window first) and *smaller* than `r`. There
is no future window containing `i` but not `r`, and while both are present `r`
dominates. So `i` can never be the maximum again — it is safe to discard
permanently. Popping those off the back is exactly what maintains monotonicity,
and it is why the total work is linear: each index is pushed once and popped at
most once across the entire run.

The sweep, per right endpoint `r`:

1. **Admit.** Pop from the back of `max_dq` while its value `<= usage[r]`, push
   `r`. Pop from the back of `min_dq` while its value `>= usage[r]`, push `r`.
2. **Restore.** While `usage[max_dq.front()] - usage[min_dq.front()] > delta`:
   if either deque's front index equals `left`, pop it from the front; then
   `left += 1`. This terminates because at `left == r` the window is one
   sample with spread 0, and `delta >= 0`.
3. **Count in bulk.** `left` is now `L(r)`. Valid starts are the integers in
   `[left, r - min_len + 1]`, so add `max(0, r - min_len + 2 - left)`.

The key invariant to state out loud: *after step 2, `[left, r]` is the longest
stable range ending at `r`, and `left` is monotonically non-decreasing across
the whole run.*

## Complexity

- **Time:** O(n). The `right` loop runs n times; `left` advances at most n
  times in total; each index enters and leaves each deque at most once. The
  nested `while`s are amortised, not multiplicative.
- **Space:** O(n) worst case for the deques — a strictly decreasing series
  never lets `max_dq` pop anything.

Why the naive approach fails: the O(n²) double loop over `(i, j)` performs
~2 × 10^10 iterations at n = 200 000. Even an O(n log n) sparse-table or
segment-tree range-max/range-min lookup per pair is still quadratic in pairs —
the fix is not a faster extreme query, it is *not asking the question per
pair*.

There is one legitimate alternative worth knowing: binary search `L(j)` using a
sparse table for O(1) range max/min. That is O(n log n) with O(n log n) space
and passes the bounds, but it is strictly more machinery than the deque version
for a strictly worse constant.

## Common wrong turns

- **Storing values instead of indices in the deques.** You then cannot tell
  whether the front element has fallen out of the window. Works on small tests,
  breaks as soon as duplicate values straddle `left`.
- **Popping the front unconditionally after advancing `left`.** The front is
  only stale if its index *is* the departing one. Popping blindly desynchronises
  the deque from the window, usually producing an over-count.
- **Using one deque and recomputing the other extreme.** The spread needs
  *both* ends; a running min alongside a max-deque fails for exactly the same
  reason the running max failed.
- **`if` instead of `while` in step 2.** One eviction is not always enough —
  the sawtooth test (`[0, 1e9, 0, 1e9, ...]`) forces multi-step contraction.
- **Applying `min_len` as a post-filter.** You cannot filter ranges you never
  enumerated. It has to be folded into the arithmetic of step 3, which is where
  the off-by-one lives: valid starts run to `r - min_len + 1`, *inclusive*.
- **Unsigned underflow in step 3.** In Rust, `right - min_len + 1` panics in
  debug and wraps in release when `min_len > right + 1`. Guard with
  `right + 1 >= min_len` before subtracting; the `[1,2,3], delta=10,
  min_len=5` example targets this directly.
- **32-bit accumulator.** `large_flat` returns 20 000 100 000, roughly 9× past
  `u32::MAX`. That test exists solely to catch this.
- **Strict vs non-strict deque comparisons.** Both `<=` and `<` on admit are
  correct *provided* front-eviction is index-based; `<=` just keeps the deques
  shorter. If you use `<` and also evict by value, duplicates break you.

## Interview follow-ups

1. **Return the longest stable run, not the count.** Trivial modification —
   but then ask for *all* maximal stable runs, and discuss why they can overlap.
2. **Drop the contiguity requirement.** Count subsets with spread ≤ delta. The
   window argument evaporates; sorting plus a different two-pointer scan gives
   `sum over i of 2^(j-i)`, and now you need modular arithmetic. Good test of
   whether the candidate understood *why* contiguity mattered.
3. **Online / streaming.** Readings arrive one at a time and you must report
   the running total after each. The algorithm is already online — worth making
   the candidate notice that, since it means no rework.
4. **Sliding `delta`.** The tolerance itself varies per tick. Does `L(j)` stay
   monotone? (No — a loosened tolerance can pull `L` backward, which kills the
   two-pointer and pushes you toward a sparse table or a divide-and-conquer.)
5. **Count ranges whose spread is *exactly* `delta`.** Answer via
   `count(<= delta) - count(<= delta - 1)`: two runs of the same routine. A nice
   check on whether the candidate reaches for inclusion–exclusion rather than
   rewriting the algorithm.
6. **Distributed.** The series is sharded across machines by time range. What
   must each shard emit so a coordinator can stitch the global count together?
   (Its internal count, plus the prefix/suffix monotone stacks at its
   boundaries — this is the mergeable-summary conversation.)
