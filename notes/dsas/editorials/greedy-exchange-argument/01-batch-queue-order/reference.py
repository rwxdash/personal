"""Reference solution — Batch Queue Order.

Verified against
problems/greedy-exchange-argument/01-batch-queue-order/python/challenge.py.
O(n log n) time, O(n) space.
"""

from fractions import Fraction
from typing import List


def min_total_cost(duration: List[int], rate: List[int]) -> int:
    # Swapping two ADJACENT jobs changes nothing outside the pair -- everything
    # before is untouched, and everything after finishes at the same time
    # because the pair occupies the same block either way. Working out that
    # local swap gives the whole ordering rule:
    #
    #   A before B costs an extra rB * dA;  B before A costs an extra rA * dB.
    #   So A goes first exactly when  dA * rB < dB * rA,  i.e. dA/rA < dB/rB.
    #
    # Since no adjacent pair wants to swap once sorted this way, and any order
    # can be reached from any other by adjacent swaps, the sorted order is
    # optimal.
    jobs = list(zip(duration, rate))

    # Fraction keeps the comparison EXACT. A float ratio (d / r) can make two
    # genuinely-equal ratios compare as unequal, which is not just a wrong
    # answer -- an inconsistent comparator is undefined behaviour for some
    # sorts. Cross-multiplication (d_a * r_b vs d_b * r_a) is the other exact
    # option and is what reference.rs uses.
    jobs.sort(key=lambda job: Fraction(job[0], job[1]))

    clock = 0
    total = 0
    for d, r in jobs:
        # Advance the clock FIRST: a job pays for the time up to and including
        # its own run, not up to the moment it started.
        clock += d
        total += r * clock

    return total
