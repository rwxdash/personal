//! Reference solution — Compaction Windows.
//!
//! Verified against
//! problems/binary-search-on-answer/01-compaction-windows/rust.
//! O(n log T) time, O(1) extra space.

pub fn min_window_bytes(sizes: &[i64], checkpoints: &[bool], k: usize) -> i64 {
    // Deciding the optimal arrangement directly is expensive. Deciding whether
    // SOME arrangement fits under a fixed ceiling is a single linear pass -- and
    // that predicate is monotone in the ceiling, so the answer is the position
    // of its single false->true flip.
    let feasible = |cap: i64| -> bool {
        let mut windows = 1usize;
        let mut current: i64 = 0;
        for (i, &s) in sizes.iter().enumerate() {
            if i > 0 && checkpoints[i] {
                // Forced cut. Fires regardless of how much room is left, which
                // is the one thing that distinguishes this from the textbook
                // version of the problem.
                windows += 1;
                current = s;
            } else if current + s > cap {
                windows += 1;
                current = s;
            } else {
                current += s;
            }
            if windows > k {
                return false; // early exit; the count never decreases
            }
        }
        windows <= k
    };

    // lo cannot be beaten: some window must hold the largest segment. Starting
    // here also means `feasible` never sees a segment that fits nowhere.
    let mut lo = *sizes.iter().max().expect("n >= 1");
    let mut hi: i64 = sizes.iter().sum(); // one window holds everything

    while lo < hi {
        // `lo + (hi - lo) / 2` rather than `(lo + hi) / 2`: the latter is safe
        // in i64 at these bounds, but the habit is what keeps it safe when the
        // bounds change.
        let mid = lo + (hi - lo) / 2;
        if feasible(mid) {
            hi = mid; // mid works, so the answer is mid or smaller
        } else {
            lo = mid + 1; // mid fails, so the answer is strictly larger
        }
    }

    lo
}
