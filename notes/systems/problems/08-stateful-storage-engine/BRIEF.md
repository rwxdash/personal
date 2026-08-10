# 08 — The Stateful Storage Engine

**Difficulty:** Staff+
**Format:** Take-home + 45-minute live extension — read
[TAKEHOME-FORMAT.md](../../TAKEHOME-FORMAT.md) first
**Role this maps to:** Senior Infra Engineer — Storage

---

## The prompt

Written in the same shape as the two real ones, for the Storage opening.

> Imagine a theoretical or actual system like Railway which can manage
> stateless and stateful compute workloads. **Design the engine that
> provides durable storage to stateful workloads.**
>
> *Pre-work: complete your solution. 0–5m introduction. 5–50m building (or
> expanding) your solution. 50–60m questions.*

**Stop here if you're working in realistic mode.**

---

## What you have to invent

- **What is a volume, as a product?** A block device? A filesystem? An
  object namespace? A managed database the customer never sees the disk of?
  These are four different companies.
- **What durability do you promise, and to whom?** Is it the same for a
  hobby project and a production database? Tiered durability is a real
  product decision and most candidates never consider it.
- **Local or network-attached?** This is the decision the whole design turns
  on and both answers are used in production by serious platforms.
- **Whose job is replication?** Yours, or the customer's database's? A
  Postgres knows how to replicate itself. Do you replicate underneath it,
  and if so are you paying for it twice?
- **What happens when the machine dies?** Not *if*.
- **Backups, snapshots, restore, resize, fork** — which are in scope?
- **What scale?** Pick, state, derive.

---

## Reference scale

*Plausible figures for a PaaS of this shape — not any real company's
published data.*

**The fleet**

| | |
| --- | --- |
| Machines | ~3,000 bare metal, 4 sites |
| NVMe drives per machine | 8 |
| **Total drives** | **~24,000** |
| Drive annualised failure rate | ~1% |
| Machine-level failure rate (PSU, board, NIC, kernel) | ~3%/year |
| Network | 25 Gbps per machine |

**The workload**

| | |
| --- | --- |
| Volumes | ~200,000 |
| Volume size, p50 / p99 | 1 GB / 100 GB |
| Total stored | ~2 PB logical |
| Volumes per machine | ~67 |
| What's on them | customer Postgres, MySQL, Redis, Mongo, uploaded files, and a long tail of things people should not run on a volume |
| IOPS profile | mostly idle; a small number of production databases doing thousands of IOPS sustained |
| Growth | volumes are never smaller next month |

**The contracts**

| | |
| --- | --- |
| Attach latency (volume available to a starting container) | should not dominate the wake budget from problem 07 |
| Durability | you decide — and defend the number |
| Backup | customers expect point-in-time restore of a database |
| Resize | customers do this live, at 3am, when the disk fills |

**Two properties that are not negotiable**

1. **Hardware fails on a schedule you can compute.** With 24,000 drives at
   ~1% AFR and 3,000 machines at ~3%, something breaks most days. Do the
   arithmetic before you design anything — it is the whole problem.

2. **This is someone's database.** A stateless container that dies is
   rescheduled and nobody notices. A volume that is lost is a customer's
   production data, permanently, and it is the kind of incident companies do
   not recover their reputation from.

---

## What to produce

Produce **the artifact you would actually bring to the call** — see
[TAKEHOME-FORMAT.md](../../TAKEHOME-FORMAT.md).

Create `DESIGN.md` in this directory. Use [WORKSHEET.md](WORKSHEET.md) for
coverage, and **do Part 10**.

When a round is ready, say so and a review will land in `reviews/round-1.md`.
