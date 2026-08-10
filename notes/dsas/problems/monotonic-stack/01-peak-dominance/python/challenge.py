"""
Peak Dominance
problems/monotonic-stack/01-peak-dominance

Fill in `dominance_spans`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List


def dominance_spans(load: List[int]) -> List[int]:
    """For each sample, the width of the widest stretch it dominates.

    A sample dominates a contiguous stretch if it lies inside that stretch and
    no reading in the stretch is strictly higher than it. Ties count as
    dominated -- only a strictly greater reading ends a sample's reign.

    Equivalently, for each index i: find the nearest strictly greater value to
    the left and to the right; the answer is the number of samples strictly
    between them.

    Args:
        load: Bandwidth readings in time order. 0 <= n <= 200_000, each
              value 0..=10^9. Repeated values are common, not exceptional.

    Returns:
        A list of length n. Every entry is at least 1, since a sample always
        dominates at least itself.

    Required: O(n) time, O(n) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# Large cases use series whose spans are known in closed form.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(load: List[int]) -> List[int]:
    """Expand outward from every index. O(n^2); hopeless at the real bounds."""
    n = len(load)
    out = []
    for i in range(n):
        lo = i
        while lo - 1 >= 0 and load[lo - 1] <= load[i]:
            lo -= 1
        hi = i
        while hi + 1 < n and load[hi + 1] <= load[i]:
            hi += 1
        out.append(hi - lo + 1)
    return out


def _test_examples() -> None:
    assert dominance_spans([2, 1, 3]) == [2, 1, 3]
    assert dominance_spans([5, 5, 5]) == [3, 3, 3]
    assert dominance_spans([1, 2, 5, 2, 1]) == [1, 2, 5, 2, 1]
    assert dominance_spans([4, 1, 4]) == [3, 1, 3]


def _test_edges() -> None:
    # Empty and singleton.
    assert dominance_spans([]) == []
    assert dominance_spans([42]) == [1]
    assert dominance_spans([0]) == [1]
    # Strictly increasing: each element is blocked only by the one after it.
    assert dominance_spans([1, 2, 3]) == [1, 2, 3]
    # Strictly decreasing: mirror image.
    assert dominance_spans([3, 2, 1]) == [3, 2, 1]
    # Two equal values dominate each other.
    assert dominance_spans([7, 7]) == [2, 2]
    # A plateau flanked by higher ground.
    assert dominance_spans([9, 4, 4, 9]) == [4, 2, 2, 4]
    # A plateau flanked by lower ground.
    assert dominance_spans([1, 4, 4, 1]) == [1, 4, 4, 1]
    # Zeros are ordinary values.
    assert dominance_spans([0, 0, 0]) == [3, 3, 3]
    # Value range boundaries.
    assert dominance_spans([0, 10**9]) == [1, 2]
    assert dominance_spans([10**9, 0]) == [2, 1]
    # Alternating. Both 5s are joint maxima, and since ties do not break a
    # reign, neither blocks the other -- each dominates the WHOLE series, not
    # just its immediate neighbours.
    assert dominance_spans([1, 5, 1, 5, 1]) == [1, 5, 1, 5, 1]


def _test_ties_matter() -> None:
    """Equal values must NOT end a reign.

    Every assert here changes if the stack pops on `<` instead of `<=`, which
    is the single most common way this problem is got wrong.
    """
    assert dominance_spans([5, 5, 5]) == [3, 3, 3]
    assert dominance_spans([2, 2, 1, 2, 2]) == [5, 5, 1, 5, 5]
    assert dominance_spans([3, 1, 3, 1, 3]) == [5, 1, 5, 1, 5]
    assert dominance_spans([6, 6, 7]) == [2, 2, 3]
    assert dominance_spans([7, 6, 6]) == [3, 2, 2]


def _test_random_against_brute() -> None:
    rng = _Lcg(0xC0AC29B7C97C50DD)
    for _ in range(400):
        n = rng.below(26)
        # Tiny alphabet on purpose: forces constant ties, which is where the
        # strict/non-strict comparison bugs live.
        spread = 1 + rng.below(4)
        load = [rng.below(spread) for _ in range(n)]
        got = dominance_spans(load)
        want = _brute(load)
        assert got == want, "load=%r got=%r want=%r" % (load, got, want)


def _test_random_wide_alphabet() -> None:
    """Same cross-check with mostly-distinct values."""
    rng = _Lcg(0x9216D5D98979FB1B)
    for _ in range(300):
        n = rng.below(30)
        load = [rng.below(10**9) for _ in range(n)]
        assert dominance_spans(load) == _brute(load)


def _test_large_flat() -> None:
    """200k identical readings: every sample dominates the whole series."""
    n = 200_000
    assert dominance_spans([17] * n) == [n] * n


def _test_large_increasing() -> None:
    """Strictly increasing: span[i] = i + 1. Worst case for a left-scan naive."""
    n = 200_000
    assert dominance_spans(list(range(n))) == list(range(1, n + 1))


def _test_large_decreasing() -> None:
    """Strictly decreasing: span[i] = n - i."""
    n = 200_000
    assert dominance_spans(list(range(n, 0, -1))) == list(range(n, 0, -1))


def _test_large_mountain() -> None:
    """Up then down; the apex dominates everything, the flanks mirror."""
    half = 100_000
    load = list(range(half)) + list(range(half, 0, -1))
    n = len(load)
    spans = dominance_spans(load)
    assert len(spans) == n
    assert spans[half] == n, "the apex dominates the whole series"
    # Left flank i (value i) is blocked on the right by i+1 and unblocked left.
    assert spans[0] == 1
    assert spans[half - 1] == half
    # Every span is at least 1 and at most n.
    assert min(spans) >= 1 and max(spans) == n


def _test_large_plateaus() -> None:
    """Blocks of equal values with strictly increasing block heights.

    Inside block b (0-indexed, height b), every element is dominated only by
    later, taller blocks -- so its span reaches back to the start of the series
    and forward to the end of its own block.
    """
    block = 500
    blocks = 400
    n = block * blocks
    load = [(i // block) for i in range(n)]
    spans = dominance_spans(load)
    for b in range(blocks):
        expected = (b + 1) * block  # everything from index 0 through this block
        for k in (0, block // 2, block - 1):
            assert spans[b * block + k] == expected, (b, k, spans[b * block + k])


def _test_large_sawtooth() -> None:
    """Alternating low/high, at scale.

    Every 1 is a joint maximum, and ties do not break a reign, so each of the
    100_000 peaks dominates the entire series. Each 0 is boxed in by strictly
    greater neighbours and dominates only itself. An implementation that pops
    on `<` instead of `<=` reports 3 for the peaks here instead of n.
    """
    n = 200_000
    load = [0 if i % 2 == 0 else 1 for i in range(n)]
    spans = dominance_spans(load)
    assert all(spans[i] == 1 for i in range(0, n, 2)), "valleys dominate only themselves"
    assert all(spans[i] == n for i in range(1, n, 2)), "joint maxima dominate everything"


if __name__ == "__main__":
    _test_examples()
    _test_edges()
    _test_ties_matter()
    _test_random_against_brute()
    _test_random_wide_alphabet()
    _test_large_flat()
    _test_large_increasing()
    _test_large_decreasing()
    _test_large_mountain()
    _test_large_plateaus()
    _test_large_sawtooth()
    print("All tests passed.")
