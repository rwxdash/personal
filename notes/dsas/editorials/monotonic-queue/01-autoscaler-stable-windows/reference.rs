//! Reference solution — Autoscaler Stable Windows.
//!
//! Verified against problems/monotonic-queue/01-autoscaler-stable-windows/rust.
//! O(n) time, O(n) space.

use std::collections::VecDeque;

pub fn count_stable_windows(usage: &[i64], delta: i64, min_len: usize) -> u64 {
    // Two monotonic deques of *indices* into `usage`:
    //   max_dq: values non-increasing front-to-back, so front = window maximum
    //   min_dq: values non-decreasing front-to-back, so front = window minimum
    // A plain running max/min cannot work: when `left` advances past the
    // current extreme there is no way to recover the next one. The deques keep
    // the runners-up in the exact order they will be promoted.
    let mut max_dq: VecDeque<usize> = VecDeque::new();
    let mut min_dq: VecDeque<usize> = VecDeque::new();

    let mut total: u64 = 0;
    let mut left: usize = 0;

    for right in 0..usage.len() {
        let value = usage[right];

        // Admit `right`. Any earlier index holding a value <= this one can
        // never be the maximum again -- older and smaller means dominated
        // forever. Mirrored for the minimum.
        while let Some(&back) = max_dq.back() {
            if usage[back] <= value {
                max_dq.pop_back();
            } else {
                break;
            }
        }
        max_dq.push_back(right);
        while let Some(&back) = min_dq.back() {
            if usage[back] >= value {
                min_dq.pop_back();
            } else {
                break;
            }
        }
        min_dq.push_back(right);

        // Restore stability by advancing `left`. This terminates: once
        // left == right the window is a single sample with spread 0 <= delta.
        while usage[max_dq[0]] - usage[min_dq[0]] > delta {
            if max_dq[0] == left {
                max_dq.pop_front();
            }
            if min_dq[0] == left {
                min_dq.pop_front();
            }
            left += 1;
        }

        // `left` is now L(right): the smallest start keeping the range stable.
        // Every start in [left, right] gives a stable range; the length floor
        // trims that to [left, right - min_len + 1]. Counting the interval in
        // one step is what keeps a ~2e10 answer reachable in n steps.
        // Guard the subtraction -- these are usize.
        if right + 1 >= min_len {
            let last_start = right + 1 - min_len;
            if last_start >= left {
                total += (last_start - left + 1) as u64;
            }
        }
    }

    total
}
