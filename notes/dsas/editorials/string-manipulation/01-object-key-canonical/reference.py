"""Reference solution — Object Key Canonicalisation.

Verified against
problems/string-manipulation/01-object-key-canonical/python/challenge.py.
O(n) time, O(n) space.
"""

from typing import List, Optional


def canonical_key(raw: str) -> Optional[str]:
    # The ".." rule removes the MOST RECENT survivor and can never bring it
    # back, which is exactly a stack.
    stack: List[str] = []

    # Splitting is what turns leading, trailing and doubled slashes into empty
    # segments, so one rule ("empty contributes nothing") covers all three
    # instead of three special cases. Note "".split("/") gives [""], a single
    # empty segment, so an empty key falls out correctly with no guard.
    for segment in raw.split("/"):
        if segment == "" or segment == ".":
            continue

        # EXACT equality, not startswith or a substring test: "...", "..a",
        # ".hidden" and "a..b" are all ordinary names.
        if segment == "..":
            if not stack:
                # Reject on the spot rather than clamping at the top level.
                # Clamping is the classic path-traversal vulnerability, and it
                # would turn "a/../.." into "" instead of None.
                return None
            stack.pop()
            continue

        stack.append(segment)

    # join gives no leading or trailing slash, and yields "" for an empty
    # stack -- which is a VALID result meaning "the top level itself", distinct
    # from None. Joining once rather than concatenating in a loop is what keeps
    # this linear.
    return "/".join(stack)
