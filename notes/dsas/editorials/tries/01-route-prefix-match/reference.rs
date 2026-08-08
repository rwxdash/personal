//! Reference solution — Route Prefix Match.
//!
//! Verified against problems/tries/01-route-prefix-match/rust.
//! O(R + P) time in the total character counts, O(R) space.

use std::collections::HashMap;

pub fn longest_route_match(routes: &[String], paths: &[String]) -> Vec<i64> {
    // Trie held in parallel arrays rather than linked nodes: flat, fast, and
    // nothing deep to walk or drop.
    //
    // children[i] maps a byte to a node id; terminal[i] records whether a
    // registered route ENDS at node i. Those two facts are different -- a node
    // existing only means some route passes through it -- and conflating them
    // makes "/api/v2" wrongly report a match for "/api/xyz".
    let mut children: Vec<HashMap<u8, usize>> = vec![HashMap::new()];
    let mut terminal: Vec<bool> = vec![false];

    for route in routes {
        let mut node = 0usize;
        for &b in route.as_bytes() {
            let next = match children[node].get(&b) {
                Some(&n) => n,
                None => {
                    let n = children.len();
                    children.push(HashMap::new());
                    terminal.push(false);
                    children[node].insert(b, n);
                    n
                }
            };
            node = next;
        }
        // A duplicate route just re-marks the same node; no handling needed.
        terminal[node] = true;
    }

    paths
        .iter()
        .map(|path| {
            let mut node = 0usize;
            // The empty route is checked BEFORE consuming any characters.
            // Doing it inside the loop misses it and turns 0 into -1.
            let mut best: i64 = if terminal[0] { 0 } else { -1 };
            for (idx, &b) in path.as_bytes().iter().enumerate() {
                match children[node].get(&b) {
                    Some(&n) => {
                        node = n;
                        if terminal[node] {
                            // idx + 1 is exactly the length of the route
                            // ending here.
                            best = (idx + 1) as i64;
                        }
                    }
                    // No route continues this way, so none can match further.
                    // Breaking rather than reading the rest of the path keeps
                    // the query cost proportional to the match.
                    None => break,
                }
            }
            best
        })
        .collect()
}
