"""
Config Propagation
problems/bfs/01-config-propagation

Fill in `propagation_rounds`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Tuple


def propagation_rounds(
    n: int,
    links: List[Tuple[int, int]],
    seeds: List[int],
    quarantined: List[bool],
) -> List[int]:
    """Round at which each node receives the rolled-out config.

    Seeds hold the config at round 0. Each round, every node already holding it
    forwards it to all direct peers simultaneously. A quarantined node ACCEPTS
    the config (and records its round) but never forwards it onward -- including
    when it is itself a seed.

    Args:
        n:           Node count, 1 <= n <= 200_000. Nodes are 0 .. n-1.
        links:       Bidirectional peering links, up to 400_000. May contain
                     duplicate pairs and self-loops. Graph may be disconnected.
        seeds:       Nodes holding the config at round 0. May be empty and may
                     contain duplicates.
        quarantined: Whether each node refuses to forward. Length n.

    Returns:
        A list of length n; element i is the round node i receives the config,
        0 for a seed, or -1 if it never receives it.

    Required: O(n + m) time and space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check runs an independent per-seed relaxation that makes no
# assumption about visit order, so it is an honest oracle.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(
    n: int,
    links: List[Tuple[int, int]],
    seeds: List[int],
    quarantined: List[bool],
) -> List[int]:
    """Relax every edge until nothing changes. O(n * m), order-agnostic."""
    adj: List[List[int]] = [[] for _ in range(n)]
    for u, v in links:
        adj[u].append(v)
        adj[v].append(u)

    inf = float("inf")
    dist = [inf] * n
    for s in seeds:
        dist[s] = 0

    for _ in range(n + 1):
        changed = False
        for u in range(n):
            if dist[u] == inf or quarantined[u]:
                continue  # unreached, or reached but forwards nothing
            for v in adj[u]:
                if dist[u] + 1 < dist[v]:
                    dist[v] = dist[u] + 1
                    changed = True
        if not changed:
            break
    return [-1 if d == inf else int(d) for d in dist]


def _test_examples() -> None:
    assert propagation_rounds(4, [(0, 1), (1, 2), (2, 3)], [0], [False] * 4) == [
        0,
        1,
        2,
        3,
    ]
    assert propagation_rounds(
        5, [(0, 1), (1, 2), (2, 3), (3, 4)], [0, 4], [False] * 5
    ) == [0, 1, 2, 1, 0]
    assert propagation_rounds(
        4, [(0, 1), (1, 2), (2, 3)], [0], [False, False, True, False]
    ) == [0, 1, 2, -1]
    assert propagation_rounds(
        3, [(0, 1), (1, 2)], [0], [True, False, False]
    ) == [0, -1, -1]


def _test_quarantine_semantics() -> None:
    """A quarantined node is recorded but never expands.

    Each assert below changes if quarantined nodes are skipped entirely rather
    than merely prevented from forwarding.
    """
    # The quarantined node still gets its own round number.
    assert propagation_rounds(2, [(0, 1)], [0], [False, True]) == [0, 1]
    # ...and blocks everything behind it.
    assert propagation_rounds(
        3, [(0, 1), (1, 2)], [0], [False, True, False]
    ) == [0, 1, -1]
    # A detour around the quarantined node still works.
    assert propagation_rounds(
        4, [(0, 1), (1, 3), (0, 2), (2, 3)], [0], [False, True, False, False]
    ) == [0, 1, 1, 2]
    # Every node quarantined: only seeds ever hold the config.
    assert propagation_rounds(3, [(0, 1), (1, 2)], [1], [True] * 3) == [-1, 0, -1]
    # A quarantined seed among healthy seeds: the others still propagate.
    assert propagation_rounds(
        4, [(0, 1), (2, 3)], [0, 2], [True, False, False, False]
    ) == [0, -1, 0, 1]


def _test_multi_source() -> None:
    """The answer is the minimum over all seeds, not any single seed's view."""
    assert propagation_rounds(
        5, [(0, 1), (1, 2), (2, 3), (3, 4)], [0, 4], [False] * 5
    ) == [0, 1, 2, 1, 0]
    # Every node a seed.
    assert propagation_rounds(
        4, [(0, 1), (1, 2), (2, 3)], [0, 1, 2, 3], [False] * 4
    ) == [0, 0, 0, 0]
    # Duplicate seeds change nothing.
    assert propagation_rounds(
        3, [(0, 1), (1, 2)], [0, 0, 0], [False] * 3
    ) == [0, 1, 2]
    # Seeds at both ends of a long chain meet in the middle.
    n = 9
    links = [(i, i + 1) for i in range(n - 1)]
    assert propagation_rounds(n, links, [0, 8], [False] * n) == [
        0, 1, 2, 3, 4, 3, 2, 1, 0
    ]


