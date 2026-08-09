# Problem Index

Self-contained DSA practice problems, Medium → Hard/Advanced, in Python and
Rust. Every pack ships a spoiler-free statement, runnable tests, graduated
hints, and an editorial kept outside the problem tree.

**New here? Start with [SOLVE-ORDER.md](SOLVE-ORDER.md)** — a recommended
order through the Medium tier with time boxes and what each problem is for.

**Fill in the Status column yourself** — it is left blank on purpose. Track
attempts and timings in `PROGRESS.md`.

## How to work a problem

```bash
cd problems/<pattern>/<NN>-<slug>

# Python
$EDITOR python/challenge.py     # implement the function
python3 python/challenge.py     # runs every test, prints "All tests passed."

# Rust
$EDITOR rust/src/lib.rs         # implement the function
cd rust && cargo test
```

Read `PROBLEM.md` first. It contains no spoilers — no pattern names, no
approach. If you stall, open `HINTS.md` and expand **one** level at a time;
level 1 only restates the problem, level 4 gives the full algorithm in prose.
Open the editorial only after solving or genuinely surrendering.

The test files are the specification. They include randomised cross-checks
against a brute-force oracle and large inputs sized so that the naive approach
cannot pass — if your solution is correct but too slow, the large cases will
tell you.

Directory names reveal the pattern, so if you want a cold read, work from this
index's Theme column and open `PROBLEM.md` directly rather than browsing the
tree.

## Problems

| ID | Pattern | Difficulty | Theme | Status |
| --- | --- | --- | --- | --- |
| [sliding-window/01-conveyor-pick-window](problems/sliding-window/01-conveyor-pick-window/PROBLEM.md) | Sliding window | Medium | Warehouse conveyor picking | |
| [monotonic-queue/01-autoscaler-stable-windows](problems/monotonic-queue/01-autoscaler-stable-windows/PROBLEM.md) | Monotonic queue | Medium–Hard | Autoscaler CPU time series | |
| [topological-sort/01-pipeline-critical-path](problems/topological-sort/01-pipeline-critical-path/PROBLEM.md) | Topological sort + DAG DP | Medium–Hard | CI/CD pipeline scheduling | |
| [union-find/01-mesh-link-failures](problems/union-find/01-mesh-link-failures/PROBLEM.md) | Union-find | Medium–Hard | Service mesh chaos testing | |
| [two-pointers/01-host-consolidation](problems/two-pointers/01-host-consolidation/PROBLEM.md) | Two pointers (converging) | Medium | VM fleet consolidation | |
| [prefix-sums/01-settlement-runs](problems/prefix-sums/01-settlement-runs/PROBLEM.md) | Prefix sums | Medium | Payments ledger settlement | |
| [monotonic-stack/01-peak-dominance](problems/monotonic-stack/01-peak-dominance/PROBLEM.md) | Monotonic stack | Medium | CDN bandwidth dashboard | |
| [intervals-sweep-line/01-gateway-peak-concurrency](problems/intervals-sweep-line/01-gateway-peak-concurrency/PROBLEM.md) | Sweep line | Medium | API gateway pool sizing | |
| [binary-search-on-answer/01-compaction-windows](problems/binary-search-on-answer/01-compaction-windows/PROBLEM.md) | Binary search on answer | Medium | Log-store compaction |  |
| [heaps-k-way-merge/01-sharded-audit-log](problems/heaps-k-way-merge/01-sharded-audit-log/PROBLEM.md) | Heap k-way merge | Medium | Sharded audit log paging |  |
| [bfs/01-config-propagation](problems/bfs/01-config-propagation/PROBLEM.md) | Multi-source BFS | Medium | Datacentre config rollout |  |
| [dfs/01-resource-ancestry](problems/dfs/01-resource-ancestry/PROBLEM.md) | DFS entry/exit times | Medium | Cloud IAM hierarchy |  |
| [dp-1d/01-maintenance-windows](problems/dp-1d/01-maintenance-windows/PROBLEM.md) | 1D dynamic programming | Medium | Maintenance scheduling |  |
| [dp-2d/01-schema-migration-cost](problems/dp-2d/01-schema-migration-cost/PROBLEM.md) | 2D dynamic programming | Medium | Schema migration downtime |  |
| [dp-on-trees/01-audit-sampling](problems/dp-on-trees/01-audit-sampling/PROBLEM.md) | Tree dynamic programming | Medium | Compliance audit sampling |  |
| [tries/01-route-prefix-match](problems/tries/01-route-prefix-match/PROBLEM.md) | Trie | Medium | API gateway route table |  |
| [string-manipulation/01-object-key-canonical](problems/string-manipulation/01-object-key-canonical/PROBLEM.md) | String manipulation | Medium | Object key canonicalisation |  |
| [greedy-exchange-argument/01-batch-queue-order](problems/greedy-exchange-argument/01-batch-queue-order/PROBLEM.md) | Greedy + exchange argument | Medium | Batch job queue ordering |  |
| [bit-manipulation/01-cidr-aggregation](problems/bit-manipulation/01-cidr-aggregation/PROBLEM.md) | Bit manipulation | Medium | Firewall CIDR aggregation |  |
| [bit-manipulation/01b-extent-cover](problems/bit-manipulation/01b-extent-cover/PROBLEM.md) | Bit manipulation | Medium | Object store aligned extents |  |
| [lru-lfu-design/01-byte-budget-cache](problems/lru-lfu-design/01-byte-budget-cache/PROBLEM.md) | LRU cache design | Medium | Object store response cache |  |

