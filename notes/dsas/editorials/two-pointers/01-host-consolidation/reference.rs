//! Reference solution — Host Consolidation.
//!
//! Verified against problems/two-pointers/01-host-consolidation/rust.
//! O(n log n) time, O(n) space.

pub fn min_hosts(footprints: &[i64], pinned: &[bool], cap: i64) -> usize {
    // Pinned VMs never interact with anything: one host each, then forget them.
    let mut hosts = 0usize;
    let mut free: Vec<i64> = Vec::with_capacity(footprints.len());
    for (&size, &is_pinned) in footprints.iter().zip(pinned) {
        if is_pinned {
            hosts += 1;
        } else {
            free.push(size);
        }
    }

    free.sort_unstable();

    // Converging two pointers. Minimising hosts == maximising pairs, and the
    // exchange argument says the largest VM should take the SMALLEST partner
    // that fits: if the smallest does not fit, nothing does.
    //
    // Signed indices on purpose: `hi` must be allowed to fall to -1 to end the
    // sweep, which a usize cannot express without extra branching.
    let mut lo: isize = 0;
    let mut hi: isize = free.len() as isize - 1;
    while lo <= hi {
        if free[lo as usize] + free[hi as usize] <= cap {
            lo += 1; // the smallest rides along with the largest
        }
        hi -= 1; // the largest is placed on this host either way
        hosts += 1; // exactly one host consumed per iteration
    }

    // The lo == hi case needs no special handling: the test becomes
    // 2 * free[lo] <= cap, and whichever way it goes the pointers cross with
    // exactly one host counted for that final VM.
    hosts
}
