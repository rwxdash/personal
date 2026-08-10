"""Reference solution — Settlement Runs.

Verified against
problems/prefix-sums/01-settlement-runs/python/challenge.py.
O(n) time, O(min(n, m)) space.
"""

from typing import Dict, List, Tuple


def settlement_runs(deltas: List[int], m: int) -> Tuple[int, int]:
    # A run covering positions i..j-1 sums to P[j] - P[i], where P is the array
    # of running totals. That difference is divisible by m exactly when P[i] and
    # P[j] share a residue mod m -- so the problem is "count pairs of equal
    # residues", never "examine runs".
    #
    # Python's % already normalises into 0..m-1 for positive m. Rust's does not
    # (see reference.rs); a hand-rolled remainder must do it explicitly.
    seen_count: Dict[int, int] = {0: 1}  # the empty prefix, P[0] = 0
    first_index: Dict[int, int] = {0: 0}  # earliest position of each residue

    total = 0
    count = 0
    longest = 0

    for j in range(1, len(deltas) + 1):
        total += deltas[j - 1]
        r = total % m

        # Every earlier prefix sharing this residue closes one settleable run
        # ending here. Reading the tally BEFORE incrementing is what stops a
        # prefix being paired with itself.
        count += seen_count.get(r, 0)
        seen_count[r] = seen_count.get(r, 0) + 1

        # Only the FIRST occurrence of a residue can start the longest run, so
        # never overwrite it.
        if r in first_index:
            span = j - first_index[r]
            if span > longest:
                longest = span
        else:
            first_index[r] = j

    return (count, longest)
