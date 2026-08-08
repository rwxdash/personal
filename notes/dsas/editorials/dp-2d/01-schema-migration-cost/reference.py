"""Reference solution — Schema Migration Cost.

Verified against
problems/dp-2d/01-schema-migration-cost/python/challenge.py.
O(n * m) time, O(min(n, m)) space.
"""

from typing import List


def migration_cost(
    current: List[str],
    target: List[str],
    insert_cost: int,
    drop_cost: int,
    retype_cost: int,
) -> int:
    # Keep the SHORTER sequence as the inner loop so the rolling rows are
    # O(min(n, m)). Swapping the sequences means swapping the direction of the
    # migration, so insert and drop must swap with them -- "dropping from A to
    # reach B" is "inserting into B to reach A". Forgetting this is silent: it
    # only shows up when the two costs differ AND current is the shorter side.
    if len(current) < len(target):
        current, target = target, current
        insert_cost, drop_cost = drop_cost, insert_cost

    n, m = len(current), len(target)

    # prev[j] = cost to turn the first i columns of `current` into the first j
    # of `target`. Row 0 is "consume nothing, insert j columns".
    prev = [j * insert_cost for j in range(m + 1)]

    for i in range(1, n + 1):
        # Column 0 of this row: drop all i consumed columns, produce nothing.
        cur = [i * drop_cost] + [0] * m
        left = current[i - 1]
        for j in range(1, m + 1):
            # Three moves out of this state. The min is taken unconditionally,
            # including when the names match. (Short-circuiting to the free
            # diagonal on a match is also correct -- see the editorial for the
            # two inequalities that prove it -- but this is simpler.)
            step = 0 if left == target[j - 1] else retype_cost
            a = prev[j] + drop_cost        # drop current[i-1]
            b = cur[j - 1] + insert_cost   # insert target[j-1]
            c = prev[j - 1] + step         # handle both at once
            best = a if a < b else b
            cur[j] = best if best < c else c
        prev = cur

    # No special case is needed for "retype costs more than drop + insert":
    # the min finds that route through the grid on its own.
    return prev[m]
