# 03 — Inventory & Reservations

**Difficulty:** Staff
**Theme:** Multi-region inventory availability and reservation under contention
**Estimated effort:** 6–10 hours of writing, across several sessions

> **Do not start designing until you have completed Part 1 of
> [WORKSHEET.md](WORKSHEET.md).** This brief gives you numbers that most
> inventory briefs don't. They are there for a reason, and none of them is
> decoration.

---

## Context

*(Written as the design request would actually arrive — three days after an
incident.)*

---

**From:** VP Engineering, Commerce
**To:** Staff Engineer, Fulfilment Platform
**Subject:** Inventory — we need a real design before the next drop

You saw what happened on the 14th. For the people who didn't: we ran a
limited drop, 4,000 units of one SKU, and we took 61,000 orders for it. Some
of that was the reservation service timing out and the retry creating a
second hold. Some of it was the three regions each thinking they had the
full 4,000. Customer service spent nine days on it and we paid out about
$2.4M in credits and goodwill. The post-incident review concluded, roughly,
"the inventory system was never designed for this," which is true and not
useful.

I need a design for what replaces it. Constraints as I understand them:

**Catalogue and volume.** About 80M SKUs. 3,000 retail locations and 12
fulfilment centres, and stock is tracked per location because a customer in
Lyon can't have a unit that's sitting in Ohio. Normal peak is around 40,000
reservation attempts per second across the whole catalogue, which the
current system handles fine.

**Drops.** This is the part that breaks. On a scheduled drop, a single SKU
takes roughly **250,000 attempts per second for about 90 seconds**. We know
the schedule in advance — marketing publishes it weeks ahead — so this is
not a surprise, it's a Tuesday.

**Geography.** We serve from three regions: `us-east`, `eu-west`,
`ap-southeast`. Checkout has a p99 budget of 300 ms end to end and inventory
is allotted 50 ms of it. Our EU legal team has started asking where customer
data lives; I don't have a requirement for you yet.

**The warehouse system.** This is the thing people forget. Our WMS is the
physical source of truth and it is not ours — it's a vendor product, it
publishes stock counts on a **15-minute** cycle, and it is itself wrong.
Shrinkage, damaged units, mis-picks, and units that are physically present
but not yet received into the system. Our reconciliation team's own number
is that WMS counts are within about **1.5%** of physical reality on a good
day. So whatever you build, it is downstream of a source of truth that
already lies to us.

**The money.** Finance ran the numbers after the 14th so we finally have
them. An oversell costs us an average of **$40** — refund handling, a
goodwill credit, the support contact, and some amount of churn we can't
measure well. Refusing a sale we could have fulfilled costs an average of
**$18** in lost margin. Those are averages across the catalogue; on the drop
SKUs the oversell number is considerably worse because those customers are
loud.

**What exists now.** Reservations live in a Redis cluster with a 20-minute
TTL, written by the checkout service, and the authoritative count lives in a
Postgres table that the monolith also uses for six other things. About 70%
of holds expire without converting. The Postgres table is not going away
this year — order management, returns, and the finance close all read it.

What I want: a design that survives the next drop, doesn't oversell in a way
that costs us more than it saves, works in three regions, and has a
migration story that doesn't require the monolith to change on day one.
Tell me what you're explicitly not solving. Tell me what breaks at 10x.

And tell me what you'd do about the 15-minute WMS cycle, because I suspect
that's the real answer and nobody has wanted to say it.

---

## What to produce

Create `DESIGN.md` in this directory. It's yours; nobody else writes to it.

Work through [WORKSHEET.md](WORKSHEET.md) as you go — Part 1 **before** you
start designing, the rest alongside.

When a round is ready, say so and a review will land in `reviews/round-1.md`.
Expect 2–3 rounds. The final round is graded.
