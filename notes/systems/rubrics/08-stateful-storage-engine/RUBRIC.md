# Rubric — 08 The Stateful Storage Engine

> **Spoiler.** Do not open until the final review round is written.

Weight: **scoping 25% · design 40% · live extension 25% · questions 10%.**

---

## The central tension

**Durability, performance, cost, and placement freedom are four corners and
you get two.**

- **Local NVMe** gives a customer database the latency it needs (tens of
  microseconds), costs nothing extra, and **pins the workload to one machine
  that will eventually die**, taking the data with it.
- **Network-attached, replicated** gives durability and mobility at a
  latency multiple of roughly 10–50x on the write path, plus 2–3x the
  hardware, plus a network that now carries all storage traffic.
- **Replication** is the only answer to hardware failure and it multiplies
  the cost of the largest cost centre in the design.
- **Placement freedom** is what problem 07's density depends on, and every
  pinned volume is a hole in the bin-packing.

**The arithmetic is the argument, and it should come before any
architecture.** 24,000 drives at ~1% AFR is a drive failure every ~1.5 days.
3,000 machines at ~3% is a machine failure every ~4 days. With 67 volumes
per machine and no replication, that is **~6,000 customer volumes lost per
year**. Not degraded — lost. A candidate who computes that number has
finished the argument; a candidate who doesn't is designing on vibes.

**The insight the problem is built around: the customer's database already
knows how to replicate itself, and if you replicate underneath it you are
paying for durability twice.** A Postgres with a streaming replica on
another machine survives a machine failure without any storage-level
replication at all — and it does so with *local* disk performance. That
observation reframes the whole design: the storage engine's job may be to
provide *fast local disk plus a fast rebuild path*, and to let the layer
above handle redundancy where it can.

That is not a free win either, and the strongest answers say why: most
customers deploy a single Postgres and never configure a replica, the
platform cannot assume application-level redundancy exists, and "your data
is safe if you configured it correctly" is not a product. So the real answer
is usually **tiered**: local-only for the hobby tier with backups as the
recovery mechanism, replicated for production, and honesty about which is
which.

**Secondary tension:** 67 volumes sharing 8 drives means one tenant's batch
job destroys another tenant's database p99. And during a rebuild, *your own*
repair traffic is the noisy neighbour.

---

## Scoping — what a strong candidate establishes unprompted

**25% of the grade.**

| # | Decision | Why it matters |
| --- | --- | --- |
| 1 | **What a volume is, as a product.** Block device, filesystem, or managed database? | Four different systems. A managed Postgres offering makes the storage problem *easier* (you control the replication) and the product problem harder. |
| 2 | **Local vs network-attached.** | The decision everything turns on. Both are used in production; the candidate must pick and defend with numbers. |
| 3 | **Who replicates.** | The "paying twice" observation. Missing it is the single biggest gap available in this problem. |
| 4 | **Durability per tier.** | Uniform durability across a free tier and a production tier is a choice, usually the wrong one, and almost nobody considers tiering it. |
| 5 | **Backup scope and restore semantics.** | Customers think backups are in scope. If they're not, say so; if they are, a crash-consistent snapshot of a running database is not what a customer thinks it is. |
| 6 | **The interface to the scheduler.** | Volume placement is a hard constraint on compute placement. The two systems have a contract and it should be named. |
| 7 | **Scale.** | Pick, state, derive. |
| 8 | **What "lost data" means operationally.** | Is there a scenario where the answer to a customer is "it's gone"? Most designs have one and most candidates won't say it. |

6+ with reasons is a Staff signal. Fewer than 4 caps at Senior.

---

## Back-of-envelope

| Quantity | Defensible value | Working |
| --- | --- | --- |
| Total drives | **24,000** | 3,000 × 8 |
| Drive failures | **~240/year ≈ one every 1.5 days** | 1% AFR |
| Machine failures | **~90/year ≈ one every 4 days** | 3% AFR |
| Volumes per machine | **~67** | 200,000 ÷ 3,000 |
| **Volumes lost/year, unreplicated** | **~6,000** | 90 machine failures × 67. **The number that ends the argument** |
| 3x replication | **2 PB → 6 PB raw** | And roughly 3x the storage cost |
| Rebuild of one machine's data | **~4+ hours** | ~15 TB at ~1 GB/s effective; slower if throttled to protect customers |
| Reduced-redundancy window | hours per failure, several times a week | Which is why a second-failure-during-rebuild analysis is required, not optional |
| Local NVMe latency | **~20–100 µs** | Order of magnitude |
| Network-attached latency | **~0.5–2 ms** | Order of magnitude — **roughly 10–50x local** |
| Effect on Postgres commit | one fsync per commit → the multiple lands directly on commit latency | Group commit amortises it, which is the honest mitigation |
| IOPS fairly shared | 8 drives ÷ 67 volumes | Meaningless as an average — the distribution is what matters, most are idle |

**Staff:** computes the 6,000-volumes-lost number and leads with it.
Quantifies the local-vs-network latency multiple and connects it to a
database commit rather than leaving it abstract. Computes rebuild time and
notices the reduced-redundancy window recurs several times a week.

**Red flags:** No failure arithmetic. Treating 2 PB as the hard part.
Quoting "three nines of durability" with no derivation — durability numbers
that aren't computed from failure rates and replication factor are
decoration.

---

## Design

### The local/network decision

Must be made with numbers and the losing side's advantages acknowledged.
Strong answers frequently land on a **hybrid**, and the hybrid must be
principled rather than a dodge:

- **Local NVMe for the data path** — the performance a database needs.
- **Replication at the storage layer** (synchronous to one peer, async to a
  second) for tiers that pay for it, giving durability without giving up
  local reads.
