//! Mesh Link Failures
//! problems/union-find/01-mesh-link-failures
//!
//! Fill in `partitions_after_failures`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Partition count after each successive link failure.
///
/// Two nodes are in the same partition when some chain of intact links joins
/// them; a node with no intact links is a partition of one.
///
/// * `n` — node count, `1 <= n <= 200_000`. Nodes are `0 .. n-1`.
/// * `links` — bidirectional links, up to 400_000. May contain the same pair
///   more than once (redundant cables, separate indices) and self-loops
///   `(u, u)`, which connect nothing.
/// * `failures` — distinct **indices into `links`**, in the order they go
///   down. Never endpoint pairs, so duplicates are unambiguous.
///
/// Returns `failures.len()` counts; element `k` is the partition count after
/// the first `k + 1` failures have been applied. Empty in, empty out.
///
/// Required: O((n + m) * alpha(n)) time, O(n + m) space.
pub fn partitions_after_failures(
    n: usize,
    links: &[(usize, usize)],
    failures: &[usize],
) -> Vec<usize> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The large cases use
// topologies whose partition counts are known by construction.
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

    /// Rebuild the graph and flood-fill from scratch after every failure.
    /// O(q * (n + m)) -- the thing the real solution has to beat.
    fn brute(n: usize, links: &[(usize, usize)], failures: &[usize]) -> Vec<usize> {
        let mut dead = vec![false; links.len()];
        let mut out = Vec::with_capacity(failures.len());
        for &idx in failures {
            dead[idx] = true;
            let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
            for (i, &(u, v)) in links.iter().enumerate() {
                if !dead[i] {
                    adj[u].push(v);
                    adj[v].push(u);
                }
            }
            let mut seen = vec![false; n];
            let mut count = 0usize;
            for s in 0..n {
                if seen[s] {
                    continue;
                }
                count += 1;
                seen[s] = true;
                let mut stack = vec![s];
                while let Some(x) = stack.pop() {
                    for i in 0..adj[x].len() {
                        let y = adj[x][i];
                        if !seen[y] {
                            seen[y] = true;
                            stack.push(y);
                        }
                    }
                }
            }
            out.push(count);
        }
        out
    }

    #[test]
    fn examples() {
        assert_eq!(
            partitions_after_failures(4, &[(0, 1), (1, 2), (2, 3)], &[1]),
            vec![2]
        );
        assert_eq!(
            partitions_after_failures(4, &[(0, 1), (1, 2), (2, 3)], &[1, 0, 2]),
            vec![2, 3, 4]
        );
        assert_eq!(
            partitions_after_failures(2, &[(0, 1), (0, 1)], &[0, 1]),
            vec![1, 2],
            "redundant cables are separate links; index 0 dying leaves index 1 intact"
        );
        assert_eq!(
            partitions_after_failures(3, &[(0, 0), (1, 2)], &[0, 1]),
            vec![2, 3]
        );
    }

    #[test]
    fn edges() {
        // No failures at all.
        assert_eq!(
            partitions_after_failures(5, &[(0, 1), (2, 3)], &[]),
            Vec::<usize>::new()
        );
        assert_eq!(partitions_after_failures(1, &[], &[]), Vec::<usize>::new());
        // Single node with a self-loop: one partition, cutting changes nothing.
        assert_eq!(partitions_after_failures(1, &[(0, 0)], &[0]), vec![1]);
        // Cutting a link inside a cycle never fragments anything.
        let tri = [(0usize, 1usize), (1, 2), (2, 0)];
        assert_eq!(partitions_after_failures(3, &tri, &[0]), vec![1]);
        assert_eq!(partitions_after_failures(3, &tri, &[0, 1]), vec![1, 2]);
        assert_eq!(partitions_after_failures(3, &tri, &[0, 1, 2]), vec![1, 2, 3]);
        // Links that are never cut keep holding the mesh together.
        assert_eq!(
            partitions_after_failures(4, &[(0, 1), (1, 2), (2, 3)], &[0]),
            vec![2]
        );
        // Isolated nodes are counted from the start.
        assert_eq!(partitions_after_failures(6, &[(0, 1)], &[0]), vec![6]);
        // Failures given out of index order.
        assert_eq!(
            partitions_after_failures(5, &[(0, 1), (1, 2), (2, 3), (3, 4)], &[3, 0, 2, 1]),
            vec![2, 3, 4, 5]
        );
        // Triple-redundant cable: only the last cut matters.
        assert_eq!(
            partitions_after_failures(2, &[(0, 1), (0, 1), (0, 1)], &[1, 2, 0]),
            vec![1, 1, 2]
        );
        // Self-loops interleaved with real links.
        assert_eq!(
            partitions_after_failures(
                4,
                &[(0, 0), (0, 1), (1, 1), (1, 2), (3, 3)],
                &[0, 2, 3, 4, 1]
            ),
            vec![2, 2, 3, 3, 4]
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x5DEECE66D1234567);
        for _ in 0..300 {
            let n = (1 + rng.below(9)) as usize;
            let m = rng.below(14) as usize;
            let links: Vec<(usize, usize)> = (0..m)
                .map(|_| (rng.below(n as u64) as usize, rng.below(n as u64) as usize))
                .collect();
            // A random distinct subset of link indices, in random order.
            let mut idx: Vec<usize> = (0..m).collect();
            let q = rng.below(m as u64 + 1) as usize;
            for i in 0..q {
                let j = i + rng.below((m - i) as u64) as usize;
                idx.swap(i, j);
            }
            let failures: Vec<usize> = idx[..q].to_vec();
            assert_eq!(
                partitions_after_failures(n, &links, &failures),
                brute(n, &links, &failures),
                "n={} links={:?} failures={:?}",
                n,
                links,
                failures
            );
        }
    }

    #[test]
    fn large_chain_scrambled() {
        // 200k-node path graph, 100k links cut in scrambled order. Cutting k
        // distinct edges of a path always yields exactly k + 1 components,
        // whatever the order, so the expected output is pinned by construction.
        // The flood-fill checker would need ~2 * 10^10 operations here.
        let n = 200_000usize;
        let links: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let q = 100_000usize;
        let mut idx: Vec<usize> = (0..links.len()).collect();
        let mut rng = Lcg::new(0x27D4EB2F165667C5);
        for i in 0..q {
            let j = i + rng.below((links.len() - i) as u64) as usize;
            idx.swap(i, j);
        }
        let failures: Vec<usize> = idx[..q].to_vec();
        let expected: Vec<usize> = (2..q + 2).collect();
        assert_eq!(partitions_after_failures(n, &links, &failures), expected);
    }

    #[test]
    fn large_duplicate_cables() {
        // Every chain link doubled: the first cut of each pair must be inert.
        let n = 100_000usize;
        let mut links: Vec<(usize, usize)> = Vec::new();
        for i in 0..n - 1 {
            links.push((i, i + 1));
            links.push((i, i + 1)); // redundant cable, separate index
        }
        let first_of_each: Vec<usize> = (0..n - 1).map(|i| 2 * i).collect();
        assert_eq!(
            partitions_after_failures(n, &links, &first_of_each),
            vec![1usize; n - 1]
        );
    }

    #[test]
    fn large_cycle() {
        // A ring: the first cut is free, every later cut fragments.
        let n = 200_000usize;
        let links: Vec<(usize, usize)> = (0..n).map(|i| (i, (i + 1) % n)).collect();
        let q = 50_000usize;
        let failures: Vec<usize> = (0..q).collect();
        // Cutting k edges of a cycle gives max(1, k) components.
        let mut expected = vec![1usize];
        expected.extend(2..=q);
        assert_eq!(partitions_after_failures(n, &links, &failures), expected);
    }

    #[test]
    fn large_untouched_backbone() {
        // A backbone that is never cut keeps everything in one partition.
        let n = 200_000usize;
        let mut links: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let spurs = links.len();
        links.extend((1..=100_000usize).map(|i| (0, i))); // redundant spurs
        let failures: Vec<usize> = (spurs..spurs + 100_000).collect();
        assert_eq!(
            partitions_after_failures(n, &links, &failures),
            vec![1usize; 100_000]
        );
    }

    #[test]
    fn large_self_loops() {
        // 200k self-loops: nothing is ever connected, nothing ever changes.
        let n = 200_000usize;
        let links: Vec<(usize, usize)> = (0..n).map(|i| (i, i)).collect();
        let failures: Vec<usize> = (0..n).step_by(2).collect();
        let want = vec![n; failures.len()];
        assert_eq!(partitions_after_failures(n, &links, &failures), want);
    }

    #[test]
    fn large_star_shatter() {
        // A star: each cut peels exactly one leaf off the hub.
        let n = 200_000usize;
        let links: Vec<(usize, usize)> = (1..n).map(|i| (0, i)).collect();
        let q = 150_000usize;
        let failures: Vec<usize> = (0..q).collect();
        let expected: Vec<usize> = (2..q + 2).collect();
        assert_eq!(partitions_after_failures(n, &links, &failures), expected);
    }
}
