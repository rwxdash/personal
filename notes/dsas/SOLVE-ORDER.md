# Recommended Solve Order — Medium tier

21 problems across 20 patterns. This is the order to work them in on a **first
pass**, grouped so that related tools sit next to each other and you can feel
the contrasts between them.

Log each attempt in `PROGRESS.md`.

## How to use this list

**Read `PROBLEM.md` before you look at the path.** Directory names give the
pattern away, which is the one thing you most want to work out for yourself.
Open the file directly from this list rather than browsing the tree.

**Give yourself a time box.** Target times below are for a first attempt.
When you hit the box, open `HINTS.md` — one level at a time, not all four.
Stopping and taking a hint is not failure; grinding for two hours on a
recognition you do not have yet teaches nothing.

**Run the tests before you believe you are done.** They include randomised
cross-checks against a slow reference and inputs sized so a too-slow solution
fails. A passing run is a real signal.

**Read the editorial even when you solve it.** Each one ends with the follow-up
questions an interviewer would actually ask next, and those are usually where
the real conversation goes.

---

## Phase 1 — Warm up (3 problems, ~25 min each)

Short implementations. The point is to get the loop going: read the spec, write
the code, run the tests.

| # | Problem | What you are practising |
| --- | --- | --- |
| 1 | [object-key-canonical](problems/string-manipulation/01-object-key-canonical/PROBLEM.md) | Reading a spec precisely. No algorithm to discover — every case is stated, and the difficulty is handling all of them. Start here because every later problem needs this habit. |
| 2 | [cidr-aggregation](problems/bit-manipulation/01-cidr-aggregation/PROBLEM.md) | One idea (XOR to find where two numbers diverge), a handful of lines. |
| 2b | [extent-cover](problems/bit-manipulation/01b-extent-cover/PROBLEM.md) | **Optional second rep.** Same rung as #2 and the inverse operation — #2 rounds a range outward to one block, this fills a range inward with many. Take it if #2 needed hints; skip it if it did not. |
| 3 | [conveyor-pick-window](problems/sliding-window/01-conveyor-pick-window/PROBLEM.md) | The most classic shape in the whole set. Two pointers over a contiguous range with running state. |

## Phase 2 — Array techniques (4 problems, ~35 min each)

Four different ways to avoid re-scanning an array. Done together, the
differences between them become obvious.

| # | Problem | What you are practising |
| --- | --- | --- |
| 4 | [host-consolidation](problems/two-pointers/01-host-consolidation/PROBLEM.md) | Pointers converging from both ends. Also your first exchange argument — proving a greedy rule rather than guessing it. |
| 5 | [settlement-runs](problems/prefix-sums/01-settlement-runs/PROBLEM.md) | Rewriting "sum of a range" as a difference of running totals. Note *why* a sliding window is not merely slow here but wrong. |
| 6 | [peak-dominance](problems/monotonic-stack/01-peak-dominance/PROBLEM.md) | Discarding data you can prove you will never need again. |
| 7 | [gateway-peak-concurrency](problems/intervals-sweep-line/01-gateway-peak-concurrency/PROBLEM.md) | Turning intervals into events and sweeping. The tie-break at equal timestamps is the whole problem. |

## Phase 3 — Graphs (4 problems, ~40 min each)

The four core graph tools, back to back, so you see what each is actually for.

| # | Problem | What you are practising |
| --- | --- | --- |
| 8 | [config-propagation](problems/bfs/01-config-propagation/PROBLEM.md) | Breadth-first for shortest distance, started from many sources at once. |
| 9 | [resource-ancestry](problems/dfs/01-resource-ancestry/PROBLEM.md) | The thing depth-first gives you that breadth-first cannot: entry/exit times that turn a tree into intervals. |
| 10 | [pipeline-critical-path](problems/topological-sort/01-pipeline-critical-path/PROBLEM.md) | Ordering by dependency, with the computation folded into the same pass. |
| 11 | [mesh-link-failures](problems/union-find/01-mesh-link-failures/PROBLEM.md) | The biggest single leap in the tier: a structure that only merges, applied to a problem that only splits. Expect to need a hint. |

## Phase 4 — Search and structures (3 problems, ~40 min each)

| # | Problem | What you are practising |
| --- | --- | --- |
| 12 | [compaction-windows](problems/binary-search-on-answer/01-compaction-windows/PROBLEM.md) | Searching the space of *answers* rather than the data. A genuinely different move from binary search over a sorted array. |
| 13 | [sharded-audit-log](problems/heaps-k-way-merge/01-sharded-audit-log/PROBLEM.md) | A heap holding one candidate per source. Cost scaling with the query, not the corpus. |
| 14 | [route-prefix-match](problems/tries/01-route-prefix-match/PROBLEM.md) | Sharing work across strings with a common start. |

## Phase 5 — Dynamic programming (3 problems, ~45 min each)

Same idea in three shapes: a line, a grid, a tree.

| # | Problem | What you are practising |
| --- | --- | --- |
| 15 | [maintenance-windows](problems/dp-1d/01-maintenance-windows/PROBLEM.md) | The simplest DP here. Get the recurrence and the index arithmetic right before adding dimensions. |
| 16 | [schema-migration-cost](problems/dp-2d/01-schema-migration-cost/PROBLEM.md) | Two sequences, one table, then rolling it down to two rows. |
| 17 | [audit-sampling](problems/dp-on-trees/01-audit-sampling/PROBLEM.md) | Two states per node, computed children-first — without recursion. |

## Phase 6 — The heavy ones (3 problems, ~50 min each)

Most moving parts. Leave these until the rest feels comfortable.

| # | Problem | What you are practising |
| --- | --- | --- |
| 18 | [batch-queue-order](problems/greedy-exchange-argument/01-batch-queue-order/PROBLEM.md) | Short to write, but the point is deriving *why* the rule is right. Do not skip the derivation — that is the interview. |
| 19 | [autoscaler-stable-windows](problems/monotonic-queue/01-autoscaler-stable-windows/PROBLEM.md) | Two monotonic deques plus bulk counting. The most simultaneous bookkeeping in the tier. |
| 20 | [byte-budget-cache](problems/lru-lfu-design/01-byte-budget-cache/PROBLEM.md) | The most code, and a specification with several rules that all have to hold at once. Closest to a real design round. |

---

## Second pass

Once you have been through all 20, go again **in shuffled order** and time
yourself. The first pass teaches the tools; the second pass trains recognition,
which is the part interviews actually test. Working them grouped by family a
second time would hand you the answer before you read the question.

Pick a random order, or sort `PROGRESS.md` by the ones that took longest.

## When you want more repetition

Most patterns have one problem. If a pattern does not stick after one attempt,
that is normal — the fix is more problems at the *same* rung, not harder ones.
Ask for extra Mediums on the patterns you want to drill.

They are filed as **letter siblings** of the rung they belong to: the second
Medium for `bit-manipulation` is `01b-extent-cover`, not `02`. That keeps `02`
free to mean Hard, so the ladder still reads top to bottom.

The Hard and Advanced rungs are deliberately still empty. They are worth
generating once you have attempted most of this tier, so they can target
whatever turns out to be shaky.
