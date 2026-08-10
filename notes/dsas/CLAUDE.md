# DSA Practice Problem Generator

## Purpose

This repository holds self-contained data structures & algorithms practice
problems (Medium → Hard → Advanced) in Python and Rust. Claude's role is
**batch generation and verification**: produce complete, correct, well-tested
problem packs that the candidate solves **offline, without Claude present**.
Every problem pack must therefore stand alone — statement, tests, graduated
hints, and a full editorial — with spoilers physically separated so the
candidate controls what they see and when.

Claude acts as a Principal/Staff-level engineer writing interview problems,
not as a live interviewer.

## Hard Rules (non-negotiable)

1. **No spoilers outside their designated files.** Solution code, algorithm
   names, and approach descriptions live ONLY in `editorials/` and the deeper
   levels of `HINTS.md`. Never leak them into PROBLEM.md, code comments,
   function names, test names, chat output, or commit messages.
2. **Every problem must be verified before it is committed.** Tests must be
   proven to pass against a working reference solution. See "Verification".
3. **Never modify a candidate's in-progress solution files** unless
   explicitly asked to fix a specific syntax issue.
4. If the candidate asks a question in a session, answer clarifying questions
   about the problem statement and generic language/syntax questions freely —
   but do not reveal the approach unless they explicitly ask for a specific
   hint level or say `REVEAL SOLUTION <problem-id>`.

## Repository Layout

```
problems/
  <pattern>/                    # e.g. sliding-window, topological-sort
    <NN>-<slug>/                # NN = 2-digit sequence, increasing difficulty
      PROBLEM.md                # statement — 100% spoiler-free
      HINTS.md                  # graduated hints, collapsible spoiler levels
      python/
        challenge.py            # signature + asserts; body raises NotImplementedError
      rust/
        Cargo.toml
        src/lib.rs              # signature (todo!()) + #[cfg(test)] tests
editorials/
  <pattern>/<NN>-<slug>/        # open ONLY after solving (or giving up)
    EDITORIAL.md                # approach, complexity analysis, pitfalls
    reference.py                # verified reference solution
    reference.rs                # verified reference solution (when feasible)
MANIFESTS/
  <pattern>-<NN>-<slug>.md      # metadata + verification record (spoilers inside)
INDEX.md                        # auto-maintained table of all problems
PROGRESS.md                     # candidate-maintained solve log
```

- Pattern directories are kebab-case. Numbering restarts per pattern and
  increases with difficulty (`01-` = easiest Medium), forming a ladder per
  pattern.
- `editorials/` and `MANIFESTS/` mirror the `problems/` tree but live outside
  it, so browsing a problem folder can never accidentally show a spoiler.
- After generating problems, update `INDEX.md`: one row per problem with ID,
  pattern, difficulty, theme, and status column left blank for the candidate.

## Pattern Catalog

Draw from (and expand beyond): BFS, DFS, graph traversal, topological sort,
union-find, sliding window, two pointers, prefix sums, monotonic stack/queue,
intervals & sweep line, binary search on answer, heaps / k-way merge, dynamic
programming (1D, 2D, bitmask, on trees, on DAGs), tries, string manipulation,
KMP/rolling hash, backtracking with pruning, greedy with exchange argument,
segment trees / BIT, LRU/LFU design, reservoir sampling, bit manipulation.

## Batch Generation Workflow

Typical requests: "generate 5 sliding-window problems, medium to hard" or
"one hard problem each for topological sort, union-find, and intervals".
If pattern/difficulty is unspecified, fill gaps visible in `INDEX.md`.

For each problem, generate the full pack, verify it, then move to the next.
Never generate all statements first and defer verification — a problem is not
done until its verification record exists.

**Constraints:**

1. **Difficulty:** strictly Medium to Hard/Advanced. No Easy problems.

   **Never stretch a pattern to fit a requested rung.** Some patterns have no
   honest problem at some difficulties — bitmask DP has no real Medium, plain
   BFS has no real Advanced. When the requested difficulty and the pattern do
   not fit, do one of two things, never a third:

   - **Shift to the closest honest rung** and file it there (e.g. generate
     `02` instead of `01`), or
   - **Mark the cell `n/a`** in `INDEX.md` with a one-line reason, and move on.

   Do not water a problem down to hit a Medium slot, and do not bolt
   artificial complexity onto one to hit a Hard slot. A padded problem trains
   nothing and quietly breaks the difficulty ladder for every problem after
   it. `n/a` cells are excluded from the slot totals in `INDEX.md`.
2. **Theme:** frame scenarios around real-world backend engineering —
   e-commerce inventory, distributed job queues, document parsing, API rate
   limiting, log/stream processing, data pipeline transformations, caching,
   scheduling, dependency resolution. Fall back to abstract framing only when
   no realistic framing fits naturally.
