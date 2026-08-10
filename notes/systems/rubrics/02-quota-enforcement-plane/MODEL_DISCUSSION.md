# Model Discussion — 02 The Quota Enforcement Plane

> **Spoiler.** Do not open until the final review round is written.

One credible design, why its main alternative loses, and where competent
Staff engineers would still disagree. Not the correct answer.

---

## 1. The calculation that ends the debate

The latency budget is **1 ms added at p99**. A same-region network round
trip to a counter service is ~0.5–2 ms before the service does any work.
Cross-region is 70–150 ms.

**Zero round trips fit in the budget.** Not "one is tight" — zero.

Everything follows. Enforcement must happen **in-process on the edge node**,
against state already in memory, at microsecond cost. Any design with a
network hop in the enforcement path has already failed, and the only
remaining question is where the coordination that makes local state
*meaningful* happens — which is off the request path, by definition.

The second calculation: **36M decisions/second**. Even if latency were free,
a central store at ~150k ops/s per node needs ~240 nodes to serve it, which
costs more than the thing it protects. Latency kills it first and fatally;
cost kills it again.

---

## 2. The reframe: two systems, one stream

The brief asks for one plane doing two jobs whose tolerances are opposite:

| | Enforcement | Accounting |
| --- | --- | --- |
| Latency requirement | < 1 ms | minutes–hours |
| Accuracy requirement | approximate is fine | must be defensible to a CFO |
| Failure behaviour | fail open | never lose a record |
| State lifetime | seconds–minutes | 7 years |
| Cost per decision | must be ~free | can afford real work |

Trying to satisfy both with one set of counters means paying accounting's
accuracy cost on enforcement's latency budget. It cannot be done.

**So: two systems that consume the same event, and are reconcilable by
construction.**

```
                     request
                        │
        ┌───────────────▼────────────────────────────────┐
        │  EDGE NODE (2,800 of them)                     │
        │                                                │
        │  in-process limiter                            │
        │   · config cache (versioned, TTL'd)            │
        │   · local token buckets per (key, limit)       │
        │   · spends from a leased/allocated budget      │
        │   · decision in ~µs, no network                │
        │                                                │
        │  emits: usage record ──────────────┐           │
        └──────────┬─────────────────────────┼───────────┘
                   │ (async, batched)        │ (async, batched, durable)
        ┌──────────▼──────────┐   ┌──────────▼─────────────────────┐
        │  ALLOCATOR          │   │  USAGE LOG (per region)         │
        │  regional, then     │   │  · at-least-once + request id   │
        │  global rollup      │   │  · retained 7 days hot          │
        │  · observes usage   │   └──────────┬─────────────────────┘
        │  · redistributes    │              │
        │    global budget    │   ┌──────────▼─────────────────────┐
        │    every 1s         │   │  ACCOUNTING                     │
        └──────────┬──────────┘   │  · dedup by request id          │
                   │              │  · exact per-account/period     │
                   │ allocations  │  · durable, auditable, 7y       │
                   └──────────────┤  · feeds invoices AND the       │
                                  │    allocator's monthly ceiling  │
                                  └─────────────────────────────────┘
```

**The reconciliation is designed in, not bolted on.** The usage log is the
single authoritative record of what happened. Accounting is an exact fold
over it. Enforcement is a *fast approximation of the same fold*, running
ahead of it. The statement you make to a customer is therefore precise:

> "Your invoice is the exact count. Enforcement is a fast approximation of
> it and may permit up to 1% over your limit before it catches up. It will
> never bill you for calls you didn't make."

That sentence is the product of the architecture, and it is what the current
system cannot say.

---

## 3. How the counting actually works

### 3.1 Per-minute protective limits: allocation, not local division

Naive local division (`limit ÷ 2,800 nodes`) is useless under skew: a node
seeing 40% of a key's traffic gets 0.036% of the budget, and 2,799 nodes sit
on budget they'll never spend.

Instead, a **two-tier allocator**:

- Every node reports, every second, its observed rate per active
  (key, limit) tuple.
- A **regional allocator** aggregates and redistributes that region's share
  of the global budget in proportion to observed demand, plus a small floor
  so a node that suddenly starts seeing a key isn't stuck at zero.
