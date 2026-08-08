# Pipeline Critical Path

**Difficulty:** Medium–Hard
**ID:** `topological-sort-01-pipeline-critical-path`

## Scenario

Your CI system runs a build pipeline made of `n` jobs, numbered `0 .. n-1`.
Each job takes a fixed number of milliseconds to run once it starts, and each
job may declare other jobs that must **finish** before it may **start**.

Two extra facts about the platform:

- **Workers are unlimited.** Any number of jobs may run at the same instant, so
  a job starts the moment it is allowed to, never later.
- **Jobs have an availability time.** Job `i` pulls an external artifact that
  does not exist until `ready[i]` milliseconds into the run (a nightly image
  mirror, a licence server that comes up late, a scheduled data export). Job `i`
  cannot start before `ready[i]` no matter how quickly its dependencies clear.

The pipeline starts at time 0. You want the **makespan**: the earliest instant
at which every job has finished.

A misconfigured pipeline may contain a dependency cycle, in which case no
schedule exists at all and the platform rejects it.

## Task

Implement:

```python
pipeline_makespan(durations: List[int], ready: List[int], deps: List[Tuple[int, int]]) -> Optional[int]
```

- `durations[i]` — how long job `i` runs, in ms.
- `ready[i]` — the earliest instant job `i` may start, in ms.
- `deps` — pairs `(a, b)` meaning **job `a` must finish before job `b`
  starts**.

Return the makespan, or `None` if the dependency graph contains a cycle.

Formally, job `i` starts at
`start[i] = max(ready[i], max(finish[a] for every (a, i) in deps))`
(the inner max is 0 when job `i` has no dependencies), finishes at
`finish[i] = start[i] + durations[i]`, and the makespan is
`max(finish[i])` over all jobs.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `durations` | `List[int]` | `1 <= n <= 200_000`; each value in `0 ..= 10^9` |
| `ready` | `List[int]` | same length as `durations`; each value in `0 ..= 10^9` |
| `deps` | `List[Tuple[int, int]]` | `0 <= len(deps) <= 400_000`; each endpoint in `0 .. n-1` |

`deps` may contain **duplicate pairs** (the same edge declared twice in
config) and **self-loops** `(a, a)`. A self-loop is a job that must finish
before itself, which is a cycle. The graph may be disconnected, and jobs with
a duration of 0 are legal (config-only gate jobs).

The makespan can reach roughly `2 * 10^14`, so a 64-bit result type is
required.

## Output

`Optional[int]` — the makespan in ms, or `None` if the dependencies are cyclic.

## Required complexity

- **Time:** `O(n + m)` where `m = len(deps)`.
- **Space:** `O(n + m)`.

Note also that `n` reaches 200 000 and the dependency graph may be a single
chain that deep, so any approach whose call depth grows with the chain will
exhaust the default Python recursion limit.

## Examples

### Example 1 — a chain

```
durations = [3, 2, 4]
ready     = [0, 0, 0]
deps      = [(0, 1), (1, 2)]
```

**Answer:** `9`

Job 0 runs from 0 to 3. Job 1 cannot start until job 0 finishes, so it runs 3
to 5. Job 2 runs 5 to 9. Nothing overlaps, so the makespan is the sum, 9.

### Example 2 — full parallelism

```
durations = [3, 2, 4]
ready     = [0, 0, 0]
deps      = []
```

**Answer:** `4`

With no dependencies and unlimited workers, all three jobs start at time 0 and
run concurrently. The pipeline is done when the slowest one is, at 4 ms — not
at 9. The makespan is the maximum, not the sum.

### Example 3 — a diamond

```
durations = [1, 5, 2, 1]
ready     = [0, 0, 0, 0]
deps      = [(0, 1), (0, 2), (1, 3), (2, 3)]
```

**Answer:** `7`

Job 0 finishes at 1. Jobs 1 and 2 then run in parallel: job 1 covers 1 to 6,
job 2 covers 1 to 3. Job 3 needs *both*, so it waits for the later of them (job
1, at 6) and runs 6 to 7. The answer is driven entirely by the heaviest route
through the graph, `0 → 1 → 3`; the lighter route `0 → 2 → 3` is free.

### Example 4 — availability dominates

```
durations = [1, 1]
ready     = [0, 10]
deps      = [(0, 1)]
```

**Answer:** `11`

Job 0 runs 0 to 1. Job 1's dependency is satisfied at 1, but its artifact does
not exist until 10, so it idles and runs 10 to 11. An availability time can
push the finish far past what the dependencies alone would suggest, and that
delay then propagates to everything downstream of job 1.

### Example 5 — a cycle

```
durations = [1, 1]
ready     = [0, 0]
deps      = [(0, 1), (1, 0)]
```

**Answer:** `None`

Job 0 waits on job 1 and job 1 waits on job 0, so neither can ever start.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/topological-sort/01-pipeline-critical-path/EDITORIAL.md`.
