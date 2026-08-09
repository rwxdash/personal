//! Resource Ancestry
//! problems/dfs/01-resource-ancestry
//!
//! Fill in `ancestor_queries`, then run `cargo test` in this directory.
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Answer "is `u` an ancestor of `v`?" for every query.
///
/// `u` is an ancestor of `v` when `u == v`, or when `u` lies on the path from
/// `v` upward to its root. Resources in different trees are never ancestors of
/// one another.
///
/// `parent[i]` is the parent of resource `i`, or `-1` if `i` is a root.
/// Guaranteed a valid forest: following parent pointers always terminates at a
/// root, with no cycles. The hierarchy may be a single chain up to 200_000
/// deep.
///
/// Required: O(n + q) time, O(n) space — so O(1) per query after linear
/// preprocessing.
pub fn ancestor_queries(parent: &[i64], queries: &[(usize, usize)]) -> Vec<bool> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check walks
// the parent chain per query — the naive approach the complexity bound forbids.
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

    /// Walk upward from v looking for u. O(q * depth).
    fn brute(parent: &[i64], queries: &[(usize, usize)]) -> Vec<bool> {
        queries
            .iter()
            .map(|&(u, v)| {
                let mut cur = v as i64;
                while cur != -1 {
                    if cur as usize == u {
                        return true;
                    }
                    cur = parent[cur as usize];
                }
                false
            })
            .collect()
    }

    #[test]
    fn examples() {
        assert_eq!(
            ancestor_queries(&[-1, 0, 0, 1], &[(0, 3), (1, 3), (2, 3), (3, 0), (1, 1)]),
            vec![true, true, false, false, true]
        );
        assert_eq!(
            ancestor_queries(&[-1, 0, -1, 2], &[(0, 1), (0, 3), (2, 3), (2, 1)]),
            vec![true, false, true, false]
        );
        assert_eq!(
            ancestor_queries(&[-1, 0, 1, 2], &[(0, 3), (1, 2), (2, 1), (3, 3)]),
            vec![true, true, false, true]
        );
    }

    #[test]
    fn self_and_direction() {
        let parent = [-1i64, 0, 1];
        // Every node is its own ancestor.
        let selfq: Vec<(usize, usize)> = (0..3).map(|i| (i, i)).collect();
        assert_eq!(ancestor_queries(&parent, &selfq), vec![true; 3]);
        // Downward is True, upward is False, for every ordered pair.
        assert_eq!(
            ancestor_queries(&parent, &[(0, 1), (1, 0)]),
            vec![true, false]
        );
        assert_eq!(
            ancestor_queries(&parent, &[(0, 2), (2, 0)]),
            vec![true, false]
        );
        assert_eq!(
            ancestor_queries(&parent, &[(1, 2), (2, 1)]),
            vec![true, false]
        );
    }

    #[test]
    fn cross_tree() {
        // Nodes in different trees are never ancestors. These fail if the
        // traversal timer is reset per root instead of running globally.
        let parent = [-1i64, -1, 0, 1]; // tree A: 0 -> 2, tree B: 1 -> 3
        assert_eq!(
            ancestor_queries(&parent, &[(0, 2), (1, 3)]),
            vec![true, true]
        );
        assert_eq!(
            ancestor_queries(&parent, &[(0, 3), (1, 2)]),
            vec![false, false]
        );
        assert_eq!(
            ancestor_queries(&parent, &[(0, 1), (1, 0)]),
            vec![false, false]
        );
        assert_eq!(
            ancestor_queries(&parent, &[(2, 3), (3, 2)]),
            vec![false, false]
        );
        // Three separate single-node trees.
        let singles = [-1i64, -1, -1];
        assert_eq!(
            ancestor_queries(&singles, &[(0, 1), (1, 2), (2, 0)]),
            vec![false; 3]
        );
        assert_eq!(
            ancestor_queries(&singles, &[(0, 0), (1, 1), (2, 2)]),
            vec![true; 3]
        );
    }

    #[test]
    fn edges() {
        // Single node.
        assert_eq!(ancestor_queries(&[-1], &[(0, 0)]), vec![true]);
        // No queries.
        assert_eq!(ancestor_queries(&[-1, 0], &[]), Vec::<bool>::new());
        // A wide star.
        let star = [-1i64, 0, 0, 0, 0, 0];
        let down: Vec<(usize, usize)> = (1..6).map(|i| (0, i)).collect();
        let up: Vec<(usize, usize)> = (1..6).map(|i| (i, 0)).collect();
        assert_eq!(ancestor_queries(&star, &down), vec![true; 5]);
        assert_eq!(ancestor_queries(&star, &up), vec![false; 5]);
        // Siblings are never ancestors of each other.
        assert_eq!(
            ancestor_queries(&star, &[(1, 2), (2, 1), (3, 5)]),
            vec![false; 3]
        );
        // Root is not index 0.
        let off = [1i64, -1, 1]; // 1 is the root, with children 0 and 2
        assert_eq!(
            ancestor_queries(&off, &[(1, 0), (1, 2), (0, 2), (0, 1)]),
            vec![true, true, false, false]
        );
        // Every node a root: a forest of singletons.
        let n = 6usize;
        let all_roots = vec![-1i64; n];
        let pairs: Vec<(usize, usize)> =
            (0..n).flat_map(|i| (0..n).map(move |j| (i, j))).collect();
        let want: Vec<bool> = pairs.iter().map(|&(i, j)| i == j).collect();
        assert_eq!(ancestor_queries(&all_roots, &pairs), want);
        // Deep-then-wide: a chain that fans out at the bottom.
        let fan = [-1i64, 0, 1, 2, 2, 2];
        assert_eq!(
            ancestor_queries(&fan, &[(0, 5), (1, 4), (2, 3), (3, 4)]),
            vec![true, true, true, false]
        );
        // Repeated identical queries.
        assert_eq!(ancestor_queries(&[-1, 0], &[(0, 1); 5]), vec![true; 5]);
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x71374491BE5466CF);
        for _ in 0..400 {
            let n = (1 + rng.below(12)) as usize;
            // Node i's parent is a strictly smaller index, or -1: never cyclic.
            let mut parent: Vec<i64> = Vec::with_capacity(n);
            for i in 0..n {
                if i == 0 || rng.below(3) == 0 {
                    parent.push(-1);
                } else {
                    parent.push(rng.below(i as u64) as i64);
                }
            }
            let qn = rng.below(15);
            let queries: Vec<(usize, usize)> = (0..qn)
                .map(|_| (rng.below(n as u64) as usize, rng.below(n as u64) as usize))
                .collect();
            assert_eq!(
                ancestor_queries(&parent, &queries),
                brute(&parent, &queries),
                "parent={:?} queries={:?}",
                parent,
                queries
            );
        }
    }

    #[test]
    fn random_shuffled_labels() {
        // Same cross-check with labels permuted, so parents are not always
        // smaller indices and traversal order cannot be assumed.
        let mut rng = Lcg::new(0x2748774CDF8EEB99);
        for _ in 0..300 {
            let n = (1 + rng.below(10)) as usize;
            let mut base: Vec<i64> = Vec::with_capacity(n);
            for i in 0..n {
                if i == 0 || rng.below(3) == 0 {
                    base.push(-1);
                } else {
                    base.push(rng.below(i as u64) as i64);
                }
            }
            let mut perm: Vec<usize> = (0..n).collect();
            for i in (1..n).rev() {
                let j = rng.below(i as u64 + 1) as usize;
                perm.swap(i, j);
            }
            let mut parent = vec![-1i64; n];
            for old in 0..n {
                parent[perm[old]] = if base[old] == -1 {
                    -1
                } else {
                    perm[base[old] as usize] as i64
                };
            }
            let qn = rng.below(12);
            let queries: Vec<(usize, usize)> = (0..qn)
                .map(|_| (rng.below(n as u64) as usize, rng.below(n as u64) as usize))
                .collect();
            assert_eq!(
                ancestor_queries(&parent, &queries),
                brute(&parent, &queries),
                "parent={:?} queries={:?}",
                parent,
                queries
            );
        }
    }

    #[test]
    fn large_deep_chain() {
        // 200k-deep chain. A recursive traversal overflows the stack here.
        let n = 200_000usize;
        let mut parent = vec![-1i64; n];
        for i in 1..n {
            parent[i] = (i - 1) as i64;
        }
        let queries = [
            (0, n - 1),
            (n - 1, 0),
            (n / 2, n - 1),
            (n - 1, n / 2),
            (7, 7),
        ];
        assert_eq!(
            ancestor_queries(&parent, &queries),
            vec![true, false, true, false, true]
        );
    }

    #[test]
    fn large_chain_many_queries() {
        // Deep chain with 200k queries: O(q * depth) would be 4 * 10^10 steps.
        let n = 200_000usize;
        let mut parent = vec![-1i64; n];
        for i in 1..n {
            parent[i] = (i - 1) as i64;
        }
        let mut rng = Lcg::new(0x9BDC06A725C71235);
        let queries: Vec<(usize, usize)> = (0..200_000)
            .map(|_| (rng.below(n as u64) as usize, rng.below(n as u64) as usize))
            .collect();
        let want: Vec<bool> = queries.iter().map(|&(u, v)| u <= v).collect();
        assert_eq!(ancestor_queries(&parent, &queries), want);
    }

    #[test]
    fn large_star() {
        // 200k-leaf star.
        let n = 200_000usize;
        let mut parent = vec![0i64; n];
        parent[0] = -1;
        let mut queries: Vec<(usize, usize)> = (0..n).map(|i| (0, i)).collect();
        queries.extend((1..n).map(|i| (i, 0)));
        let mut expected = vec![true; n];
        expected.extend(vec![false; n - 1]);
        assert_eq!(ancestor_queries(&parent, &queries), expected);
    }

    #[test]
    fn large_forest() {
        // 1000 separate chains of 200 nodes; cross-tree queries must be False.
        // Fails if the traversal timer restarts at each root.
        let trees = 1_000usize;
        let depth = 200usize;
        let n = trees * depth;
        let mut parent = vec![-1i64; n];
        for t in 0..trees {
            let base = t * depth;
            for d in 1..depth {
                parent[base + d] = (base + d - 1) as i64;
            }
        }
        let mut queries: Vec<(usize, usize)> = Vec::new();
        let mut expected: Vec<bool> = Vec::new();
        for t in 0..trees {
            let base = t * depth;
            queries.push((base, base + depth - 1));
            expected.push(true);
            let other = ((t + 1) % trees) * depth;
            queries.push((base, other + depth - 1));
            expected.push(false);
        }
        assert_eq!(ancestor_queries(&parent, &queries), expected);
    }

    #[test]
    fn large_binary_tree() {
        // A complete binary tree: ancestry follows the heap index rule.
        let n = 200_000usize;
        let mut parent = vec![-1i64; n];
        for i in 1..n {
            parent[i] = ((i - 1) / 2) as i64;
        }
        fn is_ancestor(u: usize, mut v: usize) -> bool {
            while v > u {
                v = (v - 1) / 2;
            }
            u == v
        }
        let mut rng = Lcg::new(0xC19BF1749EF14AD2);
        let queries: Vec<(usize, usize)> = (0..50_000)
            .map(|_| (rng.below(n as u64) as usize, rng.below(n as u64) as usize))
            .collect();
        let want: Vec<bool> = queries.iter().map(|&(u, v)| is_ancestor(u, v)).collect();
        assert_eq!(ancestor_queries(&parent, &queries), want);
    }
}
