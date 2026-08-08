//! Reference solution — CIDR Aggregation.
//!
//! Verified against problems/bit-manipulation/01-cidr-aggregation/rust.
//! O(ranges.len()) time, O(1) space beyond the output.

pub fn smallest_cidr(ranges: &[(u32, u32)]) -> Vec<(u32, u32, bool)> {
    ranges
        .iter()
        .map(|&(start, end)| {
            // XOR is zero wherever the two addresses agree, so its highest set
            // bit marks the first position where they differ. Everything from
            // there down must be left free by the block.
            let diff = start ^ end;

            // leading_zeros() is 32 when diff is 0, giving free = 0 -- exactly
            // the start == end case, a /32. No branch needed.
            let free = 32 - diff.leading_zeros();
            let prefix_len = 32 - free;

            // `1u32 << 32` is invalid, so the whole-address-space case (free
            // == 32) needs the mask handled separately. Python's unbounded
            // integers hide this entirely; here it is a debug panic.
            let network = if free == 32 {
                0
            } else {
                start & !((1u32 << free) - 1)
            };

            // Size can be 2^32, which does not fit in u32 -- do the span
            // arithmetic in u64. The block equals the requested range only
            // when BOTH ends line up; checking only the start would call
            // (5, 6) exact.
            let size = 1u64 << free;
            let exact = network == start && network as u64 + size - 1 == end as u64;

            (network, prefix_len, exact)
        })
        .collect()
}
