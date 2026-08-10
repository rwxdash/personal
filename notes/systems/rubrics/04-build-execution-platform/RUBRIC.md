# Rubric — 04 The Build Execution Platform

> **Spoiler.** Do not open until the final review round is written.

---

## The central tension

**Isolation, utilisation, and latency form a triangle, and the budget only
buys two.**

- **Isolation** for untrusted fork code means a hardware-backed boundary —
  microVMs, or separate node pools, or separate accounts. That costs
  start-up latency and a per-job memory floor, and it forbids the
  overcommit and bin-packing that make shared infrastructure cheap.
- **Latency** (p95 queue under 30 s) cannot be met by provisioning, because
  node provisioning is 60–180 s. It requires **warm idle capacity**, which
  is by definition unutilised.
- **Utilisation** is forced by the budget. Average concurrency is ~33% of
  peak, and perfect elasticity on-demand costs ~$2.6M against a $2.5M
  target — meaning *even a perfectly elastic design with zero waste is over
  budget*. There is no room for warm pools, no room for isolation overhead,
  no room for error.

That last number is the trap, and it is deliberate. A candidate who does the
cost arithmetic discovers that the three requirements are jointly
unsatisfiable at their stated values, and the design becomes a negotiation
about **which one gives**. A candidate who doesn't do the arithmetic
produces a design that is quietly 2–3x over budget and doesn't know it.

The credible resolutions, any of which can earn a Staff grade if argued with
numbers:

1. **Relax the SLO for a class of jobs.** p95 < 30 s for interactive PR
   checks; nightly, release, and matrix jobs get a queue budget of minutes.
   This is by far the cheapest concession — most of the volume is not
   latency-sensitive — and it is the answer I'd expect from someone who has
   run a build platform.
2. **Buy spot.** 70% spot at typical discounts brings perfect-elasticity
   cost to well under target and creates the headroom for warm pools and
   isolation. The cost is interruption, which collides with the p99
   55-minute and 4-hour jobs. Requires a job-class-aware placement policy.
3. **Relax isolation for internal jobs** and pay for it only on fork PRs
   (which are a tiny fraction of volume). This is the highest-leverage move
   and most candidates find it; the strong version quantifies what fraction
   of jobs actually need the expensive boundary.
4. **Attack demand, not supply** — the cache is worth 60% of wall clock, so
   raising the hit rate reduces required capacity directly. A candidate who
   treats the cache as a *capacity* lever rather than a *speed* lever is
   thinking correctly.

**A design that keeps all three at their stated values and claims to hit
$2.5M is wrong, and the review should show the arithmetic.**

**Secondary tension:** the cache is simultaneously the largest cost lever
and the largest attack surface. Untrusted fork code that can write to a
shared cache can poison the builds of the payments service. Making the cache
safe (per-trust-domain, read-only for forks) reduces its hit rate and
therefore its value. Quantifying that trade is a strong signal.

---

## Ambiguities a strong candidate must catch

**Catching 6+ of the first tier is a Staff signal. Fewer than 4 caps at
Senior.**

### First tier — changes the architecture

| # | Ambiguity | Why it matters |
| --- | --- | --- |
| 1 | **Does the 30-second SLO apply to all job classes?** | The single cheapest concession available. Never stated. |
| 2 | **What isolation does a fork PR actually require, and how many jobs are forks?** | If forks are 2% of volume, the expensive boundary costs 2% of the fleet. If it's applied to everything, it costs everything. The brief never gives the fraction — a candidate should ask for it and reason about the answer either way. |
| 3 | **May fork PRs read the shared cache? Write to it?** | The brief says "I don't know what a shared cache means for the fork problem and I'd like you to tell me" — an explicit invitation. Read-only for forks is the obvious answer; the interesting part is whether *reading* leaks anything. |
| 4 | **Is spot/preemptible acceptable, and for which job classes?** | Decisive for the budget. Never stated. |
| 5 | **Do test jobs get secrets?** | "Nobody has audited this." A design that carries the current behaviour forward has recreated the vulnerability on new infrastructure. |
| 6 | **What is the fairness policy?** | FIFO, per-team quota, priority classes. Never stated, and it determines what happens during the spike, which is the whole problem. |
| 7 | **Is the $2.5M a hard ceiling or a target?** | Whether the negotiation in 1.2 is even permitted. |
| 8 | **Does a fork job get network egress?** | Arbitrary code with unrestricted egress is exfiltration and cryptomining. It's also required by most builds (package registries). Resolving this is real design work, not a checkbox. |

