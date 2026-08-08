//! Sharded Audit Log
//! problems/heaps-k-way-merge/01-sharded-audit-log
//!
//! Fill in `nth_merged_event`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// The event at `offset` in the globally merged view of all shards.
///
/// Each shard is individually sorted non-decreasing. The merged order is by
/// timestamp, and events sharing a timestamp are ordered by ascending shard
/// index — so a shard holding several events at the same timestamp emits all
/// of them before any higher-numbered shard emits one.
///
/// Up to 100_000 shards, 2_000_000 events in total, timestamps `0 ..= 10^18`,
/// `offset` at most 200_000. Shards may be empty; the slice itself may be
/// empty.
///
/// Returns `(timestamp, shard_index)`, or `None` if the shards hold `offset`
/// or fewer events in total.
///
/// Required: O(k + offset * log k) time, O(k) space. The cost must scale with
/// `offset`, not with the total event count.
pub fn nth_merged_event(shards: &[Vec<i64>], offset: usize) -> Option<(i64, usize)> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check
// materialises and sorts the whole merged view — exactly the approach the
// complexity bound forbids — so it is an honest oracle.
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

    fn shardify(rows: &[&[i64]]) -> Vec<Vec<i64>> {
        rows.iter().map(|r| r.to_vec()).collect()
    }

    /// Materialise the entire merged view and index into it. O(N log N).
    fn brute(shards: &[Vec<i64>], offset: usize) -> Option<(i64, usize)> {
        let mut everything: Vec<(i64, usize)> = Vec::new();
        for (i, shard) in shards.iter().enumerate() {
            for &value in shard {
                everything.push((value, i));
            }
        }
        everything.sort();
        everything.get(offset).copied()
    }

    #[test]
    fn examples() {
        assert_eq!(
            nth_merged_event(&shardify(&[&[10, 40], &[20, 30], &[50]]), 2),
            Some((30, 1))
        );
        assert_eq!(
            nth_merged_event(&shardify(&[&[5, 5], &[5], &[5]]), 1),
            Some((5, 0))
        );
        assert_eq!(
            nth_merged_event(&shardify(&[&[], &[7], &[], &[3, 9]]), 0),
            Some((3, 3))
        );
        assert_eq!(nth_merged_event(&shardify(&[&[1], &[2]]), 5), None);
    }

    #[test]
    fn full_traversal() {
        let shards = shardify(&[&[10, 40], &[20, 30], &[50]]);
        assert_eq!(nth_merged_event(&shards, 0), Some((10, 0)));
        assert_eq!(nth_merged_event(&shards, 1), Some((20, 1)));
        assert_eq!(nth_merged_event(&shards, 2), Some((30, 1)));
        assert_eq!(nth_merged_event(&shards, 3), Some((40, 0)));
        assert_eq!(nth_merged_event(&shards, 4), Some((50, 2)));
        assert_eq!(nth_merged_event(&shards, 5), None);
    }

    #[test]
    fn ties() {
        // Ties break by ascending shard index, not round-robin.
        let shards = shardify(&[&[5, 5], &[5], &[5]]);
        assert_eq!(nth_merged_event(&shards, 0), Some((5, 0)));
        assert_eq!(
            nth_merged_event(&shards, 1),
            Some((5, 0)),
            "shard 0 emits BOTH before shard 1"
        );
        assert_eq!(nth_merged_event(&shards, 2), Some((5, 1)));
        assert_eq!(nth_merged_event(&shards, 3), Some((5, 2)));
        assert_eq!(nth_merged_event(&shards, 4), None);
        // Same timestamp everywhere, many shards.
        let flat: Vec<Vec<i64>> = (0..6).map(|_| vec![7i64]).collect();
        for i in 0..6 {
            assert_eq!(nth_merged_event(&flat, i), Some((7, i)));
        }
        // A tie between a shard's later event and another shard's first.
        let mixed = shardify(&[&[1, 9], &[9]]);
        assert_eq!(nth_merged_event(&mixed, 1), Some((9, 0)));
        assert_eq!(nth_merged_event(&mixed, 2), Some((9, 1)));
    }

    #[test]
    fn edges() {
        // No shards at all.
        assert_eq!(nth_merged_event(&[], 0), None);
        assert_eq!(nth_merged_event(&[], 200_000), None);
        // Only empty shards.
        assert_eq!(nth_merged_event(&shardify(&[&[], &[], &[]]), 0), None);
        // One shard holding everything.
        let one = shardify(&[&[1, 2, 3, 4, 5]]);
        assert_eq!(nth_merged_event(&one, 3), Some((4, 0)));
        assert_eq!(nth_merged_event(&one, 4), Some((5, 0)));
        assert_eq!(nth_merged_event(&one, 5), None);
        // Exactly one event.
        assert_eq!(nth_merged_event(&shardify(&[&[42]]), 0), Some((42, 0)));
        assert_eq!(nth_merged_event(&shardify(&[&[42]]), 1), None);
        // Leading empty shards must not shift the indices reported.
        assert_eq!(nth_merged_event(&shardify(&[&[], &[], &[8]]), 0), Some((8, 2)));
        // Fully interleaved.
        let inter = shardify(&[&[1, 3, 5], &[2, 4, 6]]);
        assert_eq!(nth_merged_event(&inter, 0), Some((1, 0)));
        assert_eq!(nth_merged_event(&inter, 1), Some((2, 1)));
        assert_eq!(nth_merged_event(&inter, 5), Some((6, 1)));
        // Disjoint ranges: one shard drains entirely before the other starts.
        let disj = shardify(&[&[100, 200], &[1, 2]]);
        assert_eq!(nth_merged_event(&disj, 1), Some((2, 1)));
        assert_eq!(nth_merged_event(&disj, 2), Some((100, 0)));
        // Timestamp ceiling.
        let big = shardify(&[&[1_000_000_000_000_000_000], &[999_999_999_999_999_999]]);
        assert_eq!(
            nth_merged_event(&big, 0),
            Some((999_999_999_999_999_999, 1))
        );
        assert_eq!(
            nth_merged_event(&big, 1),
            Some((1_000_000_000_000_000_000, 0))
        );
        // Duplicates inside a single shard.
        assert_eq!(nth_merged_event(&shardify(&[&[3, 3, 3]]), 2), Some((3, 0)));
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x1F83D9ABFB41BD6B);
        for _ in 0..400 {
            let k = rng.below(7);
            let mut shards: Vec<Vec<i64>> = Vec::new();
            for _ in 0..k {
                let m = rng.below(5);
                let mut shard: Vec<i64> = (0..m).map(|_| rng.below(8) as i64).collect();
                shard.sort();
                shards.push(shard);
            }
            let total: usize = shards.iter().map(|s| s.len()).sum();
            // Probe every valid offset plus one past the end.
            for offset in 0..total + 2 {
                assert_eq!(
                    nth_merged_event(&shards, offset),
                    brute(&shards, offset),
                    "shards={:?} offset={}",
                    shards,
                    offset
                );
            }
        }
    }

    #[test]
    fn large_many_shards() {
        // 100k shards, shallow paging. Rescanning all heads is 2 * 10^10.
        // Shard i holds i, i + k, i + 2k, so the merged view is 0, 1, 2, ...
        // and position p holds timestamp p from shard p % k.
        let k = 100_000i64;
        let shards: Vec<Vec<i64>> = (0..k).map(|i| vec![i, i + k, i + 2 * k]).collect();
        assert_eq!(nth_merged_event(&shards, 0), Some((0, 0)));
        assert_eq!(nth_merged_event(&shards, 1), Some((1, 1)));
        assert_eq!(nth_merged_event(&shards, 99_999), Some((99_999, 99_999)));
        assert_eq!(nth_merged_event(&shards, 100_000), Some((100_000, 0)));
        assert_eq!(nth_merged_event(&shards, 200_000), Some((200_000, 0)));
    }

    #[test]
    fn large_deep_offset() {
        // Maximum offset against 2M total events.
        let k = 1_000i64;
        let per = 2_000i64;
        let shards: Vec<Vec<i64>> = (0..k)
            .map(|i| (0..per).map(|j| i * per + j).collect())
            .collect();
        let total: usize = shards.iter().map(|s| s.len()).sum();
        assert_eq!(total, 2_000_000);
        assert_eq!(nth_merged_event(&shards, 200_000), Some((200_000, 100)));
        assert_eq!(nth_merged_event(&shards, 0), Some((0, 0)));
        assert_eq!(nth_merged_event(&shards, 1_999), Some((1_999, 0)));
        assert_eq!(nth_merged_event(&shards, 2_000), Some((2_000, 1)));
    }

    #[test]
    fn large_all_ties() {
        // Every event shares one timestamp: the tie rule alone decides order.
        let k = 100_000usize;
        let shards: Vec<Vec<i64>> = (0..k).map(|_| vec![99i64]).collect();
        assert_eq!(nth_merged_event(&shards, 0), Some((99, 0)));
        assert_eq!(nth_merged_event(&shards, 1), Some((99, 1)));
        assert_eq!(nth_merged_event(&shards, 50_000), Some((99, 50_000)));
        assert_eq!(nth_merged_event(&shards, k - 1), Some((99, k - 1)));
        assert_eq!(nth_merged_event(&shards, k), None);
    }

    #[test]
    fn large_one_fat_shard() {
        // A single shard holding 2M events, plus many empty ones.
        let fat: Vec<i64> = (0..4_000_000i64).step_by(2).collect();
        let mut shards: Vec<Vec<i64>> = (0..50_000).map(|_| Vec::new()).collect();
        shards.push(fat);
        assert_eq!(nth_merged_event(&shards, 0), Some((0, 50_000)));
        assert_eq!(nth_merged_event(&shards, 123_456), Some((246_912, 50_000)));
        assert_eq!(nth_merged_event(&shards, 200_000), Some((400_000, 50_000)));
    }

    #[test]
    fn large_offset_past_end() {
        // Sparse data with a deep offset: must return None.
        let shards: Vec<Vec<i64>> = (0..1_000i64).map(|i| vec![i]).collect();
        assert_eq!(nth_merged_event(&shards, 999), Some((999, 999)));
        assert_eq!(nth_merged_event(&shards, 1_000), None);
        assert_eq!(nth_merged_event(&shards, 200_000), None);
    }
}
