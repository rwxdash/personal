"""
Schema Migration Cost
problems/dp-2d/01-schema-migration-cost

Fill in `migration_cost`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List


def migration_cost(
    current: List[str],
    target: List[str],
    insert_cost: int,
    drop_cost: int,
    retype_cost: int,
) -> int:
    """Cheapest way to turn the `current` column layout into `target`.

    Column order matters and columns cannot be reordered. Three operations are
    available: insert a column, drop a column, or retype one column into
    another. A column already matching the target at that position is free.

    The three costs are independent -- in particular, retyping may be more
    expensive than dropping and re-inserting, or much cheaper.

    Args:
        current:     Current column names, 0 <= n <= 2_000. Names may repeat.
        target:      Target column names, 0 <= m <= 2_000.
        insert_cost: Cost to add a column, 0 ..= 10^6.
        drop_cost:   Cost to remove a column, 0 ..= 10^6.
        retype_cost: Cost to change one column into another, 0 ..= 10^6.

    Returns:
        Minimum total cost. Can reach about 4 * 10^9.

    Required: O(n * m) time, O(min(n, m)) space -- a full n x m table is NOT
    acceptable.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check explores every operation sequence recursively without
# memoisation, so it shares no structure with a filled table.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(
    current: List[str],
    target: List[str],
    insert_cost: int,
    drop_cost: int,
    retype_cost: int,
) -> int:
    """Explore every operation sequence. Exponential; only viable for tiny input."""

    def go(i: int, j: int) -> int:
        if i == len(current):
            return (len(target) - j) * insert_cost
        if j == len(target):
            return (len(current) - i) * drop_cost
        step = 0 if current[i] == target[j] else retype_cost
        return min(
            go(i + 1, j) + drop_cost,
            go(i, j + 1) + insert_cost,
            go(i + 1, j + 1) + step,
        )

    return go(0, 0)


def _test_examples() -> None:
    assert migration_cost(["id", "name", "email"], ["id", "email"], 1, 1, 1) == 1
    assert migration_cost(["a"], ["b"], 1, 1, 10) == 2
    assert migration_cost(["a", "b"], ["x", "y"], 5, 5, 1) == 2
    assert migration_cost(["a", "b", "c"], [], 7, 2, 3) == 6
    assert migration_cost(["a", "b"], ["b"], 1, 1, 1) == 1


def _test_cost_asymmetry() -> None:
    """The three costs interact; no branch may be assumed cheapest.

    Each line below picks a different winning route through the same shapes.
    """
    # Retype beats drop + insert.
    assert migration_cost(["a"], ["b"], 5, 5, 1) == 1
    # Drop + insert beats retype.
    assert migration_cost(["a"], ["b"], 1, 1, 10) == 2
    # Exactly equal: either route, same total.
    assert migration_cost(["a"], ["b"], 1, 1, 2) == 2
    # Free inserts make growing the layout costless.
    assert migration_cost([], ["a", "b", "c"], 0, 5, 5) == 0
    # Free drops make shrinking costless.
    assert migration_cost(["a", "b", "c"], [], 5, 0, 5) == 0
    # All costs zero.
    assert migration_cost(["a", "b"], ["c", "d"], 0, 0, 0) == 0
    # Asymmetric: dropping is dear, inserting is cheap.
    assert migration_cost(["a", "b", "c"], ["a"], 1, 100, 1) == 200
    assert migration_cost(["a"], ["a", "b", "c"], 1, 100, 1) == 2


def _test_edges() -> None:
    # Both empty.
    assert migration_cost([], [], 3, 4, 5) == 0
    # Identical layouts cost nothing.
    assert migration_cost(["a", "b", "c"], ["a", "b", "c"], 9, 9, 9) == 0
    # One empty side.
    assert migration_cost([], ["a", "b"], 3, 4, 5) == 6
    assert migration_cost(["a", "b"], [], 3, 4, 5) == 8
    # Single column, matching or not.
    assert migration_cost(["a"], ["a"], 1, 1, 1) == 0
    assert migration_cost(["a"], ["b"], 1, 1, 1) == 1
    # Repeated names within a layout.
    assert migration_cost(["a", "a", "a"], ["a"], 1, 1, 1) == 2
    assert migration_cost(["a"], ["a", "a", "a"], 1, 1, 1) == 2
    assert migration_cost(["a", "a"], ["a", "a"], 5, 5, 5) == 0
    # Reordering is not allowed, so a swap costs real work.
    assert migration_cost(["a", "b"], ["b", "a"], 1, 1, 1) == 2
    # Prefix and suffix matches.
    assert migration_cost(["x", "a", "b"], ["a", "b"], 1, 1, 1) == 1
    assert migration_cost(["a", "b", "x"], ["a", "b"], 1, 1, 1) == 1
    # Cost ceiling.
    assert migration_cost(["a"], ["b"], 10**6, 10**6, 10**6) == 10**6