- A **global allocator** does the same across regions, on a slower cadence
  (~5 s), because cross-region coordination is expensive and regional demand
  shifts slowly.

Enforcement spends against the local allocation with a normal token bucket.
No network call.

**Bounded overshoot.** The worst case is roughly one allocation interval of
unanticipated traffic: if demand for a key spikes on a node between
allocations, that node can overspend by (spike rate × interval) before the
next allocation corrects it. At a 1-second regional interval this is small
for anything but a genuine attack — and an attack is an abuse problem, not
a quota problem, which is why abuse is out of scope.

**The tuning dial is the allocation interval**, and stating that explicitly
is the point: shorter interval → less overshoot, more coordination traffic.
1 s regional / 5 s global costs ~2,800 small reports/s per region, which is
nothing compared to 36M decisions/s.

**Stranded budget** is the failure people forget. A PoP that stops seeing a
key holds allocation nobody can spend. Allocations therefore carry a TTL
slightly longer than the interval and are re-granted, not held — so a dead
node's budget returns automatically within one interval, at the cost of a
brief global undershoot.

### 3.2 Monthly quotas: a different object entirely

A monthly quota is not a rate limit with a long window. Treating it as one
means keeping a 30-day bucket in edge memory, which is both wasteful and
wrong across node restarts.

Instead: **accounting owns the monthly counter**, and the allocator
periodically pushes each account's *remaining* quota to the edge as a
budget. Enforcement checks a cheap local "has this account exhausted its
month" flag plus a leased block of the remainder.

Consequence: the monthly ceiling is enforced with a lag equal to the push
interval (say 30 s), which for a 5M-call monthly allowance is an overshoot
of at most a few thousand calls — well inside any tolerance a customer would
notice, and *billed correctly regardless*. As the account approaches its
limit the lease size shrinks, so precision increases exactly where it
matters. That graduated precision is the neat part.

### 3.3 One call, not four

The four checks per request are evaluated in a single in-process pass over a
pre-compiled rule set. A design that leaves four sequential lookups in the
hot path has spent its budget four times over on cache misses alone.

---

## 4. Config safety without breaking the kill switch

The January outage and the two-second kill switch are the same mechanism
seen from two directions. Resolving it requires distinguishing **direction
of change**, not slowing propagation:

- **Permissive changes** — raising a limit, disabling a limit, an emergency
  bypass — propagate immediately, globally, as fast as the network allows.
  Their worst case is *less* enforcement, which cannot cause a global
  outage. The kill switch keeps working.
- **Restrictive changes** — a new limit, a lowered limit, a new default —
  roll out progressively: one PoP, then 5%, then 25%, then global, with
  automatic rollback if the reject rate for affected keys exceeds a
  threshold. A staged rollout of a config change is a deploy, and it gets a
  deploy's safety machinery.

Plus, specifically targeting the January bug:
- **Schema validation rejects a config without an explicit default.** The
  bug was a missing default falling through to zero; the fix is that
  "unmatched key" must be an explicitly stated behaviour, and the default
  default is *allow*.
- **Shadow evaluation**: a new config runs against live traffic in
  report-only mode, and the operator sees "this would have rejected 14% of
  account X's traffic" before it enforces anything. This catches intent
  errors that schema validation cannot.
- A hard cap on fleet fraction affected per unit time, enforced by the
  control plane rather than by the operator's discipline.

---

## 5. Failure behaviour

**Both classes fail open — but accounting keeps counting.**

This is the answer that surprises people and it falls out of the two-system
split. If the allocator is unreachable:

- Enforcement continues on the last allocation, then on a conservative
  static fallback derived from the cached config, then — if config itself
  is stale beyond its TTL — allows.
- **Usage records keep flowing to the log regardless.** So a customer who
  blows past their quota during an allocator outage is still billed
  correctly. We lose the *ceiling*, not the *revenue*.

Rationale: the protective limits exist to keep backends up, and the backends
are almost certainly healthy during a limiter-only failure; refusing all
traffic would convert a limiter outage into a total outage. And the
commercial risk of a lapsed ceiling is bounded and recoverable, whereas an
availability incident is not.

