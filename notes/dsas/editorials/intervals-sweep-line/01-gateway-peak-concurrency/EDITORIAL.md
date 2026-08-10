# Editorial — Gateway Peak Concurrency

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

Three constraints in the statement each eliminate an approach, and what
survives is the answer:

1. **Timestamps reach 10^9.** You cannot allocate an array over the timeline
   and bump a counter per millisecond — that is 10^9 buckets for at most
   200 000 sessions. The cost must scale with the number of *sessions*, not the
   length of time.
2. **`O(n log n)` budget with `n` unsorted intervals.** A log factor over
   unsorted input is an invitation to sort, and the only question is *what* to
   sort.
3. **Half-open intervals, stated explicitly and given their own worked
   example.** When a statement spends a paragraph on `[start, end)` semantics,
   that is where the test cases live.

The unlocking observation: **concurrency is a step function that changes only
where a session opens or closes.** There are at most `2n` such instants, and
between them nothing happens. So the continuous timeline collapses to `2n`
points, and the 10^9 range becomes irrelevant — which is exactly what
constraint 1 was pushing you toward.

Better still, the count only *rises* at a `start`. So the peak is always
attained at some session's start, and you never evaluate concurrency anywhere
else.

## The approach

Stop thinking about intervals as objects. Turn each into two **events**:

```
(start, +1)   a slot is claimed
(end,   -1)   a slot is released
```

Sort all `2n` events by time, sweep left to right with a running counter, and
record the counter's maximum along with the timestamp where it first occurred.

```
events = [(s, +1) for s, e in sessions] + [(e, -1) for s, e in sessions]
events.sort()

current = best = best_time = 0
for time, delta in events:
    current += delta
    if current > best:
        best, best_time = current, time
return (best, best_time)
```

Everything load-bearing is in exactly two places.

### 1. The tie-break at equal timestamps

**Releases must be processed before claims.** That *is* the half-open rule: a
session with `end == t` has already let go before anything claims a slot at
`t`. Get this backwards and `[(1,5), (5,9)]` reports a peak of 2 instead of 1.

The pleasant accident: sorting `(time, delta)` tuples lexicographically already
does it, because `-1 < +1`. No custom comparator needed. But *know* that you
are relying on it — in a language whose sort works differently, this becomes a
silent bug rather than a compile error.

Verified rather than asserted: mutating the reference so opens sort before
closes fails **8 of the 13** Rust tests.

### 2. Strict `>` when recording the peak

Events are visited in non-decreasing time order, so the first time the counter
reaches its maximum is the first time you record it. Using `>=` keeps
overwriting `best_time` with later instants that merely *tie* the peak, and you
end up reporting the last one.

This one is even better covered: mutating `>` to `>=` fails **11 of the 13**
tests.

### Zero-length sessions

A session with `start == end` emits `+1` and `-1` at the same timestamp. With
releases ordered first, the `-1` is processed *before* its own `+1`, so the
counter momentarily dips to −1 relative to where it was.

This is harmless, and it is worth understanding why rather than reflexively
filtering:

- A dip can never create a new maximum, so `best` is unaffected.
- The net change across that timestamp is zero, so every later event sees the
  correct count.
- In Rust, keep the counter **signed** so the transient dip cannot underflow. A
  `usize` counter panics in debug and wraps in release.

You *may* filter zero-length sessions up front; it is equivalent. But the
general code handles them, and a special case is one more thing to get wrong.

## Complexity

- **Time:** O(n log n) — building `2n` events is O(n), sorting dominates, the
  sweep is O(n).
- **Space:** O(n) for the event list.

Why the naive approaches fail:

- **Pairwise overlap comparison**, or evaluating concurrency at every candidate
  instant: O(n²). Measured 0.53 s at n = 4 000 with clean 4×-per-doubling
  scaling → roughly **22 minutes** at n = 200 000.
- **Bucketing the timeline** and incrementing per millisecond: O(max
  timestamp) = 10^9 operations and 10^9 memory, for an input of 200 000
  sessions. This is the approach the coordinate range exists to kill, and it is
  worth noticing that it is not merely slow — it is *unbounded in the wrong
  variable*. If the constraint had been `end <= 1000`, bucketing would be the
  right answer, and knowing which regime you are in is the real skill.

## Common wrong turns

- **Ordering opens before closes at equal timestamps.** The headline bug.
  Every back-to-back handover reads as an overlap.
- **`>=` instead of `>`.** Returns the latest peak instant instead of the
  earliest. Passes any test where the peak occurs exactly once, which is why
  the test file includes `[(0,5),(0,5),(100,105),(100,105)]`.
- **Unsigned counter.** Underflows on the zero-length transient dip.
- **Sorting the intervals instead of the events.** Sorting by start and
  tracking ends in a heap also works and is O(n log n) — but candidates who
  sort by start and then do something ad-hoc with ends usually end up
  rediscovering the sweep badly.
- **Returning a timestamp when the peak is 0.** Empty input and all-zero-length
  input must both give `(0, 0)`.
- **Assuming the input is sorted.** It is not, and `[(50,60), (10,20)]` returns
  `(1, 10)` — the earliest *time*, not the first session listed.
- **Materialising the timeline.** See above.
- **Forgetting that identical sessions all count.** Three copies of `(3,7)`
  give a peak of 3, not 1. There is no deduplication anywhere in this problem.

## Interview follow-ups

1. **Return the interval over which the peak holds**, not just its first
   instant. Requires noticing that the peak persists until the next event, so
   you need the following event's timestamp too.
2. **Which sessions were open at the peak?** Forces you to keep a live set
   rather than just a counter — and to think about whether that changes the
   complexity (it does, in the worst case).
3. **Minimum number of servers if each can hold `c` connections.** Just
   `ceil(peak / c)`, which is a nice check on whether the candidate sees that
   the peak is the only quantity that matters.
4. **Streaming.** Sessions arrive in start order but ends are unknown until
   they close. Now you need a min-heap of pending ends, which is the classic
   "meeting rooms" formulation and a natural bridge to the heap pattern.
5. **Weighted sessions.** Each carries a bandwidth cost rather than 1 slot;
   find peak total bandwidth. The sweep is unchanged with `+w` / `-w` deltas —
   good for confirming they understood the shape rather than the specific `±1`.
6. **Closed intervals instead of half-open.** Ask them to change exactly one
   line. If they can point straight at the sort key, they understood it.
7. **Why not a segment tree?** It works, it is O(n log n) too, and it is
   strictly more machinery. Being able to say "the sweep already gives me this
   for free" is the right instinct — reach for the tree only when queries
   become dynamic.
