# Editorial — Peak Dominance

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

The statement hands you the reduction — "nearest strictly greater value to the
left, nearest to the right, count what's between" — so the recognition work is
not *what* to compute but *how to compute it n times without n scans*.

The signals:

1. **Per-index output, not a per-range answer.** You produce `n` results, one
   per element. There is no single window being maintained, no predicate being
   repaired. If you reached for two pointers, you had the wrong shape in mind:
   this problem's twin sub-questions are "nearest bigger thing on each side",
   which is a fundamentally different query than "is this range valid".
2. **The naive is redundant, not just slow.** Walking left from index `i` over
   a long run of small values, then doing it again from `i+1`, re-reads the
   same run. Whenever repeated scans re-derive the same facts, the fix is to
   carry state forward that makes the re-derivation unnecessary.
3. **"Nearest greater element" is a named primitive.** Once the problem reduces
   to previous-greater and next-greater, the tool is fixed.

## The approach

**Monotonic stack, two symmetric passes.** Note it is a *stack*, not a deque:
everything happens at the top, you never inspect the far end. That is the
structural difference from window-extreme problems, which need both ends.

The correctness argument — worth stating explicitly, because it is also the
complexity argument:

> When you reach index `i`, any earlier index `x` still on the stack with
> `load[x] <= load[i]` is **permanently useless**. If `x` would ever qualify as
> the strictly-greater neighbour of some future `j > i` — meaning
> `load[x] > load[j]` — then `load[i] >= load[x] > load[j]`, so `i` qualifies
> too, and `i` is nearer to `j`. So `i` shadows `x` forever, and `x` can be
> discarded permanently.

That is why elements leave the stack once and never return, and why the stack
stays strictly decreasing.

```
# Pass 1: previous strictly greater
stack = []
for i in 0 .. n-1:
    while stack and load[stack[-1]] <= load[i]: stack.pop()
    left[i] = stack[-1] if stack else -1
    stack.append(i)

# Pass 2: next strictly greater — same loop, reversed, sentinel n
# Combine:
answer[i] = right[i] - left[i] - 1
```

### The comparison direction is the whole problem

You want the nearest **strictly** greater neighbour, so you pop on `<=` —
equal elements are discarded, not treated as boundaries. Pop on `<` and equal
values survive on the stack, get reported as blockers, and every plateau
collapses to 1.

This is not a subtle nicety, it is most of the test surface. Mutating the
reference from `<=` to `<` fails **7 of the 11** Rust tests, including all four
example cases. Verified, not assumed.

The consequence people find counter-intuitive: **joint maxima do not block each
other.** In `[1, 5, 1, 5, 1]` both 5s dominate the entire series — each is the
maximum, nothing strictly exceeds either, so neither bounds the other. The
answer is `[1, 5, 1, 5, 1]`, not `[1, 3, 1, 3, 1]`. Same at scale: in an
alternating `0, 1, 0, 1, ...` series of 200 000 samples, every one of the
100 000 peaks dominates all 200 000 positions. If your mental model said "each
peak owns its neighbourhood", re-read the definition — it says *strictly*
higher.

### The sentinel arithmetic

`answer[i] = right[i] - left[i] - 1` counts the indices strictly between the
two blockers. Check it on the global maximum, where both bounds are sentinels:
`n - (-1) - 1 = n`. Correct. If any answer comes out `0` or negative, the
sentinels are wrong — every element dominates at least itself.

## Complexity

- **Time:** O(n). The `while` loop is nested but amortised: each index is
  pushed exactly once and popped at most once per pass, so total pops ≤ `n`.
  Being able to say this out loud is the expected follow-up.
- **Space:** O(n) for the two boundary arrays plus the stack, which reaches
  depth `n` on a strictly decreasing series.

Why the naive fails: expanding outward from each index is O(n²) in the worst
case, and its worst case is the *common* case for this metric — a flat series,
where every element scans the entire array. Measured 5.60 s at n = 8 000 with
clean 4×-per-doubling scaling → roughly **58 minutes** at n = 200 000.

## Common wrong turns

- **Popping on `<` instead of `<=`.** The dominant failure. Plateaus collapse
  to 1 and joint maxima wrongly block each other.
- **Using `<=` in one pass and `<` in the other.** Looks correct on
  all-distinct inputs and silently corrupts every tie. Worse than being
  consistently wrong, because random testing with a *wide* alphabet will not
  catch it — which is why the test file cross-checks with a 1-to-4 value
  alphabet specifically.
- **Storing values instead of indices.** You need positions for the arithmetic,
  and duplicate values become ambiguous.
- **Forgetting the sentinels**, or using `0` and `n-1` instead of `-1` and `n`.
  Off-by-one on every element whose reign reaches an end of the series.
- **Trying to do it in one pass.** Possible with more cleverness (the pop
  moment tells you the *next* greater element for the popped index), but the
  two-pass version is clearer and equally fast; one-pass attempts usually get
  the tie handling wrong at the boundary.
- **Reaching for a sliding window or a deque.** Wrong shape — there is no
  range being maintained here.
- **Recursion.** Unnecessary, and a 200 000-deep monotone series will blow the
  stack.

## Interview follow-ups

1. **Largest rectangle in a histogram.** This problem *is* the core subroutine:
   once you know each bar's dominance span, the rectangle is
   `height * span`. Ask the candidate to make that connection — it is the most
   natural next question and shows whether they see the primitive or just the
   puzzle.
2. **Sum of subarray minimums.** Same span machinery, contribution counting on
   top: each element contributes `value * left_count * right_count`. Here the
   strict/non-strict asymmetry becomes *mandatory* rather than a bug — you need
   `<` on one side and `<=` on the other to attribute each subarray to exactly
   one element. Good contrast with this problem, where symmetry is required.
3. **Streaming.** Readings arrive one at a time; report each element's span as
   soon as it is known. Previous-greater is available immediately;
   next-greater is not, so the question becomes what you must buffer and for
   how long.
4. **Second-nearest greater element.** Breaks the single-stack approach and
   forces a discussion of what the stack was actually encoding.
5. **Updates.** A reading is corrected; recompute affected spans. Opens the
   door to segment trees over range-max.
6. **Why is the nested loop not O(n²)?** Ask them to produce the amortised
   argument. Candidates who memorised the pattern cannot, and it is the
   cleanest way to tell them apart from those who derived it.
