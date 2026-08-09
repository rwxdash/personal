# Hints — Extent Cover

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Cut the integer interval `[start, end]` into the fewest possible pieces, where
> every piece must have a power-of-two length and must begin at a multiple of
> its own length.

Two things are worth staring at.

**The answer is tiny compared to the input.** A request can be 281 trillion
bytes wide, but the reply is never more than 94 extents. Those two numbers are
nowhere near each other, which tells you the extents have to be produced
directly from `start` and `end` — you can never touch a byte position, or an
array indexed by one.

**Legality of an extent depends on exactly two numbers.** Look again at the
three rules in the statement and ask, for a piece beginning at some position
`p`: what is the largest size you could possibly put there? Two separate things
limit it, and neither one knows about the other:

- something about `p` itself, and
- something about how much of the request is still unserved.

Work out what each of those limits is on its own. That is most of the problem.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

Build the answer left to right, one extent at a time, always placing the largest
legal extent you can at the current position. That greedy choice is optimal
here, and the reason is worth convincing yourself of: if you ever place a
*smaller* extent than you were allowed to, the next position you land on is
*less* aligned than the one you are on now, so you have not bought yourself
anything — you have strictly reduced your options for every step that follows.

That reduces the problem to the question Hint 1 left you with: at position `p`,
with `r` bytes still unserved, how big can this extent be?

Both limits are facts about the **binary representation** of a number:

- The alignment limit is a question about the low end of `p` — how many zero
  bits does it end with?
- The fit limit is a question about the high end of `r` — what is the largest
  power of two that does not exceed it?

Compute both, take the smaller, and you have the size. Then step forward and
repeat.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Bit manipulation: trailing zeros for alignment, bit length for fit, take the
minimum of the two exponents.**

Work in *exponents* (the `k` in `2^k`), not sizes, and convert to a size only at
the end. Mixing the two is the main way this goes wrong.

**The alignment limit.** `p` is a multiple of `2^k` exactly when its low `k`
bits are all zero. So the largest power of two dividing `p` is `2^t`, where `t`
is the number of trailing zero bits in `p`:

- Python: `(p & -p).bit_length() - 1`
- Rust: `p.trailing_zeros()`

**The fit limit.** The largest power of two that is `<= r` is `2^f`, where `f`
is one less than the bit length of `r`:

- Python: `r.bit_length() - 1`
- Rust: `63 - r.leading_zeros()` on a `u64`, or `u64::BITS - 1 - r.leading_zeros()`

Then `k = min(t, f)`, `size = 1 << k`, emit `(p, size)`, advance `p` by `size`,
and reduce `r` by `size`. Stop when `r` reaches zero.

**The `p == 0` case.** Every power of two divides zero, so there is no alignment
limit at all at position 0 — the fit limit alone decides. Check what your
language's trailing-zero primitive actually returns for 0 before you rely on it;
the two languages disagree, and one of them will hand you a value that is not
safe to shift by.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

For each request, keep a cursor `p` at the first unserved byte and a counter `r`
of how many bytes remain. While `r` is greater than zero:

1. Find `t`, the number of trailing zero bits in `p`. That is the largest
   exponent the alignment rule permits, because an extent of size `2^k` needs
   `p`'s low `k` bits to be zero. At `p == 0` treat `t` as unlimited.
2. Find `f`, one less than the bit length of `r`. That is the largest exponent
   that fits in what is left, because `2^f <= r < 2^(f+1)`.
3. Take `k = min(t, f)` and `size = 1 << k`. Emit the extent `(p, size)`.
4. Advance `p` by `size` and subtract `size` from `r`.

**Traced on `start = 1, end = 6`:**

| `p` | `r` | trailing zeros of `p` → `t` | bit length of `r` − 1 → `f` | `k = min` | emit |
| --- | --- | --- | --- | --- | --- |
| 1 | 6 | `1` = `…001`, so `t = 0` | `6` = `110`, 3 bits, `f = 2` | 0 | `(1, 1)` |
| 2 | 5 | `2` = `…010`, so `t = 1` | `5` = `101`, 3 bits, `f = 2` | 1 | `(2, 2)` |
| 4 | 3 | `4` = `…100`, so `t = 2` | `3` = `11`, 2 bits, `f = 1` | 1 | `(4, 2)` |
| 6 | 1 | `6` = `…110`, so `t = 1` | `1` = `1`, 1 bit, `f = 0` | 0 | `(6, 1)` |

Alignment is the binding limit on the first two rows, fit on the last two. That
shape — sizes growing while alignment allows, then shrinking as the tail runs
out — is what every answer looks like, and it is why the count is bounded by
roughly twice the number of address bits.

**Traced on `start = 4, end = 11`:**

| `p` | `r` | `t` | `f` | `k` | emit |
| --- | --- | --- | --- | --- | --- |
| 4 | 8 | `4` = `100`, `t = 2` | `8` = `1000`, 4 bits, `f = 3` | 2 | `(4, 4)` |
| 8 | 4 | `8` = `1000`, `t = 3` | `4` = `100`, 3 bits, `f = 2` | 2 | `(8, 4)` |

**Points to be deliberate about:**

- **Position 0 has no alignment limit.** In Rust, `0u64.trailing_zeros()` is
  `64`, and `1u64 << 64` panics — so take the `min` with `f` *before* you shift,
  never after. In Python, `0 & -0` is `0` and `(0).bit_length() - 1` is `-1`,
  which silently gives you a size of `2^-1`; guard the zero case explicitly.
- **`r` is a count of bytes, `t` and `f` are exponents.** They are never
  interchangeable. Convert with `1 << k` at one single point.
- **Advance by the size you emitted**, not by the size you were allowed. When
  `f` is the binding limit those differ.
- **Loop on `r > 0`**, not on `p <= end`. They are equivalent here, but `p` can
  land exactly on `end + 1`, and if `end` is the last address in the space that
  comparison is where an off-by-one hides.
- The last extent always ends exactly on `end`; if it does not, one of the two
  limits was computed wrong.

Complexity: at most 94 iterations per request, each a fixed handful of
operations, so `O(E)` total where `E` is the number of extents returned, and
`O(1)` space beyond the output.

</details>
