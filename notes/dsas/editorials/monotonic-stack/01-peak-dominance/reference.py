"""Reference solution — Peak Dominance.

Verified against
problems/monotonic-stack/01-peak-dominance/python/challenge.py.
O(n) time, O(n) space.
"""

from typing import List


def dominance_spans(load: List[int]) -> List[int]:
    n = len(load)

    # left[i]  = index of the nearest STRICTLY greater value to the left, or -1
    # right[i] = index of the nearest STRICTLY greater value to the right, or n
    left = [-1] * n
    right = [n] * n

    # Pass 1, left to right. The stack holds indices whose values are strictly
    # decreasing from bottom to top.
    #
    # The pop condition is `<=`, not `<`. An element that is older AND no taller
    # than the current one is shadowed forever: if it would ever qualify as the
    # "strictly greater" neighbour for some future index j, then the current
    # index qualifies too and is nearer. Using `<` leaves equal values on the
    # stack, they get reported as boundaries, and every plateau collapses to 1.
    stack: List[int] = []
    for i in range(n):
        while stack and load[stack[-1]] <= load[i]:
            stack.pop()
        left[i] = stack[-1] if stack else -1
        stack.append(i)

    # Pass 2, right to left. Identical logic, mirrored.
    stack = []
    for i in range(n - 1, -1, -1):
        while stack and load[stack[-1]] <= load[i]:
            stack.pop()
        right[i] = stack[-1] if stack else n
        stack.append(i)

    # The count of samples strictly between the two blocking indices. For the
    # global maximum this is n - (-1) - 1 = n, as it should be.
    #
    # Each index is pushed once and popped at most once across a pass, so the
    # inner `while` is amortised: both passes are O(n), not O(n^2).
    return [right[i] - left[i] - 1 for i in range(n)]
