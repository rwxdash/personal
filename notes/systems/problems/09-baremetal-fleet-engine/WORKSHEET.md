# Worksheet — 09 The Bare-Metal Fleet Engine

---

## Part 1 — Scoping the unstated

### 1.1 What you decided the question was

Three sentences, as you'd say them in the first two minutes.

### 1.2 Decisions you had to make unprompted

| # | Question the prompt didn't answer | Your answer | Why |
| --- | --- | --- | --- |
| 1 | Where does the engine begin? | | |
| 2 | What does "schedulable" mean, precisely? | | |
| 3 | Is capacity planning in scope? | | |
| 4 | What is automated vs what needs hands? | | |
| 5 | Who owns failure-domain information? | | |
| 6 | Buy vs lease vs colo vs own the building? | | |
| 7 | What scale? | | |
| 8 | | | |

### 1.3 Explicitly out of scope

At least four, with boundaries.

### 1.4 The lifecycle

Define every state a machine can be in and every transition, including who
or what triggers it and how long it takes.

| State | Entered when | Exits to | Duration | Automatic? |
| --- | --- | --- | --- | --- |
| | | | | |

If any transition requires a human, say who and where they physically are.

---

## Part 2 — Back-of-envelope

**Do this before designing.** The economics are the problem.

### 2.1 Growth and lead time

| Quantity | Calculation | Result |
| --- | --- | --- |
| Fleet in 4 / 6 / 12 months at 8%/month | | |
| Machines to order for a 6-month horizon | | |
| Racks that represents | | |
| Power that represents | | |
| Capital committed | | |

### 2.2 The forecast error

This is the whole problem. Cost being wrong in both directions.

| Scenario | Machines short/long at month 6 | Cost |
| --- | --- | --- |
| Forecast 8%, actual 8% | | |
| Forecast 3%, actual 8% (under-buy) | | |
| Forecast 8%, actual 3% (over-buy) | | |

What does "short" actually cost? Convert it to revenue, not machines.
Then state which error you'd rather make and why.

### 2.3 Provisioning throughput

| Quantity | Calculation | Result |
| --- | --- | --- |
| Machines to bring online per month at current growth | | |
| Per week | | |
| Time to provision one machine, unattended | | |
| Concurrent provisioning your process supports | | |
| **Is your process fast enough to keep up with growth?** | | |

### 2.4 Churn and maintenance

| Quantity | Calculation | Result |
| --- | --- | --- |
| Machine failures/year and per week | | |
| Machines needing a kernel/firmware update per cycle | | |
| Drain time for a stateless-only machine | | |
| Drain time for a machine with volumes | | |
| **Machines in a non-serving state at steady state** | | |
| That, as a percentage of the fleet you paid for | | |

### 2.5 Capacity headroom

| Quantity | Calculation | Result |
| --- | --- | --- |
| Headroom for failure absorption | | |
| Headroom for maintenance | | |
| Headroom for growth between orders | | |
| Headroom for a single-rack loss | | |
| **Total, and its annual cost** | | |

---

## Part 3 — Data model & API

- What is the record of a machine? What does it know about itself, and what
  does the fleet know about it?
- How are failure domains represented, and who consumes them?
- What does the compute scheduler (problem 07) ask this system, and what
  does it get?
- How is hardware heterogeneity expressed so a scheduler can reason about
  it?
- What is the interface for "I need this machine back" and "this machine is
  suspect"?

---

## Part 4 — Architecture

Diagram plus one sentence per component. Show explicitly:

- The path from a machine arriving on a loading dock to being schedulable.
- The health signal path, and what it does automatically.
- The drain-and-return path.
- Where the source of truth for fleet state lives, and what happens when
  it's wrong.
- The network: addressing, fabric config, and how a new rack joins.

---

## Part 5 — Deep dives

**Pick two.**

**Candidate A — Provisioning from bare metal to schedulable.**
Design the automated path: discovery, firmware, network config, OS image,
identity, burn-in, and join. What is verified before a machine is trusted
with customer workloads, and what does burn-in actually test? How do you
provision a whole rack at once? What happens when a machine fails
provisioning at 2am — does anyone wake up?

**Candidate B — Capacity planning under lead time.**
You commit capital 4–5 months ahead of demand you cannot predict. Design the
system: what signals you forecast from, how you express uncertainty, what
you do when you're wrong in each direction, and what levers exist on a
shorter timescale than the lead time. Include the arithmetic on both failure
modes.

**Candidate C — Maintenance and drain at fleet scale.**
Kernel and firmware updates across 3,000 machines carrying paying customers,
some with volumes attached. Design it: batching, failure-domain awareness,
drain orchestration, rollback, and how long a full fleet cycle takes. What
is the maximum fraction of the fleet you may touch at once, and how did you
derive it?

**Candidate D — Failure domains and the blast radius contract.**
Rack, PDU, switch, hall, site. Design how this information is produced,
kept accurate, and consumed by the compute and storage schedulers. What
happens when the model is wrong — when two "independent" racks turn out to
share an upstream? How would you find that out before an incident rather
than during one?

---

## Part 6 — Failure modes

| Failure | Blast radius | Detection | Mitigation | Degraded behaviour |
| --- | --- | --- | --- | --- |
| A machine fails hardware diagnostics in service | | | | |
| A rack loses power | | | | |
| A top-of-rack switch fails | | | | |
| A site loses connectivity | | | | |
| A firmware update bricks machines | | | | |
| Your fleet database disagrees with physical reality | | | | |
| A shipment is 8 weeks late | | | | |
| Growth doubles unexpectedly | | | | |
| A whole hardware generation has a defect | | | | |

Then: **what is the largest single failure your design absorbs without
customer impact, and what does that absorption cost per year?**

---

## Part 7 — Tradeoffs ledger

| Decision | Chosen | Rejected | Cost accepted | Who feels it |
| --- | --- | --- | --- | --- |

At least 10 rows. At least one where the cost is capital and one where it's
a person doing something manual, forever.

---

## Part 8 — Evolution

- What breaks first at 10x — component, limit, number.
- What breaks second.
- What changes when you open a fifth site?
- What changes if you need GPUs, with their power and cooling profile?
- What changes if you move from colo to a building you operate?
- At what fleet size does a piece of this need to stop being a script and
  start being a product?

---

## Part 9 — Operations & cost

- Dominant cost driver, with the number, and the most effective lever.
- Top 3 alerts.
- **The 3am page:** a rack has lost power and is not coming back. Walk
  through it. What is automatic and what needs a human?
- How many people does your design require, and doing what?
- What is the one operation you'd most want during an incident?
- What is the metric that tells you this engine is healthy?

---

## Part 10 — The live-extension drill

| # | "What if…" | Your answer | What changes |
| --- | --- | --- | --- |
| 1 | …growth is 20%/month for two quarters? | | |
| 2 | …a supplier slips 10 weeks on 600 machines? | | |
| 3 | …you must patch a kernel CVE across 3,000 machines this week? | | |
| 4 | …a rack's worth of machines all fail the same way? | | |
| 5 | …your fleet database says a machine is serving and it's been unplugged for a month? | | |
| 6 | …finance asks you to cut capital spend 30% next quarter? | | |
| 7 | …you need to evacuate an entire site in 48 hours? | | |
| 8 | …a customer wants dedicated machines with a contractual SLA? | | |
| 9 | …power in one hall is capped and you can't add racks? | | |
| 10 | …you need to run in a region where you'd have to rent cloud instead? | | |

Then: **name the three questions you most hope they don't ask**, and answer
them anyway.
