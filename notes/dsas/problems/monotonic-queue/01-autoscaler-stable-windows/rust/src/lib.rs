//! Autoscaler Stable Windows
//! problems/monotonic-queue/01-autoscaler-stable-windows
//!
//! Fill in `count_stable_windows`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Count the contiguous ranges of `usage` that are stable and long enough.
///
/// A range `usage[i..=j]` qualifies when both hold:
///   * `max(usage[i..=j]) - min(usage[i..=j]) <= delta`
///   * `j - i + 1 >= min_len`
///
/// Ranges are counted by position: `usage[3..=7]` and `usage[4..=8]` are
/// distinct even if they hold the same values.
///
/// `min_len` may exceed `usage.len()`, in which case the answer is 0. The
/// result can reach ~2 * 10^10, hence `u64`.
///
/// Required: O(n) time expected, O(n) extra space.
pub fn count_stable_windows(usage: &[i64], delta: i64, min_len: usize) -> u64 {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The large cases use series
// whose exact answer is known in closed form.
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

    /// Obviously-correct O(n^2) checker. Hopeless at the real bounds.
    fn brute(usage: &[i64], delta: i64, min_len: usize) -> u64 {
        let mut total = 0u64;
        for i in 0..usage.len() {
            let (mut lo, mut hi) = (usage[i], usage[i]);
            for j in i..usage.len() {
                lo = lo.min(usage[j]);
                hi = hi.max(usage[j]);
                if hi - lo <= delta && (j - i + 1) >= min_len {
                    total += 1;
                }
            }
        }
        total
    }

    /// Exact count for `usage[i] = i`: a range is stable iff its length is at
    /// most `delta + 1`, and there are `n - L + 1` ranges of length `L`.
    fn ramp_expected(n: u64, delta: u64, min_len: u64) -> u64 {
        let mut total = 0u64;
        let hi = (delta + 1).min(n);
        let mut length = min_len.max(1);
        while length <= hi {
            total += n - length + 1;
            length += 1;
        }
        total
    }

    #[test]
    fn examples() {
        assert_eq!(count_stable_windows(&[5, 7, 6, 9], 2, 1), 7);
        assert_eq!(count_stable_windows(&[5, 7, 6, 9], 2, 2), 3);
        assert_eq!(count_stable_windows(&[4, 4, 4, 1], 0, 1), 7);
        assert_eq!(count_stable_windows(&[1, 2, 3], 10, 5), 0);
    }

    #[test]
    fn edges() {
        // Empty series.
        assert_eq!(count_stable_windows(&[], 0, 1), 0);
        assert_eq!(count_stable_windows(&[], 1_000_000_000, 1), 0);
        // Single sample: stable by definition, but only if min_len allows it.
        assert_eq!(count_stable_windows(&[42], 0, 1), 1);
        assert_eq!(count_stable_windows(&[42], 1_000_000_000, 2), 0);
        // min_len exactly equal to n, and one past it.
        assert_eq!(count_stable_windows(&[3, 3, 3], 0, 3), 1);
        assert_eq!(count_stable_windows(&[3, 3, 3], 0, 4), 0);
        // delta = 0 with no two equal neighbours: only the singles qualify.
        assert_eq!(count_stable_windows(&[1, 2, 3, 4], 0, 1), 4);
        // All identical, generous delta: every one of the n*(n+1)/2 ranges.
        assert_eq!(count_stable_windows(&[7; 10], 0, 1), 55);
        assert_eq!(count_stable_windows(&[7; 10], 5, 1), 55);
        // Extreme values at the type boundary.
        assert_eq!(count_stable_windows(&[0, 1_000_000_000], 1_000_000_000, 1), 3);
        assert_eq!(
            count_stable_windows(&[0, 1_000_000_000], 999_999_999, 1),
            2
        );
        // Ties and duplicates around the window edges.
        assert_eq!(count_stable_windows(&[2, 2, 5, 2, 2], 0, 2), 2);
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x9E3779B97F4A7C15);
        for _ in 0..400 {
            let n = rng.below(30);
            let spread = 1 + rng.below(8);
            let usage: Vec<i64> = (0..n).map(|_| rng.below(spread) as i64).collect();
            let delta = rng.below(6) as i64;
            let min_len = (1 + rng.below(6)) as usize;
            assert_eq!(
                count_stable_windows(&usage, delta, min_len),
                brute(&usage, delta, min_len),
                "usage={:?} delta={} min_len={}",
                usage,
                delta,
                min_len
            );
        }
    }

    #[test]
    fn large_ramp() {
        // 200k strictly increasing readings; answer known in closed form.
        // The brute checker would need ~2 * 10^10 iterations here.
        let n = 200_000usize;
        let usage: Vec<i64> = (0..n as i64).collect();
        assert_eq!(count_stable_windows(&usage, 999, 1), 199_500_500);
        assert_eq!(
            count_stable_windows(&usage, 999, 1),
            ramp_expected(n as u64, 999, 1)
        );
        assert_eq!(count_stable_windows(&usage, 999, 500), 99_824_751);
        assert_eq!(
            count_stable_windows(&usage, 999, 500),
            ramp_expected(n as u64, 999, 500)
        );
    }

    #[test]
    fn large_flat() {
        // Every range qualifies: forces a 64-bit accumulator.
        let n = 200_000u64;
        let usage = vec![17i64; n as usize];
        let expected = n * (n + 1) / 2; // 20_000_100_000
        assert_eq!(count_stable_windows(&usage, 0, 1), expected);
        assert!(
            expected > u32::MAX as u64,
            "this case exists to overflow 32-bit counters"
        );
    }

    #[test]
    fn large_sawtooth() {
        // Alternating extremes: the window never grows past a single sample.
        let n = 200_000u64;
        let usage: Vec<i64> = (0..n)
            .map(|i| if i % 2 == 0 { 0 } else { 1_000_000_000 })
            .collect();
        assert_eq!(count_stable_windows(&usage, 0, 1), n);
        assert_eq!(count_stable_windows(&usage, 0, 2), 0);
        // With the full tolerance every range qualifies again.
        assert_eq!(
            count_stable_windows(&usage, 1_000_000_000, 1),
            n * (n + 1) / 2
        );
    }

    #[test]
    fn large_blocks() {
        // Equal-valued blocks: sum of per-block triangular numbers.
        let n = 200_000u64;
        let block = 500u64;
        let usage: Vec<i64> = (0..n).map(|i| ((i / block) * 10) as i64).collect();
        let blocks = n / block;
        let expected = blocks * (block * (block + 1) / 2);
        assert_eq!(count_stable_windows(&usage, 0, 1), expected);
        // min_len = block leaves exactly one full range per block.
        assert_eq!(count_stable_windows(&usage, 0, block as usize), blocks);
    }

    #[test]
    fn large_random_shape() {
        // Randomised 200k series checked against an independent recomputation.
        // Values only ever move by 0 or +1, so the series is non-decreasing and
        // the count can be recovered by binary search rather than by reusing
        // the solution's own structure.
        let n = 200_000usize;
        let mut rng = Lcg::new(0xDEADBEEFCAFEF00D);
        let mut usage: Vec<i64> = Vec::with_capacity(n);
        let mut cur = 0i64;
        for _ in 0..n {
            cur += rng.below(2) as i64;
            usage.push(cur);
        }
        let delta = 50i64;
        let min_len = 3usize;
        let mut expected = 0u64;
        for j in 0..n {
            // First index whose value is >= usage[j] - delta.
            let i = usage.partition_point(|&v| v < usage[j] - delta);
            if j + 1 >= min_len {
                let last_start = j + 1 - min_len;
                if last_start >= i {
                    expected += (last_start - i + 1) as u64;
                }
            }
        }
        assert_eq!(count_stable_windows(&usage, delta, min_len), expected);
    }
}
