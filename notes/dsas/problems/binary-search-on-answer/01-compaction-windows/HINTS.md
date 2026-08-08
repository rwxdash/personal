# Hints — Compaction Windows

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Cut an array into at most `k` contiguous pieces, respecting a set of forced
> cut positions, minimising the largest piece sum.

The instinct most people have first is to search over *arrangements* — where do
the cuts go? — and that space is enormous: choosing `k-1` cut positions out of
`n-1` is combinatorial, and a dynamic program over it costs `O(n²k)`.

Before going down that road, look at the shape of what you are asked to return.
It is **a single number**, and that number is bounded:

- It can never be below `max(sizes)` — whichever window holds the biggest
  segment is at least that big.
- It can never exceed `sum(sizes)` — one window holding everything.

So the answer lives somewhere in a range you can name up front. That is worth
sitting with. Instead of asking "what is the best arrangement?", you could ask
a much simpler yes/no question about a *candidate* answer, and let the answers
to that question locate the true value.

What yes/no question would that be, and is it easier than the original problem?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

The question to ask is:

> **"Can I do it with no window exceeding `C` bytes?"**

Two properties make this the right move, and you should verify both before
building on them:

1. **The predicate is monotone.** If a limit of `C` is achievable, then so is
   any larger limit — the same arrangement still works. So as `C` runs from
   `max(sizes)` upward, the answer to the question is `no, no, no, …, yes, yes,
   yes`, flipping exactly once. The value you want is the position of that
   flip.
2. **The predicate is far cheaper than the original problem.** Deciding *how*
   to arrange windows optimally is hard; deciding whether *some* arrangement
   fits under a fixed ceiling is easy, because with a ceiling in hand there is
   an obviously best strategy — and it takes a single linear pass.

That second point is the crux, and it is what separates this technique from
"binary search over a sorted array". You are not searching data. You are
searching the *space of possible answers*, and the search is only worthwhile
because checking a candidate is dramatically cheaper than optimising directly.

Think about what that linear check should do when it walks the segments with a
ceiling `C` in hand — and what the checkpoints force it to do differently.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Binary search on the answer, with a greedy feasibility check.**

Search the integer range `[max(sizes), sum(sizes)]` for the smallest `C` whose
check returns true.

**The check, `feasible(C)`:** walk the segments left to right, packing greedily
into the current window. Start a new window only when forced. Count the windows
and return `count <= k`.

Greedy is optimal *for a fixed ceiling* — this is the part worth convincing
yourself of rather than assuming. Packing as much as possible into the current
window never hurts: any legal arrangement can be transformed into the greedy
one by pushing segments leftward, and pushing a segment into an earlier window
never increases the number of windows. So if any arrangement fits under `C`,
the greedy one does.

There are exactly **two** reasons to start a new window, and conflating them is
the usual bug:

- The current window would exceed `C` by adding this segment.
- This segment is a **checkpoint** (and is not segment 0), which must begin a
  window regardless of how much room is left.

The checkpoint case is unconditional. It fires even when the current window has
plenty of space, and even when starting a new window is obviously wasteful.

One guard before the loop: if any single segment exceeds `C`, no arrangement
works — but starting the search at `lo = max(sizes)` makes that impossible by
construction, which is why the lower bound is chosen that way rather than at 0.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

```
def feasible(C):
    windows = 1
    current = 0
    for i, s in enumerate(sizes):
        if i > 0 and checkpoints[i]:
            windows += 1          # forced cut, regardless of remaining room
            current = s
        elif current + s > C:
            windows += 1          # cut because the ceiling would be breached
            current = s
        else:
            current += s
    return windows <= k

lo, hi = max(sizes), sum(sizes)
while lo < hi:
    mid = lo + (hi - lo) // 2
    if feasible(mid):
        hi = mid                  # mid works; the answer is mid or smaller
    else:
        lo = mid + 1              # mid fails; the answer is strictly larger
return lo
```

Points worth being deliberate about:

- **`lo = max(sizes)`, not `0` or `1`.** Any `C` below the largest segment is
  infeasible, and starting there also means `feasible` never has to
  special-case a segment that cannot fit anywhere.
- **`hi = sum(sizes)`** is always feasible (one window holds everything, and
  `k >= 1`), so the search always terminates on a real answer.
- **The `lo < hi` / `hi = mid` / `lo = mid + 1` shape** converges on the
  *smallest* feasible value. Using `hi = mid - 1` here would skip past the
  answer; using `lo = mid` would loop forever when `hi = lo + 1`.
- **Compute `mid` as `lo + (hi - lo) // 2`.** In Rust, `(lo + hi) / 2` on
  values summing to 2 × 10^14 is fine in `i64` but the habit matters.
- **Use 64-bit types throughout.** The total reaches 2 × 10^14.
- **The problem guarantees solvability**, so you never need to detect and
  report failure — but note *why* it is guaranteed: with at most `k - 1`
  checkpoints after index 0, the forced cuts alone can never exceed `k`
  windows.

Complexity: `O(log T)` iterations of an `O(n)` check, so `O(n log T)` with
`T ≈ 2 × 10^14`, i.e. about 48 passes over the array. Space is `O(1)` beyond
the input.

</details>
