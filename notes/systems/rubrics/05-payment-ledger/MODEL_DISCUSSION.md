# Model Discussion — 05 The Payment Ledger

> **Spoiler.** Do not open until the final review round is written.

One credible design, why its main alternative loses, and where competent
Staff engineers would still disagree.

---

## 1. The arithmetic says the problem is contention, not volume

| | Value | Verdict |
| --- | --- | --- |
| Ledger entries/s, peak | ~98,000 | 2–5x a single relational primary — tight, not fatal |
| 7-year storage | ~120–230 TB compressed | Large, not the binding constraint |
| **Fee-account updates/s, peak** | **14,000 on one row** | **3–14x a serialised row, and unsplittable by account** |
| Balance reads/s | 200,000 | 29x the payment rate — a projection is mandatory |
| Fold cost for a large merchant, 14 months | millions of entries | Why the last audit request took eleven days |

The naive read of "86M events/day, shard it" is wrong. Volume is roughly one
order of magnitude from a single node — uncomfortable but survivable.
**What kills the design is that 100% of transactions touch two internal
accounts.** Sharding by account makes it *worse*: now every transaction is
a distributed transaction, and the shard holding the fee account serialises
the company.

So the design question is not "how do we scale writes." It is **"how do we
stop every transaction from meeting at the same point."**

---

## 2. Two reframes

### 2.1 A balance is not state. It is a fold.

Stop storing mutable balances. Store **immutable entries** and derive
balances.

The hot row disappears — not because it was optimised, but because there is
no longer a row to update. 14,000 appends/second to an ordered log is
unremarkable; 14,000 updates/second to one row is impossible. The contention
was an artifact of the representation.

This is also what makes the system auditable: an immutable, ordered,
append-only record is what an auditor wants, and every derived thing
(current balance, as-of balance, snapshots, reports) is a **disposable
cache** that can be rebuilt from it. The entries are the asset; everything
else is a projection.

### 2.2 The fee account is not one account.

Split the internal accounts into **N sub-accounts, one per shard**. A
transaction on shard 7 credits `platform_fees:7`. The true fee balance is
`Σ platform_fees:i`, computed on demand or rolled up on a schedule.

The consequence is the important part: **every transaction becomes
single-shard.** Buyer, merchant, and the local fee/clearing sub-accounts all
live on the same shard, so there is no distributed transaction, no 2PC, no
coordinator, and the double-entry invariant is enforced by an ordinary local
transaction — the strongest guarantee available, at the cheapest price.

The cost is stated plainly: **the platform fee balance is eventually
consistent intraday.** A finance user querying it at 14:00 gets a sum that
may lag by the rollup interval. That is a real cost, it lands on a real
person, and it is the row in the tradeoffs ledger that the Director of
Financial Controls has to accept. In practice it is easily accepted —
nobody makes decisions on an intraday fee balance — but it must be asked,
not assumed.

---

## 3. Bitemporality: the requirement hiding in the Director's message

Every entry carries **two independent times**:

| Axis | Meaning | Mutable? | Set by |
| --- | --- | --- | --- |
| **Effective time** | When the economic event occurred | Not for a given entry, but new entries can land with past effective times | The caller |
| **Posting time** | When the ledger recorded it | Never | The ledger |

This is what makes the Director's three questions all answerable:

| Question | Query |
| --- | --- |
| "Balance on 14 March, as we understand it today" | effective ≤ 14 Mar, posting ≤ now |
| "Balance on 14 March, as we believed on 14 March" | effective ≤ 14 Mar, posting ≤ 14 Mar |
| "What changed our understanding since?" | the difference between the two |

And it answers *"does a past balance change?"* precisely:

> **The balance as of an effective instant can change as corrections
> arrive. The balance as recorded at a past posting time never changes.**

Immutability lives on the posting axis. That is the sentence that resolves
the apparent contradiction in the Director's paragraph, and a design that
cannot say it has not met the requirement.

**A chargeback today for a 90-day-old payment** is therefore one entry with
`effective = 90 days ago, posting = today`, linked to the entry it corrects.
The March effective-time view now includes it; the "as known in March" view
does not; the closed March books are untouched. Both readings agree because
they are the same data queried on different axes.

**Corrections into closed periods** follow the standard accounting policy:
the closed period is *not restated*; the correction is recognised in the
current period with a past effective date. Finance keeps its closed books,
the regulator gets the effective-time truth, and nobody is lying to anyone.

---

## 4. The design

