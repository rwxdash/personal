"""Reference solution — Pipeline Critical Path.

Verified against
problems/topological-sort/01-pipeline-critical-path/python/challenge.py.
O(n + m) time and space.
"""

from collections import deque
from typing import Deque, List, Optional, Tuple


def pipeline_makespan(
    durations: List[int],
    ready: List[int],
    deps: List[Tuple[int, int]],
) -> Optional[int]:
    n = len(durations)

    # Build successor lists and in-degrees. Every entry of `deps` is counted
    # exactly once in BOTH structures -- duplicates are deliberately NOT
    # collapsed. Deduplicating one structure but not the other desynchronises
    # the counters and either strands a node or enqueues it twice.
    adj: List[List[int]] = [[] for _ in range(n)]
    indeg = [0] * n
    for a, b in deps:
        adj[a].append(b)
        indeg[b] += 1

    # start[i] = earliest instant job i is currently known to be able to begin.
    # It starts at the artifact availability time and only ever increases as
    # predecessors report their finish times.
    start = list(ready)

    queue: Deque[int] = deque(i for i in range(n) if indeg[i] == 0)
    processed = 0
    answer = 0

    while queue:
        u = queue.popleft()
        processed += 1

        # INVARIANT: a node is enqueued only once its in-degree hits zero, which
        # happens only after every incoming edge was relaxed, which happens only
        # after every predecessor was popped. So start[u] is final here, and so
        # is finish_u.
        finish_u = start[u] + durations[u]
        if finish_u > answer:
            answer = finish_u  # max over ALL nodes, not just sinks

        for v in adj[u]:
            if finish_u > start[v]:
                start[v] = finish_u
            indeg[v] -= 1
            if indeg[v] == 0:
                queue.append(v)

    # Kahn's algorithm pays for itself twice: the nodes it never reached are
    # exactly those trapped in or behind a cycle. A self-loop (a, a) gives `a`
    # an in-degree it can never shed, so it is caught by the same test.
    if processed < n:
        return None
    return answer
