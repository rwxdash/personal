//! Reference solution — Batch Queue Order.
//!
//! Verified against
//! problems/greedy-exchange-argument/01-batch-queue-order/rust.
//! O(n log n) time, O(n) space.

pub fn min_total_cost(duration: &[i64], rate: &[i64]) -> i64 {
    // Swapping two ADJACENT jobs changes nothing outside the pair -- everything
    // before is untouched, and everything after finishes at the same time
    // because the pair occupies the same block either way. Working out that
    // local swap gives the whole ordering rule:
    //
    //   A before B costs an extra rB * dA;  B before A costs an extra rA * dB.
    //   So A goes first exactly when  dA * rB < dB * rA,  i.e. dA/rA < dB/rB.
    //
    // Since no adjacent pair wants to swap once sorted this way, and any order
    // can be reached from any other by adjacent swaps, the sorted order is
    // optimal.
    let mut jobs: Vec<(i64, i64)> = duration.iter().copied().zip(rate.iter().copied()).collect();

    // Cross-multiplication, NOT a floating-point ratio. Both values are at most
    // 10_000, so each product is at most 10^8 and cannot overflow. A float
    // comparator can order two genuinely-equal ratios inconsistently, which is
    // undefined behaviour for sort_by, not merely a wrong answer.
    jobs.sort_by(|a, b| (a.0 * b.1).cmp(&(b.0 * a.1)));

    let mut clock: i64 = 0;
    let mut total: i64 = 0;
    for (d, r) in jobs {
        // Advance the clock FIRST: a job pays for the time up to and including
        // its own run, not up to the moment it started.
        clock += d;
        total += r * clock;
    }

    total
}
