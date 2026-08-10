# Rubric — 05 The Payment Ledger

> **Spoiler.** Do not open until the final review round is written.

This is the hardest problem in the set. Grade accordingly — a design that
would be Staff on problem 03 may be Borderline Staff here.

---

## The central tension

**Correctness and auditability demand an immutable, totally-ordered,
strictly-invariant record. Throughput and latency demand partitioning,
concurrency, and asynchrony. In a ledger, the usual escape hatch — relaxing
consistency and reconciling later — is only partly available, because the
thing you would reconcile is the thing being audited.**

The specific squeeze:

1. **Double-entry is a cross-entity invariant.** Every transaction must sum
   to zero across accounts that live on different shards. That is a
   distributed transaction by definition — you cannot express "debits equal
   credits" as a CRDT and you cannot last-writer-wins it (interviews 07·A15).
2. **The internal accounts are touched by 100% of transactions.** So
   *every* transaction is cross-shard, and the shard holding the fee account
   serialises the entire system. Sharding by account does not help; it makes
   things worse by turning a hot row into a hot row plus 2PC.
3. **80 ms budget** forbids anything that looks like a cross-region quorum
   per write, and makes even a same-region 2PC expensive at 14k/s.

**The insight the problem is built around: a ledger entry is an
append-only fact, and a balance is a derived projection. Once you stop
storing balances as mutable state, the hot account stops being a hot *row*
and becomes a hot *append*, which is a solved problem.**

That reframe — from "update a balance" to "append an entry, project a
balance" — is what unlocks throughput without losing correctness. Everything
else (snapshots, accumulators, projections) follows from it.

But the reframe does not come free, and the problem stays hard because:
- A balance is now a fold over history, and folds are expensive at 7 years.
  Snapshots fix that and introduce their own correctness question (a
  snapshot is a cached derivation of an auditable number).
- "No negative balance" and "debits equal credits" still need enforcement,
  and enforcing them on an append-only log requires either a serialisation
  point per account or an accept-then-detect model, which for money is a
  business decision.
- The projection is now asynchronous, so a balance read is stale by
  construction, and the brief never says how stale is acceptable.

---

## The requirement that is actually the hardest constraint

**The Director's paragraph about the chargeback.** It is written as a
clarification and it is the tightest constraint in the brief.

It requires **bitemporal modelling**: every entry carries at least two
independent times —

- **effective / value time** — when the economic event occurred (the
  original payment, 90 days ago),
- **posting / record time** — when the ledger learned of it (today).

And both readings must be derivable and must agree:
- *"What was the balance on 14 March?"* → as-of on effective time.
- *"What did we believe the balance was on 14 March, on 14 March?"* → as-of
  on both axes ("as known then").
- *"What do we now believe the balance was on 14 March?"* → effective-time
  as-of with today's record time.

The last two differ precisely by corrections that arrived later, and both
are legitimate questions from different askers — the second is what was
reported to the regulator at the time; the third is the truth as currently
understood. A financial system that cannot produce both cannot explain a
restatement.

**A candidate who stores one timestamp has, without realising it, made the
Director's requirement unsatisfiable.** This is the single highest-signal
discriminator in the problem, and it is why the Director's message is in the
brief at all.

The follow-on question — *"does a past balance ever change?"* — has a
precise answer: **the balance as of an effective instant can change as
corrections arrive; the balance as recorded at a past record time never
changes.** Immutability lives on the record-time axis. A candidate who says
"no, the past never changes" has missed the chargeback; one who says "yes,
we update it" has broken immutability. The correct answer is "both, on
different axes," and it is worth probing hard for.

---

## Ambiguities a strong candidate must catch

**Catching 6+ of the first tier is a Staff signal. Fewer than 4 caps at
Senior. This problem has more first-tier items than the others because it is
harder.**

### First tier — changes the architecture

