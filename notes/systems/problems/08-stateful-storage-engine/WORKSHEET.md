# Worksheet — 08 The Stateful Storage Engine

---

## Part 1 — Scoping the unstated

### 1.1 What you decided the question was

Three sentences, as you'd say them in the first two minutes.

### 1.2 Decisions you had to make unprompted

| # | Question the prompt didn't answer | Your answer | Why |
| --- | --- | --- | --- |
| 1 | What is a volume, as a product? | | |
| 2 | Local disk or network-attached? | | |
| 3 | Who replicates — you or the customer's database? | | |
| 4 | What durability do you promise, per tier? | | |
| 5 | Are backups in scope? Snapshots? Restore? | | |
| 6 | What's the interface to the scheduler? | | |
| 7 | What scale? | | |
| 8 | | | |

### 1.3 Explicitly out of scope

At least four, with boundaries.

### 1.4 The durability promise

Write it as a sentence you would put in front of a customer, with a number.
Then write what it costs you to keep.

| Tier | Durability promise | Mechanism | Cost multiple | What a customer loses in the worst case |
| --- | --- | --- | --- | --- |
| | | | | |

If every tier gets the same promise, say why that's right rather than
defaulting to it.

---

## Part 2 — Back-of-envelope

**Do this section before you design anything.** The failure arithmetic is
the problem.

### 2.1 How often things break

| Quantity | Calculation | Result |
| --- | --- | --- |
| Total drives | | |
| Drive failures/year, and interval between them | | |
| Machine failures/year, and interval | | |
| Volumes per machine | | |
| **Volumes affected per machine failure** | | |
| **Volumes lost per year with no replication** | | |

State that last number out loud. It is the argument.

### 2.2 What replication costs

| Scheme | Raw capacity needed | Write amplification | Failures tolerated | $/month at your assumed disk cost |
| --- | --- | --- | --- | --- |
| None | | | | |
| 2x replication | | | | |
| 3x replication | | | | |
| Erasure coding (state the scheme) | | | | |

### 2.3 Rebuild

| Quantity | Calculation | Result |
| --- | --- | --- |
| Data on one machine | | |
| Rebuild time at your network speed | | |
| Time at reduced redundancy | | |
| Probability of a second failure during that window | | |
| Rebuild traffic as a share of your network | | |

### 2.4 Performance

| Quantity | Calculation | Result |
| --- | --- | --- |
| Local NVMe latency (state your reference figure) | | |
| Network-attached latency over your fabric | | |
| Ratio | | |
| Effect on a Postgres commit (fsync per commit) | | |
| Effect with group commit | | |
| IOPS available per machine, and per volume if shared fairly | | |

### 2.5 Cost

| Component | Driver | Share |
| --- | --- | --- |

What does 1 GB/month cost you to store at your chosen durability, and what
would you have to charge?

---

## Part 3 — Data model & API

- What does the compute engine ask for, and what does it get back?
- How is a volume identified, and what is its lifecycle independent of the
  container using it?
- What does "attach" mean, mechanically, and how long does it take?
- Snapshot, restore, resize, fork, delete — define the semantics of each you
  support, including what a snapshot of a running database actually
  guarantees.

---

## Part 4 — Architecture

Diagram plus one sentence per component. Show explicitly:

- The write path from the customer's `fsync()` to durable media, with the
  latency budget.
- Where replicas live and what chooses their placement.
- The control plane: who decides where a volume lives and who repairs it.
- The interface to the scheduler in problem 07.
- The backup path and where backups are stored.

---

## Part 5 — Deep dives

**Pick two.**

**Candidate A — Local versus network-attached, decided properly.**
Local NVMe: the performance a database needs, and it pins the workload and
dies with the machine. Network-attached replicated: mobility and durability
at a large latency multiple. Design your answer — including a hybrid if you
propose one — with the numbers. What does each do to the scheduler in
problem 07? What does each do to your machine-maintenance story?

**Candidate B — Failure and repair.**
Something breaks most days. Design detection, isolation, repair, and the
customer experience for each of: a drive failing, a machine failing, a
machine being *slow* rather than dead, a rack losing power, and silent data
corruption. Include rebuild time, the reduced-redundancy window, and what
happens if a second failure lands inside it.

**Candidate C — Backups, snapshots and restore.**
A snapshot of a running Postgres is a crash-consistent image, not a
consistent backup. Design what you actually offer: what is snapshotted, how
often, where it's stored, what point-in-time recovery means, what restore
costs and how long it takes. Then design the thing nobody designs: how do
you know a backup is restorable *before* a customer needs it?

**Candidate D — Multi-tenant IOPS.**
67 volumes per machine sharing 8 drives. One tenant's batch job can destroy
another tenant's database latency. Design the isolation: what you limit,
where you enforce it, how you decide a fair share when most volumes are
idle, and what a customer sees when they're throttled. Include what happens
during a rebuild, which is your *own* workload competing with customers.

---

## Part 6 — Failure modes

| Failure | Blast radius | Detection | Mitigation | Degraded behaviour |
| --- | --- | --- | --- | --- |
| A drive fails | | | | |
| A machine fails with 67 volumes | | | | |
| A machine is slow, not dead | | | | |
| A rack loses power | | | | |
| Silent corruption on one replica | | | | |
| A second failure during a rebuild | | | | |
| A customer fills their volume at 3am | | | | |
| A customer deletes their data and wants it back | | | | |
| A site is lost | | | | |

Then: **when you cannot guarantee durability for a write, do you accept it
or refuse it?** Answer for each tier you defined.

---

## Part 7 — Tradeoffs ledger

| Decision | Chosen | Rejected | Cost accepted | Who feels it |
| --- | --- | --- | --- | --- |

At least 10 rows. At least one where the accepted cost is a bounded amount
of customer data loss — and if there is no such row, say why not.

---

## Part 8 — Evolution

- What breaks first at 10x — component, limit, number.
- What breaks second.
- What changes if a customer wants a 10 TB volume?
- What changes if you offer a managed Postgres rather than a raw volume?
  (Does the storage engine get simpler or harder?)
- What changes if a customer needs their data to stay in one jurisdiction?
- What would you need to add to support a volume attached to more than one
  container at once?

---

## Part 9 — Operations & cost

- Dominant cost driver, with the number, and the most effective lever.
- Top 3 alerts.
- **The 3am page:** a machine with 67 volumes is unresponsive, 12 of them
  are production databases. Walk through it. Be specific about what the
  on-call must *not* do.
- How do you take a machine out of service for a firmware update?
- A customer says they lost data. What do you need to answer them?
- What is the one operation you'd most want during an incident?

---

## Part 10 — The live-extension drill

| # | "What if…" | Your answer | What changes |
| --- | --- | --- | --- |
| 1 | …a machine dies right now with 67 volumes on it? | | |
| 2 | …the customer's database already replicates itself — are you paying twice? | | |
| 3 | …a customer needs 50k IOPS sustained? | | |
| 4 | …you have to cut storage cost in half? | | |
| 5 | …a rebuild is saturating the network and customers are complaining? | | |
| 6 | …a customer wants to restore to 10 minutes ago? | | |
| 7 | …one tenant's I/O is destroying a neighbour's database latency? | | |
| 8 | …you discover a replica has been silently corrupt for a month? | | |
| 9 | …the free tier can't be replicated economically? | | |
| 10 | …we want volumes to move between sites? | | |

Then: **name the three questions you most hope they don't ask**, and answer
them anyway.
