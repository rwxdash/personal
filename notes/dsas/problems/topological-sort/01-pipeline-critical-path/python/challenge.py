"""
Pipeline Critical Path
problems/topological-sort/01-pipeline-critical-path

Fill in `pipeline_makespan`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List, Optional, Tuple


def pipeline_makespan(
    durations: List[int],
    ready: List[int],
    deps: List[Tuple[int, int]],
) -> Optional[int]:
    """Earliest instant at which every job in the pipeline has finished.

    Workers are unlimited, so a job starts as soon as it is allowed to:

        start[i]  = max(ready[i], max(finish[a] for every (a, i) in deps))
        finish[i] = start[i] + durations[i]
        makespan  = max(finish[i] for all i)

    (the inner max is 0 for a job with no dependencies).

    Args:
        durations: Run time per job in ms. 1 <= n <= 200_000, values 0..=10^9.
        ready:     Earliest permitted start per job in ms, same length, 0..=10^9.
        deps:      Pairs (a, b) meaning job a must finish before job b starts.
                   Up to 400_000 pairs. May contain duplicate pairs and
                   self-loops (a, a). The graph may be disconnected.

    Returns:
        The makespan in ms (can reach ~2 * 10^14), or None if `deps` contains
        a cycle, since no schedule then exists.

    Required: O(n + m) time and space, m = len(deps).
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check uses a fixpoint relaxation that makes no assumption
# about evaluation order, so it is a genuinely independent oracle.

class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _has_cycle(n: int, deps: List[Tuple[int, int]]) -> bool:
    """O(n * (n + m)) reachability check: is any node reachable from itself?

    Deliberately naive and order-agnostic, so it shares no machinery with the
    intended solution and can act as an honest oracle.
    """
    succ: List[List[int]] = [[] for _ in range(n)]
    for a, b in deps:
        succ[a].append(b)
    for source in range(n):
        seen = [False] * n
        stack = list(succ[source])
        while stack:
            v = stack.pop()
            if v == source:
                return True
            if not seen[v]:
                seen[v] = True
                stack.extend(succ[v])
    return False


def _brute(
    durations: List[int], ready: List[int], deps: List[Tuple[int, int]]
) -> Optional[int]:
    """Order-agnostic O(n * m) fixpoint, guarded by an explicit cycle check.

    The cycle check cannot be folded into the relaxation: a cycle whose jobs
    all have duration 0 reaches a fixpoint immediately, so "the values stopped
    moving" does not imply "the graph is schedulable".
    """
    n = len(durations)
    if _has_cycle(n, deps):
        return None
    start = list(ready)
    for _ in range(n + 1):
        moved = False
        for a, b in deps:
            candidate = start[a] + durations[a]
            if candidate > start[b]:
                start[b] = candidate
                moved = True
        if not moved:
            break
    return max(start[i] + durations[i] for i in range(n))


def _test_examples() -> None:
    assert pipeline_makespan([3, 2, 4], [0, 0, 0], [(0, 1), (1, 2)]) == 9
    assert pipeline_makespan([3, 2, 4], [0, 0, 0], []) == 4
    assert (
        pipeline_makespan([1, 5, 2, 1], [0] * 4, [(0, 1), (0, 2), (1, 3), (2, 3)]) == 7
    )
    assert pipeline_makespan([1, 1], [0, 10], [(0, 1)]) == 11
    assert pipeline_makespan([1, 1], [0, 0], [(0, 1), (1, 0)]) is None


def _test_cycles() -> None:
    # A self-loop is a job that must finish before itself.
    assert pipeline_makespan([5], [0], [(0, 0)]) is None
    # Cycle sitting off to the side of a perfectly fine component.
    assert pipeline_makespan(
        [1, 1, 1, 1], [0] * 4, [(0, 1), (2, 3), (3, 2)]
    ) is None
    # Three-node cycle.
    assert pipeline_makespan([1, 1, 1], [0] * 3, [(0, 1), (1, 2), (2, 0)]) is None
    # Cycle unreachable from any zero-in-degree node still has to be caught.
    assert pipeline_makespan(
        [1, 1, 1], [0] * 3, [(0, 1), (1, 2), (2, 1)]
    ) is None


def _test_edges() -> None:
    # Single job.
    assert pipeline_makespan([7], [5], []) == 12
    assert pipeline_makespan([0], [0], []) == 0
    # Zero-duration gate jobs are ordinary nodes.
    assert pipeline_makespan([0, 0, 3], [0, 0, 0], [(0, 1), (1, 2)]) == 3
    # Duplicate edges must not corrupt in-degree bookkeeping.
    assert pipeline_makespan([2, 3], [0, 0], [(0, 1), (0, 1), (0, 1)]) == 5
    # Disconnected: an isolated job with a late artifact sets the makespan.
    assert pipeline_makespan([1, 1], [0, 500], []) == 501
    assert pipeline_makespan([1, 1, 1], [0, 0, 900], [(0, 1)]) == 901
    # ready time on an upstream job propagates downstream.
    assert pipeline_makespan([1, 1, 1], [100, 0, 0], [(0, 1), (1, 2)]) == 103
    # ready time already satisfied by dependencies changes nothing.
    assert pipeline_makespan([10, 1], [0, 5], [(0, 1)]) == 11
    # No deps at all, answer is the max finish, not the sum.
    assert pipeline_makespan([4, 9, 2], [0, 0, 0], []) == 9
    # 64-bit range.
    assert pipeline_makespan([10**9, 10**9], [10**9, 0], [(0, 1)]) == 3 * 10**9


