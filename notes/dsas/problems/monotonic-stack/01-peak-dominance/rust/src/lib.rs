//! Peak Dominance
//! problems/monotonic-stack/01-peak-dominance
//!
//! Fill in `dominance_spans`, then run `cargo test` in this directory.
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// For each sample, the width of the widest stretch it dominates.
///
/// A sample dominates a contiguous stretch if it lies inside that stretch and
/// no reading in the stretch is strictly higher than it. Ties count as
/// dominated -- only a strictly greater reading ends a sample's reign.
///
/// Equivalently, for each index `i`: find the nearest strictly greater value to
/// the left and to the right; the answer is the number of samples strictly
/// between them.
///
/// Every entry of the result is at least 1, since a sample always dominates at
/// least itself.
///
/// Required: O(n) time, O(n) space.
pub fn dominance_spans(load: &[i64]) -> Vec<usize> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case.
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

    /// Expand outward from every index. O(n^2); hopeless at the real bounds.
    fn brute(load: &[i64]) -> Vec<usize> {
        let n = load.len();
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let mut lo = i;
            while lo > 0 && load[lo - 1] <= load[i] {
                lo -= 1;
            }
            let mut hi = i;
            while hi + 1 < n && load[hi + 1] <= load[i] {
                hi += 1;
            }
            out.push(hi - lo + 1);
        }
        out
    }

    #[test]
    fn examples() {
        assert_eq!(dominance_spans(&[2, 1, 3]), vec![2, 1, 3]);
        assert_eq!(dominance_spans(&[5, 5, 5]), vec![3, 3, 3]);
        assert_eq!(dominance_spans(&[1, 2, 5, 2, 1]), vec![1, 2, 5, 2, 1]);
        assert_eq!(dominance_spans(&[4, 1, 4]), vec![3, 1, 3]);
    }

    #[test]
    fn edges() {
        // Empty and singleton.
        assert_eq!(dominance_spans(&[]), Vec::<usize>::new());
        assert_eq!(dominance_spans(&[42]), vec![1]);
        assert_eq!(dominance_spans(&[0]), vec![1]);
        // Strictly increasing: each element is blocked only by the one after it.
        assert_eq!(dominance_spans(&[1, 2, 3]), vec![1, 2, 3]);
        // Strictly decreasing: mirror image.
        assert_eq!(dominance_spans(&[3, 2, 1]), vec![3, 2, 1]);
        // Two equal values dominate each other.
        assert_eq!(dominance_spans(&[7, 7]), vec![2, 2]);
        // A plateau flanked by higher ground.
        assert_eq!(dominance_spans(&[9, 4, 4, 9]), vec![4, 2, 2, 4]);
        // A plateau flanked by lower ground.
        assert_eq!(dominance_spans(&[1, 4, 4, 1]), vec![1, 4, 4, 1]);
        // Zeros are ordinary values.
        assert_eq!(dominance_spans(&[0, 0, 0]), vec![3, 3, 3]);
        // Value range boundaries.
        assert_eq!(dominance_spans(&[0, 1_000_000_000]), vec![1, 2]);
        assert_eq!(dominance_spans(&[1_000_000_000, 0]), vec![2, 1]);
        // Alternating. Both 5s are joint maxima, and since ties do not break a
        // reign, neither blocks the other -- each dominates the WHOLE series.
        assert_eq!(dominance_spans(&[1, 5, 1, 5, 1]), vec![1, 5, 1, 5, 1]);
    }

    #[test]
    fn ties_matter() {
        // Equal values must NOT end a reign. Every assert here changes if the
        // stack pops on `<` instead of `<=`.
        assert_eq!(dominance_spans(&[5, 5, 5]), vec![3, 3, 3]);
        assert_eq!(dominance_spans(&[2, 2, 1, 2, 2]), vec![5, 5, 1, 5, 5]);
        assert_eq!(dominance_spans(&[3, 1, 3, 1, 3]), vec![5, 1, 5, 1, 5]);
        assert_eq!(dominance_spans(&[6, 6, 7]), vec![2, 2, 3]);
        assert_eq!(dominance_spans(&[7, 6, 6]), vec![3, 2, 2]);
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0xC0AC29B7C97C50DD);
        for _ in 0..400 {
            let n = rng.below(26);
            // Tiny alphabet on purpose: forces constant ties, which is where
            // the strict/non-strict comparison bugs live.
            let spread = 1 + rng.below(4);
            let load: Vec<i64> = (0..n).map(|_| rng.below(spread) as i64).collect();
            assert_eq!(dominance_spans(&load), brute(&load), "load={:?}", load);
        }
    }

    #[test]
    fn random_wide_alphabet() {
        // Same cross-check with mostly-distinct values.
        let mut rng = Lcg::new(0x9216D5D98979FB1B);
        for _ in 0..300 {
            let n = rng.below(30);
            let load: Vec<i64> = (0..n).map(|_| rng.below(1_000_000_000) as i64).collect();
            assert_eq!(dominance_spans(&load), brute(&load), "load={:?}", load);
        }
    }

    #[test]
    fn large_flat() {
        // 200k identical readings: every sample dominates the whole series.
        let n = 200_000usize;
        assert_eq!(dominance_spans(&vec![17i64; n]), vec![n; n]);
    }

    #[test]
    fn large_increasing() {
        // Strictly increasing: span[i] = i + 1.
        let n = 200_000usize;
        let load: Vec<i64> = (0..n as i64).collect();
        let expected: Vec<usize> = (1..=n).collect();
        assert_eq!(dominance_spans(&load), expected);
    }

    #[test]
    fn large_decreasing() {
        // Strictly decreasing: span[i] = n - i.
        let n = 200_000usize;
        let load: Vec<i64> = (1..=n as i64).rev().collect();
        let expected: Vec<usize> = (1..=n).rev().collect();
        assert_eq!(dominance_spans(&load), expected);
    }

    #[test]
    fn large_mountain() {
        // Up then down; the apex dominates everything, the flanks mirror.
        let half = 100_000usize;
        let mut load: Vec<i64> = (0..half as i64).collect();
        load.extend((1..=half as i64).rev());
        let n = load.len();
        let spans = dominance_spans(&load);
        assert_eq!(spans.len(), n);
        assert_eq!(spans[half], n, "the apex dominates the whole series");
        assert_eq!(spans[0], 1);
        assert_eq!(spans[half - 1], half);
        assert!(spans.iter().all(|&s| s >= 1));
        assert_eq!(*spans.iter().max().unwrap(), n);
    }

    #[test]
    fn large_plateaus() {
        // Blocks of equal values with strictly increasing block heights. Inside
        // block b, every element reaches back to index 0 and forward to the end
        // of its own block.
        let block = 500usize;
        let blocks = 400usize;
        let n = block * blocks;
        let load: Vec<i64> = (0..n).map(|i| (i / block) as i64).collect();
        let spans = dominance_spans(&load);
        for b in 0..blocks {
            let expected = (b + 1) * block;
            for k in [0usize, block / 2, block - 1] {
                assert_eq!(spans[b * block + k], expected, "block={} k={}", b, k);
            }
        }
    }

    #[test]
    fn large_sawtooth() {
        // Every 1 is a joint maximum, and ties do not break a reign, so each of
        // the 100_000 peaks dominates the entire series. Each 0 is boxed in by
        // strictly greater neighbours. Popping on `<` reports 3 here, not n.
        let n = 200_000usize;
        let load: Vec<i64> = (0..n).map(|i| if i % 2 == 0 { 0 } else { 1 }).collect();
        let spans = dominance_spans(&load);
        assert!(
            (0..n).step_by(2).all(|i| spans[i] == 1),
            "valleys dominate only themselves"
        );
        assert!(
            (1..n).step_by(2).all(|i| spans[i] == n),
            "joint maxima dominate everything"
        );
    }
}
