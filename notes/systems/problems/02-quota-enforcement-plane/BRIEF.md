# 02 — The Quota Enforcement Plane

**Difficulty:** Staff
**Theme:** Global rate limiting and quota enforcement as a platform service
**Estimated effort:** 6–10 hours of writing, across several sessions

> **Do not start designing until you have completed Part 1 of
> [WORKSHEET.md](WORKSHEET.md).** The requirements in this brief pull in
> different directions. Working out which of them belong to the same system
> is most of the exercise.

---

## Context

*(Written as the design request would actually arrive.)*

---

**From:** Principal Engineer, API Platform
**To:** Staff Engineer, Edge
**Subject:** Consolidating rate limiting — design needed

We have forty-something rate limiters and I would like to have one.

Here is how we got here. Every service that ever got knocked over by a
customer built its own limiter. Some are token buckets in process memory,
some are Lua scripts against a shared Redis, two of them are nginx
`limit_req` zones, and the payments team has one backed by Postgres, which
is exactly as fast as you'd imagine. None of them agree on what a "minute"
is. A customer who hits three of our services gets three different limits
with three different reset behaviours and three different error formats, and
our docs are a work of fiction.

That's the tidiness argument, and on its own it wouldn't get funding. Here
is what got it funded. Last quarter we launched usage-based pricing. Plans
now include a monthly call allowance — 5M calls on Growth, 50M on Scale,
custom above that — and overage is billed. Finance discovered that the
numbers we bill from and the numbers we enforce on are computed by different
systems that disagree by up to 4%. On our largest accounts that is a
six-figure argument, and we have already given credits rather than have it.
So whatever you build has to be the thing that both enforces the limit and
produces the number on the invoice, or we will be having this conversation
again next year.

The constraints I know about:

- **9M requests/second at peak** across the edge fleet, ~4M average. 180
  PoPs, about 2,800 edge nodes. Growth plan is 2x in two years, which for
  once is not the hard part.
- A request typically triggers **about four limit checks** — per API key,
  per source IP, per endpoint class, and the account's plan quota. So call
  it 36M decisions/second at peak.
- **~1.2M active API keys** across ~90k accounts. The distribution is what
  you'd expect: our top 20 accounts are about half the traffic, and one
  account is 11% of it on its own.
- **Latency budget: p99 of 1 millisecond added at the edge.** This came from
  the CDN team and they are not joking about it. Our edge p50 for a cached
  response is under 10 ms and they will not accept a 10% regression on it.
- The plane sits in front of everything, so its availability has to be
  better than the things behind it. Today the aggregate is about 99.95%; I
  am being asked for 99.99% on the enforcement path.

Two things that make this harder than it sounds.

First, we do not only limit external customers. Internal service-to-service
calls need limiting too — that's where the noisy-neighbour incidents
actually come from — and internal traffic is another 6M rps that never
touches the edge fleet. I don't know yet whether that's the same system or a
different one. You tell me.

Second, and I want to be honest that I don't have a clean answer: some of
these limits exist to stop us falling over, and some of them exist because
a customer signed a contract. I have been treating them as the same kind of
object and I am increasingly sure that's wrong, but I don't know what it
costs to treat them differently.

Also — the January incident. Someone pushed a limit config with a missing
default and every unmatched key fell through to a limit of zero. Global,
about ninety seconds, everything. I would like the design to have an opinion
about that, because right now config reaches every PoP in under two seconds
and that is a feature until it isn't.

What I want: one plane, both use cases served, the latency budget held, and
a story for how a bad config doesn't become a global outage. Tell me what
you're deliberately not solving. Tell me what breaks at 10x.

---

## What to produce

Create `DESIGN.md` in this directory. It's yours; nobody else writes to it.

Work through [WORKSHEET.md](WORKSHEET.md) as you go — Part 1 **before** you
start designing, the rest alongside.

When a round is ready, say so and a review will land in `reviews/round-1.md`.
Expect 2–3 rounds. The final round is graded.
