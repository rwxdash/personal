//! Route Prefix Match
//! problems/tries/01-route-prefix-match
//!
//! Fill in `longest_route_match`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Longest registered route that is a prefix of each request path.
///
/// Matching is on raw characters, not path segments: a route matches a path
/// when the route is a literal prefix of it. A route equal to the path counts
/// as a match. A route longer than the path never matches.
///
/// `routes` may contain duplicates and the empty string. Characters are
/// lowercase `a-z` and `/`.
///
/// Returns one entry per path: the LENGTH of the longest matching route, or
/// `-1` if no route matches. Note `0` and `-1` differ — a registered empty
/// route matches every path with length 0.
///
/// Required: O(R + P) time in the total character counts, O(R) space.
pub fn longest_route_match(routes: &[String], paths: &[String]) -> Vec<i64> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check
// compares every path against every route directly.
#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic 64-bit LCG, byte-identical to the Python test generator.
    struct Lcg(u64);

    impl Lcg {
        fn new(seed: u64) -> Self {
            Lcg(seed)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 % n
        }
    }

    fn strs(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// Check every path against every route.
    fn brute(routes: &[String], paths: &[String]) -> Vec<i64> {
        paths
            .iter()
            .map(|p| {
                let mut best: i64 = -1;
                for r in routes {
                    if r.len() <= p.len() && p.as_bytes()[..r.len()] == *r.as_bytes() {
                        best = best.max(r.len() as i64);
                    }
                }
                best
            })
            .collect()
    }

    #[test]
    fn examples() {
        assert_eq!(
            longest_route_match(
                &strs(&["/api", "/api/v2", "/health"]),
                &strs(&["/api/v2/users", "/api/v1/users", "/health", "/metrics"])
            ),
            vec![7, 4, 7, -1]
        );
        assert_eq!(
            longest_route_match(&strs(&["", "/api"]), &strs(&["/api/x", "/other", ""])),
            vec![4, 0, 0]
        );
        assert_eq!(
            longest_route_match(
                &strs(&["/api/v2/users"]),
                &strs(&["/api", "/api/v2/users", "/api/v2/users/42"])
            ),
            vec![-1, 13, 13]
        );
        assert_eq!(
            longest_route_match(&strs(&["/api"]), &strs(&["/apixyz", "/ap"])),
            vec![4, -1]
        );
    }

    #[test]
    fn empty_route_vs_no_match() {
        // 0 and -1 are different answers. These fail if the root's terminal
        // flag is checked inside the character loop rather than before it.
        assert_eq!(
            longest_route_match(&strs(&[""]), &strs(&["anything"])),
            vec![0]
        );
        assert_eq!(longest_route_match(&strs(&[""]), &strs(&[""])), vec![0]);
        assert_eq!(longest_route_match(&strs(&["a"]), &strs(&[""])), vec![-1]);
        assert_eq!(longest_route_match(&[], &strs(&[""])), vec![-1]);
        assert_eq!(
            longest_route_match(&strs(&["", "a"]), &strs(&["", "a", "b"])),
            vec![0, 1, 0]
        );
    }

    #[test]
    fn edges() {
        // No routes at all.
        assert_eq!(
            longest_route_match(&[], &strs(&["/a", "/b"])),
            vec![-1, -1]
        );
        // No paths at all.
        assert_eq!(longest_route_match(&strs(&["/a"]), &[]), Vec::<i64>::new());
        // Both empty.
        assert_eq!(longest_route_match(&[], &[]), Vec::<i64>::new());
        // Duplicate routes change nothing.
        assert_eq!(
            longest_route_match(&strs(&["/a", "/a", "/a"]), &strs(&["/abc"])),
            vec![2]
        );
        // Exact match.
        assert_eq!(
            longest_route_match(&strs(&["/abc"]), &strs(&["/abc"])),
            vec![4]
        );
        // Nested routes: the deepest one wins, whatever the insertion order.
        assert_eq!(
            longest_route_match(&strs(&["/a", "/ab", "/abc"]), &strs(&["/abcd"])),
            vec![4]
        );
        assert_eq!(
            longest_route_match(&strs(&["/abc", "/ab", "/a"]), &strs(&["/abcd"])),
            vec![4]
        );
        // A route that exists only as an internal node is not a match.
        assert_eq!(longest_route_match(&strs(&["/abc"]), &strs(&["/ab"])), vec![-1]);
        assert_eq!(longest_route_match(&strs(&["/abc"]), &strs(&["/abz"])), vec![-1]);
        // Sibling branches do not interfere.
        assert_eq!(
            longest_route_match(&strs(&["/ax", "/ay"]), &strs(&["/ax1", "/ay1", "/az1"])),
            vec![3, 3, -1]
        );
        // The same path queried repeatedly.
        assert_eq!(
            longest_route_match(&strs(&["/a"]), &vec!["/ab".to_string(); 5]),
            vec![2; 5]
        );
        // Single characters.
        assert_eq!(
            longest_route_match(&strs(&["a", "b"]), &strs(&["a", "b", "c"])),
            vec![1, 1, -1]
        );
        // Only slashes.
        assert_eq!(longest_route_match(&strs(&["/", "//"]), &strs(&["///"])), vec![2]);
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x19A4C116B8D2D0C8);
        let alphabet = ['a', 'b', '/'];
        for _ in 0..400 {
            let nr = rng.below(7);
            let np = rng.below(7);
            let mut routes: Vec<String> = Vec::new();
            for _ in 0..nr {
                let len = rng.below(5);
                routes.push((0..len).map(|_| alphabet[rng.below(3) as usize]).collect());
            }
            let mut paths: Vec<String> = Vec::new();
            for _ in 0..np {
                let len = rng.below(6);
                paths.push((0..len).map(|_| alphabet[rng.below(3) as usize]).collect());
            }
            assert_eq!(
                longest_route_match(&routes, &paths),
                brute(&routes, &paths),
                "routes={:?} paths={:?}",
                routes,
                paths
            );
        }
    }

    #[test]
    fn large_shared_prefix() {
        // 50k routes all sharing a long prefix.
        let base = "/service/v1/resource/";
        let routes: Vec<String> = (0..50_000).map(|i| format!("{}{:05}", base, i)).collect();
        let paths: Vec<String> = (0..50_000)
            .step_by(5)
            .map(|i| format!("{}{:05}/detail", base, i))
            .collect();
        let expected = vec![(base.len() + 5) as i64; paths.len()];
        assert_eq!(longest_route_match(&routes, &paths), expected);
    }

    #[test]
    fn large_nested_ladder() {
        // Routes nested inside one another; the deepest must win.
        let depth = 2_000usize;
        let routes: Vec<String> = (1..=depth).map(|k| "a".repeat(k)).collect();
        let paths = vec![
            "a".repeat(depth),
            "a".repeat(depth / 2),
            "a".repeat(depth + 500),
            format!("b{}", "a".repeat(depth)),
        ];
        assert_eq!(
            longest_route_match(&routes, &paths),
            vec![depth as i64, (depth / 2) as i64, depth as i64, -1]
        );
    }

    #[test]
    fn large_no_match() {
        // 100k paths sharing no first character with any route.
        let routes: Vec<String> = (0..50_000).map(|i| format!("/api/{:06}", i)).collect();
        let paths: Vec<String> = (0..100_000).map(|i| format!("zzz{:06}", i)).collect();
        assert_eq!(longest_route_match(&routes, &paths), vec![-1; 100_000]);
    }

    #[test]
    fn large_catch_all() {
        // An empty route registered alongside 50k real ones.
        let mut routes = vec![String::new()];
        routes.extend((0..50_000).map(|i| format!("/svc/{:05}", i)));
        let paths = strs(&["/svc/00042/x", "/nowhere", "", "/svc/49999"]);
        assert_eq!(longest_route_match(&routes, &paths), vec![10, 0, 0, 10]);
    }

    #[test]
    fn large_many_duplicates() {
        // The same route registered 200k times.
        let routes = vec!["/dup".to_string(); 200_000];
        let paths = vec!["/dup/x".to_string(); 1_000];
        assert_eq!(longest_route_match(&routes, &paths), vec![4; 1_000]);
    }

    #[test]
    fn large_long_strings() {
        // A few very long routes and paths, near the character budget.
        let long_route = format!("/{}", "ab".repeat(200_000)); // 400_001 chars
        let routes = vec![long_route.clone(), "/ab".to_string()];
        let paths = vec![
            format!("{}/tail", long_route),
            "/abc".to_string(),
            long_route[..100].to_string(),
        ];
        assert_eq!(
            longest_route_match(&routes, &paths),
            vec![long_route.len() as i64, 3, 3]
        );
    }

    #[test]
    fn large_wide_fanout() {
        // A root with many distinct single-character branches.
        let letters = "abcdefghijklmnopqrstuvwxyz";
        let routes: Vec<String> = letters.chars().map(|c| c.to_string()).collect();
        let mut paths: Vec<String> = letters.chars().map(|c| format!("{}tail", c)).collect();
        paths.push("/nope".to_string());
        let mut expected = vec![1i64; 26];
        expected.push(-1);
        assert_eq!(longest_route_match(&routes, &paths), expected);
    }
}
