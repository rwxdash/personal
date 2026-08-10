"""
Audit Sampling
problems/dp-on-trees/01-audit-sampling

Fill in `max_audit_evidence`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List


def max_audit_evidence(parent: List[int], evidence: List[int]) -> int:
    """Maximum audit evidence with no audited service directly under another.

    Choose a set of services such that no chosen service is the direct parent
    of another chosen service. Siblings are fine; grandparent and grandchild
    are fine. Maximise the total evidence. Auditing nothing is allowed.

    Args:
        parent:   parent[i] is the parent of service i; parent[0] is -1.
                  Guaranteed a valid tree rooted at 0, 1 <= n <= 200_000.
                  The tree may be a single chain 200_000 deep, and a parent's
                  index is NOT guaranteed to be smaller than its children's.
        evidence: Audit coverage points per service, each 0 ..= 10^9.

    Returns:
        The maximum total evidence. Can reach 2 * 10^14. Never negative.

    Required: O(n) time, O(n) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check enumerates every subset of services and checks the
# parent-child rule directly, so it shares no structure with a bottom-up sweep.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(parent: List[int], evidence: List[int]) -> int:
    """Try every subset. O(2^n * n); only viable for tiny trees."""
    n = len(parent)
    best = 0
    for mask in range(1 << n):
        ok = True
        for i in range(n):
            if mask & (1 << i) and parent[i] != -1 and mask & (1 << parent[i]):
                ok = False
                break
        if ok:
            total = sum(evidence[i] for i in range(n) if mask & (1 << i))
            best = max(best, total)
    return best


def _test_examples() -> None:
    assert max_audit_evidence([-1, 0, 0], [10, 6, 6]) == 12
    assert max_audit_evidence([-1, 0, 0], [10, 3, 3]) == 10
    assert max_audit_evidence([-1, 0, 1, 2], [4, 1, 1, 4]) == 8
    assert max_audit_evidence([-1, 0, 1], [5, 100, 5]) == 100


def _test_simple_rules_fail() -> None:
    """Greedy by value and by depth level both give wrong answers."""
    # Greedy takes the root's 10 and loses 12.
    assert max_audit_evidence([-1, 0, 0], [10, 6, 6]) == 12
    # Even-depth-only gives 5 + 5 = 10; the answer is the middle node's 100.
    assert max_audit_evidence([-1, 0, 1], [5, 100, 5]) == 100
    # Odd-depth-only gives 1; even-depth-only gives 8, which is right here --
    # so neither rule is consistently right.
    assert max_audit_evidence([-1, 0, 1, 2], [4, 1, 1, 4]) == 8


def _test_edges() -> None:
    # Single service.
    assert max_audit_evidence([-1], [42]) == 42
    assert max_audit_evidence([-1], [0]) == 0
    # All zero evidence.
    assert max_audit_evidence([-1, 0, 0, 1], [0, 0, 0, 0]) == 0
    # Two services: take the larger one.
    assert max_audit_evidence([-1, 0], [3, 9]) == 9
    assert max_audit_evidence([-1, 0], [9, 3]) == 9
    # A star: all leaves beat the hub, or not.
    assert max_audit_evidence([-1, 0, 0, 0], [5, 2, 2, 2]) == 6
    assert max_audit_evidence([-1, 0, 0, 0], [10, 2, 2, 2]) == 10
    # Deep chain of equal values: every other one.
    assert max_audit_evidence([-1, 0, 1, 2, 3], [1, 1, 1, 1, 1]) == 3
    # The root is best left unaudited even though it is worth something.
    assert max_audit_evidence([-1, 0, 1], [1, 50, 1]) == 50
    # Grandchildren stack up.
    assert max_audit_evidence([-1, 0, 1, 2, 3, 4], [5, 1, 5, 1, 5, 1]) == 15
    # A parent whose index is LARGER than its child's. The tree is
    # 0 -> 2 -> 1, so node 1's parent is node 2. The root is still node 0, as
    # the contract requires -- only the internal numbering is out of order.
    assert max_audit_evidence([-1, 2, 0], [7, 7, 20]) == 20
    assert max_audit_evidence([-1, 2, 0], [7, 7, 5]) == 14
    # Value ceiling.
    assert max_audit_evidence([-1, 0], [10**9, 10**9]) == 10**9


def _test_random_against_brute() -> None:
    rng = _Lcg(0xD192E819D6EF5218)
    for _ in range(400):
        n = 1 + rng.below(12)
        parent = [-1] + [rng.below(i) for i in range(1, n)]
        evidence = [rng.below(20) for _ in range(n)]
        got = max_audit_evidence(parent, evidence)
        want = _brute(parent, evidence)
        assert got == want, "parent=%r evidence=%r got=%d want=%d" % (
            parent,
            evidence,
            got,
            want,
        )


