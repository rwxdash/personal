# Worksheet — 02 The Quota Enforcement Plane

Part 1 comes **before** any architecture. The rest can be written alongside
`DESIGN.md`, but everything here should be answered before you call a round
ready.

Prose, tables, ASCII/mermaid diagrams. No code.

---

## Part 1 — Requirements & scope

### 1.1 Ambiguities in the brief

| # | Question | Why it changes the design | Your assumption |
| --- | --- | --- | --- |
| 1 | | | |

> Aim for at least 8. The brief contains one ambiguity that, once resolved,
> reorganises the entire design. If your list doesn't contain something that
> feels structural rather than clarifying, read it again.

### 1.2 Classes of limit

The brief says the author suspects there are two kinds of limit and doesn't
know what it costs to separate them. Answer that.

| Class | Purpose | Accuracy requirement | Latency requirement | Failure behaviour | Who is harmed if it's wrong |
| --- | --- | --- | --- | --- | --- |
| | | | | | |

Then state, in one sentence, whether these are one system or two, and what
that decision costs.

### 1.3 Functional requirements

Include: what a limit is defined over, what the caller receives when
limited, what the billing consumer receives, and what an operator can change
at runtime.

### 1.4 Non-functional requirements

| Property | Target | Source (stated / derived / assumed) |
| --- | --- | --- |
| Added latency at enforcement | | |
| Enforcement availability | | |
| Counting accuracy — protective limits | | |
| Counting accuracy — billable quotas | | |
| Config propagation time | | |
| Durability of usage counters | | |
| Cost ceiling | | |

For **accuracy**, express it as a bound you could test: "no more than X%
overshoot over a window of Y." "Approximately correct" is not a requirement.

### 1.5 Explicitly out of scope

At least four, each with what you're assuming someone else guarantees.

---

## Part 2 — Back-of-envelope

Show the arithmetic.

### 2.1 The decision path

| Quantity | Calculation | Result |
| --- | --- | --- |
| Limit decisions/s at peak | | |
| Decisions/s per edge node | | |
| Latency budget per decision | | |
| Cross-region RTT (state your reference figure) | | |
| **How many round trips fit in the budget?** | | |

That last row is the one that decides the architecture. State the
conclusion explicitly.

### 2.2 The centralised-counter alternative

Cost it out honestly before you reject it.

| Quantity | Calculation | Result |
| --- | --- | --- |
| Ops/s against a central store | | |
| Nodes required (state your per-node op rate assumption) | | |
| Added latency, same-region | | |
| Added latency, cross-region | | |
| Verdict, and on which ground it fails first | | |

### 2.3 Local enforcement error

If each edge node enforces locally, quantify the overshoot.

| Quantity | Calculation | Result |
| --- | --- | --- |
| Nodes that may see traffic for one key | | |
| Worst-case overshoot for a limit of L | | |
| Same, for the top account (11% of traffic) | | |
| Same, for a key seen by only one node | | |
| Overshoot as a % for a typical limit | | |

Then say which limit classes can live with that number and which cannot.

### 2.4 State size

| Quantity | Calculation | Result |
| --- | --- | --- |
| Distinct (key, limit) tuples active per minute | | |
| Bytes of counter state per tuple | | |
| Total state per edge node | | |
| Total state globally | | |
| Monthly-quota state (longer window) | | |

### 2.5 Cost

| Component | Driver | Rough share |
| --- | --- | --- |
| | | |

State the **cost per million requests** your design implies. The plane
touches every request, so this number is multiplied by everything.

---

## Part 3 — API & data model

### 3.1 Core operations

The enforcement call, the config path, the usage-export path, and whatever
an operator uses in an incident.

### 3.2 Entities

Limit definition, limit key, counter, quota period, usage record. What
identifies each? What is the difference between a *limit* and a *limit
instance*?

### 3.3 Partitioning and key design

- What is the shard key for counter state?
- What happens to the account that is 11% of global traffic?
- Is a "monthly quota" the same kind of object as a "1000/minute" limit?
  Justify.
- Month boundaries: whose clock, whose timezone, and what happens to a
  request at the boundary?

---

## Part 4 — High-level architecture

Diagram plus one sentence per component. Show explicitly:

- The path of a request through enforcement, with the latency budget
  annotated at each hop.
- Where counter state lives for each limit class.
- How config reaches an edge node.
- How usage reaches billing.
- What the edge does when it cannot reach anything else.

---

## Part 5 — Deep dives

**Pick two.** Say which and why those are the hardest.

**Candidate A — Distributed counting.**
How do 2,800 nodes enforce a single global limit? Local buckets with
periodic reconciliation, budget allocation from a central authority, leased
token blocks, or something else. Quantify the error and the coordination
traffic. What happens when a key's traffic shifts between PoPs mid-window?
What happens to the allocation when a node dies holding an unspent budget?

**Candidate B — Billing-grade accounting.**
The number on the invoice has to be defensible to a customer's finance
team. What is the accounting path, how is it reconciled against enforcement,
and what is the tolerance? What happens when they disagree? Does a rejected
request count against quota? Does a request that fails downstream? What
makes a usage record durable enough to bill from, and what's the retention?

**Candidate C — Config safety.**
A bad config reached every PoP in under two seconds and caused a global
outage. Design the config plane so that cannot happen, without making
propagation so slow that a kill switch is useless. Include: validation,
staged rollout, automatic rollback criteria, and what an operator does when
they need a change *everywhere, now*.

**Candidate D — Failure and degradation.**
The enforcement plane must be more available than everything behind it.
What does the edge do when it cannot reach the control plane, the counter
tier, or its peers? Fail open or fail closed — per limit class. What does a
customer experience during each degraded mode, and what does the business
lose?

---

## Part 6 — Failure modes

| Failure | Blast radius | How you detect it | Mitigation | Degraded behaviour |
| --- | --- | --- | --- | --- |
| Counter tier unreachable from one PoP | | | | |
| Counter tier unreachable globally | | | | |
| Control plane down for 4 hours | | | | |
| A PoP is isolated but still serving traffic | | | | |
| An edge node's clock is 90 seconds off | | | | |
| A single key attracts 40x its normal traffic | | | | |
| A bad limit config is published | | | | |
| Usage export backs up for a day | | | | |
| Your own deploy | | | | |

Then answer explicitly, per limit class: **fail open or fail closed?**
Justify each. A single answer for both classes is probably wrong.

---

## Part 7 — Tradeoffs ledger

| Decision | Chosen | Rejected | Cost accepted | Who feels it |
| --- | --- | --- | --- | --- |
| | | | | |

At least 10 rows. At least one where the cost is genuinely painful.

---

## Part 8 — Evolution

- What breaks first at 10x — component, limit, and number.
- What breaks second.
- The migration: 40+ existing limiters, all with different semantics, all
  in front of live traffic. How do you cut over without changing what
  customers experience? What is the rollback at each step?
- The brief mentions 6M rps of internal traffic that never touches the edge.
  What does including it do to your design? What does excluding it cost?
- If the company adds a second product with its own plan structure, what
  in your design has to change?

---

## Part 9 — Operations & cost

- Dominant cost driver, with the number, and the single most effective
  lever.
- Top 3 alerts on this system. Why each is a page not a ticket.
- **The 3am page:** write the scenario. What fired, what the on-call sees,
  what they can do at 3am versus what waits for morning.
- A customer calls and says "you rate limited me and I was under my limit."
  What do you need to be able to answer them, and does your design produce
  it?
- What is the one operation you'd most want available during an incident,
  and does your design allow it?
