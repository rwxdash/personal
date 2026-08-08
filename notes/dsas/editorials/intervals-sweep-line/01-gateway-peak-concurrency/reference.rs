//! Reference solution — Gateway Peak Concurrency.
//!
//! Verified against
//! problems/intervals-sweep-line/01-gateway-peak-concurrency/rust.
//! O(n log n) time, O(n) space.

pub fn peak_concurrency(sessions: &[(i64, i64)]) -> (usize, i64) {
    // Concurrency is a step function that changes only at the 2n instants where
    // a session opens or closes, so the 10^9 coordinate range never matters --
    // only the number of sessions does.
    let mut events: Vec<(i64, i8)> = Vec::with_capacity(sessions.len() * 2);
    for &(start, end) in sessions {
        events.push((start, 1));
        events.push((end, -1));
    }

    // Tuples sort lexicographically, so equal timestamps fall through to
    // comparing deltas, and -1 < 1 puts every release ahead of every claim at
    // the same instant. THAT ordering is the half-open semantics: a session
    // ending at t has already let go before anything claims a slot at t.
    // Reverse it and back-to-back sessions wrongly read as overlapping.
    events.sort_unstable();

    let mut current: i64 = 0;
    let mut best: i64 = 0;
    let mut best_time: i64 = 0;

    for &(time, delta) in &events {
        current += delta as i64;
        // Strict `>` keeps the EARLIEST instant at which the peak is reached:
        // events are visited in non-decreasing time order, so a later tie never
        // overwrites the recorded time. `>=` would report the last such instant.
        if current > best {
            best = current;
            best_time = time;
        }
    }

    // Zero-length sessions emit -1 and +1 at the same timestamp, and with
    // releases ordered first the counter dips by one before recovering. That is
    // harmless: a dip can never create a new maximum, and the net change across
    // the timestamp is zero. `current` is signed precisely so that transient
    // dip cannot underflow.
    //
    // When nothing ever overlaps -- empty input, or only zero-length sessions --
    // best stays 0 and best_time stays 0, which is the required (0, 0).
    (best as usize, best_time)
}
