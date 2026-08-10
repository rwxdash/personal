# 09 — The Bare-Metal Fleet Engine

**Difficulty:** Staff+
**Format:** Take-home + 45-minute live extension — read
[TAKEHOME-FORMAT.md](../../TAKEHOME-FORMAT.md) first
**Roles this maps to:** Senior Infra Engineer — Baremetal Orchestration /
Datacenters

---

## The prompt

Written in the same shape as the two real ones, for the bare-metal and
datacenter openings.

> Imagine a theoretical or actual system like Railway which runs on hardware
> it owns. **Design the engine that turns racks of machines into schedulable
> capacity, and keeps them that way.**
>
> *Pre-work: complete your solution. 0–5m introduction. 5–50m building (or
> expanding) your solution. 50–60m questions.*

**Stop here if you're working in realistic mode.**

---

## What you have to invent

- **Where does this engine begin?** At a purchase order? A pallet on a
  loading dock? A machine that has power and a network cable? Pick a
  boundary and defend it.
- **What does "schedulable capacity" mean?** A machine that boots? One that
  has joined the fleet and had its network configured? One you would trust
  with a customer's production database? These are days apart, and the
  distance between them is a decision you have to make.
- **What is the lifecycle?** Provision, burn-in, serve, degrade, drain,
  repair, decommission — and which transitions are automatic versus which
  need a human with a screwdriver.
- **Is capacity planning in scope?** On owned hardware there is no
  autoscaling group, and lead times are measured in months. Somebody has to
  decide what to buy and when. If that's not your engine, say whose it is.
- **What is a failure domain, and who knows about them?** Rack, PDU, switch,
  hall, site. The compute and storage schedulers need this information and
  it comes from here.
- **What scale?** Pick, state, derive.

---

## Reference scale

*Plausible figures for a platform of this shape — not any real company's
published data.*

**Today**

| | |
| --- | --- |
| Machines | ~3,000 |
| Sites | 4 (a mix of colo halls and leased cages) |
| Machines per rack | ~40 |
| Racks | ~75 |
| Power per rack | ~20 kW |
| Machine cost, amortised | ~$15,000 over 4 years |
| Hardware generations in service | 3, with different CPU, RAM and NVMe profiles |

**Growth and lead time**

| | |
| --- | --- |
| Fleet growth | **~8% per month** |
| Order → racked and schedulable | **12–20 weeks** |
| Therefore | you commit capital for demand ~4–5 months out |
| Fleet in 6 months at current growth | ~4,760 machines |
| Machines to be delivered in that window | ~1,760 |

**Failure and churn**

| | |
| --- | --- |
| Machine-level failures | ~3%/year |
| Drive failures | ~1%/year across ~24,000 drives |
| Kernel/firmware update cadence | you decide, but security patches are not optional |
| A machine holds | ~67 customer containers, some with volumes attached |

**Two properties that are not negotiable**

1. **You cannot buy capacity on demand.** The elasticity you sell to
   customers is manufactured from a fixed fleet. If you are short, you turn
   away revenue for months; if you over-buy, you have idle capital measured
   in millions.

2. **Every machine you take out of service is carrying paying customers.**
   Draining is not free, it is not instant, and for machines with volumes
   attached it may take hours.

---

## What to produce

Produce **the artifact you would actually bring to the call** — see
[TAKEHOME-FORMAT.md](../../TAKEHOME-FORMAT.md).

Create `DESIGN.md` in this directory. Use [WORKSHEET.md](WORKSHEET.md) for
coverage, and **do Part 10**.

When a round is ready, say so and a review will land in `reviews/round-1.md`.
