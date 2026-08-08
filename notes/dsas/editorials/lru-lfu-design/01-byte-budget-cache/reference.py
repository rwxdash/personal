"""Reference solution — Byte-Budget Cache.

Verified against
problems/lru-lfu-design/01-byte-budget-cache/python/challenge.py.
O(len(ops)) time overall, O(len(ops)) space.
"""

from collections import OrderedDict
from typing import List, Tuple


def cache_simulate(budget: int, ops: List[Tuple[int, int, int, int]]) -> List[int]:
    # OrderedDict IS a hash map plus a doubly linked list -- exactly the
    # structure this problem needs. move_to_end and popitem(last=False) are both
    # O(1), which is what keeps the whole run linear.
    #
    # Order convention here: LEAST recently used first, most recent last.
    cache: "OrderedDict[int, Tuple[int, int]]" = OrderedDict()  # key -> (value, size)
    total = 0
    out: List[int] = []

    for kind, key, value, size in ops:
        if kind == 0:
            if key in cache:
                cache.move_to_end(key)  # a HIT refreshes recency
                out.append(cache[key][0])
            else:
                out.append(-1)  # a MISS changes nothing at all
            continue

        # Rejected outright: do not evict, do not disturb an existing entry.
        # This check must come before anything else touches the cache.
        if size > budget:
            continue

        if key in cache:
            # Release the OLD size before adding the new one, or the running
            # total drifts upward and the cache evicts for no reason.
            total -= cache[key][1]

        cache[key] = (value, size)
        cache.move_to_end(key)  # a PUT also refreshes recency
        total += size

        # while, not if: one large insert can evict several entries. The
        # just-inserted key is never the victim -- it is at the most-recent end,
        # and since size <= budget the loop always stops before reaching it.
        while total > budget:
            _, (_, victim_size) = cache.popitem(last=False)
            total -= victim_size

    return out
