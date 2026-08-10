# 06 — The Observability Engine

**Difficulty:** Staff
**Format:** Take-home + 45-minute live extension — read
[TAKEHOME-FORMAT.md](../../TAKEHOME-FORMAT.md) first
**Role this maps to:** Senior Infra Engineer — Observability

---

## The prompt

This is the entire brief, as you would receive it.

> Imagine a theoretical or actual system like Railway which can manage
> stateless and stateful compute workloads. **Design the engine for managing
> observability.**
>
> *Pre-work: complete your solution. 0–5m introduction. 5–50m building (or
> expanding) your solution. 50–60m questions.*

**Stop here if you're working in realistic mode.** Establish your own scale,
constraints and scope, produce your artifact, and only then read on.

---

## What you have to invent

Nothing below was given to you. Every line of it is a decision you must make
and state, and the interviewer will ask about several of them:

- **Who is the customer of this system?** The platform's users, looking at
  their own service's logs in a dashboard? Or the platform's own engineers,
  operating the fleet? Both? They want different things and the answer
  reshapes the design.
- **What signals?** Logs, metrics, traces, events, profiles — all of them,
  or a defensible subset?
- **Is this a product feature or a cost centre?** If customers pay for it,
  it has an SLO and a price. If it's internal, it has a budget and no
  revenue.
- **What is "managing" observability?** Collection, storage, query,
  alerting, retention, quota enforcement, billing — where does the engine
  start and stop?
- **What scale?** No numbers were given to you. Pick them, state them,
  derive from them.
- **Stateless *and* stateful workloads** is in the prompt for a reason. What
  is different about observing a database the customer deployed versus a
  stateless web service?

---

## Reference scale

*Use this in guided mode, or compare against your own numbers afterwards.
These are plausible figures for a PaaS of this shape — they are not any real
company's published data, and you should say so if you use them.*

**The platform**

| | |
| --- | --- |
| Customers / projects | ~200,000 |
| Deployed services | ~500,000 (a mix of stateless apps and customer-run databases) |
| Fleet | ~3,000 bare-metal machines across 4 sites |
| Containers created per day | ~2,000,000 (deploys, restarts, crashes, scale events) |
| Typical service lifetime | minutes for a preview environment, months for production |

**The signal volume**

| | |
| --- | --- |
| Log lines/second, peak | ~10,000,000 |
| Average line size | ~150 bytes |
| **Raw log ingest** | **~1.5 GB/s ≈ 130 TB/day** |
| Distribution | heavy power law — most services log almost nothing; the top 1% produce most of the volume |
| A single customer can produce | **500 MB/s** with no warning (a logging loop in a deploy) |
| Metrics | ~20M active series (CPU, memory, network, disk, restarts, per container) |
| Concurrent live-tail sessions | ~5,000 |
| Retention | 30 days paid, 7 days free tier |

**The money**

| | |
| --- | --- |
| Paying customers | ~20,000, average ~$50/month |
| Platform revenue | ~$1M/month |
| Observability's share of infrastructure spend | you decide — but if it exceeds compute, the business does not work |

**Two properties that are not negotiable**

1. **You do not control what customers log.** There is no agent you can
   configure on their side, no sampling you can ask them to apply, and no
   schema. A customer can deploy a service that writes 500 MB/s of
   unstructured text at 03:00 on a Sunday, and that is a legitimate use of
   the product.

2. **Containers die.** Constantly, and often unexpectedly — OOM kills,
   crashes, evictions, deploys. The log lines a customer most wants are the
   last ones a container wrote before it died.

---

## What to produce

Produce **the artifact you would actually bring to the call** — see
[TAKEHOME-FORMAT.md](../../TAKEHOME-FORMAT.md) for what form that should
take and why a polished static document is the wrong answer.

Create `DESIGN.md` in this directory (or a canvas, or both — your choice, it
is yours). Use [WORKSHEET.md](WORKSHEET.md) to make sure you have covered the
ground, and **do Part 10** — the live-extension drill — because that is the
part that prepares you for the 45 minutes that decide it.

When a round is ready, say so and a review will land in `reviews/round-1.md`.
Reviews for this format include a simulated extension block: you will be
pushed on a dimension and expected to redesign in place.
