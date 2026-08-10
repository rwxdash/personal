# Model Discussion — 04 The Build Execution Platform

> **Spoiler.** Do not open until the final review round is written.

One credible design, why its main alternative loses, and where competent
Staff engineers would still disagree.

---

## 1. The arithmetic that reframes the request

Mean job duration is **not** the p50. With p50 = 4 min, p99 = 55 min, and a
4-hour tail, the mean lands around **7 minutes**. Using the p50 under-sizes
the fleet by ~1.75x, and it is the most common error in this problem.

```
average concurrent = 380,000 × 420 s ÷ 86,400 s  ≈  1,850 jobs
peak concurrent    = 1,850 × 3                   ≈  5,500 jobs
peak vCPU          = 5,500 × 4                   ≈  22,000 vCPU
average / peak                                   ≈  33%
```

Now cost it (at an order-of-magnitude $0.04/vCPU-hour on-demand):

| Scenario | $/year |
| --- | --- |
| Provisioned for peak, 24/7 | **~$7.8M** |
| **Perfect elasticity, on-demand** | **~$2.6M** |
| Target | **$2.5M** |

**A perfectly elastic design with zero waste is already over budget.**

That single line reframes the entire request. It means there is no budget
for warm pools, none for isolation overhead, none for retry waste — unless
something else pays for them. Three requirements — 30-second queue SLO,
hardware isolation, $2.5M — cannot all hold at their stated values.

Two things pay for them:

**Spot.** At a 60–70% discount on ~70% of the fleet, elastic cost drops to
roughly **$1.2–1.6M/year**, creating ~$1M of headroom. That headroom buys
the warm capacity and the isolation. Spot is not an optimisation here; it is
load-bearing.

**The cache.** 60% of wall clock on cacheable jobs is worth on the order of
**$1.0–1.5M/year of compute**. The cache is worth roughly the size of the
budget gap. It is not a performance feature that happens to save money — it
is the primary capacity strategy, and it should be resourced like one.

---

## 2. The concession I'd ask for

**Split the queue-time SLO by job class.**

| Class | Share of jobs | Queue SLO | Capacity | Interruptible |
| --- | --- | --- | --- | --- |
| Interactive (PR checks, pushes) | ~70% | **p95 < 30 s** | warm + on-demand | no |
| Scheduled (nightly, matrix, scans) | ~25% | **p95 < 10 min** | spot, preemptible | yes |
| Release / deploy | ~5% | p95 < 60 s | on-demand, stable nodes | no |

This is the cheapest concession available and it costs almost nobody
anything real — nobody is waiting on a nightly job at 02:00. It is also the
enabler for everything else, because it creates a large pool of work that
is *allowed* to be evicted, and that work is what makes warm capacity
nearly free (§3.2).

The counterparty is the Director; the ask is one sentence: *"the 30-second
number applies to jobs a human is waiting on, and I'd like the other 30% of
volume to have a minutes-scale budget so I can use it as ballast."*

---

## 3. The design

```
 git event
    │
    ▼
┌────────────────────────────────────────────────────────────────┐
│ CONTROL PLANE                                                   │
│  · classify job: trust class × latency class                    │
│  · per-team fair queue (weighted, with concurrency quotas)      │
│  · placement: trust class → pool, duration estimate → node type │
└──────┬──────────────────────┬───────────────────┬──────────────┘
       │                      │                   │
       ▼                      ▼                   ▼
┌──────────────┐   ┌────────────────────┐  ┌──────────────────────┐
│ TRUSTED POOL │   │ UNTRUSTED POOL     │  │ STABLE POOL          │
│ (own account)│   │ (SEPARATE ACCOUNT) │  │ (on-demand)          │
│              │   │                    │  │                      │
│ containers,  │   │ microVM per job    │  │ long jobs (>30 min)  │
│ warm nodes,  │   │ no instance role   │  │ release/deploy       │
│ ~70% spot    │   │ egress allowlist   │  │ never reclaimed      │
│              │   │ no secrets, ever   │  │                      │
│ interactive  │   │ cache: READ-only   │  │                      │
│ + preemptible│   │ snapshot           │  │                      │
│ ballast      │   │                    │  │                      │
└──────┬───────┘   └─────────┬──────────┘  └──────────┬───────────┘
       │                     │                        │
       └─────────────────────┴────────────────────────┘
                             │
                  ┌──────────▼───────────────────────────┐
                  │ CACHE TIER (content-addressed)        │
                  │  · shared RW  ← trusted builds only   │
                  │  · snapshot RO → untrusted            │
                  │  · per-run ephemeral scope → untrusted│
                  │    writes (discarded)                 │
                  │  · AZ-local replicas (egress cost)    │
                  └───────────────────────────────────────┘
```

