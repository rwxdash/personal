# Model Discussion — 01 The Alerting Plane

> **Spoiler.** Do not open until the final review round is written.

This is **not** the correct answer. It is one credible design, an honest
account of why its main alternative loses under these specific constraints,
and the places where competent Staff engineers would still disagree with it.
If your design differs and your tradeoffs are defended, that is not a worse
answer.

---

## 1. The numbers that decide the architecture

Two calculations dominate everything.

**Read amplification.** 400,000 rules at a 30-second interval is **13,333
evaluations/second**. A typical rule aggregates ~40 series over a 5-minute
window at 15-second resolution — 800 datapoints per evaluation. That is
**~10.7M datapoints/second read**, against a 4M samples/second peak write
rate. A **2.7:1 read:write ratio**, and it explains the director's
observation that the ruler tier is larger than the ingest tier exactly.

At 3x, if rule count grows with the business, this becomes ~32M/s. If you
size a query tier for that, the budget is gone before you have built
anything else.

The conclusion: **you cannot afford one query per rule.** Any design that
keeps the per-rule query model is sizing around the problem rather than
solving it.

**The late-data burst.** 15% of the fleet partitioned for 4 hours buffers
~4.3 billion samples (300k/s × 14,400 s). Drained over 20 minutes that is
**~3.6M samples/second of backfill on top of normal traffic** — roughly
doubling peak ingest, from a single regional incident. Sizing the steady
state for this is absurdly expensive; the burst has to be absorbed
structurally.

---

## 2. The design

```
                       ┌──────────── agents (fleet + customer-hosted) ────────────┐
                       │  local buffer up to 6h · self-timestamped                │
                       └──────────────────────────┬──────────────────────────────┘
                                                  │ (batch, compressed, tenant-authenticated)
                                    ┌─────────────▼─────────────┐
                                    │  Regional ingest gateway   │
                                    │  · authn, tenant tagging   │
                                    │  · clock-skew clamp        │
                                    │  · per-tenant rate limit   │
                                    │  · LIVE vs BACKFILL split  │◄── the key structural move
                                    └───────┬───────────┬────────┘
                                            │           │
                        ┌───────────────────▼──┐   ┌────▼──────────────────┐
                        │  LIVE log             │   │  BACKFILL log          │
                        │  (partitioned by      │   │  (separate topics,     │
                        │   series hash)        │   │   throttled drain)     │
                        │  retention: 24h       │   │  retention: 72h        │
                        └───┬───────────────┬───┘   └────┬───────────────────┘
                            │               │            │
             ┌──────────────▼──┐   ┌────────▼────────────▼────┐
             │  Ingesters      │   │  Evaluation workers       │
             │  · WAL + head   │   │  · own a rule-group shard │
             │  · flush blocks │   │  · consume the same log   │
             └────────┬────────┘   │  · shared subscription    │
                      │            │  · in-memory window state │
             ┌────────▼────────┐   │  · state journalled       │
             │  Object storage │   └────────┬──────────────────┘
             │  (blocks, 15mo, │            │
             │   downsampled)  │            │  alert transitions
             └────────┬────────┘            │  + evaluation records
                      │                     ▼
             ┌────────▼────────┐   ┌────────────────────────────┐
             │  Query tier     │   │  Alert state store          │
             │  (dashboards +  │◄──┤  · per-instance state       │
             │   audit re-runs)│   │  · durable evaluation log   │
             └─────────────────┘   └────────┬───────────────────┘
                                            │
                                            ▼  (out of scope)
                                   notification / routing team
```

### 2.1 The single most important decision: invert the read path

Instead of each rule querying storage, **evaluation workers consume the
same log the ingesters consume**, and each worker holds the state for a
*group* of rules. A sample arriving for series S is delivered once to the
worker that owns S's partition, and that worker updates the windows of
**every rule that selects S**.

The read amplification collapses. Instead of 10.7M datapoints/s of *reads*,
each of the 4M samples/s is delivered once and fans out in memory to the
rules that care about it. In-memory fan-out is nearly free compared to a
storage round trip. The cost moves from I/O to **memory and to the rule-to-
series index**, which is a far better trade at this scale.

Concretely, each worker maintains:
- A **series → rule-instance** index, built by evaluating each rule's
  label selector against series metadata as series appear.
- Per rule-instance, a **windowed aggregate** (a sliding window of the
  aggregation the rule needs, not the raw points — a p95 rule keeps a sketch,
  a `sum` rule keeps a running sum with expiry buckets).
- Pending-state for `for:` clauses ("firing since T").

This is the answer to "why is the ruler tier bigger than the ingest tier":
it shouldn't be a separate tier at all.

**What this costs.** Rule changes are no longer free. Editing a rule
invalidates its window state, which must be rebuilt by replaying the log
(hence the 24-hour LIVE retention — it bounds how far back a rule with a
long window can be rebuilt). Rules with windows longer than the log
retention need a bootstrap read from storage. And an evaluation worker
holding state is now a stateful service with all that implies: rebalancing
costs, restart costs, and a memory ceiling per worker.

