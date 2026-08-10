# Rubric — 03 Inventory & Reservations

> **Spoiler.** Do not open until the final review round is written.

---

## The central tension

**The system cannot be more correct than its source of truth, and its source
of truth is 15 minutes stale and 1.5% wrong.**

This is what makes the problem Staff-level rather than a CAP exercise. A
candidate can build a beautifully linearizable inventory system — consensus
per SKU, strict serialisability, zero oversell by construction — and it will
still oversell, because the WMS told it there were 40 units and there were
39. All that engineering bought precision on a number that was already
inaccurate by more than the precision gained.

So the real question is not "how do we never oversell" but **"how much
correctness is worth buying, and where do we spend it?"** The brief supplies
the numbers to answer that ($40 oversell, $18 lost sale, 1.5% WMS error) and
deliberately does not draw the conclusion.

The conclusion a strong candidate reaches: **strict consistency is the wrong
target for the long tail and the right target for the drop.**

- For 80M SKUs at 40k/s with 1.5% source error, a strongly consistent
  distributed inventory system costs far more than the oversells it
  prevents, and its precision is swamped by WMS drift. Regional soft
  allocation with a safety buffer is economically correct.
- For a 4,000-unit drop at 250k/s, the arithmetic inverts: the units are
  known exactly (a drop is received and counted deliberately), oversell cost
  per unit is much higher, and the contention is concentrated on one key
  where coordination is affordable *because it's one key*.

**Two regimes, one system.** A design that applies one policy uniformly is
either too expensive for the tail or too weak for the drop.

**Secondary tension:** under partition, a region that stops selling loses
$18 per refused sale; a region that keeps selling risks $40 per oversell.
With three regions and independent allocation, the worst case is 3x the
stock sold. The brief provides everything needed to compute where the line
is, and "we choose consistency" or "we choose availability" as a *principle*
rather than as arithmetic is a Senior answer.

**Third tension, quieter:** the 70% cart-abandonment rate means most
reservations are never converted. Holding hard stock for 20 minutes for a
70%-abandoned cart is a large, permanent undersell. Overselling *deliberately
against expected abandonment* — the airline model — is the economically
optimal answer and is genuinely risky. Candidates who raise it are thinking
like the business; candidates who implement it without bounding the downside
are dangerous. Both reactions are informative.

---

## Ambiguities a strong candidate must catch

**Catching 6+ of the first tier is a Staff signal. Fewer than 4 caps at
Senior.**

### First tier — changes the architecture

| # | Ambiguity | Why it matters |
| --- | --- | --- |
| 1 | **Is any oversell acceptable, and how much?** | The brief supplies $40 and $18 and refuses to conclude. This is the question the whole design hangs on. |
| 2 | **Does a region keep selling during a partition?** | CAP made concrete and monetary. Never stated. |
| 3 | **Is a reservation a hard allocation or a soft hold?** | With 70% abandonment these are wildly different systems; the current one holds hard stock and therefore undersells constantly. |
| 4 | **Is the reservation atomic across a multi-item basket?** | Never mentioned. All-or-nothing across 6 SKUs on different shards is a distributed transaction; per-item is not. Enormous design consequence. |
| 5 | **Who wins when WMS and our allocation disagree?** | The brief says WMS is truth and also says WMS is wrong. Both cannot be operative. |
| 6 | **Is stock per-location or aggregated for the sell decision?** | "A customer in Lyon can't have a unit in Ohio" implies per-location, but availability display and basket feasibility may need an aggregate view. Different partitioning. |
| 7 | **What is the required freshness of displayed availability?** | Displaying "3 left" is a different consistency requirement from reserving. Conflating them means paying reservation-grade consistency on the read path, which is ~10x the volume. |
| 8 | **Is EU residency mandatory?** | "I don't have a requirement for you yet." A design that can't regionalise later is a bet; one that regionalises now pays immediately. Either is fine; not noticing is not. |

### Second tier — changes sizing or operations

| # | Ambiguity |
| --- | --- |
| 9 | Does the 20-minute TTL extend during checkout? What happens if payment takes 25 minutes? |
| 10 | Are drops fair-ordered (queue position) or free-for-all? This is a product decision with a large architectural consequence. |
| 11 | Backorder / pre-order semantics — can you sell stock you don't have yet? |
| 12 | Are returns/restocks in scope, and how fast do they re-enter availability? |
| 13 | Can a customer hold multiple units, and is there a per-customer cap on a drop? (Bot mitigation is adjacent and probably out of scope — but the cap is an inventory concern.) |
| 14 | Who is allowed to write to the monolith's Postgres table during migration? |
| 15 | Is the 50 ms inventory budget for one call or for the whole basket? |
| 16 | What happens to in-flight reservations during a deploy? |