### 3.1 Isolation, applied where it's needed and nowhere else

Fork PRs are nine repos out of 900. Even if those repos are busy, fork jobs
are a small single-digit percentage of 380k/day. **The expensive boundary
therefore applies to a small percentage of the fleet**, and the utilisation
penalty is bounded to that slice. A design that runs every job in a microVM
has spent the entire spot saving on isolation that 95%+ of jobs didn't need.

For the untrusted pool, the boundary is layered and the *account* boundary
is the one that matters most:

- **Separate cloud account.** An escape reaches an account containing
  nothing but ephemeral build nodes — no shared IAM, no VPC peering to
  production, no artifact registry write access. This bounds the incident
  response to "rebuild that account," which is the difference between a bad
  week and a company-wide credential rotation.
- **microVM per job** (Firecracker/Kata class). Boot is fast enough to be
  irrelevant next to image pull; the real cost is the per-VM memory floor
  and the loss of overcommit.
- **No instance role**, IMDSv2 with a hop limit of 1. The metadata service
  is how most container escapes become cloud compromises, and it's the first
  thing to close.
- **Egress through an allowlist proxy** — package registries and nothing
  else. This is also the cryptomining control, and it is worth stating that
  it will break some legitimate builds and needs a request path.
- **Secrets are structurally unavailable**, not conventionally. The pipeline
  definition of a fork-triggered run cannot request a secret; the control
  plane refuses to mint one. Convention fails the first time someone copies
  a pipeline file.
- Fresh rootfs per job; no shared writable state.

Internal PRs get containers on shared nodes. That is a deliberate acceptance:
an engineer's code is not hostile, and treating it as hostile costs more than
the risk. It's a row in the tradeoffs ledger, and security is the party who
pays for it.

### 3.2 Warm capacity that isn't idle

Provisioning is 60–180 s; the SLO is 30 s. So the capacity must already
exist. The trick is that it doesn't have to be *idle*.

- **Predictive scale-up on a schedule.** Three time-zone mornings is the
  most predictable load curve in infrastructure. Scale ahead of Berlin, New
  York, and Bangalore by 15 minutes. Reactive autoscaling is the fallback,
  not the plan.
- **Preemptible ballast.** The warm pool runs *scheduled-class* jobs, which
  are interruptible by definition. When interactive jobs arrive, ballast is
  evicted and requeued. The warm capacity is therefore ~100% utilised while
  still being instantly available — which is what makes the 30-second SLO
  affordable inside the budget. This only works because of the concession in
  §2, which is why the concession comes first.
- Warm pool is sized to the **arrival rate during a provisioning window**,
  not to the peak: if the spike ramps at 40 jobs/second and provisioning
  takes 120 s, you need ~4,800 job-slots of buffer, not 5,500 jobs' worth of
  fleet.

### 3.3 Long jobs and spot

The p99 is 55 minutes and the tail is 4 hours. Spot interruption notices are
short. Placing a 4-hour release build on spot and losing it at hour three
wastes more compute than the spot discount saved, and it produces exactly
the developer experience the project exists to fix.

So: **duration-aware placement.** Jobs with an estimated duration above a
threshold (from historical p50 for that job definition) go to the stable
on-demand pool. Short jobs go to spot. The estimate is wrong sometimes, so a
job that exceeds its estimate on spot gets migrated or is allowed to finish
with a "do not reclaim" hint where the provider supports it.

Interrupted scheduled-class jobs are simply requeued — they were declared
interruptible, and their SLO has room for a retry.

### 3.4 The cache: content-addressed, scoped by trust

- **Keys are content-addressed** — a hash of lockfiles, toolchain versions,
  and relevant inputs. An entry cannot be produced by *choosing* a key, so a
  fork cannot place malicious content where a trusted build will look for
  it. This is the property that makes the whole scheme safe; without it,
  scope restrictions are a speed bump.
- **Trusted builds write** to the shared cache. **Fork jobs read** a
  read-only snapshot and write to an ephemeral per-run scope that is
  discarded. Forks keep most of the benefit; they cannot poison.
- What a fork *learns* from reading is worth stating: dependency sets and
  build artifacts of public repos, which are already public. Fork jobs must
  not read caches belonging to private repos, which falls out of scoping the
  snapshot per repo.
- **AZ-local replicas.** At this volume, cross-AZ cache egress is a real
  line item (interviews 04·A13), and a cache that saves compute while
  spending it back on transfer is a bad trade.
