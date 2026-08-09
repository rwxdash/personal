# Manifest — bit-manipulation/01b-extent-cover

**Contains spoilers.** Metadata and verification record only.

| Field | Value |
| --- | --- |
| Problem ID | `bit-manipulation-01b-extent-cover` |
| Pattern | Bit manipulation |
| Difficulty | Medium |
| Rung | `01` — a **second Medium** alongside `01-cidr-aggregation`, not a step up |
| Theme | Object store aligned-extent reads |
| Generated | 2026-08-09 |

## Why a sibling rather than `02`

Requested as extra reps at the same level after `01-cidr-aggregation` proved
hard. It is deliberately the *inverse* operation — `01` rounds a range **outward**
to the smallest containing block, this one fills a range **inward** with the
fewest contained blocks — so the alignment intuition transfers directly while
the algorithm is new. Filed as `01b` so the `02` (Hard) slot stays free.

## Intended approach

Greedy left to right. Keep a cursor `p` at the first unserved byte and a
remaining count `r`. The extent size at `p` is capped independently by
alignment (`t` = trailing zeros of `p`; unbounded at `p == 0`) and by fit
(`f` = bit length of `r`, minus one). Emit `2^min(t, f)`, advance, repeat.

Optimality by exchange: every block of a minimal cover meeting the region
`[start, start + 2^k)` must nest inside it, so using two or more there could be
replaced by the single greedy block — contradiction. The first block is
therefore forced, and induction gives the rest. The minimal cover is unique.

Sizes strictly increase while alignment binds, then strictly decrease once fit
binds, and never switch back — two monotone runs over 48 exponents, hence the
94 bound (realised by `(1, 2^48 - 2)`).

**Complexity:** `O(E)` time where `E` is total extents returned (≤ 94 per
request), `O(1)` space beyond the output.

**Naive it must beat:** DP over positions, `dp[i]` = fewest extents covering
`[start, i)`, at `O(width × log width)`. Measured at 2.28 M positions/s, so a
single full-width request is ≈ **4 years**. `large_wide_ranges` has 5 000 of
them.

## Trap / edge cases

| Trap | Test | Mutation result |
| --- | --- | --- |
| `max(t, f)` instead of `min` | all | **verified**: fails **11 of 11** |
| Fit cap ignored (alignment only) | all | **verified**: fails **11 of 11** |
| Fit off by one (`64 -` not `63 - leading_zeros`) | all | **verified**: fails **11 of 11** |
| Advance by the allowed size, not the emitted one | all | **verified**: fails **10 of 11** |
| Alignment cap ignored (fit only) | `which_limit_binds`, `random_*` | **verified**: fails **8 of 11** |
| Shift before the min → `1u64 << 64` at position 0 | `zero_start`, `examples`, `whole_blocks`, `large_*` | **verified**: fails **4 of 11** (debug panic) |
| Per-byte / smallest-extent-every-time | `examples` | **verified**: fails in 0.00 s with a readable diff, before the large tests can hang |
| Position 0 has no alignment cap | `zero_start` (5 cases incl. the full 2^48 space) | |
| Single-byte requests | `single_byte`, `examples` | |
| Request already one aligned block | `whole_blocks` (every `k` in 0..=48) | |
| Empty input | `edges` | |
| Worst-case fragmentation | `worst_case_shape` — asserts exactly 94 extents | |
| General correctness | 400 small ranges vs a BFS shortest-path minimum (assumes nothing about which extent to pick); 2 000 full-width ranges vs the contract plus a no-mergeable-pair minimality check | |

Greedy optimality was additionally checked **exhaustively**: every range
`(s, e)` with `0 <= s <= e < 96` against the BFS oracle — all minimal.

`large_wide_ranges` and `large_batch` assert a total-extent checksum (229 651
and 450 115) rather than full lists. Both languages produce the same number,
which also proves the two LCGs generate byte-identical inputs.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py bit-manipulation/01b-extent-cover
=== python: bit-manipulation/01b-extent-cover ===
All tests passed.
=== rust: bit-manipulation/01b-extent-cover ===
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
=== rust doc-tests (as shipped): bit-manipulation/01b-extent-cover ===
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
stubs intact: python=True rust=True
RESULT bit-manipulation/01b-extent-cover: PASS
```

Reference spliced ahead of the untouched test block in both languages; candidate
files confirmed still at `raise NotImplementedError` / `todo!()` afterwards.
Mutation results in the table above were produced the same way, against
`reference.rs` with one line changed.
