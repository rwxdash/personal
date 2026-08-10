# Hints — Maintenance Windows

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Pick a subset of array positions, no two closer together than
> `cooldown + 1`, skipping forbidden positions, maximising the sum of the
> values picked.

The first thing to test is whether a simple rule works, because if one did the
problem would be over. Try "repeatedly take the largest remaining legal slot":

```
value = [5, 1, 8, 4],  cooldown = 1
```

Greedy takes 8 (slot 2), which blocks slots 1 and 3, leaving only slot 0 for a
total of 13. That happens to be right. Now try:

```
value = [4, 5, 4],  cooldown = 1
```

Greedy takes 5 (slot 1), which blocks both neighbours, total 5. But taking
slots 0 and 2 gives 8. **Greedy is wrong.** Taking the biggest thing available
can cost you two smaller things that together were worth more.

So you need to actually weigh alternatives. And the alternatives are nested:
whether slot 3 is worth taking depends on what you did earlier, which depends
on what you did before that.

The question to sit with: when you are standing at slot `i` deciding what to
do, how much of the past do you actually need to remember?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

Walk the slots left to right. At each slot you face exactly **two** choices:

- **Skip it.** Then the best you can do is whatever the best was through the
  previous slot.
- **Take it** (if it is not a blackout). Then you collect `value[i]`, and you
  must add the best achievable from slots that are far enough back to be
  compatible — everything from slot `i - cooldown - 1` and earlier.

That second branch is the key. Once you take slot `i`, you do not need to know
*which* earlier slots were chosen, only the best total achievable up to a
certain cut-off point. All the detail of the earlier arrangement collapses into
a single number.

That collapse is what makes the problem tractable: the answer for each position
depends on a small, fixed amount of earlier information, not on the entire
history. So you can compute the answer for slot 0, then slot 1, then slot 2,
each in constant time, storing one number per slot.

The remaining question is the "best achievable up to a cut-off point" part. If
you scan backwards to find it each time, each slot costs `O(n)` and the whole
thing is `O(n²)`. Is there something about the sequence of best-so-far numbers
that makes the lookup free?

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**One-dimensional dynamic programming over slot index.**

Define `best[i]` = the maximum total value obtainable using only slots
`0 .. i`. Then:

```
best[i] = max( best[i-1],                                   # skip slot i
               value[i] + best[i - cooldown - 1] )          # take slot i
```

where the second branch only applies when slot `i` is not blacked out, and
`best[j]` is treated as `0` for any `j < 0` (nothing chosen yet).

The answer is `best[n-1]`, or `0` when `n == 0`.

Now the lookup question from the last level. Notice that `best` is
**non-decreasing**: `best[i]` is at least `best[i-1]`, because skipping is
always allowed. That means `best[i - cooldown - 1]` is *already* the best value
achievable at or before that cut-off — you never need to scan backwards for a
maximum, because the array's own monotonicity guarantees the single entry you
want is the largest one available.

That is what turns the natural `O(n²)` formulation into `O(n)`. It is worth
proving to yourself that `best` really is non-decreasing before relying on it,
because the whole complexity bound rests on that one property.

Watch the indexing. `i - cooldown - 1` goes negative for small `i`, and
`cooldown` can be larger than the entire array.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

```
if n == 0: return 0

best = [0] * n
for i in range(n):
    skip = best[i-1] if i > 0 else 0

    take = 0
    if not blackout[i]:
        j = i - cooldown - 1
        take = value[i] + (best[j] if j >= 0 else 0)

    best[i] = max(skip, take)

return best[n-1]
```

Points to be careful about:

- **`best[j]` for `j < 0` is `0`**, not "invalid". Taking slot `i` when nothing
  before it is reachable is perfectly legal — it just collects `value[i]` alone.
  Getting this wrong usually shows up as an index error or as a wrong answer on
  small inputs.
- **A blackout slot still gets a `best` value.** It is `best[i-1]` — the best
  achievable so far, carried forward. Do not skip writing the entry, or the
  next slot reads garbage.
- **`cooldown` may exceed `n`.** Then `j` is always negative, and the answer
  becomes the largest single non-blackout value, which the code above produces
  without a special case.
- **`cooldown = 0`** gives `j = i - 1`, so taking every non-blackout slot is
  allowed and the answer is their sum. Also handled without a special case — a
  good check that your indexing is right.
- **Use 64-bit numbers.** 200 000 slots at 10^9 each reaches 2 × 10^14.
- The answer is never negative, since choosing nothing is always an option.

Complexity: one pass, constant work per slot, so `O(n)` time and `O(n)` space
for the array. (With care you can drop to `O(cooldown)` space by keeping only
a sliding buffer of recent entries, but that is not required.)

</details>
