"""Reference solution — Compaction Windows.

Verified against
problems/binary-search-on-answer/01-compaction-windows/python/challenge.py.
O(n log T) time, O(1) extra space.
"""

from typing import List


def min_window_bytes(sizes: List[int], checkpoints: List[bool], k: int) -> int:
    # Deciding the optimal arrangement directly is expensive. Deciding whether
    # SOME arrangement fits under a fixed ceiling is a single linear pass -- and
    # that predicate is monotone in the ceiling, so the answer is the position
    # of its single false->true flip.
    def feasible(cap: int) -> bool:
        windows = 1
        current = 0
        for i, s in enumerate(sizes):
            if i > 0 and checkpoints[i]:
                # Forced cut. Fires regardless of how much room is left, which
                # is the one thing that distinguishes this from the textbook
                # version of the problem.
                windows += 1
                current = s
            elif current + s > cap:
                windows += 1
                current = s
            else:
                current += s
            if windows > k:
                return False  # early exit; the count never decreases
        return windows <= k

    # lo is already feasible-or-better by construction: no arrangement can beat
    # the largest single segment, and starting here means `feasible` never has
    # to handle a segment that fits nowhere.
    lo = max(sizes)
    hi = sum(sizes)  # one window holding everything; always feasible since k >= 1

    while lo < hi:
        mid = lo + (hi - lo) // 2
        if feasible(mid):
            hi = mid  # mid works, so the answer is mid or smaller
        else:
            lo = mid + 1  # mid fails, so the answer is strictly larger

    return lo
