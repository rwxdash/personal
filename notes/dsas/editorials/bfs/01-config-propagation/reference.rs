//! Reference solution — Config Propagation.
//!
//! Verified against problems/bfs/01-config-propagation/rust.
//! O(n + m) time and space.

use std::collections::VecDeque;

pub fn propagation_rounds(
    n: usize,
    links: &[(usize, usize)],
    seeds: &[usize],
    quarantined: &[bool],
) -> Vec<i64> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(u, v) in links {
        adj[u].push(v);
        adj[v].push(u); // self-loops add u twice; harmless
    }

    // `round_of` doubles as the visited marker: -1 means "not yet reached",
    // which is also the required output for unreachable nodes. One array, so
    // the two can never disagree.
    let mut round_of = vec![-1i64; n];
    let mut queue: VecDeque<usize> = VecDeque::new();

    // MULTI-SOURCE: every seed enters at distance 0 before the sweep starts.
    // Equivalent to a virtual node joined to all seeds, minus its one hop.
    // Because BFS dequeues in non-decreasing distance order, the first time a
    // node is reached its round is already the minimum over ALL seeds -- no
    // per-seed passes and no post-hoc minimum.
    for &s in seeds {
        if round_of[s] == -1 {
            // guards against duplicate seeds
            round_of[s] = 0;
            queue.push_back(s);
        }
    }

    while let Some(u) = queue.pop_front() {
        if quarantined[u] {
            // Dequeued and already recorded (its round was written when it was
            // ENQUEUED), but it expands nothing. Placing the check here rather
            // than at enqueue time is what preserves its own round number --
            // and makes a quarantined seed work with no special case.
            continue;
        }
        for i in 0..adj[u].len() {
            let v = adj[u][i];
            if round_of[v] == -1 {
                round_of[v] = round_of[u] + 1;
                queue.push_back(v);
            }
        }
    }

    round_of
}
