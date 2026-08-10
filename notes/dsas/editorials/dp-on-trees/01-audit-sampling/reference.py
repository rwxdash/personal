"""Reference solution — Audit Sampling.

Verified against
problems/dp-on-trees/01-audit-sampling/python/challenge.py.
O(n) time, O(n) space.
"""

from typing import List


def max_audit_evidence(parent: List[int], evidence: List[int]) -> int:
    n = len(parent)

    # Children lists, in one pass.
    children: List[List[int]] = [[] for _ in range(n)]
    for i in range(n):
        p = parent[i]
        if p != -1:
            children[p].append(i)

    # Pass 1: an order in which every node appears AFTER its parent. Reversing
    # it then puts every node after all of its children, which is what the
    # recurrence needs.
    #
    # Two flat passes instead of one recursive walk: a 200k-deep chain would
    # blow the stack, and this avoids any "return to the node after its
    # children" bookkeeping.
    order: List[int] = []
    stack = [0]
    while stack:
        v = stack.pop()
        order.append(v)
        for c in children[v]:
            stack.append(c)

    # skip[v] = best total in v's subtree when v is NOT audited
    # take[v] = best total in v's subtree when v IS audited
    skip = [0] * n
    take = [0] * n

    # Pass 2: children before parents.
    for idx in range(len(order) - 1, -1, -1):
        v = order[idx]
        total_forced = 0  # children must skip, because v is audited
        total_free = 0    # children choose freely, because v is skipped
        for c in children[v]:
            total_forced += skip[c]
            # max, NOT take[c]: skipping v does not oblige a child to be
            # audited, it only removes the restriction.
            total_free += skip[c] if skip[c] > take[c] else take[c]
        skip[v] = total_free
        take[v] = evidence[v] + total_forced

    # Leaves need no special case: their sums are empty, so skip = 0 and
    # take = evidence[v].
    #
    # The root may well be better left unaudited, so take the max here too.
    return skip[0] if skip[0] > take[0] else take[0]
