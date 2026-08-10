"""
Object Key Canonicalisation
problems/string-manipulation/01-object-key-canonical

Fill in `canonical_key`, then run:

    python3 challenge.py

Statement: ../PROBLEM.md   Stuck? ../HINTS.md
"""

from typing import Optional


def canonical_key(raw: str) -> Optional[str]:
    """Reduce a user-supplied object key to its canonical form.

    Segments are separated by '/'. A segment of "." contributes nothing; ".."
    removes the previous surviving segment; an empty segment (from "//", a
    leading '/' or a trailing '/') contributes nothing; anything else is an
    ordinary name and is kept.

    ONLY exactly "." and exactly ".." are special -- "...", "..a", ".hidden"
    and "a..b" are ordinary names.

    The canonical form is the survivors joined by single '/', with no leading
    and no trailing slash. Zero survivors is the empty string, which is a valid
    result meaning "the top level itself".

    A ".." with no surviving segment to remove is a path traversal attempt and
    rejects the whole key -- including when valid segments follow it.

    Args:
        raw: The key, 0 <= len(raw) <= 1_000_000. Characters are lowercase
             letters, digits, '.', '-', '_' and '/'.

    Returns:
        The canonical key, or None if the key traverses above the top level.
        Note "" and None are different results.

    Required: O(n) time, O(n) space.
    """
    raise NotImplementedError


# ============================ TESTS — do not edit ============================
# The random cross-check resolves each key with an independent character-level
# scanner that never splits the string, so it shares no structure with the
# intended approach.


class _Lcg:
    """Deterministic 64-bit LCG, byte-identical to the Rust test generator."""

    _MASK = (1 << 64) - 1

    def __init__(self, seed: int) -> None:
        self._s = seed & self._MASK

    def below(self, n: int) -> int:
        self._s = (self._s * 6364136223846793005 + 1442695040888963407) & self._MASK
        return self._s % n


def _brute(raw: str) -> Optional[str]:
    """Character-level scanner: accumulate a segment, resolve at each '/'."""
    kept = []
    seg = []
    i = 0
    n = len(raw)
    while i <= n:
        if i == n or raw[i] == "/":
            s = "".join(seg)
            seg = []
            if s == "" or s == ".":
                pass
            elif s == "..":
                if not kept:
                    return None
                kept.pop()
            else:
                kept.append(s)
        else:
            seg.append(raw[i])
        i += 1
    out = ""
    for idx, part in enumerate(kept):
        if idx:
            out += "/"
        out += part
    return out


def _test_examples() -> None:
    assert canonical_key("uploads//2024/../2025/./report.pdf") == "uploads/2025/report.pdf"
    assert canonical_key("/logs/app/") == "logs/app"
    assert canonical_key("a/../..") is None
    assert canonical_key("a/b/../..") == ""
    assert canonical_key("..hidden/.../a..b/.") == "..hidden/.../a..b"


def _test_empty_vs_rejected() -> None:
    """"" and None are different results.

    These fail if a traversal above the top level is silently clamped instead
    of rejected -- the classic path-traversal bug.
    """
    assert canonical_key("a/b/../..") == "", "cancels to the top level: valid"
    assert canonical_key("a/../..") is None, "one .. too many: rejected"
    assert canonical_key("..") is None
    assert canonical_key("/..") is None
    assert canonical_key("../a") is None
    assert canonical_key("a/../../b") is None, "rejection is not undone by later segments"
    assert canonical_key("a/..") == ""
    assert canonical_key("") == ""
    assert canonical_key("/") == ""
    assert canonical_key("///") == ""
    assert canonical_key(".") == ""
    assert canonical_key("./././.") == ""


def _test_dots_that_are_not_special() -> None:
    """Only exactly '.' and exactly '..' are special.

    These fail for any check based on startswith, 'contains a dot', or
    counting dots rather than exact equality.
    """
    assert canonical_key("...") == "..."
    assert canonical_key("....") == "...."
    assert canonical_key("..a") == "..a"
    assert canonical_key("a..") == "a.."
    assert canonical_key("a..b") == "a..b"
    assert canonical_key(".hidden") == ".hidden"
    assert canonical_key(".../..") == "", "... is a name, so .. cancels it"
    assert canonical_key(".../../..") is None
    assert canonical_key("a/.../b") == "a/.../b"
    assert canonical_key("a/..../..") == "a"


