# 01 — The Alerting Plane

**Difficulty:** Staff
**Theme:** Multi-tenant metric ingestion and alert evaluation at global scale
**Estimated effort:** 6–10 hours of writing, across several sessions

> **Do not start designing until you have completed Part 1 of
> [WORKSHEET.md](WORKSHEET.md).** The brief is deliberately incomplete. If
> your design begins before your assumptions are written down, you are
> designing for a problem nobody asked you to solve.

---

## Context

*(This is written as the internal design request would actually arrive —
from a director, with the gaps a real request has.)*

---

**From:** Director of Engineering, Platform
**To:** Staff Engineer, Observability Platform
**Subject:** Alerting plane — design review needed before Q3 planning

We need a design for the next generation of the alerting plane. The current
one is at its limits and Sales has committed to numbers we cannot serve on
the existing architecture.

Some background in case you haven't worked on this part of the system. We
sell an observability product. Customers deploy collection agents — some run
in their own infrastructure, some are our globally distributed probe fleet
running synthetic checks from ~90 countries. Those agents emit metrics to
us. Customers define alert rules over those metrics ("p95 latency from
Frankfurt above 400 ms for 5 minutes"), and when a rule fires we notify them
through their configured channels. That last part — notification routing,
deduplication, and delivery — is owned by another team and is not what I'm
asking you about. I want the plane that gets data in and decides when a rule
has fired.

Where we are today: a sharded Prometheus-derived stack with a ruler tier
that queries the same storage the dashboards query. It works, but the ruler
tier is now bigger than the ingest tier, our largest tenant's rules
routinely delay evaluation for everyone else on the same shard, and we have
had three incidents this year where alerts fired late enough that customers
noticed the outage before we did. Twice we have had to manually reconstruct
whether a customer's SLA was breached, from raw data, because we could not
answer it from the alerting system's own records. That is not a position I
want us in again — those conversations end in credits.

The hard numbers I have:

- **Peak ingest today: 4M samples/second** at the edge; roughly 2M/s
  average. We are planning for **3x within 18 months**.
- **180M active series**, growing faster than sample rate because customers
  add dimensions faster than they add checks.
- **~400,000 alert rules** across **~3,000 tenants**. Rule count per tenant
  follows the distribution you'd expect: our largest tenant has just over
  50,000 rules and roughly 60% of one region's series; the median tenant has
  40 rules.
- **Detection SLO: p95 within 45 seconds.** This is in three enterprise
  contracts, so it is not negotiable downward.
- **Retention: 15 months.** Contractual. Full resolution for the first 7
  days; after that it's up to us.
- **Budget: the plane currently costs about $2.1M/year. I can get to $3M for
  the 3x. I cannot get to $6M.**

Two things about the collection fleet that make this harder than a textbook
metrics pipeline. First, a meaningful fraction of agents sit on networks we
don't control — customer datacentres, edge sites, and probes in countries
with unreliable transit. When an agent loses connectivity it **buffers
locally, for up to 6 hours, and flushes everything when the link comes
back**. During last November's transit incident, one region went dark for
just over 4 hours and then delivered its entire backlog in about 20 minutes.
Our ingest tier fell over, which turned a regional collection problem into a
global outage. Second, agents timestamp their own samples, and their clocks
are only as good as their NTP setup.

We also have European customers asking pointed questions about where their
data lives. I don't have a firm requirement for you yet — legal is still
working on it — but I'd rather you didn't design something that makes it
impossible.

What I want from you: a design that gets us to 3x, holds the detection SLO,
stops one tenant from degrading another, and lets me answer "did this
customer's alert fire when it should have" with evidence rather than
archaeology. Tell me what you're not solving and why. Tell me what breaks
first at 10x.

I don't need a decision on every open question — I need to see which ones
you identified and how you resolved them.

---

## What to produce

Create `DESIGN.md` in this directory. It's yours; nobody else writes to it.

Work through [WORKSHEET.md](WORKSHEET.md) as you go — Part 1 **before** you
start designing, the rest alongside. The worksheet is the interview
structure; the design doc is the artifact.

When a round is ready, say so and a review will land in `reviews/round-1.md`.
Expect 2–3 rounds. The final round is graded.
