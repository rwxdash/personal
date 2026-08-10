//! Extent Cover
//! problems/bit-manipulation/01b-extent-cover
//!
//! Fill in `extent_cover`, then run `cargo test` in this directory.
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Fewest naturally-aligned extents that exactly cover each byte range.
///
/// An extent `(offset, size)` is legal when `size` is a power of two, `offset`
/// is a multiple of `size`, and the extent lies entirely inside the request.
///
/// Each input is a `(start, end)` byte request, inclusive, with `start <= end`
/// and `end < 2^48`. Up to 50_000 requests.
///
/// Returns one list per request: the smallest possible set of legal extents
/// whose covered bytes are exactly `start..=end`, sorted by offset ascending.
/// No request needs more than 94 extents.
///
/// Required: O(total extents returned) time, O(1) space beyond the output.
pub fn extent_cover(ranges: &[(u64, u64)]) -> Vec<Vec<(u64, u64)>> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check finds
// the minimum by shortest path over positions, which assumes nothing about
// which extent to pick at each step.
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, VecDeque};

    const LIMIT: u64 = 1 << 48;
    const MAX_BYTE: u64 = LIMIT - 1;

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

    /// Every rule from the statement, checked independently of any algorithm.
    fn check_contract(start: u64, end: u64, extents: &[(u64, u64)]) {
        assert!(!extents.is_empty(), "empty cover for ({}, {})", start, end);
        let mut pos = start;
        for &(off, size) in extents {
            assert!(
                size > 0 && size & (size - 1) == 0,
                "size {} is not a power of two, range=({}, {})",
                size,
                start,
                end
            );
            assert_eq!(
                off % size,
                0,
                "offset {} not aligned to size {}, range=({}, {})",
                off,
                size,
                start,
                end
            );
            assert_eq!(
                off, pos,
                "gap or overlap at {} (expected {}), range=({}, {})",
                off, pos, start, end
            );
            assert!(
                off + size - 1 <= end,
                "extent ({}, {}) spills past {}",
                off,
                size,
                end
            );
            pos = off + size;
        }
        assert_eq!(
            pos,
            end + 1,
            "cover stops at {}, range=({}, {})",
            pos - 1,
            start,
            end
        );
    }

    /// A minimal cover can never hold two adjacent equal extents that combine.
    fn no_mergeable_pair(extents: &[(u64, u64)]) {
        for w in extents.windows(2) {
            let ((o1, s1), (o2, s2)) = (w[0], w[1]);
            assert!(
                !(s1 == s2 && o1 % (2 * s1) == 0),
                "({}, {}) and ({}, {}) merge into one extent",
                o1,
                s1,
                o2,
                s2
            );
        }
    }

    /// Fewest extents, by shortest path from start to end+1 over legal steps.
    fn brute_min(start: u64, end: u64) -> usize {
        let target = end + 1;
        let mut dist: HashMap<u64, usize> = HashMap::new();
        dist.insert(start, 0);
        let mut queue = VecDeque::new();
        queue.push_back(start);
        while let Some(p) = queue.pop_front() {
            if p == target {
                return dist[&p];
            }
            let d = dist[&p];
            for k in 0..64u32 {
                let size = 1u64 << k;
                if p % size != 0 || p + size > target {
                    break;
                }
                dist.entry(p + size).or_insert_with(|| {
                    queue.push_back(p + size);
                    d + 1
                });
            }
        }
        panic!("no cover for ({}, {})", start, end);
    }

    #[test]
    fn examples() {
        assert_eq!(extent_cover(&[(0, 7)]), vec![vec![(0, 8)]]);
        assert_eq!(
            extent_cover(&[(1, 6)]),
            vec![vec![(1, 1), (2, 2), (4, 2), (6, 1)]]
        );
        assert_eq!(extent_cover(&[(4, 11)]), vec![vec![(4, 4), (8, 4)]]);
        assert_eq!(extent_cover(&[(5, 5)]), vec![vec![(5, 1)]]);
        assert_eq!(extent_cover(&[(0, MAX_BYTE)]), vec![vec![(0, LIMIT)]]);
    }

    /// Position 0 has no alignment limit -- only the remaining length caps it.
    #[test]
    fn zero_start() {
        assert_eq!(extent_cover(&[(0, 0)]), vec![vec![(0, 1)]]);
        assert_eq!(extent_cover(&[(0, 1)]), vec![vec![(0, 2)]]);
        assert_eq!(extent_cover(&[(0, 2)]), vec![vec![(0, 2), (2, 1)]]);
        assert_eq!(extent_cover(&[(0, 1023)]), vec![vec![(0, 1024)]]);
        // The widest possible request, where the size reaches 2^48.
        assert_eq!(extent_cover(&[(0, MAX_BYTE)]), vec![vec![(0, LIMIT)]]);
    }

    #[test]
    fn single_byte() {
        assert_eq!(extent_cover(&[(3, 3)]), vec![vec![(3, 1)]]);
        assert_eq!(
            extent_cover(&[(MAX_BYTE, MAX_BYTE)]),
            vec![vec![(MAX_BYTE, 1)]]
        );
        for p in [1u64, 2, 7, 1 << 31, (1 << 47) + 5] {
            assert_eq!(extent_cover(&[(p, p)]), vec![vec![(p, 1)]]);
        }
    }

    /// A range that already IS one aligned block must come back as one extent.
    #[test]
    fn whole_blocks() {
        for k in 0..=48u32 {
            let size = 1u64 << k;
            let offset = if k >= 47 { 0 } else { 3 * size };
            assert_eq!(
                extent_cover(&[(offset, offset + size - 1)]),
                vec![vec![(offset, size)]],
                "k={}",
                k
            );
        }
    }

    #[test]
    fn which_limit_binds() {
        // Alignment binds: 2 cannot host a 4-byte extent even though 4 fit.
        assert_eq!(extent_cover(&[(2, 5)]), vec![vec![(2, 2), (4, 2)]]);
        assert_eq!(extent_cover(&[(6, 9)]), vec![vec![(6, 2), (8, 2)]]);
        // Fit binds: 8 could host an 8-byte extent but only 2 bytes remain.
        assert_eq!(extent_cover(&[(8, 15)]), vec![vec![(8, 8)]]);
        assert_eq!(extent_cover(&[(7, 8)]), vec![vec![(7, 1), (8, 1)]]);
        // Both bind in turn -- sizes grow, then shrink.
        assert_eq!(
            extent_cover(&[(5, 20)]),
            vec![vec![(5, 1), (6, 2), (8, 8), (16, 4), (20, 1)]]
        );
        assert_eq!(
            extent_cover(&[(1024, 3071)]),
            vec![vec![(1024, 1024), (2048, 1024)]]
        );
    }

    #[test]
    fn edges() {
        assert_eq!(extent_cover(&[]), Vec::<Vec<(u64, u64)>>::new());
        // Several requests in one call, answered independently and in order.
        assert_eq!(
            extent_cover(&[(0, 7), (1, 6), (5, 5)]),
            vec![
                vec![(0, 8)],
                vec![(1, 1), (2, 2), (4, 2), (6, 1)],
                vec![(5, 1)],
            ]
        );
        // Repeated identical requests.
        assert_eq!(
            extent_cover(&[(4, 11); 3]),
            vec![vec![(4, 4), (8, 4)]; 3]
        );
    }

    /// The most fragmented request in the whole address space: 94 extents.
    #[test]
    fn worst_case_shape() {
        let got = extent_cover(&[(1, MAX_BYTE - 1)]).remove(0);
        assert_eq!(got.len(), 94, "expected 94 extents");
        assert_eq!(got[0], (1, 1));
        assert_eq!(got[got.len() - 1], (MAX_BYTE - 1, 1));
        check_contract(1, MAX_BYTE - 1, &got);
        no_mergeable_pair(&got);
    }

    /// Small ranges checked against a shortest-path minimum.
    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x4528_21E6_38D0_1377);
        for _ in 0..400 {
            let start = rng.below(300);
            let end = start + rng.below(120);
            let got = extent_cover(&[(start, end)]).remove(0);
            check_contract(start, end, &got);
            let want = brute_min(start, end);
            assert_eq!(
                got.len(),
                want,
                "range=({}, {}) used {} extents, minimum is {}",
                start,
                end,
                got.len(),
                want
            );
        }
    }

    /// Full-width random ranges: contract plus a necessary minimality check.
    #[test]
    fn random_contract() {
        let mut rng = Lcg::new(0xBE54_66CF_34E9_0C6C);
        for _ in 0..2_000 {
            let a = rng.below(LIMIT);
            let b = rng.below(LIMIT);
            let (start, end) = if a <= b { (a, b) } else { (b, a) };
            let got = extent_cover(&[(start, end)]).remove(0);
            check_contract(start, end, &got);
            no_mergeable_pair(&got);
            assert!(got.len() <= 94);
        }
    }

    /// 5 000 requests spanning the 2^48 space -- fatal to anything per-byte.
    #[test]
    fn large_wide_ranges() {
        let mut rng = Lcg::new(0x243F_6A88_85A3_08D3);
        let mut ranges = Vec::with_capacity(5_000);
        for _ in 0..5_000 {
            let a = rng.below(LIMIT);
            let b = rng.below(LIMIT);
            ranges.push(if a <= b { (a, b) } else { (b, a) });
        }
        let out = extent_cover(&ranges);
        assert_eq!(out.len(), ranges.len());
        let mut total = 0usize;
        for (&(start, end), extents) in ranges.iter().zip(out.iter()) {
            check_contract(start, end, extents);
            no_mergeable_pair(extents);
            total += extents.len();
        }
        assert_eq!(total, 229_651, "total extents {}", total);
    }

    /// 50 000 requests at the stated input bound.
    #[test]
    fn large_batch() {
        let mut rng = Lcg::new(0x1319_8A2E_0370_7344);
        let mut ranges = Vec::with_capacity(50_000);
        for _ in 0..50_000 {
            let start = rng.below(LIMIT);
            let end = (start + rng.below(1024)).min(MAX_BYTE);
            ranges.push((start, end));
        }
        let out = extent_cover(&ranges);
        assert_eq!(out.len(), ranges.len());
        let mut total = 0usize;
        for (&(start, end), extents) in ranges.iter().zip(out.iter()) {
            check_contract(start, end, extents);
            total += extents.len();
        }
        assert_eq!(total, 450_115, "total extents {}", total);
    }
}
