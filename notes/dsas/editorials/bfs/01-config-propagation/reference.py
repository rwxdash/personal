"""Reference solution — Config Propagation.

Verified against problems/bfs/01-config-propagation/python/challenge.py.
O(n + m) time and space.
"""

from collections import deque
from typing import Deque, List, Tuple


def propagation_rounds(
    n: int,
    links: List[Tuple[int, int]],
    seeds: List[int],
    quarantined: List[bool],
) -> List[int]:
    adj: List[List[int]] = [[] for _ in range(n)]
    for u, v in links:
        adj[u].append(v)
        adj[v].append(u)  # self-loops add u twice; harmless

    # `round_of` doubles as the visited marker: -1 means "not yet reached",
    # which is also the required output for unreachable nodes. One array, so
    # the two can never disagree.
    round_of = [-1] * n
    queue: Deque[int] = deque()

    # MULTI-SOURCE: every seed enters at distance 0 before the sweep starts.
    # Equivalent to a virtual node joined to all seeds, minus its one hop.
    # Because BFS dequeues in non-decreasing distance order, the first time a
    # node is reached its round is already the minimum over ALL seeds -- no
    # per-seed passes and no post-hoc minimum.
    for s in seeds:
        if round_of[s] == -1:  # guards against duplicate seeds
            round_of[s] = 0
            queue.append(s)

    while queue:
        u = queue.popleft()
        if quarantined[u]:
            # Dequeued and already recorded (its round was written when it was
            # ENQUEUED), but it expands nothing. Placing the check here rather
            # than at enqueue time is what preserves its own round number --
            # and makes a quarantined seed work with no special case.
            continue
        for v in adj[u]:
            if round_of[v] == -1:
                round_of[v] = round_of[u] + 1
                queue.append(v)

    return round_of
