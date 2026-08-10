"""Reference solution — CIDR Aggregation.

Verified against
problems/bit-manipulation/01-cidr-aggregation/python/challenge.py.
O(len(ranges)) time, O(1) space beyond the output.
"""

from typing import List, Tuple


def smallest_cidr(ranges: List[Tuple[int, int]]) -> List[Tuple[int, int, bool]]:
    out: List[Tuple[int, int, bool]] = []

    for start, end in ranges:
        # XOR is zero wherever the two addresses agree, so its highest set bit
        # marks the first position where they differ. Everything from there
        # down must be left free by the block.
        diff = start ^ end

        # bit_length is 0 when diff is 0, which is exactly the start == end
        # case: no free bits, so a /32. No branch needed.
        free = diff.bit_length()
        prefix_len = 32 - free

        # Clear the free low bits to get the network address.
        network = start & ~((1 << free) - 1)

        # The block spans 2^free addresses starting at `network`. It equals the
        # requested range only when BOTH ends line up -- checking only the start
        # would call (5, 6) exact.
        size = 1 << free
        exact = network == start and network + size - 1 == end

        out.append((network, prefix_len, exact))

    return out
