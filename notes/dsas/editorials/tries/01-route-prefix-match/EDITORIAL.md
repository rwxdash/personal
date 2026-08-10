# Editorial — Route Prefix Match

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

The signal is the **product of the two bounds**. 200 000 routes and 200 000
paths means 4 × 10^10 pairs before you count a single character. And the
required `O(R + P)` — linear in the *total characters*, with the two counts
added rather than multiplied — says the route table must be preprocessed once
into something that answers a query in time proportional to that query alone.

Then look at what the naive approach wastes. Comparing `/api/v2/users` against
`/api`, then against `/api/v2`, re-reads `/api` twice. Against 50 000 routes
that all start `/api`, it re-reads that prefix 50 000 times.

Routes sharing a beginning should be examined **together**, once. That is the
whole idea, and it points at exactly one structure: prefix relationships form a
tree, where walking down from the root spells out a string.

## The approach

**A trie over the routes.**

Insert each route by walking from the root, creating a child per character as
needed, and marking the final node **terminal**.

Then answer a query by walking down from the root, one character at a time:

- Track the deepest terminal node passed, remembering its depth.
- Stop as soon as there is no child for the next character.

The answer is that depth, or `-1` if no terminal node was passed.

### Terminal is not the same as "exists"

This is the load-bearing distinction. A node *existing* means some route passes
through here. A node being *terminal* means a route **ends** here. If you treat
every node you can reach as a match, then registering `/api/v2` makes `/api/xyz`
report a match at `/api` — a route nobody registered.

Verified: removing the terminal flag and relying on node existence fails **10 of
the 11** tests.

### Check the root before consuming anything

If the empty string is a registered route, it is a prefix of everything and
matches with length `0`. That check has to happen *before* the character loop,
because the loop only ever inspects nodes it has moved into.

Initialising `best = -1` and letting the loop find everything misses the empty
route entirely — `0` becomes `-1`. That mutation fails 4 of the 11 tests,
including all four worked examples.

This is why the problem is careful to say `0` and `-1` are different answers.
They mean "the catch-all route matched" and "nothing matched", which are very
different things to a gateway.

### Break, do not continue

When there is no child for the next character, no registered route can match
any further, so stop. Reading the rest of the query changes no answer and costs
real time — on the `large_no_match` case (100 000 paths that miss on the first
character) it is the difference between touching 100 000 characters and touching
900 000.

### Depth, not node id

`best` records how deep you are, because depth equals the length of the route
ending there — which is exactly what the problem asks for. No extra storage
needed.

### Things that need no special handling

- **Duplicate routes** — inserting twice re-marks the same node terminal.
- **A route longer than the path** — the query runs out of characters before
  reaching that route's terminal node.
- **Empty `routes`** — only the root exists, so every query returns `-1`.

## Complexity

- **Time:** `O(R + P)`. Building touches each route character once; each query
  consumes at most its own length. The *number* of routes never enters the
  query cost.
- **Space:** `O(R)` for the trie.

Why the naive fails: checking every path against every route is quadratic in
the list lengths. Measured 0.518 s at 2 000 routes × 2 000 paths, scaling 4× per
doubling — roughly **90 minutes** at 200 000 × 200 000.

## Common wrong turns

- **Using node existence instead of a terminal flag.** The dominant bug.
- **Initialising `best = -1` and never checking the root.** Loses the empty
  route.
- **Continuing past a missing child** instead of breaking.
- **Sorting the routes and binary searching.** This does work — sort the routes,
  then for each path binary search for the longest matching prefix — but it is
  `O(R log R + P log R · L)` with a nasty constant, and the comparisons still
  re-read shared prefixes. Fine as a fallback, strictly worse here.
- **Building one trie per query**, or rebuilding between queries.
- **Storing the matched route string** rather than its length, then comparing
  strings to find the longest.
- **Assuming path-segment matching.** The statement says character matching,
  and Example 4 (`/api` matching `/apixyz`) exists to make that explicit.

## Interview follow-ups

1. **Return which route matched**, not just the length. Store the route's index
   at its terminal node — and decide what to do about duplicates, which is the
   interesting half of the question.
2. **Segment-aware matching**, where `/api` must not match `/apixyz`. Cleanest
   fix: insert a sentinel character at segment ends. Worth asking because the
   naive fix (checking the next character after the match) breaks on the
   exact-match case.
3. **Wildcards**, e.g. `/users/*/settings`. Now a query can branch, and the walk
   becomes a small search. This is the real router problem and a natural step up
   to a Hard rung.
4. **Memory.** A million characters with a hash map per node is heavy. Discuss
   fixed arrays (27 symbols here), compressed paths (radix tree), or a
   double-array trie.
5. **Routes change at runtime.** Insertion is easy; deletion needs reference
   counts on the nodes, which is a good detail to make them work out.
6. **Why not a hash set of all prefixes?** For each query, test every prefix of
   the path against a set. That is `O(P · L)` in the worst case and needs
   `O(R · L)` memory for all prefixes of all routes — a useful thing to cost out
   loud, since it sounds cheaper than it is.
