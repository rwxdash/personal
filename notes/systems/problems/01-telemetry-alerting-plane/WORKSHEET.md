# Worksheet — 01 The Alerting Plane

Fill this in as you work. Part 1 comes **before** any architecture. The rest
can be written alongside `DESIGN.md`, but everything here should be answered
before you call a round ready.

Write in prose, tables, and ASCII/mermaid diagrams. No code.

---

## Part 1 — Requirements & scope

**Complete this before you draw anything.**

### 1.1 Ambiguities in the brief

The brief is deliberately underspecified. List every question you'd ask the
requirements owner, and — for each — either the answer you'd expect to get
or the assumption you're making in its absence.

| # | Question | Why it changes the design | Your assumption |
| --- | --- | --- | --- |
| 1 | | | |

> Aim for at least 8. A design that resolves fewer than that has probably
> not read the brief closely enough. Some of these have real answers you can
> ask for in a session; for others the correct response is "you decide,
> state your assumption."

### 1.2 Functional requirements

What must the system do? Be specific enough that someone could disagree with
you.

### 1.3 Non-functional requirements

| Property | Target | Source (stated / derived / assumed) |
| --- | --- | --- |
| Detection latency | | |
| Ingest availability | | |
| Alert correctness (false negatives) | | |
| Alert correctness (false positives) | | |
| Durability of raw samples | | |
| Reproducibility / auditability | | |
| Tenant isolation | | |
| Cost ceiling | | |

For **detection latency**, state precisely what the clock starts on. This is
the single most consequential definition in the whole design.

For **tenant isolation**, say which kind you mean: resource, availability,
security, or blast radius. They cost different amounts.

### 1.4 Explicitly out of scope

What are you **not** solving, and why? Name at least four things. For each,
say what you're assuming someone else guarantees.

---

## Part 2 — Back-of-envelope

Show the arithmetic. A number without its derivation is not an estimate.

### 2.1 Ingest

| Quantity | Calculation | Result |
| --- | --- | --- |
| Peak samples/s today | | |
| Peak samples/s at 3x | | |
| Bytes/sample on the wire | | |
| Bytes/sample stored (compressed) | | |
| Storage/day at full resolution | | |
| Storage for 15 months (state your downsampling policy) | | |
| Active series → index memory | | |
| Ingest shards required | | |

### 2.2 The alerting read path

This is the calculation the whole design turns on. Work out how many
datapoints per second your alert evaluation must read, and compare it to
the write rate.

| Quantity | Calculation | Result |
| --- | --- | --- |
| Rule evaluations/s | | |
| Series touched per evaluation (assume a distribution, not a mean) | | |
| Datapoints read per evaluation | | |
| **Total datapoint reads/s** | | |
| **Read : write ratio** | | |

Then state what that ratio means for your architecture.

### 2.3 The late-data burst

| Quantity | Calculation | Result |
| --- | --- | --- |
| Fraction of fleet that can partition together | | |
| Buffered volume after a 4-hour partition | | |
| Flush rate if drained over 20 minutes | | |
| Peak multiple over normal ingest | | |
| What you do about it | | |

### 2.4 Cost

Build a rough cost model. You don't need vendor-exact pricing — you need to
know which line item dominates and by how much.

| Component | Driver | Rough share of spend |
| --- | --- | --- |
| | | |

State the **cost per active series per month** and the **cost per rule per
month** your design implies. If you can't, you can't defend the budget.

---

## Part 3 — API & data model

### 3.1 Core operations

The write path, the rule CRUD path, and whatever the evaluation path
exposes. Signature-level, not code.

### 3.2 Entities

Series, samples, rules, alert instances, alert state, evaluation records.
What identifies each one?

### 3.3 Partitioning

- What is the partition key for ingest, and why?
- What is the partition key for rule evaluation, and why?
- Are they the same? If not, what does the mismatch cost you?
- How does a tenant with 60% of a region's series fit into this?

---

## Part 4 — High-level architecture

