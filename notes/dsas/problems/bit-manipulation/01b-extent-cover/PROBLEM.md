# Extent Cover

**Difficulty:** Medium
**ID:** `bit-manipulation-01b-extent-cover`

## Scenario

Your object store does not read arbitrary byte ranges from disk. It reads
**extents**. An extent is a `(offset, size)` pair, and the hardware only accepts
one if it is *naturally aligned*:

- `size` must be a power of two — 1, 2, 4, 8, 16, 32, …
- `offset` must be a multiple of `size`

So a 4-byte extent can start at 0, 4, 8, 12, … but never at 5. An 8-byte extent
can start at 0, 8, 16, … but never at 4.

A client asks for the bytes from `start` to `end` inclusive. You must serve that
request using whole extents, and you are **not allowed to read a single byte
outside the requested range** — the range is a permission boundary, so an extent
that spills over even by one byte is rejected.

Each extent costs one round trip, so you want as few of them as possible.

## What makes an extent legal

Given a request for bytes `start` to `end`, an extent `(offset, size)` is legal
when **all three** hold:

| # | Rule | Test |
| --- | --- | --- |
| 1 | Size is a power of two | `size` ∈ {1, 2, 4, 8, 16, …} |
| 2 | Offset is aligned to its own size | `offset % size == 0` |
| 3 | It stays inside the request | `offset >= start` and `offset + size - 1 <= end` |

Examples of **illegal** extents for the request `start = 4, end = 11`:

| Extent | Why it is rejected |
| --- | --- |
| `(4, 3)` | 3 is not a power of two |
| `(4, 8)` | 4 is not a multiple of 8 |
| `(8, 8)` | covers bytes 8–15, and 12–15 are outside the request |
| `(3, 1)` | byte 3 is outside the request |

## Task

Implement:

```python
extent_cover(ranges: List[Tuple[int, int]]) -> List[List[Tuple[int, int]]]
```

For each `(start, end)` request, return the **smallest possible list** of legal
extents whose covered bytes are exactly `start` through `end` — every requested
byte covered, no byte covered twice, nothing outside covered.

Return the extents **sorted by offset, ascending**.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `ranges` | `List[Tuple[int, int]]` | `0 <= len(ranges) <= 50_000` |
| `start`, `end` | `int` | `0 <= start <= end < 2^48` (that is `281_474_976_710_656`) |

## Output

`List[List[Tuple[int, int]]]` — one list of `(offset, size)` extents per
request, in the same order as the input.

## Required complexity

- **Time:** `O(E)` overall, where `E` is the total number of extents you return.
  No single request ever needs more than **94** extents.
- **Space:** `O(1)` beyond the output.

A single request can span the whole 2^48-byte address space, so anything that
visits byte positions — or builds a table indexed by them — is out of the
question.

## Examples

### Example 1 — the request is already one extent

```
start = 0, end = 7
```

**Answer:** `[(0, 8)]`

```
byte:  0  1  2  3  4  5  6  7
      [────────── 8 ──────────]
```

Checking the three rules for `(0, 8)`: 8 is a power of two ✓, `0 % 8 == 0` ✓,
and it covers bytes 0 through 7, exactly the request ✓.

One extent, and you obviously cannot do better than one.

### Example 2 — a request that fragments

```
start = 1, end = 6
```

**Answer:** `[(1, 1), (2, 2), (4, 2), (6, 1)]`

```
byte:  0   1   2   3   4   5   6   7
          [1] [── 2 ──] [── 2 ──] [1]
           ↑                       ↑
        offset 1                offset 6
```

Four extents. Each one checked against the rules:

| Extent | Power of two? | `offset % size` | Bytes covered | Inside 1–6? |
| --- | --- | --- | --- | --- |
| `(1, 1)` | 1 ✓ | `1 % 1 = 0` ✓ | 1 | ✓ |
| `(2, 2)` | 2 ✓ | `2 % 2 = 0` ✓ | 2–3 | ✓ |
| `(4, 2)` | 2 ✓ | `4 % 2 = 0` ✓ | 4–5 | ✓ |
| `(6, 1)` | 1 ✓ | `6 % 1 = 0` ✓ | 6 | ✓ |

Together they cover 1, 2, 3, 4, 5, 6 — the request exactly.

**Why not fewer?** The tempting move is a single 4-byte extent over bytes 2–5.
But a 4-byte extent must start at a multiple of 4, and 2 is not one. The only
4-byte extents nearby are `(0, 4)` covering 0–3 and `(4, 4)` covering 4–7, and
both spill outside the request. The same argument rules out anything larger, so
four is the minimum.

### Example 3 — both limits bite, one after the other

```
start = 4, end = 11
```

**Answer:** `[(4, 4), (8, 4)]`

```
byte:  4   5   6   7   8   9  10  11
      [───── 4 ─────][───── 4 ─────]
```

Eight bytes, two extents. Note that a single 8-byte extent would be legal by
size, but an 8-byte extent has to start at a multiple of 8, and the request
starts at 4. The nearest legal 8-byte extents are `(0, 8)` and `(8, 8)`, which
cover bytes 0–7 and 8–15 — each spills outside 4–11.

### Example 4 — a single byte

```
start = 5, end = 5
```

**Answer:** `[(5, 1)]`

One byte, so one 1-byte extent. Every offset is a multiple of 1, so this is
always legal.

### Example 5 — the whole address space

```
start = 0, end = 281474976710655
```

**Answer:** `[(0, 281474976710656)]`

That is `2^48` bytes starting at 0. A power of two ✓, and `0` is a multiple of
everything ✓. The entire space is one extent.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/bit-manipulation/01b-extent-cover/EDITORIAL.md`.
