# Editorial — Compaction Windows

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

The instinct almost everyone has first is to search over **arrangements**:
where do the cuts go? That space is combinatorial, and the natural dynamic
program — `dp[i][j]` = best largest-window value for the first `i` segments in
`j` windows — costs `O(n²k)`. At the stated bounds that is not "slow", it is
geological: measured growth on the test oracle extrapolates to roughly **three
years** at `n = 200_000`.

The unlock is to stop looking at the input and look at the **output**:

1. **You return a single number**, not an arrangement.
2. **That number lives in a range you can name up front** — at least
   `max(sizes)` (some window holds the biggest segment) and at most
   `sum(sizes)` (one window holds everything).
3. **`O(n log T)` in the required complexity**, where `T` is a *value* not a
   count. A log factor over the value range, rather than over `n`, is close to
   a signature.

Together those say: don't compute the answer, **guess it and check**. Replace
"what is the best arrangement?" with the much easier "can I do it under a
ceiling of `C`?"

Two properties make that legal and worthwhile, and both need checking before
you build on them:

- **Monotone.** If ceiling `C` is achievable, so is any larger ceiling — the
  same arrangement still fits. So as `C` increases the predicate reads
  `no, no, …, no, yes, yes, …`, flipping exactly once. Your answer is that flip
  point.
- **Cheap.** With a ceiling fixed, there is an obviously best strategy, and it
  runs in one linear pass. This is the part that distinguishes the technique
  from ordinary binary search over sorted data: you are searching the *answer
  space*, and it only pays off because checking is asymptotically cheaper than
  optimising.

## The approach

**The check.** Walk left to right, packing greedily into the current window,
starting a new one only when forced, and count windows:

```
def feasible(C):
    windows, current = 1, 0
    for i, s in enumerate(sizes):
        if i > 0 and checkpoints[i]:
            windows += 1; current = s      # forced cut
        elif current + s > C:
            windows += 1; current = s      # ceiling cut
        else:
            current += s
    return windows <= k
```

**Why greedy is optimal for a fixed ceiling.** Packing as much as possible into
the current window never hurts. Take any legal arrangement fitting under `C`
and repeatedly push a segment from the start of one window into the previous
window whenever it fits; this never breaks the ceiling (you only moved a
segment into a window that had room) and never *increases* the window count.
Repeating until nothing can move yields exactly the greedy arrangement. So if
any arrangement fits, greedy does.

**The two cut reasons are different and must not be conflated.** The ceiling
cut is conditional on remaining room; the checkpoint cut is **unconditional**.
It fires even when the current window is nearly empty and the cut is obviously
wasteful. Deleting the checkpoint branch fails 4 of the 10 Rust tests.

**The search.**

```
lo, hi = max(sizes), sum(sizes)
while lo < hi:
    mid = lo + (hi - lo) // 2
    if feasible(mid): hi = mid
    else:             lo = mid + 1
return lo
```

### Why `lo = max(sizes)` and not 0

This looks like a harmless micro-optimisation. It is load-bearing.

Look again at `feasible`. When a single segment exceeds the ceiling, the greedy
starts a new window and sets `current = s` — with `s > C`. It never notices
that the segment doesn't fit. So `feasible(C)` for `C < max(sizes)` can happily
return `True`, and the binary search converges below the true answer.

You need **either** `lo = max(sizes)` (making the situation unreachable) **or**
an explicit `if s > C: return False` guard inside the check. Not neither.
Mutating the reference to `lo = 0` fails 6 of the 10 tests — including the
random cross-check, which is how you would want to find out.

`hi = sum(sizes)` is always feasible because `k >= 1`, so the search always
terminates on a real value.

### The loop shape

`while lo < hi` with `hi = mid` / `lo = mid + 1` converges on the **smallest**
feasible value, which is what "minimise" asks for. The two classic corruptions:
`hi = mid - 1` skips past the answer, and `lo = mid` loops forever when
`hi == lo + 1`.

## Complexity

- **Time:** `O(n log T)`. The range spans up to 2 × 10^14, so about 48
  iterations of an `O(n)` check.
- **Space:** `O(1)` beyond the input.

## Common wrong turns

- **Reaching for the DP.** Correct, and hopeless at these bounds.
- **Starting `lo` at 0 or 1 without a `s > C` guard.** See above — the single
  most instructive bug in this problem.
- **Treating the checkpoint cut as conditional**, e.g. `if checkpoints[i] and
  current + s > C`. Silently ignores every checkpoint that happened to fit.
- **Applying `checkpoints[0]`.** Segment 0 always starts window 0; treating its
  flag as a real cut produces an off-by-one window count on those inputs.
- **Balancing segment counts instead of bytes.** Example 1 exists for this: the
  even split by count is not the best split by bytes.
- **`hi = mid - 1`, or `lo = mid`.** Wrong convergence, or an infinite loop.
- **32-bit arithmetic.** The total reaches 2 × 10^14.
- **Aiming for exactly `k` windows.** The problem says *at most* `k`. Using
  fewer is always allowed and sometimes forced (`k > n` cannot be met exactly
  with non-empty windows). The `<=` in the feasibility test is what handles it.

## Interview follow-ups

1. **Return the actual split, not just the value.** Run `feasible` once more at
   the final answer and record the cut positions. Cheap, and it confirms the
   candidate understands that the search located a value, not an arrangement.
2. **Maximise the smallest window instead.** The predicate flips direction —
   good check on whether they can re-derive monotonicity rather than pattern-match.
3. **Windows may reorder segments.** Now it is bin packing, and NP-hard. The
   contiguity constraint is exactly what made an exact answer reachable, and
   noticing that is the point.
4. **Weighted windows** — window `j` has its own capacity. Greedy still works
   per fixed ceiling? Make them argue it carefully; the exchange argument
   changes.
5. **Why not parametric search / Lagrangian relaxation (Alien Trick)?** It
   applies to the convex variant and is worth knowing exists, but it is far
   more machinery for no gain here.
6. **What if `sizes` could be negative?** Monotonicity of the greedy check
   breaks, and with it the whole approach. A good final question, because it
   probes whether the candidate knows *why* the technique was valid rather than
   that it worked.
