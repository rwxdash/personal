# Hints — Schema Migration Cost

> ⚠️ **Open one level at a time.** Each level reveals strictly more than the
> last. Give the current level a real attempt before expanding the next one.

<details>
<summary><b>Hint 1 — Restate</b></summary>

Stripped of framing:

> Given two sequences and three weighted operations (insert, delete, replace),
> find the cheapest way to turn the first sequence into the second. Matching
> elements at the same position cost nothing.

The trap to disarm first: **there is no simple counting formula.** It is
tempting to think the answer is something like "count the columns that differ,
multiply by a cost". Example 5 kills that idea — `["a","b"]` into `["b"]` is
cheapest by *dropping* the first column and letting the second slide into
place, not by editing columns positionally. Which columns line up with which
depends on the operations you choose, so alignment and cost have to be decided
together.

Now think about what a solution actually looks like. Process both layouts left
to right. At any moment you have consumed some prefix of `current` and produced
some prefix of `target`. That pair of positions is the entire state — how you
got there does not matter, only the cheapest way to have gotten there.

So: how many such states are there, and what are the possible moves out of one?

</details>

<details>
<summary><b>Hint 2 — Category</b></summary>

The state is a **pair** of positions: how much of `current` is consumed, and
how much of `target` is produced. With `n` and `m` up to 2000, that is 4
million states — small enough to visit them all, and the required `O(n · m)`
time is exactly the number of states.

From a state `(i, j)` there are only three moves, one per operation:

- **Drop** `current[i]`: consume one column from `current`, produce nothing.
  Pay `drop_cost`, land at `(i+1, j)`.
- **Insert** `target[j]`: produce one column without consuming anything. Pay
  `insert_cost`, land at `(i, j+1)`.
- **Handle both at once**: consume `current[i]` and produce `target[j]`. Free
  if the names already match, otherwise pay `retype_cost`. Land at
  `(i+1, j+1)`.

Every legal migration is some path through this grid of states from
`(0, 0)` to `(n, m)`, and its cost is the sum of the moves along the way. You
want the cheapest path.

Since each state's answer depends only on states with smaller indices, you can
fill the grid in order and never revisit anything.

One thing to be deliberate about: even when the names match, should you *only*
consider the free diagonal move? Think about whether one of the other two moves
could ever beat it, given that the costs are arbitrary.

</details>

<details>
<summary><b>Hint 3 — Pattern</b></summary>

**Two-dimensional dynamic programming over the two sequence prefixes.**

Let `dp[i][j]` = minimum cost to turn the first `i` columns of `current` into
the first `j` columns of `target`.

Base cases:

- `dp[0][0] = 0` — nothing to do.
- `dp[i][0] = i * drop_cost` — everything must be dropped.
- `dp[0][j] = j * insert_cost` — everything must be inserted.

Transition:

```
dp[i][j] = min( dp[i-1][j]   + drop_cost,
                dp[i][j-1]   + insert_cost,
                dp[i-1][j-1] + (0 if current[i-1] == target[j-1] else retype_cost) )
```

Answer: `dp[n][m]`.

Taking the minimum of all three branches unconditionally — including when the
names match — is always correct, and it is the simplest thing to write.

If you instead shortcut ("they match, so take the free diagonal and move on"),
that is *also* correct, and it is worth understanding why rather than guessing.
When the names match, the diagonal can never be beaten by the other two moves:

- `dp[i-1][j-1] <= dp[i-1][j] + drop_cost` — take the cheapest solution for
  `(i-1, j)` and drop its last produced column.
- `dp[i-1][j-1] <= dp[i][j-1] + insert_cost` — insert the extra source column,
  then apply the cheapest solution for `(i, j-1)`.

So both alternatives are at least as expensive. Either version passes; write
whichever you find clearer.

Notice also that you never need a special case for "retyping is more expensive
than dropping plus inserting". The `min` finds the drop-then-insert route
through the grid by itself. If you wrote an `if retype_cost > drop_cost +
insert_cost` branch, delete it.

The remaining problem is the **space bound**. A full `n × m` table is 4 million
entries, and the requirement is `O(min(n, m))`. Look at the transition again
and ask which rows it actually touches.

</details>

<details>
<summary><b>Hint 4 — Approach sketch</b></summary>

The transition for row `i` reads only row `i-1` and row `i` itself. Rows
`0 .. i-2` are never touched again. So keep **two rows**, not the whole table,
and swap them as you go.

To make the space `O(min(n, m))` rather than `O(m)`, swap the two inputs when
`current` is shorter — with the costs of insert and drop exchanged, since
"dropping from A to reach B" is "inserting into B to reach A". Then iterate
over the longer sequence in the outer loop.

```
if len(current) < len(target):
    current, target = target, current
    insert_cost, drop_cost = drop_cost, insert_cost

n, m = len(current), len(target)

prev = [j * insert_cost for j in range(m + 1)]        # row i = 0

for i in range(1, n + 1):
    cur = [i * drop_cost] + [0] * m                   # column j = 0
    for j in range(1, m + 1):
        same = current[i-1] == target[j-1]
        cur[j] = min(
            prev[j]     + drop_cost,
            cur[j-1]    + insert_cost,
            prev[j-1]   + (0 if same else retype_cost),
        )
    prev = cur

return prev[m]
```

Things to get right:

- **`cur[j-1]` is the current row, `prev[j]` and `prev[j-1]` are the previous
  row.** Mixing these up is the most common error in a rolled-up table, and it
  usually still produces plausible-looking answers on symmetric inputs.
- **Initialise column 0 of every row** to `i * drop_cost` before the inner
  loop, and row 0 to `j * insert_cost`. Forgetting either gives zeros where a
  real cost belongs.
- **The swap must exchange the costs too.** Swapping the sequences without
  swapping insert and drop silently computes the wrong direction — and it only
  shows up when the two costs differ *and* `current` is shorter than `target`.
- **Either list may be empty**, which the base cases handle.
- **Use 64-bit numbers.** 2000 × 10^6 is 2 × 10^9, past a 32-bit signed
  ceiling, and intermediate sums go higher.

Complexity: `O(n · m)` time — one constant-time step per state — and
`O(min(n, m))` space for the two rows.

</details>
