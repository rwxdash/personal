//! Maintenance Windows
//! problems/dp-1d/01-maintenance-windows
//!
//! Fill in `max_maintenance_value`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Maximum total value from scheduling maintenance under a cooldown.
///
/// Choose a set of slots such that no chosen slot is a blackout slot, and any
/// two chosen slots `i < j` satisfy `j - i > cooldown`. Maximise the sum of the
/// chosen slots' values. Choosing nothing is allowed, so the answer is never
/// negative.
///
/// `cooldown` may exceed `n` (then at most one slot fits) and may be `0` (then
/// consecutive slots are allowed). The total can reach 2 * 10^14, hence `i64`.
///
/// Required: O(n) time, O(n) space.
pub fn max_maintenance_value(value: &[i64], blackout: &[bool], cooldown: usize) -> i64 {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check
// enumerates every subset of slots.
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

    /// Try every subset of slots. O(2^n * n); only viable for tiny n.
    fn brute(value: &[i64], blackout: &[bool], cooldown: usize) -> i64 {
        let n = value.len();
        let mut best = 0i64;
        for mask in 0u64..(1u64 << n) {
            let chosen: Vec<usize> = (0..n).filter(|&i| mask & (1 << i) != 0).collect();
            if chosen.iter().any(|&i| blackout[i]) {
                continue;
            }
            let ok = chosen
                .windows(2)
                .all(|w| w[1] - w[0] > cooldown);
            if ok {
                let total: i64 = chosen.iter().map(|&i| value[i]).sum();
                best = best.max(total);
            }
        }
        best
    }

    #[test]
    fn examples() {
        assert_eq!(max_maintenance_value(&[5, 1, 8, 4], &[false; 4], 1), 13);
        assert_eq!(
            max_maintenance_value(&[5, 1, 8, 4], &[false, false, true, false], 1),
            9
        );
        assert_eq!(max_maintenance_value(&[5, 1, 8, 4], &[false; 4], 2), 9);
        assert_eq!(max_maintenance_value(&[5, 1, 8, 4], &[false; 4], 0), 18);
    }

    #[test]
    fn greedy_is_wrong() {
        // Taking the largest available slot first gives the wrong answer.
        assert_eq!(max_maintenance_value(&[4, 5, 4], &[false; 3], 1), 8);
        assert_eq!(
            max_maintenance_value(&[3, 10, 3, 3, 10, 3], &[false; 6], 2),
            20
        );
        assert_eq!(max_maintenance_value(&[5, 6, 5], &[false; 3], 1), 10);
    }

    #[test]
    fn edges() {
        // Empty input.
        assert_eq!(max_maintenance_value(&[], &[], 0), 0);
        assert_eq!(max_maintenance_value(&[], &[], 5), 0);
        // Single slot.
        assert_eq!(max_maintenance_value(&[7], &[false], 0), 7);
        assert_eq!(max_maintenance_value(&[7], &[false], 100), 7);
        assert_eq!(max_maintenance_value(&[7], &[true], 0), 0);
        // Everything blacked out.
        assert_eq!(max_maintenance_value(&[5, 5, 5], &[true; 3], 0), 0);
        // All values zero.
        assert_eq!(max_maintenance_value(&[0, 0, 0], &[false; 3], 1), 0);
        // Cooldown larger than the array: at most one slot, so the largest.
        assert_eq!(max_maintenance_value(&[3, 9, 4], &[false; 3], 3), 9);
        assert_eq!(max_maintenance_value(&[3, 9, 4], &[false; 3], 1000), 9);
        // Cooldown exactly n - 1: still only one slot fits.
        assert_eq!(max_maintenance_value(&[3, 9, 4], &[false; 3], 2), 9);
        // Cooldown 0: sum of every non-blackout slot.
        assert_eq!(
            max_maintenance_value(&[1, 2, 3, 4], &[false, true, false, false], 0),
            8
        );
        // The best single slot is blacked out, so a pair of smaller ones wins.
        assert_eq!(
            max_maintenance_value(&[4, 100, 4], &[false, true, false], 1),
            8
        );
        // Distance exactly cooldown is illegal; cooldown + 1 is legal.
        assert_eq!(max_maintenance_value(&[5, 0, 5], &[false; 3], 2), 5);
        assert_eq!(max_maintenance_value(&[5, 0, 0, 5], &[false; 4], 2), 10);
        // Value ceiling.
        assert_eq!(
            max_maintenance_value(&[1_000_000_000, 1_000_000_000], &[false; 2], 0),
            2_000_000_000
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x428A2F98D728AE22);
        for _ in 0..400 {
            let n = rng.below(13) as usize;
            let value: Vec<i64> = (0..n).map(|_| rng.below(20) as i64).collect();
            let blackout: Vec<bool> = (0..n).map(|_| rng.below(4) == 0).collect();
            let cooldown = rng.below(5) as usize;
            assert_eq!(
                max_maintenance_value(&value, &blackout, cooldown),
                brute(&value, &blackout, cooldown),
                "value={:?} blackout={:?} cooldown={}",
                value,
                blackout,
                cooldown
            );
        }
    }

    #[test]
    fn random_large_cooldown() {
        // Cooldowns that often exceed the array length.
        let mut rng = Lcg::new(0x7137449123EF65CD);
        for _ in 0..300 {
            let n = rng.below(11) as usize;
            let value: Vec<i64> = (0..n).map(|_| rng.below(30) as i64).collect();
            let blackout: Vec<bool> = (0..n).map(|_| rng.below(5) == 0).collect();
            let cooldown = rng.below(15) as usize;
            assert_eq!(
                max_maintenance_value(&value, &blackout, cooldown),
                brute(&value, &blackout, cooldown)
            );
        }
    }

    #[test]
    fn large_no_cooldown() {
        // 200k slots, cooldown 0: the answer is the sum of everything.
        let n = 200_000usize;
        let value = vec![1_000_000_000i64; n];
        assert_eq!(
            max_maintenance_value(&value, &vec![false; n], 0),
            n as i64 * 1_000_000_000
        );
    }

    #[test]
    fn large_alternating() {
        // Cooldown 1 on a uniform array: every other slot.
        let n = 200_000usize;
        let value = vec![7i64; n];
        assert_eq!(
            max_maintenance_value(&value, &vec![false; n], 1),
            (n as i64 / 2) * 7
        );
    }

    #[test]
    fn large_all_blackout() {
        let n = 200_000usize;
        assert_eq!(
            max_maintenance_value(&vec![1_000_000_000i64; n], &vec![true; n], 3),
            0
        );
    }

    #[test]
    fn large_single_choice() {
        // Cooldown at least n: only the single largest slot can be taken.
        let n = 200_000usize;
        let mut rng = Lcg::new(0xB5C0FBCFEC4D3B2F);
        let value: Vec<i64> = (0..n).map(|_| rng.below(1_000_000_000) as i64).collect();
        let want = *value.iter().max().unwrap();
        assert_eq!(max_maintenance_value(&value, &vec![false; n], n), want);
        assert_eq!(max_maintenance_value(&value, &vec![false; n], 200_000), want);
    }

    #[test]
    fn large_spaced_peaks() {
        // Tall peaks spaced exactly far enough apart to all be taken.
        let n = 200_000usize;
        let cooldown = 9usize;
        let mut value = vec![0i64; n];
        let peaks: Vec<usize> = (0..n).step_by(cooldown + 1).collect();
        for &p in &peaks {
            value[p] = 1000;
        }
        assert_eq!(
            max_maintenance_value(&value, &vec![false; n], cooldown),
            peaks.len() as i64 * 1000
        );
    }

    #[test]
    fn large_peaks_too_close() {
        // The same peaks, one slot too close: only every other one fits.
        let n = 200_000usize;
        let cooldown = 10usize; // peaks are 10 apart, which is NOT > 10
        let mut value = vec![0i64; n];
        let peaks: Vec<usize> = (0..n).step_by(10).collect();
        for &p in &peaks {
            value[p] = 1000;
        }
        let expected = ((peaks.len() + 1) / 2) as i64 * 1000;
        assert_eq!(
            max_maintenance_value(&value, &vec![false; n], cooldown),
            expected
        );
    }

    #[test]
    fn large_ramp() {
        // Increasing values with cooldown 1: alternate slots from the end.
        let n = 200_000i64;
        let value: Vec<i64> = (1..=n).collect();
        let expected: i64 = (1..=n).rev().step_by(2).sum();
        assert_eq!(
            max_maintenance_value(&value, &vec![false; n as usize], 1),
            expected
        );
    }
}
