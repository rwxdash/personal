# Editorial — Settlement Runs

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

Two signals, and the second is the one that eliminates the wrong technique:

1. **The answer dwarfs the input.** ~2 × 10^10 qualifying runs from 200 000
   elements. You cannot visit them individually — not even at O(1) each — so
   the algorithm must emit answer in *batches*.
2. **Deltas can be negative, and the statement goes out of its way to say so.**
   This is the load-bearing constraint. With non-negative values, a run's sum
   grows monotonically as you extend it, and a sliding window works. With
   refunds in the mix, extending a run can move its sum *down*, so there is no
   monotone predicate, no "grow until it qualifies then shrink", and a window
   is not merely slow here — it is **structurally invalid**. Any solution built
   on two pointers is wrong regardless of how fast it runs.

Once windows are off the table, the question becomes: what *is* a run's sum,
if not something you accumulate by walking? And the answer is that every run's
sum is a **difference of two running totals**:

```
sum(deltas[i .. j-1]) = P[j] - P[i]      where P[k] = sum of the first k deltas
```

That single identity is the prefix-sum pattern in its entirety. It converts a
question about O(n²) runs into a question about n + 1 numbers.

The divisibility condition then does the rest of the work:

> `P[j] - P[i]` is divisible by `m` **iff** `P[i] ≡ P[j] (mod m)`.

So you are not looking for runs. You are looking for **pairs of equal values**
in an array of n + 1 residues — and counting pairs of equal values is a
one-pass tally, never an enumeration.

## The approach

Walk the n + 1 prefixes, keeping two maps keyed by residue:

- **`seen_count[r]`** — how many prefixes so far had residue `r`. On arriving
  at a prefix with residue `r` that you have seen `k` times, you have just
  closed `k` settleable runs at once. `count += k`, then `seen_count[r] += 1`.
  This is the batching that makes a 10^10 answer reachable in 200 000 steps.
- **`first_index[r]`** — the earliest prefix position with residue `r`. The
  longest run ending here is `j - first_index[r]`. Write it only when the
  residue is new; never overwrite.

**Seed both with the empty prefix**: `seen_count[0] = 1`, `first_index[0] = 0`.
`P[0] = 0` is a real prefix — it is the one that every run *starting at index
0* is measured against. Omitting it is the single most common bug here, and it
is nasty because the answer is only slightly wrong: you lose exactly the runs
anchored at the start, so small hand-checked examples often still pass.

**Order matters in the tally.** Read `seen_count[r]` *before* incrementing it.
Increment first and every prefix pairs with itself, inflating the count by
exactly n.

### The modulus trap

`%` does not mean the same thing in every language, and this problem is built
on top of that:

| Expression | Python | Rust |
| --- | --- | --- |
| `-1 % 3` | `2` | `-1` |

Python's `%` is a true modulus for positive divisors and lands in `0..m-1`.
Rust's `%` is a *remainder* and keeps the sign of the dividend. Key a `HashMap`
on the raw Rust remainder and residue class 2 gets split across buckets `2` and
`-1`, so every run spanning a negative running total is silently lost. Use
`rem_euclid`, or `((x % m) + m) % m` in languages without it.

This was verified, not assumed: swapping `rem_euclid` for `%` in the reference
fails exactly `negative_modulo` and `random_against_brute` while the other nine
tests still pass. That is precisely the shape of a bug that ships.

## Complexity

- **Time:** O(n). One pass, O(1) amortised hash operations per step.
- **Space:** O(min(n, m)). The maps are keyed by *residue*, so they hold at
  most `min(n + 1, m)` entries even when `m` is 10^9. If `m` is small, an array
  of size `m` beats a hash map on constants.

Why the naive approach fails: the O(n²) double loop over `(i, j)` measures
3.69 s at n = 8 000 with clean 4×-per-doubling scaling → roughly **38 minutes**
at n = 200 000.

## Common wrong turns

- **Reaching for a sliding window.** The fatal one. It is not a performance
  mistake, it is a correctness mistake: negative deltas break the monotonicity
  a window requires. If you wrote two pointers here, the lesson is to check
  monotonicity *before* choosing the pattern, not after.
- **Forgetting to seed `P[0] = 0`.** Loses every run starting at index 0.
- **Incrementing the tally before reading it.** Over-counts by exactly n.
- **Overwriting `first_index`.** Measures the longest run against the most
  recent occurrence of the residue instead of the earliest, so `longest` comes
  out too small. `[3, 3, 1, 1, 3]` with `m = 3` targets this.
- **Tracking `longest` as "most recent match" instead of a maximum.** Same test
  catches it, because the longest run there appears early.
- **Raw `%` on negative running totals** (see above).
- **Comparing sums instead of residues** — i.e. storing `P[j]` and searching for
  `P[j] - k*m`. Correct in principle, unbounded in practice.
- **32-bit anything.** The count reaches 2 × 10^10 and the running total itself
  reaches 2 × 10^14 (200 000 × 10^9) long before that.
- **Special-casing `m = 1`.** Unnecessary — every residue is 0 and the general
  code returns `n(n+1)/2` on its own. If you needed a special case, something
  else is wrong.

## Interview follow-ups

1. **Sum equal to an exact target `t` rather than divisible by `m`.** Same
   skeleton, different key: tally `P[j] - t` instead of residues. Worth doing
   to confirm the candidate understands the identity rather than the recipe.
2. **Longest run only, no count.** Then you need `first_index` but not the
   tally — a good check on whether they know which structure serves which half.
3. **Return the actual run, not just its length.** One extra stored index.
4. **Handle updates.** A delta at position `i` is corrected; re-answer without
   a full rescan. This is the doorway to a Fenwick/segment tree conversation,
   since every prefix from `i` onward shifts.
5. **Streaming with bounded memory.** The residue map can hold `min(n, m)`
   entries; what if `m` is 10^9 *and* n is 10^9? Discussion of sketching and
   what exactness costs.
6. **Why is this not a sliding window?** The best follow-up of all — ask them
   to state the property that windows need and to point at the exact line of
   the statement that revokes it.
