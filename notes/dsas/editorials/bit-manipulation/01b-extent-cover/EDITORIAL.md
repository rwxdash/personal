# Editorial — Extent Cover

**Pattern:** bit manipulation (trailing zeros + bit length)
**Difficulty:** Medium

## Recognising it

Two signals in the statement point the same way.

The first is the gap between the input size and the output size. A request can
be 281 trillion bytes wide, but the answer is capped at 94 extents. When the
answer is exponentially smaller than the range it describes, the answer cannot
be *found* by looking through the range — it has to be *computed* from the
endpoints. That rules out every scan, every table indexed by position, every DP
over the interval, before you have thought about the algorithm at all.

The second is the pair of rules on a legal extent. "Size is a power of two" and
"offset is a multiple of size" are both statements about the binary
representation of a number, not about arithmetic. `offset % size == 0` where
`size = 2^k` is precisely "the low `k` bits of `offset` are zero" — and once you
read it that way, the primitive you need is obvious.

## The approach

Walk left to right with a cursor `p` at the first unserved byte and a counter
`r` of bytes remaining. At each step place the largest legal extent, then
advance.

The size is capped by two independent things:

**Alignment cap.** An extent of size `2^k` at `p` requires `p % 2^k == 0`, i.e.
the low `k` bits of `p` are zero. So the largest exponent alignment permits is
`t`, the number of trailing zero bits in `p`.

```
p = 40 = 101000₂   ends in three zeros   →   t = 3
```

Cross-check it against ordinary arithmetic: `40 = 8 × 5`, and `8 = 2³`, so 40 is
a multiple of 8 but not of 16. The largest block that can begin at offset 40 is
8 bytes, which is what `t = 3` says.

**Fit cap.** The extent must not exceed `r`, so the largest exponent that fits
is `f` where `2^f <= r < 2^(f+1)` — one less than the bit length of `r`.

Then `k = min(t, f)`, `size = 1 << k`. Emit `(p, size)`, add `size` to `p`,
subtract `size` from `r`, repeat until `r` is zero.

```python
while remaining > 0:
    align = 64 if pos == 0 else (pos & -pos).bit_length() - 1
    fit = remaining.bit_length() - 1
    k = min(align, fit)
    size = 1 << k
    extents.append((pos, size))
    pos += size
    remaining -= size
```

### Why greedy is optimal

Not obvious, and worth being able to argue in an interview.

Let the greedy's first extent be `(start, 2^k)`, and let `C` be any minimal
cover. Every extent in `C` is an aligned power-of-two block, and `C` partitions
`[start, end]`.

Consider the region `R = [start, start + 2^k)`. Every block of `C` that meets
`R` must lie *inside* `R`: a block that properly contained `R` would have to
start at `start` (since `C` partitions from `start` onward) and be larger than
`2^k`, contradicting that `2^k` is the largest legal size at `start`. So the
blocks of `C` meeting `R` partition `R` exactly.

If `C` used two or more blocks there, replace all of them with the single block
`(start, 2^k)` — legal by construction — and you get a valid cover with strictly
fewer blocks, contradicting minimality. So `C` uses exactly one block on `R`,
which is the greedy's choice. Induct on the remainder.

### The shape of every answer

The invariant that explains the 94 bound: **sizes strictly increase while
alignment is the binding cap, then strictly decrease once fit takes over, and
never switch back.**

While `t < f` (alignment binds), you emit `2^t` and land on `p + 2^t`, whose
trailing-zero count is at least `t + 1` — so the next extent is strictly larger.
The moment `f <= t` and you emit `2^f`, the remainder drops to `r - 2^f < 2^f`,
so the next `f` is strictly smaller, while the new `p` has exactly `f` trailing
zeros. From then on fit binds forever and sizes strictly shrink.

Two monotone runs over 48 possible exponents, so at most ~96 extents; the true
worst case is 94, realised by `(1, 2^48 - 2)`.

## Complexity

- **Time:** `O(E)` where `E` is the total extents returned, at most 94 per
  request. Each iteration is a handful of instructions.
- **Space:** `O(1)` beyond the output.

The naive approach is a DP over positions: `dp[i]` = fewest extents covering
`[start, i)`. That is `O(range width × log width)`, correct but hopeless — at a
measured 2.3 million positions per second, one full-width request would take
about **four years**. The `large_wide_ranges` test has 5 000 such requests.

## Common wrong turns

**Taking `max(t, f)` instead of `min`.** Both caps have to hold at once. This
fails every single test, which is the good outcome.

**Shifting before taking the minimum.** Writing `min(1 << t, 1 << f)` instead of
`1 << min(t, f)` looks equivalent and is not: at `p == 0` there is no alignment
cap, `0u64.trailing_zeros()` is `64`, and `1u64 << 64` is a panic in debug and
undefined shift behaviour generally. Python hides it differently — `(0 & -0)` is
`0` and `(0).bit_length() - 1` is `-1`, giving a shift by a negative number.
Take the `min` on exponents, then shift once.

**Forgetting that position 0 has no alignment cap.** Every power of two divides
zero. In Rust `trailing_zeros()` returns 64 and the `min` with `f` handles it
for free; in Python `(p & -p).bit_length() - 1` gives `-1` and needs an explicit
guard.

**Mixing counts and exponents.** `r` is a byte count up to 2^48; `t` and `f` are
exponents in 0..=48. Comparing or assigning across the two is the single most
common way this goes wrong, and the symptom is nonsense sizes rather than a
clean crash.

**Advancing by the size you were allowed rather than the one you emitted.** When
fit is the binding cap those differ, and the cursor runs past the end.

**Checking `p <= end` instead of `r > 0`.** Equivalent here, but if the address
space were the full width of the integer type, `p` would wrap on the last step.
Counting down is the habit that survives.

## Interview follow-ups

- **Ordering.** If extents could be returned in any order, would the answer
  change? (No — the minimal cover is unique, so only the order is free.)
- **Relaxed boundary.** Suppose reading up to `S` bytes outside the range is
  permitted. Now what is the minimum? (This is the inverse of
  `01-cidr-aggregation`: you are rounding outward rather than filling inward,
  and the two problems meet in the middle.)
- **Costed extents.** If an extent costs `a + b × size` rather than 1, greedy on
  count is no longer right. What changes? (Large extents stop being free; with a
  per-byte term the objective is fixed since total bytes is fixed, so greedy
  survives — but with a per-extent cost that grows with size, it does not.)
- **Streaming.** The requests arrive one at a time and you must emit extents
  without buffering. Does anything change? (No — the algorithm is already
  single-pass with O(1) state, which is the point of the O(1) space bound.)
- **Wider addresses.** At 64-bit offsets, a request covering the whole space has
  size 2^64. Where does that break, and how do you express it? (Every size
  computation; you need either a 128-bit type, an exponent-only representation,
  or a documented special case.)
