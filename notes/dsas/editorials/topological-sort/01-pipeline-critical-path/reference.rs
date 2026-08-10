//! Reference solution — Pipeline Critical Path.
//!
//! Verified against problems/topological-sort/01-pipeline-critical-path/rust.
//! O(n + m) time and space.

use std::collections::VecDeque;

pub fn pipeline_makespan(durations: &[i64], ready: &[i64], deps: &[(usize, usize)]) -> Option<i64> {
    let n = durations.len();

    // Build successor lists and in-degrees. Every entry of `deps` is counted
    // exactly once in BOTH structures -- duplicates are deliberately NOT
    // collapsed. Deduplicating one structure but not the other desynchronises
    // the counters and either strands a node or enqueues it twice.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut indeg: Vec<usize> = vec![0; n];
    for &(a, b) in deps {
        adj[a].push(b);
        indeg[b] += 1;
    }

    // start[i] = earliest instant job i is currently known to be able to begin.
    // It starts at the artifact availability time and only ever increases as
    // predecessors report their finish times.
    let mut start = ready.to_vec();

    let mut queue: VecDeque<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
    let mut processed = 0usize;
    let mut answer = 0i64;

    while let Some(u) = queue.pop_front() {
        processed += 1;

        // INVARIANT: a node is enqueued only once its in-degree hits zero, which
        // happens only after every incoming edge was relaxed, which happens only
        // after every predecessor was popped. So start[u] is final here, and so
        // is finish_u.
        let finish_u = start[u] + durations[u];
        if finish_u > answer {
            answer = finish_u; // max over ALL nodes, not just sinks
        }

        for idx in 0..adj[u].len() {
            let v = adj[u][idx];
            if finish_u > start[v] {
                start[v] = finish_u;
            }
            indeg[v] -= 1;
            if indeg[v] == 0 {
                queue.push_back(v);
            }
        }
    }

    // Kahn's algorithm pays for itself twice: the nodes it never reached are
    // exactly those trapped in or behind a cycle. A self-loop (a, a) gives `a`
    // an in-degree it can never shed, so it is caught by the same test.
    if processed < n {
        return None;
    }
    Some(answer)
}
