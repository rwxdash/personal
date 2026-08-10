//! Settlement Runs
//! problems/prefix-sums/01-settlement-runs
//!
//! Fill in `settlement_runs`, then run `cargo test` in this directory.
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Count settleable runs and measure the longest one.
///
/// A contiguous run of `deltas` is settleable when its sum is an exact multiple
/// of `m`. Zero counts (it is a multiple of everything), and negative sums
/// count when their magnitude is a multiple of `m`.
///
/// Runs are identified by position: `deltas[3..=7]` and `deltas[4..=8]` are
/// distinct even if they hold the same values.
///
/// Returns `(count, longest_length)`. `count` can reach ~2 * 10^10, hence
/// `u64`; `longest_length` is 0 when no run settles.
///
/// Required: O(n) time expected, O(n) space.
pub fn settlement_runs(deltas: &[i64], m: i64) -> (u64, usize) {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. Large cases use ledgers whose
// answers are known in closed form.
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
    fn brute(deltas: &[i64], m: i64) -> (u64, usize) {
        let mut count = 0u64;
        let mut longest = 0usize;
        for i in 0..deltas.len() {
            let mut total: i64 = 0;
            for j in i..deltas.len() {
                total += deltas[j];
                if total.rem_euclid(m) == 0 {
                    count += 1;
                    if j - i + 1 > longest {
                        longest = j - i + 1;
                    }
                }
            }
        }
        (count, longest)
    }

    #[test]
    fn examples() {
        assert_eq!(settlement_runs(&[2, -2, 3], 3), (3, 3));
        assert_eq!(settlement_runs(&[-1, -2], 3), (1, 2));
        assert_eq!(settlement_runs(&[5, -3, 7], 1), (6, 3));
        assert_eq!(settlement_runs(&[1, 1], 5), (0, 0));
    }

    #[test]
    fn edges() {
        // Empty ledger.
        assert_eq!(settlement_runs(&[], 5), (0, 0));
        assert_eq!(settlement_runs(&[], 1), (0, 0));
        // Single transaction, settleable or not.
        assert_eq!(settlement_runs(&[4], 2), (1, 1));
        assert_eq!(settlement_runs(&[3], 2), (0, 0));
        // Zero is a multiple of everything.
        assert_eq!(settlement_runs(&[0], 7), (1, 1));
        assert_eq!(settlement_runs(&[0, 0, 0], 7), (6, 3));
        // Negative single values.
        assert_eq!(settlement_runs(&[-6], 3), (1, 1));
        assert_eq!(settlement_runs(&[-5], 3), (0, 0));
        // A run summing to exactly zero settles under any unit.
        assert_eq!(settlement_runs(&[7, -7], 1_000_000_000), (1, 2));
        // Mixed signs where only the full ledger settles.
        assert_eq!(settlement_runs(&[1, -4, 3], 100), (1, 3));
        // The longest run appears early; later runs are shorter.
        assert_eq!(settlement_runs(&[3, 3, 1, 1, 3], 3), (4, 2));
        // Every prefix settles: count is triangular, longest is n.
        assert_eq!(settlement_runs(&[5, 5, 5, 5], 5), (10, 4));
    }

    #[test]
    fn negative_modulo() {
        // Residues of negative running totals must be normalised into 0..m-1.
        // In Rust `-1 % 3 == -1`, so keying a map on the raw remainder splits
        // one residue class across two buckets and loses runs. Use rem_euclid.
        assert_eq!(settlement_runs(&[-1, -2], 3), (1, 2));
        assert_eq!(settlement_runs(&[-3, -3, -3], 3), (6, 3));
        assert_eq!(settlement_runs(&[-1, 4, -3], 3), (3, 3));
        // Running total dips negative then recovers to the same residue.
        assert_eq!(settlement_runs(&[-5, 5], 5), (3, 2));
        assert_eq!(settlement_runs(&[2, -5, 3], 5), (2, 3));
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x452821E638D01377);
        for _ in 0..400 {
            let n = rng.below(25);
            let m = (1 + rng.below(7)) as i64;
            let deltas: Vec<i64> = (0..n).map(|_| rng.below(21) as i64 - 10).collect();
            assert_eq!(
                settlement_runs(&deltas, m),
                brute(&deltas, m),
                "deltas={:?} m={}",
                deltas,
                m
            );
        }
    }

    #[test]
    fn random_large_modulus() {
        // Same cross-check with m far larger than any reachable sum.
        let mut rng = Lcg::new(0xBE5466CF34E90C6C);
        for _ in 0..200 {
            let n = rng.below(18);
            let deltas: Vec<i64> = (0..n).map(|_| rng.below(11) as i64 - 5).collect();
            let m = 1_000_000_000i64;
            assert_eq!(
                settlement_runs(&deltas, m),
                brute(&deltas, m),
                "deltas={:?} m={}",
                deltas,
                m
            );
        }
    }

    #[test]
    fn large_all_zero() {
        // Every run settles: count is triangular and needs 64 bits.
        let n = 200_000u64;
        let deltas = vec![0i64; n as usize];
        let expected = n * (n + 1) / 2; // 20_000_100_000
        assert_eq!(settlement_runs(&deltas, 7), (expected, n as usize));
        assert!(
            expected > u32::MAX as u64,
            "this case exists to overflow 32-bit counters"
        );
    }

    #[test]
    fn large_unit_modulus() {
        // m = 1 makes every run settleable regardless of the values.
        let n = 200_000u64;
        let mut rng = Lcg::new(0x3F84D5B5B5470917);
        let deltas: Vec<i64> = (0..n)
            .map(|_| rng.below(2_000_000_001) as i64 - 1_000_000_000)
            .collect();
        assert_eq!(settlement_runs(&deltas, 1), (n * (n + 1) / 2, n as usize));
    }

    #[test]
    fn large_single_run() {
        // Prefix totals are 0..n and the modulus is n, so the only repeated
        // residue is 0 (empty prefix and end) -- exactly one settleable run.
        let n = 200_000usize;
        let deltas = vec![1i64; n];
        assert_eq!(settlement_runs(&deltas, n as i64), (1, n));
    }

    #[test]
    fn large_periodic() {
        // Prefix residues cycle with period p. Class 0 is hit n/p + 1 times
        // (the empty prefix); every other class n/p times.
        let n = 200_000u64;
        let p = 4u64;
        let deltas = vec![1i64; n as usize];
        let hits_zero = n / p + 1;
        let hits_other = n / p;
        let expected =
            hits_zero * (hits_zero - 1) / 2 + (p - 1) * (hits_other * (hits_other - 1) / 2);
        assert_eq!(settlement_runs(&deltas, p as i64), (expected, n as usize));
    }

    #[test]
    fn large_alternating() {
        // Alternating +v / -v: prefix totals toggle between 0 and v, so the
        // residues split into exactly two classes.
        let n = 200_000u64;
        let v = 1_000_000_000i64;
        let deltas: Vec<i64> = (0..n).map(|i| if i % 2 == 0 { v } else { -v }).collect();
        let m = 7i64;
        let half = n / 2;
        let zeros = half + 1;
        let others = half;
        let expected = zeros * (zeros - 1) / 2 + others * (others - 1) / 2;
        assert_eq!(settlement_runs(&deltas, m), (expected, n as usize));
    }

    #[test]
    fn large_no_settlement() {
        // Prefix totals 0..n are all distinct residues when m > n.
        let n = 100_000usize;
        let deltas = vec![1i64; n];
        assert_eq!(settlement_runs(&deltas, n as i64 + 1), (0, 0));
    }
}
