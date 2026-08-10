"""Reference solution — Extent Cover.

Greedy from the left. At each position take the largest extent that is legal
there; both limits on "largest" are one-instruction questions about a number's
binary representation.
"""

from typing import List, Tuple


def extent_cover(ranges: List[Tuple[int, int]]) -> List[List[Tuple[int, int]]]:
    out: List[List[Tuple[int, int]]] = []

    for start, end in ranges:
        extents: List[Tuple[int, int]] = []
        pos = start
        remaining = end - start + 1

        while remaining > 0:
            # Limit 1 -- alignment. An extent of size 2^k needs pos's low k
            # bits to be zero, so the biggest k allowed here is the number of
            # trailing zeros in pos. Every power of two divides 0, so position
            # 0 has no alignment limit at all; 64 stands in for "unlimited" and
            # is always beaten by the fit limit below (which is at most 48).
            align = 64 if pos == 0 else (pos & -pos).bit_length() - 1

            # Limit 2 -- fit. The largest power of two that is <= remaining is
            # 2^(bit_length - 1), since 2^f <= remaining < 2^(f+1).
            fit = remaining.bit_length() - 1

            # Both are exponents, so the min is taken before any shift. Doing
            # it the other way round would compute 1 << 64 at position 0.
            k = align if align < fit else fit
            size = 1 << k

            extents.append((pos, size))
            pos += size
            remaining -= size

        out.append(extents)

    return out