### Second tier — changes sizing or operations

| # | Ambiguity |
| --- | --- |
| 9 | Are job resource declarations trusted? What happens when a job declares 4 vCPU and uses 16? |
| 10 | Can jobs be preempted and retried, or is a killed job a failed build? |
| 11 | Log and artifact retention — volume and cost are non-trivial at 380k jobs/day. |
| 12 | Is the platform expected to run non-CI workloads later? (Part 8 asks; the answer changes the abstraction.) |
| 13 | Who owns a job stuck for 12 hours — is there a hard timeout, and per class? |
| 14 | What is the availability requirement? CI being down for an hour is expensive but not customer-facing; nobody states a number. |
| 15 | Are the nine public repos allowed to be on separate infrastructure entirely? |
| 16 | Does "faster builds" have a target, or is it just "not slower"? |

---

## Section-by-section

### Part 1 — Requirements & scope

**Senior:** Restates requirements. Notes that untrusted code needs
isolation. May not identify the conflict.

**Staff:** Section 1.2 identifies the three-way conflict with arithmetic and
proposes a specific concession with a named counterparty who must agree.
Section 1.3 enumerates trust classes properly — at minimum: fork PR
(untrusted), internal PR (semi-trusted, runs code from an employee), release
and deploy jobs (trusted, holds credentials), and possibly a fourth for
regulated builds.

**Red flags:** Accepting all three constraints without checking whether they
compose. A trust model with two classes ("trusted" and "untrusted") that
ignores that an internal PR also runs code an engineer wrote and that
engineers get compromised.

---

### Part 2 — Back-of-envelope

Recompute independently. This is the most important arithmetic section in
any of the problems in this repo, because the conflict is only visible
through it.

| Quantity | Defensible value | Working |
| --- | --- | --- |
| Mean job duration | **~7 min (420 s)** | p50 4 min with a p99 of 55 min and a 4 h tail pulls the mean well above the median. Anything in 6–10 min is defensible if the reasoning is stated; using the p50 as the mean is a **~1.75x under-sizing error** and should be challenged |
| Average concurrent jobs | **~1,850** | 380,000 × 420 s ÷ 86,400 s |
| Peak concurrent | **~5,500** | 3x trough-to-peak |
| Peak vCPU | **~22,000** | 5,500 × 4 |
| Peak RAM | **~44 TB** | 5,500 × 8 GB |
| Average / peak | **~33%** | The utilisation ceiling for a perfectly elastic design |
| Provisioned for peak, on-demand | **~$7.8M/yr** | 22,000 vCPU × $0.04/vCPU-hr × 8,760. Rate is an order-of-magnitude reference |
| **Perfect elasticity, on-demand** | **~$2.6M/yr** | 1,850 × 4 × $0.04 × 8,760. **Already over the $2.5M target** |
| Perfect elasticity, 70% spot | **~$1.2–1.6M/yr** | At a 60–70% spot discount. This is the headroom that makes the design possible |
| Node provisioning time | **60–180 s** | Order of magnitude. **2–6x the queue SLO** |
| Container start on a warm node | **~1–5 s** | |
| microVM start on a warm node | **~0.1–1 s** boot, plus image/rootfs setup | Firecracker-class boot is fast; the total job-ready time is dominated by image pull and workspace setup, not the VM |
| Cache value | **~$1.0–1.5M/yr of compute** | 60% of wall clock on the cacheable majority of jobs — the cache is worth more than the gap between budget and spend, which is the point |