def _test_random_against_brute() -> None:
    rng = _Lcg(0xA5A5A5A5DEADC0DE)
    for _ in range(300):
        n = 1 + rng.below(9)
        durations = [rng.below(6) for _ in range(n)]
        ready = [rng.below(12) for _ in range(n)]
        m = rng.below(14)
        # Unconstrained endpoints, so roughly a third of these graphs are
        # cyclic -- the oracle handles both cases.
        deps = [(rng.below(n), rng.below(n)) for _ in range(m)]
        got = pipeline_makespan(durations, ready, deps)
        want = _brute(durations, ready, deps)
        assert got == want, "durations=%r ready=%r deps=%r got=%r want=%r" % (
            durations,
            ready,
            deps,
            got,
            want,
        )


def _test_random_dags_against_brute() -> None:
    """Same cross-check but on guaranteed-acyclic graphs, so every answer is a
    real makespan rather than None."""
    rng = _Lcg(0x0123456789ABCDEF)
    for _ in range(300):
        n = 1 + rng.below(10)
        durations = [rng.below(7) for _ in range(n)]
        ready = [rng.below(15) for _ in range(n)]
        m = rng.below(20)
        deps = []
        for _ in range(m):
            a = rng.below(n)
            b = rng.below(n)
            if a == b:
                continue
            deps.append((min(a, b), max(a, b)))  # always points forward: acyclic
        got = pipeline_makespan(durations, ready, deps)
        want = _brute(durations, ready, deps)
        assert want is not None
        assert got == want, "durations=%r ready=%r deps=%r got=%r want=%r" % (
            durations,
            ready,
            deps,
            got,
            want,
        )


def _test_large_deep_chain() -> None:
    """200k-deep chain. Any recursive traversal dies here.

    Python's default recursion limit is ~1000, so this is not a performance
    test -- it is a hard crash for the wrong approach.
    """
    n = 200_000
    durations = [1] * n
    ready = [0] * n
    deps = [(i, i + 1) for i in range(n - 1)]
    assert pipeline_makespan(durations, ready, deps) == n


def _test_large_reversed_chain() -> None:
    """The same 200k chain, with `deps` listed back to front.

    Dependency lists arrive in whatever order the config was written in. Any
    approach that leans on the edge list already being in a usable order --
    including repeated relaxation passes -- degrades to O(n * m) here.
    """
    n = 200_000
    durations = [1] * n
    ready = [0] * n
    deps = [(i, i + 1) for i in range(n - 1)][::-1]
    assert pipeline_makespan(durations, ready, deps) == n


def _test_large_chain_with_late_artifact() -> None:
    """A late artifact halfway down a deep chain shifts everything after it."""
    n = 200_000
    durations = [1] * n
    ready = [0] * n
    ready[150_000] = 10**9
    deps = [(i, i + 1) for i in range(n - 1)]
    # finish[150_000] = 10^9 + 1, then +1 per remaining job.
    assert pipeline_makespan(durations, ready, deps) == 10**9 + 50_000


def _test_large_fan() -> None:
    """One source, 100k parallel jobs, one sink. Tests the max-not-sum rule."""
    mid = 100_000
    n = mid + 2  # 0 = source, 1..mid = parallel, mid+1 = sink
    durations = [5] + list(range(1, mid + 1)) + [3]
    ready = [0] * n
    deps = [(0, i) for i in range(1, mid + 1)] + [(i, n - 1) for i in range(1, mid + 1)]
    # source 0..5; slowest middle job finishes at 5 + mid; sink adds 3.
    assert pipeline_makespan(durations, ready, deps) == 5 + mid + 3


def _test_large_cycle() -> None:
    """A 200k chain closed into a loop by a single back edge."""
    n = 200_000
    durations = [1] * n
    ready = [0] * n
    deps = [(i, i + 1) for i in range(n - 1)]
    deps.append((n - 1, 0))
    assert pipeline_makespan(durations, ready, deps) is None


def _test_large_wide_layers() -> None:
    """Layered DAG, 400k edges, answer known by construction."""
    width = 400
    layers = 500
    n = width * layers
    durations = [2] * n
    ready = [0] * n
    deps = []
    for layer in range(layers - 1):
        base = layer * width
        nxt = base + width
        for i in range(width):
            # Two forward edges per node: 400k edges total, all pointing to the
            # next layer, so the longest route visits exactly one node per layer.
            deps.append((base + i, nxt + i))
            deps.append((base + i, nxt + (i + 1) % width))
    assert len(deps) == 2 * width * (layers - 1)
    assert pipeline_makespan(durations, ready, deps) == 2 * layers


def _test_large_disconnected_late_job() -> None:
    """The makespan is a max over all jobs, not just over sinks."""
    n = 200_000
    durations = [1] * n
    ready = [0] * n
    deps = [(i, i + 1) for i in range(n - 2)]  # leaves job n-1 isolated
    ready[n - 1] = 10**9
    assert pipeline_makespan(durations, ready, deps) == 10**9 + 1


if __name__ == "__main__":
    _test_examples()
    _test_cycles()
    _test_edges()
    _test_random_against_brute()
    _test_random_dags_against_brute()
    _test_large_deep_chain()
    _test_large_reversed_chain()
    _test_large_chain_with_late_artifact()
    _test_large_fan()
    _test_large_cycle()
    _test_large_wide_layers()
    _test_large_disconnected_late_job()
    print("All tests passed.")