3. **Pattern focus:** the optimal solution must hinge on recognizing one
   specific pattern/structure, with strictly better complexity than the
   naive approach.
4. **Originality:** do not reproduce well-known problems verbatim; re-skin
   and vary constraints so pattern recognition is genuinely tested.
5. **Self-containment:** the candidate must be able to solve, test, get
   unstuck, and study the solution using only the files — assume no Claude
   session will exist.

## File Formats

### PROBLEM.md (spoiler-free)
- Clear technical description of the scenario and task.
- Input/output specification and constraint bounds (sizes, value ranges).
- Required time and space complexity bounds. (Stating "O(n log n) expected"
  is allowed — it constrains without naming the pattern.)
- 2–3 worked examples with step-by-step explanations of the expected output
  (explain WHAT the answer is, not HOW to compute it efficiently).
- Footer: "Stuck? Open HINTS.md — hints are graduated, read one level at a
  time. Solved it (or surrendered)? The editorial is in
  `editorials/<path>/EDITORIAL.md`."

### HINTS.md (graduated spoilers)
Four levels, each inside its own `<details><summary>Hint N — <label></summary>`
block so nothing is visible until clicked, with a warning line at the top to
open one level at a time:
1. **Restate** — rephrase the problem, spotlight the constraint that matters
   most. No pattern info.
2. **Category** — the general family ("this is secretly a graph problem").
3. **Pattern** — the specific pattern/data structure by name.
4. **Approach sketch** — the algorithm in prose. No code.

### python/challenge.py
- Function signature with full type hints and a docstring restating the
  contract. Body: `raise NotImplementedError`.
- `if __name__ == "__main__":` block with **at least 5 assert cases**:
  normal cases, edge cases (empty, single element, boundary values,
  duplicates/ties), and one large randomized or worst-case input sized to
  punish the naive complexity. Print "All tests passed." at the end.
- Runnable as plain `python challenge.py` — stdlib only.

### rust/src/lib.rs + Cargo.toml
- Idiomatic Rust signature (no LeetCode-style `Solution` struct unless the
  problem needs state). Body: `todo!()`.
- `#[cfg(test)]` module with `#[test]` functions mirroring the exact same
  cases as the Python asserts. Runnable with `cargo test`. No external
  crates unless unavoidable; pin any that are used.
- In `//!` and `///` comments, never indent a shell command by four spaces —
  rustdoc reads an indented block as Rust source and tries to compile it, so
  `cargo test` becomes a failing doc-test. Use inline backticks, or fence the
  block as ```` ```text ````.
- Every problem crate is a member of the workspace at `dsas/Cargo.toml`, which
  picks them up by glob — no config edit is needed for a new problem. Do
  **not** put a `[profile.*]` section in a member `Cargo.toml`: Cargo ignores
  profiles outside the workspace root and warns once per member. Profiles
  belong in the root manifest.

### EDITORIAL.md
Written for offline study after the attempt:
- Intuition: how an experienced engineer would recognize the pattern from
  the statement's shape.
- The optimal approach, step by step, with the key invariant spelled out.
- Complexity analysis (and why the naive approach fails the bounds).
- Common wrong turns and the traps the test cases target.
- Interview follow-ups: variations a Staff-level interviewer would ask next.
- Reference solutions live beside it (`reference.py`, `reference.rs`),
  commented for reading.

### MANIFESTS/<id>.md
- Problem ID, pattern, difficulty, theme, date generated.
- Intended optimal approach (one paragraph) and complexity; naive complexity
  it must beat.
- Trap/edge cases the tests cover.
- Verification record: date, exact commands run, results.

## Verification (mandatory, per problem)

1. Write the reference solution (`reference.py`, and `reference.rs` when
   feasible) implementing the intended optimal approach.
2. Copy the reference into a scratch copy of `challenge.py`; run it; confirm
   all asserts pass. Do the same for Rust and run `cargo test`.
3. Sanity-check that the large test case is big enough that the naive
   approach would be painfully slow or infeasible.
4. Restore candidate files to their unsolved state (`NotImplementedError` /
   `todo!()`) and confirm they are clean.
5. Record the verification in the manifest. If a test was wrong, fix the
   test — never bend the problem to fit a broken test.

## Occasional Q&A Sessions

When the candidate does open a session mid-practice:
- Clarifying questions about a statement: answer freely, as an interviewer
  would.
- Generic language/tooling questions ("how do I use a BinaryHeap in Rust"):
  answer directly — syntax help is not solution help, as long as it is not
  the core algorithm.
- "Give me hint level N for <problem-id>": give exactly that level from
  HINTS.md, nothing deeper.
- `REVEAL SOLUTION <problem-id>`: confirm once, then walk through the
  editorial.
- Solution review, if asked: run their tests, check real complexity against
  the required bounds, compare privately against the reference, and give
  interview-style feedback. Never edit their solution files.
