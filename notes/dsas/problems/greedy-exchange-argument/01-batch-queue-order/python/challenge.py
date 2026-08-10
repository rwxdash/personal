"""
Batch Queue Order
problems/greedy-exchange-argument/01-batch-queue-order

Fill in `min_total_cost`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List


def min_total_cost(duration: List[int], rate: List[int]) -> int:
    """Minimum total cost of running all jobs on one worker, in the best order.

    The worker starts at time 0 and runs jobs back to back with no gaps. A job
    finishing at time t contributes rate[i] * t to the total.

    Args:
        duration: Seconds each job takes. 0 <= n <= 100_000, each 1 ..= 10_000.
        rate:     Cost per second that each job remains unfinished. Same
                  length, each 1 ..= 10_000.

    Returns:
        The minimum achievable total cost, or 0 when there are no jobs.
        Can reach about 5 * 10^17.

    Required: O(n log n) time, O(n) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check tries every permutation, which shares no reasoning
# with a sorting rule -- so a pass is evidence the ordering rule is optimal,
# not merely self-consistent.

from itertools import permutations  # noqa: E402  (used only by the oracle)


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _cost_of_order(duration: List[int], rate: List[int], order) -> int:
    clock = 0
    total = 0
    for i in order:
        clock += duration[i]
        total += rate[i] * clock
    return total


def _brute(duration: List[int], rate: List[int]) -> int:
    """Try every permutation. O(n! * n); only viable for tiny n."""
    n = len(duration)
    if n == 0:
        return 0
    best = None
    for order in permutations(range(n)):
        c = _cost_of_order(duration, rate, order)
        if best is None or c < best:
            best = c
    return best


def _test_examples() -> None:
    assert min_total_cost([3, 1], [1, 1]) == 5
    assert min_total_cost([3, 1], [10, 1]) == 34
    assert min_total_cost([2, 3], [3, 5]) == 30
    assert min_total_cost([7], [4]) == 28


def _test_simple_rules_fail() -> None:
    """Sorting by duration alone, or by rate alone, gives wrong answers."""
    # Sorting by duration ascending would run job 1 first and cost 41.
    assert min_total_cost([3, 1], [10, 1]) == 34
    # Sorting by rate descending is no help when all rates are equal.
    assert min_total_cost([3, 1], [1, 1]) == 5
    # Both rules disagree with each other here.
    assert min_total_cost([2, 3], [3, 5]) == 30
    # Longest job has the lowest rate: it must go last.
    # Order is job1, job2, job0: clock 1 -> 100, clock 2 -> 200, clock 102 -> 102.
    assert min_total_cost([100, 1, 1], [1, 100, 100]) == 402


def _test_edges() -> None:
    # No jobs.
    assert min_total_cost([], []) == 0
    # One job.
    assert min_total_cost([1], [1]) == 1
    assert min_total_cost([10_000], [10_000]) == 100_000_000
    # Identical jobs: order cannot matter.
    assert min_total_cost([5, 5, 5], [2, 2, 2]) == 2 * (5 + 10 + 15)
    # Equal ratios, different magnitudes: any order costs the same.
    assert min_total_cost([2, 4], [1, 2]) == min_total_cost([4, 2], [2, 1])
    # All durations 1: the answer is driven purely by rate order.
    assert min_total_cost([1, 1, 1], [1, 2, 3]) == 3 * 1 + 2 * 2 + 1 * 3
    # All rates 1: the answer is driven purely by duration order.
    assert min_total_cost([3, 1, 2], [1, 1, 1]) == 1 + 3 + 6
    # Value ceilings on both sides.
    assert min_total_cost([10_000, 10_000], [10_000, 10_000]) == (
        10_000 * 10_000 + 10_000 * 20_000
    )


def _test_ties() -> None:
    """Jobs with equal duration/rate ratios cost the same in any order."""
    # 2/1 and 4/2 and 6/3 all have ratio 2.
    a = min_total_cost([2, 4, 6], [1, 2, 3])
    b = min_total_cost([6, 4, 2], [3, 2, 1])
    c = min_total_cost([4, 2, 6], [2, 1, 3])
    assert a == b == c
    # A tie mixed with a clear winner and a clear loser.
    assert min_total_cost([2, 4, 1], [1, 2, 100]) == min_total_cost(
        [4, 2, 1], [2, 1, 100]
    )


