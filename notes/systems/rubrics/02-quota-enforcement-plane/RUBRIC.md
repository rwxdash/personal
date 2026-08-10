# Rubric — 02 The Quota Enforcement Plane

> **Spoiler.** Do not open until the final review round is written.

---

## The central tension

**Exact enforcement requires coordination, and there is no coordination
budget.**

The latency budget is 1 ms at p99. Same-region RTT to a counter service is
~0.5–2 ms before the service does any work; cross-region is 70–150 ms. A
globally-exact limit requires a round trip to a single authority per
decision, which is somewhere between "the entire budget" and "150x the
budget." So global exactness at enforcement time is arithmetically
impossible, and every design must be approximate *somewhere*.

That would be a one-answer problem — enforce locally, reconcile
asynchronously — except for the second half of the brief: **the same system
must produce the invoice.** Billing tolerates latency but not error;
enforcement tolerates error but not latency. They have opposite tolerances,
and the brief deliberately treats them as one requirement.

**The structural insight the problem is built around: these are two
different systems that share an input.**

- **Enforcement** is approximate, local, fast, fails open, and its counters
  are disposable.
- **Accounting** is exact, asynchronous, durable, auditable, and its
  latency requirement is minutes-to-hours.

They share the same event stream and the same key space, and they must be
*reconcilable* — you must be able to explain why a customer was limited at
4.03M when the invoice says 4.11M — but they should not be the same
counters.

A candidate who builds one exact system pays for exactness on 36M
decisions/second and blows the latency budget. A candidate who builds one
approximate system bills customers from an approximate number, which is the
problem they were asked to fix. A candidate who separates them, and then
designs the reconciliation between them, has solved it.

**There is still no free answer**, which is what keeps it Staff-level. The
separation costs: two code paths, two sources of truth that will disagree,
a reconciliation process that is itself a system, and the awkward customer
conversation when enforcement and billing differ by a defensible-but-nonzero
margin. The design must state the tolerance and defend it.

**Secondary tension:** propagation speed of config is simultaneously the
product's best feature (a kill switch that works in two seconds) and the
cause of its worst outage (a bad config that reaches everywhere in two
seconds). Any safety mechanism that slows propagation degrades the kill
switch. Resolving this requires distinguishing *classes of config change*,
not slowing all of them.

---

## Ambiguities a strong candidate must catch

**Catching 6+ of the first tier is a Staff signal. Fewer than 4 caps the
grade at Senior.**

### First tier — changes the architecture

| # | Ambiguity | Why it matters |
| --- | --- | --- |
| 1 | **Are protective limits and billable quotas the same kind of object?** | The brief explicitly flags the author's uncertainty and does not resolve it. This is *the* question; a design that treats them uniformly has answered a different problem. |
| 2 | **What overshoot is acceptable, per class?** | Never stated. Without a number, "approximate" is undefined and the design cannot be evaluated. A strong candidate proposes one and justifies it against the business cost. |
| 3 | **Is the limit global, per-region, or per-PoP?** | Never stated. "1000 requests/minute" means three different things and they differ by 180x in enforcement cost. Many products define limits per-region *precisely because* it's cheaper — that's a legitimate answer if argued. |
| 4 | **Does a rejected (429) request consume quota?** | Affects both the counting semantics and the abuse surface. If rejections are free, an attacker gets unlimited rejected requests; if they count, a customer can be billed for calls we refused. |
| 5 | **Does a request that fails downstream (5xx) consume quota?** | Same shape, different politics. Billing a customer for our own errors is a credit conversation. |
| 6 | **Is the internal 6M rps in scope?** | The brief says "you tell me." A real answer with a reason is expected — including "no, and here is what internal needs instead." |
| 7 | **Fail open or fail closed, per class?** | Never stated. Failing open on a billable quota gives away revenue; failing closed on a protective limit turns a control-plane blip into a total outage. |
| 8 | **What is the month boundary for a monthly quota?** | Whose timezone, whose clock, and what happens to in-flight requests at the boundary. Sounds like a detail; it is a recurring source of billing disputes and it constrains the counter design (you need per-period state, not a rolling window). |

