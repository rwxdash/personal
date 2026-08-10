"""Reference solution — Sharded Audit Log.

Verified against
problems/heaps-k-way-merge/01-sharded-audit-log/python/challenge.py.
O(k + offset * log k) time, O(k) space.
"""

import heapq
from typing import List, Optional, Tuple


def nth_merged_event(
    shards: List[List[int]], offset: int
) -> Optional[Tuple[int, int]]:
    # The globally earliest remaining event is always at the FRONT of some
    # shard -- it cannot be buried mid-shard, because everything before it there
    # is smaller or equal. So only k candidates are ever live, one per shard,
    # and the heap holds exactly those.
    #
    # Entries are (timestamp, shard_index, position). Tuples compare
    # lexicographically, so ordering by (timestamp, shard_index) -- the required
    # tie rule -- comes for free. `position` never participates in a tie, since
    # two entries from the same shard are never in the heap at once.
    heap: List[Tuple[int, int, int]] = [
        (shard[0], i, 0) for i, shard in enumerate(shards) if shard
    ]
    heapq.heapify(heap)  # O(k), cheaper than k pushes

    value = shard_idx = 0
    for _ in range(offset + 1):
        if not heap:
            # One check covers all three exhaustion cases: no shards, only
            # empty shards, and an offset past the end.
            return None
        value, shard_idx, pos = heapq.heappop(heap)
        nxt = pos + 1
        if nxt < len(shards[shard_idx]):
            heapq.heappush(heap, (shards[shard_idx][nxt], shard_idx, nxt))

    # The answer is the LAST entry popped, not the current heap minimum.
    return (value, shard_idx)
