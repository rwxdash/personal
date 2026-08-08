"""Reference solution — Gateway Peak Concurrency.

Verified against
problems/intervals-sweep-line/01-gateway-peak-concurrency/python/challenge.py.
O(n log n) time, O(n) space.
"""

from typing import List, Tuple


def peak_concurrency(sessions: List[Tuple[int, int]]) -> Tuple[int, int]:
    # Concurrency is a step function that changes only at the 2n instants where
    # a session opens or closes, so the 10^9 coordinate range never matters --
    # only the number of sessions does.
    events: List[Tuple[int, int]] = []
    for start, end in sessions:
        events.append((start, +1))
        events.append((end, -1))

    # Tuples sort lexicographically, so equal timestamps fall through to
    # comparing deltas, and -1 < +1 puts every release ahead of every claim at
    # the same instant. THAT ordering is the half-open semantics: a session
    # ending at t has already let go before anything claims a slot at t.
    # Reverse it and back-to-back sessions wrongly read as overlapping.
    events.sort()

    current = 0
    best = 0
    best_time = 0

    for time, delta in events:
        current += delta
        # Strict `>` keeps the EARLIEST instant at which the peak is reached:
        # events are visited in non-decreasing time order, so a later tie never
        # overwrites the recorded time. `>=` would report the last such instant.
        if current > best:
            best = current
            best_time = time

    # Zero-length sessions emit -1 and +1 at the same timestamp, and with
    # releases ordered first the counter dips by one before recovering. That is
    # harmless: a dip can never create a new maximum, and the net change across
    # the timestamp is zero. No filtering needed.
    #
    # When nothing ever overlaps -- empty input, or only zero-length sessions --
    # best stays 0 and best_time stays 0, which is the required (0, 0).
    return (best, best_time)
