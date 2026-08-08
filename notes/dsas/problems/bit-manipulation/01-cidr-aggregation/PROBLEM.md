# CIDR Aggregation

**Difficulty:** Medium
**ID:** `bit-manipulation-01-cidr-aggregation`

## Scenario

Your firewall accepts rules as CIDR blocks — an IPv4 network address plus a
prefix length, written `10.0.0.0/24`. A block with prefix length `p` covers
exactly the `2^(32-p)` addresses that share the same top `p` bits.

Your security team, however, hands you rules as **address ranges**: a first and
last address. You need to convert each range into the **smallest single CIDR
block that contains it**, so the rule can be installed.

A range does not always line up with a block. `10.0.0.5` to `10.0.0.9` needs a
block covering `10.0.0.0` to `10.0.0.15` — sixteen addresses to express a range
of five. The team wants to know when that happens, because an over-wide rule
opens more traffic than intended and needs review.

Addresses are given as plain 32-bit integers, not dotted strings.

## Task

Implement:

```python
smallest_cidr(ranges: List[Tuple[int, int]]) -> List[Tuple[int, int, bool]]
```

For each `(start, end)` range, return `(network, prefix_len, exact)`:

- `network` — the network address of the smallest CIDR block containing every
  address from `start` to `end` inclusive.
- `prefix_len` — that block's prefix length, `0` to `32`.
- `exact` — `True` if the block covers *exactly* the requested range and
  nothing more, `False` if it is wider.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `ranges` | `List[Tuple[int, int]]` | `0 <= len(ranges) <= 200_000` |
| `start`, `end` | `int` | `0 <= start <= end <= 4_294_967_295` (that is `2^32 - 1`) |

## Output

`List[Tuple[int, int, bool]]`, one entry per input range.

## Required complexity

- **Time:** `O(len(ranges))` — constant work per range.
- **Space:** `O(1)` beyond the output.

A single range can span all 4.3 billion addresses, so anything that walks the
addresses in a range is out of the question.

## Examples

### Example 1 — a range that is already a block

```
ranges = [(0, 255)]
```

**Answer:** `[(0, 24, True)]`

Addresses 0 through 255 are exactly the 256 addresses sharing the top 24 bits,
so this is `0.0.0.0/24` and it is exact.

### Example 2 — a range that needs a wider block

```
ranges = [(5, 9)]
```

**Answer:** `[(0, 28, False)]`

In binary, 5 is `...00101` and 9 is `...01001`. They agree on every bit except
the lowest four, so the smallest block containing both must leave those four
bits free: prefix length `32 - 4 = 28`, covering addresses 0 through 15.

The requested range was 5 addresses but the block covers 16, so `exact` is
`False`.

### Example 3 — a single address

```
ranges = [(42, 42)]
```

**Answer:** `[(42, 32, True)]`

Start and end agree on all 32 bits, so the block is a single address:
`/32`, exact.

### Example 4 — the whole address space

```
ranges = [(0, 4294967295)]
```

**Answer:** `[(0, 0, True)]`

The first and last addresses differ in the very top bit, so no bits can be
fixed at all. Prefix length 0 covers everything, and since the range *is*
everything, it is exact.

### Example 5 — crossing a boundary

```
ranges = [(255, 256)]
```

**Answer:** `[(0, 23, False)]`

Two adjacent addresses, but they sit either side of a power-of-two boundary:
255 is `011111111` and 256 is `100000000` in their low 9 bits. They differ in
bit 8, so the block must leave the low 9 bits free — prefix length 23, covering
0 through 511. A two-address range needs a 512-address block.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/bit-manipulation/01-cidr-aggregation/EDITORIAL.md`.
