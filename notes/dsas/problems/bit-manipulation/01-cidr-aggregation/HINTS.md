# Hints — CIDR Aggregation

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Given two 32-bit integers `start <= end`, find the longest bit prefix they
> share. Report that prefix's length, the value with all lower bits zeroed, and
> whether the resulting block is exactly the requested range.

The bound that decides the approach: a range can span all 4.3 billion
addresses. So anything that visits addresses one at a time — even to AND them
together — is finished before it starts.

Whatever you compute must come from `start` and `end` **directly**, in constant
time.

Now think about what a CIDR block actually is. A block with prefix length `p`
is the set of all addresses agreeing with the network address on the top `p`
bits, with the low `32 - p` bits free to be anything. So:

- A block **contains** an address exactly when they share those top `p` bits.
- For a block to contain both `start` and `end`, the top `p` bits of `start`
  and `end` must be identical.
- The *smallest* such block is the one with the **largest** `p` for which that
  holds.

So the whole problem reduces to one question: how many leading bits do `start`
and `end` have in common?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

To find where two numbers first differ, the natural tool is **XOR**. `a ^ b`
has a 1 in exactly the positions where `a` and `b` disagree, and 0 everywhere
they agree.

So `start ^ end` has all zeros in the shared leading prefix, and its **highest
set bit** marks the first position where the two disagree. Everything from that
bit downward must be left free.

That gives you the two numbers you need:

- Let `k` = the position of the highest set bit in `start ^ end`, counted so
  that `k` is the number of bit positions from that bit down to the bottom
  (in other words, the bit *length* of `start ^ end`).
- The block must leave `k` low bits free, so `prefix_len = 32 - k`.
- The network address is `start` with those `k` low bits cleared.

When `start == end` the XOR is 0, which has no set bits at all — `k = 0`,
`prefix_len = 32`, and the network is `start` itself. That case falls out of
the general formula rather than needing a branch, which is a good sign the
formulation is right.

For the `exact` flag: you now know the block's size is `2^k`. When is a block
exactly equal to the requested range?

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Bit manipulation: XOR to find the differing bits, then a mask to clear
them.**

```
diff = start ^ end
k = bit_length(diff)          # 0 when diff is 0
prefix_len = 32 - k
mask = (1 << k) - 1           # k low bits set
network = start & ~mask       # clear those bits
```

Most languages have a bit-length or leading-zeros primitive:

- Python: `int.bit_length()`
- Rust: `32 - x.leading_zeros()` on a `u32`, or `u32::BITS - x.leading_zeros()`

If you write the loop by hand (`while diff: diff >>= 1; k += 1`) it is still
constant time — at most 32 iterations — so that is fine too.

**The `exact` flag.** The block spans `2^k` addresses starting at `network`, so
it runs from `network` to `network + 2^k - 1`. It equals the requested range
exactly when both ends line up:

```
exact = (network == start) and (network + (1 << k) - 1 == end)
```

Equivalently, `exact` is true when `start` has its low `k` bits all zero and
`end` has them all one — which is another way of saying the range is
block-aligned.

**The overflow trap.** With `k = 32` (the whole address space), `1 << k` is
`4_294_967_296`, which does **not** fit in a 32-bit unsigned integer. In Rust,
computing `1u32 << 32` is a panic in debug and undefined shift behaviour in
general. Do the size arithmetic in a 64-bit type, or handle `k == 32`
separately. Python's unbounded integers hide this entirely — which is exactly
why the Rust version needs the care.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

```python
def smallest_cidr(ranges):
    out = []
    for start, end in ranges:
        diff = start ^ end
        k = diff.bit_length()            # 0 when start == end
        prefix_len = 32 - k
        network = start & ~((1 << k) - 1) if k else start
        size = 1 << k
        exact = network == start and network + size - 1 == end
        out.append((network, prefix_len, exact))
    return out
```

In Rust, keep the addresses in `u32` but do the size and comparison arithmetic
in `u64`:

```rust
let diff = start ^ end;                       // u32
let k = 32 - diff.leading_zeros();            // 0 when diff == 0
let prefix_len = 32 - k;
let network = if k == 32 { 0 } else { start & !((1u32 << k) - 1) };
let size = 1u64 << k;                         // u64: k can be 32
let exact = network == start && network as u64 + size - 1 == end as u64;
```

Points to be deliberate about:

- **`k == 32`** is the whole-address-space case. `1u32 << 32` is invalid, and
  `!((1u32 << 32) - 1)` is doubly so. Special-case the mask or widen the type.
- **`k == 0`** is the single-address case. `1 << 0` is 1, the mask is 0,
  `network == start`, and `exact` is true — all correct without a branch,
  provided your mask arithmetic handles a zero shift.
- **`network + size - 1` can reach `2^32`** before the `- 1`. Compute it in 64
  bits.
- **`prefix_len` is `32 - k`**, not `k`. Easy to invert by accident, and the
  single-address case (`/32`) versus the whole-space case (`/0`) are the two
  that catch it.
- The `exact` check needs **both** ends. Checking only that `network == start`
  wrongly reports `(5, 6)` as exact.

Complexity: a fixed handful of operations per range, so `O(len(ranges))` time
and `O(1)` extra space.

</details>
