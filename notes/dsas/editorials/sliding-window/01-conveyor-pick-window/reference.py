"""Reference solution — Conveyor Pick Window.

Verified against problems/sliding-window/01-conveyor-pick-window/python/challenge.py.
O(n) time, O(k) space.
"""

from typing import Dict, List, Optional, Tuple


def shortest_fulfilling_run(
    stream: List[str],
    order: Dict[str, int],
) -> Optional[Tuple[int, int]]:
    # `missing` counts the number of *units* still needed to fill the order.
    # It is the whole trick: it turns "is this window good?" from an O(k)
    # map comparison into an O(1) integer test.
    missing = sum(order.values())
    have: Dict[str, int] = {}
    best: Optional[Tuple[int, int]] = None
    left = 0

    for right, sku in enumerate(stream):
        need = order.get(sku)
        if need is not None:
            count = have.get(sku, 0)
            # Only a copy that is still *below* the requirement covers a
            # genuinely missing unit. The 5th copy of a SKU we need 2 of
            # changes nothing about how filled the order is.
            if count < need:
                missing -= 1
            have[sku] = count + 1

        # The window is filled; record it, then shrink from the left for as
        # long as it stays filled. Every index leaves the window at most once,
        # so this inner loop is amortised O(1) per step.
        while missing == 0:
            # Strict `<` keeps the earliest window of any tied length, because
            # windows are recorded in increasing order of `left`.
            if best is None or (right - left) < (best[1] - best[0]):
                best = (left, right)

            out = stream[left]
            need_out = order.get(out)
            if need_out is not None:
                have[out] -= 1
                # Dropping below the requirement is what re-opens a deficit.
                if have[out] < need_out:
                    missing += 1
            left += 1

    return best
