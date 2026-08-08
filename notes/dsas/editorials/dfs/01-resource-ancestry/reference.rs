//! Reference solution — Resource Ancestry.
//!
//! Verified against problems/dfs/01-resource-ancestry/rust.
//! O(n + q) time, O(n) space.

pub fn ancestor_queries(parent: &[i64], queries: &[(usize, usize)]) -> Vec<bool> {
    let n = parent.len();

    // Children lists and roots, in one O(n) pass.
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut roots: Vec<usize> = Vec::new();
    for (i, &p) in parent.iter().enumerate() {
        if p == -1 {
            roots.push(i);
        } else {
            children[p as usize].push(i);
        }
    }

    // A node's subtree is CONTIGUOUS in depth-first visit order: once you step
    // into u you visit all of u's subtree, and nothing else, before stepping
    // back out. Stamping an entry and an exit time therefore turns each subtree
    // into an interval, and ancestry into interval containment.
    let mut tin = vec![0usize; n];
    let mut tout = vec![0usize; n];
    let mut timer = 0usize;

    for &r in &roots {
        // ITERATIVE on purpose. Recursion overflows the stack on a 200k-deep
        // chain, and this problem explicitly allows one. The bool is the
        // "returning to this node after its children" flag.
        let mut stack: Vec<(usize, bool)> = vec![(r, false)];
        while let Some((node, returning)) = stack.pop() {
            if returning {
                tout[node] = timer;
                timer += 1;
                continue;
            }
            tin[node] = timer;
            timer += 1;
            // Push the exit marker BEFORE the children so it pops AFTER them.
            stack.push((node, true));
            for i in 0..children[node].len() {
                stack.push((children[node][i], false));
            }
        }
    }

    // `timer` is global across all roots, never reset. That is what makes
    // different trees occupy disjoint intervals, so cross-tree queries are
    // False automatically. Both comparisons are non-strict, which is what makes
    // u == v return true with no special case.
    queries
        .iter()
        .map(|&(u, v)| tin[u] <= tin[v] && tout[v] <= tout[u])
        .collect()
}
