# Batch Queue Order

**Difficulty:** Medium
**ID:** `greedy-exchange-argument-01-batch-queue-order`

## Scenario

A single-threaded batch worker has a queue of jobs to run tonight. It runs them
one at a time, in whatever order you choose, with no gaps and no interruptions.

Job `i` takes `duration[i]` seconds. Each job also has a **cost rate**
`rate[i]`: the number of currency units it costs your company for every second
that job remains unfinished. A report that blocks the morning finance run has a
high rate; a housekeeping job nobody is waiting for has a low one.

If a job finishes at time `t` seconds after the run begins, it contributes
`rate[i] * t` to the total cost. The worker starts at time 0, so the first job
in your order finishes at its own duration, the second at the sum of the first
two durations, and so on.

Choose the order that minimises the **total cost across all jobs**.

## Task

Implement:

```python
min_total_cost(duration: List[int], rate: List[int]) -> int
```

Return the minimum achievable total cost.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `duration` | `List[int]` | `0 <= n <= 100_000`; each value in `1 ..= 10_000` |
| `rate` | `List[int]` | same length as `duration`; each value in `1 ..= 10_000` |

The total can reach about `5 * 10^17`, so 64-bit arithmetic is required.

## Output

`int` — the minimum total cost. `0` when there are no jobs.

## Required complexity

- **Time:** `O(n log n)`.
- **Space:** `O(n)`.

Trying every order is `n!`. At `n = 12` that is already 479 million
permutations, and `n` goes to 100 000.

## Examples

### Example 1

```
duration = [3, 1]
rate     = [1, 1]
```

**Answer:** `5`

Both jobs cost the same per second, so only the durations matter.

| Order | Finish times | Cost |
| --- | --- | --- |
| job 0 then job 1 | 3, 4 | 1·3 + 1·4 = 7 |
| job 1 then job 0 | 1, 4 | 1·1 + 1·4 = **5** |

Running the short job first is better. It finishes early *and* it only delays
the long job by 1 second, whereas running the long job first delays the short
one by 3.

### Example 2 — rates matter too

```
duration = [3, 1]
rate     = [10, 1]
```

**Answer:** `34`

| Order | Finish times | Cost |
| --- | --- | --- |
| job 0 then job 1 | 3, 4 | 10·3 + 1·4 = **34** |
| job 1 then job 0 | 1, 4 | 1·1 + 10·4 = 41 |

Now the *longer* job goes first, because its cost rate is ten times higher.
Sorting by duration alone is wrong.

### Example 3 — neither duration nor rate alone decides

```
duration = [2, 3]
rate     = [3, 5]
```

**Answer:** `30`

| Order | Finish times | Cost |
| --- | --- | --- |
| job 0 then job 1 | 2, 5 | 3·2 + 5·5 = 31 |
| job 1 then job 0 | 3, 5 | 5·3 + 3·5 = **30** |

Job 1 is both longer *and* higher-rate, and it should go first. Note that job 0
has the shorter duration and job 1 has the higher rate — the two simple sorting
rules disagree here, and the one that sorts by rate happens to win. Example 1
is a case where sorting by rate alone gives no information at all.

The margin is narrow (30 against 31), which is worth noticing: the two orders
are close because the two jobs are similar. Both jobs deliver roughly the same
cost per unit of time occupied, so it barely matters which goes first.

### Example 4 — a single job

```
duration = [7]
rate     = [4]
```

**Answer:** `28`

One job, finishing at time 7, at a rate of 4: `4 · 7 = 28`. With one job there
is nothing to order.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/greedy-exchange-argument/01-batch-queue-order/EDITORIAL.md`.
