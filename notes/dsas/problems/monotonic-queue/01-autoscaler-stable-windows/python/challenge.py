"""
Autoscaler Stable Windows
problems/monotonic-queue/01-autoscaler-stable-windows

Fill in `count_stable_windows`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List


def count_stable_windows(usage: List[int], delta: int, min_len: int) -> int:
    """Count the contiguous ranges of `usage` that are stable and long enough.

    A range `usage[i..j]` (inclusive, i <= j) qualifies when both hold:
      * max(usage[i..j]) - min(usage[i..j]) <= delta
      * j - i + 1 >= min_len

    Ranges are counted by position: usage[3..7] and usage[4..8] are distinct
    even if they hold the same values.

    Args:
        usage:   CPU readings in tick order. 0 <= len(usage) <= 200_000,
                 each reading in 0..=10^9.
        delta:   Maximum tolerated spread, 0 <= delta <= 10^9.
        min_len: Minimum run length, 1 <= min_len <= 200_000. May exceed
                 len(usage), in which case the answer is 0.

    Returns:
        The number of qualifying ranges. Can be ~2 * 10^10.

    Required: O(n) time expected, O(n) extra space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The large cases use series whose exact answer is known in closed form, so a
# pass here is a real correctness signal and not a self-fulfilling check.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(usage: List[int], delta: int, min_len: int) -> int:
    """Obviously-correct O(n^2) checker. Hopeless at the real bounds."""
    total = 0
    for i in range(len(usage)):
        lo = hi = usage[i]
        for j in range(i, len(usage)):
            lo = min(lo, usage[j])
            hi = max(hi, usage[j])
            if hi - lo <= delta and (j - i + 1) >= min_len:
                total += 1
    return total


def _ramp_expected(n: int, delta: int, min_len: int) -> int:
    """Exact count for usage[i] = i.

    On a strictly increasing unit ramp, usage[i..j] has spread j - i, so a range
    is stable iff its length is at most delta + 1. Counting by length L, there
    are (n - L + 1) ranges of length L.
    """
    total = 0
    for length in range(max(min_len, 1), min(delta + 1, n) + 1):
        total += n - length + 1
    return total


def _test_examples() -> None:
    assert count_stable_windows([5, 7, 6, 9], 2, 1) == 7
    assert count_stable_windows([5, 7, 6, 9], 2, 2) == 3
    assert count_stable_windows([4, 4, 4, 1], 0, 1) == 7
    assert count_stable_windows([1, 2, 3], 10, 5) == 0


def _test_edges() -> None:
    # Empty series.
    assert count_stable_windows([], 0, 1) == 0
    assert count_stable_windows([], 10**9, 1) == 0
    # Single sample: stable by definition, but only if min_len allows it.
    assert count_stable_windows([42], 0, 1) == 1
    assert count_stable_windows([42], 10**9, 2) == 0
    # min_len exactly equal to n, and one past it.
    assert count_stable_windows([3, 3, 3], 0, 3) == 1
    assert count_stable_windows([3, 3, 3], 0, 4) == 0
    # delta = 0 with no two equal neighbours: only the singles qualify.
    assert count_stable_windows([1, 2, 3, 4], 0, 1) == 4
    # All identical, generous delta: every one of the n*(n+1)/2 ranges counts.
    assert count_stable_windows([7] * 10, 0, 1) == 55
    assert count_stable_windows([7] * 10, 5, 1) == 55
    # Extreme values at the type boundary.
    assert count_stable_windows([0, 10**9], 10**9, 1) == 3
    assert count_stable_windows([0, 10**9], 10**9 - 1, 1) == 2
    # Ties and duplicates around the window edges.
    assert count_stable_windows([2, 2, 5, 2, 2], 0, 2) == 2


def _test_random_against_brute() -> None:
    rng = _Lcg(0x9E3779B97F4A7C15)
    for _ in range(400):
        n = rng.below(30)
        spread = 1 + rng.below(8)
        usage = [rng.below(spread) for _ in range(n)]
        delta = rng.below(6)
        min_len = 1 + rng.below(6)
        got = count_stable_windows(usage, delta, min_len)
        want = _brute(usage, delta, min_len)
        assert got == want, "usage=%r delta=%d min_len=%d got=%d want=%d" % (
            usage,
            delta,
            min_len,
            got,
            want,
        )


def _test_large_ramp() -> None:
    """200k strictly increasing readings; answer known in closed form.

    The O(n^2) checker would need ~2 * 10^10 iterations here.
    """
    n = 200_000
    usage = list(range(n))
    assert count_stable_windows(usage, 999, 1) == 199_500_500
    assert count_stable_windows(usage, 999, 1) == _ramp_expected(n, 999, 1)
    assert count_stable_windows(usage, 999, 500) == 99_824_751
    assert count_stable_windows(usage, 999, 500) == _ramp_expected(n, 999, 500)


def _test_large_flat() -> None:
    """Every range qualifies: forces a 64-bit accumulator."""
    n = 200_000
    usage = [17] * n
    expected = n * (n + 1) // 2  # 20_000_100_000
    assert count_stable_windows(usage, 0, 1) == expected
    assert expected > 2**31, "this case exists to overflow 32-bit counters"


def _test_large_sawtooth() -> None:
    """Alternating extremes: the window can never grow past a single sample."""
    n = 200_000
    usage = [0 if i % 2 == 0 else 10**9 for i in range(n)]
    assert count_stable_windows(usage, 0, 1) == n
    assert count_stable_windows(usage, 0, 2) == 0
    # With the full tolerance every range qualifies again.
    assert count_stable_windows(usage, 10**9, 1) == n * (n + 1) // 2


def _test_large_blocks() -> None:
    """Equal-valued blocks: answer is the sum of per-block triangular numbers."""
    n = 200_000
    block = 500
    usage = [(i // block) * 10 for i in range(n)]
    blocks = n // block
    expected = blocks * (block * (block + 1) // 2)
    assert count_stable_windows(usage, 0, 1) == expected
    # min_len = block leaves exactly one full range per block.
    assert count_stable_windows(usage, 0, block) == blocks


def _test_large_random_shape() -> None:
    """Randomised 200k series checked against an independent recomputation.

    The values only ever move by 0 or +1, so the series is non-decreasing and
    the stable-range count can be recomputed by a direct two-pointer scan on
    the sorted-by-construction data without reusing the solution's structure.
    """
    n = 200_000
    rng = _Lcg(0xDEADBEEFCAFEF00D)
    usage: List[int] = []
    cur = 0
    for _ in range(n):
        cur += rng.below(2)
        usage.append(cur)
    delta = 50
    min_len = 3
    # Independent check: on a non-decreasing series, usage[i..j] is stable iff
    # usage[j] - usage[i] <= delta, so binary search gives the boundary.
    import bisect

    expected = 0
    for j in range(n):
        i = bisect.bisect_left(usage, usage[j] - delta)
        last_start = j - min_len + 1
        if last_start >= i:
            expected += last_start - i + 1
    assert count_stable_windows(usage, delta, min_len) == expected


if __name__ == "__main__":
    _test_examples()
    _test_edges()
    _test_random_against_brute()
    _test_large_ramp()
    _test_large_flat()
    _test_large_sawtooth()
    _test_large_blocks()
    _test_large_random_shape()
    print("All tests passed.")
