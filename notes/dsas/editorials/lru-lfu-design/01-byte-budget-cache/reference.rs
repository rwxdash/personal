//! Reference solution — Byte-Budget Cache.
//!
//! Verified against problems/lru-lfu-design/01-byte-budget-cache/rust.
//! O(ops.len()) time overall, O(ops.len()) space.

use std::collections::HashMap;

/// A linked-list node. `prev`/`next` are INDICES into the node arena rather
/// than pointers -- the standard way to build an intrusive list in Rust, since
/// it sidesteps the borrow checker entirely.
struct Node {
    key: i64,
    value: i64,
    size: i64,
    prev: Option<usize>,
    next: Option<usize>,
}

pub fn cache_simulate(budget: i64, ops: &[(i64, i64, i64, i64)]) -> Vec<i64> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut map: HashMap<i64, usize> = HashMap::new();
    let mut head: Option<usize> = None; // most recently used
    let mut tail: Option<usize> = None; // least recently used
    let mut total: i64 = 0;
    let mut out: Vec<i64> = Vec::new();

    // Detach a node from wherever it currently sits. The four branches are the
    // four positions a node can occupy: middle, front, back, only-element.
    // A single-element list takes both `else` arms and correctly empties the
    // list.
    fn unlink(
        nodes: &mut [Node],
        head: &mut Option<usize>,
        tail: &mut Option<usize>,
        i: usize,
    ) {
        let (p, n) = (nodes[i].prev, nodes[i].next);
        match p {
            Some(pi) => nodes[pi].next = n,
            None => *head = n,
        }
        match n {
            Some(ni) => nodes[ni].prev = p,
            None => *tail = p,
        }
        nodes[i].prev = None;
        nodes[i].next = None;
    }

    fn push_front(
        nodes: &mut [Node],
        head: &mut Option<usize>,
        tail: &mut Option<usize>,
        i: usize,
    ) {
        nodes[i].prev = None;
        nodes[i].next = *head;
        if let Some(h) = *head {
            nodes[h].prev = Some(i);
        }
        *head = Some(i);
        if tail.is_none() {
            *tail = Some(i);
        }
    }

    for &(kind, key, value, size) in ops {
        if kind == 0 {
            match map.get(&key).copied() {
                Some(i) => {
                    unlink(&mut nodes, &mut head, &mut tail, i);
                    push_front(&mut nodes, &mut head, &mut tail, i); // HIT refreshes
                    out.push(nodes[i].value);
                }
                // A MISS changes nothing at all.
                None => out.push(-1),
            }
            continue;
        }

        // Rejected outright: do not evict, do not disturb an existing entry.
        // This check must come before anything else touches the cache.
        if size > budget {
            continue;
        }

        match map.get(&key).copied() {
            Some(i) => {
                // Release the OLD size before adding the new one, or the
                // running total drifts upward and the cache evicts for no
                // reason.
                total -= nodes[i].size;
                nodes[i].value = value;
                nodes[i].size = size;
                unlink(&mut nodes, &mut head, &mut tail, i);
                push_front(&mut nodes, &mut head, &mut tail, i);
            }
            None => {
                let i = nodes.len();
                nodes.push(Node { key, value, size, prev: None, next: None });
                map.insert(key, i);
                push_front(&mut nodes, &mut head, &mut tail, i);
            }
        }
        total += size;

        // while, not if: one large insert can evict several entries. The
        // just-inserted key is never the victim -- it sits at the front, and
        // since size <= budget the loop stops before reaching it.
        while total > budget {
            let victim = tail.expect("total > budget implies a non-empty cache");
            total -= nodes[victim].size;
            map.remove(&nodes[victim].key); // remove from BOTH structures
            unlink(&mut nodes, &mut head, &mut tail, victim);
        }
    }

    out
}