---

## Section-by-section

### Part 1 — Requirements & scope

**Senior:** Restates requirements. May state "we must never oversell" as a
requirement without noticing that the WMS makes it unachievable.

**Staff:** Section 1.2 is answered with arithmetic. Notices that the $40/$18
ratio means an oversell is worth about 2.2 lost sales, so refusing more than
~45% of marginal sales to prevent one oversell is value-destroying — and
that this bound is generous enough to permit soft allocation for the tail.
Notices the WMS error bound *exceeds* the precision a consistent design
would add for typical SKUs, and says so.

Distinguishes the display path from the reservation path in the NFRs.

**Red flags:** "Never oversell" as an unexamined requirement. No engagement
with the $40/$18 numbers. Treating availability display and reservation as
one consistency requirement.

---

### Part 2 — Back-of-envelope

Recompute independently.

| Quantity | Defensible value | Working |
| --- | --- | --- |
| Normal peak attempts/s | **40,000/s** | Stated |
| (SKU, location) pairs | **up to ~240B nominal**, realistically ~100–500M non-zero | 80M SKUs × 3,012 locations is the nominal cross product; most SKUs are in few locations. A candidate who uses the nominal number without noticing the sparsity has made a 1000x sizing error |
| Live reservations | **~48M at any time** | 40k/s × 20 min TTL × 0.7 abandonment ≈ 33.6M expiring + converting; order tens of millions. The point is that reservation state is large and short-lived |
| Drop attempts/s on one key | **250,000/s** | Stated |
| Attempts per unit of stock | **~5,600 : 1** | 250k/s × 90 s = 22.5M attempts for 4,000 units |
| Single-row relational update rate | **~1,000–5,000/s** | Order of magnitude for a serialised hot row |
| **Factor over naive** | **50–250x** | And that's for the *row*; the attempts are 5,600x the stock |
| Attempts that must be rejected | **>99.98%** | 22.5M attempts, 4,000 succeed. **This is the reframe: a drop is not an inventory problem, it is an admission-control problem** |
| Cross-region RTT | **~70–90 ms** US–EU, **~150 ms+** US–APAC | Order of magnitude |
| **Sync cross-region ops in a 50 ms budget** | **Zero** | Same conclusion as problem 02, different domain |
| Units at risk under 3-way partition | **3x stock** | Each region allows the full count |
| Oversells in the $2.4M incident | **~57,000 units** | 61,000 orders − 4,000 real, at ~$40. Consistent with the stated numbers — a candidate should check this and note that the brief's numbers are internally coherent |
| Cost of one oversell vs one lost sale | **2.2 : 1** | $40 / $18 |

**Senior:** Gets the drop rate and notes it's large.

**Staff:** Computes **attempts-per-unit (5,600:1)** and draws the
conclusion — that 99.98% of the load is work you must *shed*, not serve, and
that the system's job during a drop is to reject fast and cheaply. This
reframe is the single highest-value insight in the problem. Also notices the
(SKU, location) sparsity, and that zero cross-region round trips fit the
budget.

**Red flags:** Using 80M × 3,012 as the state size. Sizing a database to
serve 250k successful reservations/s when only 4,000 can succeed. No
partition arithmetic.

---

### Part 3 — API & data model

**Senior:** Reserve/confirm/release with a TTL. Count per SKU.

**Staff:** Distinguishes **physical stock**, **available-to-promise**, and
**reserved** as three different quantities with different owners and
freshness. Reservations are idempotent with a client-supplied key — the
brief explicitly names retry-created duplicate holds as a cause of the
incident, so a design without idempotency keys has failed to read the brief
(interviews 02·A6).

Addresses multi-item baskets explicitly: either per-item reservation with
compensation on partial failure (a saga — interviews 07·A9), or an ordered
acquisition protocol to avoid deadlock, or an explicit statement that
baskets are non-atomic and what the customer experiences. Any of the three
is fine; silence is not.

Partition key: (SKU, location) is the natural unit, and the design must say
what happens when one key is 250k/s — which is where the hot-key treatment
from Deep Dive A attaches.

**Red flags:** No idempotency. One "quantity" field. No answer on baskets.

---

### Part 4 — High-level architecture

**Senior:** Reservation service → a fast store → async sync to Postgres and
WMS.

**Staff:** Shows the two regimes visibly — the normal path and the drop
path — and where the switch happens. Annotates the 50 ms budget per hop.
Shows the WMS ingest as a *proposal* that is reconciled, not a
write-through that clobbers live allocations. Shows the path back to the
monolith's Postgres table as a distinct, asynchronous concern with its own
lag budget.

