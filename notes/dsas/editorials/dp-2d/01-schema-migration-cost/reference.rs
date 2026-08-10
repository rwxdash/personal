//! Reference solution — Schema Migration Cost.
//!
//! Verified against problems/dp-2d/01-schema-migration-cost/rust.
//! O(n * m) time, O(min(n, m)) space.

pub fn migration_cost(
    current: &[String],
    target: &[String],
    insert_cost: i64,
    drop_cost: i64,
    retype_cost: i64,
) -> i64 {
    // Keep the SHORTER sequence as the inner loop so the rolling rows are
    // O(min(n, m)). Swapping the sequences means swapping the direction of the
    // migration, so insert and drop must swap with them -- "dropping from A to
    // reach B" is "inserting into B to reach A". Forgetting this is silent: it
    // only shows up when the two costs differ AND current is the shorter side.
    let (long, short, ins, del) = if current.len() < target.len() {
        (target, current, drop_cost, insert_cost)
    } else {
        (current, target, insert_cost, drop_cost)
    };

    let n = long.len();
    let m = short.len();

    // prev[j] = cost to turn the first i columns of `long` into the first j of
    // `short`. Row 0 is "consume nothing, insert j columns".
    let mut prev: Vec<i64> = (0..=m as i64).map(|j| j * ins).collect();
    let mut cur: Vec<i64> = vec![0; m + 1];

    for i in 1..=n {
        // Column 0 of this row: drop all i consumed columns, produce nothing.
        cur[0] = i as i64 * del;
        let left = &long[i - 1];
        for j in 1..=m {
            // Three moves out of this state. The min is taken unconditionally,
            // including when the names match. (Short-circuiting to the free
            // diagonal on a match is also correct -- see the editorial for the
            // two inequalities that prove it -- but this is simpler.)
            let step = if *left == short[j - 1] { 0 } else { retype_cost };
            let a = prev[j] + del; // drop long[i-1]
            let b = cur[j - 1] + ins; // insert short[j-1]
            let c = prev[j - 1] + step; // handle both at once
            cur[j] = a.min(b).min(c);
        }
        std::mem::swap(&mut prev, &mut cur);
    }

    // No special case is needed for "retype costs more than drop + insert":
    // the min finds that route through the grid on its own.
    prev[m]
}