### 2.2 Live and backfill are separate paths

The gateway classifies each sample by how far its event time is behind now:
- **Live** (within ~2 minutes): goes to the live log. Evaluation happens
  here.
- **Backfill** (older): goes to a *separate* log with its own topics and a
  *throttled* drain, per tenant and per region.

This is what stops November's incident from recurring. A regional flush
lands in the backfill path and drains at a bounded rate; live ingest and
therefore live detection for every other tenant is untouched. The flush
takes longer to land — hours instead of 20 minutes — and that is the
accepted cost, and it is much cheaper than the alternative.

Backfill samples are written to storage normally. They **do not** re-drive
live rule evaluation.

### 2.3 The event-time model and what happens to late data

**Detection SLO measured from arrival time at the gateway**, not event time.
Stated explicitly, because under event time a 4-hour buffered sample is
permanently out of SLO and no design fixes that. Arrival-time SLO is
honest about what the system controls; the gap between event time and
arrival time is reported as a **separate freshness metric per agent and per
tenant**, which is the number that actually tells a customer whether their
collection is healthy.

**Allowed lateness for live evaluation: 2 minutes.** Beyond that, a sample
is backfill.

The four hard cases, answered:

1. **Data arrives 4 hours late and shows a breach.** It is **recorded as a
   retroactive evaluation, and it does not page.** A retroactive evaluation
   record is written to the alert state store, flagged as `late`, with the
   event-time window it covers. It is visible in the UI, it counts for SLA
   reconstruction, and it does not wake anyone at 3am about an incident that
   ended three hours ago. Paging on stale data trains people to ignore
   pages.

2. **An alert fired and late data shows it shouldn't have.** It is
   **annotated, not retracted.** The original firing stands as a historical
   fact (it *did* fire, on the data available at the time), with a linked
   correction record. Retraction would require a notification contract we
   don't own and would make the alert history non-monotonic, which breaks
   every downstream consumer.

3. **The flush itself:** §2.2.

4. **Clock skew:** the gateway clamps. Samples more than 5 minutes in the
   future are rejected (and counted, per agent, as a health signal — a
   skewed agent is a fixable problem and hiding it is not a kindness).
   Samples in the past are accepted up to the backfill horizon; beyond the
   log retention they are rejected rather than silently dropped.

### 2.4 Reproducibility: the evaluation log

The director's real requirement, extracted from "we had to reconstruct it
from raw data," is: **every evaluation that produced or sustained an alert
state must be reconstructible.**

So every state transition writes a durable **evaluation record**:

```
evaluation_id, tenant, rule_id, rule_version, instance_labels,
window_start_event_time, window_end_event_time,
input_summary (the aggregated value, plus the series set and sample counts),
outcome, evaluated_at, worker_id, lateness_class
```

Retention matches the contractual window: **15 months**. Volume is bounded
by *transitions*, not by evaluations — a rule that stays OK writes nothing —
so this is a small fraction of the metric volume, and that is what makes it
affordable.

This is what lets you answer an SLA dispute in a query rather than an
investigation, and it is why a pure streaming design without this log would
not meet the actual requirement.

Re-running an evaluation from storage may give a different answer than the
original, because late data arrived. That is expected and it is **not a
bug**: the record says what was known at the time, and the correction record
says what is known now. Both are true, and the SLA conversation is about
which one the contract references — which is a commercial question, not a
technical one, and the design surfaces it rather than hiding it.

### 2.5 Isolation

**Cells.** The plane is deployed as ~8–12 independent cells, each a
complete stack (gateway, log, ingesters, evaluators, state store). Tenants
are assigned to a cell; the assignment is a mutable mapping, not a hash, so
a tenant can be moved.

- Blast radius: any cell-level failure affects ~1/10th of tenants.
- The 50,000-rule tenant with 60% of a region's series **gets its own
  cell.** That is the answer to "what does a tenant that outgrows the unit
  do" — it stops being a tenant of a shared unit. This costs real money and
  is the correct place to spend it.
- Within a cell: per-tenant limits on active series, ingestion rate, rule
  count, and **evaluation cost** (a budget in series-touched-per-second, not
  rule count, because rule cost varies by orders of magnitude). Exceeding a
  limit throttles that tenant's *new* series or *lowest-priority* rules,
  and raises a tenant-visible signal — never silently degrades their
  existing alerting.
- The rule-to-worker assignment within a cell is by **rule group**, sized by
  measured evaluation cost, so a single expensive rule group can be isolated
  or given a dedicated worker.

EU residency: cells are already region-scoped, so a residency requirement
means pinning EU tenants to EU cells. The parts that would need work are
the global control plane (rule management, tenant mapping) and cross-cell
observability. This is why cells were chosen over a single global fleet even
before residency was confirmed — it is a cheap option on a likely
requirement.

### 2.6 Failure behaviour

**Fail closed on evaluation, fail open on ingest.**

- If an evaluation worker knows its input is incomplete — log consumer lag
  above a threshold, or a partition it owns is unavailable — it **does not
  evaluate** and marks the affected rule instances `unknown`, which is a
  distinct state from OK and from firing. `unknown` is itself alertable, by
  us, not by the customer.
