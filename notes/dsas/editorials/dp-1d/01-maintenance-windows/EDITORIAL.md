# Editorial — Maintenance Windows

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

Two observations, in order.

**First: greedy does not work.** Before reaching for anything complicated, test
the simple rule "repeatedly take the largest legal slot". On
`value = [4, 5, 4]` with `cooldown = 1`, greedy grabs the 5, which blocks both
neighbours, for a total of 5. Taking the two 4s gives 8. So the biggest
available thing can cost you two smaller things that were worth more together.

Ruling greedy out early matters, because it tells you the choices genuinely
interact and you have to weigh whole arrangements rather than pick locally.

**Second: the interaction is shallow.** Standing at slot `i`, you have two
options — skip it, or take it. If you take it, you collect `value[i]` and
everything else must be at slot `i - cooldown - 1` or earlier. And here is the
part that makes the problem easy once you see it: you do **not** need to know
*which* earlier slots you used. You only need the best total achievable up to
that cut-off. The entire history collapses into one number.

That collapse — many possible pasts, one number that summarises all of them —
is what makes this a dynamic programming problem rather than a search.

## The approach

Let `best[i]` be the maximum total value using only slots `0 .. i`.

```
best[i] = max( best[i-1],                                # skip slot i
               value[i] + best[i - cooldown - 1] )       # take slot i
```

The take branch applies only when slot `i` is not blacked out, and `best[j]`
for `j < 0` is `0` — nothing chosen yet.

Answer: `best[n-1]`, or `0` when the input is empty.

### Why this is O(n) and not O(n²)

The natural worry is that "the best total up to the cut-off" requires scanning
backwards for a maximum. It does not, because **`best` is non-decreasing**:
`best[i] >= best[i-1]` always, since skipping is a legal option that carries
the previous total forward unchanged.

So `best[i - cooldown - 1]` is already the largest value achievable at or
before that point. One array lookup, no scan.

That property is worth proving to yourself rather than assuming, because the
whole complexity bound rests on it. It also fails in variants — if values could
be negative, `best` would no longer be monotone and this shortcut would break.

### The index that everyone gets wrong

`i - cooldown - 1`, not `i - cooldown`.

The rule is that two chosen slots `i < j` need `j - i > cooldown`. So if you
take slot `i`, the previous one must be at `i - cooldown - 1` or earlier.
Changing that `- 1` fails 9 of the 12 tests, including all four worked
examples. If the problem's examples pass but nothing else does, check this
first.

Two boundary cases that confirm the indexing is right, and which the general
code handles with no special cases:

- **`cooldown = 0`** gives `j = i - 1`, so every non-blackout slot can be
  taken and the answer is their sum.
- **`cooldown >= n`** makes `j` always negative, so the answer is the single
  largest non-blackout value.

### Blackout slots still get an entry

A blacked-out slot cannot be taken, but `best[i]` must still be written — it
carries `best[i-1]` forward. Skipping the write leaves the next slot reading a
stale or uninitialised value.

## Complexity

- **Time:** `O(n)`. One pass, constant work per slot.
- **Space:** `O(n)` for the array. You can drop to `O(cooldown)` with a sliding
  buffer, but the problem does not ask for it.

Why brute force fails: trying every subset is `2^n`. Measured at 42 seconds for
just **24 slots**, doubling with each additional slot. At n = 40 it would run
for years; n goes to 200 000.

## Common wrong turns

- **Greedy by value.** Fails on `[4, 5, 4]`.
- **`i - cooldown` instead of `i - cooldown - 1`.** The dominant bug.
- **Treating `j < 0` as "cannot take".** Taking slot `i` when nothing before it
  is reachable is legal — the base is `0`, not "invalid".
- **Skipping the array write for blackout slots.**
- **Scanning backwards for the maximum.** Correct but `O(n²)`; unnecessary once
  you notice `best` is non-decreasing.
- **32-bit arithmetic.** 200 000 slots at 10^9 reaches 2 × 10^14.
- **Special-casing `cooldown = 0` or `cooldown >= n`.** Both fall out of the
  general formula. If you needed a special case, the indexing is probably off.
- **Returning a negative number.** Choosing nothing is always allowed, so the
  answer is at least 0.

## Interview follow-ups

1. **Return which slots to use, not just the total.** Store a back-pointer at
   each step, or re-derive by walking the array backwards comparing
   `best[i]` against `best[i-1]`.
2. **A cap on the number of runs.** At most `k` maintenance windows. Adds a
   second dimension: `best[i][j]` with `j` runs used, so `O(nk)`.
3. **Per-slot cooldowns.** Slot `i` has its own settling period. The recurrence
   still works, but `best` monotonicity is the thing to re-check.
4. **Negative values.** Some maintenance makes things worse. Now `best` is no
   longer non-decreasing, the single-lookup shortcut breaks, and you need a
   running maximum (or a monotonic structure) to keep it linear. This is the
   sharpest follow-up, because it targets exactly the property the solution
   quietly relied on.
5. **Circular schedule.** Slot `n-1` and slot `0` are adjacent (a repeating
   weekly rota). Standard trick: solve twice, once forbidding the first slot
   and once forbidding the last, and take the better.
6. **Why is this not just "pick alternate slots"?** On a uniform array it is,
   which is why `large_alternating` exists — but any variation in the values
   breaks that intuition immediately.
