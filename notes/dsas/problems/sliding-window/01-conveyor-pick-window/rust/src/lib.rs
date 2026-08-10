//! Conveyor Pick Window
//! problems/sliding-window/01-conveyor-pick-window
//!
//! Fill in `shortest_fulfilling_run`, then run `cargo test` in this directory.
//!
//! Statement: ../../PROBLEM.md   Stuck? ../../HINTS.md

use std::collections::HashMap;

/// Find the shortest contiguous stretch of `stream` that fills `order`.
///
/// A stretch `stream[start..=end]` fills the order if, for every `sku` in
/// `order`, the stretch contains at least `order[sku]` occurrences of `sku`.
/// Surplus occurrences are allowed.
///
/// Returns the inclusive `(start, end)` index pair of the shortest filling
/// stretch, breaking ties toward the smallest `start`, or `None` if no stretch
/// can fill the order.
///
/// Required: O(n) time expected, O(k) extra space for k distinct ordered SKUs.
pub fn shortest_fulfilling_run(
    stream: &[String],
    order: &HashMap<String, u32>,
) -> Option<(usize, usize)> {
    todo!()
}

// ============================ TESTS — do not edit ============================
// These mirror the Python asserts case for case, including the same
// deterministic random cross-check against a slow reference.
#[cfg(test)]
mod tests {
    use super::*;

    fn belt(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn ord(items: &[(&str, u32)]) -> HashMap<String, u32> {
        items.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

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

    /// Obviously-correct O(n^2 * k) checker. Far too slow for the real bounds.
    fn brute(stream: &[String], order: &HashMap<String, u32>) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize)> = None;
        for left in 0..stream.len() {
            let mut have: HashMap<&str, u32> = HashMap::new();
            for right in left..stream.len() {
                let sku = stream[right].as_str();
                if order.contains_key(sku) {
                    *have.entry(sku).or_insert(0) += 1;
                }
                let filled = order
                    .iter()
                    .all(|(k, v)| have.get(k.as_str()).copied().unwrap_or(0) >= *v);
                if filled {
                    let better = match best {
                        None => true,
                        Some((bl, br)) => right - left < br - bl,
                    };
                    if better {
                        best = Some((left, right));
                    }
                    break; // extending this left end only makes it longer
                }
            }
        }
        best
    }

    #[test]
    fn examples() {
        assert_eq!(
            shortest_fulfilling_run(
                &belt(&["A", "B", "A", "C", "A", "B"]),
                &ord(&[("A", 2), ("B", 1)])
            ),
            Some((0, 2))
        );
        assert_eq!(
            shortest_fulfilling_run(&belt(&["A", "A", "B", "B"]), &ord(&[("A", 1), ("B", 1)])),
            Some((1, 2))
        );
        assert_eq!(
            shortest_fulfilling_run(
                &belt(&["A", "B", "Z", "A", "B"]),
                &ord(&[("A", 1), ("B", 1)])
            ),
            Some((0, 1)),
            "ties must break toward the smallest start index"
        );
        assert_eq!(
            shortest_fulfilling_run(&belt(&["X", "Y"]), &ord(&[("X", 2)])),
            None
        );
    }

    #[test]
    fn edges() {
        // Single item, single requirement.
        assert_eq!(
            shortest_fulfilling_run(&belt(&["A"]), &ord(&[("A", 1)])),
            Some((0, 0))
        );
        // Required SKU never appears at all.
        assert_eq!(
            shortest_fulfilling_run(&belt(&["A", "A", "A"]), &ord(&[("B", 1)])),
            None
        );
        // The whole belt is needed, exactly.
        assert_eq!(
            shortest_fulfilling_run(
                &belt(&["A", "B", "C"]),
                &ord(&[("A", 1), ("B", 1), ("C", 1)])
            ),
            Some((0, 2))
        );
        // Nothing but filler.
        assert_eq!(
            shortest_fulfilling_run(&vec!["Z".to_string(); 50], &ord(&[("A", 1)])),
            None
        );
        // Surplus must not be mistaken for progress.
        assert_eq!(
            shortest_fulfilling_run(&vec!["A".to_string(); 5], &ord(&[("A", 2), ("B", 1)])),
            None
        );
        // Deep surplus on one SKU, requirement satisfied only at the very end.
        let mut s = vec!["A".to_string(); 10];
        s.push("B".to_string());
        assert_eq!(
            shortest_fulfilling_run(&s, &ord(&[("A", 3), ("B", 1)])),
            Some((7, 10))
        );
        // Duplicates and a high quantity on one SKU.
        assert_eq!(
            shortest_fulfilling_run(&belt(&["A", "A", "B", "A", "A", "A"]), &ord(&[("A", 4)])),
            Some((0, 4)),
            "earliest window of four A's is 0..4 (B is filler inside it)"
        );
    }

