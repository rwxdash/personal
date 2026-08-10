//! Reference solution — Object Key Canonicalisation.
//!
//! Verified against
//! problems/string-manipulation/01-object-key-canonical/rust.
//! O(n) time, O(n) space.

pub fn canonical_key(raw: &str) -> Option<String> {
    // The ".." rule removes the MOST RECENT survivor and can never bring it
    // back, which is exactly a stack.
    //
    // The stack holds &str slices borrowed from `raw` rather than owned
    // Strings, so no segment is ever copied -- only the final join allocates.
    let mut stack: Vec<&str> = Vec::new();

    // Splitting is what turns leading, trailing and doubled slashes into empty
    // segments, so one rule ("empty contributes nothing") covers all three
    // instead of three special cases. Note "".split('/') yields a single empty
    // piece, so an empty key falls out correctly with no guard.
    for segment in raw.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }

        // EXACT equality, not starts_with or a substring test: "...", "..a",
        // ".hidden" and "a..b" are all ordinary names.
        if segment == ".." {
            if stack.is_empty() {
                // Reject on the spot rather than clamping at the top level.
                // Clamping is the classic path-traversal vulnerability, and it
                // would turn "a/../.." into Some("") instead of None.
                return None;
            }
            stack.pop();
            continue;
        }

        stack.push(segment);
    }

    // join gives no leading or trailing slash, and yields "" for an empty
    // stack -- a VALID result meaning "the top level itself", distinct from
    // None. Joining once rather than concatenating in a loop keeps this linear.
    Some(stack.join("/"))
}
