"""
Extent Cover
problems/bit-manipulation/01b-extent-cover

Fill in `extent_cover`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Tuple


def extent_cover(ranges: List[Tuple[int, int]]) -> List[List[Tuple[int, int]]]:
    """Fewest naturally-aligned extents that exactly cover each byte range.

    An extent (offset, size) is legal when size is a power of two, offset is a
    multiple of size, and the extent lies entirely inside the request.

    Args:
        ranges: (start, end) byte requests, inclusive, with start <= end and
                0 <= start <= end < 2**48. Up to 50_000 requests.

    Returns:
        One list per request: the smallest possible set of legal extents whose
        covered bytes are exactly start..end, sorted by offset ascending.
        No request needs more than 94 extents.

    Required: O(total extents returned) time, O(1) space beyond the output.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check finds the minimum by shortest path over positions,
# which assumes nothing about which extent to pick at each step.

from collections import deque  # noqa: E402  (kept with the tests it belongs to)

LIMIT = 1 << 48
MAX_BYTE = LIMIT - 1


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _check_contract(start: int, end: int, extents: List[Tuple[int, int]]) -> None:
    """Every rule from the statement, checked independently of any algorithm."""
    assert extents, "empty cover for (%d, %d)" % (start, end)
    pos = start
    for off, size in extents:
        assert size > 0 and size & (size - 1) == 0, (
            "size %d is not a power of two, range=(%d, %d)" % (size, start, end)
        )
        assert off % size == 0, (
            "offset %d not aligned to size %d, range=(%d, %d)" % (off, size, start, end)
        )
        assert off == pos, (
            "gap or overlap at %d (expected %d), range=(%d, %d)" % (off, pos, start, end)
        )
        assert off + size - 1 <= end, (
            "extent (%d, %d) spills past %d" % (off, size, end)
        )
        pos = off + size
    assert pos == end + 1, (
        "cover stops at %d, range=(%d, %d)" % (pos - 1, start, end)
    )


def _no_mergeable_pair(extents: List[Tuple[int, int]]) -> None:
    """A minimal cover can never hold two adjacent equal extents that combine."""
    for (o1, s1), (o2, s2) in zip(extents, extents[1:]):
        assert not (s1 == s2 and o1 % (2 * s1) == 0), (
            "(%d, %d) and (%d, %d) merge into one extent" % (o1, s1, o2, s2)
        )


def _brute_min(start: int, end: int) -> int:
    """Fewest extents, by shortest path from start to end+1 over legal steps."""
    target = end + 1
    dist = {start: 0}
    queue = deque([start])
    while queue:
        p = queue.popleft()
        if p == target:
            return dist[p]
        k = 0
        while True:
            size = 1 << k
            if p % size != 0 or p + size > target:
                break
            if p + size not in dist:
                dist[p + size] = dist[p] + 1
                queue.append(p + size)
            k += 1
    raise AssertionError("no cover for (%d, %d)" % (start, end))


def _test_examples() -> None:
    assert extent_cover([(0, 7)]) == [[(0, 8)]]
    assert extent_cover([(1, 6)]) == [[(1, 1), (2, 2), (4, 2), (6, 1)]]
    assert extent_cover([(4, 11)]) == [[(4, 4), (8, 4)]]
    assert extent_cover([(5, 5)]) == [[(5, 1)]]
    assert extent_cover([(0, MAX_BYTE)]) == [[(0, LIMIT)]]


def _test_zero_start() -> None:
    """Position 0 has no alignment limit -- only the remaining length caps it."""
    assert extent_cover([(0, 0)]) == [[(0, 1)]]
    assert extent_cover([(0, 1)]) == [[(0, 2)]]
    assert extent_cover([(0, 2)]) == [[(0, 2), (2, 1)]]
    assert extent_cover([(0, 1023)]) == [[(0, 1024)]]
    # The widest possible request, where the size reaches 2**48.
    assert extent_cover([(0, MAX_BYTE)]) == [[(0, LIMIT)]]


def _test_single_byte() -> None:
    assert extent_cover([(3, 3)]) == [[(3, 1)]]
    assert extent_cover([(MAX_BYTE, MAX_BYTE)]) == [[(MAX_BYTE, 1)]]
    for p in (1, 2, 7, 1 << 31, (1 << 47) + 5):
        assert extent_cover([(p, p)]) == [[(p, 1)]]


