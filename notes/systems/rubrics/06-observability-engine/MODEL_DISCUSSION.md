# Model Discussion — 06 The Observability Engine

> **Spoiler.** Do not open until the final review round is written.

One credible design, why its main alternative loses, and where competent
engineers would still disagree.

---

## 1. Scoping, out loud, in the first two minutes

> "I'm designing the customer-facing observability engine for a PaaS running
> ~500k services on ~3,000 machines we own. In scope: log collection,
> transport, storage and query, plus container-level metrics. Out of scope:
> alerting, dashboards, tracing, and billing — I'll say where the seams are.
> I've assumed 10M log lines/second peak, about 130 TB/day raw, 30-day
> retention, and that a single customer can produce a third of that with one
> bad deploy. Those assumptions drive everything, so tell me if any are
> wrong and I'll rework it."

That paragraph is worth more than the next twenty minutes of diagram,
because it makes everything after it assessable and it invites the
interviewer to steer.

---

## 2. Three numbers

| | | Consequence |
| --- | --- | --- |
| **130 TB/day raw, ~13 TB/day compressed** | 390 TB for 30 days | Storage bytes are *cheap* — a few thousand dollars a month. Not the problem. |
| **Full-text indexing 1.5 GB/s** | ~1 PB indexed + 60–100 nodes | **This is the problem.** Indexing compute and hot-tier hardware, not storage |
| **One tenant can be 33% of total volume** | 500 MB/s of 1.5 GB/s | Isolation must be in the ingest path, not a policy |

Against ~$1M/month revenue, a full-text cluster at this volume plausibly
consumes a double-digit percentage of *all* infrastructure spend to serve a
feature that is bundled, not sold. That is the finding that shapes the
design.

---

## 3. The design

```
  container stdout
        │  (runtime writes to disk on the machine — this matters, §3.3)
        ▼
 ┌────────────────────────────────────────────────────────┐
 │ PER-MACHINE AGENT   (budget: <0.5 core, <512 MB)        │
 │  · tails runtime log files, not a live pipe             │
 │  · attaches identity: project/service/deploy/container  │
 │  · PER-SERVICE RATE LIMIT ENFORCED HERE  ◄── the 33%    │
 │    excess dropped + counted + a visible notice injected │
 │  · local disk buffer (survives pipeline outage)         │
 │  · two outputs, one read                                │
 └───────┬────────────────────────────────┬───────────────┘
         │ TAIL (best effort, no durability)│ DURABLE
         ▼                                  ▼
 ┌───────────────────┐            ┌─────────────────────────┐
 │ TAIL FANOUT       │            │ INGEST LOG (Kafka-ish)   │
 │ · pub/sub by      │            │ · partitioned by project │
 │   service id      │            │ · 24h retention = replay │
 │ · subscribers     │            │ · absorbs bursts         │
 │   only; no store  │            └───────────┬─────────────┘
 │ · sub-second      │                        │
 └────────┬──────────┘            ┌───────────▼─────────────┐
          │                       │ COMPACTOR                │
          │                       │ · label index only       │
          │                       │ · content → compressed   │
          │                       │   chunks, time-ordered   │
          │                       └───────────┬─────────────┘
          │                        ┌──────────▼────────────┐
          │                        │ OBJECT STORE (own S3-  │
          │                        │ compatible or cloud)   │
          │                        │ hot 24h │ warm │ cold  │
          │                        └──────────┬────────────┘
          ▼                                   ▼
 ┌──────────────────────────────────────────────────────────┐
 │ QUERY API   · tail: subscribe   · search: label select    │
 │             then parallel scan of selected chunks         │
 └──────────────────────────────────────────────────────────┘
```

### 3.1 Tail and search are different systems

They share exactly one thing: the agent reads the log once. After that they
diverge completely, because their requirements are opposite:

| | Tail | Search |
| --- | --- | --- |
| Latency | sub-second | seconds |
| Durability | **none** — it's a pipe | full |
| Ordering | best effort | strict per stream |
| Cost model | per concurrent session | per byte stored |
| Failure | session drops, user refreshes | unacceptable |

