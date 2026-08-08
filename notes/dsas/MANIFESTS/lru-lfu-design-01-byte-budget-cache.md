# Manifest — lru-lfu-design/01-byte-budget-cache

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `lru-lfu-design-01-byte-budget-cache` |
| Pattern | LRU cache design (hash map + doubly linked list) |
| Difficulty | Medium |
| Theme | Object store response cache with a byte budget |
| Generated | 2026-08-09 |
| Batch | E |

## Intended approach

Hash map (key → node) paired with a doubly linked list ordered by recency,
plus a running `total` of stored sizes. GET hit moves to front; GET miss changes
nothing. PUT rejects when `size > budget`, otherwise releases any old size,
stores and moves to front, then evicts from the tail `while total > budget`.

- **Optimal complexity:** O(len(ops)) overall, amortised O(1) per operation —
  each entry is evicted at most once across the whole run.
- **Naive it must beat:** scan for the least recently used entry on each
  eviction, O(len(ops)^2) ≈ 4 × 10^10 at the bounds. That is what the test
  oracle does, so it only runs on small inputs.

## Design note

The byte budget replaces the usual entry-count capacity, which turns eviction
from a single step into a loop and adds the oversized-entry rejection rule.
Both are real cache-design concerns rather than decoration, and both are
mutation-tested below.

Operations are passed as `(kind, key, value, size)` tuples and results returned
as a list, rather than asking for a class. This keeps the contract identical
across Python and Rust and makes the behaviour fully testable.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Overwrite not releasing the old size (total drifts upward) | **verified**: mutation fails 6 of 13 Rust tests, including `large_overwrite_same_key` where the drift compounds over 200 000 writes |
| Oversized PUT not rejected (cache flushes itself for an entry that cannot fit) | `rejection`, `large_all_rejected` — **verified**: fails 4 of 13 |
| `if` instead of `while` when evicting | `sizes`, Example 2 — **verified**: fails 2 of 13 |
| GET miss updating recency or inserting | `recency_rules` |
| GET hit not refreshing recency | `recency_rules`, `large_hot_key` (a key read 100 000 times must never be evicted) |
| Entry removed from one structure but not the other | `random_against_brute`, `random_tight_budget` |
| Total recomputed by summing (hidden O(n)) | all `_test_large_*` cases |
| Value 0 confused with the `-1` miss marker | `edges` |
| Budget 0 | `rejection` → every PUT rejected |
| 32-bit overflow | `large_byte_range`, budgets to 10^12 and sizes to 10^9 |
| Quadratic LRU scan | `large_thrash` (200 000 ops), `large_no_eviction`, `large_hot_key` |
| General correctness | 800 randomised op sequences (400 general + 400 with budgets tight enough that nearly every PUT evicts) vs a Vec-scanning oracle; identical LCG in Python and Rust |

All 25 hand-written expectations were confirmed against the oracle before the
Rust mirror was written.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py lru-lfu-design/01-byte-budget-cache
=== python: lru-lfu-design/01-byte-budget-cache ===
All tests passed.
=== rust: lru-lfu-design/01-byte-budget-cache ===
running 13 tests
.............
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
stubs intact: python=True rust=True
RESULT lru-lfu-design/01-byte-budget-cache: PASS
```

- `reference.py` uses `OrderedDict`, which is this structure with the linked
  list built in.
- `reference.rs` uses a node arena: nodes in a `Vec` with `Option<usize>`
  indices for `prev`/`next`, since Rust's `LinkedList` cannot remove an
  arbitrary element by handle.
- All three headline traps confirmed by deliberate mutation.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
