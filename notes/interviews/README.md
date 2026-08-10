# Interview Question Bank — Senior/Staff SWE & SRE

Drilled verbal-round questions: networking, Linux, databases, distributed
systems, Kubernetes, observability, and cloud platform work. The kind of
question asked out loud, answered in 2–5 minutes, with no whiteboard.

For algorithm practice see [`../dsas/`](../dsas). For system design see
[`../systems/`](../systems).

**Start at [INDEX.md](INDEX.md)** — all topics with question counts and a
confidence column you fill in yourself.

## How to drill

```bash
cd topics/01-tcp-ip
$EDITOR QUESTIONS.md          # answer out loud, or write into a scratch file
$EDITOR ANSWERS.md            # then grade yourself
```

**Answer out loud.** This is the whole point. These are verbal questions and
the failure mode is not "didn't know" — it's "knew it and produced a
meandering four-minute answer that ended in the wrong place." If you cannot
say it cleanly in one breath's worth of structure, you do not have it yet.

**Answer before opening ANSWERS.md.** Even a bad answer, fully committed to,
teaches more than reading the model answer first. Write it down if you're
tempted to peek — a written answer is gradeable, a thought is not.

**Grade honestly against the three parts of the answer:**

1. Did you get the *mechanism* right?
2. Did you state the *failure mode*? An answer with no failure mode is a
   junior answer regardless of how correct the mechanism is.
3. Could you have answered the follow-ups?

Log the result in [PROGRESS.md](PROGRESS.md). The column that matters is
**Fumbled** — the specific thing you couldn't say. That's your next study
session.

## The tiers

| Tier | Shape | What's being tested | Target length |
| --- | --- | --- | --- |
| 1 | Recall | Do you know the mechanism | 30–60 seconds |
| 2 | Explain / compare | Can you state the tradeoff | 2–3 minutes |
| 3 | Scenario / debug | Do you have a *method* | 3–5 minutes, interactive |

Tier 3 questions are conversations, not monologues. The correct opening move
is almost always to ask a clarifying question and state what you'd measure
first — not to guess a root cause. Practise them that way: say your first
three hypotheses and the single observation that would eliminate two of them.

## Recommended drill order

Do a first pass in index order — the topics build on each other (TCP before
load balancing, load balancing before cloud networking). After that, drill by
weakness, not by order.

A reasonable cadence: one topic per session, ~45 minutes, revisit any topic
where you fumbled more than a third of the questions after a week.

## Cross-references

Several questions here have a full design treatment in `../systems/`. Where
that's the case the answer file says so — the verbal answer and the design
answer are different products, and you should be able to give both.
