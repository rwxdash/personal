# Hints — Sharded Audit Log

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Given `k` sorted lists, find the element at position `offset` of their merge,
> ordered by `(value, list_index)`.

The constraint doing the real work is the relationship between the bounds:

- Total events `N` can be **2 000 000**.
- `offset` is at most **200 000**.
- Shards `k` can be **100 000**.

`offset` is an order of magnitude below `N`, and the statement says so
explicitly ("your cost should scale with how far you page, not with how much
data exists"). That kills the obvious approach — concatenate everything and
sort — not because it is wrong, but because it does `O(N log N)` work and
`O(N)` allocation to answer a question about the first 10% of the data.

So the target is: **touch roughly `offset` events, not `N` of them.**

Now, the other thing to notice. Each shard is already sorted. That is an
enormous amount of pre-existing structure, and sorting the concatenation throws
all of it away. What can you conclude about the globally-first event, given
only that each shard is individually sorted?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

The globally earliest remaining event must be at the **front of some shard** —
it cannot be buried in the middle of one, because everything before it in that
shard is smaller or equal and therefore comes first.

So the whole merge reduces to a repeated question:

> Among the current front elements of the `k` shards, which is smallest?
> Emit it, advance that shard's front, repeat.

Do that `offset + 1` times and the last emitted event is your answer.

The naive way to answer "which front is smallest" is to scan all `k` fronts,
which costs `O(k)` per step and `O(offset · k)` overall — 2 × 10^10 at the
stated bounds. Too slow.

But look at what happens between two consecutive steps: only **one** shard's
front changed. The other `k - 1` candidates are exactly as they were. Rescanning
all of them re-derives facts you already had.

You need a structure that holds `k` candidates, hands you the minimum on
demand, and supports replacing one element cheaply — all in logarithmic time
rather than linear.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**A min-heap of shard heads — the k-way merge.**

Seed the heap with one entry per **non-empty** shard, then repeatedly pop the
minimum and push that shard's next element (if it has one). After `offset + 1`
pops, the last popped entry is the answer.

The heap holds at most `k` entries at any moment — one per live shard — which
is what keeps memory at `O(k)` rather than `O(N)`.

**The tie rule has to live inside the heap ordering.** Events sharing a
timestamp must come out in shard-index order, so the heap must compare on the
pair `(timestamp, shard_index)`, not on the timestamp alone. Two consequences
worth being deliberate about:

- If your heap compares only timestamps, ties break by whatever the heap's
  internal layout happens to produce. That is not deterministic in any useful
  sense, and it will disagree with the expected output.
- The tie rule is *not* round-robin. A shard that holds three events at the
  same timestamp emits all three before a higher-numbered shard emits its
  first — because after each pop you push the *same* shard's next element,
  which still compares smaller on the tie-break.

Each heap entry needs enough state to find the shard's next element: the value,
the shard index, and the position within that shard.

Also: seed the heap only with non-empty shards, and be careful that an empty
`shards` list, or a list of only-empty shards, is handled by the same code
path rather than by a special case that you then get wrong.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

```
heap = []
for i, shard in enumerate(shards):
    if shard:                                    # skip empties
        heap.push((shard[0], i, 0))              # (value, shard, position)
heapify(heap)

for _ in range(offset + 1):
    if not heap:
        return None                              # fewer than offset+1 events
    value, shard_idx, pos = heap.pop_min()
    if pos + 1 < len(shards[shard_idx]):
        heap.push((shards[shard_idx][pos + 1], shard_idx, pos + 1))

return (value, shard_idx)                        # the last popped entry
```

Points to be deliberate about:

- **Order the heap by `(value, shard_index)`.** In Python, tuples compare
  lexicographically, so pushing `(value, shard_idx, pos)` gets the tie-break
  for free — `pos` never participates in a tie because two entries from the
  same shard are never in the heap simultaneously. In Rust, `BinaryHeap` is a
  **max**-heap, so wrap entries in `Reverse(...)` or implement `Ord` inverted;
  either way, make the `(value, shard_index)` ordering explicit rather than
  relying on field order by accident.
- **Return the last popped entry, not the new heap minimum.** The loop pops
  `offset + 1` times; the answer is what came out on the final pop.
- **Check for an empty heap inside the loop**, before popping. That single
  check handles all three exhaustion cases at once: no shards at all, only
  empty shards, and an offset past the end.
- **Timestamps reach 10^18**, so use 64-bit values.

Complexity: building the heap is `O(k)`, and each of the `offset + 1` steps
does one pop and at most one push at `O(log k)`. That is
`O(k + offset · log k)` time and `O(k)` space — independent of `N` beyond the
initial scan for non-empty shards.

</details>