Merging them means paying durability cost on the tail path and latency cost
on the search path. Keeping them separate makes tail almost free — it is a
pub/sub fanout with no storage — and lets the search path be optimised
purely for cost.

The tail path being *allowed to lose data* is the unlock, and it should be
stated as a deliberate product decision: if your browser drops a line while
tailing, you refresh and read it from search. Nobody has ever complained.

### 3.2 Index labels, not content

At 130 TB/day the fork is decided by arithmetic. Index only the metadata —
project, service, deploy, container, machine, severity, timestamp — and
store the content as compressed, time-ordered chunks. A query is: select
chunks by label and time range, then scan them in parallel.

**What the customer loses**, and this must be said out loud: searching for a
string across *all* their services over 30 days is a brute-force scan and
will take seconds to minutes rather than milliseconds. Searching within one
service over an hour — which is what people overwhelmingly actually do — is
fast, because the labels narrow it to a handful of chunks first.

I'd take that trade, and I'd offer a **small full-text index over the last
24 hours only** for the "I have no idea which service" case. Recent data is
a tiny fraction of the corpus, so the expensive index applies to ~4% of the
bytes. That hybrid captures most of the product value at a fraction of the
cost, and it's the answer I'd lead with.

### 3.3 The dying container

The single most valuable log line in the system is the last one before an
OOM-kill, and it's the one most at risk. The design:

- **Read from the container runtime's on-disk log files, not from a live
  stream.** When the container is `SIGKILL`ed, the file is still on the
  machine. The agent finishes reading it after the container is gone. This
  one decision recovers most of what a naive streaming design loses.
- **Flush on container-exit event** — the agent subscribes to runtime
  lifecycle events and prioritises reading a dying container's remaining
  bytes ahead of steady-state work.
- **Local disk buffer on the agent**, so a pipeline outage doesn't lose the
  lines the agent already has.
- Be honest about the boundary: anything still sitting in the *application's*
  own stdio buffer when it's `SIGKILL`ed was never written and the platform
  cannot recover it. That's a documentation and SDK problem — tell customers
  to write unbuffered — not an infrastructure one. Knowing where your
  guarantee stops is part of the design.
- The residual failure is the **machine** dying, taking the buffer with it.
  State the loss window (seconds) rather than claiming zero.

### 3.4 The noisy tenant, stopped at the edge

Enforcement lives **on the machine**, in the agent, for a simple reason: any
enforcement downstream means you have already paid to transport 500 MB/s
across your network before deciding to drop it.

- Per-service token bucket with a burst allowance.
- Excess is dropped, **counted**, and a synthetic line is injected into the
  customer's own stream: *"rate limit exceeded, N lines dropped."* Silent
  loss is the thing that destroys trust in a logging product.
- Per-project quota tied to plan, with the limit visible in the dashboard
  before it's hit.
- Partition the ingest log by project so one tenant's burst occupies its own
  partitions and can't delay another tenant's compaction.

**The case worth raising unprompted:** a customer hits their log quota
*during their own incident*, which is exactly when they need logs most.
Dropping them is correct engineering and terrible product. I'd give every
project a **burst credit** that accumulates while idle and drains under
load, so a first incident in a quiet month is fully logged and only
sustained abuse is throttled. That's a small mechanism that buys a lot of
goodwill, and noticing the conflict at all is the point.

### 3.5 The agent's budget is sellable capacity

The agent runs on all 3,000 machines. Every core it consumes is a core not
sold. A budget of half a core and 512 MB per machine is 1,500 cores and
1.5 TB of RAM across the fleet — a real line item, and the reason the agent
must be a small, boring, carefully-profiled program rather than a
general-purpose collector with a plugin system.

This is also why compaction and indexing happen **off** the compute
machines, in a separate tier, even though doing it locally would save
network. Customer workloads get the machine; observability gets a budget.

### 3.6 Stateful workloads

The prompt says "stateless *and* stateful," and the difference is real:

- A customer's database is **pinned to a machine**, so its logs have
  locality that a stateless service's don't — tail can find it reliably,
  and its log stream is long-lived rather than churning per deploy.
