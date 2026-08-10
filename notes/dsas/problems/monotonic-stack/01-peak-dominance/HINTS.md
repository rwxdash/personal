# Hints — Peak Dominance

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

The statement already reduces the task for you:

> For each index `i`, find the nearest strictly-greater value to its left and
> the nearest strictly-greater value to its right. The answer is the gap
> between them.

So the problem is not really about stretches at all. It is two independent
sub-problems, each of the same shape:

- For every `i`, where is the nearest strictly greater element to the **left**?
- For every `i`, where is the nearest strictly greater element to the
  **right**?

Solve those two and the arithmetic is one line. Notice they are mirror images —
whatever solves one solves the other on a reversed traversal, so you only need
one idea, applied twice.

Two things to fix in your head before choosing a technique:

1. **"Strictly greater" is doing real work.** Equal values do not end a reign.
   That is why `[5, 5, 5]` gives `[3, 3, 3]` and not `[1, 1, 1]`, and it is the
   detail most likely to break an otherwise correct implementation.
2. **This is per-index, not per-range.** You are producing `n` answers, one for
   each element — there is no single window being maintained and no predicate
   being repaired. If you find yourself reaching for two pointers over a
   contiguous range, you have the wrong shape in mind.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

Think about what makes the naive approach wasteful. Scanning left from `i`
until you find something bigger is O(n) per index, but the scans are enormously
redundant: when you walk left from index 100 and pass over a long run of small
values, index 101 is about to walk over that *same* run again.

So the question is: what did the earlier scans learn that a later one could
reuse?

Here is the key observation. Suppose you are at index `i`, and some earlier
index `x` has `load[x] <= load[i]`. Then `x` is **permanently useless** as an
answer for every index `j > i`:

> If `x` would qualify for `j` — meaning `load[x] > load[j]` — then
> `load[i] >= load[x] > load[j]`, so `i` qualifies too. And `i` is closer to
> `j` than `x` is. So `i` always wins, and `x` can be thrown away for good.

An element that is both **older and no taller** than a newer one is shadowed
forever. Discard it the moment you see its shadow, and never look at it again.

That means the only elements worth remembering, at any moment, are those
forming a **decreasing** sequence going back from the current position. Every
element you can discard, you discard forever.

A structure where you push new items, discard from the same end, and inspect
the most recent survivor is a stack — and one whose contents stay sorted by
construction has a name.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Monotonic stack — two passes.**

Note this is a *stack*, not a deque or a window. You never inspect or remove
from the far end; everything happens at the top. That is the structural
difference from window-extreme problems.

**Pass 1 (left to right) — previous strictly greater.** Maintain a stack of
indices whose values are strictly decreasing from bottom to top. For each `i`:

- Pop while the value at the top is `<= load[i]`. Those elements are shadowed
  by `i` forever: `i` is newer and at least as tall, so no index to the right
  will ever stop at them.
- After popping, whatever remains on top is the nearest strictly greater
  element to the left; if the stack is empty, there is none (use `-1`).
- Push `i`.

**Pass 2 (right to left) — next strictly greater.** Identical, mirrored. Use
`n` as the sentinel for "none".

Then `answer[i] = right[i] - left[i] - 1`.

**The comparison direction is the whole problem.** You want the nearest
*strictly* greater neighbour, so you must pop elements that are `<= load[i]`,
including equal ones. Popping only on `<` leaves equal values on the stack,
they get reported as boundaries, and plateaus collapse to 1. Popping on `<` in
one pass and `<=` in the other produces an asymmetry that looks right on
distinct values and silently corrupts every tie.

Sanity-check your choice against `[5, 5, 5] → [3, 3, 3]` before writing the
second pass.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

Two arrays and two symmetric sweeps.

```
left[i]  = index of nearest strictly greater element to the left,  or -1
right[i] = index of nearest strictly greater element to the right, or n
```

**Pass 1:**

```
stack = []                       # holds indices, values strictly decreasing
for i in 0 .. n-1:
    while stack and load[stack[-1]] <= load[i]:
        stack.pop()
    left[i] = stack[-1] if stack else -1
    stack.append(i)
```

**Pass 2** is the same loop with `i` running `n-1 .. 0`, a fresh stack, and
`right[i] = stack[-1] if stack else n`.

**Combine:**

```
answer[i] = right[i] - left[i] - 1
```

Convince yourself of the arithmetic on a case where both bounds are sentinels:
the global maximum has `left = -1` and `right = n`, giving `n - (-1) - 1 = n`.
Correct — it dominates everything.

**Why this is O(n) and not O(n²).** The `while` loop looks nested, but each
index is pushed exactly once and popped at most once across the entire pass. So
the total number of pop operations over the whole sweep is bounded by `n`, and
the two passes together are O(n). This amortised argument is the thing to be
able to state out loud — an interviewer will ask why the nested loop is not
quadratic.

Watch these:

- **Store indices, not values.** You need positions to compute the gap, and
  duplicate values would be ambiguous otherwise.
- **Both passes must use `<=`**, not `<` (see level 3).
- **Empty input** returns an empty list; the loops simply never run.
- **A plateau spanning the whole series** gives every element the answer `n`,
  which is the strongest single check that your tie handling is right.
- Every answer is at least 1, since an element always dominates itself. If any
  entry comes out as 0 or negative, your sentinels are wrong.

</details>