def _test_edges() -> None:
    # Single ordinary segment, with and without surrounding slashes.
    assert canonical_key("file.txt") == "file.txt"
    assert canonical_key("/file.txt") == "file.txt"
    assert canonical_key("file.txt/") == "file.txt"
    assert canonical_key("/file.txt/") == "file.txt"
    # Repeated slashes everywhere.
    assert canonical_key("//a//b//") == "a/b"
    assert canonical_key("////a////") == "a"
    # Interleaved dots.
    assert canonical_key("a/./b/./c") == "a/b/c"
    assert canonical_key("./a/.") == "a"
    # Up and back down.
    assert canonical_key("a/b/../c") == "a/c"
    assert canonical_key("a/b/c/../../d") == "a/d"
    assert canonical_key("a/../b/../c") == "c"
    # Trailing '..' after a rebuild.
    assert canonical_key("a/b/../../c/..") == ""
    # Names using the other allowed characters.
    assert canonical_key("my-bucket_2/v1.2.3/file_name-01.tar.gz") == (
        "my-bucket_2/v1.2.3/file_name-01.tar.gz"
    )
    # Digits only.
    assert canonical_key("2024/01/02") == "2024/01/02"


def _test_random_against_brute() -> None:
    rng = _Lcg(0x2DE92C6F592B0275)
    pieces = ["a", "b", ".", "..", "...", "", "x1", ".hid"]
    for _ in range(2_000):
        count = rng.below(9)
        segs = [pieces[rng.below(len(pieces))] for _ in range(count)]
        raw = "/".join(segs)
        # Sometimes add a leading or trailing slash on top.
        if rng.below(3) == 0:
            raw = "/" + raw
        if rng.below(3) == 0:
            raw = raw + "/"
        got = canonical_key(raw)
        want = _brute(raw)
        assert got == want, "raw=%r got=%r want=%r" % (raw, got, want)


def _test_large_deep_nesting() -> None:
    """A key 100k segments deep."""
    depth = 100_000
    segs = ["seg%d" % i for i in range(depth)]
    raw = "/".join(segs)
    assert canonical_key(raw) == raw
    # Now walk all the way back up: valid, resolving to the top level.
    raw_up = raw + "/" + "/".join([".."] * depth)
    assert canonical_key(raw_up) == ""
    # One step too far: rejected.
    assert canonical_key(raw_up + "/..") is None


def _test_large_all_dots() -> None:
    """500k '.' segments contribute nothing."""
    n = 500_000
    assert canonical_key("/".join(["."] * n)) == ""
    assert canonical_key("a/" + "/".join(["."] * n)) == "a"


def _test_large_all_slashes() -> None:
    """A key that is nothing but separators."""
    assert canonical_key("/" * 1_000_000) == ""


def _test_large_zigzag() -> None:
    """Alternating descend and ascend keeps the stack shallow but busy."""
    n = 200_000
    raw = "/".join(["a", ".."] * n)
    assert canonical_key(raw) == ""
    # Ending on a descend leaves exactly one segment.
    assert canonical_key(raw + "/a") == "a"


def _test_large_rejected_early() -> None:
    """A traversal at the very start, followed by a huge valid tail.

    The whole key is rejected regardless of what follows.
    """
    tail = "/".join("seg%d" % i for i in range(100_000))
    assert canonical_key("../" + tail) is None


def _test_large_no_quadratic_join() -> None:
    """200k surviving segments: building the result must not be quadratic."""
    n = 200_000
    segs = ["s%d" % i for i in range(n)]
    raw = "/".join(segs)
    out = canonical_key(raw)
    assert out is not None
    assert len(out) == len(raw)
    assert out == raw


if __name__ == "__main__":
    _test_examples()
    _test_empty_vs_rejected()
    _test_dots_that_are_not_special()
    _test_edges()
    _test_random_against_brute()
    _test_large_deep_nesting()
    _test_large_all_dots()
    _test_large_all_slashes()
    _test_large_zigzag()
    _test_large_rejected_early()
    _test_large_no_quadratic_join()
    print("All tests passed.")