| # | Ambiguity | Why it matters |
| --- | --- | --- |
| 1 | **Which timestamps does an entry carry, and can a client set them?** | Bitemporality is the load-bearing decision. Never stated as a requirement — only implied by the Director. |
| 2 | **Is the ledger the system of record, or a projection of a payment service?** | Decides whether it may reject a write. If it's the record, a failed ledger write must fail the payment; if it's a projection, it must accept everything and reconcile. |
| 3 | **May an account go negative?** | Merchants absolutely can (refunds after payout). Buyers presumably cannot. This is a per-account-type invariant and it determines whether writes need a serialisation point per account. |
| 4 | **How stale may a balance read be?** | 200k reads/s of "a little stale" — the brief says nobody has specified it. If it must be read-your-writes after a payment, the projection is on the critical path; if seconds of lag are fine, it isn't. |
| 5 | **Is a transaction atomic across accounts, and what does the caller see on partial failure?** | The double-entry invariant made operational. |
| 6 | **Does the payout run need a consistent snapshot across all merchants, or per merchant?** | A global consistent cut is far more expensive than per-merchant, and nobody says which is required. |
| 7 | **What is the acceptable break rate, and who owns breaks?** | 200/day today with no stated target. If breaks scale with volume, 10x means 2,000/day and the team doesn't scale with it. |
| 8 | **Can a closed accounting period be modified?** | Critical. Finance closes monthly; a chargeback for a closed period is either a restatement or a current-period adjustment, and that is an accounting-policy decision that dictates the data model. |
| 9 | **Is FX / multi-currency in scope now?** | Part 8 asks; the brief says "marketplace" without stating. Retrofitting currency into an entry model is painful. |

### Second tier

| # | Ambiguity |
| --- | --- |
| 10 | Idempotency: caller-supplied key or ledger-generated? What is the dedup window against a 120-day chargeback horizon? |
| 11 | Retention granularity — 7 years of entries, or 7 years of *queryable* entries? Cold-storage latency for an audit request. |
| 12 | Who is allowed to post a correcting entry, and is there an approval path? |
| 13 | What happens to the 80 ms budget during the payout window? |
| 14 | Are balance reads authenticated per merchant (i.e. can they be served from a per-merchant cache)? |
| 15 | Does the reconciliation process write to the ledger, or only report? |
| 16 | Is there a hard requirement to keep the old Postgres schema readable after migration for the auditors? |

---

## Section-by-section

### Part 1 — Requirements & scope

**Senior:** Restates requirements. States ACID as a requirement. May treat
"as of any historical instant" as a query-performance problem.

**Staff:** Section 1.2 identifies the two timelines by name and answers the
"does a past balance change" question correctly on both axes. Section 1.3
distinguishes invariants enforced synchronously from those detected
asynchronously, with a cost per violation — a strong candidate recognises
that *some* invariants (double-entry) must be structural, while others
(no negative merchant balance) may be accept-and-detect, because refusing a
legitimate refund is worse than a temporary negative.

**Red flags:** No mention of two timestamps. "Strong consistency" as a
requirement with no object. Treating all invariants as synchronous, which
guarantees the design won't hit 14k/s.

---

### Part 2 — Back-of-envelope

Recompute independently.

| Quantity | Defensible value | Working |
| --- | --- | --- |
| Payments/s | **1,000 avg, 14,000 peak** | Stated |
| Ledger entries/s | **7,000 avg, ~98,000 peak** | × 7 |
| Entries/year | **~220B** | 7,000 × 31.5M s |
| Entries over 7 years | **~1.5T** | |
| Bytes/entry | **~80–150 B** | Account id, amounts, currency, two timestamps, transaction id, type, metadata ref |
| 7-year storage | **~120–230 TB** compressed | Large but not the binding constraint — say so |
| Single relational primary sustained writes | **~20–50k/s** | Order of magnitude |
| **Factor over naive at peak** | **~2–5x** | Tight but not absurd — which is why the *throughput* is not what kills the single node |
| **Fee-account updates/s at peak** | **14,000/s on one row** | Every payment |
| Single serialised row update rate | **~1,000–5,000/s** | **This is the binding constraint: 3–14x over, and it cannot be sharded away** |
| Balance reads/s | **200,000/s** | Stated |
| **Read : write ratio** | **~29 : 1** vs payments, **~2 : 1** vs entries | Justifies a materialised projection rather than folding on read |
| Entries for a large merchant over 14 months | **millions** | A merchant doing 1,000 payments/day × 425 days × ~2 entries ≈ 850k; a large one is far more. Folding this per audit request is why the eleven-day incident happened |
| Payouts/s in the window | **~83/s** | 900,000 ÷ 3 h. Modest in rate, but each needs a consistent balance and writes entries |
| Payout entries | **~1.8M–3.6M over 3 h** | Small compared to daily volume; the contention is the issue, not the volume |

