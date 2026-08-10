# Manifest — bit-manipulation/01-cidr-aggregation

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `bit-manipulation-01-cidr-aggregation` |
| Pattern | Bit manipulation (XOR + bit length + masking) |
| Difficulty | Medium |
| Theme | Firewall rule CIDR aggregation |
| Generated | 2026-08-09 |
| Batch | E |

## Intended approach

`diff = start ^ end` is zero across the shared leading prefix, so its bit length
gives the number of free low bits. `prefix_len = 32 - free`, `network` is
`start` with those bits cleared, and `exact` requires both
`network == start` and `network + 2^free - 1 == end`.

Equivalent framing: the answer is the bitwise AND of every address in the range.

- **Optimal complexity:** O(len(ranges)) time, O(1) space beyond the output.
- **Naive it must beat:** iterate the range. A single range spans up to
  4.3 billion addresses, so this is not slow — it is impossible.

## Traps the tests target

| Trap | Test |
| --- | --- |
| `prefix_len` inverted (`free` instead of `32 - free`) | **verified**: mutation fails **all 11** Rust tests |
| `exact` checking only the start | `exactness`, `random_near_misses` — **verified**: fails 4 of 11 |
| `1u32 << 32` on the whole-address-space case | `large_full_space`, `large_widest_ranges` (Rust only; Python's unbounded ints hide it) |
| `size` or `network + size - 1` computed in 32 bits | same cases |
| `start == end` special-cased | `boundary_prefixes`, `large_singletons` |
| Assuming the block starts at `start` | `(5, 9)` → network 0; `(5, 6)` → network 4 |
| Iterating the range | `large_widest_ranges`, `large_full_space` |
| General correctness | 6 000 randomised ranges across three generators (uniform, exactly-aligned blocks, aligned-minus-one) vs a binary-string prefix oracle; identical LCG in Python and Rust |

`large_batch` checks 200 000 results against the *contract* rather than
recomputing them: each block must be self-aligned, must contain its range, and
must not still contain it at one bit longer.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py bit-manipulation/01-cidr-aggregation
=== python: bit-manipulation/01-cidr-aggregation ===
All tests passed.
=== rust: bit-manipulation/01-cidr-aggregation ===
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
stubs intact: python=True rust=True
RESULT bit-manipulation/01-cidr-aggregation: PASS
```

- All 25 hand-written expectations confirmed against the binary-string oracle.
- Two headline traps confirmed by deliberate mutation.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.

### Note on file location

This pack was initially written to a stale `notes/algos/` path after the repo
directory was renamed to `notes/dsas/`. Files were moved into the repo and the
verification above was re-run against the correct tree.