### Second tier — changes sizing or operations

| # | Ambiguity |
| --- | --- |
| 9 | Burst semantics: may a 1000/minute limit be consumed in one second? (Token vs leaky bucket — interviews 03·A13.) |
| 10 | What happens to counters when a limit's *definition* changes mid-period? |
| 11 | Is usage data customer-visible in real time, and at what freshness? (A customer dashboard showing usage creates a second accuracy contract.) |
| 12 | Retention of usage records — billing disputes, tax, and audit windows differ. |
| 13 | Are limits hierarchical (per-key inside per-account inside per-plan)? The brief implies four simultaneous checks but never says whether they compose. |
| 14 | Who may change a limit at runtime, and is there an approval path? (The January incident is a change-management failure as much as a technical one.) |
| 15 | Multi-tenancy of the plane itself: can one account's key cardinality degrade another's enforcement? |
| 16 | Is 99.99% on the *enforcement decision* or on *not blocking traffic*? Failing open means the request succeeds — does that count as available? |

---

## Section-by-section

### Part 1 — Requirements & scope

**Senior:** Restates the requirements. Asks a few clarifying questions.
Treats all limits as one class, or notices the distinction but doesn't act
on it.

**Staff:** Section 1.2 is filled in with genuinely different requirements
per class, and the "one system or two" question is answered with a stated
cost. Accuracy is expressed as a testable bound. Out-of-scope includes
things not handed to them — e.g. abuse detection, DDoS mitigation, the
billing system itself, per-customer plan management, error-response UX.

**Red flags:** Accepting "1 ms p99" without checking what fits inside it.
An accuracy requirement of "as accurate as possible." No answer to 1.2.

---

### Part 2 — Back-of-envelope

Recompute independently before commenting.

| Quantity | Defensible value | Working |
| --- | --- | --- |
| Limit decisions/s at peak | **36M/s** | 9M rps × 4 checks |
| Decisions/s per edge node | **~12,900/s** | 36M ÷ 2,800 |
| Per-decision CPU budget | **order of a few µs** | 1 ms budget shared across 4 checks, and the budget is p99 *added*, so the steady-state cost must be far below it |
| Same-region RTT | **~0.5–2 ms** | Order of magnitude. Already at or over the entire budget for a single check |
| Cross-region RTT | **~70–150 ms** | US–EU ~70–90 ms, US–APAC ~150 ms+ |
| **Round trips that fit in budget** | **Zero** | The conclusion the design turns on |
| Central store ops/s | **36M/s** | |
| Nodes for a central store | **~240+** | At ~150k ops/s/node for simple counter ops — order of magnitude. Fails on cost *and* on latency, and latency fails first and fatally |
| Worst-case local overshoot | **L × N_nodes** | Where N is the nodes seeing that key. For a key spread across 2,800 nodes and a limit of 1,000, worst case is 2.8M — i.e. no enforcement at all |
| Realistic overshoot for the top account | **large** | 11% of traffic touches essentially every node, so naive per-node division is useless for exactly the accounts that matter |
| Overshoot with budget allocation | **bounded by allocation granularity** | The reason allocation beats naive local buckets |
| Active (key, limit) tuples/minute | **~5M** | 1.2M keys × ~4 limits, plus IP and endpoint-class keys. Order of magnitude; the point is that it fits in memory per node only if you shard or if you don't keep all keys everywhere |
| Counter state per tuple | **tens of bytes** | Count, window start, allocation. Total is GB-scale globally, MB-to-GB per node depending on distribution |

**Senior:** Computes the decision rate. May not compare the latency budget
against RTT at all.

