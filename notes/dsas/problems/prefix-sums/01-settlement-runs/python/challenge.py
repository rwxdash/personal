"""
Settlement Runs
problems/prefix-sums/01-settlement-runs

Fill in `settlement_runs`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Tuple


def settlement_runs(deltas: List[int], m: int) -> Tuple[int, int]:
    """Count settleable runs and measure the longest one.

    A contiguous run of `deltas` is settleable when its sum is an exact multiple
    of `m`. Zero counts (it is a multiple of everything), and negative sums
    count when their magnitude is a multiple of `m`.

    Runs are identified by position: deltas[3..7] and deltas[4..8] are distinct
    even if they hold the same values.

    Args:
        deltas: Chronological balance deltas; negative for refunds and
                chargebacks. 0 <= n <= 200_000, each value -10^9 ..= 10^9.
        m:      Settlement unit, 1 <= m <= 10^9.

    Returns:
        (count, longest_length). `count` can reach ~2 * 10^10.
        `longest_length` is 0 when no run settles.

    Required: O(n) time expected, O(n) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# Large cases use ledgers whose answers are known in closed form, so a pass is
# real evidence rather than a self-consistency check.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(deltas: List[int], m: int) -> Tuple[int, int]:
    """Obviously-correct O(n^2) checker. Hopeless at the real bounds."""
    count = 0
    longest = 0
    for i in range(len(deltas)):
        total = 0
        for j in range(i, len(deltas)):
            total += deltas[j]
            if total % m == 0:
                count += 1
                if j - i + 1 > longest:
                    longest = j - i + 1
    return (count, longest)


def _test_examples() -> None:
    assert settlement_runs([2, -2, 3], 3) == (3, 3)
    assert settlement_runs([-1, -2], 3) == (1, 2)
    assert settlement_runs([5, -3, 7], 1) == (6, 3)
    assert settlement_runs([1, 1], 5) == (0, 0)


def _test_edges() -> None:
    # Empty ledger.
    assert settlement_runs([], 5) == (0, 0)
    assert settlement_runs([], 1) == (0, 0)
    # Single transaction, settleable or not.
    assert settlement_runs([4], 2) == (1, 1)
    assert settlement_runs([3], 2) == (0, 0)
    # Zero is a multiple of everything.
    assert settlement_runs([0], 7) == (1, 1)
    assert settlement_runs([0, 0, 0], 7) == (6, 3)
    # Negative single values.
    assert settlement_runs([-6], 3) == (1, 1)
    assert settlement_runs([-5], 3) == (0, 0)
    # A run summing to exactly zero settles under any unit.
    assert settlement_runs([7, -7], 10**9) == (1, 2)
    # Mixed signs where only the full ledger settles.
    assert settlement_runs([1, -4, 3], 100) == (1, 3)
    # The longest run appears early; later runs are shorter. A `longest` that
    # tracks the most recent match rather than the maximum fails here.
    assert settlement_runs([3, 3, 1, 1, 3], 3) == (4, 2)
    # Every prefix settles: count is triangular, longest is n.
    assert settlement_runs([5, 5, 5, 5], 5) == (10, 4)


def _test_negative_modulo() -> None:
    """Residues of negative running totals must be normalised into 0..m-1.

    In Rust `-1 % 3 == -1`, so keying a map on the raw remainder splits one
    residue class across two buckets and loses runs. Python's `%` already
    normalises, but the same logic error appears if a candidate hand-rolls it.
    """
    assert settlement_runs([-1, -2], 3) == (1, 2)
    assert settlement_runs([-3, -3, -3], 3) == (6, 3)
    assert settlement_runs([-1, 4, -3], 3) == (3, 3)
    # Running total dips negative then recovers to the same residue.
    assert settlement_runs([-5, 5], 5) == (3, 2)
    assert settlement_runs([2, -5, 3], 5) == (2, 3)


