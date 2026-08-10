# Worksheet — 04 The Build Execution Platform

Part 1 comes **before** any architecture.

Prose, tables, ASCII/mermaid diagrams. No code.

---

## Part 1 — Requirements & scope

### 1.1 Ambiguities in the brief

| # | Question | Why it changes the design | Your assumption |
| --- | --- | --- | --- |
| 1 | | | |

> At least 8.

### 1.2 The conflict

Three of the stated requirements cannot all be satisfied at their stated
values. Name them, show why they conflict — with numbers, not adjectives —
and state which one you are proposing to relax, by how much, and who has to
agree.

| Requirement | Stated value | What it costs the other two |
| --- | --- | --- |
| | | |

**Which gives, and what do you ask for instead?**

### 1.3 Trust model

Enumerate the classes of workload by trust level. For each: what it may
access, what isolation it requires, and what an escape would cost you.

| Workload class | Runs code from | May access | Isolation required | Blast radius of escape |
| --- | --- | --- | --- | --- |
| | | | | |

### 1.4 Functional requirements

### 1.5 Non-functional requirements

| Property | Target | Source (stated / derived / assumed) |
| --- | --- | --- |
| Queue time p95 | | |
| Queue time p99 | | |
| Job success rate (excluding user error) | | |
| Isolation guarantee, per class | | |
| Cache hit rate | | |
| Platform availability | | |
| Cost ceiling | | |

### 1.6 Explicitly out of scope

At least four, each with what you assume someone else guarantees.

---

## Part 2 — Back-of-envelope

Show the arithmetic. This problem is decided by a cost model, so build one
that you could defend to a director.

### 2.1 Capacity

| Quantity | Calculation | Result |
| --- | --- | --- |
| Mean job duration (derive from p50/p99 shape; state your assumption) | | |
| Average concurrent jobs | | |
| Peak concurrent jobs | | |
| Peak vCPU / RAM | | |
| Average vCPU / RAM | | |
| **Average as a fraction of peak** | | |

### 2.2 The cost squeeze

| Scenario | Calculation | $/year |
| --- | --- | --- |
| Provisioned for peak, on-demand, 24/7 | | |
| Perfect elasticity, on-demand | | |
| Perfect elasticity, 70% spot | | |
| Target | **$2.5M** | |

State plainly what the perfect-elasticity number implies about your
freedom to be inefficient.

### 2.3 Queue time vs provisioning

| Quantity | Calculation | Result |
| --- | --- | --- |
| Time to provision a new node (state your reference figure) | | |
| Time to start a container on a warm node | | |
| Time to start a microVM on a warm node | | |
| Queue time SLO | | |
| **Warm capacity needed to absorb the spike ramp** | | |
| Cost of that warm capacity | | |

### 2.4 Spot interruption

| Quantity | Calculation | Result |
| --- | --- | --- |
| Fraction of jobs longer than a typical spot notice window | | |
| Expected interruptions/day at your spot fraction (state your assumed rate) | | |
| Jobs lost/day, and their wasted compute | | |
| Cost of a retried job vs the spot saving | | |

### 2.5 Cache

| Quantity | Calculation | Result |
| --- | --- | --- |
| Compute saved by a 60% time reduction, in $/year | | |
| Cache storage and egress volume | | |
| Cost of the cache tier | | |
| Net value of the cache | | |

Then state what a cache miss costs and what a *poisoned* cache costs.

---

## Part 3 — API & data model

### 3.1 Core operations

Submit, query, cancel, and whatever the runner agent and cache protocol
need.

### 3.2 Entities

Job, run, execution environment, cache namespace, tenant, quota. What
identifies each?

### 3.3 Scheduling inputs

What does the scheduler need to know about a job to place it, and where does
that information come from? What happens when it's wrong (a job declares
4 vCPU and uses 16)?

---

## Part 4 — High-level architecture