```
   payment authorisation (80 ms budget)
              │
              ▼
   ┌───────────────────────────────────────────────────────┐
   │ LEDGER WRITE SERVICE                                   │
   │  · idempotency key check (caller-supplied)             │
   │  · route by account-group → shard                      │
   │  · ONE local transaction containing ALL legs           │
   │    (buyer, merchant, fee:N, clearing:N)                │
   │  · double-entry checked in-transaction, structurally   │
   └──────────────┬────────────────────────────────────────┘
                  │ append
   ┌──────────────▼────────────────────────────────────────┐
   │ ENTRY STORE — sharded, append-only, immutable          │
   │  · per-shard monotonic sequence (gap detection)        │
   │  · (effective_time, posting_time) on every entry       │
   │  · 7 years; hot → warm → cold tiers                    │
   │  ─────────────── THIS IS THE SYSTEM OF RECORD ─────────│
   └──────┬───────────────────┬──────────────┬─────────────┘
          │ CDC               │ CDC          │
   ┌──────▼──────┐   ┌────────▼───────┐  ┌───▼─────────────┐
   │ BALANCE     │   │ SNAPSHOTS      │  │ RECONCILIATION   │
   │ PROJECTION  │   │ per account    │  │ vs card/ACH/     │
   │ · 200k rd/s │   │ per day        │  │ wallets          │
   │ · lag SLO   │   │ · derived      │  │ · break          │
   │   stated    │   │ · rebuildable  │  │   categories     │
   │ · rebuildable│  │ · verified     │  │ · auto-resolve   │
   └─────────────┘   └────────────────┘  └─────────────────┘
          │                   │
          └─── as-of query = nearest snapshot + bounded fold
```

### 4.1 The write path in 80 ms

Idempotency check, shard route, one local ACID transaction, done. No
cross-shard coordination, no consensus beyond the shard's own replication,
no synchronous projection update. The budget is comfortable, which is the
payoff from §2.2 — the design is fast *because* it is single-shard, and it
is single-shard *because* the internal accounts were split.

**Double-entry is structural**: all legs of a transaction are in one append
within one local transaction, so "an entry written without its counterpart"
is not a failure mode that can occur. That is worth more than any
asynchronous checker.

### 4.2 Balances at 200k reads/s

An asynchronous projection with a **stated lag SLO** (say p99 under 2
seconds) and a documented staleness contract on the API. It is rebuildable
from entries, and a background job continuously verifies a sample of
projected balances against a fold.

Merchants polling dashboards tolerate seconds. If read-your-writes is
required for a specific flow, that flow reads the shard directly rather than
the projection — an explicit, narrow exception rather than a system-wide
consistency requirement.

### 4.3 As-of queries: snapshots plus a bounded fold

Per-account daily snapshots on both axes. An as-of query becomes *nearest
snapshot + fold the entries since*, which bounds the work to one day's
entries for that account regardless of how far back the question reaches.

Two properties that matter:
- **Snapshots are derived, never authoritative.** They can always be
  rebuilt, and a verification job does exactly that on a rolling sample. A
  snapshot that silently drifts is an audit failure, so it must be checked,
  not trusted.
- Cadence is a computed tradeoff: daily snapshots for 36M accounts over 7
  years is a lot of rows, so snapshot only accounts with activity in the
  period, and tier old snapshots to cold storage alongside the entries.

The eleven-day incident becomes a query measured in seconds. That is the
concrete deliverable to the Director, and it is worth stating in those
terms.

### 4.4 Invariants: some structural, some detected

| Invariant | How | Why |
| --- | --- | --- |
| Debits = credits per transaction | **Structural** — single local transaction | Non-negotiable; must never be violable |
| No duplicate transaction | **Structural** — idempotency key uniqueness | Cheap to enforce at write |
| No lost entry | **Detected** — per-shard sequence gap detection | Cannot be prevented, must be provable |
| No negative *buyer* balance | **Structural** — checked in the shard transaction | Cheap, single-shard |
| No negative *merchant* balance | **Accept and detect** | A refund after payout legitimately overdraws; refusing it is worse than a negative. Becomes a collections workflow |
| Fee sub-accounts sum correctly | **Detected** — rollup verification | Eventual by construction |

Distinguishing these is the substance of the design. Enforcing everything
synchronously is how a ledger fails to hit its throughput; enforcing nothing
synchronously is how it fails an audit.

### 4.5 The money-leak case

**"The card network says the payment succeeded; the ledger has no record."**
This is unavoidable — two systems, one network, no distributed transaction
across the boundary (interviews 07·A8). The design must own it:

- The payment service writes a **pending intent** before calling the
  network, with the same idempotency key it will use for the ledger.
- Reconciliation matches network settlement files against ledger entries on
  that key, daily and ideally intraday.
- An unmatched network-side success becomes a repair posting, replayable
  safely because of the idempotency key.
