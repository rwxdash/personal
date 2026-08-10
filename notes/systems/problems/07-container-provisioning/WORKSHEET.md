# Worksheet — 07 The Container Provisioning Engine

Parts 1 and 10 matter most for this format.

---

## Part 1 — Scoping the unstated

### 1.1 What you decided the question was

Three sentences, as you'd say them in the first two minutes.

### 1.2 Decisions you had to make unprompted

| # | Question the prompt didn't answer | Your answer | Why |
| --- | --- | --- | --- |
| 1 | Where does the engine begin and end? | | |
| 2 | Build on an orchestrator, or build one? | | |
| 3 | What is the container runtime, and why? | | |
| 4 | Is image build in scope? | | |
| 5 | What are the latency contracts? | | |
| 6 | What changes for stateful workloads? | | |
| 7 | What scale? | | |
| 8 | | | |

### 1.3 Explicitly out of scope

At least four, with the boundary stated for each.

### 1.4 Requirements you derived

| Property | Target | Derivation |
| --- | --- | --- |
| Cold deploy latency | | |
| Wake latency | | |
| Scheduling throughput (placements/s) | | |
| Density (containers per machine) | | |
| Isolation guarantee | | |
| Control-plane availability | | |
| Data-plane survival without the control plane | | |

---

## Part 2 — Back-of-envelope

### 2.1 The wake budget

Break down the 3-second wake, hop by hop, and say which term dominates.

| Stage | Time | Notes |
| --- | --- | --- |
| Request arrives at edge, service identified as asleep | | |
| Scheduling decision | | |
| Image available on the target machine? | | |
| Container create + start | | |
| Application boot | | |
| First byte | | |
| **Total** | | |

Then answer: **which of these terms do you actually control?**

### 2.2 Image distribution

| Quantity | Calculation | Result |
| --- | --- | --- |
| Container creations/day | | |
| If every creation pulled the image: bytes/day | | |
| Pull time for an average image at 1 / 10 / 25 Gbps | | |
| **Does a cold pull fit the wake budget?** | | |
| Cache hit rate needed to hold p95 | | |
| Storage per machine to hold that working set | | |

### 2.3 Density and overcommit

| Quantity | Calculation | Result |
| --- | --- | --- |
| Running containers ÷ machines | | |
| RAM per machine at declared request sizes | | |
| RAM per machine at observed usage | | |
| **Overcommit ratio required to be economical** | | |
| What happens when the overcommit is called | | |

### 2.4 Scheduling load

| Quantity | Calculation | Result |
| --- | --- | --- |
| Placements/second, average and peak | | |
| Fleet state size the scheduler must reason over | | |
| Time budget for one placement decision | | |
| Can a single scheduler do this? Show the working | | |

### 2.5 Cost

| Component | Driver | Share |
| --- | --- | --- |

What is the cost of one sleeping service per month? That number determines
whether a free tier is viable.

---

## Part 3 — Data model & API

- What is a *service*, a *deployment*, an *instance*? What identifies each?
- What is the desired-state representation, and where does it live?
- What does the machine agent receive, and what does it report?
- How does the engine express placement constraints (volume affinity,
  region, machine class, tenant separation)?

---

## Part 4 — Architecture

Diagram plus one sentence per component. Show explicitly:

- Control plane vs data plane, and what survives the control plane being
  down.
- The wake path, with the budget annotated per hop.
- The deploy path from image reference to serving traffic.
- Where the isolation boundary is.
- How the router knows where a service is running, and how that changes when
  it moves.

---

## Part 5 — Deep dives

**Pick two.**

**Candidate A — Cold start and the wake path.**
3 seconds, from a request arriving to bytes going back, for a service that
is not running. Design it: image caching and distribution, pre-warmed
sandboxes, snapshot/restore, lazy image loading, or something else.
Quantify each. Handle the case where the machine that has the image cached
is not the machine with capacity. Handle the thundering herd when 500
sleeping services wake at once.

**Candidate B — Scheduling and placement on a fixed fleet.**
No autoscaling group, heterogeneous machines, hostile tenants, and stateful
workloads that are pinned to where their data is. Design the scheduler:
what it optimises, what constraints it honours, how it handles fragmentation,
and how it rebalances without disrupting customers. Include the arithmetic
on overcommit and what happens when it's called.

**Candidate C — Isolation and abuse.**
Anonymous tenants running arbitrary code on your hardware. Choose the
runtime boundary and defend the cost. Then design for the operational
reality: cryptomining detection, egress abuse, resource exhaustion attacks
against neighbours, and what happens the day someone escapes. What is the
blast radius, and what do you rebuild?

**Candidate D — Deploys, rollout and the router.**
The engine must roll out a customer's new version with zero downtime, on
their behalf, without them thinking about it. Design it: health gating,
traffic shifting, rollback, and how the router's view of reality stays
correct while instances move. What happens when the new version is broken
and the old one has already been reclaimed?

---

## Part 6 — Failure modes

| Failure | Blast radius | Detection | Mitigation | Degraded behaviour |
| --- | --- | --- | --- | --- |
| A machine dies with 67 containers on it | | | | |
| The control plane is unavailable for 30 minutes | | | | |
| The image registry is unavailable | | | | |
| Overcommit is called — a machine's containers all want their RAM | | | | |
| A tenant escapes the sandbox | | | | |
| A tenant saturates a machine's network | | | | |
| 500 sleeping services wake simultaneously | | | | |
| A deploy of the machine agent itself | | | | |
| A site loses connectivity | | | | |

Then: **when the control plane is down, do running customer workloads keep
serving traffic?** Answer explicitly and say what it costs you to guarantee
it.

---

## Part 7 — Tradeoffs ledger

| Decision | Chosen | Rejected | Cost accepted | Who feels it |
| --- | --- | --- | --- | --- |

At least 10 rows. At least one where the cost is paid by density (money) and
one where it's paid by isolation (risk).

---

## Part 8 — Evolution

- What breaks first at 10x — component, limit, number.
- What breaks second.
- What changes if you must support GPUs?
- What changes if a large customer wants dedicated machines?
- What changes if you have to run in a region where you don't own hardware?
- What would you have to change to support a workload that cannot be moved
  or restarted at all?

---

## Part 9 — Operations & cost

- Dominant cost driver, with the number, and the most effective lever.
- Top 3 alerts.
- **The 3am page:** a machine holding 67 customer containers has gone
  unresponsive. Walk through it.
- A customer says "my deploy is stuck." What do you need to answer them?
- How do you drain a machine for maintenance, and how long does it take?
- What is the one operation you'd most want during an incident?

---

## Part 10 — The live-extension drill

Two or three sentences each — the answer you'd give out loud — plus what
changes in the design.

| # | "What if…" | Your answer | What changes |
| --- | --- | --- | --- |
| 1 | …the wake budget is 300 ms instead of 3 s? | | |
| 2 | …a customer's service has a 40 GB volume attached? | | |
| 3 | …you can't use Kubernetes? (or: why *are* you using it?) | | |
| 4 | …someone is mining crypto on 400 machines right now? | | |
| 5 | …a machine dies with 67 containers, 12 of them stateful? | | |
| 6 | …the control plane is down and a customer wants to deploy? | | |
| 7 | …you need 3x the density to make the unit economics work? | | |
| 8 | …a customer's image is 8 GB? | | |
| 9 | …we want preview environments per pull request, so creations go 10x? | | |
| 10 | …two tenants must never share a machine, contractually? | | |

Then: **name the three questions you most hope they don't ask**, and answer
them anyway.
