"""
Compaction Windows
problems/binary-search-on-answer/01-compaction-windows

Fill in `min_window_bytes`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List


def min_window_bytes(sizes: List[int], checkpoints: List[bool], k: int) -> int:
    """Minimise the largest compaction window's byte total.

    Split `sizes` into at most `k` contiguous, non-empty windows covering every
    segment in order. A checkpoint segment must be the FIRST segment of its
    window; `checkpoints[0]` is ignored, since segment 0 always begins the
    first window.

    Args:
        sizes:       Segment byte counts in chronological order.
                     1 <= n <= 200_000, each value 1 ..= 10^9.
        checkpoints: Whether each segment must begin a window. Same length.
        k:           Maximum number of windows, 1 <= k <= n.

    Guaranteed: at most k - 1 checkpoints among indices 1..n-1, so a valid
    split always exists.

    Returns:
        The smallest achievable largest-window byte total. Can reach 2 * 10^14.

    Required: O(n log T) time where T is the total byte count, O(1) extra space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check uses an O(n^2 k) dynamic program over split points,
# which shares no reasoning with a search over candidate answers.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(sizes: List[int], checkpoints: List[bool], k: int) -> int:
    """Exact answer by DP over (segments consumed, windows used). O(n^2 k)."""
    n = len(sizes)
    prefix = [0] * (n + 1)
    for i, s in enumerate(sizes):
        prefix[i + 1] = prefix[i] + s

    inf = float("inf")
    dp = [[inf] * (k + 1) for _ in range(n + 1)]
    dp[0][0] = 0
    for i in range(1, n + 1):
        for j in range(1, k + 1):
            # Last window covers segments [t .. i-1].
            for t in range(i - 1, -1, -1):
                if dp[t][j - 1] < inf:
                    cand = max(dp[t][j - 1], prefix[i] - prefix[t])
                    if cand < dp[i][j]:
                        dp[i][j] = cand
                # A checkpoint at t means no window may start before t and
                # still contain t, so stop widening this window.
                if t > 0 and checkpoints[t]:
                    break
    best = min(dp[n][j] for j in range(1, k + 1))
    return int(best)


def _test_examples() -> None:
    assert min_window_bytes([7, 2, 5, 10, 8], [False] * 5, 2) == 18
    assert min_window_bytes([7, 2, 5, 10, 8], [False] * 5, 3) == 14
    assert min_window_bytes(
        [7, 2, 5, 10, 8], [False, False, True, False, False], 3
    ) == 15
    assert min_window_bytes([4, 9, 2], [False] * 3, 3) == 9


def _test_edges() -> None:
    # Single segment.
    assert min_window_bytes([5], [False], 1) == 5
    assert min_window_bytes([5], [True], 1) == 5, "checkpoints[0] is ignored"
    # One window: everything together.
    assert min_window_bytes([1, 2, 3, 4], [False] * 4, 1) == 10
    # A window per segment: the answer is the largest segment.
    assert min_window_bytes([1, 2, 3, 4], [False] * 4, 4) == 4
    # More windows than needed changes nothing once each segment is alone.
    assert min_window_bytes([3, 3, 3], [False] * 3, 3) == 3
    # Uniform sizes split evenly.
    assert min_window_bytes([2] * 6, [False] * 6, 3) == 4
    assert min_window_bytes([2] * 6, [False] * 6, 2) == 6
    # One dominant segment sets the floor.
    assert min_window_bytes([1, 1, 100, 1, 1], [False] * 5, 3) == 100
    # Checkpoint on the last segment forces it alone.
    assert min_window_bytes(
        [1, 1, 1, 50], [False, False, False, True], 2
    ) == 50
    # Every segment after the first is a checkpoint: k windows are all forced.
    assert min_window_bytes([4, 9, 2], [False, True, True], 3) == 9
    # Checkpoint that costs nothing, because the cut was optimal anyway.
    assert min_window_bytes(
        [5, 5, 5, 5], [False, False, True, False], 2
    ) == 10


