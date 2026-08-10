"""
Resource Ancestry
problems/dfs/01-resource-ancestry

Fill in `ancestor_queries`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Tuple


def ancestor_queries(
    parent: List[int], queries: List[Tuple[int, int]]
) -> List[bool]:
    """Answer 'is u an ancestor of v?' for every query.

    `u` is an ancestor of `v` when u == v, or when u lies on the path from v
    upward to its root. Resources in different trees are never ancestors of one
    another.

    Args:
        parent:  parent[i] is the parent of resource i, or -1 if i is a root.
                 1 <= n <= 200_000. Guaranteed a valid forest: following parent
                 pointers always terminates at a root, with no cycles. The
                 hierarchy may be a single chain up to 200_000 deep.
        queries: (u, v) pairs, 0 <= q <= 200_000, both valid indices.

    Returns:
        One boolean per query, in the same order.

    Required: O(n + q) time, O(n) space -- so O(1) per query after linear
    preprocessing.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check walks the parent chain per query, which is the naive
# approach the complexity bound forbids -- an honest oracle, not a
# re-implementation.

class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(parent: List[int], queries: List[Tuple[int, int]]) -> List[bool]:
    """Walk upward from v looking for u. O(q * depth)."""
    out = []
    for u, v in queries:
        cur = v
        found = False
        while cur != -1:
            if cur == u:
                found = True
                break
            cur = parent[cur]
        out.append(found)
    return out


def _test_examples() -> None:
    assert ancestor_queries(
        [-1, 0, 0, 1], [(0, 3), (1, 3), (2, 3), (3, 0), (1, 1)]
    ) == [True, True, False, False, True]
    assert ancestor_queries(
        [-1, 0, -1, 2], [(0, 1), (0, 3), (2, 3), (2, 1)]
    ) == [True, False, True, False]
    assert ancestor_queries(
        [-1, 0, 1, 2], [(0, 3), (1, 2), (2, 1), (3, 3)]
    ) == [True, True, False, True]


def _test_self_and_direction() -> None:
    """Non-strict ancestry, and the asymmetry of the relation."""
    parent = [-1, 0, 1]
    # Every node is its own ancestor.
    assert ancestor_queries(parent, [(i, i) for i in range(3)]) == [True] * 3
    # Downward is True, upward is False, for every ordered pair.
    assert ancestor_queries(parent, [(0, 1), (1, 0)]) == [True, False]
    assert ancestor_queries(parent, [(0, 2), (2, 0)]) == [True, False]
    assert ancestor_queries(parent, [(1, 2), (2, 1)]) == [True, False]


def _test_cross_tree() -> None:
    """Nodes in different trees are never ancestors, whatever their indices.

    These fail if the traversal timer is reset per root instead of running
    globally across the whole forest.
    """
    # Two chains, interleaved indices.
    parent = [-1, -1, 0, 1]  # tree A: 0 -> 2, tree B: 1 -> 3
    assert ancestor_queries(parent, [(0, 2), (1, 3)]) == [True, True]
    assert ancestor_queries(parent, [(0, 3), (1, 2)]) == [False, False]
    assert ancestor_queries(parent, [(0, 1), (1, 0)]) == [False, False]
    assert ancestor_queries(parent, [(2, 3), (3, 2)]) == [False, False]
    # Three separate single-node trees.
    parent = [-1, -1, -1]
    assert ancestor_queries(parent, [(0, 1), (1, 2), (2, 0)]) == [False] * 3
    assert ancestor_queries(parent, [(0, 0), (1, 1), (2, 2)]) == [True] * 3


def _test_edges() -> None:
    # Single node.
    assert ancestor_queries([-1], [(0, 0)]) == [True]
    # No queries.
    assert ancestor_queries([-1, 0], []) == []
    # A wide star: the root is an ancestor of every leaf, leaves of nobody.
    parent = [-1] + [0] * 5
    assert ancestor_queries(parent, [(0, i) for i in range(1, 6)]) == [True] * 5
    assert ancestor_queries(parent, [(i, 0) for i in range(1, 6)]) == [False] * 5
    # Siblings are never ancestors of each other.
    assert ancestor_queries(parent, [(1, 2), (2, 1), (3, 5)]) == [False] * 3
    # Root is not index 0.
    parent = [1, -1, 1]  # 1 is the root, with children 0 and 2
    assert ancestor_queries(parent, [(1, 0), (1, 2), (0, 2), (0, 1)]) == [
        True,
        True,
        False,
        False,
    ]
    # Every node a root: a forest of singletons.
    n = 6
    parent = [-1] * n
    assert ancestor_queries(parent, [(i, j) for i in range(n) for j in range(n)]) == [
        i == j for i in range(n) for j in range(n)
    ]
    # Deep-then-wide: a chain that fans out at the bottom.
    parent = [-1, 0, 1, 2, 2, 2]
    assert ancestor_queries(parent, [(0, 5), (1, 4), (2, 3), (3, 4)]) == [
        True,
        True,
        True,
        False,
    ]
    # Repeated identical queries.
    assert ancestor_queries([-1, 0], [(0, 1)] * 5) == [True] * 5


