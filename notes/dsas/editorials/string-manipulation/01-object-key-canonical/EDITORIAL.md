# Editorial — Object Key Canonicalisation

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

This problem is unusual in the set: there is no clever algorithm to find. The
approach is nearly forced, and the difficulty is entirely in **handling every
case exactly right**. That is a real interview skill and a real production
skill — the bugs it targets are the ones that ship.

The one structural observation you do need: `..` removes the **most recent**
surviving segment, and once removed it never comes back. That is last-in
first-out, so the container is a **stack**.

Everything else is case analysis.

## The approach

```
stack = []
for segment in raw.split("/"):
    if segment == "" or segment == ".":  continue
    if segment == "..":
        if not stack: return None
        stack.pop(); continue
    stack.append(segment)
return "/".join(stack)
```

Eight lines. Here is what each decision is buying.

### Splitting collapses three cases into one

`"/a//b/"` split on `/` gives `["", "a", "", "b", ""]`. A leading slash, a
trailing slash, and a doubled slash all produce **empty segments** — so the
single rule "empty contributes nothing" handles all three. Special-casing them
separately means three chances to get it wrong.

An empty input splits to `[""]` in both Python and Rust: one empty segment,
discarded, leaving an empty stack. No guard needed. (Worth checking in your
language rather than assuming — this is not universal.)

### Reject, do not clamp

When `..` finds an empty stack, return `None` immediately.

The tempting alternative is to ignore the `..` and stay at the top — "clamping".
That is the **classic path-traversal vulnerability**: it silently accepts
`a/../../../../etc/passwd` and resolves it to something harmless-looking, which
is exactly how an attacker gets a key they should not have. The problem
specifies rejection for that reason, not out of pedantry.

Verified: clamping instead of rejecting fails 6 of the 11 tests.

Rejection is also **total** — one bad `..` invalidates the whole key, even if
valid segments follow. `"a/../../b"` is `None`, not `"b"`. Returning early
handles this automatically; a flag checked at the end does not, unless you also
stop mutating the stack.

### Exact equality, always

Only *exactly* `.` and *exactly* `..` are special. `...`, `..a`, `.hidden` and
`a..b` are ordinary names that must be kept.

Any test based on `startswith("..")`, `contains(".")`, or counting dots is
wrong. Mutating the reference to `starts_with("..")` fails 3 of the 11 tests,
and `dots_that_are_not_special` exists entirely to catch this family of bugs.

### Empty output and rejection are different results

- `"a/b/../.."` cancels down to nothing and is **valid** — it resolves to the
  top level. Returns `""`.
- `"a/../.."` has one `..` too many and is **rejected**. Returns `None`.

Both "look empty". Conflating them means a caller cannot distinguish "the top
level" from "this key was an attack", which is precisely the distinction the
gateway needs.

### Joining

`"/".join(stack)` gives no leading slash, no trailing slash, and `""` for an
empty stack — all three requirements at once. If you build the string manually,
test the empty case and the single-element case, which are where the stray
slashes appear.

Do **not** concatenate inside the loop. With a million characters that is
quadratic in some languages; collect and join once. In Rust, push `&str` slices
borrowed from the input rather than owned `String`s, so no segment is ever
copied.

## Complexity

- **Time:** `O(n)` — one pass to split, one push or pop per segment, one pass
  to join.
- **Space:** `O(n)` for the stack and the output.

There is no slow-but-correct alternative to beat here, which is why the large
tests target *quadratic string building* and deep stacks instead of algorithmic
blow-up.

## Common wrong turns

- **Clamping `..` at the top level** instead of rejecting. The security bug.
- **`startswith("..")` or similar** instead of exact equality.
- **Returning `None` for a key that cancels to nothing**, or `""` for a
  rejected one.
- **Letting a later valid segment undo a rejection.**
- **Handling leading/trailing slashes as special cases** instead of letting them
  become empty segments.
- **Producing `"/"` for an empty result**, or a leading slash on a valid one.
- **Concatenating the output inside the loop.**
- **Trimming slashes from the input first**, then splitting — works, but it is
  more code and more edge cases than letting the split do it.

## Interview follow-ups

1. **Absolute versus relative keys.** If a leading `/` were meaningful, how
   would the answer change? Forces them to notice they threw that information
   away.
2. **Symbolic links.** Now `..` cannot be resolved lexically at all — it
   depends on where the link points. This is the real reason operating systems
   resolve paths against the filesystem rather than textually, and it is a
   good "why is the real thing harder" question.
3. **Percent-encoding.** `%2e%2e` decodes to `..`. Should decoding happen
   before or after canonicalisation? (Before — and decoding *after* is a
   well-known bypass. A strong candidate will say the order is the whole
   vulnerability.)
4. **Windows separators**, mixed `\` and `/`. Normalisation before splitting,
   and a discussion of why accepting both is risky.
5. **Case sensitivity.** Should `Uploads` and `uploads` canonicalise the same?
   Depends on the store — the point is to make them ask rather than assume.
6. **Streaming.** A key arriving in chunks, canonicalised without buffering the
   whole thing. Mostly works, but `..` can reach back arbitrarily far, so you
   cannot emit output until you know it will not be cancelled.
