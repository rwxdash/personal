//! Gateway Peak Concurrency
//! problems/intervals-sweep-line/01-gateway-peak-concurrency
//!
//! Fill in `peak_concurrency`, then run `cargo test` in this directory.
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Largest number of simultaneously open sessions, and when it first occurs.
///
/// A session occupies a connection slot over the HALF-OPEN interval
/// `[start, end)`: it holds the slot at instant `start` and has already
/// released it at instant `end`. A session closing at `t` and another opening
/// at `t` therefore do NOT overlap.
///
/// `sessions` is unsorted, `start <= end`, and zero-length sessions
/// (`start == end`) are legal — they occupy no instants at all and never
/// contribute to concurrency.
///
/// Returns `(peak, earliest_time)`. When the peak is 0 — no sessions, or
/// nothing but zero-length ones — returns `(0, 0)`.
///
/// Required: O(n log n) time, O(n) space.
pub fn peak_concurrency(sessions: &[(i64, i64)]) -> (usize, i64) {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check
// evaluates concurrency directly at every candidate instant, sharing no
// machinery with an event sweep.
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

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

    /// Evaluate concurrency at every distinct start. O(n^2). Concurrency only
    /// rises at a start, so the peak is attained at one of them.
    fn brute(sessions: &[(i64, i64)]) -> (usize, i64) {
        let starts: BTreeSet<i64> = sessions.iter().map(|&(s, _)| s).collect();
        let mut best = 0usize;
        let mut best_time = 0i64;
        for t in starts {
            let c = sessions.iter().filter(|&&(s, e)| s <= t && t < e).count();
            if c > best {
                best = c;
                best_time = t;
            }
        }
        (best, best_time)
    }

    #[test]
    fn examples() {
        assert_eq!(peak_concurrency(&[(1, 5), (2, 6), (8, 10)]), (2, 2));
        assert_eq!(peak_concurrency(&[(1, 5), (5, 9)]), (1, 1));
        assert_eq!(peak_concurrency(&[(0, 100), (10, 20), (12, 15)]), (3, 12));
        assert_eq!(peak_concurrency(&[(5, 5), (5, 10)]), (1, 5));
    }

    #[test]
    fn half_open() {
        // Back-to-back sessions must never be counted as overlapping. Every
        // assert here flips if closing events are ordered after opening events
        // at the same timestamp.
        assert_eq!(peak_concurrency(&[(1, 5), (5, 9)]), (1, 1));
        assert_eq!(
            peak_concurrency(&[(0, 1), (1, 2), (2, 3), (3, 4)]),
            (1, 0)
        );
        // A chain of handovers plus one genuine overlap.
        assert_eq!(peak_concurrency(&[(0, 10), (10, 20), (5, 15)]), (2, 5));
        // Overlap by exactly one instant.
        assert_eq!(peak_concurrency(&[(0, 6), (5, 10)]), (2, 5));
        // Miss by exactly one instant.
        assert_eq!(peak_concurrency(&[(0, 5), (5, 10)]), (1, 0));
    }

    #[test]
    fn zero_length() {
        // Instantaneous sessions occupy no instants and never add concurrency.
        assert_eq!(peak_concurrency(&[(5, 5)]), (0, 0));
        assert_eq!(peak_concurrency(&[(5, 5), (5, 5), (5, 5)]), (0, 0));
        assert_eq!(peak_concurrency(&[(5, 5), (5, 10)]), (1, 5));
        assert_eq!(peak_concurrency(&[(0, 10), (5, 5)]), (1, 0));
        // A zero-length session inside a busy stretch changes nothing.
        assert_eq!(peak_concurrency(&[(0, 10), (2, 8), (5, 5)]), (2, 2));
    }

    #[test]
    fn edges() {
        // Empty input.
        assert_eq!(peak_concurrency(&[]), (0, 0));
        // Single session.
        assert_eq!(peak_concurrency(&[(0, 1)]), (1, 0));
        assert_eq!(peak_concurrency(&[(7, 100)]), (1, 7));
        // Identical sessions all count.
        assert_eq!(peak_concurrency(&[(3, 7), (3, 7), (3, 7)]), (3, 3));
        // Disjoint: earliest is the first start in TIME order, not input order.
        assert_eq!(peak_concurrency(&[(50, 60), (10, 20)]), (1, 10));
        // Fully nested.
        assert_eq!(
            peak_concurrency(&[(0, 100), (25, 75), (40, 60), (45, 55)]),
            (4, 45)
        );
        // Timestamp range boundaries.
        assert_eq!(peak_concurrency(&[(0, 1_000_000_000)]), (1, 0));
        assert_eq!(
            peak_concurrency(&[(0, 1_000_000_000), (999_999_999, 1_000_000_000)]),
            (2, 999_999_999)
        );
        // Peak reached early, tied again later: report the earliest.
        assert_eq!(
            peak_concurrency(&[(0, 5), (0, 5), (100, 105), (100, 105)]),
            (2, 0)
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x2AF26013C5D1B023);
        for _ in 0..400 {
            let n = rng.below(14);
            let mut sessions: Vec<(i64, i64)> = Vec::new();
            for _ in 0..n {
                let a = rng.below(12) as i64;
                let b = rng.below(12) as i64;
                sessions.push((a.min(b), a.max(b)));
            }
            assert_eq!(
                peak_concurrency(&sessions),
                brute(&sessions),
                "sessions={:?}",
                sessions
            );
        }
    }

    #[test]
    fn random_clustered() {
        // Tiny coordinate range, so shared timestamps are constant.
        let mut rng = Lcg::new(0x70B7B98B31A2C0A9);
        for _ in 0..400 {
            let n = rng.below(10);
            let mut sessions: Vec<(i64, i64)> = Vec::new();
            for _ in 0..n {
                let a = rng.below(4) as i64;
                let b = rng.below(4) as i64;
                sessions.push((a.min(b), a.max(b)));
            }
            assert_eq!(peak_concurrency(&sessions), brute(&sessions));
        }
    }

    #[test]
    fn large_all_overlapping() {
        // 200k sessions spanning the whole timeline: peak is n at instant 0.
        let n = 200_000usize;
        let sessions = vec![(0i64, 1_000_000_000i64); n];
        assert_eq!(peak_concurrency(&sessions), (n, 0));
    }

    #[test]
    fn large_handover_chain() {
        // 200k back-to-back sessions: peak is 1 despite 200k shared timestamps.
        // Ordering opens before closes reports 2 here.
        let n = 200_000i64;
        let sessions: Vec<(i64, i64)> = (0..n).map(|i| (i, i + 1)).collect();
        assert_eq!(peak_concurrency(&sessions), (1, 0));
    }

    #[test]
    fn large_staircase() {
        // Width-k sessions at every index: peak is k, first reached at k - 1.
        let n = 200_000i64;
        let k = 1_000i64;
        let sessions: Vec<(i64, i64)> = (0..n).map(|i| (i, i + k)).collect();
        assert_eq!(peak_concurrency(&sessions), (k as usize, k - 1));
    }

    #[test]
    fn large_nested() {
        // 100k nested sessions: peak is n at the innermost start.
        let n = 100_000i64;
        let sessions: Vec<(i64, i64)> = (0..n).map(|i| (i, 2 * n - i)).collect();
        assert_eq!(peak_concurrency(&sessions), (n as usize, n - 1));
    }

    #[test]
    fn large_disjoint() {
        // 200k sessions with a gap between each: peak 1, earliest instant 0.
        let n = 200_000i64;
        let sessions: Vec<(i64, i64)> = (0..n).map(|i| (3 * i, 3 * i + 1)).collect();
        assert_eq!(peak_concurrency(&sessions), (1, 0));
    }

    #[test]
    fn large_zero_length_flood() {
        // Mostly instantaneous sessions with a single real one buried inside.
        let n = 200_000i64;
        let mut sessions: Vec<(i64, i64)> = (0..n).map(|i| (i, i)).collect();
        sessions.push((12_345, 54_321));
        assert_eq!(peak_concurrency(&sessions), (1, 12_345));
    }

    #[test]
    fn large_unsorted() {
        // The same staircase shuffled: input order must not matter.
        let n = 100_000usize;
        let k = 500i64;
        let mut sessions: Vec<(i64, i64)> =
            (0..n as i64).map(|i| (i, i + k)).collect();
        let mut rng = Lcg::new(0xF1BBCDCB7A0EA9C7);
        for i in (1..n).rev() {
            let j = rng.below(i as u64 + 1) as usize;
            sessions.swap(i, j);
        }
        assert_eq!(peak_concurrency(&sessions), (k as usize, k - 1));
    }
}
