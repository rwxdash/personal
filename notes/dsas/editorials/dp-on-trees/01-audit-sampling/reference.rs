//! Reference solution — Audit Sampling.
//!
//! Verified against problems/dp-on-trees/01-audit-sampling/rust.
//! O(n) time, O(n) space.

pub fn max_audit_evidence(parent: &[i64], evidence: &[i64]) -> i64 {
    let n = parent.len();

    // Children lists, in one pass.
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    for i in 0..n {
        if parent[i] != -1 {
            children[parent[i] as usize].push(i);
        }
    }

    // Pass 1: an order in which every node appears AFTER its parent. Reversing
    // it then puts every node after all of its children, which is what the
    // recurrence needs.
    //
    // Two flat passes instead of one recursive walk: a 200k-deep chain would
    // overflow the stack, and this avoids any "return to the node after its
    // children" bookkeeping.
    let mut order: Vec<usize> = Vec::with_capacity(n);
    let mut stack: Vec<usize> = vec![0];
    while let Some(v) = stack.pop() {
        order.push(v);
        for i in 0..children[v].len() {
            stack.push(children[v][i]);
        }
    }

    // skip[v] = best total in v's subtree when v is NOT audited
    // take[v] = best total in v's subtree when v IS audited
    let mut skip = vec![0i64; n];
    let mut take = vec![0i64; n];

    // Pass 2: children before parents.
    for idx in (0..order.len()).rev() {
        let v = order[idx];
        let mut total_forced = 0i64; // children must skip, because v is audited
        let mut total_free = 0i64; // children choose freely, because v is skipped
        for i in 0..children[v].len() {
            let c = children[v][i];
            total_forced += skip[c];
            // max, NOT take[c]: skipping v does not oblige a child to be
            // audited, it only removes the restriction.
            total_free += skip[c].max(take[c]);
        }
        skip[v] = total_free;
        take[v] = evidence[v] + total_forced;
    }

    // Leaves need no special case: their sums are empty, so skip = 0 and
    // take = evidence[v].
    //
    // The root may well be better left unaudited, so take the max here too.
    skip[0].max(take[0])
}
