"""
CIDR Aggregation
problems/bit-manipulation/01-cidr-aggregation

Fill in `smallest_cidr`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Tuple


def smallest_cidr(ranges: List[Tuple[int, int]]) -> List[Tuple[int, int, bool]]:
    """Smallest CIDR block containing each address range.

    A block with prefix length p covers the 2^(32-p) addresses sharing the same
    top p bits.

    Args:
        ranges: (start, end) pairs with start <= end, each in
                0 ..= 4_294_967_295. Up to 200_000 ranges.

    Returns:
        One (network, prefix_len, exact) per range:
          network    -- network address of the smallest containing block
          prefix_len -- that block's prefix length, 0 to 32
          exact      -- True if the block covers exactly the requested range

    Required: O(len(ranges)) time, O(1) space beyond the output.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check derives the answer from binary string prefixes, which
# shares no arithmetic with an XOR-and-mask solution.

MAX_ADDR = 4_294_967_295


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(ranges: List[Tuple[int, int]]) -> List[Tuple[int, int, bool]]:
    """Count common leading bits by comparing 32-character binary strings."""
    out = []
    for start, end in ranges:
        a = format(start, "032b")
        b = format(end, "032b")
        common = 0
        while common < 32 and a[common] == b[common]:
            common += 1
        free = 32 - common
        network = int(a[:common] + "0" * free, 2) if common else 0
        size = 1 << free
        exact = network == start and network + size - 1 == end
        out.append((network, common, exact))
    return out


def _test_examples() -> None:
    assert smallest_cidr([(0, 255)]) == [(0, 24, True)]
    assert smallest_cidr([(5, 9)]) == [(0, 28, False)]
    assert smallest_cidr([(42, 42)]) == [(42, 32, True)]
    assert smallest_cidr([(0, MAX_ADDR)]) == [(0, 0, True)]
    assert smallest_cidr([(255, 256)]) == [(0, 23, False)]


def _test_boundary_prefixes() -> None:
    """The /32 and /0 ends of the range, where the arithmetic is easiest to
    invert or overflow."""
    # /32: a single address, at both ends of the space.
    assert smallest_cidr([(0, 0)]) == [(0, 32, True)]
    assert smallest_cidr([(MAX_ADDR, MAX_ADDR)]) == [(MAX_ADDR, 32, True)]
    # /0: differing in the top bit is enough to force the widest block.
    assert smallest_cidr([(0, 2**31)]) == [(0, 0, False)]
    assert smallest_cidr([(2**31 - 1, 2**31)]) == [(0, 0, False)]
    # /31: two adjacent addresses that share all but the last bit.
    assert smallest_cidr([(0, 1)]) == [(0, 31, True)]
    assert smallest_cidr([(2, 3)]) == [(2, 31, True)]
    # ...but shifted by one, they no longer align.
    assert smallest_cidr([(1, 2)]) == [(0, 30, False)]
    # /1: the top half of the space.
    assert smallest_cidr([(2**31, MAX_ADDR)]) == [(2**31, 1, True)]
    assert smallest_cidr([(0, 2**31 - 1)]) == [(0, 1, True)]


def _test_exactness() -> None:
    """`exact` needs BOTH ends to line up, not just the start."""
    # Aligned start, short end: block is wider than the range.
    assert smallest_cidr([(0, 100)]) == [(0, 25, False)]
    # Aligned start and end: exact.
    assert smallest_cidr([(0, 127)]) == [(0, 25, True)]
    # Start aligned but end one short.
    assert smallest_cidr([(0, 126)]) == [(0, 25, False)]
    # Start one late, end aligned.
    assert smallest_cidr([(1, 127)]) == [(0, 25, False)]
    # Neither aligned.
    assert smallest_cidr([(5, 6)]) == [(4, 30, False)]
    # A block high in the space.
    assert smallest_cidr([(4_294_967_040, MAX_ADDR)]) == [
        (4_294_967_040, 24, True)
    ]


def _test_edges() -> None:
    # No ranges at all.
    assert smallest_cidr([]) == []
    # Several ranges in one call, answered independently.
    assert smallest_cidr([(0, 255), (5, 9), (42, 42)]) == [
        (0, 24, True),
        (0, 28, False),
        (42, 32, True),
    ]
    # Repeated identical ranges.
    assert smallest_cidr([(10, 20)] * 4) == [(0, 27, False)] * 4
    # Common real-world blocks.
    assert smallest_cidr([(167772160, 184549375)]) == [(167772160, 8, True)]
    assert smallest_cidr([(3232235520, 3232301055)]) == [(3232235520, 16, True)]


