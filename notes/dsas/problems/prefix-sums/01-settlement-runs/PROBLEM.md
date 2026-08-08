# Settlement Runs

**Difficulty:** Medium
**ID:** `prefix-sums-01-settlement-runs`

## Scenario

A payments ledger records a chronological stream of balance deltas for one
merchant account. A delta is positive for a capture and **negative for a
refund or chargeback**, so the running balance moves in both directions.

The settlement processor batches transactions in contiguous runs. A run is
**settleable** when its net delta — the sum of every delta in it — is an exact
multiple of the settlement unit `m` (including zero, which is a multiple of
everything). Only contiguous runs are eligible: the processor cannot skip a
transaction in the middle of a batch.

You are auditing the ledger and need two figures:

1. **How many settleable runs exist** in the ledger.
2. **How long the longest settleable run is.**

Runs are identified by position, so `deltas[3..7]` and `deltas[4..8]` are two
different runs even if they hold the same values.

## Task

Implement:

```python
settlement_runs(deltas: List[int], m: int) -> Tuple[int, int]
```

Return `(count, longest_length)` where:

- `count` is the number of contiguous runs whose sum is divisible by `m`,
- `longest_length` is the number of transactions in the longest such run, or
  `0` if there is no settleable run at all.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `deltas` | `List[int]` | `0 <= n <= 200_000`; each value in `-10^9 ..= 10^9` |
| `m` | `int` | `1 <= m <= 10^9` |

Deltas are frequently negative, and the running balance may go negative at any
point. A run's sum may be negative — a sum of `-6` is divisible by `3` just as
`6` is.

## Output

`Tuple[int, int]`. `count` can be as large as about `2 * 10^10`, so a 64-bit
accumulator is required (the Rust signature returns `u64`; Python integers are
unbounded).

## Required complexity

- **Time:** `O(n)` expected. `O(n log n)` is accepted.
- **Space:** `O(n)`.

At `n = 200_000` there are ~2 × 10^10 candidate runs, so they cannot be
enumerated even at O(1) each.

## Examples

### Example 1

```
deltas = [2, -2, 3], m = 3
```

**Answer:** `(3, 3)`

Every contiguous run and its net delta:

| Run | Values | Sum | Divisible by 3? |
| --- | --- | --- | --- |
| `0..0` | 2 | 2 | ❌ |
| `1..1` | −2 | −2 | ❌ |
| `2..2` | 3 | 3 | ✅ |
| `0..1` | 2, −2 | 0 | ✅ |
| `1..2` | −2, 3 | 1 | ❌ |
| `0..2` | 2, −2, 3 | 3 | ✅ |

Three settleable runs. The longest is `0..2`, holding all 3 transactions. Note
that `0..1` sums to exactly 0, which counts — zero is a multiple of 3.

### Example 2 — negatives

```
deltas = [-1, -2], m = 3
```

**Answer:** `(1, 2)`

`-1` alone is not divisible by 3, and `-2` alone is not. But `-1 + -2 = -3`,
which **is** divisible by 3. A negative sum is settleable exactly when its
absolute value is a multiple of `m`. One run, of length 2.

### Example 3 — everything settles

```
deltas = [5, -3, 7], m = 1
```

**Answer:** `(6, 3)`

With a settlement unit of 1, every integer is a multiple, so all
`3 * 4 / 2 = 6` contiguous runs qualify. The longest is the whole ledger.

### Example 4 — nothing settles

```
deltas = [1, 1], m = 5
```

**Answer:** `(0, 0)`

The possible sums are 1, 1 and 2, none of them a multiple of 5. When no run
settles, the longest length is reported as 0.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/prefix-sums/01-settlement-runs/EDITORIAL.md`.