## Coverage matrix

Every pattern in the catalog × the three ladder rungs. This is the generation
tracker: **filled cells are done and verified, blank cells are the backlog.**

- ✅ = pack exists, tests verified against a reference, manifest recorded in
  `MANIFESTS/`
- `—` = not generated yet
- `n/a` = no honest problem exists at this difficulty for this pattern, so the
  cell will never be filled. Excluded from the slot totals. See
  "Difficulty floors" below for the reason on each.

Rungs follow the numbering convention: `01` is the easiest Medium of a pattern,
and difficulty increases from there. Three rungs per pattern is the target
ladder, not a hard cap — a pattern that deserves a fourth gets `04`.

A **letter suffix** means a sibling at the same rung, not a step up: `01b` is a
second Medium for extra reps on a pattern that did not stick the first time. It
fills no new slot, so it does not move the count below.

**Filled: 20 / 72 slots (28%)**, holding **21 problems**. By rung: `01` **20/20 — the Medium tier is complete**, `02` 0/26, `03` 0/26.
By difficulty label: Medium 18, Medium–Hard 3, Hard 0, Advanced 0.

> ⚠️ The entire Hard and Advanced half of the ladder is empty. Everything so
> far sits on the bottom rung.

### Generation order: breadth-first

**Fill the whole Medium tier first, then come back for Hard, then Advanced.**
Pattern *recognition* is the skill being trained, and it is only exercised when
you do not already know which tool applies. Generating a full ladder per
pattern defeats that — solving `sliding-window/02` right after `01` hands you
the recognition step for free. A complete Medium tier also lets the solve order
be shuffled, so practice stays interleaved rather than blocked.

### Difficulty floors

Per `CLAUDE.md`, a pattern is never stretched to fit a rung: when the
difficulty and the pattern do not fit, the problem either **shifts to the
closest honest rung** or the cell is marked **`n/a`**. Six `01` cells are `n/a`,
so the slot total above is 72 rather than 78.

| Pattern | Floor | Why |
| --- | --- | --- |
| `dp-bitmask` | `02` | Subset-state enumeration is the entire difficulty |
| `segment-tree-bit` | `02` | A Medium segment-tree problem is just prefix sums in costume |
| `kmp-rolling-hash` | `02` | Failure-function reasoning is Hard-rung by nature |
| `backtracking-pruning` | `02` | Plain backtracking is Easy; the pruning is the problem |
| `reservoir-sampling` | `02` | Standalone it is trivia; fold it into a streaming/design problem |
| `dp-on-dags` | `02` | Medium DAG-DP ground is already taken by `topological-sort/01`; the distinct material (path counting, probability propagation, multi-dimensional state) is Hard |

More cells will be marked `n/a` as they are reached — the `03` Advanced rung in
particular is unlikely to be honest for every pattern. They are recorded when
a batch actually hits them, not guessed at in advance.

Medium tier: **20 patterns, 20 done — complete.** Next rung up is `02` (Hard), entirely empty.

### Graphs & trees

| Pattern | `01` Medium | `02` Hard | `03` Advanced |
| --- | --- | --- | --- |
| `bfs` | ✅ [config-propagation](problems/bfs/01-config-propagation/PROBLEM.md) | — | — |
| `dfs` | ✅ [resource-ancestry](problems/dfs/01-resource-ancestry/PROBLEM.md) | — | — |
| `topological-sort` | ✅ [pipeline-critical-path](problems/topological-sort/01-pipeline-critical-path/PROBLEM.md) | — | — |
| `union-find` | ✅ [mesh-link-failures](problems/union-find/01-mesh-link-failures/PROBLEM.md) | — | — |
| `dp-on-dags` | n/a | — | — |
| `dp-on-trees` | ✅ [audit-sampling](problems/dp-on-trees/01-audit-sampling/PROBLEM.md) | — | — |

### Arrays & windows

| Pattern | `01` Medium | `02` Hard | `03` Advanced |
| --- | --- | --- | --- |
| `sliding-window` | ✅ [conveyor-pick-window](problems/sliding-window/01-conveyor-pick-window/PROBLEM.md) | — | — |
| `two-pointers` | ✅ [host-consolidation](problems/two-pointers/01-host-consolidation/PROBLEM.md) | — | — |
| `prefix-sums` | ✅ [settlement-runs](problems/prefix-sums/01-settlement-runs/PROBLEM.md) | — | — |
| `monotonic-stack` | ✅ [peak-dominance](problems/monotonic-stack/01-peak-dominance/PROBLEM.md) | — | — |
| `monotonic-queue` | ✅ [autoscaler-stable-windows](problems/monotonic-queue/01-autoscaler-stable-windows/PROBLEM.md) | — | — |
| `intervals-sweep-line` | ✅ [gateway-peak-concurrency](problems/intervals-sweep-line/01-gateway-peak-concurrency/PROBLEM.md) | — | — |

