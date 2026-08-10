"""
Byte-Budget Cache
problems/lru-lfu-design/01-byte-budget-cache

Fill in `cache_simulate`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Tuple


def cache_simulate(budget: int, ops: List[Tuple[int, int, int, int]]) -> List[int]:
    """Run a byte-budgeted LRU cache over a sequence of operations.

    Each op is (kind, key, value, size):
      kind 0 -- GET key.  Returns the stored value, or -1 on a miss. A HIT
                marks the key most recently used; a MISS changes nothing.
      kind 1 -- PUT key, value, size. If size > budget the op is rejected and
                the cache is left exactly as it was, including any existing
                entry under that key. Otherwise the key is stored with the new
                value and size and becomes most recently used, releasing its
                old size if it was already present. Then, while the total size
                exceeds the budget, the least recently used entry is evicted.

    Args:
        budget: Total byte budget, 0 ..= 10^12. A budget of 0 rejects every
                PUT, since every size is at least 1.
        ops:    Up to 200_000 operations. key and value are 0 ..= 10^9;
                size is 1 ..= 10^9 and only meaningful on a PUT.

    Returns:
        One entry per GET, in order.

    Required: O(len(ops)) time overall (amortised O(1) per op),
    O(len(ops)) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check runs a deliberately naive cache that keeps entries in
# a plain list and scans for the least recently used one, so it shares no
# structure with a hash map plus linked list.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(budget: int, ops: List[Tuple[int, int, int, int]]) -> List[int]:
    """List-based cache, least-recent at index 0. O(len(ops)^2)."""
    entries: List[List[int]] = []  # [key, value, size], least recent first
    total = 0
    out: List[int] = []

    def find(key: int) -> int:
        for i, e in enumerate(entries):
            if e[0] == key:
                return i
        return -1

    for kind, key, value, size in ops:
        if kind == 0:
            idx = find(key)
            if idx < 0:
                out.append(-1)
            else:
                entry = entries.pop(idx)
                entries.append(entry)  # most recent goes last
                out.append(entry[1])
        else:
            if size > budget:
                continue
            idx = find(key)
            if idx >= 0:
                total -= entries[idx][2]
                entries.pop(idx)
            entries.append([key, value, size])
            total += size
            while total > budget:
                victim = entries.pop(0)
                total -= victim[2]
    return out


def _test_examples() -> None:
    assert cache_simulate(
        10,
        [(1, 1, 100, 6), (1, 2, 200, 3), (0, 1, 0, 0), (1, 3, 300, 4), (0, 2, 0, 0), (0, 1, 0, 0)],
    ) == [100, -1, 100]
    assert cache_simulate(
        10,
        [(1, 1, 10, 3), (1, 2, 20, 3), (1, 3, 30, 3), (1, 4, 40, 9),
         (0, 1, 0, 0), (0, 2, 0, 0), (0, 3, 0, 0), (0, 4, 0, 0)],
    ) == [-1, -1, -1, 40]
    assert cache_simulate(
        10, [(1, 1, 10, 8), (1, 2, 20, 11), (0, 1, 0, 0), (0, 2, 0, 0)]
    ) == [10, -1]
    assert cache_simulate(
        10, [(1, 1, 10, 8), (1, 2, 20, 2), (1, 1, 99, 3), (0, 1, 0, 0), (0, 2, 0, 0)]
    ) == [99, 20]
    assert cache_simulate(
        6, [(1, 1, 10, 3), (1, 2, 20, 3), (0, 9, 0, 0), (1, 3, 30, 3), (0, 1, 0, 0)]
    ) == [-1, -1]


