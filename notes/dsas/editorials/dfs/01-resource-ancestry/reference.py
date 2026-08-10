"""Reference solution — Resource Ancestry.

Verified against problems/dfs/01-resource-ancestry/python/challenge.py.
O(n + q) time, O(n) space.
"""

from typing import List, Tuple


def ancestor_queries(
    parent: List[int], queries: List[Tuple[int, int]]
) -> List[bool]:
    n = len(parent)

    # Children lists and roots, in one O(n) pass.
    children: List[List[int]] = [[] for _ in range(n)]
    roots: List[int] = []
    for i, p in enumerate(parent):
        if p == -1:
            roots.append(i)
        else:
            children[p].append(i)

    # A node's subtree is CONTIGUOUS in depth-first visit order: once you step
    # into u you visit all of u's subtree, and nothing else, before stepping
    # back out. Stamping an entry and an exit time therefore turns each subtree
    # into an interval, and ancestry into interval containment.
    tin = [0] * n
    tout = [0] * n
    timer = 0

    for r in roots:
        # ITERATIVE on purpose. Recursion dies on a 200k-deep chain, and this
        # problem explicitly allows one. The boolean is the "returning to this
        # node after its children" flag that recursion gets for free.
        stack: List[Tuple[int, bool]] = [(r, False)]
        while stack:
            node, returning = stack.pop()
            if returning:
                tout[node] = timer
                timer += 1
                continue
            tin[node] = timer
            timer += 1
            # Push the exit marker BEFORE the children so it pops AFTER them.
            stack.append((node, True))
            for c in children[node]:
                stack.append((c, False))

    # `timer` is global across all roots, never reset. That is what makes
    # different trees occupy disjoint intervals, so cross-tree queries are
    # False automatically. Both comparisons are non-strict, which is what makes
    # u == v return True with no special case.
    return [tin[u] <= tin[v] and tout[v] <= tout[u] for u, v in queries]
