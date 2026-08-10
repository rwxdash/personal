# Model Discussion — 08 The Stateful Storage Engine

> **Spoiler.** Do not open until the final review round is written.

---

## 1. Open with the failure arithmetic

Before any architecture:

```
24,000 drives  × 1% AFR   = ~240 drive failures/year   = one every 1.5 days
 3,000 machines × 3% AFR  = ~90 machine failures/year  = one every 4 days
 67 volumes per machine
 ────────────────────────────────────────────────────────────────────
 With no replication: 90 × 67 ≈ 6,000 customer volumes lost per year
```

**Six thousand customer databases per year.** That number is the entire
argument for everything that follows, and stating it in the first three
minutes is worth more than any diagram. It also reframes the question from
"how do we store data" to "how do we survive a hardware failure rate we
cannot reduce."

The second number that matters:

```
Local NVMe write   ~20–100 µs
Network-attached   ~0.5–2 ms          →  roughly 10–50x
Postgres commit = one fsync           →  the multiple lands on every commit
```

So network-attached storage is not "slightly slower" for a database; it
changes commit latency by an order of magnitude, and group commit is the
only thing that hides it.

---

## 2. The reframe: the customer may already be replicating

A customer's Postgres with a streaming replica on another machine already
survives a machine failure. If we *also* replicate the block device
underneath it, we are paying for durability twice — 3x storage under a
system that is already 2x replicated is 6x the raw data for one logical
database.

That observation is the most valuable one available in this problem, and it
points at a design where the storage engine provides **fast local disk and a
fast recovery path**, and redundancy lives wherever it is cheapest for that
workload.

**But it does not survive contact with the customer base**, and the strong
version of the answer says why: most customers deploy one Postgres, never
configure a replica, and would be astonished to learn their data lives on
one disk. "Your data is safe if you configured it correctly" is not a
product.

So the resolution is **tiering**, honestly labelled:

| Tier | Storage | Survives a machine loss? | Recovery | Cost multiple |
| --- | --- | --- | --- | --- |
| **Hobby / free** | Local NVMe, single copy | **No** | Restore from last backup — minutes to hours, some data loss | 1x + backup |
| **Production** | Local NVMe + synchronous replica on another rack | Yes | Promote replica, seconds to minutes | ~2x |
| **Critical** | Above + async third copy in another site | Yes, including site loss | Cross-site failover | ~3x |

The hobby tier explicitly does not survive a machine failure, and the
product must say so in plain words rather than in a durability figure nobody
reads. That honesty is the design decision; the alternative is either
lying or replicating 200,000 volumes for customers paying nothing.

---

## 3. The design

```
   container (customer's Postgres)
        │  write() / fsync()
        ▼
 ┌──────────────────────────────────────────────────────────┐
 │ MACHINE: local NVMe (the data path)                       │
 │  · volume as a thin-provisioned LV / file on NVMe         │
 │  · checksum per block                                     │
 │  · reads ALWAYS local  ◄── this is why local wins         │
 └───────┬──────────────────────────────────────────────────┘
         │ writes replicated (tier-dependent)
         ├──────────────► peer machine, DIFFERENT RACK  (sync, prod tier)
         └──────────────► second site (async, critical tier)
         │
         ▼ continuous
 ┌──────────────────────────────────────────────────────────┐
 │ BACKUP PATH → object storage                              │
 │  · block snapshots (crash-consistent) — all tiers         │
 │  · app-aware base + WAL archive (real PITR) — paid tiers  │
 │  · CONTINUOUS RESTORE VERIFICATION  ◄── §3.4              │
 └──────────────────────────────────────────────────────────┘

 ┌──────────────────────────────────────────────────────────┐
 │ VOLUME CONTROL PLANE                                      │
 │  · placement (failure domains: rack, PDU, switch, site)   │
 │  · health: comparative, not threshold (§3.3)              │
 │  · repair orchestration, THROTTLED below customer I/O     │
 │  · publishes attach constraints → problem 07's scheduler  │
 └──────────────────────────────────────────────────────────┘
```

### 3.1 Local disk, replicated at the block layer

Reads are always local — that is the whole reason to choose this shape, and
it means a read-heavy database gets NVMe latency regardless of replication.

Writes on the production tier go to local NVMe *and* synchronously to a peer
in another rack. That costs a network round trip on the write path, which is
the same cost network-attached storage would impose — but only on writes,
and Postgres's group commit amortises it across concurrent transactions.
The RPO is zero for a single machine loss.

The async third copy for the critical tier gives site-loss survival with a
bounded RPO, stated as a number rather than implied.

**Placement respects failure domains, not just capacity.** Two replicas in
the same rack survive a drive failure and not a PDU. This is the kind of
constraint that is trivial to state and easy to get wrong when the scheduler
is optimising for free space.

### 3.2 Repair is a workload that competes with customers

A machine failure means rebuilding ~15 TB. At 1 GB/s that is over four
hours, and during it the volume is at reduced redundancy — a window that
recurs roughly weekly given the failure rate, so a second failure inside it
is a design input, not a tail event.

Two consequences:
- **Throttle rebuild below customer I/O.** An unthrottled rebuild converts
  one machine's failure into a latency incident for every neighbour of every
  replica involved. This is the most predictable noisy neighbour in the
  system.
- **But not too far below** — a slower rebuild extends the reduced-redundancy
  window, which raises the probability of the second failure. The throttle
  is a genuine tradeoff between neighbour latency now and data loss risk
  later, and naming it is better than picking a number silently.

