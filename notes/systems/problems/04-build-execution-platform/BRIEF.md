# 04 — The Build Execution Platform

**Difficulty:** Staff+
**Theme:** Multi-tenant compute for untrusted workloads under a hard cost ceiling
**Estimated effort:** 8–12 hours of writing, across several sessions

> **Do not start designing until you have completed Part 1 of
> [WORKSHEET.md](WORKSHEET.md).** Some of the stated requirements may not
> compose at the values given. Whether you find that before or after you
> draw the architecture determines how much of your design survives.

---

## Context

*(Written as the design request would actually arrive.)*

---

**From:** Director, Developer Productivity
**To:** Staff Engineer, Platform
**Subject:** Build platform — consolidation design

I have approval to consolidate CI onto one platform and I need a design
before I can spend any of it.

Where we are: about forty team-owned Jenkins installations, a couple of
GitLab runner fleets, and two teams on a hosted SaaS runner that we're
paying list price for. Every one of them is a pet. Total spend across all of
it is **$4.2M/year** and my mandate is to land at **$2.5M** while making
builds *faster*, which I recognise is the kind of sentence that gets
designs handed back to me.

The numbers I have:

- **1,400 engineers, ~900 repositories, ~380,000 jobs per day.**
- Job duration: **p50 about 4 minutes, p99 about 55 minutes**, and a long
  tail of nightly and release jobs that run up to 4 hours.
- Load is savagely uneven. We have engineering in Berlin, New York, and
  Bangalore, and each of them produces a morning spike. About two-thirds of
  the day's jobs land in three windows. Trough to peak is roughly **3x**.
- **Queue time SLO: p95 under 30 seconds.** This is the number developers
  actually care about and the one that made the current setup untenable —
  the Jenkins fleets are sized for average, so the morning spike puts people
  in a 6-minute queue and they context-switch and lose the hour.
- A typical job asks for **4 vCPU and 8 GB**. Some ask for a lot more.

Some things you'll need to know that aren't in the spreadsheet.

**We have public repositories.** Nine of them, and two are genuinely popular.
Pull requests from forks run CI. That means we execute arbitrary code
submitted by strangers on our infrastructure, on the same platform that
builds our payments service. Security have been polite about this so far and
I don't expect that to last.

**Caching is most of the speed.** Our internal measurement is that a warm
dependency and build cache saves roughly **60% of wall-clock time** on a
typical job. It's also the thing teams have hand-tuned most and are most
attached to. I don't know what a shared cache means for the fork problem and
I'd like you to tell me.

**Secrets.** Deploy jobs need credentials. Test jobs mostly don't but
plenty of them have them anyway because that's how the pipelines were
written. Nobody has audited this.

**The migration is the political part.** Every team believes their Jenkins
is special, and about a dozen of them are right — there are jobs that depend
on a specific plugin, a machine with a licensed toolchain attached, or in one
case a physical device on a desk in Berlin. I can force a migration but I
can only do it once, so I'd rather the design tell me which teams move last
and why.

What I want: one platform, the queue-time SLO held through the morning
spike, fork PRs that can't reach anything they shouldn't, and a credible
path from $4.2M to $2.5M. Tell me what you're explicitly not solving. Tell
me what breaks at 10x.

---

## What to produce

Create `DESIGN.md` in this directory. It's yours; nobody else writes to it.

Work through [WORKSHEET.md](WORKSHEET.md) as you go — Part 1 **before** you
start designing, the rest alongside.

When a round is ready, say so and a review will land in `reviews/round-1.md`.
Expect 2–3 rounds. The final round is graded.
