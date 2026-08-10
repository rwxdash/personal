# Hints — Byte-Budget Cache

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Maintain a key → value map with a total-size budget. Reads and writes both
> mark a key as most recently used. When the total exceeds the budget, remove
> least-recently-used keys until it fits. Report every read.

Before thinking about data structures, pin down the exact behaviour. Write these
five rules out, because each has a test aimed squarely at it:

1. **A successful GET marks the key most recently used.** A *missing* GET does
   nothing at all.
2. **Eviction is a loop.** One large insert can evict several entries.
3. **An oversized entry is rejected and evicts nothing.** The cache is left
   exactly as it was.
4. **Overwriting a key releases its old size**, then adds the new one — it does
   not accumulate.
5. **The just-inserted entry is never the one evicted.** (Worth convincing
   yourself: it is the most recently used, and since its size fits the budget,
   the loop always stops before reaching it.)

Now the structural problem. You need two things at once, and they pull in
different directions:

- **Look up a key instantly.** That is a hash map.
- **Find the least recently used entry instantly**, and move an arbitrary entry
  to "most recent" instantly. A hash map cannot do that.

Scanning for the least recent is `O(n)` per eviction, which is the quadratic
solution the bounds rule out. So what maintains an ordering under "move this
arbitrary element to the front" in constant time?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

An array or vector cannot do it: moving an element to the front shifts
everything else, which is `O(n)`.

What you want is a structure where an element can be **unlinked from its
current position and re-attached at one end** without touching anything else.
That is a **doubly linked list**: each node knows its neighbours, so removing it
is a matter of pointing those two neighbours at each other — constant time,
regardless of where in the list it sat.

Order the list by recency: most recently used at the front, least recently used
at the back. Then:

- **GET hit** → unlink that node, push it to the front.
- **PUT** → unlink if present, push to the front with the new value and size.
- **Evict** → remove the node at the back.

All constant time, *provided you can get from a key to its node directly*. That
is where the hash map comes back in: it maps each key to its node, so you never
search the list.

**The pairing is the whole design.** The hash map gives lookup; the linked list
gives ordering; neither works alone. Each entry lives in both structures at
once, and every operation has to keep them in step — a key removed from one
must be removed from the other.

Keep a running total of the stored sizes as well, updated on every insert,
overwrite, and eviction. Recomputing it by summing is another hidden `O(n)`.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Hash map + doubly linked list**, the standard LRU cache structure, with the
count-based capacity replaced by a byte total.

State:

- `map`: key → node
- a doubly linked list of nodes, each holding `key`, `value`, `size`
- `head` (most recent), `tail` (least recent)
- `total`: the sum of all stored sizes

**GET(key)**

```
if key not in map: record -1; done   # no recency change on a miss
node = map[key]
move node to front
record node.value
```

**PUT(key, value, size)**

```
if size > budget: done               # rejected; nothing else happens
if key in map:
    node = map[key]
    total -= node.size               # release the OLD size first
    node.value, node.size = value, size
    move node to front
else:
    node = new node
    map[key] = node
    push node to front
total += size
while total > budget:
    victim = tail
    total -= victim.size
    remove victim from the list
    delete map[victim.key]
```

Note the ordering inside the overwrite branch: subtract the old size *before*
adding the new one, or the total drifts upward permanently and the cache starts
evicting for no reason.

**Language notes.** Python's `collections.OrderedDict` is exactly this
structure with the linked list built in — `move_to_end(key)` and
`popitem(last=False)` give you both operations in `O(1)`. Using it is fine and
idiomatic.

Rust has no equivalent in the standard library, and `LinkedList` does not
support removing an arbitrary element by handle. The standard workaround is to
allocate nodes in a `Vec` and use **indices instead of pointers** — `prev` and
`next` become `Option<usize>` into that `Vec`. This sidesteps the borrow
checker entirely and is how such structures are normally written in Rust.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

The linked-list plumbing is where the bugs live, so write those two helpers
first and get them right in isolation:

```
unlink(i):
    p, n = node[i].prev, node[i].next
    if p is not None: node[p].next = n
    else:             head = n           # i was the front
    if n is not None: node[n].prev = p
    else:             tail = p           # i was the back
    node[i].prev = node[i].next = None

push_front(i):
    node[i].prev = None
    node[i].next = head
    if head is not None: node[head].prev = i
    head = i
    if tail is None: tail = i            # the list was empty
```

The four `if` branches in `unlink` are the four positions a node can occupy:
middle, front, back, and only-element. A single-element list hits *both*
`else` branches and correctly leaves `head` and `tail` empty.

Then the main loop over the operations, following the rules from level 3.

Things to be deliberate about:

- **Reject before touching anything.** The `size > budget` check must come
  first, and must return without evicting or modifying an existing entry.
- **Release the old size before adding the new one** on an overwrite.
- **`while`, not `if`**, when evicting.
- **Delete from both structures** when evicting. A stale map entry pointing at
  a recycled node produces answers that look almost right.
- **A GET miss changes nothing** — no recency update, no insertion.
- **Use 64-bit arithmetic.** 200 000 entries at 10^9 bytes reaches 2 × 10^14,
  and the budget itself goes to 10^12.
- **Only GETs produce output.** PUTs contribute nothing to the result list.

Complexity: every operation does constant work apart from evictions, and each
entry can be evicted at most once across the whole run — so the total is
`O(len(ops))` amortised, with `O(len(ops))` space.

</details>
