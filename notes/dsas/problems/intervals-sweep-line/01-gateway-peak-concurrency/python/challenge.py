"""
Gateway Peak Concurrency
problems/intervals-sweep-line/01-gateway-peak-concurrency

Fill in `peak_concurrency`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Tuple


def peak_concurrency(sessions: List[Tuple[int, int]]) -> Tuple[int, int]:
    """Largest number of simultaneously open sessions, and when it first occurs.

    A session occupies a connection slot over the HALF-OPEN interval
    [start, end): it holds the slot at instant `start` and has already released
    it at instant `end`. A session closing at t and another opening at t
    therefore do NOT overlap.

    Args:
        sessions: (start, end) pairs with start <= end. Unsorted.
                  0 <= n <= 200_000, timestamps 0 ..= 10^9.
                  Zero-length sessions (start == end) are legal and occupy no
                  instants at all, so they never contribute to concurrency.

    Returns:
        (peak, earliest_time). When the peak is 0 -- no sessions, or nothing
        but zero-length ones -- returns (0, 0).

    Required: O(n log n) time, O(n) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check evaluates concurrency directly at every candidate
# instant, which shares no machinery with an event sweep.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(sessions: List[Tuple[int, int]]) -> Tuple[int, int]:
    """Evaluate concurrency at every distinct start. O(n^2).

    Concurrency only rises at a start, so the peak is attained at one of them.
    """
    best = 0
    best_time = 0
    for t in sorted({s for s, _ in sessions}):
        c = sum(1 for s, e in sessions if s <= t < e)
        if c > best:
            best = c
            best_time = t
    return (best, best_time)


def _test_examples() -> None:
    assert peak_concurrency([(1, 5), (2, 6), (8, 10)]) == (2, 2)
    assert peak_concurrency([(1, 5), (5, 9)]) == (1, 1)
    assert peak_concurrency([(0, 100), (10, 20), (12, 15)]) == (3, 12)
    assert peak_concurrency([(5, 5), (5, 10)]) == (1, 5)


def _test_half_open() -> None:
    """Back-to-back sessions must never be counted as overlapping.

    Every assert here flips if closing events are ordered after opening events
    at the same timestamp -- the single most common way to get this wrong.
    """
    assert peak_concurrency([(1, 5), (5, 9)]) == (1, 1)
    assert peak_concurrency([(0, 1), (1, 2), (2, 3), (3, 4)]) == (1, 0)
    # A chain of handovers plus one genuine overlap.
    assert peak_concurrency([(0, 10), (10, 20), (5, 15)]) == (2, 5)
    # Overlap by exactly one instant.
    assert peak_concurrency([(0, 6), (5, 10)]) == (2, 5)
    # Miss by exactly one instant.
    assert peak_concurrency([(0, 5), (5, 10)]) == (1, 0)


def _test_zero_length() -> None:
    """Instantaneous sessions occupy no instants and never add concurrency."""
    assert peak_concurrency([(5, 5)]) == (0, 0)
    assert peak_concurrency([(5, 5), (5, 5), (5, 5)]) == (0, 0)
    assert peak_concurrency([(5, 5), (5, 10)]) == (1, 5)
    assert peak_concurrency([(0, 10), (5, 5)]) == (1, 0)
    # A zero-length session sitting inside a busy stretch changes nothing.
    assert peak_concurrency([(0, 10), (2, 8), (5, 5)]) == (2, 2)


def _test_edges() -> None:
    # Empty input.
    assert peak_concurrency([]) == (0, 0)
    # Single session.
    assert peak_concurrency([(0, 1)]) == (1, 0)
    assert peak_concurrency([(7, 100)]) == (1, 7)
    # Identical sessions all count.
    assert peak_concurrency([(3, 7), (3, 7), (3, 7)]) == (3, 3)
    # Disjoint sessions: peak 1, earliest is the first start in time order,
    # not the first in input order.
    assert peak_concurrency([(50, 60), (10, 20)]) == (1, 10)
    # Fully nested.
    assert peak_concurrency([(0, 100), (25, 75), (40, 60), (45, 55)]) == (4, 45)
    # Timestamp range boundaries.
    assert peak_concurrency([(0, 10**9)]) == (1, 0)
    assert peak_concurrency([(0, 10**9), (10**9 - 1, 10**9)]) == (2, 10**9 - 1)
    # Peak reached early, tied again later: report the earliest.
    assert peak_concurrency([(0, 5), (0, 5), (100, 105), (100, 105)]) == (2, 0)