**Staff:** Explicitly concludes that **zero round trips fit in the budget**,
and lets that drive the architecture rather than treating it as a
constraint to optimise around. Quantifies local-enforcement overshoot as
`L × N` and shows why the naive "divide the limit by node count" is broken
for skewed traffic (a node with 0.001% of a key's traffic still gets 1/2800
of the budget and wastes it, while the hot node exhausts its share
instantly). Costs the central alternative before rejecting it.

**Red flags:** Rejecting a central store on "it wouldn't scale" without the
latency arithmetic. Not quantifying overshoot, which makes "approximate is
fine" an assertion rather than a conclusion. Estimates off by more than
~2–3x from the table without a stated reason — show the delta alongside
theirs.

---

### Part 3 — API & data model

**Senior:** A `check(key, limit)` API returning allow/deny. Counters keyed
by (key, window).

**Staff:** Distinguishes a **limit definition** (config, versioned, changes
rarely) from a **limit instance** (a live counter for one key in one
period). Notices that a **monthly quota is a fundamentally different object**
from a per-minute limit — different state lifetime, different storage,
different accuracy requirement, different reset semantics — and does not
force them into one abstraction. Handles the month-boundary question with a
stated policy.

Addresses the 11%-of-traffic account explicitly. Good answers: that account's
keys get dedicated treatment (their own allocation authority, or pinning, or
a larger allocation granularity because their volume makes coordination
amortise better).

Notices that the four checks per request should be **one call, not four** —
batching the decision is free latency, and a candidate who leaves four
sequential lookups in the hot path hasn't costed their own design.

**Red flags:** One counter abstraction for both per-second and per-month
limits. No mention of limit versioning, which makes the config-change
question unanswerable.

---

### Part 4 — High-level architecture

**Senior:** Edge nodes → a limiter service → a shared counter store; config
pushed from a control plane.

**Staff:** Enforcement is **in-process at the edge** (a library or a
sidecar on the same host, not a network hop) with the latency budget
annotated per hop to prove it. A separate asynchronous path carries usage
events to the accounting system. The two paths are visibly different
components with different SLOs.

Shows what the edge does with no control plane reachable, and for how long
it can operate that way (cached config with a stated staleness bound, and
what happens when it expires — this is a real decision, not a detail).

**Red flags:** A network call to a "rate limiter service" in the request
path, without acknowledging it consumes the entire budget. A single "usage
store" serving both enforcement reads and billing. Magic boxes labelled
"sync" or "aggregator" with no internal design.

---

### Part 5 — Deep dives

The two hardest are **A (distributed counting)** and **B (billing-grade
accounting)**, and they are the two the central tension lives between.
A+B is the strongest pairing. A+C or B+D are defensible. C+D avoids the
core tension and should be named as such in the review.

#### A — Distributed counting

**Senior:** Local token buckets, periodically synced to a central store.
Acknowledges drift.

**Staff:** Compares the family properly and picks with numbers:
- **Independent local buckets** (`L / N` per node): simple, zero
  coordination, and *badly* wrong under skew. Show why.
- **Local buckets + async aggregation**: nodes report usage, a controller
  redistributes the global budget proportionally to observed traffic every
  1–10 s. Enforcement stays local and free; overshoot is bounded by one
  redistribution interval's worth of traffic. This is the workhorse answer.
- **Leased token blocks**: a node leases N tokens from an authority and
  spends locally, refetching when low. Bounded overshoot equal to
  outstanding unspent leases; coordination cost falls as N rises. Better for
  scarce/expensive limits; the failure case — a node dying with an unspent
  lease — must be addressed (lease TTL, and the accepted undershoot).
- **Sticky routing by limit key**: makes local state authoritative and
  exact, at the price of reintroducing hot keys and losing PoP locality.
  Worth naming as the option that *does* give exactness, and saying why the
  latency cost of routing a request to a distant PoP exceeds the benefit.

Must handle: traffic shifting between PoPs mid-window (an allocation held by
a PoP that stops seeing traffic is stranded budget — needs reclaim), node
death with unspent allocation, and the cold-start problem for a key nobody
has seen before.

