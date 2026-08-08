"""
Host Consolidation
problems/two-pointers/01-host-consolidation

Fill in `min_hosts`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List


def min_hosts(footprints: List[int], pinned: List[bool], cap: int) -> int:
    """Minimum number of hosts needed to place every VM.

    A host has capacity `cap` and may run at most two VMs; two VMs share a host
    only if their footprints sum to at most `cap`. A pinned VM always occupies a
    host by itself.

    Args:
        footprints: Memory footprint per VM. 0 <= n <= 200_000, each 1..=10^9.
                    Guaranteed footprints[i] <= cap, so a placement exists.
        pinned:     Whether each VM must occupy a host alone. Same length.
        cap:        Host memory capacity, 1..=10^9.

    Returns:
        The minimum host count. Zero VMs need zero hosts.

    Required: O(n log n) time, O(n) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check solves the same instances by exhaustive bitmask
# matching, which shares no reasoning with the intended approach -- so it also
# doubles as a proof that the greedy rule is actually optimal.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(footprints: List[int], pinned: List[bool], cap: int) -> int:
    """Exponential exact answer via maximum matching over compatible pairs.

    hosts = pinned_count + (unpinned_count - max_pairs). Only viable for tiny n.
    """
    free = [f for f, p in zip(footprints, pinned) if not p]
    n = len(free)
    memo = {}

    def max_pairs(mask: int) -> int:
        if mask == 0:
            return 0
        if mask in memo:
            return memo[mask]
        i = (mask & -mask).bit_length() - 1  # lowest set bit
        rest = mask ^ (1 << i)
        best = max_pairs(rest)  # leave VM i alone
        j = 0
        m = rest
        while m:
            if m & 1 and free[i] + free[j] <= cap:
                best = max(best, 1 + max_pairs(rest ^ (1 << j)))
            m >>= 1
            j += 1
        memo[mask] = best
        return best

    return sum(pinned) + n - max_pairs((1 << n) - 1)


def _test_examples() -> None:
    assert min_hosts([1, 2], [False, False], 3) == 1
    assert min_hosts([3, 2, 2, 1], [False] * 4, 3) == 3
    assert min_hosts([1, 1, 1], [True, False, False], 2) == 2
    assert min_hosts([4] * 6, [False] * 6, 7) == 6
    assert min_hosts([4] * 6, [False] * 6, 8) == 3


def _test_edges() -> None:
    # No VMs.
    assert min_hosts([], [], 5) == 0
    # One VM, pinned or not.
    assert min_hosts([5], [False], 5) == 1
    assert min_hosts([5], [True], 5) == 1
    # Every VM pinned: pairing is never allowed.
    assert min_hosts([1, 1, 1], [True, True, True], 10) == 3
    # Capacity exactly twice the footprint: everything pairs.
    assert min_hosts([4] * 6, [False] * 6, 8) == 3
    # Odd count that pairs perfectly except for one leftover.
    assert min_hosts([1, 1, 1], [False] * 3, 2) == 2
    # Largest VM fills a host alone; the rest pair up.
    assert min_hosts([5, 1, 1], [False] * 3, 5) == 2
    # A VM equal to cap can never share.
    assert min_hosts([3, 3, 3], [False] * 3, 3) == 3
    # Mixed pinning where the pinned one would otherwise have paired.
    assert min_hosts([1, 1], [True, False], 2) == 2
    assert min_hosts([1, 1], [False, False], 2) == 1
    # Pairing must be smallest-with-largest, not adjacent-in-input.
    assert min_hosts([5, 4, 3, 2], [False] * 4, 7) == 2


def _test_random_against_brute() -> None:
    rng = _Lcg(0x13198A2E03707344)
    for _ in range(300):
        n = rng.below(11)
        cap = 1 + rng.below(20)
        footprints = [1 + rng.below(cap) for _ in range(n)]
        pinned = [rng.below(4) == 0 for _ in range(n)]
        got = min_hosts(footprints, pinned, cap)
        want = _brute(footprints, pinned, cap)
        assert got == want, "footprints=%r pinned=%r cap=%d got=%d want=%d" % (
            footprints,
            pinned,
            cap,
            got,
            want,
        )


def _test_large_perfect_pairing() -> None:
    """200k VMs that pair exactly; answer is n / 2 by construction."""
    n = 200_000
    cap = 100
    footprints = [1 if i % 2 == 0 else 99 for i in range(n)]
    assert min_hosts(footprints, [False] * n, cap) == n // 2


def _test_large_nothing_pairs() -> None:
    """Every pair overflows by exactly one byte."""
    n = 200_000
    footprints = [60] * n
    assert min_hosts(footprints, [False] * n, 119) == n
    # One more byte of capacity and everything pairs.
    assert min_hosts(footprints, [False] * n, 120) == n // 2


def _test_large_all_pinned() -> None:
    n = 200_000
    assert min_hosts([1] * n, [True] * n, 10**9) == n


def _test_large_half_pinned() -> None:
    """Half pinned, half perfectly pairable."""
    n = 200_000
    footprints = [50] * n
    pinned = [i % 2 == 0 for i in range(n)]
    # n/2 pinned take one host each; the other n/2 pair up into n/4 hosts.
    assert min_hosts(footprints, pinned, 100) == n // 2 + n // 4


def _test_large_staircase() -> None:
    """Sizes 1..n with cap = n + 1: i pairs with n + 1 - i, so n/2 hosts."""
    n = 200_000
    footprints = list(range(1, n + 1))
    assert min_hosts(footprints, [False] * n, n + 1) == n // 2
    # cap = n leaves the largest VM unpaired, shifting the whole cascade.
    assert min_hosts(footprints, [False] * n, n) == n // 2 + 1


if __name__ == "__main__":
    _test_examples()
    _test_edges()
    _test_random_against_brute()
    _test_large_perfect_pairing()
    _test_large_nothing_pairs()
    _test_large_all_pinned()
    _test_large_half_pinned()
    _test_large_staircase()
    print("All tests passed.")