A diagram (ASCII or mermaid) plus one sentence per component explaining why
it exists. If you can't justify a component in one sentence, you don't need
it yet.

Show explicitly:
- The path a sample takes from agent to durable storage.
- The path from a sample to an alert firing.
- Where alert state lives.
- Where the tenant boundary is.

---

## Part 5 — Deep dives

**Pick two.** Design them properly — mechanisms, data structures, failure
behaviour, and the numbers that justify the choice. A deep dive that could
be a paragraph in Part 4 is not a deep dive.

**Candidate A — Alert evaluation.**
Streaming evaluation over the ingest path, versus querying the store,
versus a hybrid. What does each do to your read amplification, your
detection latency, your reproducibility, and your ability to change a rule?
Where does alert state live, how does it survive a restart, and what happens
to a rule mid-window when its evaluator moves?

**Candidate B — Late and out-of-order data.**
What is your event-time model? Watermarks, allowed lateness, or something
else? What happens to an alert that should have fired 4 hours ago and whose
data has just arrived — does it fire, fire retroactively, get recorded but
not notified, or get dropped? What happens to an alert that *did* fire and
whose late data now shows it shouldn't have? How do you absorb the flush
without taking down ingest for everyone?

**Candidate C — Multi-tenant isolation.**
How do you stop the 50,000-rule tenant from delaying the 40-rule tenant?
How do you stop a cardinality explosion in one tenant from consuming the
ingest tier? What is your unit of isolation, and what does a tenant that
outgrows it do? Include the arithmetic on how many isolation units you need
and what they cost.

**Candidate D — Reproducibility and audit.**
The director wants to answer "did this alert fire when it should have" with
evidence. What do you record, where, and for how long? What makes an
evaluation deterministic enough to re-run? What do you do when re-running it
gives a different answer than the original?

Say which two you picked and, in one line, why those two are the hardest
part of this problem.

---

## Part 6 — Failure modes

For each, give blast radius, detection, mitigation, and degraded mode.
**At least one must be a partition scenario.**

| Failure | Blast radius | How you detect it | Mitigation | Degraded behaviour |
| --- | --- | --- | --- | --- |
| An ingest shard is lost | | | | |
| The region hosting alert state partitions away | | | | |
| A tenant's cardinality explodes 50x in 10 minutes | | | | |
| The late-data flush arrives | | | | |
| Rule evaluation falls behind by 10 minutes | | | | |
| An agent's clock is 3 hours wrong | | | | |
| Your own deploy is the incident | | | | |

Then answer: **when your system is degraded, do alerts fail open or fail
closed?** Justify it. There is a defensible answer either way and you must
pick one.

---

## Part 7 — Tradeoffs ledger

Every significant decision, as "chose X over Y, accepting cost Z." Be
specific about the cost — "slightly more complex" is not a cost.

| Decision | Chosen | Rejected alternative | Cost accepted | Who feels the cost |
| --- | --- | --- | --- | --- |
| | | | | |

Aim for at least 10 rows. If every row's accepted cost is trivial, you are
not being honest with yourself about the tradeoffs.

---

## Part 8 — Evolution

- **What breaks first at 10x?** Name the specific component and the specific
  limit, with the number.
- **What breaks second?**
- What is the migration story from the current Prometheus-derived stack to
  yours? Can it be done incrementally, and what is the rollback at each
  step?
- Which of your decisions would you regret at 10x, and what would you do
  differently if you knew today that 10x was certain?
- What does the EU data residency question do to your design if legal comes
  back and says it's mandatory?

---

## Part 9 — Operations & cost

- **Dominant cost driver**, with the number, and the single change that
  would most reduce it.
- **Top 3 alerts** on this system itself. For each: what fires it, why it's
  a page rather than a ticket, and what the responder does first.
- **What does a 3am page look like?** Write the actual scenario: what fired,
  what the on-call sees, what they check, what they can do about it at 3am
  versus what has to wait for morning.
- What is the **capacity headroom** you're carrying, and what does it cost?
- What is the **one operation** you'd most want to be able to perform during
  an incident, and does your design allow it?
