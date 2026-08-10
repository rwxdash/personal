# Worksheet — 05 The Payment Ledger

Part 1 comes **before** any architecture.

Prose, tables, ASCII/mermaid diagrams. No code.

---

## Part 1 — Requirements & scope

### 1.1 Ambiguities in the brief

| # | Question | Why it changes the design | Your assumption |
| --- | --- | --- | --- |
| 1 | | | |

> At least 8. The second author added one requirement that constrains the
> data model more tightly than anything the first author wrote. If your list
> doesn't reflect that, read the thread again.

### 1.2 The two timelines

The Director's last paragraph asserts that a chargeback received today for a
90-day-old transaction is *both* a correction to a past position and an
event that happened today, and that both readings must be available and must
agree.

- Name the two timelines precisely.
- What does each one answer, and who asks?
- What has to be true of the data model for both to be derivable?
- What goes wrong if you store only one?
- Does a past balance ever *change*? Answer carefully — the wrong answer
  here invalidates most of a design.

### 1.3 Invariants

State the invariants the ledger must hold. For each: is it enforced at
write time, checked asynchronously, or merely reported on? What does a
violation cost, and how would you find out about one?

| Invariant | Enforcement | Cost of violation | Detection |
| --- | --- | --- | --- |
| | | | |

Include at least: double-entry balance, no lost entry, no duplicate entry,
and whatever you decide about negative balances.

### 1.4 Functional requirements

### 1.5 Non-functional requirements

| Property | Target | Source (stated / derived / assumed) |
| --- | --- | --- |
| Write latency (p99) | | |
| Write throughput (peak) | | |
| Balance read latency and staleness | | |
| Durability | | |
| Consistency of a balance read | | |
| Historical query — latency and range | | |
| Retention | | |
| Availability of the write path | | |

For **consistency of a balance read**, be precise. "Strongly consistent" is
not precise enough; say with respect to what.

### 1.6 Explicitly out of scope

At least four, each with what you assume someone else guarantees.

---

## Part 2 — Back-of-envelope

Show the arithmetic.

### 2.1 Write path

| Quantity | Calculation | Result |
| --- | --- | --- |
| Payment events/s, average and peak | | |
| Ledger entries/s, average and peak | | |
| Entries per year, and over 7 years | | |
| Bytes per entry (state your assumption) | | |
| Storage over 7 years, before and after compression | | |
| Sustained write rate of a single relational primary (state your reference figure) | | |
| **Factor over the naive single-node design** | | |

### 2.2 The hot account

| Quantity | Calculation | Result |
| --- | --- | --- |
| Updates/s to the platform fee account at peak | | |
| Sustained update rate for one serialised row | | |
| **Factor over** | | |
| If the ledger is sharded, what fraction of transactions touch >1 shard? | | |
| What does that imply about cross-shard coordination? | | |

State in one sentence why sharding by account does not, on its own, solve
this.

### 2.3 Read path

| Quantity | Calculation | Result |
| --- | --- | --- |
| Balance reads/s | | |
| Read : write ratio | | |
| Cost of computing a balance by folding history (state entries per account) | | |
| Cost of reading a materialised balance | | |
| Snapshot interval needed to bound historical-query cost | | |

### 2.4 The historical query

| Quantity | Calculation | Result |
| --- | --- | --- |
| Entries for a large merchant over 14 months | | |
| Fold cost with no snapshots | | |
| Fold cost with snapshots every N entries or every T | | |
| Storage cost of snapshots at your chosen interval | | |
| Target latency for an as-of query, and what it implies | | |

### 2.5 The payout run

| Quantity | Calculation | Result |
| --- | --- | --- |
| Payouts/s sustained over the window | | |
| Balance computations required | | |
| Entries written by the payout run | | |
| Peak multiple over normal traffic during the window | | |
| What the run contends with | | |

---

## Part 3 — API & data model

### 3.1 Core operations

Post a transaction, read a balance, read a balance as of an instant, list
entries, reverse/correct. Include idempotency semantics explicitly.

### 3.2 The entry model

Define the entry. What fields does it carry, and specifically: which
timestamps, and what does each mean? Which of them can a client set?

### 3.3 Entities

