"""
Mesh Link Failures
problems/union-find/01-mesh-link-failures

Fill in `partitions_after_failures`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Tuple


def partitions_after_failures(
    n: int,
    links: List[Tuple[int, int]],
    failures: List[int],
) -> List[int]:
    """Partition count after each successive link failure.

    Two nodes are in the same partition when some chain of intact links joins
    them; a node with no intact links is a partition of one.

    Args:
        n:        Node count, 1 <= n <= 200_000. Nodes are 0 .. n-1.
        links:    Bidirectional links, up to 400_000. May contain the same pair
                  more than once (redundant cables, separate indices) and
                  self-loops (u, u), which connect nothing.
        failures: Distinct *indices into `links`*, in the order they go down.
                  Never endpoint pairs, so duplicates are unambiguous.

    Returns:
        A list of len(failures) integers; element k is the partition count
        after the first k + 1 failures have been applied. Empty in, empty out.

    Required: O((n + m) * alpha(n)) time, O(n + m) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The large cases use topologies whose partition counts are known by
# construction, so a pass is real evidence and not a self-consistency check.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(
    n: int, links: List[Tuple[int, int]], failures: List[int]
) -> List[int]:
    """Rebuild the graph and flood-fill from scratch after every failure.

    O(q * (n + m)) -- the thing the real solution has to beat.
    """
    dead = set()
    out: List[int] = []
    for idx in failures:
        dead.add(idx)
        adj: List[List[int]] = [[] for _ in range(n)]
        for i, (u, v) in enumerate(links):
            if i not in dead:
                adj[u].append(v)
                adj[v].append(u)
        seen = [False] * n
        count = 0
        for s in range(n):
            if seen[s]:
                continue
            count += 1
            seen[s] = True
            stack = [s]
            while stack:
                x = stack.pop()
                for y in adj[x]:
                    if not seen[y]:
                        seen[y] = True
                        stack.append(y)
        out.append(count)
    return out


def _test_examples() -> None:
    assert partitions_after_failures(4, [(0, 1), (1, 2), (2, 3)], [1]) == [2]
    assert partitions_after_failures(4, [(0, 1), (1, 2), (2, 3)], [1, 0, 2]) == [2, 3, 4]
    assert partitions_after_failures(2, [(0, 1), (0, 1)], [0, 1]) == [
        1,
        2,
    ], "redundant cables are separate links; index 0 dying leaves index 1 intact"
    assert partitions_after_failures(3, [(0, 0), (1, 2)], [0, 1]) == [2, 3]


def _test_edges() -> None:
    # No failures at all.
    assert partitions_after_failures(5, [(0, 1), (2, 3)], []) == []
    assert partitions_after_failures(1, [], []) == []
    # Single node with a self-loop: still one partition, cutting changes nothing.
    assert partitions_after_failures(1, [(0, 0)], [0]) == [1]
    # Cutting a link inside a cycle never fragments anything.
    assert partitions_after_failures(3, [(0, 1), (1, 2), (2, 0)], [0]) == [1]
    assert partitions_after_failures(3, [(0, 1), (1, 2), (2, 0)], [0, 1]) == [1, 2]
    assert partitions_after_failures(3, [(0, 1), (1, 2), (2, 0)], [0, 1, 2]) == [1, 2, 3]
    # Links that are never cut keep holding the mesh together.
    assert partitions_after_failures(4, [(0, 1), (1, 2), (2, 3)], [0]) == [2]
    # Isolated nodes are counted from the start.
    assert partitions_after_failures(6, [(0, 1)], [0]) == [6]
    # Failures given out of index order.
    assert partitions_after_failures(5, [(0, 1), (1, 2), (2, 3), (3, 4)], [3, 0, 2, 1]) == [
        2,
        3,
        4,
        5,
    ]
    # Triple-redundant cable: only the last cut matters.
    assert partitions_after_failures(2, [(0, 1), (0, 1), (0, 1)], [1, 2, 0]) == [1, 1, 2]
    # Self-loops interleaved with real links.
    assert partitions_after_failures(
        4, [(0, 0), (0, 1), (1, 1), (1, 2), (3, 3)], [0, 2, 3, 4, 1]
    ) == [2, 2, 3, 3, 4]


