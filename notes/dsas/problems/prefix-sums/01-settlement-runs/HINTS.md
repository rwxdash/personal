# Hints — Settlement Runs

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Count the contiguous subarrays whose sum is divisible by `m`, and report the
> length of the longest one.

Before choosing a technique, notice two things that together rule out a whole
family of approaches:

1. **The answer can be ~2 × 10^10 on an input of 200 000 elements.** Whenever
   the output is vastly larger than the input, you cannot visit qualifying runs
   one at a time — you have to produce many units of answer per unit of work.
2. **Deltas can be negative.** This matters more than it looks. Growing a run
   does not push its sum in a predictable direction: adding a refund can move
   the sum *down*. So there is no "extend until it qualifies, then shrink"
   structure available, and any approach built on the sum being monotone in the
   run's length is not merely slow here — it is *wrong*.

Given that, stop thinking about runs as things you grow and shrink. Think
instead about what a run's sum is *made of*, and whether the sum of a run can
be expressed in terms of quantities that do not depend on the run at all.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

Every contiguous run's sum is the difference of two **running totals**. If
`P[i]` is the sum of the first `i` deltas (so `P[0] = 0`), then the run
covering positions `i .. j-1` has sum `P[j] - P[i]`. Computing all `P` values
takes one pass.

That reframing converts the problem from "examine runs" into "examine pairs of
running totals", which is progress only if pairs are cheaper — and here they
are, because of the divisibility condition:

> `P[j] - P[i]` is divisible by `m` **exactly when** `P[i]` and `P[j]` leave
> the same remainder on division by `m`.

So a run is settleable iff its two endpoints' running totals are *congruent
mod m*. You are no longer looking for runs at all. You are looking for
**pairs of equal values** in a derived array of `n + 1` remainders — and
counting pairs of equal values is something you can do in one pass with a
tally, never enumerating the pairs themselves.

Two details that will bite if you skip them:

- The array of remainders has `n + 1` entries, not `n`. The extra one is the
  empty prefix `P[0] = 0`, and forgetting it loses every run that starts at
  index 0.
- Remainders of *negative* numbers need care — see the next level.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Prefix sums plus a hash map (or array) keyed by remainder mod `m`.**

Walk the `n + 1` prefix remainders once, maintaining two structures keyed by
remainder value:

- **A tally** — how many times each remainder has been seen so far. When you
  arrive at a remainder you have seen `k` times before, that closes exactly `k`
  new settleable runs (one per earlier occurrence). Add `k`, then increment the
  tally. This is what lets you count ~10^10 runs in 200 000 steps: you add
  them in batches, never one at a time.
- **A first-seen index** — the earliest prefix position at which each remainder
  appeared. The longest run ending at the current position is the current index
  minus that first-seen index. Record the first occurrence only; never
  overwrite it, or you will be measuring against a later, shorter start.

The trap that language semantics hide: **`%` does not mean the same thing
everywhere.** Python's `%` always returns a non-negative result for a positive
modulus, so `-1 % 3 == 2`. Rust's `%` is a remainder, not a modulus, and keeps
the sign of the dividend: `-1 % 3 == -1`. If you key your map on the raw
remainder in Rust, `-1` and `2` become different buckets even though they are
the same residue class, and every run spanning a negative running total is
missed. Normalise explicitly (`rem_euclid`, or `((x % m) + m) % m`).

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

Maintain a running total `total = 0`, a 64-bit `count = 0`, a `longest = 0`,
and two maps keyed by remainder: `seen_count` and `first_index`.

**Seed both with the empty prefix before the loop:** `seen_count[0] = 1` and
`first_index[0] = 0`. This single line is what makes runs starting at index 0
work, and omitting it is the most common bug in this problem.

Then for `j` from `1` to `n`:

1. `total += deltas[j-1]`.
2. `r = total mod m`, **normalised into `0 .. m-1`** (`rem_euclid` in Rust;
   Python's `%` already does this).
3. `count += seen_count.get(r, 0)` — every earlier prefix with this remainder
   closes one settleable run ending here.
4. `seen_count[r] += 1`.
5. If `r` is already in `first_index`, then `longest = max(longest, j -
   first_index[r])`. Otherwise `first_index[r] = j`.

Return `(count, longest)`.

Order matters in steps 3–4: read the tally *before* incrementing it, or a
prefix will be paired with itself and every remainder will be over-counted by
one. Likewise in step 5, only write `first_index[r]` when the remainder is new.

Complexity: one pass, O(1) amortised per step → **O(n)** time, and O(min(n, m))
space for the maps. Note the map is keyed by remainder, so it never holds more
than `min(n + 1, m)` entries even when `m` is 10^9.

Watch these:

- **64-bit count.** All-zero deltas give `n(n+1)/2 ≈ 2 × 10^10`, about 9× past
  a 32-bit ceiling.
- **The running total itself overflows 32 bits** long before that: 200 000
  deltas of 10^9 reach 2 × 10^14.
- **`m = 1`** makes every remainder 0, so every run qualifies — a good
  self-check that your batching arithmetic is right.
- **Empty ledger** returns `(0, 0)`; the loop simply never runs.

</details>
