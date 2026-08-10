# Model Discussion — 03 Inventory & Reservations

> **Spoiler.** Do not open until the final review round is written.

One credible design, why its main alternative loses, and where competent
Staff engineers would still disagree.

---

## 1. Three numbers that decide everything

**1. The drop is 5,600 attempts per unit.** 250,000/s × 90 s = 22.5M
attempts for 4,000 units. **99.98% of the load must be rejected.** A drop is
not an inventory problem; it is an admission-control problem with a small
inventory problem hiding inside it. Any design that sizes a datastore for
250k reservations/second has misread what the system is being asked to do —
it needs to reject 250k/s cheaply and serve 4,000 correctly.

**2. Zero cross-region round trips fit the budget.** Inventory gets 50 ms;
US–EU RTT is ~70–90 ms and US–APAC is ~150 ms+. Authority must be local to
the region that serves the request. This is not a preference.

**3. Consistency cannot exceed 1.5%.** The WMS is the physical source of
truth, is 15 minutes stale, and is ~1.5% wrong. On a SKU with 40 units,
1.5% is 0.6 units — so a linearizable inventory system and an
eventually-consistent one with a sensible buffer produce *the same customer
outcome* most of the time, at wildly different cost. Buying strict
consistency for the long tail is buying precision that the input doesn't
have.

