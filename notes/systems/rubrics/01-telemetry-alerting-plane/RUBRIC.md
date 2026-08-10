# Rubric — 01 The Alerting Plane

> **Spoiler.** Do not open until the final review round is written.

This grades the design against what a strong Staff-level answer contains. It
is not a checklist to score mechanically — a candidate can miss items here
and still be Staff if their reasoning is strong and their tradeoffs are
honest. Grade on **demonstrated signals**, not coverage.

---

## The central tension

**Alert evaluation cannot simultaneously be fast, cheap, and reproducible
under late data.**

- Evaluating on the **stream** is fast and avoids the read amplification,
  but the evaluator holds state that must be rebuilt on restart or
  reassignment, late data arriving hours later has no window to land in,
  and "why did this fire" is answerable only if you deliberately record it.
- Evaluating by **querying the store** is reproducible (the data is there,
  you can re-run it), handles rule changes trivially, and handles late data
  naturally on the next evaluation — but the read amplification is ~2.7x
  the write rate, which is the number that makes the current architecture's
  ruler tier bigger than its ingest tier, and it costs detection latency.
- A **hybrid** gets fast detection and an auditable record, at the price of
  **two systems that can disagree**, and the design must say which one is
  authoritative for an SLA dispute. Most candidates who propose a hybrid do
  not answer that question, and it is the most important question in the
  problem.

There is no dominant answer. A design that picks one and states the cost is
strong. A design that claims to have all three is wrong and should be
challenged with the arithmetic.

The **second** tension, subordinate but real: detection latency measured
from **event time** and from **arrival time** are different SLOs, and the
brief never says which. Under event time, the 6-hour buffered flush is
*permanently* out of SLO and no design can fix it. Under arrival time, the
SLO is achievable but is measuring something the customer doesn't
experience. A candidate who does not notice this has not read the brief.

---

## Ambiguities a strong candidate must catch

Graded in Part 1. **Catching 6+ of the first tier is a Staff signal.**
Catching fewer than 4 caps the grade at Senior regardless of the rest.

### First tier — changes the architecture

| # | Ambiguity | Why it matters |
| --- | --- | --- |
| 1 | **Detection latency measured from event time or arrival time?** | Determines whether late data can ever be in SLO. The single most consequential unstated requirement. |
| 2 | **Must late data retroactively fire alerts?** | Decides whether you need re-evaluation machinery at all, which is a large fraction of the system. |
| 3 | **Must a fired alert be retractable** if late data shows it shouldn't have fired? | Retraction is a much harder problem than firing and implies a different notification contract. |
| 4 | **Is the alert record the source of truth for SLA credits?** | "Twice we had to reconstruct whether an SLA was breached" strongly implies yes, but it is never stated. If yes, reproducibility is a hard requirement, not a nice-to-have, and that rules out pure streaming without a durable evaluation log. |
| 5 | **At-least-once or at-most-once alert emission?** | The notification team owns dedup — but do they? The brief says they own "deduplication," which a candidate should notice means at-least-once is acceptable *if confirmed*. Assuming it silently is weaker than naming it. |
| 6 | **What is "tenant isolation"?** Resource, availability, blast radius, or security? | Costs differ by an order of magnitude. The brief only says "stops one tenant from degrading another," which is resource + availability, not security. |
| 7 | **Is the dashboard/query workload on this system?** | The brief says the ruler queries "the same storage the dashboards query." Whether the new design keeps that coupling is a major architectural fork and the brief does not say. |
| 8 | **Is EU data residency mandatory?** | Legal is "still working on it." A design that can't be regionalised later is a bet. A design that regionalises now pays for it immediately. Either is defensible; not noticing is not. |

### Second tier — changes sizing or operations

