# Editorial — Sharded Audit Log

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

The recognition here is driven entirely by the **relationship between the
bounds**, not by the shape of the question:

| Quantity | Bound |
| --- | --- |
| total events `N` | 2 000 000 |
| shards `k` | 100 000 |
| `offset` | 200 000 |

`offset` sits an order of magnitude below `N`. When a problem hands you far
more data than the answer could possibly depend on, it is telling you the cost
must scale with the *query*, not the *corpus*. That single observation kills
"concatenate everything and sort": `O(N log N)` time and `O(N)` memory to
answer a question about the first 10%.

The second signal is the pre-existing structure. **Each shard is already
sorted**, and sorting the concatenation throws all of that away. Whenever you
find yourself destroying structure the input handed you for free, back up.

What does per-shard sortedness buy? This:

> The globally earliest remaining event is always at the **front** of some
> shard. It can never be buried mid-shard, because everything before it in that
> shard is smaller or equal and therefore comes first.

So at any moment there are only `k` candidates, one per shard, and the merge is
just "repeatedly take the smallest candidate, advance that shard".

## The approach

The naive way to find the smallest of `k` candidates is to scan all of them:
`O(k)` per step, `O(offset · k)` overall — measured at 1.83 s for k = 8 000 /
offset = 2 000, scaling linearly in both, which extrapolates to roughly **38
minutes** at the full bounds.

But between consecutive steps only **one** candidate changed. The other `k - 1`
are exactly as they were, and rescanning re-derives what you already knew. What
you need is a structure that holds `k` items, yields the minimum on demand, and
replaces one item in logarithmic time.

**A min-heap of shard heads.**

```
heap = [(shard[0], i, 0) for i, shard in enumerate(shards) if shard]
heapify(heap)                                   # O(k), beats k pushes

for _ in range(offset + 1):
    if not heap: return None
    value, shard_idx, pos = heap.pop_min()
    if pos + 1 < len(shards[shard_idx]):
        heap.push((shards[shard_idx][pos + 1], shard_idx, pos + 1))

return (value, shard_idx)
```

The heap never exceeds `k` entries — one live candidate per shard — which is
what keeps memory at `O(k)` instead of `O(N)`.

### The tie rule must live in the heap ordering

Events sharing a timestamp must emerge in ascending shard order, so the heap
compares `(timestamp, shard_index)` — never the timestamp alone. If you order
by timestamp only, ties resolve according to the heap's internal array layout,
which is arbitrary and will not match the specification.

Storing `(value, shard_idx, pos)` as a tuple gets this for free in Python, since
tuples compare lexicographically. `pos` never participates in a tie: two entries
from the same shard are never in the heap simultaneously.

In Rust, `BinaryHeap` is a **max**-heap, so wrap in `Reverse(...)`. Make the
ordering explicit rather than relying on struct field order by luck.

Mutating the reference so ties break by *highest* shard index fails 4 of the 10
Rust tests.

**The tie rule is not round-robin.** A shard holding three events at the same
timestamp emits all three before a higher-numbered shard emits one — because
after each pop you push that same shard's next element, which still compares
smaller on the tie-break. `[[5,5],[5],[5]]` yields `(5,0), (5,0), (5,1), (5,2)`,
and Example 2 exists specifically to make that visible before you code.

### Return the last thing popped

The loop pops `offset + 1` times and the answer is the final pop, **not** the
heap's minimum afterwards (which is the *next* event). This is the easiest bug
to write and the easiest to catch: mutating the reference to return the heap
minimum fails **all 10** tests.

### One emptiness check covers three cases

Checking `if not heap` at the top of the loop handles: no shards at all, only
empty shards, and an offset past the end. A special case for any of them
individually is a case you can get wrong.

## Complexity

- **Time:** `O(k + offset · log k)`. Heapifying is `O(k)`; each step is one pop
  and at most one push.
- **Space:** `O(k)`.

Independent of `N` beyond the initial scan for non-empty shards — which is the
whole point.

## Common wrong turns

- **Concatenate and sort.** Correct, `O(N log N)` time and `O(N)` memory, and
  it ignores both the per-shard sortedness and the "without materialising"
  requirement.
- **Rescanning all `k` heads each step.** `O(offset · k)` ≈ 2 × 10^10.
- **Ordering the heap by timestamp only.** Ties resolve by heap layout.
- **Returning the heap minimum after the loop** instead of the last pop.
- **Pushing all of a shard's elements up front.** Turns the heap into `O(N)`
  memory and defeats the purpose.
- **Round-robin tie handling.** See above.
- **Forgetting to skip empty shards when seeding**, then indexing `shard[0]`.
- **Reporting the wrong shard index after filtering empties.** The index must
  be the position in the original list — `[[], [], [8]]` returns shard 2, not 0.
- **32-bit timestamps.** They reach 10^18.

## Interview follow-ups

1. **Return a whole page, not one event.** Pop `page_size` more times. Then ask
   for page *`p`* directly without walking from 0 — which is genuinely harder
   and leads to a binary-search-on-timestamp discussion (count how many events
   across all shards are `<= t`, in `O(k log N)` with per-shard binary search).
   That variant is `O(k log N log T)` and beats the heap when `offset` is huge.
2. **Streaming shards.** The shards are network cursors, not arrays; you can
   only advance forward. Does the algorithm still work? (Yes — it only ever
   advances. Worth making them notice.)
3. **Merge all `N` events, not just the first `offset`.** Now `O(N log k)`, and
   the comparison against `O(N log N)` sorting is the classic reason k-way
   merge exists — external sorting.
4. **Duplicate suppression.** Emit each distinct timestamp once. Where does the
   check go, and why is it not enough to compare against the previous pop?
5. **Bound the heap to `m` entries.** What if `k` exceeds memory? Leads to
   tournament trees and multi-pass merging.
6. **What if shards were not individually sorted?** The whole approach
   collapses. Asking them to state *why* separates understanding from recall.
