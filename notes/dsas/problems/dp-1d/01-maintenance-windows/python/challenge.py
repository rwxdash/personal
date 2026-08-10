"""
Maintenance Windows
problems/dp-1d/01-maintenance-windows

Fill in `max_maintenance_value`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List


def max_maintenance_value(
    value: List[int], blackout: List[bool], cooldown: int
) -> int:
    """Maximum total value from scheduling maintenance under a cooldown.

    Choose a set of slots such that no chosen slot is a blackout slot, and any
    two chosen slots i < j satisfy j - i > cooldown. Maximise the sum of the
    chosen slots' values. Choosing nothing is allowed.

    Args:
        value:    Value of running maintenance in each slot.
                  0 <= n <= 200_000, each value 0 ..= 10^9.
        blackout: Whether each slot is unavailable. Same length as `value`.
        cooldown: Slots that must sit empty between two runs.
                  0 ..= 200_000; may exceed n. 0 means no settling period.

    Returns:
        The maximum total value, or 0 if nothing can be chosen. Can reach
        2 * 10^14.

    Required: O(n) time, O(n) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check enumerates every subset of slots, which shares no
# reasoning with a left-to-right sweep.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(value: List[int], blackout: List[bool], cooldown: int) -> int:
    """Try every subset of slots. O(2^n * n); only viable for tiny n."""
    n = len(value)
    best = 0
    for mask in range(1 << n):
        chosen = [i for i in range(n) if mask & (1 << i)]
        if any(blackout[i] for i in chosen):
            continue
        ok = all(
            chosen[t + 1] - chosen[t] > cooldown for t in range(len(chosen) - 1)
        )
        if ok:
            best = max(best, sum(value[i] for i in chosen))
    return best


def _test_examples() -> None:
    assert max_maintenance_value([5, 1, 8, 4], [False] * 4, 1) == 13
    assert max_maintenance_value([5, 1, 8, 4], [False, False, True, False], 1) == 9
    assert max_maintenance_value([5, 1, 8, 4], [False] * 4, 2) == 9
    assert max_maintenance_value([5, 1, 8, 4], [False] * 4, 0) == 18


def _test_greedy_is_wrong() -> None:
    """Taking the largest available slot first gives the wrong answer here.

    Greedy picks the 5 in the middle, blocking both 4s, for 5. The right answer
    takes both 4s.
    """
    assert max_maintenance_value([4, 5, 4], [False] * 3, 1) == 8
    assert max_maintenance_value([3, 10, 3, 3, 10, 3], [False] * 6, 2) == 20
    # Greedy on the first: taking 6 blocks 5 and 5.
    assert max_maintenance_value([5, 6, 5], [False] * 3, 1) == 10


def _test_edges() -> None:
    # Empty input.
    assert max_maintenance_value([], [], 0) == 0
    assert max_maintenance_value([], [], 5) == 0
    # Single slot.
    assert max_maintenance_value([7], [False], 0) == 7
    assert max_maintenance_value([7], [False], 100) == 7
    assert max_maintenance_value([7], [True], 0) == 0
    # Everything blacked out.
    assert max_maintenance_value([5, 5, 5], [True] * 3, 0) == 0
    # All values zero: nothing to gain.
    assert max_maintenance_value([0, 0, 0], [False] * 3, 1) == 0
    # Cooldown larger than the array: at most one slot, so the largest.
    assert max_maintenance_value([3, 9, 4], [False] * 3, 3) == 9
    assert max_maintenance_value([3, 9, 4], [False] * 3, 1000) == 9
    # Cooldown exactly n - 1: still only one slot fits.
    assert max_maintenance_value([3, 9, 4], [False] * 3, 2) == 9
    # Cooldown 0: sum of every non-blackout slot.
    assert max_maintenance_value([1, 2, 3, 4], [False, True, False, False], 0) == 8
    # The best single slot is blacked out, so a pair of smaller ones wins.
    assert max_maintenance_value([4, 100, 4], [False, True, False], 1) == 8
    # Boundary: distance exactly cooldown is illegal, cooldown + 1 is legal.
    assert max_maintenance_value([5, 0, 5], [False] * 3, 2) == 5
    assert max_maintenance_value([5, 0, 0, 5], [False] * 4, 2) == 10
    # Value ceiling.
    assert max_maintenance_value([10**9, 10**9], [False] * 2, 0) == 2 * 10**9


