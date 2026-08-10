# Editorial — CIDR Aggregation

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

One constraint decides everything: a single range can span all 4.3 billion
addresses. So any approach that visits addresses — even just to AND them
together — is finished before it starts. The answer must come from `start` and
`end` **directly**, in constant time.

Then the definition does the rest of the work. A CIDR block with prefix length
`p` is the set of addresses agreeing with the network address on the top `p`
bits, with the low `32 - p` bits free. So:

- A block contains both `start` and `end` only if their top `p` bits match.
- The *smallest* such block is the one with the **largest** `p` for which that
  holds.

The whole problem is therefore: **how many leading bits do `start` and `end`
share?**

## The approach

To find where two numbers first differ, use **XOR**. `start ^ end` is zero
everywhere the two agree and has a 1 at every disagreement, so its **highest set
bit** marks the first differing position.

```
diff = start ^ end
free = bit_length(diff)          # 0 when diff == 0
prefix_len = 32 - free
network = start & ~((1 << free) - 1)
size = 1 << free
exact = network == start and network + size - 1 == end
```

`bit_length` is `int.bit_length()` in Python and `32 - x.leading_zeros()` in
Rust. A hand-written shift loop is fine too — at most 32 iterations is still
constant time.

**`start == end` needs no branch.** The XOR is 0, so `free` is 0, `prefix_len`
is 32, the mask is 0, and `network` is `start`. The general formula produces the
`/32` answer on its own, which is a good sign the formulation is right.

An equivalent view worth knowing: the answer is the bitwise **AND of every
address in the range**. Common prefix followed by zeros is exactly what that AND
produces. Same computation, different framing — and the AND framing is how this
problem usually appears in interviews.

### The `exact` flag needs both ends

The block spans `size` addresses from `network`, so it equals the requested
range only when `network == start` **and** `network + size - 1 == end`.

Checking only the start calls `(5, 6)` exact — it is not, since the block is
`(4, 7)`. That mutation fails 4 of the 11 tests.

Equivalently: exact means `start` has its low `free` bits all zero and `end` has
them all one.

### The overflow trap, and why only Rust hits it

With `free == 32` — the whole address space — `1u32 << 32` is invalid: a panic
in debug, and shift-overflow generally. `!((1u32 << 32) - 1)` is doubly so. Two
fixes: special-case `free == 32` for the mask, and compute `size` in `u64`.

```rust
let network = if free == 32 { 0 } else { start & !((1u32 << free) - 1) };
let size = 1u64 << free;                       // u64: free can be 32
let exact = network == start && network as u64 + size - 1 == end as u64;
```

Python's unbounded integers hide this completely, which is exactly what makes it
worth flagging — the Python solution works and the direct Rust translation
crashes on the `(0, 4294967295)` case.

`network + size - 1` also reaches `2^32` before the `- 1`, so that arithmetic
belongs in 64 bits too.

### `prefix_len` is `32 - free`, not `free`

Easy to invert by accident. The two ends catch it: a single address is `/32`
(where `free` is 0) and the whole space is `/0` (where `free` is 32). Inverting
it fails **all 11** tests, so at least it fails loudly.

## Complexity

- **Time:** `O(len(ranges))` — a fixed handful of operations each.
- **Space:** `O(1)` beyond the output.

## Common wrong turns

- **Iterating the range.** Impossible at 4.3 billion addresses.
- **Inverting `prefix_len`.**
- **Checking only the start for `exact`.**
- **`1u32 << 32`** on the whole-address-space case.
- **Computing `size` or `network + size - 1` in 32 bits.**
- **Special-casing `start == end`** — unnecessary, and usually a sign the mask
  arithmetic is wrong for a zero shift.
- **Looping bit by bit from the top** to find the common prefix. Correct and
  still constant time, just more code than a XOR and a bit-length.
- **Assuming the block starts at `start`.** It starts at `start` with the free
  bits cleared, which is different whenever the range is not aligned.

## Interview follow-ups

1. **Cover a range with the *fewest exact blocks*** instead of one wide block.
   This is the real CIDR-splitting problem: greedily emit the largest aligned
   block that fits at the current position and advance. A genuine step up and
   the natural Hard rung here.
2. **IPv6.** 128 bits, so no primitive integer holds an address. Discuss using
   two `u64`s or a `u128`, and what changes in the bit-length step.
3. **Merge overlapping or adjacent blocks** into the shortest equivalent list.
   Sorting plus a sweep, with the wrinkle that merging two blocks only yields a
   block when they are aligned siblings.
4. **Longest-prefix match against a routing table.** This is where a binary trie
   over address bits comes in — and it connects directly to the `tries` problem
   in this set.
5. **Parse and format dotted-quad strings.** Not algorithmic, but it is what the
   real function signature looks like, and byte-order mistakes live there.
6. **Why is the AND of a range equal to the common prefix?** Worth being able to
   explain, because it is the same fact from two directions and shows the
   candidate understands rather than recalls.
