# Hints — Gateway Peak Concurrency

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Given `n` half-open intervals, find the maximum number that overlap at any
> single point, and the smallest coordinate where that maximum is attained.

Three facts to nail down before choosing a technique, because each one closes
off an approach that would otherwise look reasonable:

1. **Coordinates go up to 10^9.** You cannot allocate an array over the
   timeline and increment a counter per millisecond — that is 10^9 buckets for
   at most 200 000 sessions. The answer has to depend on the number of
   *sessions*, not the length of the timeline.
2. **The intervals are half-open, `[start, end)`.** A session ending at `t` and
   one starting at `t` do not overlap. This single rule decides several test
   cases, and it is not something you can bolt on afterwards — it has to be
   baked into how you order things.
3. **The input is unsorted.** Whatever order the log gave you is meaningless,
   so you are free to reorder — and given the `O(n log n)` budget, you are
   being *invited* to sort.

Now the key question: concurrency changes only at certain instants. Which ones?
And in particular, can the peak ever occur at an instant that is not one of the
`start` values?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

Concurrency is a step function over time. It is constant everywhere except at
the finitely many instants where a session opens or closes — and there are at
most `2n` of those. So the whole continuous timeline collapses to `2n`
interesting points, which is what makes the 10^9 coordinate range irrelevant.

Better still, the count only ever **increases** at a `start`. It decreases at
an `end`. So the peak is always attained at some session's `start`, and you
never need to evaluate the function anywhere else.

That gives the shape of the algorithm: forget intervals as objects, and turn
each one into two **events** — "a slot is claimed here" and "a slot is released
here". Sort all `2n` events by time, then walk them in order maintaining a
single running counter. Sweep the timeline once, left to right, and watch the
counter.

The entire difficulty then compresses into one question: **when an opening
event and a closing event share a timestamp, which do you process first?**
Think about what half-open intervals demand, and check your answer against
`[(1,5), (5,9)]`, which must give a peak of 1 rather than 2.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Sweep line over sorted events.**

Build `2n` events: `(start, +1)` and `(end, -1)` for each session. Sort by
timestamp — and **at equal timestamps, process the `-1` releases before the
`+1` claims.**

That tie-break *is* the half-open semantics. At instant `t`, a session with
`end == t` has already let go, so its release must land before any claim at the
same instant. Order them the other way and `[(1,5), (5,9)]` reports 2 instead
of 1. Concretely, sort by the key `(time, delta)` — since `-1 < +1`, the
natural ordering of the pair already puts releases first, which is a pleasant
accident worth exploiting rather than writing a custom comparator.

Then sweep: run the counter through the events in order, and every time it
reaches a value strictly greater than the best seen so far, record both the new
best and the current timestamp. Using a *strict* comparison is what gives you
the **earliest** instant for free — later ties never overwrite the recorded
time.

Two subtleties to be deliberate about:

- **Zero-length sessions** produce a `+1` and a `-1` at the *same* timestamp.
  With releases ordered first, the `-1` is processed before its own `+1`, so
  the counter momentarily dips. That is harmless for the maximum (a dip never
  creates a new peak), and the net effect at the end of that timestamp is
  correctly zero. You do not need to filter them — but you should convince
  yourself of that rather than assume it.
- **A peak of 0** — no sessions, or nothing but zero-length ones — must return
  `(0, 0)`, not `(0, <some timestamp>)`.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

```
events = []
for (start, end) in sessions:
    events.append((start, +1))
    events.append((end, -1))

events.sort()          # (time, delta); -1 sorts before +1 at equal time

current = 0
best = 0
best_time = 0
for (time, delta) in events:
    current += delta
    if current > best:          # STRICT: keeps the earliest peak instant
        best = current
        best_time = time

return (best, best_time)
```

That is the whole algorithm. Everything load-bearing is in two places: the sort
key, and the strictness of the comparison.

**Why `events.sort()` alone gets the tie-break right.** Tuples compare
lexicographically, so equal timestamps fall through to comparing the deltas,
and `-1 < +1` puts every release ahead of every claim at that instant. No
custom comparator is needed. If your language sorts differently, be explicit —
do not leave this to chance, because it is the difference between passing and
failing Example 2.

**Why strict `>` yields the earliest time.** Events are visited in
non-decreasing time order, so the first moment the counter reaches its maximum
is the first time you record it. A `>=` would keep overwriting `best_time` with
later instants that merely *tie* the peak, and would return the last one
instead of the first.

**Why the peak is always at a `start`.** The counter only rises on `+1` events,
so any new maximum is recorded at a claim's timestamp. You never need to
evaluate concurrency between events.

Complexity: building events is O(n), sorting `2n` of them is O(n log n), the
sweep is O(n). Space O(n) for the event list.

Edge cases to check before you run:

- Empty input → `(0, 0)`; the loop never executes and the initial values are
  already correct.
- All sessions zero-length → peak stays 0 → `(0, 0)`.
- One session → `(1, start)`.
- Duplicate identical sessions → they all count, so `k` copies give peak `k`.

</details>
