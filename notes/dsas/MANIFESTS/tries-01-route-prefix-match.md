# Manifest — tries/01-route-prefix-match

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `tries-01-route-prefix-match` |
| Pattern | Trie (prefix tree) |
| Difficulty | Medium |
| Theme | API gateway route table replay |
| Generated | 2026-08-09 |
| Batch | D |

## Intended approach

Build a trie over the routes with a **terminal** flag per node, held in
parallel arrays (`children: Vec<HashMap<u8, usize>>`, `terminal: Vec<bool>`).
Answer each query by walking down from the root, recording the depth of the
deepest terminal node passed and breaking as soon as a child is missing.
Initialise `best` from the root's terminal flag before consuming any
characters, so a registered empty route yields 0 rather than −1.

- **Optimal complexity:** O(R + P) time in total character counts, O(R) space.
  The *number* of routes never enters the query cost.
- **Naive it must beat:** compare every path against every route. Measured
  0.518 s at 2 000 × 2 000, scaling 4× per doubling → ~90 minutes at
  200 000 × 200 000.
- **Correct but worse:** sort routes and binary search per query —
  O(R log R + P·L·log R) with repeated re-reading of shared prefixes.

## Traps the tests target

| Trap | Test |
| --- | --- |
| Node existence used instead of a terminal flag (reports unregistered prefixes) | **verified**: dropping the flag fails **10 of 11** Rust tests |
| Root terminal flag not checked before the character loop (empty route lost, 0 becomes −1) | `empty_route_vs_no_match`, `large_catch_all` — **verified**: mutation fails 4 of 11 |
| `0` vs `-1` conflated | `[""]` vs `["a"]` against `[""]` → `[0]` vs `[-1]` |
| Not breaking on a missing child | `large_no_match`: 100 000 paths missing at character 1 |
| Route longer than the path treated as a match | Example 3 → `-1`; `["/abc"]` vs `["/ab"]` |
| Duplicate routes | `["/a","/a","/a"]` → 2; `large_many_duplicates` (200 000 copies) |
| Insertion order affecting the answer | `["/a","/ab","/abc"]` and its reverse both → 4 |
| Segment matching assumed instead of character matching | Example 4: `/api` matches `/apixyz` |
| Empty route list / empty path list | → `[-1, -1]` and `[]` |
| Quadratic all-pairs scan | `large_shared_prefix` (50 000 routes sharing a 21-char prefix), `large_no_match` |
| General correctness | 400 randomised route/path sets over a 3-symbol alphabet with empty strings occurring naturally, vs an all-pairs oracle; identical LCG in Python and Rust |

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py tries/01-route-prefix-match
=== python: tries/01-route-prefix-match ===
All tests passed.
=== rust: tries/01-route-prefix-match ===
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
stubs intact: python=True rust=True
RESULT tries/01-route-prefix-match: PASS
```

- All 22 hand-written expectations independently confirmed against the
  all-pairs oracle before the Rust mirror was written.
- `reference.py` / `reference.rs` — both verified, both using flat parallel
  arrays rather than linked nodes.
- Both headline traps confirmed by deliberate mutation (recorded above).
- Quadratic blow-up confirmed by direct timing.
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
