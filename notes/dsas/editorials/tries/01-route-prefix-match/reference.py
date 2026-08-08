"""Reference solution — Route Prefix Match.

Verified against problems/tries/01-route-prefix-match/python/challenge.py.
O(R + P) time in the total character counts, O(R) space.
"""

from typing import Dict, List


def longest_route_match(routes: List[str], paths: List[str]) -> List[int]:
    # Trie held in parallel arrays rather than objects: faster, and it keeps
    # everything flat so there is no deep structure to walk or free.
    #
    # children[i] maps a character to a node id; terminal[i] records whether a
    # registered route ENDS at node i. Those two facts are different -- a node
    # existing only means some route passes through it -- and conflating them
    # makes "/api/v2" wrongly report a match for "/api/xyz".
    children: List[Dict[str, int]] = [{}]
    terminal: List[bool] = [False]

    for route in routes:
        node = 0
        for ch in route:
            nxt = children[node].get(ch)
            if nxt is None:
                nxt = len(children)
                children.append({})
                terminal.append(False)
                children[node][ch] = nxt
            node = nxt
        # A duplicate route just re-marks the same node; no handling needed.
        terminal[node] = True

    result: List[int] = []
    for path in paths:
        node = 0
        # The empty route is checked BEFORE consuming any characters. Doing it
        # inside the loop misses it entirely and turns 0 into -1.
        best = 0 if terminal[0] else -1
        for depth, ch in enumerate(path, start=1):
            nxt = children[node].get(ch)
            if nxt is None:
                # No route continues this way, so none can match any further.
                # Breaking rather than reading the rest of the path is what
                # keeps the query cost proportional to the match, not the query.
                break
            node = nxt
            if terminal[node]:
                # depth is exactly the length of the route ending here.
                best = depth
        result.append(best)

    return result
