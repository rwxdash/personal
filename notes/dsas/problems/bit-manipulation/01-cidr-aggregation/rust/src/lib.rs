//! CIDR Aggregation
//! problems/bit-manipulation/01-cidr-aggregation
//!
//! Fill in `smallest_cidr`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Smallest CIDR block containing each address range.
///
/// A block with prefix length `p` covers the `2^(32-p)` addresses sharing the
/// same top `p` bits.
///
/// Each input is a `(start, end)` pair with `start <= end`, both in
/// `0 ..= 4_294_967_295`. Up to 200_000 ranges.
///
/// Returns one `(network, prefix_len, exact)` per range: the network address of
/// the smallest containing block, its prefix length (0 to 32), and whether the
/// block covers exactly the requested range.
///
/// Required: O(ranges.len()) time, O(1) space beyond the output.
pub fn smallest_cidr(ranges: &[(u32, u32)]) -> Vec<(u32, u32, bool)> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check derives
// the answer from binary string prefixes, sharing no arithmetic with an
// XOR-and-mask solution.
#[cfg(test)]
mod tests {
    use super::*;

    const MAX_ADDR: u32 = 4_294_967_295;

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

    /// Count common leading bits by comparing 32-character binary strings.
    fn brute(ranges: &[(u32, u32)]) -> Vec<(u32, u32, bool)> {
        ranges
            .iter()
            .map(|&(start, end)| {
                let a = format!("{:032b}", start);
                let b = format!("{:032b}", end);
                let ab = a.as_bytes();
                let bb = b.as_bytes();
                let mut common = 0usize;
                while common < 32 && ab[common] == bb[common] {
                    common += 1;
                }
                let free = 32 - common;
                let network: u32 = if common == 0 {
                    0
                } else {
                    let s: String = a[..common].chars().chain(std::iter::repeat('0').take(free)).collect();
                    u32::from_str_radix(&s, 2).unwrap()
                };
                let size = 1u64 << free;
                let exact = network == start && network as u64 + size - 1 == end as u64;
                (network, common as u32, exact)
            })
            .collect()
    }

    #[test]
    fn examples() {
        assert_eq!(smallest_cidr(&[(0, 255)]), vec![(0, 24, true)]);
        assert_eq!(smallest_cidr(&[(5, 9)]), vec![(0, 28, false)]);
        assert_eq!(smallest_cidr(&[(42, 42)]), vec![(42, 32, true)]);
        assert_eq!(smallest_cidr(&[(0, MAX_ADDR)]), vec![(0, 0, true)]);
        assert_eq!(smallest_cidr(&[(255, 256)]), vec![(0, 23, false)]);
    }

    #[test]
    fn boundary_prefixes() {
        // /32 at both ends of the space.
        assert_eq!(smallest_cidr(&[(0, 0)]), vec![(0, 32, true)]);
        assert_eq!(
            smallest_cidr(&[(MAX_ADDR, MAX_ADDR)]),
            vec![(MAX_ADDR, 32, true)]
        );
        // /0: differing in the top bit forces the widest block.
        assert_eq!(smallest_cidr(&[(0, 1 << 31)]), vec![(0, 0, false)]);
        assert_eq!(
            smallest_cidr(&[((1u32 << 31) - 1, 1 << 31)]),
            vec![(0, 0, false)]
        );
        // /31: two adjacent addresses sharing all but the last bit.
        assert_eq!(smallest_cidr(&[(0, 1)]), vec![(0, 31, true)]);
        assert_eq!(smallest_cidr(&[(2, 3)]), vec![(2, 31, true)]);
        // ...shifted by one, they no longer align.
        assert_eq!(smallest_cidr(&[(1, 2)]), vec![(0, 30, false)]);
        // /1: the halves of the space.
        assert_eq!(
            smallest_cidr(&[(1 << 31, MAX_ADDR)]),
            vec![(1 << 31, 1, true)]
        );
        assert_eq!(
            smallest_cidr(&[(0, (1u32 << 31) - 1)]),
            vec![(0, 1, true)]
        );
    }

    #[test]
    fn exactness() {
        // `exact` needs BOTH ends to line up, not just the start.
        assert_eq!(smallest_cidr(&[(0, 100)]), vec![(0, 25, false)]);
        assert_eq!(smallest_cidr(&[(0, 127)]), vec![(0, 25, true)]);
        assert_eq!(smallest_cidr(&[(0, 126)]), vec![(0, 25, false)]);
        assert_eq!(smallest_cidr(&[(1, 127)]), vec![(0, 25, false)]);
        assert_eq!(smallest_cidr(&[(5, 6)]), vec![(4, 30, false)]);
        assert_eq!(
            smallest_cidr(&[(4_294_967_040, MAX_ADDR)]),
            vec![(4_294_967_040, 24, true)]
        );
    }