def _test_random_against_brute() -> None:
    rng = _Lcg(0x71374491BE5466CF)
    for _ in range(400):
        n = 1 + rng.below(12)
        # Build a random forest: node i's parent is a strictly smaller index,
        # or -1. This can never create a cycle.
        parent = []
        for i in range(n):
            if i == 0 or rng.below(3) == 0:
                parent.append(-1)
            else:
                parent.append(rng.below(i))
        queries = [(rng.below(n), rng.below(n)) for _ in range(rng.below(15))]
        got = ancestor_queries(parent, queries)
        want = _brute(parent, queries)
        assert got == want, "parent=%r queries=%r got=%r want=%r" % (
            parent,
            queries,
            got,
            want,
        )


def _test_random_shuffled_labels() -> None:
    """Same cross-check with node labels permuted, so parents are not always
    smaller indices and traversal order cannot be assumed."""
    rng = _Lcg(0x2748774CDF8EEB99)
    for _ in range(300):
        n = 1 + rng.below(10)
        base = []
        for i in range(n):
            base.append(-1 if (i == 0 or rng.below(3) == 0) else rng.below(i))
        # Permute the labels.
        perm = list(range(n))
        for i in range(n - 1, 0, -1):
            j = rng.below(i + 1)
            perm[i], perm[j] = perm[j], perm[i]
        parent = [-1] * n
        for old in range(n):
            parent[perm[old]] = -1 if base[old] == -1 else perm[base[old]]
        queries = [(rng.below(n), rng.below(n)) for _ in range(rng.below(12))]
        assert ancestor_queries(parent, queries) == _brute(parent, queries)


def _test_large_deep_chain() -> None:
    """200k-deep chain. A recursive traversal dies here.

    Python's default recursion limit is ~1000, so this is a hard crash for the
    wrong implementation rather than merely a slow one.
    """
    n = 200_000
    parent = [-1] + list(range(n - 1))  # 0 <- 1 <- 2 <- ... <- n-1
    queries = [(0, n - 1), (n - 1, 0), (n // 2, n - 1), (n - 1, n // 2), (7, 7)]
    assert ancestor_queries(parent, queries) == [True, False, True, False, True]


def _test_large_chain_many_queries() -> None:
    """Deep chain with 200k queries: O(q * depth) would be 4 * 10^10 steps."""
    n = 200_000
    parent = [-1] + list(range(n - 1))
    # Ancestry on a chain is just index comparison.
    rng = _Lcg(0x9BDC06A725C71235)
    queries = [(rng.below(n), rng.below(n)) for _ in range(200_000)]
    got = ancestor_queries(parent, queries)
    assert got == [u <= v for u, v in queries]


def _test_large_star() -> None:
    """200k-leaf star."""
    n = 200_000
    parent = [-1] + [0] * (n - 1)
    queries = [(0, i) for i in range(n)] + [(i, 0) for i in range(1, n)]
    expected = [True] * n + [False] * (n - 1)
    assert ancestor_queries(parent, queries) == expected


def _test_large_forest() -> None:
    """1000 separate chains of 200 nodes each; cross-tree queries must be False.

    Fails if the traversal timer restarts at each root.
    """
    trees = 1_000
    depth = 200
    n = trees * depth
    parent = [-1] * n
    for t in range(trees):
        base = t * depth
        for d in range(1, depth):
            parent[base + d] = base + d - 1
    queries = []
    expected = []
    for t in range(trees):
        base = t * depth
        # Within the tree: root is an ancestor of the deepest node.
        queries.append((base, base + depth - 1))
        expected.append(True)
        # Across trees: never.
        other = ((t + 1) % trees) * depth
        queries.append((base, other + depth - 1))
        expected.append(False)
    assert ancestor_queries(parent, queries) == expected


def _test_large_binary_tree() -> None:
    """A complete binary tree: ancestry follows the heap index rule."""
    n = 200_000
    parent = [-1] + [(i - 1) // 2 for i in range(1, n)]

    def is_ancestor(u: int, v: int) -> bool:
        while v > u:
            v = (v - 1) // 2
        return u == v

    rng = _Lcg(0xC19BF1749EF14AD2)
    queries = [(rng.below(n), rng.below(n)) for _ in range(50_000)]
    got = ancestor_queries(parent, queries)
    assert got == [is_ancestor(u, v) for u, v in queries]


if __name__ == "__main__":
    _test_examples()
    _test_self_and_direction()
    _test_cross_tree()
    _test_edges()
    _test_random_against_brute()
    _test_random_shuffled_labels()
    _test_large_deep_chain()
    _test_large_chain_many_queries()
    _test_large_star()
    _test_large_forest()
    _test_large_binary_tree()
    print("All tests passed.")
