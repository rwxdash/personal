"""Reference solution — Maintenance Windows.

Verified against problems/dp-1d/01-maintenance-windows/python/challenge.py.
O(n) time, O(n) space.
"""

from typing import List


def max_maintenance_value(
    value: List[int], blackout: List[bool], cooldown: int
) -> int:
    n = len(value)
    if n == 0:
        return 0

    # best[i] = maximum total value using only slots 0..i.
    best = [0] * n

    for i in range(n):
        # Option 1: skip slot i. Whatever was achievable through i-1 still is.
        skip = best[i - 1] if i > 0 else 0

        # Option 2: take slot i, if allowed. Taking it collects value[i] and
        # forces everything else back to slot i - cooldown - 1 or earlier.
        # We do NOT need to know which earlier slots were used -- only the best
        # total achievable by that cut-off, which is one array lookup.
        take = 0
        if not blackout[i]:
            j = i - cooldown - 1
            # j < 0 means nothing before is reachable; taking slot i alone is
            # still legal, so the base is 0 rather than "invalid".
            take = value[i] + (best[j] if j >= 0 else 0)

        best[i] = skip if skip > take else take

    # `best` is non-decreasing (skipping is always allowed), which is why the
    # single lookup at j is already the maximum over everything at or before j
    # -- no backward scan needed, so this stays O(n) rather than O(n^2).
    return best[n - 1]
