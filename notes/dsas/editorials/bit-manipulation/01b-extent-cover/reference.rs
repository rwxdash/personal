//! Reference solution — Extent Cover.
//!
//! Greedy from the left. At each position take the largest extent that is legal
//! there; both limits on "largest" are one-instruction questions about a
//! number's binary representation.

pub fn extent_cover(ranges: &[(u64, u64)]) -> Vec<Vec<(u64, u64)>> {
    ranges
        .iter()
        .map(|&(start, end)| {
            let mut extents = Vec::new();
            let mut pos = start;
            let mut remaining = end - start + 1;

            while remaining > 0 {
                // Limit 1 -- alignment. An extent of size 2^k needs pos's low
                // k bits to be zero, so the biggest k allowed here is the
                // trailing-zero count. At pos == 0 this returns 64: every
                // power of two divides zero, so there is no alignment limit.
                let align = pos.trailing_zeros();

                // Limit 2 -- fit. The largest power of two that is <=
                // remaining is 2^f, where f is one less than the bit length.
                // remaining is non-zero here, so leading_zeros() <= 63 and the
                // subtraction cannot underflow. f is at most 48.
                let fit = 63 - remaining.leading_zeros();

                // Both are exponents, so the min is taken BEFORE any shift.
                // The other order computes 1u64 << 64 at position 0, which is
                // a panic in debug and undefined shift behaviour in general.
                let k = align.min(fit);
                let size = 1u64 << k;

                extents.push((pos, size));
                pos += size;
                remaining -= size;
            }

            extents
        })
        .collect()
}
