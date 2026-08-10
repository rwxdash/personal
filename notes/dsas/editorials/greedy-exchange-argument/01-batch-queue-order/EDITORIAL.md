# Editorial — Batch Queue Order

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

The examples are built to kill both obvious rules before you commit to one:

- **Shortest job first** wins Example 1 and loses Example 2, where a long job
  with ten times the cost rate should clearly run first.
- **Highest rate first** wins Example 2 and tells you nothing in Example 1,
  where every rate is identical.

So the order depends on both numbers together. The productive move is *not* to
reason about the whole ordering at once — it is to look at **two adjacent
jobs**.

That works because an adjacent swap is almost entirely local. Everything before
the pair is untouched. Everything *after* the pair is also untouched, because
the pair occupies the same block of time whichever way round it sits, so every
later job finishes at exactly the same moment.

Only the two swapped jobs change. That is a two-term comparison, and it decides
everything.

## The approach

Two adjacent jobs A and B starting at time `T`:

- **A first:** `rA(T + dA) + rB(T + dA + dB)`
- **B first:** `rB(T + dB) + rA(T + dB + dA)`

Subtract. Every `T` term cancels, and so does `rA·dA + rB·dB`. What survives:

- A first costs an extra `rB · dA`
- B first costs an extra `rA · dB`

> **A goes before B exactly when `dA · rB < dB · rA`** — equivalently
> `dA/rA < dB/rB`.

Two things follow, and both matter:

1. **The comparison is local.** It does not involve `T` or anything else in the
   queue, so it defines a consistent total order and sorting by it is
   meaningful.
2. **The sorted order is optimal.** This is the exchange argument: any ordering
   can be turned into the sorted one by a sequence of adjacent swaps, and by
   the rule above none of those swaps increases the cost. So no ordering beats
   it.

Then sweep the sorted list with a running clock:

```
clock = total = 0
for d, r in sorted_jobs:
    clock += d          # this job finishes now
    total += r * clock
```

Read the rule intuitively: **run the jobs that are cheap in time and expensive
in delay first.** A job's "priority" is its rate per unit of time it occupies.

### Compare with multiplication, never division

Write the comparator as `dA * rB` versus `dB * rA`. Not `dA / rA < dB / rB` on
floating point.

Both values are at most 10 000, so each product is at most 10^8 — no overflow
risk anywhere. Meanwhile a float ratio can order two *genuinely equal* ratios
inconsistently, and an inconsistent comparator is not merely a wrong answer:
for `sort_by` in Rust and `std::sort` in C++ it is undefined behaviour. The
`random_ratio_collisions` test generates inputs where equal ratios are constant
for exactly this reason.

Ties need no tie-breaking rule. The swap arithmetic says both orders cost
exactly the same, so any consistent choice is optimal — which is what
`large_all_ties` (100 000 jobs, all ratio 2) checks.

### Advance the clock before charging

```
clock += d
total += r * clock      # correct
```

not

```
total += r * clock      # wrong
clock += d
```

A job pays for the time up to **and including** its own run, not up to the
moment it started. Getting this backwards under-counts every job by
`rate[i] * duration[i]` — and it fails 10 of the 11 tests, so it is at least
loud when wrong.

## Complexity

- **Time:** `O(n log n)`, dominated by the sort. The sweep is `O(n)`.
- **Space:** `O(n)` for the paired list.
- The clock reaches `n × 10 000 = 10^9` and the total about `5 × 10^17` — fine
  in signed 64-bit, far past 32.

Why brute force fails: trying every order is `n!`. Measured **30.6 seconds at
n = 11**, and each additional job multiplies that by the new `n`. At n = 15 it
is over a year; the bound is 100 000.

## Common wrong turns

- **Sorting by duration alone.** Fails 5 of the 11 tests.
- **Sorting by rate alone.** Fails 6 of the 11.
- **Charging before advancing the clock.** Fails 10 of the 11.
- **Floating-point ratio comparison.** Passes most tests, then produces an
  inconsistent order on ties.
- **Sorting by `rate - duration`, or any other combination that is not the
  ratio.** Easy to talk yourself into; the swap arithmetic is the check.
- **32-bit accumulator.** The total reaches 5 × 10^17.
- **Assuming the input arrives in a useful order.** `large_sorted_input` runs
  the same jobs best-first and worst-first and requires identical answers.

## Interview follow-ups

1. **Return the order, not just the cost.** Trivial, but it forces them to
   decide what to do about ties and about reporting original indices.
2. **Release times.** A job cannot start before time `r_i`. The greedy breaks —
   this becomes a genuinely hard scheduling problem, and recognising that the
   exchange argument no longer applies is the point.
3. **Deadlines instead of rates**, minimising maximum lateness. Different
   greedy (earliest deadline first), same style of exchange proof. A good
   comparison because the *proof technique* transfers even though the rule does
   not.
4. **Multiple workers.** Now it is NP-hard in general. Discuss list scheduling
   and approximation bounds.
5. **Prove the greedy.** Ask them to state the exchange argument out loud. This
   is the real test — a candidate who pattern-matched to "sort by ratio" will
   not be able to derive *why*, and this problem is in the set precisely to
   make that derivation the main event.
6. **Preemption allowed.** Jobs can be paused and resumed. Does the answer
   change? (For this objective, no — worth making them argue it.)
