"""Reference solution — Host Consolidation.

Verified against
problems/two-pointers/01-host-consolidation/python/challenge.py.
O(n log n) time, O(n) space.
"""

from typing import List


def min_hosts(footprints: List[int], pinned: List[bool], cap: int) -> int:
    # Pinned VMs never interact with anything: one host each, then forget them.
    hosts = 0
    free: List[int] = []
    for size, is_pinned in zip(footprints, pinned):
        if is_pinned:
            hosts += 1
        else:
            free.append(size)

    free.sort()

    # Converging two pointers. Minimising hosts == maximising pairs, and the
    # exchange argument says the largest VM should take the SMALLEST partner
    # that fits: if the smallest does not fit, nothing does.
    lo, hi = 0, len(free) - 1
    while lo <= hi:
        if free[lo] + free[hi] <= cap:
            lo += 1  # the smallest rides along with the largest
        hi -= 1  # the largest is placed on this host either way
        hosts += 1  # exactly one host consumed per iteration

    # The lo == hi case needs no special handling: the test becomes
    # 2 * free[lo] <= cap, and whichever way it goes the pointers cross with
    # exactly one host counted for that final VM.
    return hosts