- Rationale: a false page generated by our own degradation is how customers
  stop trusting the alerting product. An explicit "we don't know" is
  honest and actionable; a guess is neither.
- Ingest fails open: accept the sample, buffer it, sort it out later. Losing
  data is worse than delaying it.

**Partition of the alert state store from the evaluators.** Evaluators
continue with in-memory state and journal transitions locally; on heal they
reconcile. Duplicate transitions are possible, so alert emission is
**at-least-once with a stable idempotency key** (`rule_id + instance_labels
+ transition_event_time`), and the notification team's dedup — which the
brief says they own — absorbs it. This assumption is stated and would be
confirmed with them before building.

**Our own deploy** is a coverage gap. Evaluation workers hand off state to
their replacement before exiting (state is journalled and the successor
replays from the journal plus the log), so a rolling deploy costs seconds of
evaluation lag rather than a window of blindness. This is designed for
explicitly, because with 400k rules a deploy happens weekly and a naive
restart loses every pending `for:` clause.

---

## 3. Why the main alternative loses

**The alternative: keep the store-query model, and scale it.** Partition
rules across a much larger ruler tier, put a caching layer in front of
storage, and buy the hardware.

It loses on three counts, in order of decisiveness:

1. **Cost.** 10.7M datapoints/s of reads today, ~32M/s at 3x. A query tier
   serving that, with the storage IOPS behind it, does not fit in $3M/year
   alongside ingest and 15 months of retention. The read path *is* the
   budget. This is the argument that ends the discussion.

2. **It doesn't fix the stated symptom.** The director's complaint is that
   one tenant's rules delay everyone's evaluation. Sharding rules across
   more rulers reduces the frequency but not the mechanism — a single
   expensive rule still occupies a ruler for its duration, and the tail of
   the rule-cost distribution is where the problem lives. Isolation has to
   be a unit boundary, not a bigger shared pool.

3. **Detection latency.** A store-query design has a floor of (scrape
   interval + ingestion visibility delay + query time + evaluation
   interval). At p95 across 400k rules under load, holding 45 seconds
   requires the query tier to be provisioned for peak with headroom — which
   compounds problem 1.

Where the alternative genuinely wins, and it is worth being honest about
this: **it is much simpler**, rule changes are free, late data is handled
automatically on the next evaluation with no special machinery, and there is
no stateful evaluation tier to rebalance. If the rule count were 40,000
instead of 400,000, it would be the right answer, and the correct engineering
judgement would be to keep it. The streaming inversion is justified by the
read:write ratio and by nothing else.

---

## 4. Where reasonable Staff engineers would disagree

**4.1 Not paging on late data.** A serious counter-argument: if a customer's
service was down for four hours and our collection was partitioned, they
absolutely want to know now, even at 3am. My position is that a page should
be actionable and a four-hour-old incident isn't — but a design that pages
with a prominent "STALE — event time 4h ago" and lets the customer configure
the behaviour per rule is defensible and arguably better product. I would
not push back hard on a candidate who chose it.

**4.2 Annotating rather than retracting.** For an alerting product, leaving
a false alert in the history unretracted is uncomfortable, and a customer
using our records for their own SLA reporting may reasonably want the
correction to be authoritative. The counter-position — that alert history
should be append-only and monotonic — is an engineering preference, not a
law. This is genuinely unresolved.

**4.3 Arrival-time SLO.** Measuring detection from arrival time is
defensible and honest, and it is also *convenient* for us — it excludes
exactly the failures we can't control from the number we're measured on. A
customer would reasonably say the SLO should be event-time-based with the
partition periods excluded by agreement. That's a harder measurement and a
better contract, and I could be argued into it.

**4.4 Stateful evaluators.** The streaming design trades an I/O problem for
a state-management problem: rebalancing, memory ceilings, state rebuild on
rule change, and a much harder operational story. A team without experience
running stateful stream processing at this scale would plausibly be better
served by the simpler design plus aggressive per-tenant isolation and a
negotiated increase in evaluation intervals (30s → 60s halves the read
rate, and for most alerts nobody would notice). **"Change the requirement
instead of the architecture" is a legitimate Staff move** and a candidate who
proposes it with the arithmetic deserves credit, not a mark against.

**4.5 Cells vs a single global fleet with quotas.** Cells cost utilisation —
every cell needs its own headroom, and a dedicated cell for one tenant is
expensive. A single fleet with rigorous per-tenant admission control and
shuffle sharding is cheaper and is what several large observability vendors
actually run. My preference for cells rests heavily on the residency option
and on blast radius; someone weighting cost more heavily would choose
differently and be right for their constraints.

**4.6 Whether to build this at all.** Nobody in the brief asks whether the
15-month retention, the 15-second resolution, or the 30-second evaluation
interval are all actually required. A Staff engineer should at least *ask*,
because relaxing any one of them changes the cost by a large factor, and the
cheapest system is the one you don't have to build. A design that opens with
"before we build this, here are three requirements whose cost the business
may not realise it is buying" is doing the job properly.