- **Bounded detection time is the requirement**, and it should be stated:
  intraday reconciliation gives hours, not days. A design that only
  reconciles at T+1 has a day-long window of silent leakage and should say
  so.

---

## 5. Why the main alternative loses

**The alternative: keep the strongly-consistent relational model, scale it
up, and use distributed transactions across shards where necessary.** Every
transaction is an ACID transaction spanning the buyer's shard, the
merchant's shard, and the fee account's shard, coordinated by 2PC.

It loses on three grounds:

1. **The fee account is a serialisation point for the entire company.**
   Every transaction takes a lock on one row; at 14k/s against a
   ~1–5k/s serialised ceiling, the queue never drains and latency diverges.
   No amount of hardware fixes a serialised row. This ends it.
2. **2PC on 100% of transactions.** Every payment becomes a distributed
   transaction with a prepare phase holding locks across shards. Latency is
   multiple round trips inside an 80 ms budget, throughput is bounded by the
   slowest participant, and a coordinator failure leaves participants
   blocked holding locks on the hottest rows in the system (interviews
   07·A9). The failure mode is a company-wide payment stall.
3. **Mutable balances are not auditable.** An `UPDATE balances SET
   amount = ...` destroys the history that the seven-year requirement is
   about. You would have to keep an audit log alongside — at which point you
   have written the entry log anyway, but as a secondary artifact that can
   disagree with the primary one. Two sources of truth in a system of
   financial record is the worst outcome available.

**Where the alternative genuinely wins**, and it should be conceded: it is
*far* simpler, and it gives strictly-consistent balance reads for free with
no projection lag, no snapshot verification, and no eventual-consistency
conversation with finance. At 1,000 payments/second with no peak and no
internal-account concentration, it would be the right design and the
event-sourced version would be over-engineering. The current system has
worked for nine years for exactly that reason. It is the *peak* and the
*concentration* that break it, not the average.

---

## 6. Where reasonable Staff engineers would disagree

**6.1 Event sourcing as a whole.** It is a significant complexity
commitment: projections to build and rebuild, snapshot verification,
eventual consistency to explain to non-engineers, and a much larger surface
for subtle bugs. A well-partitioned relational design with sub-accounts and
a *separate* immutable audit log would satisfy most of the requirements with
far less novelty, at the cost of two artifacts that must be kept in
agreement. Several serious payment companies run exactly that. I prefer one
source of truth; I would not call the alternative wrong.

**6.2 Accept-and-detect on negative merchant balances.** I've argued that
refusing a legitimate refund because the merchant's balance would go
negative is worse than allowing it. A risk team might reasonably say that an
unbounded negative balance is an unsecured loan to a merchant who may
disappear, and that the system should refuse past a threshold. That's a
credit-policy question, not an engineering one, and the right answer is
probably "allow, with a per-merchant limit set by risk."

**6.3 Snapshot cadence.** Daily per active account is my default, and it
generates a lot of rows. Weekly with a longer fold is cheaper and slower;
on-demand snapshot materialisation (compute and cache when first asked) is
cheapest and has a terrible first-query latency for exactly the audit
request you care about. Reasonable people optimise this differently, and the
right answer depends on the actual distribution of as-of queries, which
nobody has measured.

**6.4 Whether the ledger may reject a payment.** I've made the ledger the
system of record, which implies a failed ledger write should fail the
payment. The counter-argument is strong: refusing revenue because a
bookkeeping system is degraded is a bad trade, and a durable pending queue
lets you accept and record later. My objection is that the queue then *is*
the system of record and should be designed as one rather than treated as a
buffer — but a design that makes that explicit and accepts writes into a
durable log ahead of the ledger is arguably the same thing with better
availability.

**6.5 Sub-account count.** More sub-accounts means less contention and a
more expensive rollup and a more confusing chart of accounts for finance. I'd
match N to the shard count and no more. Someone optimising purely for write
throughput would go higher and have to explain 256 fee sub-accounts to an
auditor, which is a real cost in a way that doesn't show up in a latency
graph.

**6.6 Whether to build this at all.** There are mature ledger products and
double-entry engines. For a marketplace at this scale the ledger is
arguably core — the fee logic, the payout rules, and the reconciliation are
business-specific — but a Staff engineer should price the alternative before
proposing a nine-year replacement for a nine-year-old system, if only to
document why not. "We never evaluated buying" is not something you want to
say to a Director of Financial Controls.

**6.7 The riskiest thing in this design.** The migration, not the
architecture. A ledger that is 99.99% correct is worthless; the equivalence
proof during parallel running is where this project succeeds or fails, and
it deserves more engineering than the write path does. If I had to cut
scope, I would cut features from the new ledger before I cut a single week
of parallel verification.
