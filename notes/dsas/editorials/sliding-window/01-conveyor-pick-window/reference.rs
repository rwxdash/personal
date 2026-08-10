//! Reference solution — Conveyor Pick Window.
//!
//! Verified against problems/sliding-window/01-conveyor-pick-window/rust.
//! O(n) time, O(k) space.

use std::collections::HashMap;

pub fn shortest_fulfilling_run(
    stream: &[String],
    order: &HashMap<String, u32>,
) -> Option<(usize, usize)> {
    // `missing` counts the number of *units* still needed to fill the order.
    // It is the whole trick: it turns "is this window good?" from an O(k) map
    // comparison into an O(1) integer test.
    let mut missing: u64 = order.values().map(|v| *v as u64).sum();
    let mut have: HashMap<&str, u32> = HashMap::new();
    let mut best: Option<(usize, usize)> = None;
    let mut left = 0usize;

    for right in 0..stream.len() {
        let sku = stream[right].as_str();
        if let Some(&need) = order.get(sku) {
            let count = have.entry(sku).or_insert(0);
            // Only a copy still *below* the requirement covers a genuinely
            // missing unit; surplus copies change nothing.
            if *count < need {
                missing -= 1;
            }
            *count += 1;
        }

        // The window is filled; record it, then shrink from the left while it
        // stays filled. Each index leaves at most once, so this is amortised
        // O(1) per step.
        while missing == 0 {
            // Strict `<` keeps the earliest window of any tied length, since
            // windows are recorded in increasing order of `left`.
            let better = match best {
                None => true,
                Some((bl, br)) => right - left < br - bl,
            };
            if better {
                best = Some((left, right));
            }

            let out = stream[left].as_str();
            if let Some(&need_out) = order.get(out) {
                let count = have.get_mut(out).expect("evicted SKU must be counted");
                *count -= 1;
                // Dropping below the requirement re-opens a deficit.
                if *count < need_out {
                    missing += 1;
                }
            }
            left += 1;
        }
    }

    best
}
