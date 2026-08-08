# Hints — Object Key Canonicalisation

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Split a string on `/`, walk the pieces left to right, drop the ones that mean
> nothing, let `..` cancel the most recent surviving piece, reject if a `..`
> has nothing to cancel, and join what is left with single slashes.

This problem is not about finding a clever algorithm — the approach is almost
forced. It is about **handling every case exactly right**, which is a different
skill and the one being tested here. So before writing anything, write down the
full list of things a segment can be:

1. `.` — contributes nothing
2. `..` — cancels the previous survivor, or rejects the key if there is none
3. empty (from `//`, a leading `/`, or a trailing `/`) — contributes nothing
4. anything else — an ordinary name

And note the two traps hiding in that list:

- **Only *exactly* `.` and *exactly* `..` are special.** `...`, `..a`,
  `.hidden` and `a..b` are ordinary names. A check like "starts with a dot" or
  "contains `..`" is wrong.
- **Empty output and rejection are different.** `"a/b/../.."` cancels down to
  nothing and is *valid* — it resolves to the top level. `"a/../.."` has one
  `..` too many and is *rejected*. One returns `""`, the other returns `None`.

Now the structural question: `..` removes *the most recent* surviving segment.
What container gives you "most recent" for free?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

The `..` rule is last-in-first-out. The segment it cancels is always the one
most recently kept, and once cancelled it can never come back. That is exactly
a **stack**.

So the whole algorithm is: split, then push and pop.

```
for each segment:
    if it is "." or empty:  ignore
    elif it is "..":        pop, or reject if the stack is empty
    else:                   push it
```

The stack at the end holds the surviving segments in order, and joining them
with `/` gives the answer.

Two things worth planning before you write it:

- **Splitting is where the empty segments come from.** `"/a//b/"` split on `/`
  gives `["", "a", "", "b", ""]`. Rather than special-casing leading and
  trailing slashes, let them become empty segments and let rule 3 discard them.
  That is one rule instead of three.
- **Joining is where the leading and trailing slash bugs come from.** Build the
  result by joining with a single `/` between elements — do not prepend a slash
  and do not append one. An empty stack must join to the empty string, not to
  `"/"`.

The `O(n)` requirement is not a real constraint here — any sane implementation
meets it. But do not build the output by repeated string concatenation inside a
loop, which is quadratic in some languages.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Split, stack, join.** There is no cleverness to find; the work is in the case
handling.

The specific things that separate a correct implementation from a
nearly-correct one:

1. **Compare segments for exact equality** with `"."` and `".."`. Not
   `startswith`, not `in`, not "count the dots". Example 5 exists solely to
   catch this.
2. **Reject on underflow, do not clamp.** When `..` finds an empty stack, the
   whole key is invalid — return `None` immediately. Silently ignoring the `..`
   (clamping at the top) is the standard path-traversal vulnerability, and it
   makes `"a/../.."` return `""` instead of `None`.
3. **Reject the whole key, not just the tail.** Once one `..` underflows, the
   answer is `None` regardless of what comes after. `"a/../../b"` is rejected
   even though `b` is a fine segment.
4. **An empty stack at the end is a valid empty string**, not `None` and not
   `"/"`.
5. **An empty input** splits to a single empty segment, which is discarded,
   leaving an empty stack — so the answer is `""`.

Write those five down and check your implementation against each one
explicitly. Every one of them is a case where a plausible implementation
produces plausible-looking output that is wrong.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

```
def canonical_key(raw):
    stack = []
    for segment in raw.split("/"):
        if segment == "" or segment == ".":
            continue                       # contributes nothing
        if segment == "..":
            if not stack:
                return None                # traversal above the top level
            stack.pop()
            continue
        stack.append(segment)              # ordinary name
    return "/".join(stack)                 # "" when the stack is empty
```

That is the whole solution. The care is in the details around it:

- **`raw.split("/")` on an empty string** gives `[""]` in Python — one empty
  segment, discarded, so the answer is `""`. In Rust, `"".split('/')` also
  yields one empty piece, so the behaviour matches. Check this in your language
  rather than assuming.
- **Return `None` on the spot**, not after the loop. A flag checked at the end
  works too, but only if you also stop mutating the stack — otherwise a later
  `..` can pop a segment that should not have existed and you get a confusing
  second failure.
- **`"/".join(stack)`** produces no leading or trailing slash automatically, and
  gives `""` for an empty stack. If you are building the string manually,
  test both the empty case and the single-element case.
- **Do not concatenate in a loop.** Collect into a list (or a `Vec<&str>`) and
  join once. With a million characters, repeated concatenation is the
  difference between instant and slow.
- In Rust, you can push `&str` slices borrowed from `raw` rather than owned
  `String`s, which avoids copying every segment.

Complexity: one pass over the input to split, one push or pop per segment, and
one pass to join — `O(n)` time and `O(n)` space.

</details>
