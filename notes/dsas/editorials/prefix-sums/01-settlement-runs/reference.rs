//! Reference solution — Settlement Runs.
//!
//! Verified against problems/prefix-sums/01-settlement-runs/rust.
//! O(n) time, O(min(n, m)) space.

use std::collections::HashMap;

pub fn settlement_runs(deltas: &[i64], m: i64) -> (u64, usize) {
    // A run covering positions i..j-1 sums to P[j] - P[i], where P is the array
    // of running totals. That difference is divisible by m exactly when P[i] and
    // P[j] share a residue mod m -- so the problem is "count pairs of equal
    // residues", never "examine runs".
    let mut seen_count: HashMap<i64, u64> = HashMap::new();
    let mut first_index: HashMap<i64, usize> = HashMap::new();
    seen_count.insert(0, 1); // the empty prefix, P[0] = 0
    first_index.insert(0, 0);

    let mut total: i64 = 0;
    let mut count: u64 = 0;
    let mut longest: usize = 0;

    for j in 1..=deltas.len() {
        total += deltas[j - 1];

        // rem_euclid, NOT `%`. Rust's `%` keeps the sign of the dividend, so
        // -1 % 3 == -1, which would split one residue class across two buckets
        // and silently lose every run spanning a negative running total.
        let r = total.rem_euclid(m);

        // Every earlier prefix sharing this residue closes one settleable run
        // ending here. Reading the tally BEFORE incrementing is what stops a
        // prefix being paired with itself.
        let slot = seen_count.entry(r).or_insert(0);
        count += *slot;
        *slot += 1;

        // Only the FIRST occurrence of a residue can start the longest run, so
        // never overwrite it.
        match first_index.get(&r) {
            Some(&start) => {
                let span = j - start;
                if span > longest {
                    longest = span;
                }
            }
            None => {
                first_index.insert(r, j);
            }
        }
    }

    (count, longest)
}