- **Eviction is a cost decision.** A miss costs ~60% more wall clock, so the
  cache should be sized by the value of the hits, not by a storage budget.
- **Invalidation storms** are a planned capacity event: a toolchain upgrade
  invalidates broadly and produces a synchronised miss spike. Roll such
  changes progressively, like a deploy.

### 3.5 Fairness

Per-team weighted fair queueing with concurrency quotas. Without it, one
team's 2,000-job matrix build during the Berlin spike consumes the warm pool
and everyone else waits — which is the exact failure the platform is
supposed to eliminate, reproduced centrally. Quotas are soft (a team can
exceed them when the fleet is idle) and bind only under contention, which is
the same shape as CPU shares (interviews 05·A8).

---

## 4. Why the main alternative loses

**The alternative: one homogeneous pool, microVMs for every job, sized for
peak on on-demand capacity.** It is simple, uniformly secure, has no trust
classification to get wrong, and no job-class negotiation.

It loses on cost, decisively: **~$7.8M/year against a $2.5M target**, and
that is before the microVM memory floor reduces packing density further. It
is 3x over, and no amount of tuning closes a 3x gap.

Even the elastic version of it — one uniform microVM pool, perfectly
autoscaled — is ~$2.6M before isolation overhead and before any warm
capacity, so it fails the budget *and* the queue SLO simultaneously.

**Where the alternative genuinely wins**, and it is worth conceding: it is
enormously simpler to operate and to reason about. There is one security
story, not four. There is no trust classifier to be wrong, no job-class
policy to be gamed, no duration estimator to mis-predict. If the budget were
$8M, it would be the right design and the multi-pool version would be
premature optimisation. The complexity here is bought entirely by the cost
ceiling, and that should be said out loud — including to the Director, who
should understand that the $1.7M saving is being paid for in operational
complexity that his team will carry.

---

## 5. Where reasonable Staff engineers would disagree

**5.1 Containers for internal PRs.** I've accepted that an employee's code
is not hostile. A security engineer would point out that a compromised
laptop, a malicious insider, or a supply-chain attack in a dependency all
execute as "internal," and that the payments build sharing a kernel with
another team's test suite is a real risk. The counter-argument is cost and
the fact that the same engineer can push to production anyway. This is a
genuine disagreement and the answer depends on the threat model the company
has actually adopted, not on engineering.

**5.2 Buying instead of building.** Hosted CI at 380k jobs/day would cost
far more than $2.5M at list, which is presumably why the Director isn't
proposing it — but with a negotiated enterprise agreement it might not, and
it would eliminate a platform team. A Staff engineer should price it before
designing, if only to be able to say why not. My view: at 1,400 engineers
the control and cost case for self-hosting is real, but "we never asked" is
not an acceptable answer to a director.

**5.3 Whether spot is acceptable at all.** Making 70% of the fleet
interruptible means accepting that builds occasionally die for reasons the
developer didn't cause, which is corrosive to trust in the platform. A
reasonable alternative is heavy use of committed-use discounts on a stable
fleet (30–50% off, no interruption) plus a smaller spot tier — less saving,
better experience. If the developer-trust cost is high enough, that's the
better trade, and it's a judgement call, not a calculation.

**5.4 Whether the cache should be shared at all.** Per-team caches
eliminate the entire poisoning question and most of the scoping complexity,
at the cost of a meaningfully lower hit rate and duplicated storage. For a
company with 900 repos and heavy dependency overlap, sharing is worth it —
but if the toolchain diversity is high, the shared cache's hit rate may not
justify the security design, and someone could reasonably prefer the simpler
model.

**5.5 The physical device in Berlin.** I'd support bring-your-own-runner —
register the machine with the platform, let it receive jobs, give it the
platform's observability, and leave it on the desk. Someone else would say
that permitting self-hosted runners is a permanent escape hatch that
guarantees a long tail of unmigrated snowflakes forever, and that forcing
the hardware into a datacentre is the only way the consolidation ever
finishes. They have a point; the counter-point is that one exception is
cheaper than one team refusing to move.

**5.6 Whether to promise "faster builds" at all.** The mandate is cheaper
*and* faster. Cheaper comes from utilisation; faster comes overwhelmingly
from the cache and from queue time, not from the execution platform's
architecture. A Staff engineer should be explicit that most of the "faster"
will come from caching and from eliminating the 6-minute morning queue — and
that a job that was 20 minutes of actual compilation will still be 20
minutes. Managing that expectation before the migration is part of the job.
