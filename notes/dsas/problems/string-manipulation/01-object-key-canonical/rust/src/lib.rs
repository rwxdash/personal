//! Object Key Canonicalisation
//! problems/string-manipulation/01-object-key-canonical
//!
//! Fill in `canonical_key`, then run:
//!
//!     cargo test
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

/// Reduce a user-supplied object key to its canonical form.
///
/// Segments are separated by `/`. A segment of `"."` contributes nothing;
/// `".."` removes the previous surviving segment; an empty segment (from
/// `"//"`, a leading `/` or a trailing `/`) contributes nothing; anything else
/// is an ordinary name and is kept.
///
/// ONLY exactly `"."` and exactly `".."` are special — `"..."`, `"..a"`,
/// `".hidden"` and `"a..b"` are ordinary names.
///
/// The canonical form is the survivors joined by single `/`, with no leading
/// and no trailing slash. Zero survivors is the empty string, a valid result
/// meaning "the top level itself".
///
/// A `".."` with no surviving segment to remove is a path traversal attempt and
/// rejects the whole key — including when valid segments follow it.
///
/// Returns the canonical key, or `None` if the key traverses above the top
/// level. Note `Some("")` and `None` are different results.
///
/// Required: O(n) time, O(n) space.
pub fn canonical_key(raw: &str) -> Option<String> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case. The random cross-check uses a
// character-level scanner that never splits the string.
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

    /// Character-level scanner: accumulate a segment, resolve at each '/'.
    fn brute(raw: &str) -> Option<String> {
        let bytes = raw.as_bytes();
        let n = bytes.len();
        let mut kept: Vec<String> = Vec::new();
        let mut seg = String::new();
        for i in 0..=n {
            if i == n || bytes[i] == b'/' {
                if seg.is_empty() || seg == "." {
                    // contributes nothing
                } else if seg == ".." {
                    if kept.is_empty() {
                        return None;
                    }
                    kept.pop();
                } else {
                    kept.push(seg.clone());
                }
                seg.clear();
            } else {
                seg.push(bytes[i] as char);
            }
        }
        let mut out = String::new();
        for (idx, part) in kept.iter().enumerate() {
            if idx > 0 {
                out.push('/');
            }
            out.push_str(part);
        }
        Some(out)
    }

    fn s(x: &str) -> Option<String> {
        Some(x.to_string())
    }

    #[test]
    fn examples() {
        assert_eq!(
            canonical_key("uploads//2024/../2025/./report.pdf"),
            s("uploads/2025/report.pdf")
        );
        assert_eq!(canonical_key("/logs/app/"), s("logs/app"));
        assert_eq!(canonical_key("a/../.."), None);
        assert_eq!(canonical_key("a/b/../.."), s(""));
        assert_eq!(canonical_key("..hidden/.../a..b/."), s("..hidden/.../a..b"));
    }

    #[test]
    fn empty_vs_rejected() {
        // Some("") and None are different results. These fail if a traversal
        // above the top level is silently clamped instead of rejected.
        assert_eq!(canonical_key("a/b/../.."), s(""));
        assert_eq!(canonical_key("a/../.."), None);
        assert_eq!(canonical_key(".."), None);
        assert_eq!(canonical_key("/.."), None);
        assert_eq!(canonical_key("../a"), None);
        assert_eq!(
            canonical_key("a/../../b"),
            None,
            "rejection is not undone by later segments"
        );
        assert_eq!(canonical_key("a/.."), s(""));
        assert_eq!(canonical_key(""), s(""));
        assert_eq!(canonical_key("/"), s(""));
        assert_eq!(canonical_key("///"), s(""));
        assert_eq!(canonical_key("."), s(""));
        assert_eq!(canonical_key("./././."), s(""));
    }

    #[test]
    fn dots_that_are_not_special() {
        // Only exactly "." and exactly ".." are special. These fail for any
        // check based on starts_with, "contains a dot", or counting dots.
        assert_eq!(canonical_key("..."), s("..."));
        assert_eq!(canonical_key("...."), s("...."));
        assert_eq!(canonical_key("..a"), s("..a"));
        assert_eq!(canonical_key("a.."), s("a.."));
        assert_eq!(canonical_key("a..b"), s("a..b"));
        assert_eq!(canonical_key(".hidden"), s(".hidden"));
        assert_eq!(canonical_key(".../.."), s(""), "... is a name, so .. cancels it");
        assert_eq!(canonical_key(".../../.."), None);
        assert_eq!(canonical_key("a/.../b"), s("a/.../b"));
        assert_eq!(canonical_key("a/..../.."), s("a"));
    }

    #[test]
    fn edges() {
        // Single ordinary segment, with and without surrounding slashes.
        assert_eq!(canonical_key("file.txt"), s("file.txt"));
        assert_eq!(canonical_key("/file.txt"), s("file.txt"));
        assert_eq!(canonical_key("file.txt/"), s("file.txt"));
        assert_eq!(canonical_key("/file.txt/"), s("file.txt"));
        // Repeated slashes everywhere.
        assert_eq!(canonical_key("//a//b//"), s("a/b"));
        assert_eq!(canonical_key("////a////"), s("a"));
        // Interleaved dots.
        assert_eq!(canonical_key("a/./b/./c"), s("a/b/c"));
        assert_eq!(canonical_key("./a/."), s("a"));
        // Up and back down.
        assert_eq!(canonical_key("a/b/../c"), s("a/c"));
        assert_eq!(canonical_key("a/b/c/../../d"), s("a/d"));
        assert_eq!(canonical_key("a/../b/../c"), s("c"));
        // Trailing '..' after a rebuild.
        assert_eq!(canonical_key("a/b/../../c/.."), s(""));
        // Names using the other allowed characters.
        assert_eq!(
            canonical_key("my-bucket_2/v1.2.3/file_name-01.tar.gz"),
            s("my-bucket_2/v1.2.3/file_name-01.tar.gz")
        );
        // Digits only.
        assert_eq!(canonical_key("2024/01/02"), s("2024/01/02"));
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x2DE92C6F592B0275);
        let pieces = ["a", "b", ".", "..", "...", "", "x1", ".hid"];
        for _ in 0..2_000 {
            let count = rng.below(9);
            let segs: Vec<&str> = (0..count)
                .map(|_| pieces[rng.below(pieces.len() as u64) as usize])
                .collect();
            let mut raw = segs.join("/");
            if rng.below(3) == 0 {
                raw = format!("/{}", raw);
            }
            if rng.below(3) == 0 {
                raw = format!("{}/", raw);
            }
            assert_eq!(canonical_key(&raw), brute(&raw), "raw={:?}", raw);
        }
    }

    #[test]
    fn large_deep_nesting() {
        // A key 100k segments deep.
        let depth = 100_000usize;
        let segs: Vec<String> = (0..depth).map(|i| format!("seg{}", i)).collect();
        let raw = segs.join("/");
        assert_eq!(canonical_key(&raw), Some(raw.clone()));
        // Walk all the way back up: valid, resolving to the top level.
        let ups = vec![".."; depth].join("/");
        let raw_up = format!("{}/{}", raw, ups);
        assert_eq!(canonical_key(&raw_up), s(""));
        // One step too far: rejected.
        assert_eq!(canonical_key(&format!("{}/..", raw_up)), None);
    }

    #[test]
    fn large_all_dots() {
        // 500k "." segments contribute nothing.
        let n = 500_000usize;
        let dots = vec!["."; n].join("/");
        assert_eq!(canonical_key(&dots), s(""));
        assert_eq!(canonical_key(&format!("a/{}", dots)), s("a"));
    }

    #[test]
    fn large_all_slashes() {
        // A key that is nothing but separators.
        let raw = "/".repeat(1_000_000);
        assert_eq!(canonical_key(&raw), s(""));
    }

    #[test]
    fn large_zigzag() {
        // Alternating descend and ascend keeps the stack shallow but busy.
        let n = 200_000usize;
        let mut parts: Vec<&str> = Vec::with_capacity(2 * n);
        for _ in 0..n {
            parts.push("a");
            parts.push("..");
        }
        let raw = parts.join("/");
        assert_eq!(canonical_key(&raw), s(""));
        assert_eq!(canonical_key(&format!("{}/a", raw)), s("a"));
    }

    #[test]
    fn large_rejected_early() {
        // A traversal at the very start, followed by a huge valid tail.
        let tail: Vec<String> = (0..100_000).map(|i| format!("seg{}", i)).collect();
        let raw = format!("../{}", tail.join("/"));
        assert_eq!(canonical_key(&raw), None);
    }

    #[test]
    fn large_no_quadratic_join() {
        // 200k surviving segments: building the result must not be quadratic.
        let n = 200_000usize;
        let segs: Vec<String> = (0..n).map(|i| format!("s{}", i)).collect();
        let raw = segs.join("/");
        let out = canonical_key(&raw).expect("valid key");
        assert_eq!(out.len(), raw.len());
        assert_eq!(out, raw);
    }
}
