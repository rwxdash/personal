//! Audit Sampling
//! problems/dp-on-trees/01-audit-sampling
//!
//! Fill in `max_audit_evidence`, then run `cargo test` in this directory.
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Maximum audit evidence with no audited service directly under another.
///
/// Choose a set of services such that no chosen service is the direct parent
/// of another chosen service. Siblings are fine; grandparent and grandchild
/// are fine. Auditing nothing is allowed, so the answer is never negative.
///
/// `parent[0]` is `-1`; the input is guaranteed to be a valid tree rooted at 0.
/// The tree may be a single chain 200_000 deep, and a parent's index is NOT
/// guaranteed to be smaller than its children's.
///
/// The total can reach 2 * 10^14, hence `i64`.
///
/// Required: O(n) time, O(n) space.
pub fn max_audit_evidence(parent: &[i64], evidence: &[i64]) -> i64 {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check
// enumerates every subset and checks the parent-child rule directly.
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

    /// Try every subset. O(2^n * n); only viable for tiny trees.
    fn brute(parent: &[i64], evidence: &[i64]) -> i64 {
        let n = parent.len();
        let mut best = 0i64;
        for mask in 0u64..(1u64 << n) {
            let mut ok = true;
            for i in 0..n {
                if mask & (1 << i) != 0 && parent[i] != -1 && mask & (1 << parent[i]) != 0 {
                    ok = false;
                    break;
                }
            }
            if ok {
                let total: i64 = (0..n)
                    .filter(|&i| mask & (1 << i) != 0)
                    .map(|i| evidence[i])
                    .sum();
                best = best.max(total);
            }
        }
        best
    }

    #[test]
    fn examples() {
        assert_eq!(max_audit_evidence(&[-1, 0, 0], &[10, 6, 6]), 12);
        assert_eq!(max_audit_evidence(&[-1, 0, 0], &[10, 3, 3]), 10);
        assert_eq!(max_audit_evidence(&[-1, 0, 1, 2], &[4, 1, 1, 4]), 8);
        assert_eq!(max_audit_evidence(&[-1, 0, 1], &[5, 100, 5]), 100);
    }

    #[test]
    fn simple_rules_fail() {
        // Greedy by value takes the root's 10 and loses 12.
        assert_eq!(max_audit_evidence(&[-1, 0, 0], &[10, 6, 6]), 12);
        // Even-depth-only gives 10; the answer is the middle node's 100.
        assert_eq!(max_audit_evidence(&[-1, 0, 1], &[5, 100, 5]), 100);
        // Here even-depth-only happens to be right -- neither rule is
        // consistently correct.
        assert_eq!(max_audit_evidence(&[-1, 0, 1, 2], &[4, 1, 1, 4]), 8);
    }

    #[test]
    fn edges() {
        // Single service.
        assert_eq!(max_audit_evidence(&[-1], &[42]), 42);
        assert_eq!(max_audit_evidence(&[-1], &[0]), 0);
        // All zero evidence.
        assert_eq!(max_audit_evidence(&[-1, 0, 0, 1], &[0, 0, 0, 0]), 0);
        // Two services: take the larger one.
        assert_eq!(max_audit_evidence(&[-1, 0], &[3, 9]), 9);
        assert_eq!(max_audit_evidence(&[-1, 0], &[9, 3]), 9);
        // A star: all leaves beat the hub, or not.
        assert_eq!(max_audit_evidence(&[-1, 0, 0, 0], &[5, 2, 2, 2]), 6);
        assert_eq!(max_audit_evidence(&[-1, 0, 0, 0], &[10, 2, 2, 2]), 10);
        // Deep chain of equal values: every other one.
        assert_eq!(max_audit_evidence(&[-1, 0, 1, 2, 3], &[1, 1, 1, 1, 1]), 3);
        // The root is best left unaudited even though it is worth something.
        assert_eq!(max_audit_evidence(&[-1, 0, 1], &[1, 50, 1]), 50);
        // Grandchildren stack up.
        assert_eq!(
            max_audit_evidence(&[-1, 0, 1, 2, 3, 4], &[5, 1, 5, 1, 5, 1]),
            15
        );
        // A parent whose index is LARGER than its child's. The tree is
        // 0 -> 2 -> 1, so node 1's parent is node 2. The root is still node 0,
        // as the contract requires -- only the internal numbering is out of
        // order.
        assert_eq!(max_audit_evidence(&[-1, 2, 0], &[7, 7, 20]), 20);
        assert_eq!(max_audit_evidence(&[-1, 2, 0], &[7, 7, 5]), 14);
        // Value ceiling.
        assert_eq!(
            max_audit_evidence(&[-1, 0], &[1_000_000_000, 1_000_000_000]),
            1_000_000_000
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0xD192E819D6EF5218);
        for _ in 0..400 {
            let n = (1 + rng.below(12)) as usize;
            let mut parent = vec![-1i64];
            for i in 1..n {
                parent.push(rng.below(i as u64) as i64);
            }
            let evidence: Vec<i64> = (0..n).map(|_| rng.below(20) as i64).collect();
            assert_eq!(
                max_audit_evidence(&parent, &evidence),
                brute(&parent, &evidence),
                "parent={:?} evidence={:?}",
                parent,
                evidence
            );
        }
    }

    #[test]
    fn random_shuffled_labels() {
        // Labels permuted so parents may have larger indices than children.
        // Node 0 stays the root, as the contract requires.
        let mut rng = Lcg::new(0xD69906245565A910);
        for _ in 0..300 {
            let n = (1 + rng.below(10)) as usize;
            let mut base = vec![-1i64];
            for i in 1..n {
                base.push(rng.below(i as u64) as i64);
            }
            let mut perm: Vec<usize> = (0..n).collect();
            for i in (2..n).rev() {
                let j = 1 + rng.below(i as u64) as usize;
                perm.swap(i, j);
            }
            let evidence_base: Vec<i64> = (0..n).map(|_| rng.below(25) as i64).collect();
            let mut parent = vec![-1i64; n];
            let mut evidence = vec![0i64; n];
            for old in 0..n {
                parent[perm[old]] = if base[old] == -1 {
                    -1
                } else {
                    perm[base[old] as usize] as i64
                };
                evidence[perm[old]] = evidence_base[old];
            }
            assert_eq!(
                max_audit_evidence(&parent, &evidence),
                brute(&parent, &evidence),
                "parent={:?} evidence={:?}",
                parent,
                evidence
            );
        }
    }

    #[test]
    fn large_chain() {
        // 200k-deep chain of equal values: every other service.
        let n = 200_000usize;
        let mut parent = vec![-1i64; n];
        for i in 1..n {
            parent[i] = (i - 1) as i64;
        }
        let evidence = vec![1i64; n];
        assert_eq!(max_audit_evidence(&parent, &evidence), ((n + 1) / 2) as i64);
    }

    #[test]
    fn large_chain_alternating() {
        // A chain where the odd-depth services are far more valuable.
        let n = 200_000usize;
        let mut parent = vec![-1i64; n];
        for i in 1..n {
            parent[i] = (i - 1) as i64;
        }
        let evidence: Vec<i64> = (0..n).map(|i| if i % 2 == 0 { 1 } else { 1000 }).collect();
        assert_eq!(max_audit_evidence(&parent, &evidence), (n as i64 / 2) * 1000);
    }

    #[test]
    fn large_star() {
        // 200k-leaf star.
        let n = 200_000usize;
        let mut parent = vec![0i64; n];
        parent[0] = -1;
        let mut evidence = vec![1i64; n];
        evidence[0] = 1_000_000_000;
        assert_eq!(max_audit_evidence(&parent, &evidence), 1_000_000_000);
        // Now make the leaves worth more in aggregate.
        let mut evidence = vec![10_000i64; n];
        evidence[0] = 1_000_000_000;
        assert_eq!(
            max_audit_evidence(&parent, &evidence),
            (n as i64 - 1) * 10_000
        );
    }

    #[test]
    fn large_all_max() {
        // Every service at the value ceiling: forces 64-bit arithmetic.
        let n = 200_000usize;
        let mut parent = vec![-1i64; n];
        for i in 1..n {
            parent[i] = (i - 1) as i64;
        }
        let evidence = vec![1_000_000_000i64; n];
        let expected = ((n as i64 + 1) / 2) * 1_000_000_000;
        assert_eq!(max_audit_evidence(&parent, &evidence), expected);
        assert!(expected > u32::MAX as i64);
    }

    #[test]
    fn large_binary_tree() {
        // A perfect binary tree with equal values: every node at even depth.
        let n = (1usize << 17) - 1; // 131_071
        let mut parent = vec![-1i64; n];
        for i in 1..n {
            parent[i] = ((i - 1) / 2) as i64;
        }
        let evidence = vec![1i64; n];
        let mut depth = vec![0usize; n];
        for i in 1..n {
            depth[i] = depth[(i - 1) / 2] + 1;
        }
        let expected = (0..n).filter(|&i| depth[i] % 2 == 0).count() as i64;
        assert_eq!(max_audit_evidence(&parent, &evidence), expected);
    }

    #[test]
    fn large_wide_shallow() {
        // A root with 200k children, all worth more than the root.
        let n = 200_000usize;
        let mut parent = vec![0i64; n];
        parent[0] = -1;
        let mut evidence = vec![7i64; n];
        evidence[0] = 5;
        assert_eq!(max_audit_evidence(&parent, &evidence), (n as i64 - 1) * 7);
    }

    #[test]
    fn large_zero_root() {
        // A worthless root: taking it can only cost you.
        let n = 200_000usize;
        let mut parent = vec![-1i64; n];
        for i in 1..n {
            parent[i] = (i - 1) as i64;
        }
        let mut evidence = vec![1i64; n];
        evidence[0] = 0;
        assert_eq!(max_audit_evidence(&parent, &evidence), (n as i64) / 2);
    }
}