The genuinely abusive case — someone exploiting the outage — is an abuse and
DDoS problem, explicitly out of scope, with its own controls that do not
depend on this plane.

**The isolated PoP.** A PoP that can serve customers but not reach the
allocator is the partition case. It continues on its last allocation for one
TTL, then falls back to a static per-PoP limit derived from that PoP's
historical share (shipped with the config, so it needs no coordination to
apply). Overshoot during a partition is bounded by that static limit ×
partition duration, and it's recorded, so it's visible afterwards.

---

## 6. Why the main alternative loses

**The alternative: one exact system. A globally sharded counter service,
with the edge calling it synchronously, sized for 36M ops/s.**

It loses on three grounds, in order:

1. **Latency, fatally.** One same-region round trip is 0.5–2 ms against a
   1 ms total budget, and any key whose authority is in another region costs
   70–150 ms. There is no implementation quality that recovers this; it's
   the speed of light and a network stack. This ends it.
2. **Cost.** ~240+ nodes of counter service, plus the cross-region traffic,
   on a plane that must be cheaper than the services it protects.
3. **Availability.** It makes a synchronous dependency of every request in
   the company on a single distributed system. The plane must be *more*
   available than everything behind it; adding a hard synchronous dependency
   moves it in the wrong direction, and failing open on it means you've
   built the approximate system anyway, with worse latency.

**Where the alternative genuinely wins**, and it's worth being honest: it is
*much* simpler. One counter, one number, no reconciliation, no allocator, no
divergence to explain, and billing is trivially correct. If the traffic were
100k rps instead of 9M, or if the latency budget were 20 ms instead of 1 ms,
it would be the right answer and building the allocator would be
over-engineering. The distributed design is justified by two numbers and
nothing else.

---

## 7. Where reasonable Staff engineers would disagree

**7.1 Per-region limits instead of global.** A large fraction of real
products define rate limits per region precisely because global enforcement
is expensive, and customers barely notice. If you can get the product
requirement changed to "1,000/minute per region," the global allocator
disappears entirely and the design collapses to something much simpler.
Proposing this — with the arithmetic on what it saves — is arguably a better
Staff answer than building the allocator. I've kept global enforcement here
because the billable quota genuinely is global (a monthly allowance is a
contractual number, not a per-region one), but a design that splits the
difference — regional protective limits, global billable quotas — is
defensible and cheaper, and I'd have a hard time arguing against it.

**7.2 Failing open on billable quotas.** Giving away calls during an outage
is a revenue decision, not an engineering one, and Finance may reasonably
prefer failing closed on an exhausted account. My position is that the
recorded usage preserves the revenue and the ceiling is a courtesy, but a
company whose margin depends on the ceiling would see it differently.

**7.3 At-least-once usage records.** Dedup by request id costs storage and
processing at 36M/s. An alternative is at-most-once with an accepted loss
rate, which is cheaper and — if the loss is under 0.1% and *biased toward
undercounting* — is arguably fine, because undercounting favours the
customer and never produces a dispute. That's a genuinely attractive
simplification and I could be talked into it.

**7.4 Sampling the decision audit trail.** You cannot log 36M decisions/s.
I'd log all *rejections* and sample allows, which means a customer disputing
a rejection can always be answered but a customer disputing an *allow* can't
be. Someone could reasonably argue the opposite priority.

**7.5 Whether internal traffic belongs here at all.** I'd put internal
limiting at the mesh/sidecar with the same control plane and accounting but
a different enforcement point, because the edge fleet isn't in that path.
That's two enforcement implementations sharing one control plane, which is
more code than one. A single implementation deployed in both places is
cleaner and slower to build. Reasonable people differ.

**7.6 Whether to build this at all.** Nobody asked whether 40 limiters is
actually costing more than one plane will. The tidiness argument didn't get
funded; the billing argument did. A narrower project — *fix the billing
number, leave the 40 limiters alone* — would deliver the funded outcome in a
fraction of the time, and a Staff engineer should at least price that option
before designing a fleet-wide plane. Opening the design with "here is the
cheaper thing that solves the funded problem, and here is why I'm still
recommending the bigger one" is doing the job properly.
