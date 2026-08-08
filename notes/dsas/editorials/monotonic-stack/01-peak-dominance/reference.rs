//! Reference solution — Peak Dominance.
//!
//! Verified against problems/monotonic-stack/01-peak-dominance/rust.
//! O(n) time, O(n) space.

pub fn dominance_spans(load: &[i64]) -> Vec<usize> {
    let n = load.len();

    // left[i]  = nearest STRICTLY greater index to the left, or -1
    // right[i] = nearest STRICTLY greater index to the right, or n
    // Signed, so the -1 sentinel is expressible and the final subtraction is
    // one expression rather than a branch.
    let mut left = vec![-1isize; n];
    let mut right = vec![n as isize; n];

    // Pass 1, left to right. The stack holds indices whose values are strictly
    // decreasing from bottom to top.
    //
    // The pop condition is `<=`, not `<`. An element that is older AND no
    // taller than the current one is shadowed forever: if it would ever qualify
    // as the "strictly greater" neighbour for some future index j, the current
    // index qualifies too and is nearer. Using `<` leaves equal values on the
    // stack, they get reported as boundaries, and every plateau collapses to 1.
    let mut stack: Vec<usize> = Vec::new();
    for i in 0..n {
        while let Some(&top) = stack.last() {
            if load[top] <= load[i] {
                stack.pop();
            } else {
                break;
            }
        }
        left[i] = stack.last().map_or(-1, |&t| t as isize);
        stack.push(i);
    }

    // Pass 2, right to left. Identical logic, mirrored.
    stack.clear();
    for i in (0..n).rev() {
        while let Some(&top) = stack.last() {
            if load[top] <= load[i] {
                stack.pop();
            } else {
                break;
            }
        }
        right[i] = stack.last().map_or(n as isize, |&t| t as isize);
        stack.push(i);
    }

    // Samples strictly between the two blocking indices. For the global maximum
    // this is n - (-1) - 1 = n, as it should be.
    //
    // Each index is pushed once and popped at most once per pass, so the inner
    // `while` is amortised: both passes are O(n), not O(n^2).
    (0..n)
        .map(|i| (right[i] - left[i] - 1) as usize)
        .collect()
}