Diagram plus one sentence per component. Show explicitly:

- The path from a git event to a running job, with the 30-second budget
  annotated.
- Where the isolation boundary is, per workload class.
- Where the cache lives and who may read and write which parts of it.
- Where capacity comes from, and how it changes through the day.

---

## Part 5 — Deep dives

**Pick two.** Say which and why.

**Candidate A — Isolation for untrusted code.**
Fork PRs execute arbitrary code beside your payments builds. Design the
boundary. Containers, microVMs, separate node pools, separate accounts,
separate clusters — what, where, and what does each cost in start latency,
memory floor, and utilisation? What does a fork job get for network egress,
for secrets, and for the cache? What happens if someone escapes anyway?

**Candidate B — Scheduling under a 3x spike with a 30-second SLO.**
Node provisioning is slower than your queue SLO. Design the mechanism:
warm pools, predictive scaling, overcommit, preemption of low-priority work,
admission control, or something else. Quantify the warm capacity required
and its cost. Handle the long tail — a 4-hour job placed on capacity you
want to reclaim. Include fairness: what stops one team's 2,000-job matrix
build from starving everyone else's PR checks?

**Candidate C — The cache.**
60% of wall-clock time is at stake, which makes the cache the single largest
performance and cost lever, and a shared mutable store that untrusted code
writes to is an obvious attack. Design it. What is the key, what is the
scope, who may write, and how does a fork PR benefit without being able to
poison? What is the eviction policy and what does it cost when wrong? What
happens on a cache-wide invalidation?

**Candidate D — Migration off 40 Jenkins fleets.**
A dozen teams have genuine blockers — a plugin, a licensed toolchain, a
physical device. Design the migration: order, mechanism, what runs in
parallel, how you prove equivalence before cutting a team over, and what the
rollback is. What do you do about the physical device in Berlin? What is
your policy for the teams whose blocker is real versus the teams whose
blocker is preference?

---

## Part 6 — Failure modes

| Failure | Blast radius | How you detect it | Mitigation | Degraded behaviour |
| --- | --- | --- | --- | --- |
| Spot reclamation takes 20% of the fleet in 2 minutes | | | | |
| A job escapes its sandbox | | | | |
| The cache tier is unavailable | | | | |
| A poisoned cache entry spreads to many builds | | | | |
| One team submits 50,000 jobs by accident | | | | |
| A job runs for 12 hours and won't die | | | | |
| The control plane is down but jobs are running | | | | |
| The container registry is unavailable | | | | |
| Your own deploy during the Berlin morning spike | | | | |

Then answer: **when you are at capacity and out of budget, what do you do?**
Queue, degrade, reject, or overspend. Justify.

---

## Part 7 — Tradeoffs ledger

| Decision | Chosen | Rejected | Cost accepted | Who feels it |
| --- | --- | --- | --- | --- |
| | | | | |

At least 10 rows. At least one where the cost is paid by developers and one
where it's paid by security.

---

## Part 8 — Evolution

- What breaks first at 10x — component, limit, number.
- What breaks second.
- What changes if the company acquires a team with a completely different
  toolchain?
- What changes if a regulator requires that builds of the payments service
  run on infrastructure that never executes third-party code?
- Your design will be asked to run things that aren't CI — batch jobs, ML
  training, one-off data tasks. Is that a feature or a threat, and what would
  you do about it?

---

## Part 9 — Operations & cost

- Dominant cost driver, with the number, and the most effective lever.
- Top 3 alerts. Why each is a page.
- **The 3am page:** which of these failures actually justifies waking
  someone, given CI is not customer-facing? Be honest, and say what the
  on-call is for if the answer is "few of them."
- A developer says "my build is slow and it's the platform's fault." What do
  you need to answer them, and does your design produce it?
- How do you charge teams for what they use, and what behaviour does your
  chargeback model create?
- What is the one operation you'd most want during an incident, and does
  your design allow it?
