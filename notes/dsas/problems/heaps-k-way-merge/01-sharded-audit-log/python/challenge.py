"""
Sharded Audit Log
problems/heaps-k-way-merge/01-sharded-audit-log

Fill in `nth_merged_event`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Optional, Tuple


def nth_merged_event(
    shards: List[List[int]], offset: int
) -> Optional[Tuple[int, int]]:
    """The event at `offset` in the globally merged view of all shards.

    Each shard is individually sorted non-decreasing. The merged order is by
    timestamp, and events sharing a timestamp are ordered by ascending shard
    index -- so a shard holding several events at the same timestamp emits all
    of them before any higher-numbered shard emits one.

    Args:
        shards: Up to 100_000 shards, each a sorted list of timestamps in
                0 ..= 10^18. Shards may be empty; the list itself may be empty.
                Total events across all shards is at most 2_000_000.
        offset: 0-indexed position in the merged view, 0 ..= 200_000.

    Returns:
        (timestamp, shard_index) at that position, or None if the shards hold
        `offset` or fewer events in total.

    Required: O(k + offset * log k) time, O(k) space. Note the cost must scale
    with `offset`, not with the total event count.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check materialises and sorts the whole merged view, which is
# exactly the approach the complexity bound forbids -- so it is an honest
# oracle rather than a re-implementation.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(
    shards: List[List[int]], offset: int
) -> Optional[Tuple[int, int]]:
    """Materialise the entire merged view and index into it. O(N log N)."""
    everything = []
    for i, shard in enumerate(shards):
        for value in shard:
            everything.append((value, i))
    everything.sort()
    if offset < len(everything):
        return everything[offset]
    return None


def _test_examples() -> None:
    assert nth_merged_event([[10, 40], [20, 30], [50]], 2) == (30, 1)
    assert nth_merged_event([[5, 5], [5], [5]], 1) == (5, 0)
    assert nth_merged_event([[], [7], [], [3, 9]], 0) == (3, 3)
    assert nth_merged_event([[1], [2]], 5) is None


def _test_full_traversal() -> None:
    """Walk one instance from offset 0 to past the end."""
    shards = [[10, 40], [20, 30], [50]]
    assert nth_merged_event(shards, 0) == (10, 0)
    assert nth_merged_event(shards, 1) == (20, 1)
    assert nth_merged_event(shards, 2) == (30, 1)
    assert nth_merged_event(shards, 3) == (40, 0)
    assert nth_merged_event(shards, 4) == (50, 2)
    assert nth_merged_event(shards, 5) is None


def _test_ties() -> None:
    """Ties break by ascending shard index, not round-robin."""
    shards = [[5, 5], [5], [5]]
    assert nth_merged_event(shards, 0) == (5, 0)
    assert nth_merged_event(shards, 1) == (5, 0), "shard 0 emits BOTH before shard 1"
    assert nth_merged_event(shards, 2) == (5, 1)
    assert nth_merged_event(shards, 3) == (5, 2)
    assert nth_merged_event(shards, 4) is None
    # Same timestamp everywhere, many shards.
    flat = [[7] for _ in range(6)]
    for i in range(6):
        assert nth_merged_event(flat, i) == (7, i)
    # A tie between a shard's later event and another shard's first.
    assert nth_merged_event([[1, 9], [9]], 1) == (9, 0)
    assert nth_merged_event([[1, 9], [9]], 2) == (9, 1)


def _test_edges() -> None:
    # No shards at all.
    assert nth_merged_event([], 0) is None
    assert nth_merged_event([], 200_000) is None
    # Only empty shards.
    assert nth_merged_event([[], [], []], 0) is None
    # One shard holding everything.
    assert nth_merged_event([[1, 2, 3, 4, 5]], 3) == (4, 0)
    assert nth_merged_event([[1, 2, 3, 4, 5]], 4) == (5, 0)
    assert nth_merged_event([[1, 2, 3, 4, 5]], 5) is None
    # Exactly one event.
    assert nth_merged_event([[42]], 0) == (42, 0)
    assert nth_merged_event([[42]], 1) is None
    # Leading empty shards must not shift the indices reported.
    assert nth_merged_event([[], [], [8]], 0) == (8, 2)
    # Fully interleaved.
    assert nth_merged_event([[1, 3, 5], [2, 4, 6]], 0) == (1, 0)
    assert nth_merged_event([[1, 3, 5], [2, 4, 6]], 1) == (2, 1)
    assert nth_merged_event([[1, 3, 5], [2, 4, 6]], 5) == (6, 1)
    # Disjoint ranges: one shard drains entirely before the other starts.
    assert nth_merged_event([[100, 200], [1, 2]], 1) == (2, 1)
    assert nth_merged_event([[100, 200], [1, 2]], 2) == (100, 0)
    # Timestamp ceiling.
    assert nth_merged_event([[10**18], [10**18 - 1]], 0) == (10**18 - 1, 1)
    assert nth_merged_event([[10**18], [10**18 - 1]], 1) == (10**18, 0)
    # Duplicates inside a single shard.
    assert nth_merged_event([[3, 3, 3]], 2) == (3, 0)


