//! Reference solution — Sharded Audit Log.
//!
//! Verified against problems/heaps-k-way-merge/01-sharded-audit-log/rust.
//! O(k + offset * log k) time, O(k) space.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub fn nth_merged_event(shards: &[Vec<i64>], offset: usize) -> Option<(i64, usize)> {
    // The globally earliest remaining event is always at the FRONT of some
    // shard -- it cannot be buried mid-shard, because everything before it there
    // is smaller or equal. So only k candidates are ever live, one per shard.
    //
    // BinaryHeap is a MAX-heap, so entries are wrapped in `Reverse` to get
    // min-first. Tuples compare lexicographically, so ordering by
    // (timestamp, shard_index) -- the required tie rule -- comes for free.
    // `position` never participates in a tie, since two entries from the same
    // shard are never in the heap at once.
    let mut heap: BinaryHeap<Reverse<(i64, usize, usize)>> = shards
        .iter()
        .enumerate()
        .filter(|(_, shard)| !shard.is_empty())
        .map(|(i, shard)| Reverse((shard[0], i, 0usize)))
        .collect(); // `collect` into a BinaryHeap heapifies in O(k)

    let mut value: i64 = 0;
    let mut shard_idx: usize = 0;

    for _ in 0..=offset {
        // One check covers all three exhaustion cases: no shards, only empty
        // shards, and an offset past the end.
        let Reverse((v, s, pos)) = heap.pop()?;
        value = v;
        shard_idx = s;
        let nxt = pos + 1;
        if nxt < shards[s].len() {
            heap.push(Reverse((shards[s][nxt], s, nxt)));
        }
    }

    // The answer is the LAST entry popped, not the current heap minimum.
    Some((value, shard_idx))
}