    #[test]
    fn edges() {
        // No ranges at all.
        assert_eq!(smallest_cidr(&[]), Vec::<(u32, u32, bool)>::new());
        // Several ranges answered independently.
        assert_eq!(
            smallest_cidr(&[(0, 255), (5, 9), (42, 42)]),
            vec![(0, 24, true), (0, 28, false), (42, 32, true)]
        );
        // Repeated identical ranges.
        assert_eq!(
            smallest_cidr(&[(10, 20); 4]),
            vec![(0, 27, false); 4]
        );
        // Common real-world blocks.
        assert_eq!(
            smallest_cidr(&[(167_772_160, 184_549_375)]),
            vec![(167_772_160, 8, true)]
        );
        assert_eq!(
            smallest_cidr(&[(3_232_235_520, 3_232_301_055)]),
            vec![(3_232_235_520, 16, true)]
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x9159015A3070DD17);
        for _ in 0..2_000 {
            let a = rng.below(1u64 << 32) as u32;
            let b = rng.below(1u64 << 32) as u32;
            let (start, end) = if a <= b { (a, b) } else { (b, a) };
            assert_eq!(
                smallest_cidr(&[(start, end)]),
                brute(&[(start, end)]),
                "range=({}, {})",
                start,
                end
            );
        }
    }

    #[test]
    fn random_aligned_blocks() {
        // Ranges that ARE blocks, so `exact` must be true every time.
        let mut rng = Lcg::new(0x152FECD8F70E5939);
        for _ in 0..2_000 {
            let p = rng.below(33) as u32; // prefix length 0..32
            let free = 32 - p;
            let size = 1u64 << free;
            let blocks = 1u64 << p;
            let idx = rng.below(blocks);
            let network = (idx * size) as u32;
            let start = network;
            let end = (network as u64 + size - 1) as u32;
            let got = smallest_cidr(&[(start, end)]);
            assert_eq!(got, vec![(network, p, true)], "p={} network={}", p, network);
            assert_eq!(got, brute(&[(start, end)]));
        }
    }

    #[test]
    fn random_near_misses() {
        // Aligned blocks shrunk by one, so `exact` must be false.
        let mut rng = Lcg::new(0x67332667FFC00B31);
        for _ in 0..2_000 {
            let p = 1 + rng.below(31) as u32; // 1..31
            let free = 32 - p;
            let size = 1u64 << free;
            let blocks = 1u64 << p;
            let network = (rng.below(blocks) * size) as u32;
            if size < 3 {
                continue;
            }
            let start = network;
            let end = (network as u64 + size - 2) as u32;
            let got = smallest_cidr(&[(start, end)]);
            assert_eq!(got, brute(&[(start, end)]), "p={} network={}", p, network);
            assert!(!got[0].2);
        }
    }

    #[test]
    fn large_batch() {
        // 200k ranges answered in one call, verified against the contract.
        let n = 200_000usize;
        let mut rng = Lcg::new(0x8EB44A8768581511);
        let mut ranges: Vec<(u32, u32)> = Vec::with_capacity(n);
        for _ in 0..n {
            let a = rng.below(1u64 << 32) as u32;
            let b = rng.below(1u64 << 32) as u32;
            ranges.push(if a <= b { (a, b) } else { (b, a) });
        }
        let got = smallest_cidr(&ranges);
        assert_eq!(got.len(), n);
        for (&(start, end), &(network, p, exact)) in ranges.iter().zip(got.iter()) {
            let size = 1u64 << (32 - p);
            assert_eq!(network as u64 % size, 0, "network must be block-aligned");
            assert!(network <= start, "must contain the range");
            assert!(end as u64 <= network as u64 + size - 1, "must contain the range");
            if p < 32 {
                let half = size >> 1;
                let tighter = start as u64 - (start as u64 % half);
                assert!(
                    !(tighter <= start as u64 && end as u64 <= tighter + half - 1),
                    "not the smallest block"
                );
            }
            assert_eq!(
                exact,
                network == start && network as u64 + size - 1 == end as u64
            );
        }
    }

    #[test]
    fn large_full_space() {
        // Many copies of the widest range: the /0 overflow case at scale.
        let n = 100_000usize;
        let ranges = vec![(0u32, MAX_ADDR); n];
        assert_eq!(smallest_cidr(&ranges), vec![(0, 0, true); n]);
    }

    #[test]
    fn large_singletons() {
        // 200k single-address ranges: the /32 case at scale.
        let n = 200_000u32;
        let ranges: Vec<(u32, u32)> = (0..n).map(|i| (i, i)).collect();
        let expected: Vec<(u32, u32, bool)> = (0..n).map(|i| (i, 32, true)).collect();
        assert_eq!(smallest_cidr(&ranges), expected);
    }

    #[test]
    fn large_widest_ranges() {
        // Ranges spanning billions of addresses.
        let ranges = [
            (0u32, MAX_ADDR),
            (1, MAX_ADDR),
            (0, MAX_ADDR - 1),
            (1 << 31, MAX_ADDR),
            ((1u32 << 31) - 1, 1 << 31),
        ];
        assert_eq!(
            smallest_cidr(&ranges),
            vec![
                (0, 0, true),
                (0, 0, false),
                (0, 0, false),
                (1 << 31, 1, true),
                (0, 0, false),
            ]
        );
    }
}
