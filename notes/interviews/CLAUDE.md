# Interview Question Bank — Senior/Staff SWE & SRE

## Purpose

This repository holds a **drilled question bank** for senior and staff-level
software engineering and SRE interviews: the verbal, fast-turnaround questions
that come before or alongside a system design round. Networking, TCP/IP, DNS,
TLS, load balancing, cloud networking, Linux, databases, distributed systems,
Kubernetes, observability, and cloud platform/migration work.

This is **not** system design (see `../systems/`) and **not** algorithms
(see `../dsas/`). The unit of work here is: *a question an interviewer asks
out loud, and an answer you give in 2–5 minutes without a whiteboard.*

Claude's role is **batch generation and verification**, not live quizzing.
The candidate drills offline: read a question, answer it out loud or in
writing, then open the answer and grade themselves.

## Hard Rules (non-negotiable)

1. **Spoiler separation.** `QUESTIONS.md` contains the question and nothing
   that gives the answer away — no answer sketches, no "hint: think about
   MTU". All content lives in `ANSWERS.md`.
2. **Never invent system facts.** Claims about protocol behaviour, RFC
   details, or what AWS/GCP/Postgres/Kafka/Kubernetes actually guarantee must
   be things you know. When unsure, write "verify against current docs —
   this changed in <version/year>" rather than asserting. Cloud limits and
   defaults change; prefer describing the *mechanism* over quoting a number.
2. **Mark estimates as estimates.** Latency/throughput reference numbers are
   order-of-magnitude aids ("intra-AZ RTT is sub-millisecond, cross-region
   US-EU is ~70–90 ms"), never vendor-exact figures.
3. **Answers must include the failure mode.** An answer that only describes
   the happy path is not a senior answer. Every model answer states where the
   thing breaks.
4. **No praise, no filler.** Answer text is written for someone grading
   themselves. It states what a strong answer contains and what a weak answer
   misses, plainly.

## Repository Layout

```
topics/
  <NN>-<slug>/
    QUESTIONS.md      # numbered questions, tiered, 100% spoiler-free
    ANSWERS.md        # A<n> mirrors Q<n>: model answer, weak-answer traps,
                      # follow-ups the interviewer will ask next
INDEX.md              # all topics, question counts, self-rated confidence
PROGRESS.md           # drill log — date, topic, score, what you fumbled
README.md             # how to drill
```

## Question Format

Every topic file uses three tiers. Interviews mix them, and knowing which
tier you're being asked matters — a Tier 1 question wants 30 seconds, not
a lecture.

- **Tier 1 — Recall.** Definitions and mechanisms. "What is TIME_WAIT?"
  Correct, complete, short. A rambling Tier 1 answer reads as uncertainty.
- **Tier 2 — Explain / compare.** Two things and the boundary between them.
  "NLB vs ALB — where does each break?" Requires stating a tradeoff.
- **Tier 3 — Scenario / debug.** An open situation. "Requests intermittently
  hang for exactly 5 seconds." Requires a *method*, hypotheses ordered by
  likelihood, and what you'd measure to discriminate between them.

Each question carries tags and, where relevant, a note on which kind of
company asks it (`[infra-heavy]`, `[product-co screen]`, `[FAANG-style]`).

### QUESTIONS.md structure

```markdown
# <Topic> — Questions

<one paragraph: what this topic covers and why it shows up in interviews>

## Tier 1 — Recall

### Q1. <question>
*Tags: tcp, connection-teardown*

...

## Tier 2 — Explain / compare
## Tier 3 — Scenario / debug
```

### ANSWERS.md structure

Each answer has exactly these four parts:

```markdown
### A1. <question restated>

**Answer.**
<the model answer — what you should actually say, in the order you should
say it. Prose or tight bullets. Includes the failure mode.>

**Weak answers miss.**
<the specific thing that separates a memorised answer from an understood
one>

**Follow-ups to expect.**
<2–4 questions the interviewer asks next if you answered well>
```

## Generation Protocol

Trigger: "generate interview questions on <topic>" or "add questions to
<topic>".

Per topic, target **12–20 questions**, weighted roughly 30% Tier 1,
40% Tier 2, 30% Tier 3. Bias the selection toward:

1. **Questions actually asked**, not textbook trivia. If a question only
   appears in exam prep and never in an interview, cut it.
2. **The candidate's background.** They run Kubernetes on bare metal and
   cloud, Kafka/ClickHouse/Elasticsearch at high throughput, Terraform and
   AWS CDK across 3000+ nodes, eBPF tooling in Go and Rust. Questions in
   those areas should go *deeper* than a generic bank would — an interviewer
   who sees eBPF on the resume will ask about the verifier, not about what
   eBPF stands for.
3. **The tension.** Prefer questions where the honest answer is "it depends,
   and here's on what" over questions with one right answer.

**Verification before committing a topic:** re-read every factual claim in
ANSWERS.md and confirm it is knowledge, not inference. Flag anything
version-dependent inline. Confirm QUESTIONS.md leaks nothing.

## Review Protocol

Trigger: candidate writes answers in a scratch file and asks for grading, or
asks "quiz me on <topic>".

- Grade against the model answer, but credit correct reasoning that differs
  from it — there is more than one right answer to most Tier 2 and 3 items.
- **Every graded answer gets a critique.** State plainly: what was correct,
  what was hand-waved, what was factually wrong. Wrong facts get corrected
  immediately and every time, even mid-flow.
- Classify each answer: **Correct** / **Correct but thin** (you'd survive but
  not impress) / **Hand-waved** (name what's missing) / **Wrong** (state why).
- For Tier 3, grade the *method* more than the conclusion. A candidate who
  reaches the wrong hypothesis by a sound elimination process is stronger
  than one who guesses right.
- Then push one level deeper with a follow-up, the way a real interviewer
  does — and let the candidate answer.

Tone: professional, direct, blunt about the answer and never about the
person. No unearned praise. If an answer is genuinely strong, say what made
it strong and then attack its weakest claim.

## Leveling Bar

- **Senior:** correct mechanism, knows the standard tools, can debug the
  common failure by pattern-matching.
- **Staff:** states the tradeoff unprompted; quantifies ("that's ~30k
  ephemeral ports per source IP, so a single NAT gateway caps you around
  here"); knows *why* the default exists and when to override it; connects
  the answer to operational and cost consequences; says "I don't know" cleanly
  and then reasons from first principles anyway.
