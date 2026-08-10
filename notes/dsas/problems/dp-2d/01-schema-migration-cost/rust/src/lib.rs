//! Schema Migration Cost
//! problems/dp-2d/01-schema-migration-cost
//!
//! Fill in `migration_cost`, then run `cargo test` in this directory.
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Cheapest way to turn the `current` column layout into `target`.
///
/// Column order matters and columns cannot be reordered. Three operations are
/// available: insert a column, drop a column, or retype one column into
/// another. A column already matching the target at that position is free.
///
/// The three costs are independent — retyping may be more expensive than
/// dropping and re-inserting, or much cheaper.
///
/// The result can reach about 4 * 10^9, hence `i64`.
///
/// Required: O(n * m) time, O(min(n, m)) space — a full n x m table is NOT
/// acceptable.
pub fn migration_cost(
    current: &[String],
    target: &[String],
    insert_cost: i64,
    drop_cost: i64,
    retype_cost: i64,
) -> i64 {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check
// explores every operation sequence recursively without memoisation.
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

    fn cols(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    /// Explore every operation sequence. Exponential; only for tiny input.
    fn brute(
        current: &[String],
        target: &[String],
        ins: i64,
        del: i64,
        rep: i64,
    ) -> i64 {
        fn go(
            cur: &[String],
            tgt: &[String],
            i: usize,
            j: usize,
            ins: i64,
            del: i64,
            rep: i64,
        ) -> i64 {
            if i == cur.len() {
                return (tgt.len() - j) as i64 * ins;
            }
            if j == tgt.len() {
                return (cur.len() - i) as i64 * del;
            }
            let step = if cur[i] == tgt[j] { 0 } else { rep };
            let a = go(cur, tgt, i + 1, j, ins, del, rep) + del;
            let b = go(cur, tgt, i, j + 1, ins, del, rep) + ins;
            let c = go(cur, tgt, i + 1, j + 1, ins, del, rep) + step;
            a.min(b).min(c)
        }
        go(current, target, 0, 0, ins, del, rep)
    }

    #[test]
    fn examples() {
        assert_eq!(
            migration_cost(&cols(&["id", "name", "email"]), &cols(&["id", "email"]), 1, 1, 1),
            1
        );
        assert_eq!(migration_cost(&cols(&["a"]), &cols(&["b"]), 1, 1, 10), 2);
        assert_eq!(
            migration_cost(&cols(&["a", "b"]), &cols(&["x", "y"]), 5, 5, 1),
            2
        );
        assert_eq!(migration_cost(&cols(&["a", "b", "c"]), &[], 7, 2, 3), 6);
        assert_eq!(migration_cost(&cols(&["a", "b"]), &cols(&["b"]), 1, 1, 1), 1);
    }

    #[test]
    fn cost_asymmetry() {
        // Retype beats drop + insert.
        assert_eq!(migration_cost(&cols(&["a"]), &cols(&["b"]), 5, 5, 1), 1);
        // Drop + insert beats retype.
        assert_eq!(migration_cost(&cols(&["a"]), &cols(&["b"]), 1, 1, 10), 2);
        // Exactly equal: either route.
        assert_eq!(migration_cost(&cols(&["a"]), &cols(&["b"]), 1, 1, 2), 2);
        // Free inserts / free drops.
        assert_eq!(migration_cost(&[], &cols(&["a", "b", "c"]), 0, 5, 5), 0);
        assert_eq!(migration_cost(&cols(&["a", "b", "c"]), &[], 5, 0, 5), 0);
        // All costs zero.
        assert_eq!(
            migration_cost(&cols(&["a", "b"]), &cols(&["c", "d"]), 0, 0, 0),
            0
        );
        // Asymmetric: dropping is dear, inserting is cheap.
        assert_eq!(
            migration_cost(&cols(&["a", "b", "c"]), &cols(&["a"]), 1, 100, 1),
            200
        );
        assert_eq!(
            migration_cost(&cols(&["a"]), &cols(&["a", "b", "c"]), 1, 100, 1),
            2
        );
    }

    #[test]
    fn edges() {
        // Both empty.
        assert_eq!(migration_cost(&[], &[], 3, 4, 5), 0);
        // Identical layouts cost nothing.
        assert_eq!(
            migration_cost(&cols(&["a", "b", "c"]), &cols(&["a", "b", "c"]), 9, 9, 9),
            0
        );
        // One empty side.
        assert_eq!(migration_cost(&[], &cols(&["a", "b"]), 3, 4, 5), 6);
        assert_eq!(migration_cost(&cols(&["a", "b"]), &[], 3, 4, 5), 8);
        // Single column.
        assert_eq!(migration_cost(&cols(&["a"]), &cols(&["a"]), 1, 1, 1), 0);
        assert_eq!(migration_cost(&cols(&["a"]), &cols(&["b"]), 1, 1, 1), 1);
        // Repeated names within a layout.
        assert_eq!(
            migration_cost(&cols(&["a", "a", "a"]), &cols(&["a"]), 1, 1, 1),
            2
        );
        assert_eq!(
            migration_cost(&cols(&["a"]), &cols(&["a", "a", "a"]), 1, 1, 1),
            2
        );
        assert_eq!(
            migration_cost(&cols(&["a", "a"]), &cols(&["a", "a"]), 5, 5, 5),
            0
        );
        // Reordering is not allowed, so a swap costs real work.
        assert_eq!(
            migration_cost(&cols(&["a", "b"]), &cols(&["b", "a"]), 1, 1, 1),
            2
        );
        // Prefix and suffix matches.
        assert_eq!(
            migration_cost(&cols(&["x", "a", "b"]), &cols(&["a", "b"]), 1, 1, 1),
            1
        );
        assert_eq!(
            migration_cost(&cols(&["a", "b", "x"]), &cols(&["a", "b"]), 1, 1, 1),
            1
        );
        // Cost ceiling.
        assert_eq!(
            migration_cost(&cols(&["a"]), &cols(&["b"]), 1_000_000, 1_000_000, 1_000_000),
            1_000_000
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0xE49B69C19EF14AD2);
        let alphabet = ["a", "b", "c"];
        for _ in 0..300 {
            let n = rng.below(7);
            let m = rng.below(7);
            let current: Vec<String> = (0..n)
                .map(|_| alphabet[rng.below(3) as usize].to_string())
                .collect();
            let target: Vec<String> = (0..m)
                .map(|_| alphabet[rng.below(3) as usize].to_string())
                .collect();
            let ins = rng.below(8) as i64;
            let del = rng.below(8) as i64;
            let rep = rng.below(8) as i64;
            assert_eq!(
                migration_cost(&current, &target, ins, del, rep),
                brute(&current, &target, ins, del, rep),
                "current={:?} target={:?} costs=({},{},{})",
                current,
                target,
                ins,
                del,
                rep
            );
        }
    }

    #[test]
    fn random_lopsided() {
        // One side much longer than the other, in both directions. Catches a
        // rolled-up table that swaps the sequences without swapping the costs.
        let mut rng = Lcg::new(0xEFBE4786384F25E3);
        let alphabet = ["p", "q"];
        for _ in 0..300 {
            let long_n = 1 + rng.below(9);
            let short_n = rng.below(3);
            let a: Vec<String> = (0..long_n)
                .map(|_| alphabet[rng.below(2) as usize].to_string())
                .collect();
            let b: Vec<String> = (0..short_n)
                .map(|_| alphabet[rng.below(2) as usize].to_string())
                .collect();
            let ins = (1 + rng.below(9)) as i64;
            let del = (1 + rng.below(9)) as i64;
            let rep = (1 + rng.below(9)) as i64;
            assert_eq!(
                migration_cost(&a, &b, ins, del, rep),
                brute(&a, &b, ins, del, rep)
            );
            assert_eq!(
                migration_cost(&b, &a, ins, del, rep),
                brute(&b, &a, ins, del, rep)
            );
        }
    }

    #[test]
    fn large_identical() {
        // 2000 identical columns: nothing to do.
        let c: Vec<String> = (0..2_000).map(|i| format!("col{}", i)).collect();
        assert_eq!(
            migration_cost(&c, &c.clone(), 1_000_000, 1_000_000, 1_000_000),
            0
        );
    }

    #[test]
    fn large_all_different() {
        // 2000 columns, none matching.
        let a: Vec<String> = (0..2_000).map(|i| format!("a{}", i)).collect();
        let b: Vec<String> = (0..2_000).map(|i| format!("b{}", i)).collect();
        assert_eq!(migration_cost(&a, &b, 10, 10, 3), 2_000 * 3);
        assert_eq!(migration_cost(&a, &b, 1, 1, 10), 2_000 * 2);
        // 2 * 10^9, past a 32-bit signed int.
        assert_eq!(
            migration_cost(&a, &b, 1_000_000, 1_000_000, 1_000_000),
            2_000 * 1_000_000
        );
    }

    #[test]
    fn large_one_empty() {
        let c: Vec<String> = (0..2_000).map(|i| format!("c{}", i)).collect();
        assert_eq!(migration_cost(&c, &[], 7, 1_000_000, 3), 2_000 * 1_000_000);
        assert_eq!(migration_cost(&[], &c, 1_000_000, 7, 3), 2_000 * 1_000_000);
    }

    #[test]
    fn large_lopsided() {
        // A long layout against a short one: exercises the space-saving swap.
        let long_side: Vec<String> = (0..2_000).map(|i| format!("x{}", i)).collect();
        let short_side: Vec<String> = (0..5).map(|i| format!("x{}", i)).collect();
        assert_eq!(migration_cost(&long_side, &short_side, 3, 7, 11), 1_995 * 7);
        // Reversed: costs stay attached to their own direction.
        assert_eq!(migration_cost(&short_side, &long_side, 3, 7, 11), 1_995 * 3);
    }

    #[test]
    fn large_interleaved() {
        // Every other column matches.
        let n = 2_000;
        let a: Vec<String> = (0..n)
            .map(|i| {
                if i % 2 == 0 {
                    format!("same{}", i)
                } else {
                    format!("old{}", i)
                }
            })
            .collect();
        let b: Vec<String> = (0..n)
            .map(|i| {
                if i % 2 == 0 {
                    format!("same{}", i)
                } else {
                    format!("new{}", i)
                }
            })
            .collect();
        assert_eq!(migration_cost(&a, &b, 50, 50, 2), 1_000 * 2);
    }

    #[test]
    fn large_prefix_shift() {
        // The target is the current layout shifted by one position.
        let n = 2_000;
        let a: Vec<String> = (0..n).map(|i| format!("c{}", i)).collect();
        let b: Vec<String> = (1..n + 1).map(|i| format!("c{}", i)).collect();
        assert_eq!(migration_cost(&a, &b, 1, 1, 1), 2);
        assert_eq!(migration_cost(&a, &b, 1, 1, 5), 2);
    }
}