**Senior:** Computes capacity. May use p50 as the mean. Doesn't reach the
"perfect elasticity is already over budget" conclusion.

**Staff:** Derives a mean that respects the long tail and says so. Reaches
the **perfect-elasticity-exceeds-budget** conclusion explicitly and treats
it as the finding that drives the design. Quantifies the warm capacity
needed to bridge the provisioning gap. Prices spot interruption against
spot savings rather than assuming spot is free money.

**Strong signal:** noticing that the cache is worth roughly the size of the
budget gap, so **improving cache hit rate is a capacity strategy**, and that
this reframes the cache from a performance feature into the primary cost
lever.

**Red flags:** Using p50 as mean. Not computing the perfect-elasticity
number. Assuming spot with no interruption analysis. No cache valuation.

---

### Part 3 — API & data model

**Senior:** Job submission API, a queue, runners poll it.

**Staff:** Treats the **execution environment** (image + resources + trust
class + network policy + cache scope + secret scope) as a first-class
entity, because that bundle is what the scheduler places and what the
isolation policy attaches to. Handles untrusted resource declarations —
a job that declares 4 vCPU and uses 16 is either throttled (hurting the
honest case) or overcommitted (hurting the neighbour), and the answer
interacts with the isolation model.

**Red flags:** No notion of trust class in the data model, which means it
can't be a scheduling input.

---

### Part 4 — High-level architecture

**Senior:** Event → queue → scheduler → runner pool → results. Autoscaler
attached.

**Staff:** Shows **multiple pools with different properties** — at minimum a
trusted pool and an untrusted pool, likely also a spot pool and an
on-demand pool for long jobs. The 30-second budget is annotated across
webhook → queue → schedule → node → image pull → workspace setup → first
command, and the candidate has noticed that **image pull is often the
dominant term** and designed for it (pre-pulled images on warm nodes, lazy
image loading, or a local registry mirror).

**Red flags:** One homogeneous pool. No annotation of where the 30 seconds
goes — this is the SLO they were given, and a design that doesn't budget it
per hop can't defend it.

---

### Part 5 — Deep dives

**A (isolation)** and **B (scheduling)** are the hardest. **C (cache)** is
where the money is. A+B, A+C, and B+C are all strong. D alone is the
political problem and is the weakest pairing partner.

#### A — Isolation

**Staff:** Knows that a container is not a security boundary against
hostile code (interviews 08·A11) and reaches for a hardware-backed one for
forks: **microVMs** (Firecracker/Kata) or dedicated node pools, ideally in a
**separate cloud account** so an escape doesn't reach the same IAM, VPC, or
metadata service as the trusted fleet.

Must address, because these are where real escapes come from:
- **Metadata service access** — a fork job that can reach the node's
  instance metadata gets the node's IAM role. IMDSv2 with hop limits, or no
  instance role at all on the untrusted pool.
- **Network egress** — arbitrary code needs package registries and wants
  nothing else. An egress allowlist through a proxy, and a statement about
  cryptomining detection and its cost.
- **Secrets** — fork jobs get none, structurally (not by convention), which
  means the pipeline definition itself must not be able to request them
  when triggered by a fork.
- **The cache** — read-only at most, and see below.
- **Persistence between jobs** — a fresh rootfs per job, and no shared
  writable volumes.

The strong move is quantifying the cost: if fork PRs are a small percentage
of jobs, the expensive isolation applies to a small percentage of the fleet,
and the utilisation penalty is bounded. A candidate who applies microVMs to
all 380k jobs/day has spent the entire spot saving on isolation nobody
needed.

**Red flags:** "We'll use containers with a restricted security context" for
hostile code. No mention of the metadata service. Secrets excluded by
convention rather than by construction.

#### B — Scheduling under the spike

