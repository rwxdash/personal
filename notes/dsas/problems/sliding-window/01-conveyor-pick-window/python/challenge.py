"""
Conveyor Pick Window
problems/sliding-window/01-conveyor-pick-window

Fill in `shortest_fulfilling_run`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import Dict, List, Optional, Tuple


def shortest_fulfilling_run(
    stream: List[str],
    order: Dict[str, int],
) -> Optional[Tuple[int, int]]:
    """Find the shortest contiguous stretch of `stream` that fills `order`.

    A stretch `stream[start..end]` (inclusive on both ends) fills the order if,
    for every `sku` in `order`, the stretch contains at least `order[sku]`
    occurrences of `sku`. Surplus occurrences are allowed.

    Args:
        stream: SKUs passing the picking station, in belt order.
                1 <= len(stream) <= 200_000.
        order:  Required quantity per SKU. Non-empty, every value >= 1,
                at most 10_000 keys, values summing to at most 200_000.

    Returns:
        The inclusive (start, end) index pair of the shortest filling stretch.
        On a tie in length, the pair with the smallest start index.
        None if no stretch of `stream` can fill the order.

    Required: O(n) time expected, O(k) extra space for k distinct ordered SKUs.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# These run automatically. They also cross-check your answer against a slow
# reference on hundreds of small random inputs, so a passing run is strong
# evidence of correctness, not just of "the examples work".

class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(
    stream: List[str], order: Dict[str, int]
) -> Optional[Tuple[int, int]]:
    """Obviously-correct O(n^2 * k) checker. Far too slow for the real bounds."""
    best: Optional[Tuple[int, int]] = None
    for left in range(len(stream)):
        have: Dict[str, int] = {}
        for right in range(left, len(stream)):
            sku = stream[right]
            if sku in order:
                have[sku] = have.get(sku, 0) + 1
            if all(have.get(k, 0) >= v for k, v in order.items()):
                if best is None or (right - left) < (best[1] - best[0]):
                    best = (left, right)
                break  # extending this left end only makes it longer
    return best


def _test_examples() -> None:
    assert shortest_fulfilling_run(
        ["A", "B", "A", "C", "A", "B"], {"A": 2, "B": 1}
    ) == (0, 2)
    assert shortest_fulfilling_run(["A", "A", "B", "B"], {"A": 1, "B": 1}) == (1, 2)
    assert shortest_fulfilling_run(
        ["A", "B", "Z", "A", "B"], {"A": 1, "B": 1}
    ) == (0, 1), "ties must break toward the smallest start index"
    assert shortest_fulfilling_run(["X", "Y"], {"X": 2}) is None


def _test_edges() -> None:
    # Single item, single requirement.
    assert shortest_fulfilling_run(["A"], {"A": 1}) == (0, 0)
    # Required SKU never appears at all.
    assert shortest_fulfilling_run(["A", "A", "A"], {"B": 1}) is None
    # The whole belt is needed, exactly.
    assert shortest_fulfilling_run(["A", "B", "C"], {"A": 1, "B": 1, "C": 1}) == (0, 2)
    # Nothing but filler.
    assert shortest_fulfilling_run(["Z"] * 50, {"A": 1}) is None
    # Surplus must not be mistaken for progress: 5 A's do not cover A:2 + B:1.
    assert shortest_fulfilling_run(["A"] * 5, {"A": 2, "B": 1}) is None
    # Deep surplus on one SKU, requirement satisfied only at the very end.
    assert shortest_fulfilling_run(["A"] * 10 + ["B"], {"A": 3, "B": 1}) == (7, 10)
    # Duplicates and a high quantity on one SKU.
    assert shortest_fulfilling_run(
        ["A", "A", "B", "A", "A", "A"], {"A": 4}
    ) == (0, 4), "earliest window of four A's is 0..4 (B is filler inside it)"


def _test_random_against_brute() -> None:
    rng = _Lcg(0x243F6A8885A308D3)
    for _ in range(400):
        n = 1 + rng.below(24)
        alpha = 1 + rng.below(4)
        stream = ["S%d" % rng.below(alpha) for _ in range(n)]
        k = 1 + rng.below(alpha)
        order = {"S%d" % i: 1 + rng.below(3) for i in range(k)}
        got = shortest_fulfilling_run(stream, order)
        want = _brute(stream, order)
        assert got == want, "stream=%r order=%r got=%r want=%r" % (
            stream,
            order,
            got,
            want,
        )


def _test_large_planted() -> None:
    """200k belt whose only tight window is planted; decoys are far away.

    The O(n^2) checker above would do ~2 * 10^10 steps here.
    """
    n = 200_000
    stream = ["Z"] * n
    # Decoys: they do fill the order, but only across a 15k-wide stretch.
    stream[10] = "A"
    stream[5_000] = "A"
    stream[9_000] = "B"
    stream[15_000] = "C"
    # The unique tight cluster: A, B, A, C.
    stream[100_000] = "A"
    stream[100_001] = "B"
    stream[100_002] = "A"
    stream[100_003] = "C"
    order = {"A": 2, "B": 1, "C": 1}
    assert shortest_fulfilling_run(stream, order) == (100_000, 100_003)


def _test_large_whole_belt() -> None:
    """Worst case for the window: the answer spans the entire belt."""
    n = 200_000
    stream = ["A"] * (n - 1) + ["B"]
    assert shortest_fulfilling_run(stream, {"A": n - 1, "B": 1}) == (0, n - 1)


def _test_large_uniform() -> None:
    """Every position is a candidate; punishes anything quadratic in n."""
    n = 200_000
    stream = ["A"] * n
    assert shortest_fulfilling_run(stream, {"A": 1}) == (0, 0)
    assert shortest_fulfilling_run(stream, {"A": n}) == (0, n - 1)
    assert shortest_fulfilling_run(stream, {"A": n + 1}) is None


def _test_large_shuffled() -> None:
    """Randomised 200k belt; the answer is pinned by construction.

    Filler-only belt, then one planted minimal window, then a shuffle-proof
    check: the planted window is the unique stretch of length 3 holding P,Q,R.
    """
    n = 200_000
    rng = _Lcg(0xB5026F5AA96619E9)
    filler = ["Z", "Y", "X"]
    stream = [filler[rng.below(3)] for _ in range(n)]
    plant = 173_456
    stream[plant] = "P"
    stream[plant + 1] = "Q"
    stream[plant + 2] = "R"
    assert shortest_fulfilling_run(stream, {"P": 1, "Q": 1, "R": 1}) == (
        plant,
        plant + 2,
    )


if __name__ == "__main__":
    _test_examples()
    _test_edges()
    _test_random_against_brute()
    _test_large_planted()
    _test_large_whole_belt()
    _test_large_uniform()
    _test_large_shuffled()
    print("All tests passed.")
