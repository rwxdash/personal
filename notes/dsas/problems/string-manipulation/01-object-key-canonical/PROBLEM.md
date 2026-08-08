# Object Key Canonicalisation

**Difficulty:** Medium
**ID:** `string-manipulation-01-object-key-canonical`

## Scenario

Your object storage gateway accepts user-supplied keys like
`uploads//2024/../2025/./report.pdf` and must reduce them to one canonical
form before storing or looking anything up. Two keys that resolve to the same
object must produce byte-identical canonical forms, or the cache will serve the
wrong file.

The rules, applied to the key read left to right as a sequence of `/`-separated
segments:

| Segment | Meaning |
| --- | --- |
| `.` | the current directory — contributes nothing |
| `..` | go up one level — removes the previous surviving segment |
| empty | caused by `//`, a leading `/`, or a trailing `/` — contributes nothing |
| anything else | an ordinary name — kept |

The canonical form is the surviving segments joined by single `/`, with **no
leading and no trailing slash**. A key that resolves to nothing at all — every
segment cancelled out — canonicalises to the empty string.

**Security requirement.** A `..` that would go above the top level is a path
traversal attempt and must be rejected outright, not silently clamped. If at
any point a `..` has no surviving segment to remove, the whole key is invalid.

Note that only exactly `.` and exactly `..` are special. A segment like `...`
or `..a` or `.hidden` is an ordinary name.

## Task

Implement:

```python
canonical_key(raw: str) -> Optional[str]
```

Return the canonical form, or `None` if the key attempts to traverse above the
top level.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `raw` | `str` | `0 <= len(raw) <= 1_000_000` |

Characters are lowercase letters, digits, `.`, `-`, `_`, and `/`. The key may
be empty, may begin or end with `/`, and may contain any number of consecutive
slashes.

## Output

`Optional[str]` — the canonical key, or `None` for a rejected key.

## Required complexity

- **Time:** `O(n)`.
- **Space:** `O(n)`.

## Examples

### Example 1

```
raw = "uploads//2024/../2025/./report.pdf"
```

**Answer:** `"uploads/2025/report.pdf"`

Reading the segments in order: `uploads` is kept. The empty segment from `//`
contributes nothing. `2024` is kept. Then `..` removes `2024`. Then `2025` is
kept, `.` contributes nothing, and `report.pdf` is kept. The survivors are
`uploads`, `2025`, `report.pdf`.

### Example 2 — leading and trailing slashes

```
raw = "/logs/app/"
```

**Answer:** `"logs/app"`

The leading `/` produces an empty first segment and the trailing `/` produces
an empty last one; both contribute nothing. The canonical form never begins or
ends with a slash.

### Example 3 — traversal above the top level

```
raw = "a/../.."
```

**Answer:** `None`

`a` is kept, then the first `..` removes it, leaving nothing. The second `..`
now has nothing to remove, so the key is rejected. Note that it is rejected
even though the first part was perfectly valid — one bad `..` invalidates the
whole key.

### Example 4 — everything cancels

```
raw = "a/b/../.."
```

**Answer:** `""` (the empty string)

`a` and `b` are kept, then the two `..` segments remove `b` and then `a`. Zero
survivors is a valid result — the key resolves to the top level itself. This is
different from `None`, which means the key was rejected.

### Example 5 — dots that are not special

```
raw = "..hidden/.../a..b/."
```

**Answer:** `"..hidden/.../a..b"`

Only exactly `.` and exactly `..` are special. `..hidden`, `...` and `a..b` are
all ordinary names and are kept. The trailing `.` contributes nothing.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/string-manipulation/01-object-key-canonical/EDITORIAL.md`.
