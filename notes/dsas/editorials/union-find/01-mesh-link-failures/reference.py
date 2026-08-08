"""Reference solution — Mesh Link Failures.

Verified against
problems/union-find/01-mesh-link-failures/python/challenge.py.
O((n + m) * alpha(n)) time, O(n + m) space.
"""

from typing import List, Tuple


def partitions_after_failures(
    n: int,
    links: List[Tuple[int, int]],
    failures: List[int],
) -> List[int]:
    q = len(failures)
    if q == 0:
        return []

    # Union-find cannot split a set, so deletions are run backwards: the state
    # after ALL failures is built first, then failures are replayed in reverse,
    # which turns every deletion into an insertion.
    doomed = [False] * len(links)
    for idx in failures:
        doomed[idx] = True

    parent = list(range(n))
    size = [1] * n
    count = n  # every node starts as its own partition

    def find(x: int) -> int:
        # Iterative on purpose: a recursive find can nest 200k deep.
        root = x
        while parent[root] != root:
            root = parent[root]
        while parent[x] != root:  # path compression, second pass
            parent[x], x = root, parent[x]
        return root

    def union(a: int, b: int) -> None:
        # `count` drops only on a real merge. That single line is what makes
        # redundant cables and self-loops inert for free: both hit ra == rb.
        nonlocal count
        ra, rb = find(a), find(b)
        if ra == rb:
            return
        if size[ra] < size[rb]:  # union by size keeps trees shallow
            ra, rb = rb, ra
        parent[rb] = ra
        size[ra] += size[rb]
        count -= 1

    # Phase 1: the most fragmented state, S_q -- only links that never fail.
    for i, (u, v) in enumerate(links):
        if not doomed[i]:
            union(u, v)

    # Phase 2: walk time backwards. After re-adding failures[k], the structure
    # represents S_k (the state after the first k failures), whose count is the
    # answer for step k, stored at answer[k - 1].
    answer = [0] * q
    answer[q - 1] = count  # count currently describes S_q
    for k in range(q - 1, 0, -1):
        u, v = links[failures[k]]
        union(u, v)
        answer[k - 1] = count

    return answer