- **Backups to object storage** as the universal floor, including for tiers
  that get no replication.

The candidate should notice that synchronous replication to a peer costs
roughly the network latency on every write — which is the same cost as
network-attached storage, unless reads stay local (they do) and unless the
replication is asynchronous (which trades RPO for latency, and that trade
must be stated).

### Failure and repair

Must cover, with the *slow* machine called out specifically:

- **Drive failure**: detected by SMART plus I/O errors; repair by rebuild
  from replicas; the volume stays available if replicated.
- **Machine failure**: 67 volumes affected. Stateless neighbours reschedule
  in seconds; these need volumes reattached or rebuilt, and the customer is
  *down* until then. Recovery time is the product-visible number.
- **A slow machine is worse than a dead one** (interviews 07·A13). A drive
  with rising latency, a failing NIC, or a machine thrashing will serve
  reads slowly and hold write quorums open, degrading every volume on it
  while passing every health check. Detection must be *comparative* — this
  machine versus its peers — not threshold-based.
- **Silent corruption**: checksums on every block, verified on read and by a
  background scrubber. Without scrubbing, a corrupt replica is discovered at
  the worst possible moment: during a rebuild, when it's the only copy left.
- **Second failure during rebuild**: with a several-hour window recurring
  several times a week, this is not a tail event and must be quantified.
- **Rack/power domain**: replicas must not share a rack, a PDU, or a switch.
  Placement is a failure-domain problem, not a capacity problem.

### Backups and the thing nobody designs

A snapshot of a running database is **crash-consistent, not
application-consistent** — it's equivalent to pulling the power. Postgres
will recover from it via WAL replay, usually; "usually" is not a backup
product. Strong answers distinguish:
- Block snapshot (fast, crash-consistent, fine as a floor)
- Application-aware backup (`pg_basebackup` + WAL archiving, giving real
  PITR)
- And state which is offered for which tier.

**The highest-signal element in this deep dive: how do you know a backup is
restorable before a customer needs it?** The answer is continuous automated
restore testing — restore a sample of backups, start the database, run a
query, compare. Almost nobody designs this and it is the difference between
a backup system and a backup-shaped hope.

### Multi-tenant IOPS

67 volumes on 8 drives. Needs cgroup v2 `io.max`/`io.latency` or equivalent,
per-volume, with a fair-share model that lets idle capacity be used but
guarantees a floor under contention (the same shape as CPU shares —
interviews 05·A8). And crucially: **rebuild traffic must be throttled below
customer traffic**, because your own repair is the most predictable noisy
neighbour you have, and an unthrottled rebuild turns one machine's failure
into every neighbour's latency incident.

---

## The live extension (25%)

Highest-value pushes:

1. *A machine dies right now with 67 volumes.* Tests whether recovery is
   designed or assumed.
2. *The customer's Postgres already replicates — are you paying twice?*
   Tests the central insight.
3. *Cut storage cost in half.* Tests whether they know which knob and what
   it costs (tiering, erasure coding, dropping replication for a tier,
   compression, thin provisioning).
4. *A rebuild is saturating the network.* Tests whether repair was designed
   as a workload that competes with customers.
5. *A replica has been silently corrupt for a month.* Tests scrubbing.
6. *The free tier can't be replicated economically.* Tests whether they'll
   defend tiered durability or retreat into promising everything.

---

## Grading calibration

| Grade | Profile |
| --- | --- |
| **Below Senior** | No failure arithmetic. "Use Ceph" or "use EBS" with no analysis. Durability asserted, not derived. No account of what a machine failure does to 67 customers. |
| **Senior** | Computes failure rates. Picks local or network-attached with a reason. 3x replication. Backups to object storage. Handles drive and machine failure. Misses the replicate-twice observation, treats all tiers identically, and doesn't design repair as a workload. |
| **Borderline Staff** | Failure arithmetic drives the design *or* the local/network tradeoff is quantified against a database commit, not both. Rebuild time computed but the second-failure window not analysed. Backups present, restore-testing absent. |
| **Staff** | Leads with the 6,000-volumes-lost number. Local/network decided with the latency multiple applied to a real commit path. Replication placement respects failure domains. Slow-machine detection is comparative. Rebuild throttled below customer traffic. Backups distinguish crash- from application-consistency. Per-tier durability with an honest promise. Scheduler interface named. 6+ scoping decisions. |
| **Strong Staff** | All of the above, plus: raises the paying-for-durability-twice observation and resolves it with tiering rather than dodging it; designs continuous restore verification; quantifies the second-failure-during-rebuild probability and treats it as a design input rather than a tail risk; states plainly the scenario in which the answer to a customer is "your data is gone" and what tier that applies to; and connects volume pinning back to the compute scheduler's density problem as a contract between two systems. |

---

## Review guidance

**Round 1:** Ask for the failure arithmetic before reading anything else. If
it isn't there, that is the headline finding. Then recompute drive and
machine failure intervals, volumes-lost-per-year, rebuild time, and the
local-vs-network latency multiple.

The single best probe: *"a machine just died with 67 volumes on it, twelve
of them production databases — walk me through the next four hours."* It
exposes replication, rebuild, customer communication, and whether they know
their own recovery time.

Second best: *"the customer is running Postgres with a replica they
configured themselves; what is your replication for?"*

**Round 2+:** Commonly dodged: the slow-machine case, restore verification,
and whether the free tier gets replication. Escalate those.

**Final:** The highest-leverage improvement is usually one of: "you never
computed how often hardware fails, and it decides everything," "you're
replicating underneath a database that replicates itself, and paying for
both," or "you have backups and no evidence any of them can be restored."