**Staff:** Recognises that provisioning (60–180 s) exceeds the SLO (30 s),
so the SLO can only be met by capacity that already exists. Then designs the
warm capacity economically:
- **Predictive scaling** on a known-in-advance pattern. The spikes are three
  time-zone mornings — this is the most predictable load curve in
  infrastructure, and scaling *ahead* of it on a schedule costs far less
  than reactive warm pools. Candidates who exploit the predictability are
  thinking correctly.
- **Warm pool** sized to the ramp rate, not the peak — you need to absorb
  the arrival rate during the minutes it takes to provision, not the whole
  spike.
- **Preemptible filler work**: run the nightly/batch/low-priority jobs on
  the warm capacity so it isn't idle, and evict them when interactive jobs
  arrive. This is the elegant answer — it converts idle warm capacity into
  useful work at near-zero marginal cost, and it *requires* the job-class
  distinction from 1.2.
- **Fairness**: per-team concurrency quotas or a weighted fair queue so one
  team's 2,000-job matrix can't monopolise. Must be addressed — the brief's
  spike is exactly when this bites.
- **Long jobs on stable capacity**: a 4-hour job must not be placed on a
  node scheduled for scale-in, which means the scheduler needs job-duration
  estimates and node-lifetime awareness. Bin-packing long jobs onto
  on-demand and short jobs onto spot is the standard answer.

**Red flags:** Cluster autoscaler alone. No fairness mechanism. No handling
of long jobs vs scale-in — a design that reclaims a node holding a 3-hour
release build has a serious operational defect.

#### C — The cache

**Staff:** Separates **cache scopes by trust domain**. The workable model:
- Trusted builds (main branch, internal PRs) **write** to a shared cache.
- Fork PRs **read** from a snapshot and write only to an ephemeral,
  per-run scope that is discarded.
- Cache keys are **content-addressed** (hash of dependency lockfile,
  toolchain version, and inputs) so an entry cannot be silently substituted,
  and a poisoned entry can't be produced by choosing a key.

That last point is the crux: content-addressing plus write restriction means
a fork cannot poison, and the read path still gives forks most of the
benefit. Quantifying the residual hit-rate loss for forks is a strong
signal.

Must also address: eviction policy and its cost (a miss is 60% more wall
clock, so eviction is a *cost* decision, not a storage decision), egress
charges if the cache is cross-AZ (interviews 04·A13 — at this volume it is a
real line item), and what a cache-wide invalidation costs (a toolchain
upgrade invalidating everything produces a synchronised miss storm — the
capacity spike must be planned for).

**Red flags:** One shared writable cache for all trust levels. No
content-addressing. Treating the cache as free.

#### D — Migration

**Staff:** Orders the migration by *risk and blocker type*, not by team
size. Runs new and old in parallel with result comparison before cutting
over. Categorises the dozen blocked teams honestly:
- **Real and solvable**: a licensed toolchain → a dedicated node pool with
  the license attached; a plugin → find or build the equivalent.
- **Real and not worth solving**: the physical device on a desk in Berlin →
  a self-hosted runner registered to the platform, so it gets the platform's
  scheduling and observability while staying physically where it is. This is
  the right answer and it's a good discriminator — the platform should
  support *bring your own capacity* rather than demanding everything move.
- **Preference, not blocker**: a deadline and a migration guide.

**Red flags:** A big-bang migration. No equivalence-verification step. No
answer on the physical device.

---

### Part 6 — Failure modes

**Staff:** The **spot reclamation** case is the one that tests whether the
budget strategy is real — losing 20% of the fleet in two minutes must be
survivable, which means checkpointing or fast retry for interrupted jobs and
a non-spot floor for jobs that can't tolerate it.

The **sandbox escape** case must have a *response*, not just a prevention:
what is contained, what is rotated, what is rebuilt. If the untrusted pool
is in a separate account with no shared credentials, the answer is short and
credible; if it isn't, the answer is "everything."

The **poisoned cache** case tests whether content-addressing was actually
designed in.

