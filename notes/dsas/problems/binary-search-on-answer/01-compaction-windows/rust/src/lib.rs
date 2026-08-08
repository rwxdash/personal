//! Compaction Windows
//! problems/binary-search-on-answer/01-compaction-windows
//!
//! Fill in `min_window_bytes`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Minimise the largest compaction window's byte total.
///
/// Split `sizes` into at most `k` contiguous, non-empty windows covering every
/// segment in order. A checkpoint segment must be the FIRST segment of its
/// window; `checkpoints[0]` is ignored, since segment 0 always begins the
/// first window.
///
/// Guaranteed: at most `k - 1` checkpoints among indices `1..n-1`, so a valid
/// split always exists.
///
/// The answer can reach 2 * 10^14, hence `i64`.
///
/// Required: O(n log T) time where T is the total byte count, O(1) extra space.
pub fn min_window_bytes(sizes: &[i64], checkpoints: &[bool], k: usize) -> i64 {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check uses an
// O(n^2 k) dynamic program over split points, sharing no reasoning with a
// search over candidate answers.
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

    /// Exact answer by DP over (segments consumed, windows used). O(n^2 k).
    fn brute(sizes: &[i64], checkpoints: &[bool], k: usize) -> i64 {
        let n = sizes.len();
        let mut prefix = vec![0i64; n + 1];
        for i in 0..n {
            prefix[i + 1] = prefix[i] + sizes[i];
        }
        const INF: i64 = i64::MAX / 4;
        let mut dp = vec![vec![INF; k + 1]; n + 1];
        dp[0][0] = 0;
        for i in 1..=n {
            for j in 1..=k {
                // Last window covers segments [t .. i-1].
                for t in (0..i).rev() {
                    if dp[t][j - 1] < INF {
                        let cand = dp[t][j - 1].max(prefix[i] - prefix[t]);
                        if cand < dp[i][j] {
                            dp[i][j] = cand;
                        }
                    }
                    // A checkpoint at t means no window may start before t and
                    // still contain t, so stop widening this window.
                    if t > 0 && checkpoints[t] {
                        break;
                    }
                }
            }
        }
        (1..=k).map(|j| dp[n][j]).min().unwrap()
    }

    #[test]
    fn examples() {
        assert_eq!(min_window_bytes(&[7, 2, 5, 10, 8], &[false; 5], 2), 18);
        assert_eq!(min_window_bytes(&[7, 2, 5, 10, 8], &[false; 5], 3), 14);
        assert_eq!(
            min_window_bytes(&[7, 2, 5, 10, 8], &[false, false, true, false, false], 3),
            15
        );
        assert_eq!(min_window_bytes(&[4, 9, 2], &[false; 3], 3), 9);
    }

    #[test]
    fn edges() {
        // Single segment.
        assert_eq!(min_window_bytes(&[5], &[false], 1), 5);
        assert_eq!(
            min_window_bytes(&[5], &[true], 1),
            5,
            "checkpoints[0] is ignored"
        );
        // One window: everything together.
        assert_eq!(min_window_bytes(&[1, 2, 3, 4], &[false; 4], 1), 10);
        // A window per segment: the answer is the largest segment.
        assert_eq!(min_window_bytes(&[1, 2, 3, 4], &[false; 4], 4), 4);
        // More windows than needed changes nothing once each segment is alone.
        assert_eq!(min_window_bytes(&[3, 3, 3], &[false; 3], 3), 3);
        // Uniform sizes split evenly.
        assert_eq!(min_window_bytes(&[2; 6], &[false; 6], 3), 4);
        assert_eq!(min_window_bytes(&[2; 6], &[false; 6], 2), 6);
        // One dominant segment sets the floor.
        assert_eq!(min_window_bytes(&[1, 1, 100, 1, 1], &[false; 5], 3), 100);
        // Checkpoint on the last segment forces it alone.
        assert_eq!(
            min_window_bytes(&[1, 1, 1, 50], &[false, false, false, true], 2),
            50
        );
        // Every segment after the first is a checkpoint.
        assert_eq!(min_window_bytes(&[4, 9, 2], &[false, true, true], 3), 9);
        // Checkpoint that costs nothing, because the cut was optimal anyway.
        assert_eq!(
            min_window_bytes(&[5, 5, 5, 5], &[false, false, true, false], 2),
            10
        );
    }

    #[test]
    fn checkpoints_bind() {
        // A forced cut must fire even when the current window has room left.
        let sizes = [1i64, 1, 1, 1, 8];
        assert_eq!(min_window_bytes(&sizes, &[false; 5], 2), 8);
        // Forcing a window to start at index 1 strands [1] alone, so the rest
        // must fit in one window: 1 + 1 + 1 + 8 = 11.
        assert_eq!(
            min_window_bytes(&sizes, &[false, true, false, false, false], 2),
            11
        );
        // Two checkpoints with k = 3.
        assert_eq!(
            min_window_bytes(&[6, 6, 6, 6], &[false, true, true, false], 3),
            12
        );
        // Consecutive checkpoints each start their own window.
        assert_eq!(min_window_bytes(&[1, 2, 3], &[false, true, true], 3), 3);
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x8AA07D6C1F6D3A2B);
        for _ in 0..400 {
            let n = (1 + rng.below(9)) as usize;
            let sizes: Vec<i64> = (0..n).map(|_| 1 + rng.below(12) as i64).collect();
            // Choose k first, then place at most k-1 checkpoints so the
            // instance is guaranteed solvable.
            let k = (1 + rng.below(n as u64)) as usize;
            let mut checkpoints = vec![false; n];
            let mut budget = k - 1;
            for i in 1..n {
                if budget > 0 && rng.below(3) == 0 {
                    checkpoints[i] = true;
                    budget -= 1;
                }
            }
            assert_eq!(
                min_window_bytes(&sizes, &checkpoints, k),
                brute(&sizes, &checkpoints, k),
                "sizes={:?} checkpoints={:?} k={}",
                sizes,
                checkpoints,
                k
            );
        }
    }

    #[test]
    fn large_uniform() {
        // 200k equal segments into k windows: a clean closed form.
        let n = 200_000usize;
        let sizes = vec![1000i64; n];
        let cps = vec![false; n];
        for k in [1usize, 2, 4, 5, 1000] {
            let per = (n + k - 1) / k;
            assert_eq!(min_window_bytes(&sizes, &cps, k), per as i64 * 1000);
        }
    }

    #[test]
    fn large_single_dominant() {
        // One huge segment sets the floor no matter how many windows exist.
        let n = 200_000usize;
        let mut sizes = vec![1i64; n];
        sizes[123_456] = 1_000_000_000;
        assert_eq!(
            min_window_bytes(&sizes, &vec![false; n], 1000),
            1_000_000_000
        );
    }

    #[test]
    fn large_all_max() {
        // Every segment at the value ceiling: total 2 * 10^14, needs 64 bits.
        let n = 200_000usize;
        let sizes = vec![1_000_000_000i64; n];
        let cps = vec![false; n];
        assert_eq!(min_window_bytes(&sizes, &cps, 1), n as i64 * 1_000_000_000);
        assert_eq!(
            min_window_bytes(&sizes, &cps, 2),
            (n as i64 / 2) * 1_000_000_000
        );
        assert_eq!(min_window_bytes(&sizes, &cps, n), 1_000_000_000);
    }

    #[test]
    fn large_max_windows() {
        // k = n means every segment is alone.
        let n = 200_000usize;
        let mut rng = Lcg::new(0x5C4E2A1B9F03D7E6);
        let sizes: Vec<i64> = (0..n)
            .map(|_| 1 + rng.below(1_000_000_000) as i64)
            .collect();
        let want = *sizes.iter().max().unwrap();
        assert_eq!(min_window_bytes(&sizes, &vec![false; n], n), want);
    }

    #[test]
    fn large_checkpoint_every_other() {
        // A checkpoint at every odd index means 100_000 forced cuts, so k must
        // be at least 100_001. At exactly that k the split is fully determined:
        // windows are [0], [1,2], [3,4], ..., [n-1].
        let n = 200_000usize;
        let mut sizes = vec![1i64; n];
        sizes[7] = 5; // index 7 is odd, so it starts a window; its pair is (7, 8)
        let checkpoints: Vec<bool> = (0..n).map(|i| i % 2 == 1).collect();
        let forced = checkpoints[1..].iter().filter(|&&c| c).count();
        let k = forced + 1;
        assert_eq!(k, 100_001);
        // Every window is a (odd, even) pair totalling 2, except [0] and [n-1]
        // which hold one segment each, and [7, 8] which holds 5 + 1 = 6.
        assert_eq!(min_window_bytes(&sizes, &checkpoints, k), 6);
    }

    #[test]
    fn large_binary_search_range() {
        // Forces many search iterations: a wide value range, unique answer.
        let n = 100_000usize;
        let sizes: Vec<i64> = (0..n)
            .map(|i| if i == 0 { 1_000_000_000 } else { 1 })
            .collect();
        let cps = vec![false; n];
        assert_eq!(min_window_bytes(&sizes, &cps, 2), 1_000_000_000);
        assert_eq!(
            min_window_bytes(&sizes, &cps, 1),
            1_000_000_000 + (n as i64 - 1)
        );
    }
}