def _test_random_against_brute() -> None:
    rng = _Lcg(0x428A2F98D728AE22)
    for _ in range(400):
        n = rng.below(13)
        value = [rng.below(20) for _ in range(n)]
        blackout = [rng.below(4) == 0 for _ in range(n)]
        cooldown = rng.below(5)
        got = max_maintenance_value(value, blackout, cooldown)
        want = _brute(value, blackout, cooldown)
        assert got == want, "value=%r blackout=%r cooldown=%d got=%d want=%d" % (
            value,
            blackout,
            cooldown,
            got,
            want,
        )


def _test_random_large_cooldown() -> None:
    """Same cross-check with cooldowns that often exceed the array length."""
    rng = _Lcg(0x7137449123EF65CD)
    for _ in range(300):
        n = rng.below(11)
        value = [rng.below(30) for _ in range(n)]
        blackout = [rng.below(5) == 0 for _ in range(n)]
        cooldown = rng.below(15)
        assert max_maintenance_value(value, blackout, cooldown) == _brute(
            value, blackout, cooldown
        )


def _test_large_no_cooldown() -> None:
    """200k slots, cooldown 0: the answer is the sum of everything."""
    n = 200_000
    value = [10**9] * n
    assert max_maintenance_value(value, [False] * n, 0) == n * 10**9


def _test_large_alternating() -> None:
    """Cooldown 1 on a uniform array: every other slot, so half of them."""
    n = 200_000
    value = [7] * n
    assert max_maintenance_value(value, [False] * n, 1) == (n // 2) * 7


def _test_large_all_blackout() -> None:
    n = 200_000
    assert max_maintenance_value([10**9] * n, [True] * n, 3) == 0


def _test_large_single_choice() -> None:
    """Cooldown at least n: only the single largest slot can be taken."""
    n = 200_000
    rng = _Lcg(0xB5C0FBCFEC4D3B2F)
    value = [rng.below(10**9) for _ in range(n)]
    assert max_maintenance_value(value, [False] * n, n) == max(value)
    assert max_maintenance_value(value, [False] * n, 200_000) == max(value)


def _test_large_spaced_peaks() -> None:
    """Tall peaks spaced exactly far enough apart to all be taken."""
    n = 200_000
    cooldown = 9
    value = [0] * n
    peaks = list(range(0, n, cooldown + 1))
    for p in peaks:
        value[p] = 1000
    # Every peak is exactly cooldown + 1 apart, so all of them fit.
    assert max_maintenance_value(value, [False] * n, cooldown) == len(peaks) * 1000


def _test_large_peaks_too_close() -> None:
    """The same peaks, one slot too close together: only every other one fits."""
    n = 200_000
    cooldown = 10  # peaks are 10 apart, which is NOT > 10
    value = [0] * n
    peaks = list(range(0, n, 10))
    for p in peaks:
        value[p] = 1000
    # Peaks 10 apart are illegal at cooldown 10, so take every second peak.
    expected = ((len(peaks) + 1) // 2) * 1000
    assert max_maintenance_value(value, [False] * n, cooldown) == expected


def _test_large_ramp() -> None:
    """Increasing values with cooldown 1: take alternate slots from the end."""
    n = 200_000
    value = list(range(1, n + 1))
    # Best is to take the largest and every other one going down: n, n-2, ...
    expected = sum(range(n, 0, -2))
    assert max_maintenance_value(value, [False] * n, 1) == expected


if __name__ == "__main__":
    _test_examples()
    _test_greedy_is_wrong()
    _test_edges()
    _test_random_against_brute()
    _test_random_large_cooldown()
    _test_large_no_cooldown()
    _test_large_alternating()
    _test_large_all_blackout()
    _test_large_single_choice()
    _test_large_spaced_peaks()
    _test_large_peaks_too_close()
    _test_large_ramp()
    print("All tests passed.")