def _test_edges() -> None:
    # No seeds at all.
    assert propagation_rounds(3, [(0, 1), (1, 2)], [], [False] * 3) == [-1, -1, -1]
    # No links: only seeds are reached.
    assert propagation_rounds(3, [], [1], [False] * 3) == [-1, 0, -1]
    # Single node.
    assert propagation_rounds(1, [], [0], [False]) == [0]
    assert propagation_rounds(1, [], [], [False]) == [-1]
    assert propagation_rounds(1, [(0, 0)], [0], [True]) == [0]
    # Self-loops and duplicate links change nothing.
    assert propagation_rounds(
        3, [(0, 0), (0, 1), (0, 1), (1, 2)], [0], [False] * 3
    ) == [0, 1, 2]
    # Disconnected component is unreachable.
    assert propagation_rounds(4, [(0, 1), (2, 3)], [0], [False] * 4) == [0, 1, -1, -1]
    # A star: the hub reaches every leaf in one round.
    assert propagation_rounds(
        5, [(0, 1), (0, 2), (0, 3), (0, 4)], [0], [False] * 5
    ) == [0, 1, 1, 1, 1]
    # A leaf seed reaches the far leaves in two rounds.
    assert propagation_rounds(
        5, [(0, 1), (0, 2), (0, 3), (0, 4)], [1], [False] * 5
    ) == [1, 0, 2, 2, 2]
    # A cycle: the config wraps around both ways.
    assert propagation_rounds(
        6, [(i, (i + 1) % 6) for i in range(6)], [0], [False] * 6
    ) == [0, 1, 2, 3, 2, 1]


def _test_random_against_brute() -> None:
    rng = _Lcg(0xC19BF174CF692694)
    for _ in range(400):
        n = 1 + rng.below(9)
        m = rng.below(14)
        links = [(rng.below(n), rng.below(n)) for _ in range(m)]
        seeds = [rng.below(n) for _ in range(rng.below(4))]
        quarantined = [rng.below(4) == 0 for _ in range(n)]
        got = propagation_rounds(n, links, seeds, quarantined)
        want = _brute(n, links, seeds, quarantined)
        assert got == want, (
            "n=%d links=%r seeds=%r quarantined=%r got=%r want=%r"
            % (n, links, seeds, quarantined, got, want)
        )


def _test_large_chain() -> None:
    """200k-deep chain from one end: rounds are just the index."""
    n = 200_000
    links = [(i, i + 1) for i in range(n - 1)]
    assert propagation_rounds(n, links, [0], [False] * n) == list(range(n))


def _test_large_chain_both_ends() -> None:
    """Seeds at both ends: each node's round is its distance to the nearer end."""
    n = 200_001  # odd, so there is a unique midpoint
    links = [(i, i + 1) for i in range(n - 1)]
    got = propagation_rounds(n, links, [0, n - 1], [False] * n)
    assert got == [min(i, n - 1 - i) for i in range(n)]
    assert max(got) == (n - 1) // 2


def _test_large_many_seeds() -> None:
    """100k seeds on a 200k chain. Per-seed searches would be ~6 * 10^10 steps."""
    n = 200_000
    links = [(i, i + 1) for i in range(n - 1)]
    seeds = list(range(0, n, 2))  # every even node
    got = propagation_rounds(n, links, seeds, [False] * n)
    # Even nodes are seeds; odd nodes sit next to one.
    assert got == [0 if i % 2 == 0 else 1 for i in range(n)]


def _test_large_star() -> None:
    """200k-leaf star: everything is within two rounds of any leaf."""
    n = 200_000
    links = [(0, i) for i in range(1, n)]
    assert propagation_rounds(n, links, [0], [False] * n) == [0] + [1] * (n - 1)
    got = propagation_rounds(n, links, [1], [False] * n)
    assert got[1] == 0 and got[0] == 1
    assert all(got[i] == 2 for i in range(2, n))


def _test_large_quarantine_wall() -> None:
    """A quarantined node bisects the chain: everything past it is cut off."""
    n = 200_000
    links = [(i, i + 1) for i in range(n - 1)]
    quarantined = [False] * n
    wall = 100_000
    quarantined[wall] = True
    got = propagation_rounds(n, links, [0], quarantined)
    assert got[:wall + 1] == list(range(wall + 1)), "the wall itself is recorded"
    assert all(r == -1 for r in got[wall + 1:]), "everything past the wall is cut"


def _test_large_all_quarantined() -> None:
    """Every node quarantined: only the seeds ever hold the config."""
    n = 200_000
    links = [(i, i + 1) for i in range(n - 1)]
    seeds = [0, 12_345, n - 1]
    got = propagation_rounds(n, links, seeds, [True] * n)
    assert [i for i, r in enumerate(got) if r == 0] == sorted(seeds)
    assert sum(1 for r in got if r == -1) == n - 3


def _test_large_disconnected() -> None:
    """100k isolated pairs, seeding only one node of each."""
    pairs = 100_000
    n = 2 * pairs
    links = [(2 * i, 2 * i + 1) for i in range(pairs)]
    seeds = [2 * i for i in range(pairs)]
    got = propagation_rounds(n, links, seeds, [False] * n)
    assert got == [0 if i % 2 == 0 else 1 for i in range(n)]


if __name__ == "__main__":
    _test_examples()
    _test_quarantine_semantics()
    _test_multi_source()
    _test_edges()
    _test_random_against_brute()
    _test_large_chain()
    _test_large_chain_both_ends()
    _test_large_many_seeds()
    _test_large_star()
    _test_large_quarantine_wall()
    _test_large_all_quarantined()
    _test_large_disconnected()
    print("All tests passed.")