def _test_recency_rules() -> None:
    """A hit refreshes recency; a miss does not."""
    # Without the GET, key 1 would be evicted. With it, key 2 goes instead.
    ops_no_get = [(1, 1, 10, 3), (1, 2, 20, 3), (1, 3, 30, 3), (0, 1, 0, 0)]
    assert cache_simulate(6, ops_no_get) == [-1]
    ops_with_get = [(1, 1, 10, 3), (1, 2, 20, 3), (0, 1, 0, 0), (1, 3, 30, 3), (0, 1, 0, 0)]
    assert cache_simulate(6, ops_with_get) == [10, 10]
    # A PUT also refreshes recency.
    ops_reput = [(1, 1, 10, 3), (1, 2, 20, 3), (1, 1, 11, 3), (1, 3, 30, 3), (0, 1, 0, 0), (0, 2, 0, 0)]
    assert cache_simulate(6, ops_reput) == [11, -1]
    # A missing GET must not create an entry or change order.
    ops_miss = [(1, 1, 10, 3), (0, 42, 0, 0), (0, 42, 0, 0), (1, 2, 20, 3), (0, 1, 0, 0)]
    assert cache_simulate(6, ops_miss) == [-1, -1, 10]


def _test_rejection() -> None:
    """An oversized PUT is rejected and evicts nothing."""
    # The existing entry survives untouched.
    assert cache_simulate(
        5, [(1, 1, 10, 5), (1, 2, 20, 6), (0, 1, 0, 0), (0, 2, 0, 0)]
    ) == [10, -1]
    # Rejection does not even disturb recency.
    assert cache_simulate(
        6,
        [(1, 1, 10, 3), (1, 2, 20, 3), (1, 9, 90, 100), (1, 3, 30, 3), (0, 1, 0, 0), (0, 2, 0, 0)],
    ) == [-1, 20]
    # An oversized PUT on an EXISTING key leaves the old entry in place.
    assert cache_simulate(
        5, [(1, 1, 10, 4), (1, 1, 99, 9), (0, 1, 0, 0)]
    ) == [10]
    # Budget 0 rejects everything.
    assert cache_simulate(0, [(1, 1, 10, 1), (0, 1, 0, 0)]) == [-1]


def _test_sizes() -> None:
    """Overwrite releases the old size; eviction is a loop."""
    # Exactly full, then an overwrite that shrinks: nothing is evicted.
    assert cache_simulate(
        10, [(1, 1, 10, 8), (1, 2, 20, 2), (1, 1, 11, 3), (0, 1, 0, 0), (0, 2, 0, 0)]
    ) == [11, 20]
    # Overwrite that grows: the other entry is evicted.
    assert cache_simulate(
        10, [(1, 1, 10, 2), (1, 2, 20, 2), (1, 1, 11, 10), (0, 1, 0, 0), (0, 2, 0, 0)]
    ) == [11, -1]
    # An entry exactly equal to the budget evicts everything else.
    assert cache_simulate(
        10, [(1, 1, 10, 4), (1, 2, 20, 4), (1, 3, 30, 10), (0, 1, 0, 0), (0, 2, 0, 0), (0, 3, 0, 0)]
    ) == [-1, -1, 30]
    # Filling exactly to the budget evicts nothing.
    assert cache_simulate(
        10, [(1, 1, 10, 5), (1, 2, 20, 5), (0, 1, 0, 0), (0, 2, 0, 0)]
    ) == [10, 20]


def _test_edges() -> None:
    # No operations.
    assert cache_simulate(100, []) == []
    # Only PUTs: no output.
    assert cache_simulate(100, [(1, 1, 10, 5), (1, 2, 20, 5)]) == []
    # GET on an empty cache.
    assert cache_simulate(100, [(0, 1, 0, 0)]) == [-1]
    # Same key put repeatedly.
    assert cache_simulate(
        10, [(1, 1, 10, 5), (1, 1, 20, 5), (1, 1, 30, 5), (0, 1, 0, 0)]
    ) == [30]
    # Value 0 is a real value, distinct from the -1 miss marker.
    assert cache_simulate(10, [(1, 1, 0, 5), (0, 1, 0, 0)]) == [0]
    # Key 0 is a real key.
    assert cache_simulate(10, [(1, 0, 77, 5), (0, 0, 0, 0)]) == [77]
    # Large budget: nothing is ever evicted.
    assert cache_simulate(
        10**12, [(1, 1, 10, 10**9), (1, 2, 20, 10**9), (0, 1, 0, 0), (0, 2, 0, 0)]
    ) == [10, 20]


