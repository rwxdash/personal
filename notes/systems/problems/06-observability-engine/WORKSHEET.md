# Worksheet — 06 The Observability Engine

Parts 1 and 10 are the ones that matter most for this format. Part 1 because
the prompt gave you nothing; Part 10 because the interview is a live
extension, not a presentation.

---

## Part 1 — Scoping the unstated

### 1.1 What you decided the question was

The prompt is two sentences. Write the version of the problem you chose to
solve, in three sentences, as you would say it in the first two minutes of
the call.

### 1.2 Decisions you had to make unprompted

| # | Question the prompt didn't answer | Your answer | Why |
| --- | --- | --- | --- |
| 1 | Who is the consumer of this system? | | |
| 2 | Which signals are in scope? | | |
| 3 | Product feature or internal cost centre? | | |
| 4 | Where does "the engine" start and stop? | | |
| 5 | What scale? | | |
| 6 | What's different about stateful workloads? | | |
| 7 | | | |
| 8 | | | |

### 1.3 Explicitly out of scope

At least four. For each, one line on why it's a defensible exclusion — not
"no time," but "this is a different system and here's the boundary."

### 1.4 Requirements you derived

| Property | Target | How you derived it |
| --- | --- | --- |
| Live tail latency | | |
| Search latency, recent | | |
| Search latency, 30 days | | |
| Ingest availability | | |
| Acceptable log loss | | |
| Retention | | |
| Cost ceiling | | |

**Acceptable log loss** is the interesting one. State a number and defend
it. Zero is a possible answer and it is expensive.

---

## Part 2 — Back-of-envelope

### 2.1 Volume

| Quantity | Calculation | Result |
| --- | --- | --- |
| Log lines/s and bytes/s | | |
| Raw TB/day | | |
| Compressed TB/day, and your compression assumption | | |
| Storage for your retention | | |
| Metrics active series | | |
| Per-machine log rate (fleet ÷ volume) | | |

### 2.2 The indexing decision

This is where the money is. Cost both.

| Approach | Ingest cost | Storage cost | Query capability | $/month |
| --- | --- | --- | --- | --- |
| Full-text index everything | | | | |
| Index labels/metadata only, scan content | | | | |

Then state which you chose and what the customer loses.

### 2.3 The noisy tenant

| Quantity | Calculation | Result |
| --- | --- | --- |
| One tenant at 500 MB/s, as a share of platform volume | | |
| What that does to a shared pipeline | | |
| Where you stop it, and what the customer sees | | |

### 2.4 Cost per customer

| Quantity | Calculation | Result |
| --- | --- | --- |
| Total observability cost/month | | |
| Cost per paying customer | | |
| As a share of that customer's revenue | | |
| Cost of a free-tier customer's logs | | |

If observability costs more per customer than the customer pays, say so
plainly and say what you'd do about it.

---

## Part 3 — Data model & API

- What is a log line, as a record? What identifies it, and what is it
  labelled with?
- What is the query interface? What can a customer ask, and what can they
  not?
- How are logs correlated with a deploy, a container, a service, a project?
- Metrics: what's the model, and does it share anything with logs?
- What does the live-tail API look like, and how does it differ from search?

---

## Part 4 — Architecture

Diagram plus one sentence per component. Show explicitly:

- The path from a line written to stdout in a container to a customer's
  browser in live tail, with a latency budget per hop.
- The path from that same line to durable, searchable storage.
- Where those two paths diverge, and why.
- Where a tenant's quota is enforced.
- What runs on the compute machine itself, and what its resource budget is.

That last point matters: whatever you run on every machine competes with
paying customer workloads for CPU and memory.

---

## Part 5 — Deep dives

**Pick two.**

**Candidate A — Live tail and search are different systems.**
Sub-second streaming from an arbitrary machine to a browser, versus indexed
search over 30 days. Design both and the boundary between them. How does a
tail session find the machine? What happens when the container is
rescheduled mid-session? What happens when 5,000 sessions want the same
noisy service?

**Candidate B — The dying container.**
The most valuable log lines are the last ones before a crash, and they are
the ones most likely to be sitting in a buffer when the process dies.
Design for it. What is buffered where, what is the flush guarantee, what
happens on OOM-kill versus a clean exit versus a node failure? What is your
actual loss rate and how do you know?

**Candidate C — Multi-tenant isolation and quota.**
One customer at 500 MB/s. Where do you stop them, what do they experience,
and what do their neighbours experience? Design the quota mechanism —
per-service, per-project, per-plan — including what happens when the limit
is hit mid-incident, which is exactly when a customer most needs their logs.

**Candidate D — Storage and query at 130 TB/day.**
The index-everything-vs-index-labels decision, made concretely. Tiering, the
query path, what a 30-day search actually costs and how long it takes. What
you store, in what format, on what hardware, and what you throw away.

---

## Part 6 — Failure modes

| Failure | Blast radius | Detection | Mitigation | Degraded behaviour |
| --- | --- | --- | --- | --- |
| The log pipeline is down | | | | |
| One machine's agent falls behind | | | | |
| A tenant 100x's their volume in 30 seconds | | | | |
| Query load from one customer saturates the read path | | | | |
| Storage tier is unavailable | | | | |
| A site loses connectivity to the others | | | | |
| Your own deploy of the agent | | | | |

Then: **when the observability system is degraded, does the customer's
workload keep running?** Answer explicitly. Then answer the harder one: does
the *platform team* still have observability of the platform when the
customer-facing observability is down?

---

## Part 7 — Tradeoffs ledger

| Decision | Chosen | Rejected | Cost accepted | Who feels it |
| --- | --- | --- | --- | --- |

At least 10 rows.

---

## Part 8 — Evolution

- What breaks first at 10x — component, limit, number.
- What breaks second.
- What changes if customers demand traces?
- What changes if a customer needs their logs to stay in one jurisdiction?
- What changes if you need to expose this to a customer's own external
  tooling (Datadog, an S3 export, an OTLP endpoint)?

---

## Part 9 — Operations & cost

- Dominant cost driver, with the number, and the most effective lever.
- Top 3 alerts on this system.
- **The 3am page:** what actually justifies waking someone here?
- A customer says "my logs are missing." What do you need to answer them?
- What is the one operation you'd most want during an incident?

---

## Part 10 — The live-extension drill

**This is the part that prepares you for the 45 minutes.** For each prompt
below, write two or three sentences — the answer you would give out loud —
and note which component of your design changes.

Do not skip the ones you find uncomfortable. Those are the ones you'll get.

| # | "What if…" | Your answer | What changes in the design |
| --- | --- | --- | --- |
| 1 | …one customer is a third of your total log volume? | | |
| 2 | …we need sub-second tail but the container is on a machine in another site? | | |
| 3 | …a customer wants to search 30 days of logs by a value inside the message body? | | |
| 4 | …the cost of this is currently higher than the compute revenue it supports? | | |
| 5 | …we want to offer this as a paid tier with an SLA? | | |
| 6 | …a customer's database container is the thing being observed — what's different? | | |
| 7 | …you have to run it on bare metal you own, with no managed services at all? | | |
| 8 | …we need to keep 12 months instead of 30 days? | | |
| 9 | …the agent on the machine is using 15% of a core that we could be selling? | | |
| 10 | …we lose a whole site? | | |

Then: **name the three questions you most hope they don't ask**, and write
the answer to each anyway. That converts your worst moment into a prepared
one.