def _test_random_against_brute() -> None:
    rng = _Lcg(0x9159015A3070DD17)
    for _ in range(2_000):
        a = rng.below(1 << 32)
        b = rng.below(1 << 32)
        start, end = (a, b) if a <= b else (b, a)
        got = smallest_cidr([(start, end)])
        want = _brute([(start, end)])
        assert got == want, "range=(%d, %d) got=%r want=%r" % (start, end, got, want)


def _test_random_aligned_blocks() -> None:
    """Ranges that ARE blocks, so `exact` should be True every time."""
    rng = _Lcg(0x152FECD8F70E5939)
    for _ in range(2_000):
        p = rng.below(33)  # prefix length 0..32
        free = 32 - p
        size = 1 << free
        # Pick an aligned network address for this prefix length.
        blocks = 1 << p
        idx = rng.below(blocks) if blocks > 0 else 0
        network = idx * size
        start = network
        end = network + size - 1
        got = smallest_cidr([(start, end)])
        assert got == [(network, p, True)], (
            "aligned block p=%d network=%d got=%r" % (p, network, got)
        )
        assert got == _brute([(start, end)])


def _test_random_near_misses() -> None:
    """Aligned blocks shrunk or shifted by one, so `exact` must be False."""
    rng = _Lcg(0x67332667FFC00B31)
    for _ in range(2_000):
        p = 1 + rng.below(31)  # 1..31, so the block has room to shrink
        free = 32 - p
        size = 1 << free
        blocks = 1 << p
        network = rng.below(blocks) * size
        if size < 3:
            continue
        start = network
        end = network + size - 2  # one short of the block's end
        got = smallest_cidr([(start, end)])
        want = _brute([(start, end)])
        assert got == want, "p=%d network=%d got=%r want=%r" % (p, network, got, want)
        assert got[0][2] is False


def _test_large_batch() -> None:
    """200k ranges answered in one call."""
    n = 200_000
    rng = _Lcg(0x8EB44A8768581511)
    ranges = []
    for _ in range(n):
        a = rng.below(1 << 32)
        b = rng.below(1 << 32)
        ranges.append((a, b) if a <= b else (b, a))
    got = smallest_cidr(ranges)
    assert len(got) == n
    # Verify the contract on every result rather than recomputing it: the block
    # must contain the range, and halving the prefix length must not.
    for (start, end), (network, p, exact) in zip(ranges, got):
        size = 1 << (32 - p)
        assert network % size == 0, "network must be aligned to its own block"
        assert network <= start and end <= network + size - 1, "must contain the range"
        if p < 32:
            # One bit longer would be a block half the size; it must fail.
            half = size >> 1
            tighter = start - (start % half)
            assert not (tighter <= start and end <= tighter + half - 1), "not smallest"
        assert exact == (network == start and network + size - 1 == end)


def _test_large_full_space() -> None:
    """Many copies of the widest possible range: the /0 overflow case at scale."""
    n = 100_000
    ranges = [(0, MAX_ADDR)] * n
    assert smallest_cidr(ranges) == [(0, 0, True)] * n


def _test_large_singletons() -> None:
    """200k single-address ranges: the /32 case at scale."""
    n = 200_000
    ranges = [(i, i) for i in range(n)]
    assert smallest_cidr(ranges) == [(i, 32, True) for i in range(n)]


def _test_large_widest_ranges() -> None:
    """Ranges spanning billions of addresses: walking them is impossible."""
    ranges = [
        (0, MAX_ADDR),
        (1, MAX_ADDR),
        (0, MAX_ADDR - 1),
        (2**31, MAX_ADDR),
        (2**31 - 1, 2**31),
    ]
    assert smallest_cidr(ranges) == [
        (0, 0, True),
        (0, 0, False),
        (0, 0, False),
        (2**31, 1, True),
        (0, 0, False),
    ]


if __name__ == "__main__":
    _test_examples()
    _test_boundary_prefixes()
    _test_exactness()
    _test_edges()
    _test_random_against_brute()
    _test_random_aligned_blocks()
    _test_random_near_misses()
    _test_large_batch()
    _test_large_full_space()
    _test_large_singletons()
    _test_large_widest_ranges()
    print("All tests passed.")