- The **cost of losing its last lines is higher** — a database that died
  needs its crash log for recovery, not just for curiosity.
- Its logs are **structured and voluminous in a different way** (slow query
  logs, WAL activity), so the label set differs.
- Restart semantics differ: a stateless container that dies is replaced; a
  stateful one restarts *in place*, so its log stream should be continuous
  across restarts rather than presenting as a new container each time.

That last one is a small product detail with a data-model consequence:
the stable identity for a stateful workload is the volume, not the
container.

---

## 4. Why the main alternative loses

**The alternative: one pipeline into Elasticsearch (or OpenSearch), serving
both tail and search, full-text indexed, hot–warm–cold via ILM.**

It's the obvious design, it's what most people build first, and it's
defensible at a tenth of this volume. It loses here on:

1. **Cost, decisively.** Full-text indexing 1.5 GB/s sustained needs a large
   cluster — order 60–100 nodes with replication and query headroom — plus
   an index footprint approaching the raw data size. Against ~$1M/month of
   revenue for a feature that is bundled rather than sold, it consumes a
   share of infrastructure spend that the business cannot justify. This ends
   it.
2. **Tail latency and durability are coupled.** Serving live tail from the
   same store means the tail's freshness is bounded by refresh interval and
   indexing lag, and you're paying durable-write cost for data a user will
   look at for eight seconds.
3. **The noisy tenant hits the shared cluster.** 500 MB/s into a shared
   index affects every other tenant's query latency and can push the cluster
   into backpressure. Isolation would require per-tenant indices or
   clusters, which multiplies the cost problem.

**Where it genuinely wins**, and it should be conceded: the product is
better. Arbitrary full-text search across everything, aggregations,
structured field queries, and a mature ecosystem. If observability were the
*product* rather than a bundled feature — if customers paid for it
specifically — the revenue would justify the cost and this would be the
right answer. The label-only design is a consequence of the business model,
not of engineering taste, and saying that shows you understand why the trade
exists.

---

## 5. Where reasonable engineers would disagree

**5.1 The 24-hour full-text index.** I've proposed a hybrid. Someone could
reasonably say it's two systems to operate for a query pattern that's rare,
and that a pure label-only design with good documentation is simpler and
adequate. They might be right; it depends on how often customers actually
search blind, which nobody has measured.

**5.2 Dropping vs backpressuring the noisy tenant.** I drop and notify.
The alternative is backpressure — slow the container's writes, or block on
stdout. That preserves every line and *changes the customer's application
behaviour*, which is a serious thing for a platform to do (a service that
blocks on a full log buffer is a service you've broken). I think dropping is
right and it should be loud, but there's a real argument that a database's
logs should never be dropped and should backpressure instead.

**5.3 Whether to build any of this.** Managed vendors exist and at this
volume they would be extremely expensive — which is presumably why a
platform company builds. But the honest position is that the build cost is a
team, forever, and a Staff engineer should price that against the vendor bill
rather than assuming self-hosting wins. For a company whose entire business
is running infrastructure, building is probably right; the reasoning should
still be explicit.

**5.4 Retention as a business lever.** I'd cut free-tier retention hard —
free customers generate logs and no revenue, and retention is the cheapest
dial available. Someone in growth would argue that logs are what makes the
free tier useful and converts people. That's a product argument I'd lose,
and the design should make retention per-plan configurable so it can be lost
gracefully.

**5.5 Reading from runtime log files vs a streaming socket.** Reading files
is more robust for the dying-container case but couples you to the runtime's
log-rotation behaviour and adds disk I/O on the compute machine. A streaming
approach is cleaner and loses more on crash. I weight the crash case heavily
because it's the case customers care most about; a different weighting is
defensible.

**5.6 What I'd expect to be pushed hardest on.** Two things. First,
*"what does a customer do when they can't find what they need with only
label search?"* — my answer is the 24-hour index plus better labels, and
it's the weakest part of the design. Second, *"you've built a lot; what
would you cut to ship in a quarter?"* — I'd cut the hybrid index, cut cold
tiering, ship tail plus 7-day label-indexed search, and say so, because a
platform that ships tail and recent search is already useful and everything
else is an increment.