**Red flags:** WMS feed writing directly over authoritative counts. A single
global inventory service in one region. Magic "inventory service" box with
no internal mechanism for the hot key.

---

### Part 5 — Deep dives

**A (hot SKU)** and **B (multi-region)** are the hardest and most coupled.
A+B is strongest. A+C is good — C is where the "lying source of truth"
insight lives. B+D avoids the drop, which is the incident that funded the
project; note it in the review.

#### A — The hot SKU

**Senior:** Redis with an atomic decrement, or a queue in front.

**Staff:** Recognises the admission-control reframe and designs for
rejection. The strong pattern:
- **Pre-allocate the drop stock into N partitions** ahead of time (the drop
  is scheduled — the brief says so twice). 4,000 units across, say, 64
  partitions = 62–63 units each. Each partition is independently
  decrementable at full speed, so contention drops by 64x with no
  coordination. Route by hash of customer id so a retry lands on the same
  partition (idempotency preserved).
- **Reject early and cheaply.** Once the aggregate is provably exhausted, a
  cached "sold out" flag at the edge rejects at ~zero cost. The expensive
  path must only be entered by requests that might succeed. Admission
  control based on a rapidly-updated "units remaining" estimate is what
  keeps the 250k/s off the datastore entirely.
- **The last-unit problem** must be addressed: with partitioned stock, some
  partitions empty before others, so customers hashed to an empty partition
  are refused while units remain elsewhere. Solutions: work-stealing between
  partitions as they drain, progressive consolidation as the total falls, or
  accepting a small undersell tail. A candidate who partitions without
  noticing this has a bug worth a probe.
- Fairness/queueing: if the product wants fair ordering, that's a queue with
  a token, which changes the design substantially. Worth naming as a product
  question.

**Strong signal:** noticing that the contention profile changes over the 90
seconds — near-uniform at the start, extreme scarcity at the end — and that
the mechanism should change with it.

#### B — Multi-region consistency

**Staff:** Zero synchronous cross-region operations fit the budget, so
authority must be local. Options, with the choice defended:
- **Per-SKU-per-location ownership** — a location's stock is owned by its
  nearest region, and there is no conflict because there is only one writer.
  This is the strongest general answer (interviews 07·A15) and it works
  because inventory *has* natural geographic locality.
- **Regional sub-allocation** — a global allocator grants each region a
  block of the stock; regions sell from their block without coordination and
  return unsold blocks. Handles drops well; needs a reclaim path for
  stranded allocation.
- Consensus per SKU (Spanner-class) — correct, and the latency and cost must
  be priced rather than dismissed.

Partition behaviour must be answered with the $40/$18 arithmetic: a region
that keeps selling from *its own allocation* is safe (nobody else can sell
those units), which is the key insight — **allocation makes partition-time
selling correct rather than risky**. The dangerous case is a region selling
from a *shared* pool during a partition, and the design should make that
structurally impossible.

On heal: reconcile allocations, and if both sides sold the same units,
detect it and compensate (the customer-facing apology path is part of the
design, not an afterthought).

#### C — Reconciling with a lying WMS

**Staff:** WMS counts are a **proposal**, not a write. The reconciliation
must:
- Never reduce available-to-promise below what has already been *confirmed*
  — you cannot un-sell a confirmed order, so a WMS correction downward
  becomes a fulfilment exception, not an inventory change.
- Apply a **safety buffer** that is *not uniform*: proportional to the SKU's
  observed WMS drift, its velocity, and the oversell cost. High-velocity,
  high-drift, high-cost SKUs get a bigger buffer. A flat 1.5% buffer across
  80M SKUs wastes enormous stock on slow movers and under-protects fast
  ones.
- Track **drift per location** as a first-class signal — a warehouse whose
  drift is worsening is an operational problem the design should surface, and
  it's the input to the buffer calculation.
- Answer what happens when WMS says 40 and 43 are allocated: the 3 become
  fulfilment exceptions with a defined customer experience (substitute,
  backorder, refund + credit), and the *rate* of those is the metric that
  tells you whether the buffer is right.

