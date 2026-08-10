"""Reference solution — Autoscaler Stable Windows.

Verified against
problems/monotonic-queue/01-autoscaler-stable-windows/python/challenge.py.
O(n) time, O(n) space.
"""

from collections import deque
from typing import Deque, List


def count_stable_windows(usage: List[int], delta: int, min_len: int) -> int:
    # Two monotonic deques of *indices* into `usage`:
    #   max_dq: values non-increasing front-to-back, so front = window maximum
    #   min_dq: values non-decreasing front-to-back, so front = window minimum
    # A plain running max/min cannot work, because when `left` advances past the
    # current extreme there is no way to recover the next one. The deques keep
    # the runners-up in the exact order they will be promoted.
    max_dq: Deque[int] = deque()
    min_dq: Deque[int] = deque()

    total = 0
    left = 0

    for right, value in enumerate(usage):
        # Admit `right`. Any earlier index holding a value <= this one can never
        # be the maximum again -- it is both older and smaller, so it is
        # dominated forever. Same argument mirrored for the minimum.
        while max_dq and usage[max_dq[-1]] <= value:
            max_dq.pop()
        max_dq.append(right)
        while min_dq and usage[min_dq[-1]] >= value:
            min_dq.pop()
        min_dq.append(right)

        # Restore stability by advancing `left`. This terminates: once
        # left == right the window is a single sample with spread 0 <= delta.
        while usage[max_dq[0]] - usage[min_dq[0]] > delta:
            if max_dq[0] == left:
                max_dq.popleft()
            if min_dq[0] == left:
                min_dq.popleft()
            left += 1

        # `left` is now L(right): the smallest start keeping the range stable.
        # Every start in [left, right] gives a stable range, and the length
        # floor trims that to [left, right - min_len + 1]. Counting the whole
        # interval at once is what keeps a ~2e10 answer reachable in n steps.
        last_start = right - min_len + 1
        if last_start >= left:
            total += last_start - left + 1

    return total
