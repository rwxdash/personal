# System Design Problem Index

Staff/Principal-level design problems, worked offline in writing, reviewed
across rounds against a rubric. No code — diagrams-as-text, tables, prose.

For rapid-fire verbal interview questions see [`../interviews/`](../interviews).
For algorithms see [`../dsas/`](../dsas).

## How to work a problem

```bash
cd problems/<NN>-<slug>

$EDITOR BRIEF.md          # the ask, deliberately underspecified
$EDITOR WORKSHEET.md      # Part 1 FIRST — before any architecture
$EDITOR DESIGN.md         # yours; create it
```

Then say a round is ready and a review lands in `reviews/round-1.md`.
Typically 2–3 rounds; the final one is graded on the leveling rubric.

**Do not open `rubrics/<NN>-<slug>/` until the final round is written.** It
mirrors `problems/` from outside it for exactly that reason.

## The rules of engagement

**Part 1 of the worksheet comes before the architecture.** Every brief is
underspecified on purpose, and the first-order Staff signal is interrogating
the requirements rather than answering the question as asked. A design that
starts at the diagram has already lost the marks that matter most.

**Show the arithmetic.** Every estimate gets its derivation. Reviews
recompute independently and will show you the delta. In three of these five
problems the arithmetic *is* the finding — it reveals that the stated
requirements don't compose.

**State what you are not solving.** An explicit out-of-scope list with the
assumption behind each item is worth more than another component in the
diagram.

**Expect the review to be blunt.** Every round contains critique, including
rounds where the design is strong.

## Problems

Difficulty increases with the number. Each turns on a **different class of
tension** — work them in order and you'll cover the space rather than
practising the same argument five times.

| ID | Theme | Difficulty | Core tension | Status |
| --- | --- | --- | --- | --- |
| [01-telemetry-alerting-plane](problems/01-telemetry-alerting-plane/BRIEF.md) | Multi-tenant metric ingest + alert evaluation at global scale | Staff | Alert evaluation can't be fast, cheap, and reproducible under late data — pick two | |
| [02-quota-enforcement-plane](problems/02-quota-enforcement-plane/BRIEF.md) | Global rate limiting and quota enforcement as a platform service | Staff | Exact enforcement needs coordination; there is no coordination budget. And the same system must bill | |
| [03-inventory-reservations](problems/03-inventory-reservations/BRIEF.md) | Multi-region inventory availability and reservation under contention | Staff | Consistency vs availability with money on both sides — and a source of truth that's already wrong | |
| [04-build-execution-platform](problems/04-build-execution-platform/BRIEF.md) | Multi-tenant compute for untrusted workloads under a cost ceiling | Staff+ | Isolation, utilisation, latency — the budget buys two | |
| [05-payment-ledger](problems/05-payment-ledger/BRIEF.md) | Correctness, auditability, and throughput in a system of financial record | Strong Staff | Immutable auditable record vs partitioned concurrent throughput — with no relax-and-reconcile escape hatch | |

*Status is yours to fill in: `not started` / `round N` / `graded — <level>`.*

## Take-home problems (06–09)

A **different interview format**: a two-sentence async prompt, then a
45-minute live session where you *extend* what you brought. Read
**[TAKEHOME-FORMAT.md](TAKEHOME-FORMAT.md) before starting any of them** —
the format rewards different preparation, and the main trap is bringing
something polished that you cannot edit live.

06 and 07 use **real prompts, verbatim**, from a PaaS company's infra
hiring loop. 08 and 09 are written in the same shape for two of their other
openings. All four are set in the same imagined platform, so they compose —
07's scheduler consumes 08's placement constraints, and both depend on 09's
failure-domain model.

| ID | Prompt | Difficulty | Core tension | Status |
| --- | --- | --- | --- | --- |
| [06-observability-engine](problems/06-observability-engine/BRIEF.md) | *"Design the engine for managing observability"* | Staff | Live tail, indexed search, and unbounded untrusted volume want incompatible things — and the volume is set by the customer | |
| [07-container-provisioning](problems/07-container-provisioning/BRIEF.md) | *"Architect a Container Provisioning Engine"* | Staff | Cold start, density, and isolation are hostile to each other — and stateful workloads destroy the placement freedom density needs | |
| [08-stateful-storage-engine](problems/08-stateful-storage-engine/BRIEF.md) | *"Design the engine that provides durable storage"* | Staff+ | Durability, performance, cost, placement freedom — four corners, you get two | |
| [09-baremetal-fleet-engine](problems/09-baremetal-fleet-engine/BRIEF.md) | *"Turn racks into schedulable capacity"* | Staff+ | Commit capital 4–5 months before you know if you need it; both errors are expensive in different currencies | |