Spreading replicas widely helps both: if a machine's volumes have replicas
on 50 different machines rather than one, the rebuild reads from 50 sources
in parallel and each contributes a trickle. Wide placement is the standard
answer and it should be reasoned to, not assumed.

### 3.3 The slow machine is the hard failure

A dead machine is easy: detect, fail over, rebuild. A machine that is
*slow* — a drive with rising latency, a NIC negotiating at the wrong speed,
a kernel thrashing — will pass every health check while degrading every
volume on it and holding synchronous write quorums open (interviews 07·A13).

Detection must be **comparative**: this drive's p99 against its 23 siblings
in the same machine, this machine against its peers in the same rack.
Absolute thresholds either fire constantly or never. And the response should
be to *eject it from write quorums first* and investigate second, because
the cost of running one replica short for an hour is much lower than the
cost of every write waiting on a sick disk.

### 3.4 The thing nobody designs: proving backups restore

Every platform has backups. Very few can tell you, right now, that a
randomly chosen backup from last Tuesday will restore.

So: **continuous restore verification**. On a schedule, pick a sample of
backups, restore them into a scratch environment, start the database, run a
consistency query, compare against expected, throw it away. Alert on
failure. Track the fraction of backups verified in the last N days as a
first-class SLI.

This costs real compute and it is the single highest-value thing in the
whole backup design, because the failure mode it prevents — discovering at
the worst possible moment that backups have been silently broken for six
weeks — is the one that ends companies.

Related honesty: a block snapshot of a running Postgres is
**crash-consistent**, equivalent to pulling the power. Postgres will usually
recover it via WAL replay. "Usually" is not a product, so paid tiers get
application-aware backup (base backup plus continuous WAL archiving), which
is what makes point-in-time recovery real.

### 3.5 IOPS isolation

67 volumes on 8 drives. Per-volume limits via cgroup v2 `io.max` and
`io.latency`, with a fair-share model: guaranteed floor under contention,
free use of idle capacity otherwise — the same shape as CPU shares
(interviews 05·A8). Most volumes are idle almost always, so hard
provisioning would waste nearly all the hardware.

The customer-visible part: when throttled, they should be able to *see* it,
with the limit and their consumption in the dashboard. Silent throttling
produces support tickets that read "the database got slow for no reason."

---

## 4. Why the main alternative loses

**The alternative: a distributed storage system — Ceph or similar — with all
volumes network-attached, erasure coded, and freely mobile.**

It is a serious, well-understood design and plenty of platforms run it. It
loses here on:

1. **Latency, for the workload that matters.** The customers who care about
   this system are running databases, and a database's commit path is
   `fsync`. Adding 0.5–2 ms to every commit, versus 20–100 µs local, is a
   10–50x regression on the operation the customer measures. Group commit
   helps and does not close a gap that large.
2. **Erasure coding is worse for small random writes**, which is what a
   database does — read-modify-write across stripes amplifies exactly the
   pattern a transactional workload generates. Erasure coding is excellent
   for large objects and poor for this.
3. **It makes storage traffic into network traffic, permanently.** Every
   read that would have been local now crosses the fabric. At 3,000 machines
   that is a large, permanent network cost and a new failure domain — the
   fabric — shared by every database on the platform.
4. **Operational weight.** Ceph is a distributed system you now operate, with
   its own failure modes, its own rebalancing behaviour, and a scarce
   expertise requirement. That is a real headcount cost.

**Where it genuinely wins**, and it should be conceded: volumes become
*mobile*, which dissolves the pinning problem that damages problem 07's
scheduling density, makes machine maintenance a non-event, and makes
capacity management vastly simpler. If the workload mix were mostly file
storage and light databases rather than production Postgres, it would be the
right answer. The choice here is driven by what customers actually run.

---

## 5. Where reasonable engineers would disagree

**5.1 A free tier with no replication.** I've argued for honesty over
uniform durability. Plenty of people would say a platform should never lose
customer data regardless of what they paid, and that the reputational cost
of "we lost your hobby project" exceeds the storage saving. That's a
defensible position and it costs roughly 200,000 volumes' worth of second
copies. The counter is that a free tier that costs 3x to store is a free
tier that gets cancelled.

**5.2 Synchronous replication on the write path.** It gives RPO 0 and it
puts a network round trip in every commit — and it means a slow peer slows
the primary (interviews 06·A9). Asynchronous with a bounded RPO of a few
seconds is much faster and loses a few seconds of transactions in a failure.
For most customer workloads I suspect async plus honest RPO disclosure is
the better product; I've chosen sync because "we lost your last four seconds
of orders" is a conversation nobody wants.

**5.3 Whether to offer raw volumes at all.** A managed Postgres offering is
a *better* product and an *easier* storage problem — you control the
replication, the backup, the version, and the failover, and you stop
supporting the long tail of things people run on a volume. It's also a much
bigger product commitment. If I were arguing strategy rather than
architecture, I'd push hard for managed databases as the primary offering
and raw volumes as the escape hatch.

**5.4 Rebuild throttling.** I throttle below customer I/O. Someone focused
on durability would argue the reduced-redundancy window is the greater risk
and rebuild should get priority. Both are right; the answer depends on the
replication factor and how likely a second failure actually is, which is
computable — and the fact that it's computable means this shouldn't be an
opinion.

**5.5 What I'd cut to ship in a quarter.** Cut the third async copy, cut
erasure coding entirely, cut volume mobility. Ship: local NVMe, synchronous
peer replication for paid tiers, snapshots to object storage for everyone,
restore verification, and per-volume I/O limits. That's a defensible
platform. Restore verification stays in the first cut, which is the choice
worth defending — it's the cheapest thing on the list and the one whose
absence is most likely to be catastrophic.