def _test_random_against_brute() -> None:
    rng = _Lcg(0x1F83D9ABFB41BD6B)
    for _ in range(400):
        k = rng.below(7)
        shards = []
        for _ in range(k):
            m = rng.below(5)
            shard = sorted(rng.below(8) for _ in range(m))
            shards.append(shard)
        total = sum(len(s) for s in shards)
        # Probe every valid offset plus one past the end.
        for offset in range(total + 2):
            got = nth_merged_event(shards, offset)
            want = _brute(shards, offset)
            assert got == want, "shards=%r offset=%d got=%r want=%r" % (
                shards,
                offset,
                got,
                want,
            )


def _test_large_many_shards() -> None:
    """100k shards, shallow paging. Rescanning all heads would be 2 * 10^10."""
    k = 100_000
    # Shard i holds timestamps i, i + k, i + 2k. Merged order is therefore
    # 0, 1, 2, ..., k-1, k, k+1, ... -- position p holds timestamp p, from
    # shard p % k.
    shards = [[i, i + k, i + 2 * k] for i in range(k)]
    assert nth_merged_event(shards, 0) == (0, 0)
    assert nth_merged_event(shards, 1) == (1, 1)
    assert nth_merged_event(shards, 99_999) == (99_999, 99_999)
    assert nth_merged_event(shards, 100_000) == (100_000, 0)
    assert nth_merged_event(shards, 200_000) == (200_000, 0)


def _test_large_deep_offset() -> None:
    """Maximum offset against 2M total events."""
    k = 1_000
    per = 2_000
    # Shard i holds i*per .. i*per + per - 1, so the merged view is just
    # 0, 1, 2, ... in order and position p holds timestamp p from shard p//per.
    shards = [[i * per + j for j in range(per)] for i in range(k)]
    assert sum(len(s) for s in shards) == 2_000_000
    assert nth_merged_event(shards, 200_000) == (200_000, 100)
    assert nth_merged_event(shards, 0) == (0, 0)
    assert nth_merged_event(shards, 1_999) == (1_999, 0)
    assert nth_merged_event(shards, 2_000) == (2_000, 1)


def _test_large_all_ties() -> None:
    """Every event shares one timestamp: the tie rule alone decides the order."""
    k = 100_000
    shards = [[99] for _ in range(k)]
    assert nth_merged_event(shards, 0) == (99, 0)
    assert nth_merged_event(shards, 1) == (99, 1)
    assert nth_merged_event(shards, 50_000) == (99, 50_000)
    assert nth_merged_event(shards, k - 1) == (99, k - 1)
    assert nth_merged_event(shards, k) is None


def _test_large_one_fat_shard() -> None:
    """A single shard holding 2M events, plus many empty ones."""
    fat = list(range(0, 4_000_000, 2))  # 2M even timestamps
    shards = [[] for _ in range(50_000)]
    shards.append(fat)
    assert nth_merged_event(shards, 0) == (0, 50_000)
    assert nth_merged_event(shards, 123_456) == (246_912, 50_000)
    assert nth_merged_event(shards, 200_000) == (400_000, 50_000)


def _test_large_offset_past_end() -> None:
    """Sparse data with a deep offset: must return None without scanning far."""
    shards = [[i] for i in range(1_000)]
    assert nth_merged_event(shards, 999) == (999, 999)
    assert nth_merged_event(shards, 1_000) is None
    assert nth_merged_event(shards, 200_000) is None


if __name__ == "__main__":
    _test_examples()
    _test_full_traversal()
    _test_ties()
    _test_edges()
    _test_random_against_brute()
    _test_large_many_shards()
    _test_large_deep_offset()
    _test_large_all_ties()
    _test_large_one_fat_shard()
    _test_large_offset_past_end()
    print("All tests passed.")
