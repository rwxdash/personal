//! Batch Queue Order
//! problems/greedy-exchange-argument/01-batch-queue-order
//!
//! Fill in `min_total_cost`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Minimum total cost of running all jobs on one worker, in the best order.
///
/// The worker starts at time 0 and runs jobs back to back with no gaps. A job
/// finishing at time `t` contributes `rate[i] * t` to the total.
///
/// Durations and rates are each in `1 ..= 10_000`, with up to 100_000 jobs.
/// The total can reach about 5 * 10^17, hence `i64`.
///
/// Required: O(n log n) time, O(n) space.
pub fn min_total_cost(duration: &[i64], rate: &[i64]) -> i64 {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check tries
// every permutation, so a pass is evidence the ordering rule is optimal rather
// than merely self-consistent.
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

    fn cost_of_order(duration: &[i64], rate: &[i64], order: &[usize]) -> i64 {
        let mut clock = 0i64;
        let mut total = 0i64;
        for &i in order {
            clock += duration[i];
            total += rate[i] * clock;
        }
        total
    }

    /// Try every permutation. O(n! * n); only viable for tiny n.
    fn brute(duration: &[i64], rate: &[i64]) -> i64 {
        let n = duration.len();
        if n == 0 {
            return 0;
        }
        let mut order: Vec<usize> = (0..n).collect();
        let mut best = i64::MAX;
        // Heap's algorithm, iterative.
        let mut c = vec![0usize; n];
        best = best.min(cost_of_order(duration, rate, &order));
        let mut i = 0;
        while i < n {
            if c[i] < i {
                if i % 2 == 0 {
                    order.swap(0, i);
                } else {
                    order.swap(c[i], i);
                }
                best = best.min(cost_of_order(duration, rate, &order));
                c[i] += 1;
                i = 0;
            } else {
                c[i] = 0;
                i += 1;
            }
        }
        best
    }

    #[test]
    fn examples() {
        assert_eq!(min_total_cost(&[3, 1], &[1, 1]), 5);
        assert_eq!(min_total_cost(&[3, 1], &[10, 1]), 34);
        assert_eq!(min_total_cost(&[2, 3], &[3, 5]), 30);
        assert_eq!(min_total_cost(&[7], &[4]), 28);
    }

    #[test]
    fn simple_rules_fail() {
        // Sorting by duration ascending would run job 1 first and cost 41.
        assert_eq!(min_total_cost(&[3, 1], &[10, 1]), 34);
        // Sorting by rate descending is no help when all rates are equal.
        assert_eq!(min_total_cost(&[3, 1], &[1, 1]), 5);
        // Both rules disagree with each other here.
        assert_eq!(min_total_cost(&[2, 3], &[3, 5]), 30);
        // Longest job has the lowest rate: it must go last.
        assert_eq!(min_total_cost(&[100, 1, 1], &[1, 100, 100]), 402);
    }

    #[test]
    fn edges() {
        // No jobs.
        assert_eq!(min_total_cost(&[], &[]), 0);
        // One job.
        assert_eq!(min_total_cost(&[1], &[1]), 1);
        assert_eq!(min_total_cost(&[10_000], &[10_000]), 100_000_000);
        // Identical jobs: order cannot matter.
        assert_eq!(min_total_cost(&[5, 5, 5], &[2, 2, 2]), 60);
        // Equal ratios, different magnitudes: any order costs the same.
        assert_eq!(
            min_total_cost(&[2, 4], &[1, 2]),
            min_total_cost(&[4, 2], &[2, 1])
        );
        // All durations 1: driven purely by rate order.
        assert_eq!(min_total_cost(&[1, 1, 1], &[1, 2, 3]), 10);
        // All rates 1: driven purely by duration order.
        assert_eq!(min_total_cost(&[3, 1, 2], &[1, 1, 1]), 10);
        // Value ceilings on both sides.
        assert_eq!(
            min_total_cost(&[10_000, 10_000], &[10_000, 10_000]),
            300_000_000
        );
    }

    #[test]
    fn ties() {
        // Jobs with equal duration/rate ratios cost the same in any order.
        let a = min_total_cost(&[2, 4, 6], &[1, 2, 3]);
        let b = min_total_cost(&[6, 4, 2], &[3, 2, 1]);
        let c = min_total_cost(&[4, 2, 6], &[2, 1, 3]);
        assert_eq!(a, b);
        assert_eq!(b, c);
        // A tie mixed with a clear winner and a clear loser.
        assert_eq!(
            min_total_cost(&[2, 4, 1], &[1, 2, 100]),
            min_total_cost(&[4, 2, 1], &[2, 1, 100])
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x650A73548BAF63DE);
        for _ in 0..300 {
            let n = rng.below(8) as usize;
            let duration: Vec<i64> = (0..n).map(|_| 1 + rng.below(9) as i64).collect();
            let rate: Vec<i64> = (0..n).map(|_| 1 + rng.below(9) as i64).collect();
            assert_eq!(
                min_total_cost(&duration, &rate),
                brute(&duration, &rate),
                "duration={:?} rate={:?}",
                duration,
                rate
            );
        }
    }

    #[test]
    fn random_ratio_collisions() {
        // Values chosen so equal ratios occur constantly -- where a
        // floating-point comparator produces an inconsistent ordering.
        let mut rng = Lcg::new(0x766A0ABB3C77B2A8);
        for _ in 0..300 {
            let n = rng.below(7) as usize;
            let mut duration: Vec<i64> = Vec::new();
            let mut rate: Vec<i64> = Vec::new();
            for _ in 0..n {
                let k = 1 + rng.below(4) as i64;
                duration.push(2 * k);
                rate.push(k); // every ratio is exactly 2
            }
            for i in 0..n {
                if rng.below(3) == 0 {
                    duration[i] += 1;
                }
            }
            assert_eq!(
                min_total_cost(&duration, &rate),
                brute(&duration, &rate),
                "duration={:?} rate={:?}",
                duration,
                rate
            );
        }
    }

    #[test]
    fn large_uniform() {
        // 100k identical jobs: closed form.
        let n = 100_000i64;
        let d = 3i64;
        let r = 2i64;
        let duration = vec![d; n as usize];
        let rate = vec![r; n as usize];
        let expected = r * d * (n * (n + 1) / 2);
        assert_eq!(min_total_cost(&duration, &rate), expected);
    }

    #[test]
    fn large_all_max() {
        // Every value at its ceiling: forces 64-bit arithmetic.
        let n = 100_000i64;
        let duration = vec![10_000i64; n as usize];
        let rate = vec![10_000i64; n as usize];
        let expected = 10_000i64 * 10_000 * (n * (n + 1) / 2);
        assert_eq!(min_total_cost(&duration, &rate), expected);
        assert!(expected > u32::MAX as i64);
    }

    #[test]
    fn large_sorted_input() {
        // Best order and worst order must give the same answer.
        let n = 50_000i64;
        let duration: Vec<i64> = (1..=n).collect();
        let rate = vec![1i64; n as usize];
        let best_first = min_total_cost(&duration, &rate);
        let reversed: Vec<i64> = duration.iter().rev().copied().collect();
        let worst_first = min_total_cost(&reversed, &rate);
        assert_eq!(best_first, worst_first);
        let expected: i64 = (1..=n).map(|k| (n - k + 1) * k).sum();
        assert_eq!(best_first, expected);
    }

    #[test]
    fn large_two_classes() {
        // Half cheap-and-fast, half dear-and-slow.
        let half = 50_000usize;
        let mut duration = vec![1i64; half];
        duration.extend(vec![10_000i64; half]);
        let mut rate = vec![10_000i64; half];
        rate.extend(vec![1i64; half]);
        let mut clock = 0i64;
        let mut expected = 0i64;
        for _ in 0..half {
            clock += 1;
            expected += 10_000 * clock;
        }
        for _ in 0..half {
            clock += 10_000;
            expected += clock;
        }
        assert_eq!(min_total_cost(&duration, &rate), expected);
    }

    #[test]
    fn large_all_ties() {
        // 100k jobs with identical ratios but different magnitudes. Every order
        // is optimal, so the total must match regardless of tie handling.
        let n = 100_000usize;
        let duration: Vec<i64> = (0..n).map(|i| 2 * (1 + (i % 100) as i64)).collect();
        let rate: Vec<i64> = (0..n).map(|i| 1 + (i % 100) as i64).collect();
        let mut clock = 0i64;
        let mut expected = 0i64;
        for i in 0..n {
            clock += duration[i];
            expected += rate[i] * clock;
        }
        assert_eq!(
            min_total_cost(&duration, &rate),
            expected,
            "all ratios equal, so every order costs the same"
        );
    }
}
