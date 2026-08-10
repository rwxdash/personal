# 05 — The Payment Ledger

**Difficulty:** Strong Staff / Principal
**Theme:** Correctness, auditability, and throughput in a system of financial record
**Estimated effort:** 10–14 hours of writing, across several sessions

> **Do not start designing until you have completed Part 1 of
> [WORKSHEET.md](WORKSHEET.md).** This is a thread with two authors. Read
> both messages before you decide what the system has to do.

---

## Context

*(Written as the design request would actually arrive — with a second author
who was added to the thread and changed its meaning.)*

---

**From:** Head of Payments Engineering
**To:** Staff Engineer, Money Movement
**Cc:** Director of Financial Controls
**Subject:** Ledger — design for the re-platform

You've inherited the ledger re-platform. Here is where we are and what I
need from you.

**What we run today.** A single Postgres, one `transactions` table and one
`balances` table, updated in the same database transaction. It has been
extended for nine years. It is the system of record for every unit of money
that moves through the marketplace, and it is the thing the auditors look
at. It is at about 60% of the write capacity of the largest instance our
provider offers, and we have already had two incidents where the
`platform_fees` balance row was the bottleneck for every payment in the
system, because every payment touches it.

**Volume.** About **86M payment events per day**, so roughly **1,000 per
second average**. Peak is around **14,000 per second** during the first
minutes of a large sale event, and marketing has three of those scheduled
next year. Each payment produces on the order of **7 ledger entries** —
debit the buyer, credit the merchant, credit the platform fee, credit tax,
and a couple of clearing entries — so call it 7k entries/second average and
~100k/second at peak.

**Accounts.** ~36M buyer accounts, ~900k active merchants, plus a few
hundred internal accounts. Those internal accounts are the problem: the
platform fee account and the clearing account are touched by **100% of
transactions**.

**Payouts.** Every night we pay out to merchants. Roughly 900k payouts in a
three-hour window, and each one needs a merchant's balance as of a specific
cut-off, plus a transfer out to a bank. Today this contends with normal
traffic and we have moved it three times to find a window where it hurts
least.

**The external world.** Card networks, ACH, and three wallet providers.
They are asynchronous, they settle on their own schedule, and they reverse
things — chargebacks arrive up to 120 days after the original payment. Our
ledger and the banks' records disagree at any given moment, by design, and
the reconciliation team runs a daily process to explain the difference.
That process currently finds around **200 breaks a day** that a human has to
look at.

**Latency.** A payment authorisation has a 400 ms budget end to end and the
ledger write is allotted 80 ms of it. Merchant balance reads are about
**200k per second** — merchants poll their dashboards constantly — and those
can be a little stale, though nobody has told me how stale.

---

**From:** Director of Financial Controls *(added to thread)*

Adding myself here because I want one requirement stated explicitly before
anyone starts drawing boxes.

We are required to be able to produce the balance of any account **as of any
historical instant**, for seven years, and to explain every entry that
contributed to it. In practice this comes up in three ways: audit sampling,
a regulator asking about a specific merchant, and litigation. Last year we
were asked for a merchant's position as of a date fourteen months prior and
it took the team eleven days to produce a number we were willing to sign.

I would also like to be clear that a chargeback we receive today, for a
transaction from ninety days ago, is not a new event that happens today. It
is a correction to a position that existed then. How you represent that is
your business, but both readings have to be available and they must not
disagree with each other.

---

**From:** Head of Payments Engineering

So — that. Plus: 80 ms, 14k/s peak, the hot internal accounts, the nightly
payout window, and a migration off a nine-year-old Postgres that the
auditors already trust and that finance closes the books from every month.

Tell me what you're explicitly not solving. Tell me what breaks at 10x.

---

## What to produce

Create `DESIGN.md` in this directory. It's yours; nobody else writes to it.

Work through [WORKSHEET.md](WORKSHEET.md) as you go — Part 1 **before** you
start designing, the rest alongside.

When a round is ready, say so and a review will land in `reviews/round-1.md`.
Expect 2–3 rounds. The final round is graded.