**Senior:** Computes volume and storage. Notes the hot row.

**Staff:** Reaches the conclusion that **raw throughput is only 2–5x over a
single node, but the hot account is 3–14x over on a single row that cannot
be partitioned** — so the problem is *contention*, not volume. That
distinction determines the whole design and separates candidates who did the
arithmetic from those who assumed "big system, must shard."

Also notices that 7 years of storage is not the binding constraint, which is
worth saying — it stops the design being organised around the wrong number.

**Red flags:** Concluding "we need to shard because the volume is high"
without noticing that sharding by account makes the internal-account problem
worse. Not computing the fold cost for a historical query, which is the
requirement that already cost eleven days.

---

### Part 3 — API & data model

This section carries more weight in this problem than in the others.

**Senior:** Entries with an account, amount, timestamp, and transaction id.
Balances table updated on write.

**Staff:**
- **Entries are immutable.** No updates, no deletes. A reversal is a new
  entry, not a mutation. If a candidate has an `UPDATE` anywhere near an
  entry, the design is not auditable and the review should say so plainly.
- **Two timestamps minimum** (effective and posting), with the client
  permitted to set effective and *never* posting.
- **Balance is a projection**, not stored state — or, if stored, explicitly
  a cache with a defined derivation and a way to rebuild.
- **Idempotency** with a caller-supplied key, with a retention window
  reasoned against the 120-day chargeback horizon.
- Correction/reversal modelled as a first-class entry type that *links to*
  the entry it corrects, so the audit trail is navigable.

Partitioning: by account is natural for merchants and buyers and fails for
internal accounts, which must be handled specially (see Deep Dive A).

**Red flags:** Mutable balances as the source of truth. One timestamp.
Reversal by deletion or update. No idempotency key.

---

### Part 4 — High-level architecture

**Staff:** The write path shows an append to an ordered, durable entry store
with the 80 ms budget allocated per hop, and the balance projection as an
*asynchronous consumer* with a stated lag SLO. The as-of query path is
visibly different from the current-balance path — snapshots plus a bounded
fold — and the reconciliation path is a distinct component with its own
inputs and outputs.

**Red flags:** One "ledger service" box. Balance served by the same
mechanism as as-of queries. Reconciliation absent from the diagram entirely.

---

### Part 5 — Deep dives

**A (hot account)** and **B (as-of / bitemporality)** are the two hardest
and the two the tension lives between. A+B is the strongest pairing.
A+C and B+D are defensible. C+D avoids both core problems — note it.

#### A — Hot account and cross-shard atomicity

**Staff:** Recognises that the fee account cannot be a single mutable row
and picks a mechanism:
- **Per-shard sub-accounts / accumulators.** The fee account is split into
  N logical sub-accounts, one per shard. Each transaction credits the fee
  sub-account *on its own shard*, so the transaction becomes single-shard
  and the contention divides by N. The true fee balance is the sum of
  sub-accounts, computed on read or rolled up periodically. This is the
  standard answer and it is correct.
  - The cost, which must be stated: the fee account's balance is now
    eventually consistent and a finance user querying it mid-day gets a sum
    that may lag. Whether that is acceptable is a finance question, and a
    strong candidate says who they'd ask.
- **Append-only with no balance row at all** — the contention disappears
  because there is nothing to update; the balance is a fold. Combined with
  sub-accounts and snapshots this is the cleanest answer.
- **Deferring the internal leg** — post the buyer/merchant legs
  synchronously and the fee leg asynchronously. This *breaks double-entry
  atomicity* for a window, which for some organisations is unacceptable and
  for others is fine with a guaranteed-completion mechanism. If a candidate
  proposes it, probe hard on what a mid-window audit sees.
- **2PC across shards** — correct, and it must be priced: latency, and a
  coordinator failure blocking participants holding locks (interviews
  07·A9). At 14k/s with every transaction cross-shard, this is the design
  that doesn't hit the budget, and saying so with numbers is the right move.