def _test_random_against_brute() -> None:
    rng = _Lcg(0x2AF26013C5D1B023)
    for _ in range(400):
        n = rng.below(14)
        sessions = []
        for _ in range(n):
            a = rng.below(12)
            b = rng.below(12)
            # start <= end, and zero-length intervals occur naturally.
            sessions.append((min(a, b), max(a, b)))
        got = peak_concurrency(sessions)
        want = _brute(sessions)
        assert got == want, "sessions=%r got=%r want=%r" % (sessions, got, want)


def _test_random_clustered() -> None:
    """Tiny coordinate range, so shared timestamps are constant."""
    rng = _Lcg(0x70B7B98B31A2C0A9)
    for _ in range(400):
        n = rng.below(10)
        sessions = []
        for _ in range(n):
            a = rng.below(4)
            b = rng.below(4)
            sessions.append((min(a, b), max(a, b)))
        assert peak_concurrency(sessions) == _brute(sessions)


def _test_large_all_overlapping() -> None:
    """200k sessions spanning the whole timeline: peak is n at instant 0."""
    n = 200_000
    sessions = [(0, 10**9)] * n
    assert peak_concurrency(sessions) == (n, 0)


def _test_large_handover_chain() -> None:
    """200k back-to-back sessions: peak is 1 despite 200k shared timestamps.

    This is the half-open rule at scale. Ordering opens before closes reports a
    peak of 2 here.
    """
    n = 200_000
    sessions = [(i, i + 1) for i in range(n)]
    assert peak_concurrency(sessions) == (1, 0)


def _test_large_staircase() -> None:
    """Sessions of width k starting at every index: peak is k, first at k - 1.

    At instant t the open sessions are those with start in (t - k, t], so the
    count is min(k, t + 1) and the peak is first reached at t = k - 1.
    """
    n = 200_000
    k = 1_000
    sessions = [(i, i + k) for i in range(n)]
    assert peak_concurrency(sessions) == (k, k - 1)


def _test_large_nested() -> None:
    """100k nested sessions: peak is n at the innermost start."""
    n = 100_000
    sessions = [(i, 2 * n - i) for i in range(n)]
    assert peak_concurrency(sessions) == (n, n - 1)


def _test_large_disjoint() -> None:
    """200k sessions with a gap between each: peak 1, earliest instant 0."""
    n = 200_000
    sessions = [(3 * i, 3 * i + 1) for i in range(n)]
    assert peak_concurrency(sessions) == (1, 0)


def _test_large_zero_length_flood() -> None:
    """Mostly instantaneous sessions with a single real one buried inside."""
    n = 200_000
    sessions = [(i, i) for i in range(n)]
    sessions.append((12_345, 54_321))
    assert peak_concurrency(sessions) == (1, 12_345)


def _test_large_unsorted() -> None:
    """The same staircase shuffled: input order must not matter."""
    n = 100_000
    k = 500
    sessions = [(i, i + k) for i in range(n)]
    rng = _Lcg(0xF1BBCDCB7A0EA9C7)
    for i in range(n - 1, 0, -1):
        j = rng.below(i + 1)
        sessions[i], sessions[j] = sessions[j], sessions[i]
    assert peak_concurrency(sessions) == (k, k - 1)


if __name__ == "__main__":
    _test_examples()
    _test_half_open()
    _test_zero_length()
    _test_edges()
    _test_random_against_brute()
    _test_random_clustered()
    _test_large_all_overlapping()
    _test_large_handover_chain()
    _test_large_staircase()
    _test_large_nested()
    _test_large_disjoint()
    _test_large_zero_length_flood()
    _test_large_unsorted()
    print("All tests passed.")