**Roles these map to:** Observability (06), Platform / Compute /
Orchestration (07), Storage (08), Baremetal Orchestration / Datacenters
(09).

Reference numbers in these briefs are plausible for a platform of that
shape. They are **not** any real company's published figures — say so if you
use them, and do your own research on the actual company before an
interview.

### What each one is for

| # | If you want to practise | It will punish you for |
| --- | --- | --- |
| 01 | Event time vs processing time; read amplification; multi-tenant isolation | Not computing the read:write ratio; accepting a latency SLO without asking what its clock starts on |
| 02 | Latency budgets that forbid coordination; approximate vs exact | Proposing a central store without checking RTT; billing from an approximate counter |
| 03 | CAP made monetary; hot keys; admission control | Designing for correctness your inputs can't support; sizing a database for load you must reject |
| 04 | Cost modelling as a design constraint; hostile multi-tenancy | Using the p50 as the mean; not noticing the requirements don't compose |
| 05 | Bitemporal modelling; immutability; cross-shard invariants | Storing balances as mutable state; one timestamp per entry |
| 06 | Scoping a prompt that gave you nothing; log pipelines at volume | Not establishing scale; one pipeline for tail and search |
| 07 | Latency budgets; density vs isolation; scheduling constraints | Not budgeting the wake path; treating stateful like stateless |
| 08 | Failure arithmetic driving a design; durability tiers | Asserting durability instead of deriving it; backups you've never restored |
| 09 | Capacity under lead time; fleet lifecycle | Designing the pipeline instead of the decision; no forecast-error cost |

## Progress

| Date | Problem | Round | Grade | Highest-leverage improvement |
| --- | --- | --- | --- | --- |
| | | | | |

## Recurring ideas

The same handful of arguments decide all five problems. If you can state
these cold, most of the design work is recognition:

- **Compute the budget before choosing an architecture.** In 01 it's the
  read:write ratio, in 02 it's RTT against the latency budget, in 04 it's
  perfect-elasticity cost against the ceiling. In each, the number tells you
  which designs are already dead.
- **A hot key is not a scaling problem, it's a representation problem.**
  03 (one SKU), 05 (the fee account). Both dissolve when you stop updating a
  single mutable thing.
- **Separate the paths with different tolerances.** 01 (live vs backfill),
  02 (enforcement vs accounting), 04 (interactive vs scheduled), 05 (write
  vs projection vs as-of). Merging paths with opposite requirements is the
  most common structural mistake.
- **Ask which requirement gives.** 02, 03, and 04 all contain stated
  requirements that cannot all hold. Negotiating one, with arithmetic and a
  named counterparty, is a Staff move — not a failure to deliver.
- **Migration is the risky part, not the architecture.** All five replace a
  live system. Incremental, shadow-verified, reversible, per-tenant.

## Connections to the question bank

These assume the fundamentals from [`../interviews/`](../interviews). If a
deep dive stalls, the relevant verbal answer is often the missing piece.

| If you're stuck on | Read |
| --- | --- |
| Why utilisation and latency are related at all | interviews 07 · A11 (Little's law) |
| Where state should live and how it survives failover | interviews 07 · A6 (fencing), 07 · A12 (leases) |
| At-least-once vs exactly-once | interviews 07 · A8; 06 · A18 (outbox) |
| Blast radius and isolation units | interviews 07 · A14 (cells, shuffle sharding) |
| Partition behaviour of a state store | interviews 07 · A17; 06 · A10 (split brain, fencing) |
| Multi-region conflict strategies | interviews 07 · A15 |
| Kafka as a durable buffer | interviews 07 · A10 |
| Storage engine and columnar tradeoffs | interviews 06 · A7, 06 · A19 |
| Rate limiting mechanisms | interviews 03 · A13 |
| Backpressure, shedding, circuit breaking | interviews 03 · A14; 07 · A16 (metastable failure) |
| Hostile multi-tenancy in containers | interviews 08 · A11; 08 · A18 |
| Cache patterns, stampede, poisoning | interviews 06 · A13 |
| Cardinality and metric cost | interviews 09 · A5, 09 · A15 |
| Picking an SLI, burn-rate alerting | interviews 09 · A2, 09 · A9, 09 · A13 |
| Alert design and what deserves a page | interviews 09 · A10 |
| Migrating a live system without a flag day | interviews 06 · A15, 06 · A16, 10 · A8, 10 · A14 |
| Cost models and where the money goes | interviews 10 · A11; 04 · A13 (cross-AZ) |
| Capacity planning for something never run | interviews 10 · A17 |