**Strong signal:** noticing that sub-accounts turn every transaction from
multi-shard to **single-shard**, which eliminates distributed transactions
entirely rather than optimising them. That is the elegant result and it
should be recognised as such.

#### B — As-of queries and bitemporality

**Staff:**
- Bitemporal entry model, as above.
- **Snapshots** at a chosen cadence (per account per period — e.g. daily or
  every N entries) so an as-of query is `nearest snapshot + bounded fold`.
  The cadence is a real tradeoff and should be computed: storage vs fold
  cost, with the target as-of latency driving it.
- **Snapshots are derived, not authoritative** — they must be rebuildable
  from entries, and there should be a verification job that rebuilds and
  compares. A snapshot that drifts silently is an audit failure.
- **Corrections into a closed period**: the accounting-policy answer is
  usually *the closed period is not restated; the correction is posted in
  the current period with an effective date in the past*, so both readings
  remain derivable — the effective-time view shows it in March, the
  record-time view shows it in June, and the closed books stay closed. A
  candidate who states this policy and shows how the model supports both
  views has answered the Director completely.
- What an auditor receives: the entries, the snapshot, the derivation, and a
  statement of which timeline the number is on.

**Red flags:** Snapshots as authoritative state. No answer for corrections
into closed periods. An as-of query that folds all history.

#### C — Reconciliation

**Staff:** Defines matching keys (network reference, our transaction id,
amount, currency, date window), categorises breaks (timing, amount, missing
on either side, duplicate, fee discrepancy, FX), and states which categories
auto-resolve. Crucially: addresses **how the break rate is kept sub-linear
in volume** — auto-matching rules, tolerance windows, and treating a
recurring break category as a bug to fix rather than a queue to staff. At
10x volume, 200 breaks/day becomes 2,000/day and the team does not 10x.

Chargebacks: a new entry with an effective time 90 days prior, linked to the
original, posted in the current period (per the policy above). And the
knock-on — a merchant already paid out for a transaction now reversed —
must be addressed, because that's a negative balance and a collection
problem, which is where the "may an account go negative" ambiguity pays off.

#### D — Migration

**Staff:** This is a system of record that auditors trust and finance closes
from, so the migration must be *provable*, not merely careful:
- **Dual-write or CDC into the new ledger**, running in parallel for a full
  audit cycle (a month minimum, a quarter better).
- **Continuous equivalence verification**: not just row counts but balance
  comparison per account per day, with any divergence investigated before
  it accumulates. This is what you show the auditor.
- **Cut reads over before writes**, per consumer, so the new system is
  proven under real query load before it owns the truth.
- The old system stays readable for the auditors, and the point of no
  return (when the old system stops being maintained) is named and signed
  by the Director of Financial Controls, not by engineering.
- The **month-end close** is a freeze window. A migration step during close
  is a career event.

**Red flags:** Big-bang. No equivalence proof. No named sign-off. Not
knowing that close is a freeze.

---

### Part 6 — Failure modes

**Staff:** The two questions at the bottom of the section are the
discriminators:

- *"If the ledger cannot accept a write, does the payment succeed?"* Both
  answers are defensible and must be argued. Fail closed (reject the
  payment) preserves the invariant that every payment is recorded, at the
  cost of availability and revenue. Fail open (accept, record later) needs a
  durable pending queue that is itself a system of record — at which point
  the queue *is* the ledger's write-ahead log and the design should say so.
- *"A payment succeeded at the card network but the ledger write failed."*
  This is the unavoidable case — money moved in the real world and your
  record doesn't have it. The answer must be a reconciliation-driven repair
  path with an idempotency key that makes replay safe, and a bounded time to
  detection. A design with no answer here has a permanent, silent money
  leak.

The **"entry written but counterpart not"** case tests whether double-entry
is structurally guaranteed (one append containing both legs) or
operationally hoped for (two writes).

---

### Part 7 — Tradeoffs ledger

**Staff:** 12+ rows for this problem. Must include a cost borne by finance
(e.g. the fee account balance is eventually consistent intraday) and one
borne by an auditor (e.g. an as-of query has a stated latency, or a
snapshot is a derived artifact they must trust the rebuild of).

