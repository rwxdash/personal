# Route Prefix Match

**Difficulty:** Medium
**ID:** `tries-01-route-prefix-match`

## Scenario

An API gateway holds a table of registered route prefixes. When a request
arrives, the gateway forwards it to the handler registered under the
**longest** prefix that matches the request path. This is how a route table
lets `/api/v2/users` be handled by a specific service while `/api` catches
everything else under it.

Matching is on raw characters, not on path segments: a registered route matches
a request path when the route is a literal prefix of the path. A route may also
be exactly equal to the path, which counts as a match.

You are building the offline analyser that replays a day of traffic against a
route table to see which routes are actually used.

## Task

Implement:

```python
longest_route_match(routes: List[str], paths: List[str]) -> List[int]
```

Return a list with one entry per path: the **length** of the longest registered
route that is a prefix of that path, or `-1` if no registered route matches.

Note that `0` and `-1` mean different things. If the empty string `""` is a
registered route, it is a prefix of every path, so those paths match with
length `0`. `-1` means nothing matched at all.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `routes` | `List[str]` | `0 <= len(routes) <= 200_000`; total characters across all routes `<= 1_000_000` |
| `paths` | `List[str]` | `0 <= len(paths) <= 200_000`; total characters across all paths `<= 1_000_000` |

Every string consists of lowercase letters `a-z` and the character `/`. Either
list may be empty, individual strings may be empty, and `routes` may contain
**duplicates**.

## Output

`List[int]` of the same length as `paths`.

## Required complexity

- **Time:** `O(R + P)` where `R` and `P` are the total character counts of
  `routes` and `paths`.
- **Space:** `O(R)`.

Checking every path against every route is `O(len(paths) · len(routes) ·
length)` — on the order of 4 × 10^10 character comparisons at these bounds.

## Examples

### Example 1

```
routes = ["/api", "/api/v2", "/health"]
paths  = ["/api/v2/users", "/api/v1/users", "/health", "/metrics"]
```

**Answer:** `[7, 4, 7, -1]`

- `/api/v2/users` — both `/api` (length 4) and `/api/v2` (length 7) are
  prefixes. The longest is 7.
- `/api/v1/users` — only `/api` matches, because `/api/v2` is not a prefix of
  it. Length 4.
- `/health` — the route `/health` is a prefix of the path *and* equal to it,
  which counts. Length 7.
- `/metrics` — nothing matches. `-1`.

### Example 2 — a catch-all route

```
routes = ["", "/api"]
paths  = ["/api/x", "/other", ""]
```

**Answer:** `[4, 0, 0]`

The empty route is a prefix of every string, so nothing ever fails to match.

- `/api/x` — both routes match; `/api` is longer, so 4.
- `/other` — only the empty route matches, giving **0**, not `-1`.
- `""` — the empty route matches the empty path, giving 0.

### Example 3 — a route longer than the path

```
routes = ["/api/v2/users"]
paths  = ["/api", "/api/v2/users", "/api/v2/users/42"]
```

**Answer:** `[-1, 13, 13]`

- `/api` — the route is *longer* than the path, so it cannot be a prefix of it.
  Prefix matching only runs one way. `-1`.
- `/api/v2/users` — exact match, length 13.
- `/api/v2/users/42` — the route is a proper prefix, length 13.

### Example 4 — character matching, not segment matching

```
routes = ["/api"]
paths  = ["/apixyz", "/ap"]
```

**Answer:** `[4, -1]`

`/api` is a literal character prefix of `/apixyz`, so it matches even though
the two are different path segments. And `/ap` is shorter than the route, so
nothing matches it.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/tries/01-route-prefix-match/EDITORIAL.md`.
