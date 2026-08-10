//! Reference solution — Mesh Link Failures.
//!
//! Verified against problems/union-find/01-mesh-link-failures/rust.
//! O((n + m) * alpha(n)) time, O(n + m) space.

/// Disjoint-set union with path compression and union by size.
struct Dsu {
    parent: Vec<usize>,
    size: Vec<usize>,
    /// Live partition count. Starts at n and drops only on a real merge.
    count: usize,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Dsu {
            parent: (0..n).collect(),
            size: vec![1; n],
            count: n,
        }
    }

    /// Iterative on purpose: a recursive find can nest 200k deep.
    fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut cur = x;
        while self.parent[cur] != root {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        root
    }

    /// `count` drops only on a real merge. That single condition is what makes
    /// redundant cables and self-loops inert for free: both hit ra == rb.
    fn union(&mut self, a: usize, b: usize) {
        let (mut ra, mut rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        if self.size[ra] < self.size[rb] {
            std::mem::swap(&mut ra, &mut rb); // union by size keeps trees shallow
        }
        self.parent[rb] = ra;
        self.size[ra] += self.size[rb];
        self.count -= 1;
    }
}

pub fn partitions_after_failures(
    n: usize,
    links: &[(usize, usize)],
    failures: &[usize],
) -> Vec<usize> {
    let q = failures.len();
    if q == 0 {
        return Vec::new();
    }

    // Union-find cannot split a set, so deletions are run backwards: the state
    // after ALL failures is built first, then failures are replayed in reverse,
    // which turns every deletion into an insertion.
    let mut doomed = vec![false; links.len()];
    for &idx in failures {
        doomed[idx] = true;
    }

    let mut dsu = Dsu::new(n);

    // Phase 1: the most fragmented state, S_q -- only links that never fail.
    for (i, &(u, v)) in links.iter().enumerate() {
        if !doomed[i] {
            dsu.union(u, v);
        }
    }

    // Phase 2: walk time backwards. After re-adding failures[k], the structure
    // represents S_k (the state after the first k failures), whose count is the
    // answer for step k, stored at answer[k - 1].
    let mut answer = vec![0usize; q];
    answer[q - 1] = dsu.count; // count currently describes S_q
    for k in (1..q).rev() {
        let (u, v) = links[failures[k]];
        dsu.union(u, v);
        answer[k - 1] = dsu.count;
    }

    answer
}
