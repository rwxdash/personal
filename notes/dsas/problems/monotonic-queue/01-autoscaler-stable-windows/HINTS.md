# Hints — Autoscaler Stable Windows

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Count the contiguous subarrays whose max minus min is at most `delta` and
> whose length is at least `min_len`.

Two observations that should shape everything you do:

1. **The count is astronomically larger than the input.** With n = 200 000 the
   answer can approach 2 × 10^10. Any algorithm that touches each qualifying
   range even once is dead on arrival — you have to count ranges in *batches*,
   deriving many of them from a single piece of work.
2. **Shrinking a range never hurts it.** If `usage[i..j]` has spread ≤ `delta`,
   then every range inside it also has spread ≤ `delta`, because removing
   elements can only lower the max and raise the min. Stability is inherited
   downward.

Point 2 is the lever. Think about what it implies for a *fixed* right endpoint:
as you pull the left endpoint back from `j` toward 0, at what point does the
range stop being stable, and can it ever become stable again after that?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

This is a **contiguous-range counting problem with a monotone predicate**, and
those decompose the same way every time:

> For each right endpoint `j`, find the smallest left endpoint `L(j)` such that
> `usage[L(j)..j]` is stable. Then *every* range `[i..j]` with `i >= L(j)` is
> stable and no range with `i < L(j)` is — so this single value tells you how
> many qualifying ranges end at `j`, in O(1), without enumerating any of them.
> Sum over `j`.

Furthermore `L(j)` never decreases as `j` grows: extending the right end can
only make a range harder to keep stable. A quantity that only moves forward
across the whole array is a two-pointer scan, giving O(n) total pointer motion.

So the counting skeleton is easy. The real problem is the sub-question buried
inside it: as the two endpoints move, you must know the **max and min of the
current range** at all times, and you must know them in amortised O(1) — a
recompute-from-scratch costs O(n) per step and puts you right back at
quadratic. A plain running max/min does not work either, because when the left
pointer advances past the current maximum you have no idea what the new maximum
is. What structure lets you *retract* an extreme value cheaply?

Do not forget the `min_len` floor when converting `L(j)` into a count — it is
not just a filter you can apply afterwards.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Sliding window + two monotonic deques** (a monotonic queue, one for the max
and one for the min).

- The max-deque holds indices with **non-increasing** values; its front is the
  current window maximum.
- The min-deque holds indices with **non-decreasing** values; its front is the
  current window minimum.

The insight that makes a deque the right structure: when you admit a new value,
any earlier value in the window that is smaller than it can *never* be the
maximum again — it is both older and smaller, so it is dominated for the rest
of time. Popping those from the back is what keeps the deque monotone, and it
is why each index is pushed and popped at most once overall.

When the left pointer advances past an index, discard that index from the front
of either deque if it is sitting there. That is the cheap retraction a plain
running max could not give you.

For the counting step: with `L(j)` known, the qualifying left endpoints for a
range ending at `j` form the contiguous integer interval `[L(j), j - min_len +
1]`. Count that interval's size, and clamp it at zero.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

Maintain `left = 0`, a max-deque and a min-deque holding **indices**, and a
64-bit `total = 0`.

For each `right` from `0` to `n-1`:

1. **Admit `right`.** Pop indices off the *back* of the max-deque while their
   value is `<= usage[right]`, then push `right`. Pop off the back of the
   min-deque while their value is `>= usage[right]`, then push `right`. (Using
   `<=` / `>=` rather than `<` / `>` keeps duplicates from piling up; either
   works for correctness as long as the front-eviction step below is
   index-based.)
2. **Restore stability.** While `usage[max_deque.front()] -
   usage[min_deque.front()] > delta`: if a deque's front index equals `left`,
   pop it from the front; then `left += 1`. After this loop, `left` is exactly
   `L(right)` — the smallest left endpoint keeping the range stable.
3. **Count in bulk.** Valid left endpoints for ranges ending at `right` are the
   integers `i` with `left <= i <= right - min_len + 1`. That interval has
   `right - min_len + 2 - left` elements when positive, so:

   ```
   total += max(0, right - min_len + 2 - left)
   ```

   Beware the subtraction if your indices are unsigned — compute it in signed
   arithmetic, or guard with `right + 1 >= min_len` first, before clamping.

Return `total`.

Each index enters and leaves each deque at most once and `left` only moves
forward, so the whole sweep is O(n) time and O(n) space. Every stable range is
counted exactly once — by its right endpoint — and none is ever materialised,
which is what keeps a 2 × 10^10 answer computable in a 200 000-step pass.

The two edge cases that break naive implementations: `min_len > n` must yield
0 (step 3's clamp handles it), and an empty series must yield 0 (the loop never
runs).

</details>