**"Control plane down but jobs running"** should conclude that running jobs
complete — the data plane surviving the control plane is the same property
as interviews 08·A9, and it's a design goal worth stating.

---

### Part 7 — Tradeoffs ledger

**Staff:** Must include a row where developers pay (a relaxed SLO for batch
jobs, or slower fork builds) and one where security pays (containers rather
than microVMs for internal jobs, or egress allowed to a broad allowlist).
A ledger where every cost falls on "the platform team" is not honest.

---

### Part 8 — Evolution

**Staff:** The regulator question is the interesting one — it forces a
physically separate execution environment for payments builds, which the
multi-pool design should already accommodate. A candidate whose design can
absorb that with a policy change rather than a redesign has built the right
abstraction, and should say so.

The "it will be asked to run non-CI workloads" question is a genuine
strategy question: a generic job-execution platform is a much larger product
than a CI runner, and the honest Staff answer is usually "we'll say no
until the abstraction has earned it," with the reason.

---

### Part 9 — Operations & cost

**Staff:** The 3am question is a good honesty test. CI is not
customer-facing; most of these failures should be *business-hours* problems,
and a mature answer says so and names the small set that genuinely pages
(total platform outage during a release window, a security event, a runaway
cost event). A candidate who pages on everything is describing a rotation
nobody will survive (interviews 09·A10).

Chargeback deserves real thought: charging per job-minute makes teams cache
better and parallelise less; charging per job makes them batch; charging
nothing makes them do neither. The behaviour the model creates is the point.

---

## Grading calibration

| Grade | Profile |
| --- | --- |
| **Below Senior** | Kubernetes with an autoscaler and one pool. Containers for untrusted code. No cost model. Doesn't notice the queue SLO is faster than provisioning. |
| **Senior** | Correct multi-pool architecture with microVMs for forks. Computes capacity. Uses spot. Warm pool for the spike. Handles component failures. Doesn't discover that the three requirements conflict, and the cost model doesn't close. |
| **Borderline Staff** | Discovers the cost squeeze *or* designs the isolation and scheduling well, but not both. Proposes a concession without quantifying it. One deep dive strong. Cache treated as a performance feature only. |
| **Staff** | Identifies the three-way conflict with arithmetic and negotiates a specific concession. Trust classes enumerated with per-class isolation and cost. Predictive scaling on a known pattern plus preemptible filler on warm capacity. Cache scoped by trust domain with content-addressing. Spot interruption priced against long jobs. Fairness mechanism. Migration ordered by blocker type with parallel verification. Catches 6+ first-tier ambiguities. |
| **Strong Staff** | All of the above, plus: reframes the cache as the primary *capacity* lever and shows it is worth roughly the budget gap; uses preemptible batch work to make warm capacity nearly free; puts the untrusted pool in a separate account so escape response is bounded and says what that response is; supports bring-your-own-runner for the physically-blocked team rather than forcing a migration; is honest that most CI failures are not 3am pages; and designs a chargeback model with the behaviour it creates stated explicitly. |

---

## Review guidance

**Round 1:** Recompute Part 2 before anything else, especially the mean job
duration and the perfect-elasticity cost. The highest-value probes: "what
does perfect elasticity cost, and how does that compare to your target?",
"how many of the 380k daily jobs actually need a microVM?", "your queue SLO
is 30 seconds and provisioning takes two minutes — where does the capacity
come from?", "a fork PR reads the shared cache; what does it learn?", and
"a 4-hour release build is on a spot node — what happens?" Do not reveal the
conflict; ask for the cost number and let them find it.

**Round 2+:** Commonly dodged: the fairness mechanism during the spike,
what happens to long jobs on reclaimed capacity, and the escape *response*
as opposed to escape prevention. Escalate those.

**Final:** The highest-leverage improvement is usually one of: "your design
is over budget and you didn't compute it," "you applied fork-grade isolation
to jobs that don't need it and spent your entire spot saving on it," or "the
cache is worth more than the gap you're trying to close and you treated it
as a feature rather than a capacity strategy."
