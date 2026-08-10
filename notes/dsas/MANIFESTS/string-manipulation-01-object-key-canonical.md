# Manifest — string-manipulation/01-object-key-canonical

> Contains spoilers.

| Field | Value |
| --- | --- |
| Problem ID | `string-manipulation-01-object-key-canonical` |
| Pattern | String manipulation (split / stack / join) |
| Difficulty | Medium |
| Theme | Object storage key canonicalisation and path-traversal rejection |
| Generated | 2026-08-09 |
| Batch | D |

## Intended approach

Split on `/`, walk the segments with a stack: empty and `.` contribute nothing,
`..` pops or rejects the whole key, anything else pushes. Join the stack with
single `/`.

- **Optimal complexity:** O(n) time, O(n) space.
- **No slow-but-correct alternative to beat.** Unlike the other problems in the
  set, this one has no algorithmic blow-up to punish — the approach is nearly
  forced. The large cases therefore target *quadratic string building*
  (`large_no_quadratic_join`, 200 000 surviving segments), deep stacks
  (`large_deep_nesting`, 100 000 levels), and degenerate inputs
  (`large_all_slashes`, a million separators).

## Design note

This is deliberately a *case-analysis* problem rather than an algorithmic one.
The `string-manipulation` catalog entry covers exactly the skill of getting
every branch right, which is where real bugs live. Both headline traps are
genuine production failure modes:

- **Clamping `..` at the top level** instead of rejecting is the classic path
  traversal vulnerability.
- **Loose dot matching** (`startswith("..")`) silently mangles legitimate names
  like `..hidden` and `...`.

## Traps the tests target

| Trap | Test |
| --- | --- |
| `..` clamped at the top level instead of rejecting the key | `empty_vs_rejected`, `large_rejected_early` — **verified**: clamping fails 6 of 11 Rust tests |
| Loose dot matching (`startswith`, `contains`, dot counting) | `dots_that_are_not_special` — **verified**: `starts_with("..")` fails 3 of 11 |
| `Some("")` and `None` conflated | `"a/b/../.."` → `Some("")` vs `"a/../.."` → `None` |
| Rejection undone by later valid segments | `"a/../../b"` → `None` |
| Leading / trailing / doubled slashes special-cased instead of becoming empty segments | `edges`: `//a//b//`, `////a////`, `/file.txt/` |
| Leading or trailing slash in the output, or `"/"` for an empty result | every empty-result case |
| Empty input | `canonical_key("")` → `Some("")` |
| Quadratic output building | `large_no_quadratic_join` (200 000 segments), `large_all_dots` (500 000 dots) |
| Deep stack | `large_deep_nesting` (100 000 levels down, then all the way back up) |
| General correctness | 2 000 randomised keys built from a piece pool that includes `.`, `..`, `...`, `.hid` and empty, with random leading/trailing slashes, vs a character-level scanner oracle that never splits; identical LCG in Python and Rust |

All 39 hand-written expectations were confirmed against the oracle before the
Rust mirror was written.

## Verification record

**Date:** 2026-08-09

```
$ python3 verify.py string-manipulation/01-object-key-canonical
=== python: string-manipulation/01-object-key-canonical ===
All tests passed.
=== rust: string-manipulation/01-object-key-canonical ===
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
stubs intact: python=True rust=True
RESULT string-manipulation/01-object-key-canonical: PASS
```

- `reference.py` / `reference.rs` — both verified. The Rust version stacks
  `&str` slices borrowed from the input, so only the final join allocates.
- Both headline traps confirmed by deliberate mutation (recorded above).
- Candidate files confirmed restored to `raise NotImplementedError` / `todo!()`.