---

### Part 8 — Evolution

**Staff:** The "someone will re-platform *your* system in nine years"
question is the best one in the section and a strong candidate takes it
seriously: an append-only entry log with a stable, self-describing schema
and no mutable state is the most re-platformable artifact you can leave;
derived projections and snapshots are disposable by construction. Saying
that explicitly — *the entries are the asset, everything else is a cache* —
is a Principal-level framing.

Multi-currency: entries must carry currency from day one even if there is
only one, and FX gains/losses are *ledger* events (they change the value of
a position), which is a common and expensive retrofit.

---

### Part 9 — Operations & cost

**Staff:** The 3am invariant-failure scenario is the sharpest question in the
worksheet. The correct shape of the answer: **do not attempt to fix the
data at 3am.** Stop the bleeding (halt the affected write path or the
payout run), preserve evidence, quantify exposure, and escalate to finance —
because a well-meaning engineer posting correcting entries under pressure is
how a recoverable discrepancy becomes an unauditable one. A candidate who
describes an engineer fixing balances at 3am has failed the question.

The auditor question — *"how do you know no entry has ever been lost?"* —
requires a real answer: sequence numbers with gap detection, a running
double-entry check, per-period control totals, and reconciliation against
external rails. "We use a durable database" is not evidence.

---

## Grading calibration

| Grade | Profile |
| --- | --- |
| **Below Senior** | Mutable balance rows. One timestamp. Sharding proposed without noticing the internal accounts. No idempotency. As-of queries treated as "add an index." |
| **Senior** | Append-only entries, balances as a projection, idempotency keys, sharding by account. Recognises the hot account and proposes sub-accounts. Handles component failures. Misses bitemporality, or treats as-of as a single-axis query. Migration is dual-write with row-count checks. |
| **Borderline Staff** | Gets bitemporality *or* the hot-account/single-shard reframe, not both. Snapshots present but their derivation/verification unaddressed. Reconciliation acknowledged but not designed. Invariants all synchronous, so the throughput doesn't close. |
| **Staff** | Bitemporal model with both readings derivable and the "does a past balance change" question answered correctly on both axes. Sub-accounts making every transaction single-shard. Snapshots with a computed cadence and a rebuild/verify job. Corrections-into-closed-periods policy stated. Idempotency reasoned against the 120-day horizon. Migration with continuous balance-level equivalence and a named sign-off. Catches 6+ first-tier ambiguities. Knows that raw throughput is not the binding constraint. |
| **Strong Staff / Principal** | All of the above, plus: recognises that the entry log is the durable asset and every projection is a disposable cache, and designs for that explicitly; distinguishes structurally-enforced invariants from accept-and-detect ones with a cost per violation; has a real answer to the "payment succeeded, ledger write failed" money-leak case with bounded detection time; keeps the break rate sub-linear in volume by treating break categories as bugs; and answers the 3am scenario by *not fixing the data*. |

---

## Review guidance

**Round 1:** Recompute Part 2 first — particularly the hot-row arithmetic
and the historical fold cost. The highest-value probes, in order:

1. *"A chargeback arrives today for a 90-day-old payment. What was the
   merchant's balance on the day of the original payment — before and after
   the chargeback arrived? Are both numbers available?"* This is the
   bitemporality probe and it is the single most informative question in the
   problem. Do not explain the answer.
2. *"Every transaction credits the platform fee account. Walk me through
   14,000 of those in one second."*
3. *"Your balance projection is 20 minutes behind and the payout run
   starts. What happens?"*
4. *"The card network says the payment succeeded. Your ledger has no
   record. How long until you know, and what happens then?"*
5. *"How does an auditor verify your snapshot is correct?"*

**Round 2+:** Commonly dodged: corrections into closed periods, the
payment-succeeded-ledger-failed case, and how equivalence is *proved* during
migration rather than merely monitored. Escalate those.

**Final:** The highest-leverage improvement is usually one of: "your entries
carry one timestamp and the Director's requirement needs two," "you sharded
by account and every transaction still touches the fee account, so every
transaction is distributed," or "your balances are authoritative state and
therefore your ledger is not auditable."