def _test_random_against_brute() -> None:
    rng = _Lcg(0x5DEECE66D1234567)
    for _ in range(300):
        n = 1 + rng.below(9)
        m = rng.below(14)
        links = [(rng.below(n), rng.below(n)) for _ in range(m)]
        # A random distinct subset of link indices, in random order.
        idx = list(range(m))
        q = rng.below(m + 1)
        for i in range(q):
            j = i + rng.below(m - i)
            idx[i], idx[j] = idx[j], idx[i]
        failures = idx[:q]
        got = partitions_after_failures(n, links, failures)
        want = _brute(n, links, failures)
        assert got == want, "n=%d links=%r failures=%r got=%r want=%r" % (
            n,
            links,
            failures,
            got,
            want,
        )


def _test_large_chain_scrambled() -> None:
    """200k-node path graph, 100k links cut in scrambled order.

    Cutting k distinct edges of a path always yields exactly k + 1 components,
    whatever order they are cut in, so the expected output is pinned by
    construction and does not depend on the shuffle. The flood-fill checker
    would need ~2 * 10^10 operations here.
    """
    n = 200_000
    links = [(i, i + 1) for i in range(n - 1)]
    q = 100_000
    idx = list(range(len(links)))
    rng = _Lcg(0x27D4EB2F165667C5)
    for i in range(q):
        j = i + rng.below(len(links) - i)
        idx[i], idx[j] = idx[j], idx[i]
    failures = idx[:q]
    assert partitions_after_failures(n, links, failures) == list(range(2, q + 2))


def _test_large_duplicate_cables() -> None:
    """Every chain link doubled: the first cut of each pair must be inert."""
    n = 100_000
    links: List[Tuple[int, int]] = []
    for i in range(n - 1):
        links.append((i, i + 1))
        links.append((i, i + 1))  # redundant cable, separate index
    # Cut only the first cable of each pair: the mesh must stay whole.
    first_of_each = [2 * i for i in range(n - 1)]
    assert partitions_after_failures(n, links, first_of_each) == [1] * (n - 1)


def _test_large_cycle() -> None:
    """A ring: the first cut is free, every later cut fragments."""
    n = 200_000
    links = [(i, (i + 1) % n) for i in range(n)]
    q = 50_000
    failures = list(range(q))
    # Cutting k edges of a cycle gives max(1, k) components.
    expected = [1] + list(range(2, q + 1))
    assert partitions_after_failures(n, links, failures) == expected


def _test_large_untouched_backbone() -> None:
    """A backbone that is never cut keeps everything in one partition."""
    n = 200_000
    links = [(i, i + 1) for i in range(n - 1)]  # backbone, indices 0..n-2
    spurs = len(links)
    links += [(0, i) for i in range(1, 100_001)]  # redundant spurs to node 0
    failures = list(range(spurs, spurs + 100_000))
    assert partitions_after_failures(n, links, failures) == [1] * 100_000


def _test_large_self_loops() -> None:
    """200k self-loops: nothing is ever connected, nothing ever changes."""
    n = 200_000
    links = [(i, i) for i in range(n)]
    failures = list(range(0, n, 2))
    assert partitions_after_failures(n, links, failures) == [n] * len(failures)


def _test_large_star_shatter() -> None:
    """A star: each cut peels exactly one leaf off the hub."""
    n = 200_000
    links = [(0, i) for i in range(1, n)]
    q = 150_000
    failures = list(range(q))
    assert partitions_after_failures(n, links, failures) == list(range(2, q + 2))


if __name__ == "__main__":
    _test_examples()
    _test_edges()
    _test_random_against_brute()
    _test_large_chain_scrambled()
    _test_large_duplicate_cables()
    _test_large_cycle()
    _test_large_untouched_backbone()
    _test_large_self_loops()
    _test_large_star_shatter()
    print("All tests passed.")
