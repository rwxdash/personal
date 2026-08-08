# Hints — Route Prefix Match

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Given a set of strings and a set of queries, report for each query the length
> of the longest set member that is a prefix of it.

The bound to look at is the *product*. Both lists reach 200 000 entries, and
comparing every path against every route means 4 × 10^10 string comparisons
before you even count characters. The required `O(R + P)` says each character
of the input may be touched a constant number of times, total — so the routes
must be preprocessed once into something that answers a query in time
proportional to the query alone.

Now think about what makes the naive approach wasteful. Comparing `/api/v2/users`
against `/api` and then against `/api/v2` re-reads `/api` twice. Comparing it
against 50 000 routes that all begin `/api` re-reads that prefix 50 000 times.

Routes that share a prefix should be *examined together*, not one after
another. What structure lets you walk the shared part once?

Two details worth fixing in your head now, because they are easy to get wrong
later: `0` and `-1` are different answers (the empty route matches everything
with length 0), and a route longer than the path can never match it.

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

The insight is that prefix relationships form a **tree**. If you lay the routes
out so that shared beginnings share a path from a common start point, then:

- every registered route corresponds to one position in that tree, and
- reading a query path character by character *walks down* that tree.

Once you are walking, matching becomes trivial. Feed the query's characters one
at a time. Each step either has somewhere to go — meaning some route continues
this way — or does not, meaning no registered route can match any further and
you can stop immediately.

And every time you pass through a position that happens to be the end of a
registered route, you have found a match. Keep the deepest one you pass, and
that is the longest match.

The cost of a query is then the number of characters you actually consume,
which stops as soon as the tree runs out. Nothing about the number of routes
enters into it.

The remaining work is deciding what to store at each position, and being
careful that "this position is the end of a registered route" is recorded
separately from "this position exists as part of some longer route".

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**A trie (prefix tree) over the routes.**

Build it once: insert every route by walking from the root, creating a child
node per character as needed, and marking the final node as **terminal** —
meaning "a registered route ends exactly here".

That terminal flag is the crux. A node existing means *some* route passes
through here; a node being terminal means *a route ends* here. Those are
different, and conflating them makes `/api/v2` alone report a match for
`/api/xyz`. Store the flag (or the route length, which is the same
information) explicitly.

Then answer each query by walking down from the root, consuming query
characters:

- Track the deepest terminal node passed so far, remembering its depth.
- **Check the root before consuming anything** — if the root is terminal, the
  empty route was registered and matches with length 0.
- Stop early the moment there is no child for the next character. No route can
  match beyond that point, so continuing is pointless.

The answer is the recorded depth, or `-1` if no terminal node was ever passed.

Duplicates need no handling: inserting the same route twice just re-marks the
same node terminal.

For the child links, either a hash map per node or a fixed array works. The
alphabet here is 27 symbols (`a-z` plus `/`), which is small enough that a
fixed-size array per node is viable — though with a million characters that is
27 million slots, so weigh it against a map.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

Represent the trie as parallel arrays rather than objects — it is faster and
avoids deep structures entirely.

```
# Build
children = [dict()]        # children[i] maps a character to a node id
terminal = [False]         # terminal[i]: does a registered route end here?

for route in routes:
    node = 0
    for ch in route:
        nxt = children[node].get(ch)
        if nxt is None:
            nxt = len(children)
            children.append(dict())
            terminal.append(False)
            children[node][ch] = nxt
        node = nxt
    terminal[node] = True          # duplicates simply re-mark the same node

# Query
result = []
for path in paths:
    node = 0
    best = 0 if terminal[0] else -1      # the empty route, checked BEFORE the loop
    for depth, ch in enumerate(path, start=1):
        nxt = children[node].get(ch)
        if nxt is None:
            break                        # no route can match any further
        node = nxt
        if terminal[node]:
            best = depth                 # depth == length of the route ending here
    result.append(best)
```

Points to be deliberate about:

- **Initialise `best` from the root's terminal flag**, before consuming any
  characters. Doing it inside the loop misses the empty route entirely, and
  makes `""` return `-1` instead of `0`.
- **`best` records a depth, not a node id.** Depth equals the length of the
  route that ends there, which is exactly what the problem asks for.
- **Break, do not continue**, when a child is missing. Continuing would read
  the rest of the query for nothing, and on adversarial input that is the
  difference between passing and timing out.
- **A route longer than the path** falls out naturally: the query simply runs
  out of characters before reaching that route's terminal node.
- **Empty `routes`** means only the root exists and every query returns `-1`.

Complexity: building visits each route character once, and each query consumes
at most its own length, so `O(R + P)` time and `O(R)` space. The number of
routes never appears in the query cost.

</details>