def _test_random_against_brute() -> None:
    rng = _Lcg(0x452821E638D01377)
    for _ in range(400):
        n = rng.below(25)
        m = 1 + rng.below(7)
        # Deliberately signed, and biased toward small magnitudes so runs
        # actually land on multiples.
        deltas = [rng.below(21) - 10 for _ in range(n)]
        got = settlement_runs(deltas, m)
        want = _brute(deltas, m)
        assert got == want, "deltas=%r m=%d got=%r want=%r" % (deltas, m, got, want)


def _test_random_large_modulus() -> None:
    """Same cross-check with m far larger than any reachable sum."""
    rng = _Lcg(0xBE5466CF34E90C6C)
    for _ in range(200):
        n = rng.below(18)
        deltas = [rng.below(11) - 5 for _ in range(n)]
        m = 10**9
        got = settlement_runs(deltas, m)
        want = _brute(deltas, m)
        assert got == want, "deltas=%r m=%d got=%r want=%r" % (deltas, m, got, want)


def _test_large_all_zero() -> None:
    """Every run settles: count is triangular and needs 64 bits."""
    n = 200_000
    deltas = [0] * n
    expected = n * (n + 1) // 2  # 20_000_100_000
    assert settlement_runs(deltas, 7) == (expected, n)
    assert expected > 2**31, "this case exists to overflow 32-bit counters"


def _test_large_unit_modulus() -> None:
    """m = 1 makes every run settleable regardless of the values."""
    n = 200_000
    rng = _Lcg(0x3F84D5B5B5470917)
    deltas = [rng.below(2 * 10**9 + 1) - 10**9 for _ in range(n)]
    assert settlement_runs(deltas, 1) == (n * (n + 1) // 2, n)


def _test_large_single_run() -> None:
    """Exactly one settleable run, spanning the whole ledger.

    Prefix totals are 0, 1, 2, ..., n and the modulus is n, so the only repeated
    residue is 0 (at the empty prefix and at the end).
    """
    n = 200_000
    deltas = [1] * n
    assert settlement_runs(deltas, n) == (1, n)


def _test_large_periodic() -> None:
    """Prefix residues cycle with period `p`, so residues split into p classes.

    With n divisible by p, each of the p residue classes is hit exactly n/p
    times across the n+1 prefixes -- except class 0, which is hit n/p + 1 times
    because of the empty prefix. Count is the sum of C(k, 2) over classes.
    """
    n = 200_000
    p = 4
    deltas = [1] * n
    hits_zero = n // p + 1
    hits_other = n // p
    expected = hits_zero * (hits_zero - 1) // 2 + (p - 1) * (
        hits_other * (hits_other - 1) // 2
    )
    assert settlement_runs(deltas, p) == (expected, n)


def _test_large_alternating() -> None:
    """Alternating +v / -v: prefix residues toggle between two classes."""
    n = 200_000
    v = 10**9
    deltas = [v if i % 2 == 0 else -v for i in range(n)]
    # Prefix totals: 0, v, 0, v, ... -> residue 0 appears at every even prefix
    # index (n/2 + 1 of them), residue v mod m at every odd one (n/2 of them).
    m = 7
    half = n // 2
    zeros = half + 1
    others = half
    expected = zeros * (zeros - 1) // 2 + others * (others - 1) // 2
    assert settlement_runs(deltas, m) == (expected, n)


def _test_large_no_settlement() -> None:
    """A ledger where nothing settles: strictly increasing residues."""
    n = 100_000
    deltas = [1] * n
    # Prefix totals 0..n are all distinct residues mod m when m > n.
    assert settlement_runs(deltas, n + 1) == (0, 0)


if __name__ == "__main__":
    _test_examples()
    _test_edges()
    _test_negative_modulo()
    _test_random_against_brute()
    _test_random_large_modulus()
    _test_large_all_zero()
    _test_large_unit_modulus()
    _test_large_single_run()
    _test_large_periodic()
    _test_large_alternating()
    _test_large_no_settlement()
    print("All tests passed.")