| # | Ambiguity |
| --- | --- |
| 9 | Cardinality limits per tenant — who enforces, and what happens on breach? (Reject the series, drop the sample, throttle the agent, bill them?) |
| 10 | Downsampling policy after 7 days — what resolution, and can rules query it? |
| 11 | What is the minimum rule evaluation interval customers may set? (400k rules at 30s vs 15s is a 2x difference in the dominant cost.) |
| 12 | What happens to alert state across a deploy? Is a missed evaluation acceptable? |
| 13 | Agent clock skew — do you trust agent timestamps, stamp on arrival, or both? |
| 14 | Is there a rule-complexity limit? (One tenant's expensive rules are a shared-resource problem and the brief flags exactly this symptom.) |
| 15 | Are customer-hosted agents' buffered data guaranteed durable, or can it be lost? |
| 16 | Growth is stated for samples/s — but series are "growing faster." What is the series growth target? The brief gives 3x for one and not the other. |

---

## Section-by-section

### Part 1 — Requirements & scope

**Senior:** Restates the brief's stated requirements accurately. Lists a few
clarifying questions. Defines out-of-scope as "notification delivery"
(which the brief already told them).

**Staff:** Surfaces the first-tier ambiguities above and resolves each with
a stated assumption and a *reason*. Defines detection latency precisely,
with the clock start named. Distinguishes the kinds of tenant isolation.
Out-of-scope list includes things the brief did *not* hand them — e.g.
"agent-side buffering behaviour," "the dashboard query path," "rule authoring
UX," "notification dedup" — each with what they're assuming someone else
guarantees.

**Red flags:** Accepting "p95 within 45 seconds" without asking from when.
An out-of-scope list that only repeats the brief. No assumptions section at
all.

---

### Part 2 — Back-of-envelope

Recompute all of these independently before commenting. Reference values:

| Quantity | Defensible value | Working |
| --- | --- | --- |
| Peak ingest at 3x | **12M samples/s** | 4M × 3 |
| Bytes/sample stored | **~1.5–2.5 B** | Gorilla/delta-of-delta on timestamps + XOR on floats; order-of-magnitude, not exact |
| Storage/day (today, avg 2M/s) | **~350 GB/day** | 2M × 86,400 = 172.8B samples × ~2 B |
| Storage/day at 3x | **~1 TB/day** | |
| 15 months full resolution at 3x | **~450 TB** | 1 TB × 456 days. Clearly demands downsampling — a candidate who keeps full resolution for 15 months has not done this sum |
| Index memory for 180M series | **~500 GB–1 TB across the fleet** | ~3 KB/active series is a defensible order of magnitude; at 3x series growth this is the number that drives ingester count |
| **Rule evaluations/s** | **13,333/s** | 400,000 rules ÷ 30 s |
| Datapoints read per evaluation | **~800** | ~40 series × 20 points (5-min window at 15 s resolution). The distribution matters far more than the mean — the 50k-rule tenant's rules are not average |
| **Total read rate** | **~10.7M datapoints/s** | 13,333 × 800 |
| **Read : write ratio** | **~2.7 : 1** at today's ingest | This is the number the whole design turns on |
| Read rate at 3x rules | **~32M/s** | If rule count scales with the business |
| Late-data burst | **~10.8M samples/s** for 20 min | 15% of fleet × 2M/s = 300k/s × 4 h = 4.3B samples, drained over 1200 s. **~3.7x peak** on top of normal traffic |
| Cost per active series/month | **~$0.001** implied | $2.1M/yr ÷ 12 ÷ 180M — worth having, because it makes the 3x budget concrete |

**Senior:** Gets ingest volume and storage roughly right. May not compute
the read amplification at all.

**Staff:** Computes the read:write ratio explicitly and **draws the
architectural conclusion from it** — that a query-per-rule design requires
a read tier larger than the write tier, which is exactly the symptom the
director described. Models the late-data burst as a multiple of peak.
Produces a cost model where one line item is identified as dominant. Uses a
distribution rather than a mean for rules-per-tenant and series-per-rule,
because the whole isolation problem lives in the tail.

**Red flags:** Any estimate presented without its arithmetic. Using the mean
rule size when the brief explicitly gives a skewed distribution. Not
noticing that 15 months at full resolution is ~450 TB. Numbers that are off
by more than ~2–3x from the table above without a stated reason — recompute
alongside theirs and show the delta.

---

### Part 3 — API & data model

**Senior:** Sensible write API, rule CRUD, series identified by metric name
plus labels. Partitions by series hash.

**Staff:** Notices that **the ingest partition key and the evaluation
partition key want to be different** — ingest wants even distribution
(hash of series), evaluation wants all series a rule touches to be
co-located (hash of tenant, or of the rule's series selector). Names the
mismatch and says what it costs: either a shuffle, or replication of series
to the evaluators that need them, or accepting a cross-shard read per
evaluation. Handles the 60%-of-a-region tenant explicitly — hash-by-tenant
gives that tenant its own hot shard, so they need a secondary split within
the tenant, or a dedicated unit.

Models **alert state** as a first-class entity (rule id, series/label set,
state, since-when, last-evaluated event time, evaluation id) rather than an
implementation detail. Distinguishes the **rule** from the **alert
instance** — one rule over a label set produces many instances, and the
cardinality of instances is its own scaling problem.

**Red flags:** No mention of alert state at all. Treating "an alert" as a
single object rather than per-series instances. A partition key chosen with
no justification.

---

### Part 4 — High-level architecture

**Senior:** Agents → gateway → Kafka → ingesters → storage; a ruler tier
alongside. Components labelled correctly.

**Staff:** Every component justified by a requirement or a number. The
diagram distinguishes the **fast path** (detection) from the **durable
path** (storage and audit) if they're separate. Shows where alert state
lives and how it's replicated. Marks the tenant boundary explicitly. Shows
where the late-data absorption happens and what it's buffered in.

Strong designs make the **buffer explicit and durable** — a log (Kafka or
equivalent) between the gateway and everything downstream — and can say what
its retention is and why (it must exceed the worst realistic downstream
outage, and it is what lets you replay for re-evaluation).

**Red flags:** A "stream processor" or "alerting service" box with no
internal design — magic components. No buffer between ingest and processing,
which means the late-data flush has nowhere to go. A design where the
dashboards and the rulers still contend for the same query tier, without
acknowledging that this is the problem they were asked to fix.

---

### Part 5 — Deep dives

The two hardest are **A (evaluation)** and **B (late data)**, and they are
coupled — a strong candidate notices that choosing an evaluation model
largely determines the late-data answer. Picking A+B, or A+C, or B+D are all
defensible. Picking C+D avoids the core tension and should be noted as such
in the review, though it is not disqualifying if C and D are done
exceptionally.

#### A — Alert evaluation

**Senior:** Picks streaming or store-based and describes it correctly.
Mentions state needs to be persisted.

**Staff:** Compares all three (stream / store / hybrid) against the read
amplification number, detection latency, reproducibility, and rule-change
cost. Picks one and states the accepted cost. Then handles the parts people
forget:
- **Where alert state lives** and how an evaluator restart doesn't lose
  "this rule has been firing for 4 minutes of a 5-minute `for` clause."
- **Evaluator reassignment mid-window** — a rule moving between workers
  either replays its window or loses its pending state. Both are costs.
- **Rule changes**: in a streaming design, editing a rule invalidates its
  state; can it be rebuilt from the log, and how far back?
- **Deploys**: a rolling restart of the evaluator tier is a gap in coverage
  unless designed for.
- **Fan-out**: one rule producing many alert instances, and what bounds it.

The strongest answers reframe the read-amplification problem: rather than
each rule independently querying, **invert it** — group rules by the series
they select, so one pass over the data serves many rules (a shared
subscription / multi-query model). This is the insight that makes the
numbers work and it is what separates a very strong answer.

#### B — Late and out-of-order data

**Senior:** Mentions the flush and proposes rate limiting the agents or
scaling the ingest tier.

**Staff:** Has an explicit **event-time model**. Names watermarks or an
equivalent, and an **allowed lateness** bound with a stated value and a
reason. Answers the four hard cases directly:
1. Data arrives 4 hours late and shows a breach → fire, record-but-don't-
   notify, or drop? Each is defensible; the answer must be *chosen* and its
   consequence stated (a page for something 4 hours stale is arguably worse
   than no page).
2. An alert fired and late data shows it shouldn't have → retract, annotate,
   or leave it? Retraction has a notification contract implication.
3. The flush itself: **separate the backfill path from the live path** so
   the burst cannot starve real-time ingest. This is the key structural
   move — a shared queue means the flush delays everyone's detection.
   Per-tenant or per-region rate limiting on replay, priority classes on the
   log, or a distinct backfill topic.
4. Clock skew: whether agent timestamps are trusted, and what bounds are
   enforced (reject samples more than N in the future; treat far-past
   samples as backfill).

**Red flags:** No event-time concept at all — treating arrival order as
event order. Proposing to "just buffer it" without saying where or how much.
No bound on lateness, which means unbounded state.

#### C — Multi-tenant isolation

**Staff:** Names a unit of isolation (cell, shard, or per-tenant evaluator
pool) and does the **arithmetic on how many units and what they cost**.
Handles the tenant that doesn't fit in one unit — dedicated capacity, or
splitting within the tenant. Addresses both directions of noisy neighbour:
rule-evaluation cost *and* cardinality/ingest. Has an admission-control
story: per-tenant series limits, rule-complexity limits, evaluation-cost
budgets, and what the tenant experiences when they hit one. Shuffle sharding
is a strong answer if the arithmetic is shown.

**Red flags:** "We'll use Kubernetes namespaces / resource quotas" as the
whole answer. Isolation for the ingest path only, ignoring that the brief's
stated symptom is *rule evaluation* delay.

#### D — Reproducibility and audit

**Staff:** Defines what a durable **evaluation record** contains (rule
version, input window, the actual values evaluated or a reference to them,
the outcome, the evaluation timestamp, the evaluator identity) and its
retention relative to the 15-month contractual window. Addresses
determinism: same inputs must give the same result, which requires versioned
rules and a way to reference the exact data read. Handles the case where
re-running gives a different answer — because late data arrived — and says
what "correct" means then. Notices that this requirement is what makes a
pure streaming design insufficient on its own.

---

### Part 6 — Failure modes

**Senior:** Covers component failures with restart/failover mitigations.

**Staff:** Blast radius quantified ("one shard = 1/40th of series = ~4.5M
series, ~75 tenants"). Detection named as a specific signal, not "we'd
monitor it." **Degraded mode** described as a deliberate product behaviour,
not "it's down."

Must cover a **partition** properly. The strong version: the region holding
alert state partitions from the region holding data. Both sides may believe
they should evaluate; the candidate must say whether they get duplicate
alerts (at-least-once, dedup downstream) or a gap (at-most-once). This is
the same fencing/split-brain question as any leader-elected system, and the
answer must connect to their at-least-once/at-most-once decision from Part 1.

**Fail open vs fail closed** must be answered and justified. Both are
defensible:
- *Fail open* (evaluate on partial data, risk false positives): missing a
  real outage is worse than a spurious page.
- *Fail closed* (don't evaluate on data you know is incomplete): a false
  page during an incident in the alerting system is how you lose trust in
  alerting, and an SLA record built on partial data is worse than an absent
  one.
A candidate who doesn't pick one is dodging.

**Red flags:** No partition scenario. "We'd page someone" as a mitigation.
Not addressing that their own deploy is a coverage gap.

---

### Part 7 — Tradeoffs ledger

**Senior:** Lists decisions. Costs are vague ("more complex," "slightly
higher latency").

**Staff:** Every accepted cost is specific and attributable — *who* pays it
(a tenant, the on-call, the budget, a future migration). At least one row
where the accepted cost is genuinely painful and stated as such. Rows that
reference the numbers from Part 2.

**Red flags:** Fewer than ~8 rows for a system this size. Every cost being
trivial, which means the candidate is presenting choices as free wins.

---

### Part 8 — Evolution

**Senior:** "We'd add more shards."

**Staff:** Names the specific component and limit that binds first at 10x,
with the number. Plausible candidates, any of which is defensible if
argued:
- The **evaluation read path** at ~107M datapoints/s if rules scale with
  ingest — likely the first wall, and it forces the shared-subscription
  inversion if they haven't already done it.
- **Alert instance cardinality** (rules × label-set fan-out), which grows
  faster than rule count.
- The **coordination tier** — whatever assigns rules to evaluators — at
  4M rules.
- **Index memory** at 1.8B active series.
- The **log/buffer** retention cost at 10 TB/day.

Migration story from the current stack must be **incremental and
reversible** — dual-run the new evaluator against the old one, compare
outcomes, and cut over per-tenant. A candidate who proposes a flag-day
cutover for the system that decides SLA credits should be pushed on it hard.

The EU residency question: a strong answer says whether their design
regionalises cleanly and what it costs (per-region evaluation and state,
cross-region rule management, and a global view that becomes harder).

---

### Part 9 — Operations & cost

**Staff:** Dominant cost driver named with a number and a lever. Alerts are
**symptom-based** (detection latency SLO burn, evaluation lag, ingest lag)
rather than cause-based (CPU, disk). The 3am scenario is concrete and
includes what the responder **cannot** fix at 3am — a mature answer
distinguishes "mitigate now" from "fix in the morning."

The "one operation you'd most want during an incident" question is a good
discriminator. Strong answers: *shed a tenant's evaluation load without
dropping their data*, *replay a time range for specific tenants*, or
*disable a specific rule that is generating an alert storm*. A design that
doesn't allow any of these has a gap worth naming in the review.

---

## Grading calibration

| Grade | Profile |
| --- | --- |
| **Below Senior** | Doesn't compute the read amplification. No isolation story. Late data treated as an edge case or ignored. Magic components. |
| **Senior** | Correct architecture for the stated requirements. Reasonable numbers on ingest and storage. Handles component failures. Misses the event-time/arrival-time ambiguity, or handles late data only by scaling. Isolation is "shards." |
| **Borderline Staff** | Computes the read amplification and draws a conclusion from it. Catches 4–5 first-tier ambiguities. One deep dive is genuinely deep, the other is thin. Tradeoffs stated but some costs are soft. Partition scenario present but the split-brain consequence isn't resolved. |
| **Staff** | Drives to the central tension unprompted and picks a side with a stated cost. Catches 6+ first-tier ambiguities including event-time vs arrival-time. Both deep dives are detailed and coupled. Quantifies isolation and cost. Failure modes have degraded behaviours, not just mitigations. Names what breaks at 10x with a number. Explicit about what they are not solving. |
| **Strong Staff** | All of the above, plus: reframes the read-amplification problem rather than sizing around it (shared subscription / multi-query inversion). Separates the backfill path from the live path structurally. Resolves the authoritative-source question for the hybrid explicitly. Migration story is incremental, dual-run, and reversible per tenant. Has a defensible answer to the EU question that doesn't require a redesign. States at least one decision they expect to regret and why they're making it anyway. |

---

## Review guidance

**Round 1:** Recompute every number in Part 2 independently before
commenting. Do not reveal any of this rubric's content. Probe hardest at:
the read amplification conclusion, the event-time definition, where alert
state lives, and what happens to the alert that should have fired 4 hours
ago. Classify claims; do not grade.

**Round 2+:** Push on what was dodged. The most commonly dodged items in
this problem are: which system is authoritative in a hybrid, the fail
open/closed decision, and the migration from the existing stack. Escalate
those rather than introducing new topics.

**Final:** Grade per the table above. The single highest-leverage
improvement is usually one of: "compute the read amplification and let it
drive the architecture," "define what your latency SLO's clock starts on,"
or "your design has two sources of truth and you haven't said which wins."