That third number is the one that reorganises the design, and it's the one
the brief hints at without stating ("I suspect that's the real answer and
nobody has wanted to say it").

---

## 2. Two regimes

| | **Tail** (79.99M SKUs) | **Drop** (a handful, scheduled) |
| --- | --- | --- |
| Rate | 40k/s across everything | 250k/s on one key |
| Stock knowledge | WMS, 15 min stale, ~1.5% error | Counted deliberately at receipt, exact |
| Oversell cost | ~$40 | much worse (loud customers, press) |
| Correct policy | soft allocation + buffer, regional ownership | hard partitioned allocation + admission control |
| Coordination affordable? | No — 80M SKUs × 3,012 locations | Yes — it's one key, and we know when |

One system, two policies, selected by a per-SKU flag that marketing's drop
schedule sets days in advance. The fact that drops are *scheduled* is the
single most exploitable fact in the brief, and it appears twice.

---

## 3. The design

```
                        customer request
                               │
      ┌────────────────────────▼─────────────────────────┐
      │  EDGE / API (regional)                            │
      │   · drop SKU? → admission gate (cached "remaining"│
      │     estimate, refreshed ~200ms) → reject ~instantly│
      │     once provably exhausted                       │
      └────────────────────────┬─────────────────────────┘
                               │ (only requests that might succeed)
      ┌────────────────────────▼─────────────────────────┐
      │  RESERVATION SERVICE (regional, no cross-region   │
      │  calls on the hot path)                           │
      │   · idempotency key = (customer, sku, request_id) │
      │   · reserve against LOCAL allocation              │
      └───────┬───────────────────────────────┬──────────┘
              │                               │
   ┌──────────▼───────────┐      ┌────────────▼─────────────┐
   │ ALLOCATION STORE      │      │ RESERVATION STORE         │
   │ (per region)          │      │ · durable, TTL'd          │
   │ · this region's units │      │ · idempotent by key       │
   │ · per (sku, location) │      │ · ~48M live records       │
   │ · drop SKUs: N shards │      └────────────┬─────────────┘
   └──────────┬────────────┘                   │
              │ async                          │ CDC
   ┌──────────▼──────────────────────────────┐ │
   │ GLOBAL ALLOCATOR                         │ │
   │ · grants regional blocks                 │ │
   │ · reclaims stranded/unsold               │ │
   │ · owns the safety buffer per SKU         │ │
   └──────────┬───────────────────────────────┘ │
              │                                 │
   ┌──────────▼───────────┐      ┌──────────────▼───────────┐
   │ WMS RECONCILER        │      │ monolith Postgres table   │
   │ · treats feed as a    │      │ (kept in old shape via    │
   │   PROPOSAL not a write│      │  CDC during migration)    │
   │ · per-SKU drift stats │      └──────────────────────────┘
   └──────────▲────────────┘
              │ 15-min feed
        [ vendor WMS ]
```

### 3.1 Regional ownership, not distributed consensus

Each (SKU, location) pair has **exactly one owning region** — normally the
region geographically nearest that location, since a Lyon customer can only
have Lyon-area stock anyway. There is one writer, so there is no conflict to
resolve, no consensus to run, and no cross-region call on the hot path.

This works because **inventory has natural geographic locality**, which is
the property that makes per-region ownership the right answer here rather
than CRDTs (no invariant possible) or last-writer-wins (silent loss —
interviews 07·A15).

Cross-region demand — a US customer wanting an EU-held unit — is rare, and
is served by an explicit cross-region reservation call that is *allowed* to
be slow, because it's off the common path. Naming that exception rather than
pretending it doesn't exist is part of the answer.

### 3.2 The safety buffer is per-SKU, not global

Available-to-promise = WMS count − outstanding confirmed − **buffer(SKU,
location)**.

The buffer is computed from that SKU/location's observed drift history, its
velocity, and its oversell cost. A slow-moving SKU with stable counts gets a
buffer of zero. A fast-moving SKU in a warehouse with 3% historical drift
gets a meaningful one.

A **flat 1.5% buffer across 80M SKUs would strand an enormous amount of
sellable stock** — 1.5% of inventory permanently unsellable, at $18 of lost
margin per unit — which is precisely the undersell cost the brief priced.
Making the buffer adaptive is where the economics are won, and the drift
statistics needed to compute it are a useful operational signal in their own
right (a warehouse whose drift is worsening is a physical problem worth a
ticket).

### 3.3 The drop: partition the stock, reject at the edge

The drop stock is loaded into the allocation store **before the drop**,
split into N shards (say 64 for 4,000 units, ~62 each), each independently
decrementable. Contention falls by 64x with zero coordination. Customers
hash to a shard by customer id, so a retry lands on the same shard and the
idempotency key does its job.

**The admission gate is what actually handles the 250k/s.** A cached
"units remaining" estimate, refreshed every ~200 ms, sits at the edge. Once
it is provably zero, requests are rejected there at the cost of a memory
lookup — never reaching the reservation service. During the 90 seconds this
is what keeps 22.5M attempts off the datastore.

**The last-unit problem** is real and must be handled: with partitioned
stock, some shards empty while others still hold units, so a customer
hashed to an empty shard is refused while stock remains. The fix is
**progressive consolidation** — as the total falls below a threshold (say
200 units), shards are merged, so the final units are served from a small
number of shards where contention is now affordable because the request rate
has also collapsed (most customers have already been rejected). The
mechanism changes as the contention profile changes, which is the point.

### 3.4 Reservations are soft, and abandonment is priced

With 70% abandonment, holding hard stock for 20 minutes means ~70% of held
stock is dead inventory at any moment. For the tail, reservations are **soft
holds against the buffered ATP** rather than hard decrements, with an
overcommit factor derived from that SKU's observed conversion rate — the
airline model, bounded.

The bound matters: overcommit is capped so that worst-case oversell (every
hold converting) stays inside the economic target, and drop SKUs are
excluded entirely (conversion on a drop is near 100%, so overcommitting
would be catastrophic). This is the highest-return and highest-risk idea in
the design, and it should be shipped last, behind a flag, per SKU class.

### 3.5 The WMS feed is a proposal

The reconciler never overwrites. It computes a new ATP from the feed and:
- **Never invalidates a confirmed order.** If WMS says 40 and 43 are
  confirmed, the 3 become *fulfilment exceptions* with a defined customer
  path (substitute, backorder, or refund + credit). The **rate** of those
  exceptions is the metric that tells you the buffer is wrong.
- **Validates the feed.** An all-zeros file, or a delta beyond a threshold
  for a whole location, is quarantined and paged rather than applied. The
  brief tells you the input is unreliable; a design that would zero the
  catalogue on one bad file has ignored that.
- **Updates drift statistics** per (SKU, location), which feed the buffer.

---

## 4. Partition behaviour, answered with money

**Each region keeps selling — from its own allocation only.**

This is safe by construction rather than by luck: a region can only sell
units that were granted to it, and no other region holds those units. A
partition means the global allocator can't rebalance, so a region may run
out while another has surplus — that's *undersell*, at $18 per unit, and it
is strictly better than the alternative.

The failure the brief describes — three regions each thinking they had all
4,000 — is impossible under allocation, because there is no shared pool to
race on. That's the structural answer to the incident, and it's worth
stating that the fix is the *ownership model*, not better locking.

Bound the damage: if a partition lasts long enough that a region exhausts
its block, it stops selling that SKU. Refusing sales at $18 each while
partitioned is a known, bounded, accepted cost, and it is roughly half the
cost of the oversell alternative.

---

## 5. Why the main alternative loses

**The alternative: a globally consistent inventory system. Consensus per
(SKU, location) — Spanner-class or a Raft group per key — with strict
serialisability and no oversell by construction.**

It loses on three grounds:

1. **Its precision is swamped by the input.** The WMS is ~1.5% wrong. A
   system that guarantees zero oversell against a count that is already
   wrong by 1.5% still oversells at roughly 1.5%. You have paid for
   consensus and bought a guarantee about a number, not about reality. This
   is the argument that ends it for the tail, and it is the insight the
   brief is built around.
2. **Latency.** Consensus with a quorum spanning regions costs a
   cross-region round trip on every reservation — 70–150 ms against a 50 ms
   budget. Keeping the quorum in-region means you've chosen regional
   ownership anyway, with more machinery.
3. **Cost at 80M SKUs × thousands of locations.** Hundreds of millions of
   consensus groups, each with a leader, is an enormous operational and
   monetary footprint for a workload where most keys see a reservation a
   month.

**Where the alternative genuinely wins**, and it's worth conceding: for the
**drop**, it's close to right. The stock count is exact, the SKU count is
tiny, the oversell cost is high, and the coordination is affordable because
it's one key. If someone proposed a consensus group per drop SKU and
regional soft allocation for everything else, that is a coherent and
defensible design that differs from mine mainly in mechanism, not in
philosophy — and it might be simpler to reason about. I've used partitioned
allocation for the drop because it degrades better under the 250k/s load
(no leader to overwhelm), but I would not argue hard.

---

## 6. Where reasonable Staff engineers would disagree

**6.1 Deliberate overcommit against abandonment.** Selling 130 units of 100
because 70% of holds are abandoned is economically optimal in expectation
and career-limiting in the tail. Finance will love it until the one day
conversion spikes. My position is that it's correct, bounded, flag-gated,
and excluded from drops; a reasonable Staff engineer would say the
reputational asymmetry (an oversell is a story, an undersell is invisible)
makes the expected-value calculation the wrong frame. That asymmetry is real
and is not in the $40 number.

**6.2 Fair queueing for drops.** I've designed a free-for-all with fast
rejection. Many retailers use a virtual waiting room with a queue token,
which is fairer, better PR, and turns 250k/s into a manageable trickle — at
the cost of a whole additional system and a worse experience for the
majority who queue and still get nothing. This is a product decision
masquerading as an architecture decision, and I'd want it made explicitly
rather than defaulted into.

**6.3 Whether to keep per-location granularity in the hot path.** Reserving
against a specific location at reservation time is correct but expensive;
reserving against a *regional aggregate* and choosing the location at
fulfilment is faster and occasionally wrong (the aggregate had units, the
reachable location didn't). Same-day delivery, mentioned in Part 8, forces
the strict version. If it's not on the roadmap, the aggregate is cheaper and
defensible.

**6.4 The buffer's complexity.** A per-SKU adaptive buffer is a small
machine-learning problem with an operational tail — it needs monitoring,
it can be wrong in interesting ways, and nobody will understand it at 3am. A
flat buffer with a manual override list for the top 10,000 SKUs captures
most of the value for a fraction of the complexity, and I would probably
ship that first and be talked out of the adaptive version if the numbers
didn't justify it.

**6.5 Whether the WMS cycle is negotiable.** I'd push hard for an
event-driven feed, at least for drop and high-velocity SKUs, and I'd put a
number on it: if 15-minute staleness forces a 1% buffer across the
fast-moving catalogue, the annual cost of that stranded stock is the budget
for the integration. Someone who has actually negotiated with a WMS vendor
would tell me that's a two-year conversation and I should design as if the
answer is no. They'd probably be right, which is why the design works
without it and improves with it.

**6.6 Whether any of this beats "buy a commerce platform."** Nobody asked
whether inventory is a differentiator. For a retailer, arguably it is — drop
mechanics and fulfilment logic are competitive. But a Staff engineer should
have priced the vendor option before designing a multi-region allocation
system, if only to be able to say why it doesn't fit.