**Strong signal:** proposing to change the 15-minute cycle — event-driven
updates from the WMS, or at least a faster feed for drop and high-velocity
SKUs — and pricing it, including the organisational cost of a vendor
negotiation. The brief invites this explicitly ("I suspect that's the real
answer"); a candidate who ignores the invitation missed a direct hint.

#### D — Migration from the monolith

**Staff:** Incremental and reversible. The shape: new system runs in shadow
first, computing what it *would* have allowed and diffing against the
monolith's decisions; then per-SKU-class or per-region cutover with the
monolith's table maintained by CDC from the new system; then reads migrate;
then the table becomes a projection. Divergence detection between the two
systems while both are live is the part usually missing, and it's the part
that makes the rollback safe.

Note the constraint that order management, returns, and finance close read
the table — so the table's *schema and semantics* must be preserved even
after authority moves, which argues for CDC-into-the-old-shape rather than
asking six consumers to change.

---

### Part 6 — Failure modes

**Staff:** Quantified blast radius. The partition scenario must be answered
with money, not principle. The **"reservation store loses 2 seconds on
failover"** case is the one that exposes whether they understood that Redis
with a TTL is not durable — losing reservations means overselling those
units, and the design needs either durability or a bounded, priced
acceptance.

The **WMS delivers all zeros** case tests whether they validate an input
they've already been told is unreliable. A design that would take the whole
catalogue to zero stock on one bad file has a serious gap.

**Deploy mid-drop** should ideally be answered with "we don't" plus an
enforced change freeze — an operational control is a legitimate design
element.

---

### Part 7 — Tradeoffs ledger

**Staff:** At least one row where the accepted cost is stated in dollars.
The oversell bound, the safety buffer's undersell cost, and the partition
policy should all appear with numbers.

---

### Part 8 — Evolution

**Staff:** Names the binding constraint at 10x with a number. Candidates:
reservation state volume (~480M live reservations), the WMS feed's ability
to describe a larger catalogue in 15 minutes, the allocation control loop's
fan-out, or the multi-item basket coordination cost.

The WMS question deserves a real proposal with a cost and a named
counterparty — this is the "organisational reality" dimension and a
technically strong answer that treats a vendor contract as immovable is
weaker than one that says "here is what it's worth to change it."

---

### Part 9 — Operations & cost

**Staff:** The pre-drop checklist and abort criterion are the tell. A drop
is a scheduled, high-stakes, repeatable event; a mature answer treats it
like a launch — pre-warmed capacity, pre-allocated partitions, a freeze, a
dashboard, a named decision-maker, and a documented abort. A candidate who
designs the mechanism but not the operation has half the answer.

The "item vanished from my cart" question requires a per-reservation audit
trail, and at tens of millions of reservations that has a cost worth
stating.

---

## Grading calibration

| Grade | Profile |
| --- | --- |
| **Below Senior** | Single global inventory table with row locks. No hot-key treatment. Oversell treated as impossible. WMS treated as reliable. |
| **Senior** | Correct reservation service with idempotency and TTLs. Recognises the hot key and proposes Redis/atomic decrement. Multi-region mentioned. Handles component failures. Doesn't engage with the $40/$18 economics or the WMS error bound; picks a consistency stance as a principle. |
| **Borderline Staff** | Computes attempts-per-unit and treats the drop as admission control, or engages the economics — but not both. One deep dive strong. Partition answered but not priced. WMS reconciliation present but with a flat buffer. |
| **Staff** | Derives the oversell target from the economics. Reframes the drop as admission control with the 5,600:1 number. Zero-cross-region-hops conclusion. Per-location ownership or regional allocation, with partition behaviour justified by money. WMS as a proposal with a non-uniform buffer. Idempotency throughout. Migration incremental and reversible. Catches 6+ first-tier ambiguities. |
| **Strong Staff** | All of the above, plus: notices that strict consistency cannot exceed the WMS's 1.5% error and uses that to justify the two-regime design explicitly; pre-allocates drop stock into partitions and handles the last-unit consolidation problem; treats the 70% abandonment rate as an economic opportunity with a bounded downside rather than ignoring it; proposes changing the WMS cycle with a cost and a counterparty; and designs the drop *operation* — freeze, checklist, abort criterion — not just the mechanism. |

---

## Review guidance

**Round 1:** Recompute Part 2. The highest-value probes: "you have 22.5M
attempts for 4,000 units — what is the system's job for the other 22.496M?",
"the WMS is 1.5% wrong; what does your consistency guarantee buy you on a
SKU with 40 units?", "a region is cut off mid-drop — sell or stop, and show
me the arithmetic", and "a customer retries a timed-out reservation — walk
me through it." Do not reveal the two-regime insight; ask whether the same
policy is right for a drop and for a slow-moving SKU and let them get there.

**Round 2+:** Commonly dodged: the last-unit problem under partitioned
stock, what happens when WMS contradicts a confirmed order, and the
migration's divergence detection. Escalate those.

**Final:** The highest-leverage improvement is usually one of: "you designed
for correctness you can't have, given the WMS," "the drop is an
admission-control problem and you built a database for it," or "you chose a
consistency stance without using the numbers you were given."