def _test_random_against_brute() -> None:
    rng = _Lcg(0x650A73548BAF63DE)
    for _ in range(300):
        n = rng.below(8)
        duration = [1 + rng.below(9) for _ in range(n)]
        rate = [1 + rng.below(9) for _ in range(n)]
        got = min_total_cost(duration, rate)
        want = _brute(duration, rate)
        assert got == want, "duration=%r rate=%r got=%d want=%d" % (
            duration,
            rate,
            got,
            want,
        )


def _test_random_ratio_collisions() -> None:
    """Values chosen so equal ratios occur constantly.

    Durations and rates are drawn from small multiples, so many pairs tie. This
    is where a floating-point comparator produces an inconsistent ordering.
    """
    rng = _Lcg(0x766A0ABB3C77B2A8)
    for _ in range(300):
        n = rng.below(7)
        duration = []
        rate = []
        for _ in range(n):
            k = 1 + rng.below(4)
            duration.append(2 * k)
            rate.append(k)  # every ratio is exactly 2
        # Perturb a few so not everything ties.
        for i in range(n):
            if rng.below(3) == 0:
                duration[i] += 1
        assert min_total_cost(duration, rate) == _brute(duration, rate)


def _test_large_uniform() -> None:
    """100k identical jobs: the answer is a closed form."""
    n = 100_000
    d = 3
    r = 2
    duration = [d] * n
    rate = [r] * n
    # Finish times are d, 2d, ..., nd; total = r * d * n(n+1)/2.
    expected = r * d * (n * (n + 1) // 2)
    assert min_total_cost(duration, rate) == expected


def _test_large_all_max() -> None:
    """Every value at its ceiling: forces 64-bit arithmetic."""
    n = 100_000
    duration = [10_000] * n
    rate = [10_000] * n
    expected = 10_000 * 10_000 * (n * (n + 1) // 2)
    assert min_total_cost(duration, rate) == expected
    assert expected > 2**31


def _test_large_sorted_input() -> None:
    """Input already in the best order, and in the worst order.

    Both must produce the same answer -- the function may not depend on the
    order it was handed.
    """
    n = 50_000
    # Job i has duration i+1 and rate 1, so the best order is ascending.
    duration = list(range(1, n + 1))
    rate = [1] * n
    best_first = min_total_cost(duration, rate)
    worst_first = min_total_cost(duration[::-1], rate)
    assert best_first == worst_first
    # Closed form: sum over k of (n - k + 1) * k for k = 1..n.
    expected = sum((n - k + 1) * k for k in range(1, n + 1))
    assert best_first == expected


def _test_large_two_classes() -> None:
    """Half the jobs are cheap-and-fast, half are dear-and-slow."""
    half = 50_000
    duration = [1] * half + [10_000] * half
    rate = [10_000] * half + [1] * half
    # Ratio 1/10000 for the first group, 10000/1 for the second, so the fast
    # high-rate jobs all run first.
    clock = 0
    expected = 0
    for _ in range(half):
        clock += 1
        expected += 10_000 * clock
    for _ in range(half):
        clock += 10_000
        expected += 1 * clock
    assert min_total_cost(duration, rate) == expected


def _test_large_all_ties() -> None:
    """100k jobs with identical ratios but different magnitudes.

    Every order is optimal, so the total must match the closed form regardless
    of how the sort breaks ties.
    """
    n = 100_000
    duration = [2 * (1 + i % 100) for i in range(n)]
    rate = [1 + i % 100 for i in range(n)]
    clock = 0
    expected = 0
    # Any order gives the same total; compute one directly.
    for i in range(n):
        clock += duration[i]
        expected += rate[i] * clock
    got = min_total_cost(duration, rate)
    assert got == expected, "all ratios equal, so every order costs the same"


if __name__ == "__main__":
    _test_examples()
    _test_simple_rules_fail()
    _test_edges()
    _test_ties()
    _test_random_against_brute()
    _test_random_ratio_collisions()
    _test_large_uniform()
    _test_large_all_max()
    _test_large_sorted_input()
    _test_large_two_classes()
    _test_large_all_ties()
    print("All tests passed.")
