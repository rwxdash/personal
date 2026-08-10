# Rubric — 06 The Observability Engine

> **Spoiler.** Do not open until the final review round is written.

Graded differently from 01–05 because the format differs. Weight:
**scoping 25% · design 40% · live extension 25% · questions 10%.** A
technically excellent design that cannot be extended live scores worse here
than a good design that can.

---

## The central tension

**Live tail, indexed search, and unbounded untrusted volume have
incompatible requirements, and the volume is set by someone who is not you.**

- **Live tail** wants sub-second end-to-end, streaming, no durability, no
  indexing. It is a pipe.
- **Search over 30 days** wants durability, indexing, compaction, and
  columnar or inverted structures. It is a database.
- **The volume is adversarial by accident.** No agent to configure, no
  schema, no sampling you can request. A single customer can produce a third
  of the platform's total volume with one bad deploy, at any hour, and it is
  a legitimate use of the product.

The naive design — one pipeline, everything indexed, one store serving both
tail and search — fails on cost before it fails on anything else, and the
arithmetic shows it clearly.

**The insight the problem is built around: tail and search are different
systems that share an ingest path, and the shared path must be the cheapest
possible thing that can absorb an adversarial burst.**

**Secondary tension, and the one that decides the business:** full-text
indexing 130 TB/day costs more than the product earns. The design fork is
*index the labels and scan the content* (Loki-shaped) versus *index
everything* (Elasticsearch-shaped). Neither is free — label-only indexing
gives the customer a materially worse search experience, and a candidate who
picks it must say what the customer loses and why that's the right trade.

**Third, quieter but high-signal:** the most valuable log lines in the whole
system are the last ones before a container dies, and they are the ones most
likely to be sitting in a buffer when the process is `SIGKILL`ed. Designing
for that specifically is what separates someone who has operated a logging
pipeline from someone who has drawn one.

---

## Scoping — what a strong candidate establishes unprompted

The prompt gave nothing. **This section is 25% of the grade** and is the
part most candidates skip.

### First tier — reshapes the design

| # | Decision | Why it matters |
| --- | --- | --- |
| 1 | **Who is the consumer — customers, the platform team, or both?** | Customer-facing means an SLO, a UI, a price, and hostile input. Internal means a budget and trusted input. Both means two products sharing a pipeline, which is a legitimate and interesting answer. |
| 2 | **Which signals?** | Logs dominate volume and cost by an order of magnitude. A candidate who treats logs, metrics and traces as equal thirds hasn't done the arithmetic. Scoping to logs + metrics and explicitly deferring traces is a *strong* answer. |
| 3 | **Product or cost centre?** | Determines whether there's revenue to size against. |
| 4 | **Where does the engine start and stop?** | The prompt says *engine*. Collection + transport + storage + query is a defensible boundary; adding alerting, dashboards, billing and retention policy makes everything shallow. |
| 5 | **What scale?** | No numbers given. Picking, stating, and deriving is the whole game. |
| 6 | **What is different about stateful workloads?** | It's in the prompt and most candidates ignore it. A customer's Postgres has different log semantics (slow-query logs, WAL, crash recovery), a much higher cost of losing the last lines, and — critically — it's pinned to a machine, so its logs have a locality the stateless case doesn't. |
| 7 | **What loss rate is acceptable?** | Nobody states it. Zero is expensive; a stated non-zero number with a defence is stronger than an unstated assumption of zero. |
| 8 | **Managed services or your own hardware?** | The prompt says "a system like Railway," which is a company that runs its own metal. A design resting on managed cloud services has answered a question they didn't ask. |

**Assessment:** 6+ established with reasons is a Staff signal. Fewer than 4
caps at Senior no matter how good the architecture is, because the
architecture is unevaluable without them.

---

## Back-of-envelope

Recompute independently.

| Quantity | Defensible value | Working |
| --- | --- | --- |
| Log ingest | **1.5 GB/s ≈ 130 TB/day raw** | 10M lines/s × 150 B |
| Compressed | **~13 TB/day at 10x** | Log text compresses well; 8–15x is defensible |
| 30-day storage | **~390 TB** | Large but *not* the binding constraint — object storage at this size is a few thousand dollars a month |
| Full-text index + replica | **~1 PB, and 3–5x the compute** | Index overhead ≈ raw, plus replication. **This is the cost that breaks the business**, not the bytes |
| Ingest nodes for full indexing | **60–100+** | ~20 minimum at ~75 MB/s/node for ingest alone, realistically 3–5x with replication and query headroom |
| **One tenant at 500 MB/s** | **33% of total platform volume** | The number that forces per-tenant isolation into the ingest path |
| Per-machine log rate | **~0.5 MB/s average** | 1.5 GB/s ÷ 3,000 — modest per machine, which is why the agent must be small |
| Cost per paying customer | derived | ~$1M/month revenue, 20,000 payers. If observability exceeds ~10–15% of infra spend it is eating the margin |
| Free-tier cost | **the interesting one** | Free customers generate logs and no revenue. Retention tiering is a *business* control, not a technical one |

**Staff:** reaches the conclusion that **storage bytes are cheap and
indexing is not**, and lets that drive the storage fork. Computes the
single-tenant share and treats it as an architectural requirement rather
than an operational annoyance.

**Red flags:** No cost model at all. Treating 390 TB as the hard part.
Sizing an Elasticsearch cluster without pricing it against revenue.

---

## Design

### Architecture

**Senior:** agent on each machine → Kafka → consumer → Elasticsearch (or
Loki) → API. Correct shape, components named, no numbers attached.