### Search & selection

| Pattern | `01` Medium | `02` Hard | `03` Advanced |
| --- | --- | --- | --- |
| `binary-search-on-answer` | ✅ [compaction-windows](problems/binary-search-on-answer/01-compaction-windows/PROBLEM.md) | — | — |
| `heaps-k-way-merge` | ✅ [sharded-audit-log](problems/heaps-k-way-merge/01-sharded-audit-log/PROBLEM.md) | — | — |

### Dynamic programming

| Pattern | `01` Medium | `02` Hard | `03` Advanced |
| --- | --- | --- | --- |
| `dp-1d` | ✅ [maintenance-windows](problems/dp-1d/01-maintenance-windows/PROBLEM.md) | — | — |
| `dp-2d` | ✅ [schema-migration-cost](problems/dp-2d/01-schema-migration-cost/PROBLEM.md) | — | — |
| `dp-bitmask` | n/a | — | — |

### Strings

| Pattern | `01` Medium | `02` Hard | `03` Advanced |
| --- | --- | --- | --- |
| `string-manipulation` | ✅ [object-key-canonical](problems/string-manipulation/01-object-key-canonical/PROBLEM.md) | — | — |
| `kmp-rolling-hash` | n/a | — | — |
| `tries` | ✅ [route-prefix-match](problems/tries/01-route-prefix-match/PROBLEM.md) | — | — |

### Structures & design

| Pattern | `01` Medium | `02` Hard | `03` Advanced |
| --- | --- | --- | --- |
| `segment-tree-bit` | n/a | — | — |
| `lru-lfu-design` | ✅ [byte-budget-cache](problems/lru-lfu-design/01-byte-budget-cache/PROBLEM.md) | — | — |

### Combinatorial, greedy & bit

| Pattern | `01` Medium | `02` Hard | `03` Advanced |
| --- | --- | --- | --- |
| `backtracking-pruning` | n/a | — | — |
| `greedy-exchange-argument` | ✅ [batch-queue-order](problems/greedy-exchange-argument/01-batch-queue-order/PROBLEM.md) | — | — |
| `bit-manipulation` | ✅ [cidr-aggregation](problems/bit-manipulation/01-cidr-aggregation/PROBLEM.md)<br>✅ [extent-cover](problems/bit-manipulation/01b-extent-cover/PROBLEM.md) *(`01b`)* | — | — |
| `reservoir-sampling` | n/a | — | — |

### Overlap notes

Kept here so future batches do not accidentally duplicate ground already
covered under a different pattern heading:

- **`dp-on-dags`** — resolved: `01` is `n/a`.
  `topological-sort/01-pipeline-critical-path` is already a DP over a
  topological order, so a Medium here would duplicate it. The genuinely
  distinct material (counting paths, probability propagation, DP with a second
  state dimension) is Hard, so this pattern enters at `02`. If that `02` ends
  up hinging on the ordering rather than the DP, file it as
  `topological-sort/02` instead and mark this row `n/a` outright.
- **`monotonic-stack` vs `monotonic-queue`** — split deliberately. The queue
  rung is window-extreme work; the stack rung should be
  next-greater-element / histogram-shaped, not another sliding window.
- **`bfs` / `dfs`** — resolved at `01`: both are filled by
  traversal-plus-a-twist, never textbook traversal. `bfs/01` is multi-source
  with absorbing nodes (the multi-source part is what makes it O(n+m) rather
  than O(S·(n+m))); `dfs/01` uses entry/exit times for interval containment,
  which is the one thing DFS gives that BFS structurally cannot. Higher rungs
  should keep that discipline — 0-1 BFS, state-augmented nodes, bridges/SCC.

### Maintenance

`CLAUDE.md` already requires updating `INDEX.md` after generating problems;
that means **both** the Problems table above and this matrix, in the same pass.
A pack is only marked ✅ once its manifest exists in `MANIFESTS/` — the
manifest is the verification record, so it is the thing that makes the claim
true.

To audit for drift, compare the two sets directly (counting checkmarks does
not work — ✅ also appears in prose on this page):

```bash
diff <(ls MANIFESTS/ | sed 's/\.md$//' | sort) \
     <(grep -o 'problems/[a-z0-9-]*/[0-9][0-9][a-z]*-[a-z0-9-]*/PROBLEM.md' INDEX.md \
       | sed 's|problems/||; s|/PROBLEM.md||; s|/|-|' | sort -u) \
  && echo "OK: INDEX.md and MANIFESTS/ agree"
```

(The `[a-z]*` after the digits is what picks up sibling rungs like `01b`.)

Currently clean: 21 linked problems, 21 manifests, no difference.
