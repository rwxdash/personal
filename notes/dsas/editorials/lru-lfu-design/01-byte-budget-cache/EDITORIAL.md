# Editorial — Byte-Budget Cache

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

This is a **design** problem rather than an algorithm-discovery one. There is no
clever insight to find; there is a structure to pick and a specification to
implement without dropping any of it.

The structural pressure comes from needing two things at once:

- **Look up a key instantly** — that is a hash map.
- **Find the least recently used entry instantly, and move an arbitrary entry to
  "most recent" instantly** — a hash map cannot do that, and neither can an
  array (moving an element to the front shifts everything else).

What supports "unlink from anywhere, re-attach at one end" in constant time is a
**doubly linked list**: a node knows its neighbours, so removing it just means
pointing those two at each other.

Neither structure works alone. The hash map maps key → node so you never search
the list; the list maintains recency order so you never search for the victim.
That pairing *is* the LRU cache, and the byte budget only changes the eviction
condition — a running total instead of a count.

## The approach

State: `map` (key → node), a doubly linked list with `head` = most recent and
`tail` = least recent, and `total` = sum of stored sizes.

**GET** — miss, append `-1` and change nothing. Hit, move the node to the front
and append its value.

**PUT** —

1. If `size > budget`, stop immediately. Nothing else happens.
2. If the key exists, subtract its old size, update value and size, move to
   front. Otherwise create a node, insert into the map, push to front.
3. `total += size`.
4. `while total > budget`, evict `tail`: subtract its size, remove it from the
   list **and** the map.

Keep `total` as a running value. Recomputing it by summing is a hidden `O(n)`
that turns the whole thing quadratic.

### The four rules that carry the tests

Each of these has a mutation test behind it:

**Eviction is a loop, not a step.** One 9-byte insert into a 10-byte cache
holding three 3-byte entries evicts all three. Changing `while` to `if` fails 2
of the 13 tests.

**Overwriting releases the old size first.** Subtract the old, then add the new.
Skipping the subtraction makes `total` drift permanently upward, so the cache
starts evicting for no reason — and it fails 6 of the 13 tests, including the
200 000-overwrite case where the drift compounds.

**An oversized entry is rejected and evicts nothing.** The `size > budget` check
must come before anything touches the cache. Removing it fails 4 of 13. This is
not pedantry: a cache that flushes itself to make room for an object that can
never fit is a real and painful production bug.

**A GET miss changes nothing.** No insertion, no recency update. It is easy to
write a lookup that accidentally touches the recency order on the way past.

### Why the new entry is never the victim

Worth convincing yourself rather than assuming. After a PUT, the new entry is at
the most-recent end, and `size <= budget` was already checked. Evicting from the
least-recent end, the total falls to at most `size` before the new entry is
reached — which fits. So the loop always stops first.

### Language notes

Python's `OrderedDict` **is** a hash map plus a doubly linked list.
`move_to_end(key)` and `popitem(last=False)` are both `O(1)`, so the reference
is essentially the specification written out. Using it is idiomatic, not
cheating.

Rust has no equivalent in the standard library, and `LinkedList` cannot remove
an arbitrary element by handle. The standard workaround, used in the reference,
is a **node arena**: allocate nodes in a `Vec` and make `prev`/`next` be
`Option<usize>` indices rather than pointers. This sidesteps the borrow checker
completely. The `unlink` helper's four branches are the four positions a node
can hold — middle, front, back, only-element — and a single-element list takes
both `else` arms, correctly emptying the list.

## Complexity

- **Time:** `O(len(ops))` overall. Every operation is constant work apart from
  evictions, and each entry is evicted at most once across the entire run, so
  eviction is amortised `O(1)`.
- **Space:** `O(len(ops))`.

Why the naive fails: scanning the cache for the least recently used entry is
`O(n)` per eviction, giving `O(len(ops)^2)` — about 4 × 10^10 steps at the
bounds. That is exactly what the test oracle does, which is why it is only run
on small inputs.

## Common wrong turns

- **`if` instead of `while`** when evicting.
- **Not releasing the old size** on an overwrite.
- **Evicting to make room for an oversized entry** that is then rejected.
- **A GET miss updating recency**, or inserting a placeholder.
- **Deleting from the list but not the map** (or vice versa). A stale map entry
  pointing at a dead node produces answers that look almost right.
- **Recomputing the total** by summing the entries.
- **Using an array or `VecDeque` for the ordering.** Moving to the front is
  `O(n)`.
- **Treating `-1` as a sentinel that a stored value cannot equal.** Values start
  at 0 and `-1` only ever means "absent" — but a cache that stores `-1`
  internally to mean "missing" will confuse the two.
- **32-bit arithmetic.** The total reaches 2 × 10^14 and the budget 10^12.

## Interview follow-ups

1. **Make it LFU instead of LRU.** Evict the least *frequently* used, breaking
   ties by recency. This is a genuine step up — you need a frequency index with
   `O(1)` increment, usually buckets of linked lists — and it is the natural
   Hard rung for this pattern.
2. **Add a TTL.** Entries expire at a wall-clock time. Where does expiry get
   checked — on read, on write, or by a background sweep? Each choice has a
   different failure mode.
3. **Thread safety.** A single lock around the whole cache serialises
   everything. Discuss sharding by key hash, and why the LRU list is the part
   that resists sharding.
4. **Admission policy.** Should a one-off large object be allowed to evict many
   hot small ones at all? Leads to TinyLFU and frequency-based admission.
5. **Approximate LRU.** Redis and others sample a few keys instead of
   maintaining exact order. What is traded away, and why is it usually fine?
6. **Report the hit rate**, or the number of evictions. Trivial to add, and a
   good check on whether the candidate kept the bookkeeping in one place.
