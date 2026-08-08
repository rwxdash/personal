# Editorial — Schema Migration Cost

> Spoilers. Read after solving or after genuinely giving up.

## Recognising it

The first thing to rule out is a counting formula. It is natural to guess the
answer is something like "count the positions that differ, multiply by a cost".
Example 5 breaks that: turning `["a","b"]` into `["b"]` is cheapest by
*dropping* the first column and letting the second slide into place, not by
editing anything positionally.

That is the real content of the problem — **which column lines up with which is
not fixed in advance**. It depends on the operations you choose, so the
alignment and the cost have to be decided together. Any approach that fixes the
alignment first and then prices it is wrong.

Now look at what a partial solution looks like. Work left to right through both
layouts. At any moment you have consumed some prefix of `current` and produced
some prefix of `target`. That pair of positions is the *entire* state — how you
got there does not matter, only the cheapest way to have gotten there.

Two positions, each up to 2000, means 4 million states. The required
`O(n · m)` is exactly the number of states, so the intended solution visits each
one once.

## The approach

Let `dp[i][j]` = minimum cost to turn the first `i` columns of `current` into
the first `j` columns of `target`.

**Base cases:**

- `dp[0][0] = 0`
- `dp[i][0] = i * drop_cost` — everything must go
- `dp[0][j] = j * insert_cost` — everything must be created

**Transition** — three moves out of each state, one per operation:

```
dp[i][j] = min( dp[i-1][j]   + drop_cost,      # drop current[i-1]
                dp[i][j-1]   + insert_cost,    # insert target[j-1]
                dp[i-1][j-1] + (0 if names match else retype_cost) )
```

Answer: `dp[n][m]`.

### No special case for expensive retyping

When `retype_cost > drop_cost + insert_cost`, retyping is never worth doing —
but you do not need to detect that. The `min` finds the drop-then-insert route
through the grid on its own, because that route exists as a pair of moves.
Example 2 is exactly this case, and the general code handles it. If you wrote
`if retype_cost > drop_cost + insert_cost`, delete it.

### The matching diagonal — both versions are correct

When the names match, you can either take the `min` of all three branches, or
short-circuit straight to `dp[i-1][j-1]`. Both are correct.

This is worth proving rather than assuming, and the proof is short. When the
names match, neither alternative can beat the free diagonal:

- `dp[i-1][j-1] <= dp[i-1][j] + drop_cost` — take the cheapest solution for
  `(i-1, j)`, then drop its last produced column.
- `dp[i-1][j-1] <= dp[i][j-1] + insert_cost` — insert the extra source column
  first, then apply the cheapest solution for `(i, j-1)`.

Verified empirically as well: the two versions agree on 300 000 random inputs
with independently varied costs. The unconditional `min` is simpler to write,
so the reference uses it.

### The space bound

The transition for row `i` reads only row `i-1` and row `i`. Rows `0 .. i-2`
are never touched again, so keep **two rows** and swap them. That gets you
`O(m)`.

To reach `O(min(n, m))`, swap the two inputs when `current` is shorter — **and
swap `insert_cost` with `drop_cost` when you do.** Swapping the sequences
reverses the direction of the migration, and "dropping from A to reach B" is
"inserting into B to reach A".

This is the nastiest bug in the problem because it is invisible on most inputs:
it only shows up when the two costs differ *and* `current` is the shorter side.
Mutating the reference to swap the sequences without swapping the costs fails 6
of the 11 tests — but every one of those failures needs both conditions to hold
at once, which is why `random_lopsided` deliberately runs each instance in both
orientations.

## Complexity

- **Time:** `O(n · m)` — one constant-time step per state, 4 million at the
  stated sizes.
- **Space:** `O(min(n, m))` — two rolling rows over the shorter sequence.

Why the naive approach fails: exploring every operation sequence without
memoising branches three ways at each step. Measured 0.393 s at n = m = 9,
growing about 5.7× for each additional column. At n = m = 20 that is already
hours; the bound is 2000.

## Common wrong turns

- **Swapping the sequences without swapping the costs.** The subtle one.
- **Assuming an alignment first, then pricing it.** Example 5.
- **Special-casing expensive retyping.** Unnecessary.
- **Forgetting a base case.** Row 0 must be `j * insert_cost` and column 0 must
  be `i * drop_cost`. Leaving either as zeros gives free migrations from
  nothing.
- **Mixing up the rolling rows.** `cur[j-1]` is the current row; `prev[j]` and
  `prev[j-1]` are the previous one. Getting these crossed still produces
  plausible answers on symmetric inputs, which is why the tests use asymmetric
  costs throughout.
- **32-bit arithmetic.** 2000 columns at 10^6 is 2 × 10^9, past a signed 32-bit
  ceiling.
- **Assuming both lists are non-empty.**
- **Assuming column names are distinct.** They repeat, and the tests check it.

## Interview follow-ups

1. **Return the operation list, not just the cost.** Store back-pointers, or
   re-derive by walking backwards from `(n, m)`. Note this needs the full table,
   so the space optimisation and the reconstruction are in direct tension —
   a good thing to make the candidate notice. (Hirschberg's algorithm resolves
   it in `O(min(n,m))` space and `O(n·m)` time; worth knowing it exists.)
2. **Allow moving a column** to a different position, at its own cost. Now the
   problem stops being a clean grid walk, and the discussion turns to why
   arbitrary reordering makes it much harder.
3. **Per-column costs.** Dropping a large column costs more than a small one.
   The recurrence barely changes; a good check that they understood the shape
   rather than memorising the formula.
4. **Only the cost, with a cap.** "Is the migration possible under `X` minutes?"
   Can you answer faster than computing the exact cost? (Banded DP: only cells
   near the diagonal can matter when the budget is small.)
5. **Longest common subsequence connection.** With insert = drop = 1 and retype
   = ∞, this computes `n + m - 2 * LCS`. Seeing that these are the same machine
   with different dials is the real payoff.
6. **Many targets.** One current layout, 100 candidate targets; find the
   cheapest. Anything shareable across the runs? (The first row, and not much
   else — but the reasoning matters more than the answer.)