def _test_random_shuffled_labels() -> None:
    """Same cross-check with labels permuted, so the root is not always index 0
    in the underlying shape and parents may have larger indices than children.

    Node 0 is still the root, as the contract requires -- the permutation maps
    the original root onto label 0.
    """
    rng = _Lcg(0xD69906245565A910)
    for _ in range(300):
        n = 1 + rng.below(10)
        base = [-1] + [rng.below(i) for i in range(1, n)]
        # A permutation that keeps the root at label 0.
        perm = list(range(n))
        for i in range(n - 1, 1, -1):
            j = 1 + rng.below(i)
            perm[i], perm[j] = perm[j], perm[i]
        parent = [-1] * n
        evidence_base = [rng.below(25) for _ in range(n)]
        evidence = [0] * n
        for old in range(n):
            parent[perm[old]] = -1 if base[old] == -1 else perm[base[old]]
            evidence[perm[old]] = evidence_base[old]
        assert max_audit_evidence(parent, evidence) == _brute(parent, evidence)


def _test_large_chain() -> None:
    """200k-deep chain of equal values: every other service, starting at the root."""
    n = 200_000
    parent = [-1] + list(range(n - 1))
    evidence = [1] * n
    assert max_audit_evidence(parent, evidence) == (n + 1) // 2


def _test_large_chain_alternating() -> None:
    """A chain where the odd-depth services are far more valuable."""
    n = 200_000
    parent = [-1] + list(range(n - 1))
    evidence = [1 if i % 2 == 0 else 1000 for i in range(n)]
    # Taking every odd index gives n/2 services at 1000 each.
    assert max_audit_evidence(parent, evidence) == (n // 2) * 1000


def _test_large_star() -> None:
    """200k-leaf star: the leaves together dwarf the hub."""
    n = 200_000
    parent = [-1] + [0] * (n - 1)
    evidence = [10**9] + [1] * (n - 1)
    # Leaves total n-1 = 199_999, which is far less than the hub's 10^9.
    assert max_audit_evidence(parent, evidence) == 10**9
    # Now make the leaves worth more in aggregate.
    evidence = [10**9] + [10_000] * (n - 1)
    assert max_audit_evidence(parent, evidence) == (n - 1) * 10_000


def _test_large_all_max() -> None:
    """Every service at the value ceiling: forces 64-bit arithmetic."""
    n = 200_000
    parent = [-1] + list(range(n - 1))
    evidence = [10**9] * n
    expected = ((n + 1) // 2) * 10**9  # 10^14
    assert max_audit_evidence(parent, evidence) == expected
    assert expected > 2**31


def _test_large_binary_tree() -> None:
    """A complete binary tree with equal values.

    On a complete binary tree with all values equal, the best choice is every
    node at even depth, since each level roughly doubles.
    """
    n = 2**17 - 1  # 131_071, a perfect binary tree of depth 17
    parent = [-1] + [(i - 1) // 2 for i in range(1, n)]
    evidence = [1] * n
    depth_of = [0] * n
    for i in range(1, n):
        depth_of[i] = depth_of[(i - 1) // 2] + 1
    expected = sum(1 for i in range(n) if depth_of[i] % 2 == 0)
    assert max_audit_evidence(parent, evidence) == expected


def _test_large_wide_shallow() -> None:
    """A root with 200k children, all worth more than the root."""
    n = 200_000
    parent = [-1] + [0] * (n - 1)
    evidence = [5] + [7] * (n - 1)
    assert max_audit_evidence(parent, evidence) == (n - 1) * 7


def _test_large_zero_root() -> None:
    """A worthless root: taking it can only cost you."""
    n = 200_000
    parent = [-1] + list(range(n - 1))
    evidence = [0] + [1] * (n - 1)
    # Best is every other service starting at index 1: services 1, 3, 5, ...
    assert max_audit_evidence(parent, evidence) == (n - 1 + 1) // 2


if __name__ == "__main__":
    _test_examples()
    _test_simple_rules_fail()
    _test_edges()
    _test_random_against_brute()
    _test_random_shuffled_labels()
    _test_large_chain()
    _test_large_chain_alternating()
    _test_large_star()
    _test_large_all_max()
    _test_large_binary_tree()
    _test_large_wide_shallow()
    _test_large_zero_root()
    print("All tests passed.")