**Strong Staff signal:** noticing that the *allocation interval* is the
tuning dial that trades overshoot against coordination traffic, and giving
the arithmetic — overshoot ≈ (traffic rate during one interval) − (allocated
budget), so a 1-second interval on a 1000/min limit is nearly exact while a
10-second interval on a bursty key is not.

#### B — Billing-grade accounting

**Staff:** Defines a durable **usage event stream** independent of
enforcement — the edge emits a record per request (or per aggregated batch,
with the aggregation being lossless for counting purposes) into a log, and
the accounting system is a consumer. Addresses:
- **Delivery semantics**: at-least-once with a request id for dedup, or
  at-most-once with an accepted loss rate. Either can be defended; the
  accepted error must be stated and it must be smaller than the 4% they're
  trying to fix.
- **Reconciliation**: the design must explain the residual difference
  between the enforcement counter and the billed number, because there will
  be one, and a customer will ask. Strongest answers make the *usage stream*
  authoritative for both — enforcement uses a fast approximation of it, and
  the reconciliation statement is "enforcement may permit up to X% over;
  billing counts exactly."
- **Durability and retention** appropriate to a billing dispute.
- The counting-semantics questions from ambiguities 4 and 5, answered.

**Red flags:** Billing directly from the enforcement counters (the problem
they were asked to fix). No dedup story on an at-least-once stream. No
stated tolerance.

#### C — Config safety

**Staff:** Distinguishes **classes of change** rather than slowing
everything:
- *Restrictive* changes (a new or tighter limit) roll out progressively —
  one PoP, then a percentage, with automatic rollback on an error-rate or
  reject-rate signal.
- *Permissive* changes (raising a limit, disabling a limit, a kill switch)
  propagate immediately, because they cannot cause the January failure — the
  worst case is less enforcement, and that is the safe direction.

This asymmetry is the insight; it preserves the two-second kill switch while
making the outage impossible.

Plus: schema validation with a **mandatory default** (the actual January
bug), a dry-run/shadow mode that reports what a config *would* have
rejected against live traffic before enforcing it, versioned configs with
one-command rollback, and a hard cap on how much of the fleet a single
change may affect within a time window.

**Red flags:** "We'd add a code review step" as the whole answer. Slowing
all propagation, which breaks the kill switch, without noticing the
tradeoff.

#### D — Failure and degradation

**Staff:** Per-class fail behaviour, justified:
- **Protective limits fail open.** The limit exists to protect a backend;
  if the plane is degraded, the backend is probably fine, and refusing all
  traffic converts a limiter outage into a total outage. Accept the risk of
  an unprotected window.
- **Billable quotas fail open too** — but with the usage still *recorded*,
  so the customer is billed correctly even though enforcement lapsed.
  Revenue is preserved; only the ceiling lapses. This is the neat answer and
  strong candidates find it.
- Genuinely abusive traffic is a different control (abuse/DDoS) which should
  be explicitly out of scope.

Also: a stated bound on how long an edge may operate on stale config, and
what it does when that expires — continuing forever is a security position,
failing closed is an outage, and picking one requires saying which.

---

### Part 6 — Failure modes

**Staff:** Blast radius quantified. The **isolated-but-serving PoP** is the
partition scenario and must be handled: it cannot reach the allocator, it is
still taking customer traffic, and it must decide whether to keep spending
its last allocation, fall back to a static local limit, or stop enforcing.
All three are defensible; not choosing is not.

The **hot key** case (40x normal traffic on one key) must show the
allocation loop reacting fast enough, or an explicit statement that it
won't and what the overshoot is.

**Red flags:** No partition scenario. "We'd fail open" applied uniformly
without per-class reasoning. Not treating their own deploy as a failure
mode, given the plane is on every request path.

---

### Part 7 — Tradeoffs ledger