def _test_whole_blocks() -> None:
    """A range that already IS one aligned block must come back as one extent."""
    for k in range(0, 49):
        size = 1 << k
        offset = 0 if k >= 47 else 3 * size
        assert extent_cover([(offset, offset + size - 1)]) == [[(offset, size)]]


def _test_which_limit_binds() -> None:
    # Alignment binds: 2 cannot host a 4-byte extent even though 4 fit.
    assert extent_cover([(2, 5)]) == [[(2, 2), (4, 2)]]
    assert extent_cover([(6, 9)]) == [[(6, 2), (8, 2)]]
    # Fit binds: 8 could host a 8-byte extent but only 2 bytes remain.
    assert extent_cover([(8, 15)]) == [[(8, 8)]]
    assert extent_cover([(7, 8)]) == [[(7, 1), (8, 1)]]
    # Both bind in turn -- sizes grow, then shrink.
    assert extent_cover([(5, 20)]) == [
        [(5, 1), (6, 2), (8, 8), (16, 4), (20, 1)]
    ]
    assert extent_cover([(1024, 3071)]) == [[(1024, 1024), (2048, 1024)]]


def _test_edges() -> None:
    assert extent_cover([]) == []
    # Several requests in one call, answered independently and in order.
    assert extent_cover([(0, 7), (1, 6), (5, 5)]) == [
        [(0, 8)],
        [(1, 1), (2, 2), (4, 2), (6, 1)],
        [(5, 1)],
    ]
    # Repeated identical requests.
    assert extent_cover([(4, 11)] * 3) == [[(4, 4), (8, 4)]] * 3


def _test_worst_case_shape() -> None:
    """The most fragmented request in the whole address space: 94 extents."""
    got = extent_cover([(1, MAX_BYTE - 1)])[0]
    assert len(got) == 94, "expected 94 extents, got %d" % len(got)
    assert got[0] == (1, 1)
    assert got[-1] == (MAX_BYTE - 1, 1)
    _check_contract(1, MAX_BYTE - 1, got)
    _no_mergeable_pair(got)


def _test_random_against_brute() -> None:
    """Small ranges checked against a shortest-path minimum."""
    rng = _Lcg(0x452821E638D01377)
    for _ in range(400):
        start = rng.below(300)
        end = start + rng.below(120)
        got = extent_cover([(start, end)])[0]
        _check_contract(start, end, got)
        want = _brute_min(start, end)
        assert len(got) == want, (
            "range=(%d, %d) used %d extents, minimum is %d" % (start, end, len(got), want)
        )


def _test_random_contract() -> None:
    """Full-width random ranges: contract plus a necessary minimality check."""
    rng = _Lcg(0xBE5466CF34E90C6C)
    for _ in range(2_000):
        a = rng.below(LIMIT)
        b = rng.below(LIMIT)
        start, end = (a, b) if a <= b else (b, a)
        got = extent_cover([(start, end)])[0]
        _check_contract(start, end, got)
        _no_mergeable_pair(got)
        assert len(got) <= 94


def _test_large_wide_ranges() -> None:
    """5 000 requests spanning the 2**48 space -- fatal to anything per-byte."""
    rng = _Lcg(0x243F6A8885A308D3)
    ranges = []
    for _ in range(5_000):
        a = rng.below(LIMIT)
        b = rng.below(LIMIT)
        ranges.append((a, b) if a <= b else (b, a))
    out = extent_cover(ranges)
    assert len(out) == len(ranges)
    total = 0
    for (start, end), extents in zip(ranges, out):
        _check_contract(start, end, extents)
        _no_mergeable_pair(extents)
        total += len(extents)
    assert total == 229_651, "total extents %d, expected 229_651" % total


def _test_large_batch() -> None:
    """50 000 requests at the stated input bound."""
    rng = _Lcg(0x13198A2E03707344)
    ranges = []
    for _ in range(50_000):
        start = rng.below(LIMIT)
        end = min(start + rng.below(1024), MAX_BYTE)
        ranges.append((start, end))
    out = extent_cover(ranges)
    assert len(out) == len(ranges)
    total = 0
    for (start, end), extents in zip(ranges, out):
        _check_contract(start, end, extents)
        total += len(extents)
    assert total == 450_115, "total extents %d, expected 450_115" % total


if __name__ == "__main__":
    _test_examples()
    _test_zero_start()
    _test_single_byte()
    _test_whole_blocks()
    _test_which_limit_binds()
    _test_edges()
    _test_worst_case_shape()
    _test_random_against_brute()
    _test_random_contract()
    _test_large_wide_ranges()
    _test_large_batch()
    print("All tests passed.")