def _test_checkpoints_bind() -> None:
    """A forced cut must fire even when the current window has room left.

    Each pair below is the same input with and without one checkpoint; the
    checkpointed version must be strictly worse.
    """
    sizes = [1, 1, 1, 1, 8]
    assert min_window_bytes(sizes, [False] * 5, 2) == 8
    # Forcing a window to start at index 1 strands [1] alone, so the remaining
    # 3 windows' worth of work must fit in one: 1 + 1 + 1 + 8 = 11.
    assert min_window_bytes(sizes, [False, True, False, False, False], 2) == 11
    # Two checkpoints with k = 3.
    assert min_window_bytes([6, 6, 6, 6], [False, True, True, False], 3) == 12
    # Consecutive checkpoints each start their own window.
    assert min_window_bytes([1, 2, 3], [False, True, True], 3) == 3


def _test_random_against_brute() -> None:
    rng = _Lcg(0x8AA07D6C1F6D3A2B)
    for _ in range(400):
        n = 1 + rng.below(9)
        sizes = [1 + rng.below(12) for _ in range(n)]
        # Choose k first, then place at most k-1 checkpoints so the instance
        # is guaranteed solvable.
        k = 1 + rng.below(n)
        checkpoints = [False] * n
        budget = k - 1
        for i in range(1, n):
            if budget > 0 and rng.below(3) == 0:
                checkpoints[i] = True
                budget -= 1
        got = min_window_bytes(sizes, checkpoints, k)
        want = _brute(sizes, checkpoints, k)
        assert got == want, "sizes=%r checkpoints=%r k=%d got=%d want=%d" % (
            sizes,
            checkpoints,
            k,
            got,
            want,
        )


def _test_large_uniform() -> None:
    """200k equal segments into k windows: a clean closed form."""
    n = 200_000
    sizes = [1000] * n
    for k in (1, 2, 4, 5, 1000):
        # Ceiling division of segments per window, times the segment size.
        per = (n + k - 1) // k
        assert min_window_bytes(sizes, [False] * n, k) == per * 1000


def _test_large_single_dominant() -> None:
    """One huge segment sets the floor no matter how many windows are allowed."""
    n = 200_000
    sizes = [1] * n
    sizes[123_456] = 10**9
    assert min_window_bytes(sizes, [False] * n, 1000) == 10**9


def _test_large_all_max() -> None:
    """Every segment at the value ceiling: total is 2 * 10^14, needs 64 bits."""
    n = 200_000
    sizes = [10**9] * n
    assert min_window_bytes(sizes, [False] * n, 1) == n * 10**9
    assert min_window_bytes(sizes, [False] * n, 2) == (n // 2) * 10**9
    assert min_window_bytes(sizes, [False] * n, n) == 10**9


def _test_large_max_windows() -> None:
    """k = n means every segment is alone."""
    n = 200_000
    rng = _Lcg(0x5C4E2A1B9F03D7E6)
    sizes = [1 + rng.below(10**9) for _ in range(n)]
    assert min_window_bytes(sizes, [False] * n, n) == max(sizes)


def _test_large_checkpoint_every_other() -> None:
    """Half the segments are checkpoints, so the split is almost fully forced.

    A checkpoint at every odd index means 100_000 forced cuts, so k must be at
    least 100_001 for the instance to be solvable at all. At exactly that k the
    split is fully determined: windows are [0], [1,2], [3,4], ..., [n-1].
    """
    n = 200_000
    sizes = [1] * n
    sizes[7] = 5  # index 7 is odd, so it starts a window; its pair is (7, 8)
    checkpoints = [i % 2 == 1 for i in range(n)]
    forced = sum(checkpoints[1:])
    k = forced + 1  # exactly enough windows; every cut is forced
    assert k == 100_001
    # Every window is a (odd, even) pair totalling 2, except [0] and [n-1]
    # which hold one segment each, and [7, 8] which holds 5 + 1 = 6.
    assert min_window_bytes(sizes, checkpoints, k) == 6


def _test_large_binary_search_range() -> None:
    """Forces many search iterations: a wide value range with a unique answer."""
    n = 100_000
    sizes = [10**9 if i == 0 else 1 for i in range(n)]
    # Two windows: the giant alone, and the rest. Largest is the giant.
    assert min_window_bytes(sizes, [False] * n, 2) == 10**9
    # One window: everything.
    assert min_window_bytes(sizes, [False] * n, 1) == 10**9 + (n - 1)


if __name__ == "__main__":
    _test_examples()
    _test_edges()
    _test_checkpoints_bind()
    _test_random_against_brute()
    _test_large_uniform()
    _test_large_single_dominant()
    _test_large_all_max()
    _test_large_max_windows()
    _test_large_checkpoint_every_other()
    _test_large_binary_search_range()
    print("All tests passed.")
