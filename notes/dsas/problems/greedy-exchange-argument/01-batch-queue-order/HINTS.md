# Hints — Batch Queue Order

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Order `n` items to minimise the sum of `rate[i] × (finish time of i)`, where
> finish times are the running total of durations in your chosen order.

The examples already rule out both obvious rules:

- **Sort by duration** (shortest first) — right in Example 1, wrong in
  Example 2, where the long job's high rate makes it worth running first.
- **Sort by rate** (highest first) — right in Example 2, useless in Example 1
  where all rates are equal.

So the answer depends on both numbers together. The question is *how*
together — and the useful way to find out is not to stare at the whole
ordering, but at **two adjacent jobs**.

Suppose you have some order, and you swap two jobs that sit next to each other.
Nothing before them moves. Nothing after them moves either — the pair occupies
the same total time block whichever way round it is, so every later job
finishes at exactly the same moment.

Only the two swapped jobs change. That is a very small comparison to work out,
and it tells you everything.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

Do the swap arithmetic. Say two adjacent jobs `A` and `B` start at time `T`.

**Running A first:** A finishes at `T + dA`, B at `T + dA + dB`. Their combined
cost is `rA(T + dA) + rB(T + dA + dB)`.

**Running B first:** the same expression with the roles exchanged:
`rB(T + dB) + rA(T + dB + dA)`.

Subtract. Every term involving `T` cancels, and so does `rA·dA + rB·dB`. What
survives is:

- A first costs `rB · dA` extra
- B first costs `rA · dB` extra

So **A should go before B exactly when `rB · dA < rA · dB`.**

Two things follow, and both matter:

1. The comparison depends only on the two jobs involved — not on `T`, not on
   what else is in the queue. That means it is a **consistent ordering rule**,
   and sorting by it is meaningful.
2. If no adjacent pair wants to swap, no ordering is better. (This is the
   exchange argument: any order can be transformed into the sorted one by
   adjacent swaps, and none of those swaps increases the cost.)

So the algorithm is: sort by that rule, then add up the cost.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Sort by a pairwise ratio, justified by an exchange argument.**

Rearranging `rB · dA < rA · dB` gives `dA / rA < dB / rB`. So the rule is:

> Sort ascending by `duration / rate` — equivalently, descending by
> `rate / duration`.

Intuitively: run the jobs that are *cheap in time and expensive in delay*
first.

Then sweep the sorted list once, keeping a running clock, and accumulate
`rate[i] × clock` after adding each job's duration.

**Compare with multiplication, not division.** Write the comparator as
`dA * rB < dB * rA`, never `dA / rA < dB / rB` on floating point. Two reasons:

- Floating-point division loses precision, and two jobs whose true ratios are
  equal can compare as unequal in either direction. That produces an
  inconsistent ordering, which in some languages is not merely a wrong answer
  but undefined behaviour in the sort itself.
- The products here are tiny — both values are at most 10 000, so `d * r` is at
  most 10^8 and fits comfortably in any 64-bit integer.

Ties (`dA * rB == dB * rA`) need no tie-breaking rule: the swap arithmetic says
both orders cost exactly the same, so any consistent choice is optimal.

Watch the accumulator size. The clock reaches `n × 10 000 = 10^9`, and the
total reaches roughly `5 × 10^17` — fine in a signed 64-bit integer, but far
past 32 bits.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

```
jobs = list(zip(duration, rate))

# Sort ascending by duration/rate, expressed as cross-multiplication.
# In Python: sort by the fraction directly using a key is NOT safe with floats,
# so use functools.cmp_to_key with an integer comparator, or exploit
# Fraction(d, r) as the key -- both keep it exact.
jobs.sort(key=lambda job: Fraction(job[0], job[1]))

clock = 0
total = 0
for d, r in jobs:
    clock += d          # this job finishes now
    total += r * clock  # and pays its rate for every second elapsed
return total
```

In Rust, `sort_by` with an explicit comparator is the natural fit:

```rust
jobs.sort_by(|a, b| (a.0 * b.1).cmp(&(b.0 * a.1)));
```

where `a.0` is a duration and `a.1` is a rate. Note this compares
`dA * rB` against `dB * rA`, exactly the rule from the swap arithmetic.

Points to be deliberate about:

- **Add the duration to the clock *before* charging the rate.** A job pays for
  the time up to and including its own run, not up to the moment it started.
  Charging before advancing the clock under-counts every job by
  `rate[i] * duration[i]`.
- **Integer comparison only.** No floating-point ratios.
- **64-bit accumulator**, and 64-bit for the clock too.
- **Empty input** returns 0 — the loop simply never runs.
- **`n = 1`** needs no special case; the single job finishes at its own
  duration.

Complexity: the sort is `O(n log n)` and dominates; the sweep is `O(n)`. Space
is `O(n)` for the paired list.

</details>