def _test_random_against_brute() -> None:
    rng = _Lcg(0xE49B69C19EF14AD2)
    alphabet = ["a", "b", "c"]
    for _ in range(300):
        n = rng.below(7)
        m = rng.below(7)
        current = [alphabet[rng.below(3)] for _ in range(n)]
        target = [alphabet[rng.below(3)] for _ in range(m)]
        ins = rng.below(8)
        dele = rng.below(8)
        rep = rng.below(8)
        got = migration_cost(current, target, ins, dele, rep)
        want = _brute(current, target, ins, dele, rep)
        assert got == want, (
            "current=%r target=%r costs=(%d,%d,%d) got=%d want=%d"
            % (current, target, ins, dele, rep, got, want)
        )


def _test_random_lopsided() -> None:
    """One side much longer than the other, in both directions.

    Catches a rolled-up table that swaps the sequences without also swapping
    the insert and drop costs.
    """
    rng = _Lcg(0xEFBE4786384F25E3)
    alphabet = ["p", "q"]
    for _ in range(300):
        long_n = 1 + rng.below(9)
        short_n = rng.below(3)
        a = [alphabet[rng.below(2)] for _ in range(long_n)]
        b = [alphabet[rng.below(2)] for _ in range(short_n)]
        ins = 1 + rng.below(9)
        dele = 1 + rng.below(9)
        rep = 1 + rng.below(9)
        # Both orientations, with the costs kept as-is: the answers differ, and
        # a solution that swaps sequences without swapping costs gets one wrong.
        assert migration_cost(a, b, ins, dele, rep) == _brute(a, b, ins, dele, rep)
        assert migration_cost(b, a, ins, dele, rep) == _brute(b, a, ins, dele, rep)


def _test_large_identical() -> None:
    """2000 identical columns: nothing to do."""
    cols = ["col%d" % i for i in range(2_000)]
    assert migration_cost(cols, list(cols), 10**6, 10**6, 10**6) == 0


def _test_large_all_different() -> None:
    """2000 columns, none matching: 2000 retypes, or drops plus inserts."""
    a = ["a%d" % i for i in range(2_000)]
    b = ["b%d" % i for i in range(2_000)]
    # Retyping is cheapest per column.
    assert migration_cost(a, b, 10, 10, 3) == 2_000 * 3
    # Drop + insert is cheapest per column.
    assert migration_cost(a, b, 1, 1, 10) == 2_000 * 2
    # Cost ceiling on 2000 columns: 2 * 10^9, past a 32-bit signed int.
    assert migration_cost(a, b, 10**6, 10**6, 10**6) == 2_000 * 10**6


def _test_large_one_empty() -> None:
    """2000 columns against nothing, both directions."""
    cols = ["c%d" % i for i in range(2_000)]
    assert migration_cost(cols, [], 7, 10**6, 3) == 2_000 * 10**6
    assert migration_cost([], cols, 10**6, 7, 3) == 2_000 * 10**6


def _test_large_lopsided() -> None:
    """A long layout against a short one: exercises the space-saving swap."""
    long_side = ["x%d" % i for i in range(2_000)]
    short_side = ["x%d" % i for i in range(5)]
    # The 5 columns match the prefix, so the rest is 1995 drops.
    assert migration_cost(long_side, short_side, 3, 7, 11) == 1_995 * 7
    # Reversed: 1995 inserts. Costs stay attached to their own direction.
    assert migration_cost(short_side, long_side, 3, 7, 11) == 1_995 * 3


def _test_large_interleaved() -> None:
    """Every other column matches, so half the positions need work."""
    n = 2_000
    a = ["same%d" % i if i % 2 == 0 else "old%d" % i for i in range(n)]
    b = ["same%d" % i if i % 2 == 0 else "new%d" % i for i in range(n)]
    # The 1000 mismatched positions each need one retype at cost 2.
    assert migration_cost(a, b, 50, 50, 2) == 1_000 * 2


def _test_large_prefix_shift() -> None:
    """The target is the current layout shifted by one position."""
    n = 2_000
    a = ["c%d" % i for i in range(n)]
    b = ["c%d" % i for i in range(1, n + 1)]
    # Cheapest is to drop c0 and insert c2000: 2 operations at cost 1 each.
    assert migration_cost(a, b, 1, 1, 1) == 2
    # With retyping cheap, retyping every column also costs 2000 -- so the
    # drop+insert route still wins at these costs.
    assert migration_cost(a, b, 1, 1, 5) == 2


if __name__ == "__main__":
    _test_examples()
    _test_cost_asymmetry()
    _test_edges()
    _test_random_against_brute()
    _test_random_lopsided()
    _test_large_identical()
    _test_large_all_different()
    _test_large_one_empty()
    _test_large_lopsided()
    _test_large_interleaved()
    _test_large_prefix_shift()
    print("All tests passed.")
