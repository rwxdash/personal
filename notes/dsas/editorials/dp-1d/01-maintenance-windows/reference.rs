//! Reference solution — Maintenance Windows.
//!
//! Verified against problems/dp-1d/01-maintenance-windows/rust.
//! O(n) time, O(n) space.

pub fn max_maintenance_value(value: &[i64], blackout: &[bool], cooldown: usize) -> i64 {
    let n = value.len();
    if n == 0 {
        return 0;
    }

    // best[i] = maximum total value using only slots 0..=i.
    let mut best = vec![0i64; n];

    for i in 0..n {
        // Option 1: skip slot i.
        let skip = if i > 0 { best[i - 1] } else { 0 };

        // Option 2: take slot i, if allowed. Taking it collects value[i] and
        // forces everything else back to slot i - cooldown - 1 or earlier. We
        // do NOT need to know which earlier slots were used -- only the best
        // total achievable by that cut-off, which is one array lookup.
        //
        // Compare in signed arithmetic: `i - cooldown - 1` underflows a usize
        // for small i, and "before the array" must mean a base of 0, not a
        // panic or a wrapped index.
        let mut take = 0i64;
        if !blackout[i] {
            let j = i as i64 - cooldown as i64 - 1;
            take = value[i] + if j >= 0 { best[j as usize] } else { 0 };
        }

        best[i] = skip.max(take);
    }

    // `best` is non-decreasing (skipping is always allowed), which is why the
    // single lookup at j is already the maximum over everything at or before j
    // -- no backward scan needed, so this stays O(n) rather than O(n^2).
    best[n - 1]
}