**Staff:** tail and search paths are **visibly separate** downstream of a
shared, cheap ingest. The latency budget is annotated on the tail path. The
per-machine agent has a **stated resource budget** — every core it uses is a
core not sold to a customer, which at 3,000 machines is real money and is
the kind of constraint that only occurs to someone who has run a fleet.
Quota enforcement is at the *edge* (the agent or the machine), not
downstream, because downstream enforcement means you've already paid to
transport the abusive volume.

**Red flags:** One pipeline serving both tail and search. A per-machine
agent with no resource budget. Quota enforced after the expensive hop.

### The storage fork

Must be *decided*, with the customer consequence stated:

- **Index labels only, scan content** (Loki-shaped): dramatically cheaper —
  the compelling answer at this volume and revenue. Cost: search by content
  is a brute-force scan over a label-selected subset, so "find this error
  string across all my services last month" is slow or impossible. The
  candidate must say that out loud and argue it's acceptable because the
  dominant query is "show me *this service's* logs around *this time*,"
  which labels serve perfectly.
- **Full-text index**: better product, and the arithmetic says it costs more
  than the revenue. Defensible only for a small hot window with everything
  older on the cheap path — which is a strong hybrid answer.

**A candidate who picks one without naming what the customer loses has not
made a decision, they've made a choice.**

### The dying container

The highest-signal sub-problem. A strong answer covers:
- Where lines are buffered (container stdout → runtime → agent → network),
  and how much is in flight at each stage.
- That an OOM-kill is `SIGKILL` — no flush, no graceful shutdown (interviews
  05·A7, 08·A15). Whatever is in the application's own stdio buffer is
  already lost before the platform sees it, which is worth saying because it
  bounds what the platform can promise.
- Reading from the container runtime's log files on disk rather than
  relying on a live stream, so a dead container's tail is still on the
  machine and recoverable.
- A flush-on-container-exit hook, and what happens when the *machine* dies
  instead.
- A stated loss rate with evidence, not an assumption of zero.

### Multi-tenant isolation

Must handle the 500 MB/s tenant with a mechanism, not a policy statement.
Strong answers: per-service rate limits enforced **on the machine**, with
excess dropped and *counted*; a distinct "you are being rate limited"
message injected into the customer's own log stream so the drop is visible
rather than silent; per-project quotas tied to plan; and priority so a
tenant's burst can't delay another tenant's tail.

The nasty case that separates strong answers: **the customer hits their log
quota during an incident**, which is exactly when they need logs most.
Dropping them then is the correct engineering answer and a terrible product
answer. Anyone who notices this tension is thinking about the customer.

---

## The live extension (25%)

Simulated in review. The candidate is pushed on a dimension and must
redesign in place. Assess:

- **Does the change stay coherent** with everything said before?
- **Do they reach for a number** or hand-wave?
- **Do they say "I don't know, here's how I'd find out"** when appropriate,
  which is a strong answer, versus inventing something, which is not?
- **Can they identify what their change breaks?**

The extensions worth using, in rough order of value:

1. *One customer is a third of your volume.* Tests whether isolation is in
   the design or was assumed away.
2. *A customer wants to grep 30 days by message content.* Tests whether they
   understood the consequence of their own storage fork.
3. *This costs more than the compute revenue.* Tests whether they have a
   cost model at all.
4. *No managed services — bare metal only.* Tests whether the design was
   secretly leaning on someone else's operational burden.
5. *The last 200 lines before an OOM must be guaranteed.* Tests the buffer
   analysis.
6. *We lose a whole site.* Tests whether multi-site was designed or assumed.

---

## Grading calibration

| Grade | Profile |
| --- | --- |
| **Below Senior** | No scale established. One pipeline for everything. Elasticsearch proposed with no cost model. No isolation story. Cannot extend live without contradicting themselves. |
| **Senior** | Establishes scale. Correct pipeline shape with sensible components. Separates hot and cold storage. Handles the noisy tenant with a rate limit. Extends live but reaches for components rather than numbers. Doesn't price the indexing decision. |
| **Borderline Staff** | Prices the indexing fork *or* separates tail from search structurally, not both. Establishes 4–5 of the first-tier scoping decisions. Live extension is coherent but one dimension collapses under pushing. |
| **Staff** | Tail and search as separate systems over a cheap shared ingest. Indexing fork decided with the customer cost stated. Per-tenant enforcement at the edge with the 33% number driving it. Agent resource budget stated as competing with sellable capacity. Dying-container buffer analysis with a real loss rate. Cost per customer computed against revenue. 6+ scoping decisions. Extends live in at least four dimensions without contradiction. |
| **Strong Staff** | All of the above, plus: notices that the log line a customer wants most is the one hardest to guarantee, and designs specifically for it; treats free-tier retention as a business lever rather than a technical parameter; observes that the platform's own observability must not depend on the customer-facing path; identifies the quota-during-an-incident conflict and proposes something humane (burst credit, grace, visible degradation); and arrives having read the actual company's engineering writing and referencing their real constraints. |

---

## Review guidance

**Round 1:** Grade the scoping before reading the architecture — if the
scale isn't established, say so first, because everything downstream is
unevaluable. Recompute the volume, the index cost, and the single-tenant
share. Then run **three live extensions** from the list above in the review
itself and assess the answers, since that is what the real interview weights.

Do not reveal the tail-vs-search split. Ask: *"walk me through one log line
from a customer's `println` to their browser, and then walk me through the
same line being found by a search next Tuesday"* — and see whether the two
walks are the same path.

**Round 2+:** Commonly dodged: the agent's own resource cost, what the
customer loses under label-only indexing, and whether the platform's
internal observability shares fate with the customer-facing one.

**Final:** The highest-leverage improvement is usually one of: "you never
established scale, so none of this is assessable," "your tail path and your
search path are the same path and they want opposite things," or "you priced
the storage and not the indexing, and the indexing is the whole cost."