    #[test]
    fn random_against_brute() {
        let mut rng = Lcg::new(0x243F6A8885A308D3);
        for _ in 0..400 {
            let n = 1 + rng.below(24);
            let alpha = 1 + rng.below(4);
            let stream: Vec<String> = (0..n).map(|_| format!("S{}", rng.below(alpha))).collect();
            let k = 1 + rng.below(alpha);
            let mut order: HashMap<String, u32> = HashMap::new();
            for i in 0..k {
                order.insert(format!("S{}", i), 1 + rng.below(3) as u32);
            }
            assert_eq!(
                shortest_fulfilling_run(&stream, &order),
                brute(&stream, &order),
                "stream={:?} order={:?}",
                stream,
                order
            );
        }
    }

    #[test]
    fn large_planted() {
        // 200k belt whose only tight window is planted; decoys are far away.
        // The brute checker above would do ~2 * 10^10 steps here.
        let n = 200_000;
        let mut stream = vec!["Z".to_string(); n];
        for (i, sku) in [(10, "A"), (5_000, "A"), (9_000, "B"), (15_000, "C")] {
            stream[i] = sku.to_string();
        }
        for (i, sku) in [
            (100_000, "A"),
            (100_001, "B"),
            (100_002, "A"),
            (100_003, "C"),
        ] {
            stream[i] = sku.to_string();
        }
        assert_eq!(
            shortest_fulfilling_run(&stream, &ord(&[("A", 2), ("B", 1), ("C", 1)])),
            Some((100_000, 100_003))
        );
    }

    #[test]
    fn large_whole_belt() {
        // Worst case for the window: the answer spans the entire belt.
        let n = 200_000usize;
        let mut stream = vec!["A".to_string(); n - 1];
        stream.push("B".to_string());
        assert_eq!(
            shortest_fulfilling_run(&stream, &ord(&[("A", (n - 1) as u32), ("B", 1)])),
            Some((0, n - 1))
        );
    }

    #[test]
    fn large_uniform() {
        // Every position is a candidate; punishes anything quadratic in n.
        let n = 200_000usize;
        let stream = vec!["A".to_string(); n];
        assert_eq!(
            shortest_fulfilling_run(&stream, &ord(&[("A", 1)])),
            Some((0, 0))
        );
        assert_eq!(
            shortest_fulfilling_run(&stream, &ord(&[("A", n as u32)])),
            Some((0, n - 1))
        );
        assert_eq!(
            shortest_fulfilling_run(&stream, &ord(&[("A", n as u32 + 1)])),
            None
        );
    }

    #[test]
    fn large_shuffled() {
        // Randomised 200k belt; the planted P,Q,R triple is the unique
        // length-3 stretch that fills the order, whatever the filler is.
        let n = 200_000usize;
        let mut rng = Lcg::new(0xB5026F5AA96619E9);
        let filler = ["Z", "Y", "X"];
        let mut stream: Vec<String> = (0..n)
            .map(|_| filler[rng.below(3) as usize].to_string())
            .collect();
        let plant = 173_456usize;
        stream[plant] = "P".to_string();
        stream[plant + 1] = "Q".to_string();
        stream[plant + 2] = "R".to_string();
        assert_eq!(
            shortest_fulfilling_run(&stream, &ord(&[("P", 1), ("Q", 1), ("R", 1)])),
            Some((plant, plant + 2))
        );
    }
}
