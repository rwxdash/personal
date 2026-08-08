//! Config Propagation
//! problems/bfs/01-config-propagation
//!
//! Fill in `propagation_rounds`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Round at which each node receives the rolled-out config.
///
/// Seeds hold the config at round 0. Each round, every node already holding it
/// forwards it to all direct peers simultaneously. A quarantined node ACCEPTS
/// the config (and records its round) but never forwards it onward — including
/// when it is itself a seed.
///
/// `links` may contain duplicate pairs and self-loops, `seeds` may be empty or
/// contain duplicates, and the graph may be disconnected.
///
/// Returns a vector of length `n`; element `i` is the round node `i` receives
/// the config, `0` for a seed, or `-1` if it never receives it.
///
/// Required: O(n + m) time and space.
pub fn propagation_rounds(
    n: usize,
    links: &[(usize, usize)],
    seeds: &[usize],
    quarantined: &[bool],
) -> Vec<i64> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check runs an
// independent per-edge relaxation to fixpoint, making no assumption about visit
// order.
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

    /// Relax every edge until nothing changes. O(n * m), order-agnostic.
    fn brute(
        n: usize,
        links: &[(usize, usize)],
        seeds: &[usize],
        quarantined: &[bool],
    ) -> Vec<i64> {
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for &(u, v) in links {
            adj[u].push(v);
            adj[v].push(u);
        }
        const INF: i64 = i64::MAX / 4;
        let mut dist = vec![INF; n];
        for &s in seeds {
            dist[s] = 0;
        }
        for _ in 0..=n {
            let mut changed = false;
            for u in 0..n {
                if dist[u] == INF || quarantined[u] {
                    continue; // unreached, or reached but forwards nothing
                }
                for i in 0..adj[u].len() {
                    let v = adj[u][i];
                    if dist[u] + 1 < dist[v] {
                        dist[v] = dist[u] + 1;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        dist.iter().map(|&d| if d == INF { -1 } else { d }).collect()
    }

    #[test]
    fn examples() {
        assert_eq!(
            propagation_rounds(4, &[(0, 1), (1, 2), (2, 3)], &[0], &[false; 4]),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            propagation_rounds(5, &[(0, 1), (1, 2), (2, 3), (3, 4)], &[0, 4], &[false; 5]),
            vec![0, 1, 2, 1, 0]
        );
        assert_eq!(
            propagation_rounds(
                4,
                &[(0, 1), (1, 2), (2, 3)],
                &[0],
                &[false, false, true, false]
            ),
            vec![0, 1, 2, -1]
        );
        assert_eq!(
            propagation_rounds(3, &[(0, 1), (1, 2)], &[0], &[true, false, false]),
            vec![0, -1, -1]
        );
    }

    #[test]
    fn quarantine_semantics() {
        // A quarantined node is recorded but never expands. Each assert below
        // changes if quarantined nodes are skipped entirely.
        assert_eq!(
            propagation_rounds(2, &[(0, 1)], &[0], &[false, true]),
            vec![0, 1]
        );
        assert_eq!(
            propagation_rounds(3, &[(0, 1), (1, 2)], &[0], &[false, true, false]),
            vec![0, 1, -1]
        );
        // A detour around the quarantined node still works.
        assert_eq!(
            propagation_rounds(
                4,
                &[(0, 1), (1, 3), (0, 2), (2, 3)],
                &[0],
                &[false, true, false, false]
            ),
            vec![0, 1, 1, 2]
        );
        // Every node quarantined: only seeds ever hold the config.
        assert_eq!(
            propagation_rounds(3, &[(0, 1), (1, 2)], &[1], &[true; 3]),
            vec![-1, 0, -1]
        );
        // A quarantined seed among healthy seeds.
        assert_eq!(
            propagation_rounds(
                4,
                &[(0, 1), (2, 3)],
                &[0, 2],
                &[true, false, false, false]
            ),
            vec![0, -1, 0, 1]
        );
    }

    #[test]
    fn multi_source() {
        // The answer is the minimum over all seeds.
        assert_eq!(
            propagation_rounds(5, &[(0, 1), (1, 2), (2, 3), (3, 4)], &[0, 4], &[false; 5]),
            vec![0, 1, 2, 1, 0]
        );
        // Every node a seed.
        assert_eq!(
            propagation_rounds(4, &[(0, 1), (1, 2), (2, 3)], &[0, 1, 2, 3], &[false; 4]),
            vec![0, 0, 0, 0]
        );
        // Duplicate seeds change nothing.
        assert_eq!(
            propagation_rounds(3, &[(0, 1), (1, 2)], &[0, 0, 0], &[false; 3]),
            vec![0, 1, 2]
        );
        // Seeds at both ends of a long chain meet in the middle.
        let n = 9usize;
        let links: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        assert_eq!(
            propagation_rounds(n, &links, &[0, 8], &vec![false; n]),
            vec![0, 1, 2, 3, 4, 3, 2, 1, 0]
        );
    }

    #[test]
    fn edges() {
        // No seeds at all.
        assert_eq!(
            propagation_rounds(3, &[(0, 1), (1, 2)], &[], &[false; 3]),
            vec![-1, -1, -1]
        );
        // No links: only seeds are reached.
        assert_eq!(propagation_rounds(3, &[], &[1], &[false; 3]), vec![-1, 0, -1]);
        // Single node.
        assert_eq!(propagation_rounds(1, &[], &[0], &[false]), vec![0]);
        assert_eq!(propagation_rounds(1, &[], &[], &[false]), vec![-1]);
        assert_eq!(propagation_rounds(1, &[(0, 0)], &[0], &[true]), vec![0]);
        // Self-loops and duplicate links change nothing.
        assert_eq!(
            propagation_rounds(3, &[(0, 0), (0, 1), (0, 1), (1, 2)], &[0], &[false; 3]),
            vec![0, 1, 2]
        );
        // Disconnected component is unreachable.
        assert_eq!(
            propagation_rounds(4, &[(0, 1), (2, 3)], &[0], &[false; 4]),
            vec![0, 1, -1, -1]
        );
        // A star: the hub reaches every leaf in one round.
        assert_eq!(
            propagation_rounds(5, &[(0, 1), (0, 2), (0, 3), (0, 4)], &[0], &[false; 5]),
            vec![0, 1, 1, 1, 1]
        );
        // A leaf seed reaches the far leaves in two rounds.
        assert_eq!(
            propagation_rounds(5, &[(0, 1), (0, 2), (0, 3), (0, 4)], &[1], &[false; 5]),
            vec![1, 0, 2, 2, 2]
        );
        // A cycle: the config wraps around both ways.
        let ring: Vec<(usize, usize)> = (0..6).map(|i| (i, (i + 1) % 6)).collect();
        assert_eq!(
            propagation_rounds(6, &ring, &[0], &[false; 6]),
            vec![0, 1, 2, 3, 2, 1]
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0xC19BF174CF692694);
        for _ in 0..400 {
            let n = (1 + rng.below(9)) as usize;
            let m = rng.below(14);
            let links: Vec<(usize, usize)> = (0..m)
                .map(|_| (rng.below(n as u64) as usize, rng.below(n as u64) as usize))
                .collect();
            let seed_count = rng.below(4);
            let seeds: Vec<usize> = (0..seed_count)
                .map(|_| rng.below(n as u64) as usize)
                .collect();
            let quarantined: Vec<bool> = (0..n).map(|_| rng.below(4) == 0).collect();
            assert_eq!(
                propagation_rounds(n, &links, &seeds, &quarantined),
                brute(n, &links, &seeds, &quarantined),
                "n={} links={:?} seeds={:?} quarantined={:?}",
                n,
                links,
                seeds,
                quarantined
            );
        }
    }

    #[test]
    fn large_chain() {
        // 200k-deep chain from one end: rounds are just the index.
        let n = 200_000usize;
        let links: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let expected: Vec<i64> = (0..n as i64).collect();
        assert_eq!(propagation_rounds(n, &links, &[0], &vec![false; n]), expected);
    }

    #[test]
    fn large_chain_both_ends() {
        // Seeds at both ends: each node's round is the distance to the nearer.
        let n = 200_001usize; // odd, so there is a unique midpoint
        let links: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let got = propagation_rounds(n, &links, &[0, n - 1], &vec![false; n]);
        let expected: Vec<i64> = (0..n).map(|i| i.min(n - 1 - i) as i64).collect();
        assert_eq!(got, expected);
        assert_eq!(*got.iter().max().unwrap(), ((n - 1) / 2) as i64);
    }

    #[test]
    fn large_many_seeds() {
        // 100k seeds on a 200k chain. Per-seed searches would be ~6 * 10^10.
        let n = 200_000usize;
        let links: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let seeds: Vec<usize> = (0..n).step_by(2).collect();
        let got = propagation_rounds(n, &links, &seeds, &vec![false; n]);
        let expected: Vec<i64> = (0..n).map(|i| if i % 2 == 0 { 0 } else { 1 }).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn large_star() {
        // 200k-leaf star: everything is within two rounds of any leaf.
        let n = 200_000usize;
        let links: Vec<(usize, usize)> = (1..n).map(|i| (0, i)).collect();
        let mut expected = vec![1i64; n];
        expected[0] = 0;
        assert_eq!(propagation_rounds(n, &links, &[0], &vec![false; n]), expected);
        let got = propagation_rounds(n, &links, &[1], &vec![false; n]);
        assert_eq!(got[1], 0);
        assert_eq!(got[0], 1);
        assert!((2..n).all(|i| got[i] == 2));
    }

    #[test]
    fn large_quarantine_wall() {
        // A quarantined node bisects the chain.
        let n = 200_000usize;
        let links: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let mut quarantined = vec![false; n];
        let wall = 100_000usize;
        quarantined[wall] = true;
        let got = propagation_rounds(n, &links, &[0], &quarantined);
        let front: Vec<i64> = (0..=wall as i64).collect();
        assert_eq!(&got[..=wall], &front[..], "the wall itself is recorded");
        assert!(
            got[wall + 1..].iter().all(|&r| r == -1),
            "everything past the wall is cut"
        );
    }

    #[test]
    fn large_all_quarantined() {
        // Every node quarantined: only the seeds ever hold the config.
        let n = 200_000usize;
        let links: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let seeds = [0usize, 12_345, n - 1];
        let got = propagation_rounds(n, &links, &seeds, &vec![true; n]);
        let zeros: Vec<usize> = (0..n).filter(|&i| got[i] == 0).collect();
        assert_eq!(zeros, vec![0, 12_345, n - 1]);
        assert_eq!(got.iter().filter(|&&r| r == -1).count(), n - 3);
    }

    #[test]
    fn large_disconnected() {
        // 100k isolated pairs, seeding only one node of each.
        let pairs = 100_000usize;
        let n = 2 * pairs;
        let links: Vec<(usize, usize)> = (0..pairs).map(|i| (2 * i, 2 * i + 1)).collect();
        let seeds: Vec<usize> = (0..pairs).map(|i| 2 * i).collect();
        let got = propagation_rounds(n, &links, &seeds, &vec![false; n]);
        let expected: Vec<i64> = (0..n).map(|i| if i % 2 == 0 { 0 } else { 1 }).collect();
        assert_eq!(got, expected);
    }
}
