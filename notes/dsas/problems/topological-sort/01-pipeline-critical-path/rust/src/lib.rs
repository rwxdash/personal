//! Pipeline Critical Path
//! problems/topological-sort/01-pipeline-critical-path
//!
//! Fill in `pipeline_makespan`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Earliest instant at which every job in the pipeline has finished.
///
/// Workers are unlimited, so a job starts as soon as it is allowed to:
///
/// ```text
/// start[i]  = max(ready[i], max(finish[a] for every (a, i) in deps))
/// finish[i] = start[i] + durations[i]
/// makespan  = max(finish[i] for all i)
/// ```
///
/// (the inner max is 0 for a job with no dependencies).
///
/// `deps` pairs are `(a, b)` meaning job `a` must finish before job `b`
/// starts. They may contain duplicate pairs and self-loops `(a, a)`, and the
/// graph may be disconnected.
///
/// Returns the makespan in ms (can reach ~2 * 10^14), or `None` if `deps`
/// contains a cycle, since no schedule then exists.
///
/// Required: O(n + m) time and space, m = deps.len().
pub fn pipeline_makespan(durations: &[i64], ready: &[i64], deps: &[(usize, usize)]) -> Option<i64> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check uses a
// fixpoint relaxation plus an explicit reachability cycle check, so it shares
// no machinery with the intended solution.
#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic 64-bit LCG, byte-identical to the Python test generator.
    struct Lcg(u64);

    impl Lcg {
        fn new(seed: u64) -> Self {
            Lcg(seed)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 % n
        }
    }

    /// O(n * (n + m)) reachability check: is any node reachable from itself?
    fn has_cycle(n: usize, deps: &[(usize, usize)]) -> bool {
        let mut succ: Vec<Vec<usize>> = vec![Vec::new(); n];
        for &(a, b) in deps {
            succ[a].push(b);
        }
        for source in 0..n {
            let mut seen = vec![false; n];
            let mut stack = succ[source].clone();
            while let Some(v) = stack.pop() {
                if v == source {
                    return true;
                }
                if !seen[v] {
                    seen[v] = true;
                    stack.extend_from_slice(&succ[v]);
                }
            }
        }
        false
    }

    /// Order-agnostic O(n * m) fixpoint, guarded by an explicit cycle check.
    /// The cycle check cannot be folded into the relaxation: a cycle whose jobs
    /// all have duration 0 reaches a fixpoint immediately.
    fn brute(durations: &[i64], ready: &[i64], deps: &[(usize, usize)]) -> Option<i64> {
        let n = durations.len();
        if has_cycle(n, deps) {
            return None;
        }
        let mut start = ready.to_vec();
        for _ in 0..=n {
            let mut moved = false;
            for &(a, b) in deps {
                let candidate = start[a] + durations[a];
                if candidate > start[b] {
                    start[b] = candidate;
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        (0..n).map(|i| start[i] + durations[i]).max()
    }

    #[test]
    fn examples() {
        assert_eq!(
            pipeline_makespan(&[3, 2, 4], &[0, 0, 0], &[(0, 1), (1, 2)]),
            Some(9)
        );
        assert_eq!(pipeline_makespan(&[3, 2, 4], &[0, 0, 0], &[]), Some(4));
        assert_eq!(
            pipeline_makespan(
                &[1, 5, 2, 1],
                &[0; 4],
                &[(0, 1), (0, 2), (1, 3), (2, 3)]
            ),
            Some(7)
        );
        assert_eq!(pipeline_makespan(&[1, 1], &[0, 10], &[(0, 1)]), Some(11));
        assert_eq!(pipeline_makespan(&[1, 1], &[0, 0], &[(0, 1), (1, 0)]), None);
    }

    #[test]
    fn cycles() {
        // A self-loop is a job that must finish before itself.
        assert_eq!(pipeline_makespan(&[5], &[0], &[(0, 0)]), None);
        // Cycle sitting off to the side of a perfectly fine component.
        assert_eq!(
            pipeline_makespan(&[1, 1, 1, 1], &[0; 4], &[(0, 1), (2, 3), (3, 2)]),
            None
        );
        // Three-node cycle.
        assert_eq!(
            pipeline_makespan(&[1, 1, 1], &[0; 3], &[(0, 1), (1, 2), (2, 0)]),
            None
        );
        // Cycle unreachable from any zero-in-degree node still has to be caught.
        assert_eq!(
            pipeline_makespan(&[1, 1, 1], &[0; 3], &[(0, 1), (1, 2), (2, 1)]),
            None
        );
    }

    #[test]
    fn edges() {
        // Single job.
        assert_eq!(pipeline_makespan(&[7], &[5], &[]), Some(12));
        assert_eq!(pipeline_makespan(&[0], &[0], &[]), Some(0));
        // Zero-duration gate jobs are ordinary nodes.
        assert_eq!(
            pipeline_makespan(&[0, 0, 3], &[0, 0, 0], &[(0, 1), (1, 2)]),
            Some(3)
        );
        // Duplicate edges must not corrupt in-degree bookkeeping.
        assert_eq!(
            pipeline_makespan(&[2, 3], &[0, 0], &[(0, 1), (0, 1), (0, 1)]),
            Some(5)
        );
        // Disconnected: an isolated job with a late artifact sets the makespan.
        assert_eq!(pipeline_makespan(&[1, 1], &[0, 500], &[]), Some(501));
        assert_eq!(
            pipeline_makespan(&[1, 1, 1], &[0, 0, 900], &[(0, 1)]),
            Some(901)
        );
        // ready time on an upstream job propagates downstream.
        assert_eq!(
            pipeline_makespan(&[1, 1, 1], &[100, 0, 0], &[(0, 1), (1, 2)]),
            Some(103)
        );
        // ready time already satisfied by dependencies changes nothing.
        assert_eq!(pipeline_makespan(&[10, 1], &[0, 5], &[(0, 1)]), Some(11));
        // No deps at all, answer is the max finish, not the sum.
        assert_eq!(pipeline_makespan(&[4, 9, 2], &[0, 0, 0], &[]), Some(9));
        // 64-bit range.
        assert_eq!(
            pipeline_makespan(&[1_000_000_000, 1_000_000_000], &[1_000_000_000, 0], &[(0, 1)]),
            Some(3_000_000_000)
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0xA5A5A5A5DEADC0DE);
        for _ in 0..300 {
            let n = (1 + rng.below(9)) as usize;
            let durations: Vec<i64> = (0..n).map(|_| rng.below(6) as i64).collect();
            let ready: Vec<i64> = (0..n).map(|_| rng.below(12) as i64).collect();
            let m = rng.below(14);
            // Unconstrained endpoints, so roughly a third of these graphs are
            // cyclic -- the oracle handles both cases.
            let deps: Vec<(usize, usize)> = (0..m)
                .map(|_| (rng.below(n as u64) as usize, rng.below(n as u64) as usize))
                .collect();
            assert_eq!(
                pipeline_makespan(&durations, &ready, &deps),
                brute(&durations, &ready, &deps),
                "durations={:?} ready={:?} deps={:?}",
                durations,
                ready,
                deps
            );
        }
    }

    #[test]
    fn random_dags_against_brute() {
        // Same cross-check on guaranteed-acyclic graphs, so every answer is a
        // real makespan rather than None.
        let mut rng = Lcg::new(0x0123456789ABCDEF);
        for _ in 0..300 {
            let n = (1 + rng.below(10)) as usize;
            let durations: Vec<i64> = (0..n).map(|_| rng.below(7) as i64).collect();
            let ready: Vec<i64> = (0..n).map(|_| rng.below(15) as i64).collect();
            let m = rng.below(20);
            let mut deps: Vec<(usize, usize)> = Vec::new();
            for _ in 0..m {
                let a = rng.below(n as u64) as usize;
                let b = rng.below(n as u64) as usize;
                if a == b {
                    continue;
                }
                deps.push((a.min(b), a.max(b))); // always points forward: acyclic
            }
            let want = brute(&durations, &ready, &deps);
            assert!(want.is_some());
            assert_eq!(
                pipeline_makespan(&durations, &ready, &deps),
                want,
                "durations={:?} ready={:?} deps={:?}",
                durations,
                ready,
                deps
            );
        }
    }

    #[test]
    fn large_deep_chain() {
        // 200k-deep chain. A recursive traversal overflows the stack here.
        let n = 200_000usize;
        let durations = vec![1i64; n];
        let ready = vec![0i64; n];
        let deps: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        assert_eq!(pipeline_makespan(&durations, &ready, &deps), Some(n as i64));
    }

    #[test]
    fn large_reversed_chain() {
        // The same 200k chain with `deps` listed back to front. Dependency
        // lists arrive in whatever order the config was written in; any
        // approach leaning on the edge list already being usable -- including
        // repeated relaxation passes -- degrades to O(n * m) here.
        let n = 200_000usize;
        let durations = vec![1i64; n];
        let ready = vec![0i64; n];
        let mut deps: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        deps.reverse();
        assert_eq!(pipeline_makespan(&durations, &ready, &deps), Some(n as i64));
    }

    #[test]
    fn large_chain_with_late_artifact() {
        // A late artifact halfway down a deep chain shifts everything after it.
        let n = 200_000usize;
        let durations = vec![1i64; n];
        let mut ready = vec![0i64; n];
        ready[150_000] = 1_000_000_000;
        let deps: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        // finish[150_000] = 10^9 + 1, then +1 per remaining job.
        assert_eq!(
            pipeline_makespan(&durations, &ready, &deps),
            Some(1_000_000_000 + 50_000)
        );
    }

    #[test]
    fn large_fan() {
        // One source, 100k parallel jobs, one sink. Tests the max-not-sum rule.
        let mid = 100_000usize;
        let n = mid + 2; // 0 = source, 1..=mid = parallel, mid+1 = sink
        let mut durations = vec![5i64];
        durations.extend((1..=mid).map(|i| i as i64));
        durations.push(3);
        let ready = vec![0i64; n];
        let mut deps: Vec<(usize, usize)> = (1..=mid).map(|i| (0, i)).collect();
        deps.extend((1..=mid).map(|i| (i, n - 1)));
        // source 0..5; slowest middle job finishes at 5 + mid; sink adds 3.
        assert_eq!(
            pipeline_makespan(&durations, &ready, &deps),
            Some((5 + mid + 3) as i64)
        );
    }

    #[test]
    fn large_cycle() {
        // A 200k chain closed into a loop by a single back edge.
        let n = 200_000usize;
        let durations = vec![1i64; n];
        let ready = vec![0i64; n];
        let mut deps: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        deps.push((n - 1, 0));
        assert_eq!(pipeline_makespan(&durations, &ready, &deps), None);
    }

    #[test]
    fn large_wide_layers() {
        // Layered DAG, ~400k edges, answer known by construction.
        let width = 400usize;
        let layers = 500usize;
        let n = width * layers;
        let durations = vec![2i64; n];
        let ready = vec![0i64; n];
        let mut deps: Vec<(usize, usize)> = Vec::new();
        for layer in 0..layers - 1 {
            let base = layer * width;
            let nxt = base + width;
            for i in 0..width {
                // Two forward edges per node, all pointing to the next layer,
                // so the longest route visits exactly one node per layer.
                deps.push((base + i, nxt + i));
                deps.push((base + i, nxt + (i + 1) % width));
            }
        }
        assert_eq!(deps.len(), 2 * width * (layers - 1));
        assert_eq!(
            pipeline_makespan(&durations, &ready, &deps),
            Some((2 * layers) as i64)
        );
    }

    #[test]
    fn large_disconnected_late_job() {
        // The makespan is a max over all jobs, not just over sinks.
        let n = 200_000usize;
        let durations = vec![1i64; n];
        let mut ready = vec![0i64; n];
        let deps: Vec<(usize, usize)> = (0..n - 2).map(|i| (i, i + 1)).collect();
        ready[n - 1] = 1_000_000_000; // leaves job n-1 isolated and late
        assert_eq!(
            pipeline_makespan(&durations, &ready, &deps),
            Some(1_000_000_001)
        );
    }
}
