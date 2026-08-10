# Worksheet — 03 Inventory & Reservations

Part 1 comes **before** any architecture.

Prose, tables, ASCII/mermaid diagrams. No code.

---

## Part 1 — Requirements & scope

### 1.1 Ambiguities in the brief

| # | Question | Why it changes the design | Your assumption |
| --- | --- | --- | --- |
| 1 | | | |

> At least 8. One of them determines whether you are building a strongly
> consistent system or an economically optimal one, and those are different
> systems.

### 1.2 The correctness question

The brief gives you $40 per oversell and $18 per lost sale, and does not
tell you what to do with them. Answer it.

- At what oversell rate does preventing oversells cost more than the
  oversells?
- Does that answer change for drop SKUs versus the long tail? Why?
- What is your stated target oversell rate, and how did you derive it?
- What would change your answer?

State plainly whether you are designing to **never oversell** or to
**oversell within a bound**, and what that decision costs.

### 1.3 Functional requirements

Include: what operations exist over inventory, what a reservation is, what
converts it, what expires it, and what the WMS integration must do in both
directions.

### 1.4 Non-functional requirements

| Property | Target | Source (stated / derived / assumed) |
| --- | --- | --- |
| Reservation latency (p99) | | |
| Availability of the reservation path | | |
| Oversell bound | | |
| Undersell / false-refusal bound | | |
| Durability of a confirmed reservation | | |
| Behaviour under region partition | | |
| Freshness of available-to-promise counts | | |

### 1.5 Explicitly out of scope

At least four, each with what you assume someone else guarantees.

---

## Part 2 — Back-of-envelope

Show the arithmetic.

### 2.1 Steady state

| Quantity | Calculation | Result |
| --- | --- | --- |
| Reservation attempts/s, normal peak | | |
| Distinct (SKU, location) pairs | | |
| Reservation records live at once (incl. 70% expiry) | | |
| Write rate to inventory state | | |
| Read rate for availability display | | |

Note the read:write ratio and what it implies about where you spend.

### 2.2 The drop

This is the calculation the design turns on.

| Quantity | Calculation | Result |
| --- | --- | --- |
| Attempts/s on one SKU | | |
| Attempts per unit of stock | | |
| Sustained single-row update rate a relational DB gives you (state your reference figure) | | |
| **Factor by which the naive design is over** | | |
| Attempts that must be *rejected*, not served | | |
| Cost of rejecting vs cost of queueing | | |

Then state what the drop actually is, in one sentence, as a systems problem
rather than a commerce problem.

### 2.3 Partition arithmetic

| Quantity | Calculation | Result |
| --- | --- | --- |
| Cross-region RTT (state your reference figures) | | |
| Inventory latency budget | | |
| **Synchronous cross-region ops that fit in budget** | | |
| Units at risk if all three regions independently allow during a partition | | |

### 2.4 The economics

| Quantity | Calculation | Result |
| --- | --- | --- |
| Cost of the 14th's incident | | |
| Oversells that $2.4M represents | | |
| Annual oversell budget you'd accept | | |
| Value of one drop, in margin | | |
| Engineering cost you can justify against it | | |

### 2.5 Infrastructure cost

| Component | Driver | Rough share |
| --- | --- | --- |
| | | |

---

## Part 3 — API & data model

### 3.1 Core operations

Availability query, reserve, confirm, release, expire, and whatever the WMS
sync path needs. Include idempotency semantics — the 14th's incident was
partly caused by a retry.

### 3.2 Entities

Stock, availability, reservation, allocation, location. What identifies
each? What is the difference between *physical stock*, *available to
promise*, and *reserved*?

### 3.3 Partitioning

- What is the partition key, and why?
- What does one SKU at 250k/s do to that choice?
- Is inventory partitioned by SKU, by location, or by (SKU, location)? What
  does each cost for a query like "can I fulfil this basket of 6 items"?
- Multi-item baskets: is a reservation atomic across items? What if it
  isn't?

---

## Part 4 — High-level architecture

Diagram plus one sentence per component. Show explicitly:

- The reservation path, with the latency budget annotated.
- Where authoritative count lives, per region.
- The WMS ingest path and what it is allowed to overwrite.
- The path back to the Postgres table the monolith still reads.
- What is different about a drop SKU, if anything.

---

## Part 5 — Deep dives

**Pick two.** Say which and why.

**Candidate A — The hot SKU.**
250,000 attempts/second against 4,000 units. Design the mechanism. Options
include partitioned allocation of the stock across shards or regions,
admission control in front of the reservation, queueing with a fair
ordering, optimistic reservation with asynchronous confirmation, or
something else. Quantify: what is the p99 under load, what fraction of
attempts are rejected and how fast, and what happens to the *last* unit —
the contention is not uniform across the 90 seconds.

**Candidate B — Multi-region consistency.**
Three regions, a 50 ms budget, and a partition that must not produce three
independent allocations of the same stock. Where does authority live? Is it
per-SKU, per-location, or per-region? What happens to a region that is cut
off — can it sell, and from what? What happens on heal when both sides
allocated? Include the arithmetic from 2.3.

**Candidate C — Reconciling with a lying source of truth.**
The WMS is 15 minutes stale and ~1.5% wrong, and it is the physical truth.
Design the reconciliation. What happens when WMS says 40 and you have
allocated 43? Who wins, and what does the customer experience? What is the
safety margin, is it uniform across SKUs, and how is it computed? What
signals tell you the WMS is drifting worse than usual?

**Candidate D — Migration from the monolith's table.**
The Postgres table is read by order management, returns, and the finance
close, and it isn't going away this year. Design the coexistence: dual
authority, dual write, or CDC in one direction. What is the cutover per
SKU or per region, what is the rollback, and how do you detect divergence
between the two systems while both are live?

---

## Part 6 — Failure modes

| Failure | Blast radius | How you detect it | Mitigation | Degraded behaviour |
| --- | --- | --- | --- | --- |
| A region partitions during a drop | | | | |
| The reservation store loses 2 seconds of writes on failover | | | | |
| WMS feed stops for 6 hours | | | | |
| WMS delivers a bad file (all counts zero) | | | | |
| A retry storm doubles reservation attempts | | | | |
| Confirmation succeeds but the response is lost | | | | |
| Reservation expiry job falls 30 minutes behind | | | | |
| Your own deploy, mid-drop | | | | |

Then answer: **under partition, does a region keep selling or stop?**
Justify with the numbers from 1.2, not with a principle.

---

## Part 7 — Tradeoffs ledger

| Decision | Chosen | Rejected | Cost accepted | Who feels it |
| --- | --- | --- | --- | --- |
| | | | | |

At least 10 rows. At least one where the accepted cost is money.

---

## Part 8 — Evolution

- What breaks first at 10x — component, limit, number.
- What breaks second.
- The migration story, with rollback at each step.
- If EU legal comes back and says customer and reservation data must stay in
  the EU, what changes?
- The brief hints that the 15-minute WMS cycle is "the real answer." What
  would you actually propose about it, what would it cost, and who would
  have to agree?
- What happens if the business asks for same-day delivery, which makes
  location choice part of the reservation decision?

---

## Part 9 — Operations & cost

- Dominant cost driver, with the number, and the most effective lever.
- Top 3 alerts. Why each is a page.
- **The 3am page:** it is 03:00 and a drop is starting in 20 minutes in
  APAC. Write what the on-call sees and what they can actually do.
- What is your pre-drop checklist, and what is the abort criterion?
- A customer says they had an item in their cart and it vanished. What do
  you need to answer them, and does your design produce it?
- What is the one operation you'd most want during an incident, and does
  your design allow it?
