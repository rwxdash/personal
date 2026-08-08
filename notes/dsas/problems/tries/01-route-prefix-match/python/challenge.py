"""
Route Prefix Match
problems/tries/01-route-prefix-match

Fill in `longest_route_match`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import List


def longest_route_match(routes: List[str], paths: List[str]) -> List[int]:
    """Longest registered route that is a prefix of each request path.

    Matching is on raw characters, not path segments: a route matches a path
    when the route is a literal prefix of it. A route equal to the path counts
    as a match. A route longer than the path never matches.

    Args:
        routes: Registered route prefixes. Up to 200_000 entries, 1_000_000
                characters in total. May contain duplicates and the empty
                string. Characters are lowercase a-z and '/'.
        paths:  Request paths. Up to 200_000 entries, 1_000_000 characters in
                total.

    Returns:
        One entry per path: the LENGTH of the longest matching route, or -1 if
        no route matches. Note 0 and -1 differ -- a registered empty route
        matches every path with length 0.

    Required: O(R + P) time in the total character counts, O(R) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check compares every path against every route directly,
# which shares no structure with a prefix tree.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(routes: List[str], paths: List[str]) -> List[int]:
    """Check every path against every route. O(len(paths) * len(routes) * L)."""
    out = []
    for path in paths:
        best = -1
        for r in routes:
            if len(r) <= len(path) and path[: len(r)] == r and len(r) > best:
                best = len(r)
        out.append(best)
    return out


def _test_examples() -> None:
    assert longest_route_match(
        ["/api", "/api/v2", "/health"],
        ["/api/v2/users", "/api/v1/users", "/health", "/metrics"],
    ) == [7, 4, 7, -1]
    assert longest_route_match(["", "/api"], ["/api/x", "/other", ""]) == [4, 0, 0]
    assert longest_route_match(
        ["/api/v2/users"], ["/api", "/api/v2/users", "/api/v2/users/42"]
    ) == [-1, 13, 13]
    assert longest_route_match(["/api"], ["/apixyz", "/ap"]) == [4, -1]


def _test_empty_route_vs_no_match() -> None:
    """0 and -1 are different answers.

    These fail if the root's terminal flag is checked inside the character loop
    rather than before it.
    """
    assert longest_route_match([""], ["anything"]) == [0]
    assert longest_route_match([""], [""]) == [0]
    assert longest_route_match(["a"], [""]) == [-1]
    assert longest_route_match([], [""]) == [-1]
    assert longest_route_match(["", "a"], ["", "a", "b"]) == [0, 1, 0]


def _test_edges() -> None:
    # No routes at all.
    assert longest_route_match([], ["/a", "/b"]) == [-1, -1]
    # No paths at all.
    assert longest_route_match(["/a"], []) == []
    # Both empty.
    assert longest_route_match([], []) == []
    # Duplicate routes change nothing.
    assert longest_route_match(["/a", "/a", "/a"], ["/abc"]) == [2]
    # Exact match.
    assert longest_route_match(["/abc"], ["/abc"]) == [4]
    # Nested routes: the deepest one wins.
    assert longest_route_match(["/a", "/ab", "/abc"], ["/abcd"]) == [4]
    assert longest_route_match(["/abc", "/ab", "/a"], ["/abcd"]) == [4]
    # A route that exists only as an internal node is not a match.
    assert longest_route_match(["/abc"], ["/ab"]) == [-1]
    assert longest_route_match(["/abc"], ["/abz"]) == [-1]
    # Sibling branches do not interfere.
    assert longest_route_match(["/ax", "/ay"], ["/ax1", "/ay1", "/az1"]) == [3, 3, -1]
    # The same path queried repeatedly.
    assert longest_route_match(["/a"], ["/ab"] * 5) == [2] * 5
    # Single characters.
    assert longest_route_match(["a", "b"], ["a", "b", "c"]) == [1, 1, -1]
    # Only slashes.
    assert longest_route_match(["/", "//"], ["///"]) == [2]


def _test_random_against_brute() -> None:
    rng = _Lcg(0x19A4C116B8D2D0C8)
    alphabet = "ab/"
    for _ in range(400):
        nr = rng.below(7)
        np_ = rng.below(7)
        routes = [
            "".join(alphabet[rng.below(3)] for _ in range(rng.below(5)))
            for _ in range(nr)
        ]
        paths = [
            "".join(alphabet[rng.below(3)] for _ in range(rng.below(6)))
            for _ in range(np_)
        ]
        got = longest_route_match(routes, paths)
        want = _brute(routes, paths)
        assert got == want, "routes=%r paths=%r got=%r want=%r" % (
            routes,
            paths,
            got,
            want,
        )


def _test_large_shared_prefix() -> None:
    """50k routes all sharing a long prefix: the naive rescans it every time."""
    base = "/service/v1/resource/"
    routes = [base + "%05d" % i for i in range(50_000)]
    paths = [base + "%05d" % i + "/detail" for i in range(0, 50_000, 5)]
    expected = [len(base) + 5] * len(paths)
    assert longest_route_match(routes, paths) == expected


def _test_large_nested_ladder() -> None:
    """Routes nested inside one another; the deepest must win every time."""
    depth = 2_000
    routes = ["a" * k for k in range(1, depth + 1)]
    paths = ["a" * depth, "a" * (depth // 2), "a" * (depth + 500), "b" + "a" * depth]
    assert longest_route_match(routes, paths) == [depth, depth // 2, depth, -1]


def _test_large_no_match() -> None:
    """100k paths that share no first character with any route: instant misses."""
    routes = ["/api/" + "%06d" % i for i in range(50_000)]
    paths = ["zzz%06d" % i for i in range(100_000)]
    assert longest_route_match(routes, paths) == [-1] * 100_000


def _test_large_catch_all() -> None:
    """An empty route registered alongside 50k real ones."""
    routes = [""] + ["/svc/" + "%05d" % i for i in range(50_000)]
    paths = ["/svc/00042/x", "/nowhere", "", "/svc/49999"]
    assert longest_route_match(routes, paths) == [10, 0, 0, 10]


def _test_large_many_duplicates() -> None:
    """The same route registered 200k times."""
    routes = ["/dup"] * 200_000
    paths = ["/dup/x"] * 1_000
    assert longest_route_match(routes, paths) == [4] * 1_000


def _test_large_long_strings() -> None:
    """A few very long routes and paths, near the character budget."""
    long_route = "/" + "ab" * 200_000  # 400_001 chars
    routes = [long_route, "/ab"]
    paths = [long_route + "/tail", "/ab" + "c", long_route[:100]]
    assert longest_route_match(routes, paths) == [len(long_route), 3, 3]


def _test_large_wide_fanout() -> None:
    """A root with many distinct single-character branches."""
    letters = "abcdefghijklmnopqrstuvwxyz"
    routes = [c for c in letters]
    paths = [c + "tail" for c in letters] + ["/nope"]
    assert longest_route_match(routes, paths) == [1] * 26 + [-1]


if __name__ == "__main__":
    _test_examples()
    _test_empty_route_vs_no_match()
    _test_edges()
    _test_random_against_brute()
    _test_large_shared_prefix()
    _test_large_nested_ladder()
    _test_large_no_match()
    _test_large_catch_all()
    _test_large_many_duplicates()
    _test_large_long_strings()
    _test_large_wide_fanout()
    print("All tests passed.")