def _test_random_against_brute() -> None:
    rng = _Lcg(0xA2BFE8A14CF10364)
    for _ in range(400):
        budget = rng.below(20)
        n = rng.below(30)
        ops = []
        for _ in range(n):
            if rng.below(2) == 0:
                ops.append((0, rng.below(6), 0, 0))
            else:
                ops.append((1, rng.below(6), rng.below(100), 1 + rng.below(9)))
        got = cache_simulate(budget, ops)
        want = _brute(budget, ops)
        assert got == want, "budget=%d ops=%r got=%r want=%r" % (
            budget,
            ops,
            got,
            want,
        )


def _test_random_tight_budget() -> None:
    """Budgets small enough that almost every PUT evicts something."""
    rng = _Lcg(0xC24B8B70D0F89791)
    for _ in range(400):
        budget = 1 + rng.below(6)
        n = rng.below(25)
        ops = []
        for _ in range(n):
            if rng.below(3) == 0:
                ops.append((0, rng.below(4), 0, 0))
            else:
                ops.append((1, rng.below(4), rng.below(50), 1 + rng.below(8)))
        assert cache_simulate(budget, ops) == _brute(budget, ops)


def _test_large_hot_key() -> None:
    """One key read constantly must never be evicted."""
    n = 200_000
    ops = [(1, 0, 999, 5)]
    for i in range(1, n // 2):
        ops.append((1, i, i, 5))
        ops.append((0, 0, 0, 0))  # keep key 0 hot
    ops.append((0, 0, 0, 0))
    got = cache_simulate(20, ops)
    assert all(v == 999 for v in got), "the hot key must survive every eviction"
    assert len(got) == n // 2


def _test_large_thrash() -> None:
    """Every insert evicts: the cache holds one entry at a time."""
    n = 100_000
    ops = []
    for i in range(n):
        ops.append((1, i, i * 2, 10))
    for i in range(n):
        ops.append((0, i, 0, 0))
    got = cache_simulate(10, ops)
    # Only the last key survives.
    assert got == [-1] * (n - 1) + [(n - 1) * 2]


def _test_large_no_eviction() -> None:
    """A budget big enough for everything: pure map behaviour."""
    n = 100_000
    ops = [(1, i, i + 7, 1) for i in range(n)]
    ops += [(0, i, 0, 0) for i in range(n)]
    assert cache_simulate(10**12, ops) == [i + 7 for i in range(n)]


def _test_large_all_rejected() -> None:
    """Every PUT is oversized: the cache stays empty and nothing is evicted."""
    n = 100_000
    ops = [(1, 0, 1, 5)]  # one entry that fits
    ops += [(1, i, i, 100) for i in range(1, n)]  # all rejected
    ops.append((0, 0, 0, 0))
    assert cache_simulate(10, ops) == [1]


def _test_large_overwrite_same_key() -> None:
    """200k overwrites of one key: the total must not drift upward."""
    n = 200_000
    ops = [(1, 7, i, 5) for i in range(n)]
    ops.append((0, 7, 0, 0))
    assert cache_simulate(10, ops) == [n - 1]


def _test_large_byte_range() -> None:
    """Sizes near the value ceiling, forcing 64-bit totals."""
    ops = [
        (1, 1, 10, 10**9),
        (1, 2, 20, 10**9),
        (1, 3, 30, 10**9),
        (0, 1, 0, 0),
        (0, 2, 0, 0),
        (0, 3, 0, 0),
    ]
    # Budget holds exactly two of them.
    assert cache_simulate(2 * 10**9, ops) == [-1, 20, 30]


if __name__ == "__main__":
    _test_examples()
    _test_recency_rules()
    _test_rejection()
    _test_sizes()
    _test_edges()
    _test_random_against_brute()
    _test_random_tight_budget()
    _test_large_hot_key()
    _test_large_thrash()
    _test_large_no_eviction()
    _test_large_all_rejected()
    _test_large_overwrite_same_key()
    _test_large_byte_range()
    print("All tests passed.")