**Staff:** Specific, attributable costs. Should include at least: the
accepted overshoot and who absorbs it, the enforcement/billing divergence
and who explains it to customers, the propagation-safety asymmetry and what
it means for restrictive changes, and the cost of running two systems.

**Red flags:** Fewer than ~8 rows. Every cost trivial.

---

### Part 8 — Evolution

**Staff:** Names the binding constraint at 10x with a number. Plausible
candidates: the **usage event stream** at 90M events/s (which likely forces
edge-side pre-aggregation, changing the billing accuracy story); **key
cardinality** in per-node memory; the **allocator's** fan-out to 28,000
nodes; or the config propagation fan-out.

The migration is the interesting part and is frequently thin. Forty
limiters with incompatible semantics, all in front of live traffic. A strong
answer runs the new plane in **shadow mode** first — computing decisions and
recording what it *would* have done, without enforcing — compares against
each existing limiter, and cuts over per-service with the old limiter still
in place as a backstop. Anything resembling a flag day on the request path
of every service should be pushed on hard.

The internal-traffic question deserves a real answer: including it likely
means a different enforcement point (mesh/sidecar rather than edge) with the
same control plane and accounting, which is a coherent design; excluding it
means the noisy-neighbour problem the brief names goes unsolved, and the
candidate should say so rather than quietly dropping it.

---

### Part 9 — Operations & cost

**Staff:** The "customer says they were limited under their limit" question
is the best discriminator in this section. Answering it requires a **per-
decision audit trail** — at least sampled — and most designs don't produce
one. A candidate who notices that they need decision-level observability,
and prices it (you cannot log 36M decisions/s, so it's sampled, or it's
only-on-reject, or it's reconstructable from the allocation record), is
demonstrating operational maturity.

Alerts should be symptom-based: reject rate by class, allocation staleness,
usage-stream lag, enforcement/billing divergence. Not CPU.

---

## Grading calibration

| Grade | Profile |
| --- | --- |
| **Below Senior** | Proposes a central Redis without checking the latency budget. No distinction between limit classes. No overshoot arithmetic. |
| **Senior** | Correct distributed-counting design with local buckets and async sync. Computes the decision rate. Handles component failures. Treats billing as "we'll also count there" without addressing divergence. Config safety is "review and canary." |
| **Borderline Staff** | Concludes that no round trip fits the budget and designs from it. Notices the two limit classes but doesn't fully separate them, or separates them without designing the reconciliation. One strong deep dive, one thin. Fail-open/closed answered but uniformly. |
| **Staff** | Separates enforcement from accounting as distinct systems with a designed reconciliation and a stated tolerance. Catches 6+ first-tier ambiguities. Quantifies overshoot and picks an allocation mechanism with numbers. Per-class fail behaviour. Config safety distinguishes restrictive from permissive changes. Names what breaks at 10x with a number. Explicit out-of-scope. |
| **Strong Staff** | All of the above, plus: makes the usage stream authoritative for both paths so the divergence is explainable by construction; the restrictive/permissive propagation asymmetry; recognises that billable quotas can fail open while still recording usage; a shadow-mode migration with per-service cutover and backstop; and a decision-level audit story that is priced rather than assumed. Bonus: proposes changing a requirement (per-region limits instead of global) with the arithmetic showing what it saves. |

---

## Review guidance

**Round 1:** Recompute Part 2 independently first. Probe hardest at: whether
the latency budget permits any coordination, what the accepted overshoot
number is, whether billing and enforcement read the same counters, and what
happens to the 11% account. Do not reveal the two-systems insight — ask
"which number goes on the invoice, and how do you explain the difference to
the customer?" and let them find it.

**Round 2+:** The commonly dodged items here are the enforcement/billing
reconciliation, the per-class fail behaviour, and the migration off 40
limiters. Escalate those rather than opening new ground.

**Final:** The single highest-leverage improvement is usually one of:
"enforcement and billing have opposite tolerances and you built one system,"
"you never stated the overshoot you're accepting," or "your config safety
mechanism breaks the kill switch you also need."