Account, entry, transaction, posting, balance, snapshot, correction. Say
which are immutable and which are not, and defend any that aren't.

### 3.4 Partitioning

- What is the partition key?
- What happens to the internal accounts that every transaction touches?
- Is a transaction atomic across partitions? If yes, how; if no, what does
  the caller see?
- How does the partitioning survive a merchant growing 100x?

---

## Part 4 — High-level architecture

Diagram plus one sentence per component. Show explicitly:

- The write path with the 80 ms budget annotated per hop.
- Where the authoritative entries live.
- How a current balance is served at 200k/s.
- How an as-of balance is served.
- Where corrections and reversals enter.
- The reconciliation path against external rails.

---

## Part 5 — Deep dives

**Pick two.** Say which and why.

**Candidate A — The hot account and cross-shard atomicity.**
Every transaction touches the platform fee and clearing accounts. Design it.
Options include sub-accounts with periodic rollup, per-shard accumulators,
a single-writer append design, deferring the internal leg, or 2PC. For each:
what does it do to the double-entry invariant, to atomicity, and to what a
finance user sees when they query the fee account mid-day? Include the
arithmetic.

**Candidate B — As-of queries and the two timelines.**
Design the data model and the query path that answers "what was this
merchant's balance on 14 March last year" in a bounded time, and that
answers it *both* ways — as known then, and as known now. What is snapshotted
and how often? What happens when a correction lands for a period that has
already been closed and reported? What does an auditor get, and what
guarantees does it carry?

**Candidate C — Reconciliation with external rails.**
The banks and networks disagree with you by design, and 200 breaks a day
reach a human. Design the reconciliation. What is matched against what, on
what key? What are the categories of break, and which can be auto-resolved?
What does a chargeback 120 days later do to your ledger, and to a period
already closed? How do you stop the break count growing with volume?

**Candidate D — Migration off the nine-year-old Postgres.**
Auditors trust it. Finance closes the books from it monthly. It cannot be
wrong for a moment, and it cannot be down for the close. Design the
migration: dual-write, CDC, shadow, or something else. How do you *prove*
equivalence to someone whose job is to distrust you? What is the rollback
after one month, after six? What is the point of no return, and who signs it?

---

## Part 6 — Failure modes

| Failure | Blast radius | How you detect it | Mitigation | Degraded behaviour |
| --- | --- | --- | --- | --- |
| A shard is unavailable mid-transaction | | | | |
| The write path is up but a partition splits the shards | | | | |
| A duplicate transaction is posted | | | | |
| An entry is written but its counterpart is not | | | | |
| The payout run double-pays a merchant | | | | |
| A correction is applied twice | | | | |
| Balance projection falls 20 minutes behind | | | | |
| Storage loses a block from six months ago | | | | |
| Your own deploy during the nightly payout | | | | |

Then answer: **if the ledger cannot accept a write, does the payment
succeed?** Justify. Then answer the inverse: what happens to a payment that
succeeded at the card network but whose ledger write failed?

---

## Part 7 — Tradeoffs ledger

| Decision | Chosen | Rejected | Cost accepted | Who feels it |
| --- | --- | --- | --- | --- |
| | | | | |

At least 12 rows for this problem. At least one where the cost is borne by
the finance team and one where it is borne by an auditor.

---

## Part 8 — Evolution

- What breaks first at 10x — component, limit, number.
- What breaks second.
- The migration story with rollback at each stage.
- What changes when the marketplace adds a second currency, and then
  twenty? Is FX a ledger concern?
- What changes if a regulator requires that EU merchants' ledger data reside
  in the EU?
- Nine years from now someone will want to re-platform *your* system. What
  did you build that makes that possible, and what did you build that makes
  it hard?

---

## Part 9 — Operations & cost

- Dominant cost driver, with the number, and the most effective lever.
- Top 3 alerts. Why each is a page.
- **The 3am page:** the double-entry invariant check has failed for 40
  transactions. Write what the on-call sees and what they do. Be specific
  about what they must *not* do.
- What does the finance team need at month-end close, and does your design
  provide it without an engineer being involved?
- An auditor asks how you know no entry has ever been lost. What is your
  answer, and what evidence backs it?
- What is the one operation you'd most want during an incident, and does
  your design allow it?
