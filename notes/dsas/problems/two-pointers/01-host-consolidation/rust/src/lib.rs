//! Host Consolidation
//! problems/two-pointers/01-host-consolidation
//!
//! Fill in `min_hosts`, then run `cargo test` in this directory.
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Minimum number of hosts needed to place every VM.
///
/// A host has capacity `cap` and may run at most two VMs; two VMs share a host
/// only if their footprints sum to at most `cap`. A pinned VM always occupies a
/// host by itself.
///
/// `footprints[i] <= cap` is guaranteed, so a placement always exists.
///
/// Required: O(n log n) time, O(n) space.
pub fn min_hosts(footprints: &[i64], pinned: &[bool], cap: i64) -> usize {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check solves
// the same instances by exhaustive bitmask matching, which shares no reasoning
// with the intended approach.
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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

    /// Exponential exact answer via maximum matching over compatible pairs:
    /// hosts = pinned_count + (unpinned_count - max_pairs).
    fn brute(footprints: &[i64], pinned: &[bool], cap: i64) -> usize {
        let free: Vec<i64> = footprints
            .iter()
            .zip(pinned)
            .filter(|(_, &p)| !p)
            .map(|(&f, _)| f)
            .collect();
        let n = free.len();
        let mut memo: HashMap<u32, usize> = HashMap::new();

        fn max_pairs(mask: u32, free: &[i64], cap: i64, memo: &mut HashMap<u32, usize>) -> usize {
            if mask == 0 {
                return 0;
            }
            if let Some(&v) = memo.get(&mask) {
                return v;
            }
            let i = mask.trailing_zeros() as usize;
            let rest = mask ^ (1 << i);
            let mut best = max_pairs(rest, free, cap, memo); // leave VM i alone
            let mut m = rest;
            while m != 0 {
                let j = m.trailing_zeros() as usize;
                if free[i] + free[j] <= cap {
                    let cand = 1 + max_pairs(rest ^ (1 << j), free, cap, memo);
                    if cand > best {
                        best = cand;
                    }
                }
                m ^= 1 << j;
            }
            memo.insert(mask, best);
            best
        }

        let pinned_count = pinned.iter().filter(|&&p| p).count();
        let full = if n == 0 { 0u32 } else { (1u32 << n) - 1 };
        pinned_count + n - max_pairs(full, &free, cap, &mut memo)
    }

    #[test]
    fn examples() {
        assert_eq!(min_hosts(&[1, 2], &[false, false], 3), 1);
        assert_eq!(min_hosts(&[3, 2, 2, 1], &[false; 4], 3), 3);
        assert_eq!(min_hosts(&[1, 1, 1], &[true, false, false], 2), 2);
        assert_eq!(min_hosts(&[4; 6], &[false; 6], 7), 6);
        assert_eq!(min_hosts(&[4; 6], &[false; 6], 8), 3);
    }

    #[test]
    fn edges() {
        // No VMs.
        assert_eq!(min_hosts(&[], &[], 5), 0);
        // One VM, pinned or not.
        assert_eq!(min_hosts(&[5], &[false], 5), 1);
        assert_eq!(min_hosts(&[5], &[true], 5), 1);
        // Every VM pinned: pairing is never allowed.
        assert_eq!(min_hosts(&[1, 1, 1], &[true; 3], 10), 3);
        // Capacity exactly twice the footprint: everything pairs.
        assert_eq!(min_hosts(&[4; 6], &[false; 6], 8), 3);
        // Odd count that pairs perfectly except for one leftover.
        assert_eq!(min_hosts(&[1, 1, 1], &[false; 3], 2), 2);
        // Largest VM fills a host alone; the rest pair up.
        assert_eq!(min_hosts(&[5, 1, 1], &[false; 3], 5), 2);
        // A VM equal to cap can never share.
        assert_eq!(min_hosts(&[3, 3, 3], &[false; 3], 3), 3);
        // Mixed pinning where the pinned one would otherwise have paired.
        assert_eq!(min_hosts(&[1, 1], &[true, false], 2), 2);
        assert_eq!(min_hosts(&[1, 1], &[false, false], 2), 1);
        // Pairing must be smallest-with-largest, not adjacent-in-input.
        assert_eq!(min_hosts(&[5, 4, 3, 2], &[false; 4], 7), 2);
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x13198A2E03707344);
        for _ in 0..300 {
            let n = rng.below(11) as usize;
            let cap = (1 + rng.below(20)) as i64;
            let footprints: Vec<i64> = (0..n).map(|_| 1 + rng.below(cap as u64) as i64).collect();
            let pinned: Vec<bool> = (0..n).map(|_| rng.below(4) == 0).collect();
            assert_eq!(
                min_hosts(&footprints, &pinned, cap),
                brute(&footprints, &pinned, cap),
                "footprints={:?} pinned={:?} cap={}",
                footprints,
                pinned,
                cap
            );
        }
    }

    #[test]
    fn large_perfect_pairing() {
        // 200k VMs that pair exactly; answer is n / 2 by construction.
        let n = 200_000usize;
        let footprints: Vec<i64> = (0..n).map(|i| if i % 2 == 0 { 1 } else { 99 }).collect();
        assert_eq!(min_hosts(&footprints, &vec![false; n], 100), n / 2);
    }

    #[test]
    fn large_nothing_pairs() {
        // Every pair overflows by exactly one byte.
        let n = 200_000usize;
        let footprints = vec![60i64; n];
        let pinned = vec![false; n];
        assert_eq!(min_hosts(&footprints, &pinned, 119), n);
        // One more byte of capacity and everything pairs.
        assert_eq!(min_hosts(&footprints, &pinned, 120), n / 2);
    }

    #[test]
    fn large_all_pinned() {
        let n = 200_000usize;
        assert_eq!(
            min_hosts(&vec![1i64; n], &vec![true; n], 1_000_000_000),
            n
        );
    }

    #[test]
    fn large_half_pinned() {
        // Half pinned, half perfectly pairable.
        let n = 200_000usize;
        let footprints = vec![50i64; n];
        let pinned: Vec<bool> = (0..n).map(|i| i % 2 == 0).collect();
        // n/2 pinned take one host each; the other n/2 pair into n/4 hosts.
        assert_eq!(min_hosts(&footprints, &pinned, 100), n / 2 + n / 4);
    }

    #[test]
    fn large_staircase() {
        // Sizes 1..n with cap = n + 1: i pairs with n + 1 - i, so n/2 hosts.
        let n = 200_000usize;
        let footprints: Vec<i64> = (1..=n as i64).collect();
        let pinned = vec![false; n];
        assert_eq!(min_hosts(&footprints, &pinned, n as i64 + 1), n / 2);
        // cap = n leaves the largest VM unpaired, shifting the whole cascade.
        assert_eq!(min_hosts(&footprints, &pinned, n as i64), n / 2 + 1);
    }
}
