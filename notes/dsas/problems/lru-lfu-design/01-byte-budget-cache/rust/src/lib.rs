//! Byte-Budget Cache
//! problems/lru-lfu-design/01-byte-budget-cache
//!
//! Fill in `cache_simulate`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Run a byte-budgeted LRU cache over a sequence of operations.
///
/// Each op is `(kind, key, value, size)`:
///
/// * `kind == 0` — GET `key`. Returns the stored value, or `-1` on a miss. A
///   HIT marks the key most recently used; a MISS changes nothing.
/// * `kind == 1` — PUT `key`, `value`, `size`. If `size > budget` the op is
///   rejected and the cache is left exactly as it was, including any existing
///   entry under that key. Otherwise the key is stored with the new value and
///   size and becomes most recently used, releasing its old size if it was
///   already present. Then, while the total size exceeds the budget, the least
///   recently used entry is evicted.
///
/// `budget` is `0 ..= 10^12` (a budget of 0 rejects every PUT). Up to 200_000
/// operations; `key` and `value` are `0 ..= 10^9`, `size` is `1 ..= 10^9`.
///
/// Returns one entry per GET, in order.
///
/// Required: O(ops.len()) time overall (amortised O(1) per op),
/// O(ops.len()) space.
pub fn cache_simulate(budget: i64, ops: &[(i64, i64, i64, i64)]) -> Vec<i64> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check runs a
// deliberately naive cache that keeps entries in a Vec and scans for the least
// recently used one.
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

    /// Vec-based cache, least recent at index 0. O(ops.len()^2).
    fn brute(budget: i64, ops: &[(i64, i64, i64, i64)]) -> Vec<i64> {
        let mut entries: Vec<(i64, i64, i64)> = Vec::new(); // (key, value, size)
        let mut total: i64 = 0;
        let mut out: Vec<i64> = Vec::new();

        for &(kind, key, value, size) in ops {
            let idx = entries.iter().position(|e| e.0 == key);
            if kind == 0 {
                match idx {
                    None => out.push(-1),
                    Some(i) => {
                        let e = entries.remove(i);
                        out.push(e.1);
                        entries.push(e); // most recent goes last
                    }
                }
            } else {
                if size > budget {
                    continue;
                }
                if let Some(i) = idx {
                    total -= entries[i].2;
                    entries.remove(i);
                }
                entries.push((key, value, size));
                total += size;
                while total > budget {
                    let victim = entries.remove(0);
                    total -= victim.2;
                }
            }
        }
        out
    }

    #[test]
    fn examples() {
        assert_eq!(
            cache_simulate(
                10,
                &[(1, 1, 100, 6), (1, 2, 200, 3), (0, 1, 0, 0), (1, 3, 300, 4), (0, 2, 0, 0), (0, 1, 0, 0)]
            ),
            vec![100, -1, 100]
        );
        assert_eq!(
            cache_simulate(
                10,
                &[(1, 1, 10, 3), (1, 2, 20, 3), (1, 3, 30, 3), (1, 4, 40, 9),
                  (0, 1, 0, 0), (0, 2, 0, 0), (0, 3, 0, 0), (0, 4, 0, 0)]
            ),
            vec![-1, -1, -1, 40]
        );
        assert_eq!(
            cache_simulate(10, &[(1, 1, 10, 8), (1, 2, 20, 11), (0, 1, 0, 0), (0, 2, 0, 0)]),
            vec![10, -1]
        );
        assert_eq!(
            cache_simulate(
                10,
                &[(1, 1, 10, 8), (1, 2, 20, 2), (1, 1, 99, 3), (0, 1, 0, 0), (0, 2, 0, 0)]
            ),
            vec![99, 20]
        );
        assert_eq!(
            cache_simulate(
                6,
                &[(1, 1, 10, 3), (1, 2, 20, 3), (0, 9, 0, 0), (1, 3, 30, 3), (0, 1, 0, 0)]
            ),
            vec![-1, -1]
        );
    }

    #[test]
    fn recency_rules() {
        // Without the GET, key 1 would be evicted. With it, key 2 goes instead.
        assert_eq!(
            cache_simulate(6, &[(1, 1, 10, 3), (1, 2, 20, 3), (1, 3, 30, 3), (0, 1, 0, 0)]),
            vec![-1]
        );
        assert_eq!(
            cache_simulate(
                6,
                &[(1, 1, 10, 3), (1, 2, 20, 3), (0, 1, 0, 0), (1, 3, 30, 3), (0, 1, 0, 0)]
            ),
            vec![10, 10]
        );
        // A PUT also refreshes recency.
        assert_eq!(
            cache_simulate(
                6,
                &[(1, 1, 10, 3), (1, 2, 20, 3), (1, 1, 11, 3), (1, 3, 30, 3), (0, 1, 0, 0), (0, 2, 0, 0)]
            ),
            vec![11, -1]
        );
        // A missing GET must not create an entry or change order.
        assert_eq!(
            cache_simulate(
                6,
                &[(1, 1, 10, 3), (0, 42, 0, 0), (0, 42, 0, 0), (1, 2, 20, 3), (0, 1, 0, 0)]
            ),
            vec![-1, -1, 10]
        );
    }

    #[test]
    fn rejection() {
        // An oversized PUT is rejected and evicts nothing.
        assert_eq!(
            cache_simulate(5, &[(1, 1, 10, 5), (1, 2, 20, 6), (0, 1, 0, 0), (0, 2, 0, 0)]),
            vec![10, -1]
        );
        // Rejection does not even disturb recency.
        assert_eq!(
            cache_simulate(
                6,
                &[(1, 1, 10, 3), (1, 2, 20, 3), (1, 9, 90, 100), (1, 3, 30, 3), (0, 1, 0, 0), (0, 2, 0, 0)]
            ),
            vec![-1, 20]
        );
        // An oversized PUT on an EXISTING key leaves the old entry in place.
        assert_eq!(
            cache_simulate(5, &[(1, 1, 10, 4), (1, 1, 99, 9), (0, 1, 0, 0)]),
            vec![10]
        );
        // Budget 0 rejects everything.
        assert_eq!(cache_simulate(0, &[(1, 1, 10, 1), (0, 1, 0, 0)]), vec![-1]);
    }

    #[test]
    fn sizes() {
        // Exactly full, then an overwrite that shrinks: nothing evicted.
        assert_eq!(
            cache_simulate(
                10,
                &[(1, 1, 10, 8), (1, 2, 20, 2), (1, 1, 11, 3), (0, 1, 0, 0), (0, 2, 0, 0)]
            ),
            vec![11, 20]
        );
        // Overwrite that grows: the other entry is evicted.
        assert_eq!(
            cache_simulate(
                10,
                &[(1, 1, 10, 2), (1, 2, 20, 2), (1, 1, 11, 10), (0, 1, 0, 0), (0, 2, 0, 0)]
            ),
            vec![11, -1]
        );
        // An entry exactly equal to the budget evicts everything else.
        assert_eq!(
            cache_simulate(
                10,
                &[(1, 1, 10, 4), (1, 2, 20, 4), (1, 3, 30, 10), (0, 1, 0, 0), (0, 2, 0, 0), (0, 3, 0, 0)]
            ),
            vec![-1, -1, 30]
        );
        // Filling exactly to the budget evicts nothing.
        assert_eq!(
            cache_simulate(10, &[(1, 1, 10, 5), (1, 2, 20, 5), (0, 1, 0, 0), (0, 2, 0, 0)]),
            vec![10, 20]
        );
    }

    #[test]
    fn edges() {
        // No operations.
        assert_eq!(cache_simulate(100, &[]), Vec::<i64>::new());
        // Only PUTs: no output.
        assert_eq!(
            cache_simulate(100, &[(1, 1, 10, 5), (1, 2, 20, 5)]),
            Vec::<i64>::new()
        );
        // GET on an empty cache.
        assert_eq!(cache_simulate(100, &[(0, 1, 0, 0)]), vec![-1]);
        // Same key put repeatedly.
        assert_eq!(
            cache_simulate(10, &[(1, 1, 10, 5), (1, 1, 20, 5), (1, 1, 30, 5), (0, 1, 0, 0)]),
            vec![30]
        );
        // Value 0 is a real value, distinct from the -1 miss marker.
        assert_eq!(cache_simulate(10, &[(1, 1, 0, 5), (0, 1, 0, 0)]), vec![0]);
        // Key 0 is a real key.
        assert_eq!(cache_simulate(10, &[(1, 0, 77, 5), (0, 0, 0, 0)]), vec![77]);
        // Large budget: nothing is ever evicted.
        assert_eq!(
            cache_simulate(
                1_000_000_000_000,
                &[(1, 1, 10, 1_000_000_000), (1, 2, 20, 1_000_000_000), (0, 1, 0, 0), (0, 2, 0, 0)]
            ),
            vec![10, 20]
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0xA2BFE8A14CF10364);
        for _ in 0..400 {
            let budget = rng.below(20) as i64;
            let n = rng.below(30);
            let mut ops: Vec<(i64, i64, i64, i64)> = Vec::new();
            for _ in 0..n {
                if rng.below(2) == 0 {
                    ops.push((0, rng.below(6) as i64, 0, 0));
                } else {
                    ops.push((
                        1,
                        rng.below(6) as i64,
                        rng.below(100) as i64,
                        1 + rng.below(9) as i64,
                    ));
                }
            }
            assert_eq!(
                cache_simulate(budget, &ops),
                brute(budget, &ops),
                "budget={} ops={:?}",
                budget,
                ops
            );
        }
    }

    #[test]
    fn random_tight_budget() {
        // Budgets small enough that almost every PUT evicts something.
        let mut rng = Lcg::new(0xC24B8B70D0F89791);
        for _ in 0..400 {
            let budget = (1 + rng.below(6)) as i64;
            let n = rng.below(25);
            let mut ops: Vec<(i64, i64, i64, i64)> = Vec::new();
            for _ in 0..n {
                if rng.below(3) == 0 {
                    ops.push((0, rng.below(4) as i64, 0, 0));
                } else {
                    ops.push((
                        1,
                        rng.below(4) as i64,
                        rng.below(50) as i64,
                        1 + rng.below(8) as i64,
                    ));
                }
            }
            assert_eq!(cache_simulate(budget, &ops), brute(budget, &ops));
        }
    }

    #[test]
    fn large_hot_key() {
        // One key read constantly must never be evicted.
        let n = 200_000usize;
        let mut ops: Vec<(i64, i64, i64, i64)> = vec![(1, 0, 999, 5)];
        for i in 1..(n / 2) {
            ops.push((1, i as i64, i as i64, 5));
            ops.push((0, 0, 0, 0)); // keep key 0 hot
        }
        ops.push((0, 0, 0, 0));
        let got = cache_simulate(20, &ops);
        assert!(
            got.iter().all(|&v| v == 999),
            "the hot key must survive every eviction"
        );
        assert_eq!(got.len(), n / 2);
    }

    #[test]
    fn large_thrash() {
        // Every insert evicts: the cache holds one entry at a time.
        let n = 100_000i64;
        let mut ops: Vec<(i64, i64, i64, i64)> = Vec::new();
        for i in 0..n {
            ops.push((1, i, i * 2, 10));
        }
        for i in 0..n {
            ops.push((0, i, 0, 0));
        }
        let mut expected = vec![-1i64; (n - 1) as usize];
        expected.push((n - 1) * 2);
        assert_eq!(cache_simulate(10, &ops), expected);
    }

    #[test]
    fn large_no_eviction() {
        // A budget big enough for everything: pure map behaviour.
        let n = 100_000i64;
        let mut ops: Vec<(i64, i64, i64, i64)> = (0..n).map(|i| (1, i, i + 7, 1)).collect();
        ops.extend((0..n).map(|i| (0, i, 0, 0)));
        let expected: Vec<i64> = (0..n).map(|i| i + 7).collect();
        assert_eq!(cache_simulate(1_000_000_000_000, &ops), expected);
    }

    #[test]
    fn large_all_rejected() {
        // Every PUT after the first is oversized and must evict nothing.
        let n = 100_000i64;
        let mut ops: Vec<(i64, i64, i64, i64)> = vec![(1, 0, 1, 5)];
        ops.extend((1..n).map(|i| (1, i, i, 100)));
        ops.push((0, 0, 0, 0));
        assert_eq!(cache_simulate(10, &ops), vec![1]);
    }

    #[test]
    fn large_overwrite_same_key() {
        // 200k overwrites of one key: the total must not drift upward.
        let n = 200_000i64;
        let mut ops: Vec<(i64, i64, i64, i64)> = (0..n).map(|i| (1, 7, i, 5)).collect();
        ops.push((0, 7, 0, 0));
        assert_eq!(cache_simulate(10, &ops), vec![n - 1]);
    }

    #[test]
    fn large_byte_range() {
        // Sizes near the value ceiling, forcing 64-bit totals.
        let ops = [
            (1, 1, 10, 1_000_000_000),
            (1, 2, 20, 1_000_000_000),
            (1, 3, 30, 1_000_000_000),
            (0, 1, 0, 0),
            (0, 2, 0, 0),
            (0, 3, 0, 0),
        ];
        assert_eq!(cache_simulate(2_000_000_000, &ops), vec![-1, 20, 30]);
    }
}
