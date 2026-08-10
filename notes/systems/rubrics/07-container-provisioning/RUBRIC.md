# Rubric — 07 The Container Provisioning Engine

> **Spoiler.** Do not open until the final review round is written.

Weight: **scoping 25% · design 40% · live extension 25% · questions 10%.**

---

## The central tension

**Cold-start latency, density, and isolation are mutually hostile, and
stateful workloads destroy the placement freedom that density depends on.**

- **Wake in 3 seconds** requires the image to already be on the machine and
  ideally a sandbox already warm. Pre-warming costs memory that isn't sold.
- **Density** requires aggressive overcommit — 200,000 containers on 3,000
  machines at declared 4 GB each is 267 GB/machine before you've sold
  anything, and the only way it works is that most services use a fraction
  of what they declare. Overcommit is the business model.
- **Isolation** against anonymous hostile tenants wants a hardware boundary,
  which raises the per-container memory floor and *reduces* density, and
  historically raised start latency.
- **Stateful workloads pin placement.** A container with a volume must run
  where the volume is. Every pinned workload is a hole in the bin-packing,
  and pinning is the enemy of both density and fast rescheduling.

The naive design — Kubernetes, one pool, containers, schedule freely — fails
on at least two of the four, and which two depends on what it got wrong.

**The insight the problem is built around: the wake budget is dominated by
terms you do not control, and the ones you do control are all about having
already done the work before the request arrived.** Image pull is the
dominant controllable term, and 400 MB at 1 Gbps is 3.2 seconds — the entire
budget. So the design is really about *cache hit rate on image content* and
*pre-warmed capacity*, not about making anything faster.

**Secondary tension:** overcommit is what makes the unit economics work and
it is a loaded gun. The day a machine's tenants all want their declared
memory simultaneously, someone gets OOM-killed, and it is a paying customer
who did nothing wrong. Designing the overcommit policy — and what happens
when it's called — is where the real engineering is.

---

## Scoping — what a strong candidate establishes unprompted

**25% of the grade.**

| # | Decision | Why it matters |
| --- | --- | --- |
| 1 | **Where the engine starts and stops.** | Build-from-source is an entire second system. Including it makes everything shallow; excluding it with a stated interface is the stronger move. |
| 2 | **Build on an orchestrator or build one.** | Both are defensible and both need justification. "Use Kubernetes" must come with what it costs — the scheduler is not designed for 2M creations/day of hostile short-lived workloads, and etcd is not designed for that churn. "Build our own" must come with what you're taking on. |
| 3 | **The container runtime.** | runc (fast, weak boundary), gVisor (syscall interception, real overhead), Firecracker/Kata (VM boundary, memory floor). This is *the* isolation/density/latency decision and it must be made explicitly. |
| 4 | **The latency contracts.** | Wake, deploy, and scale are three different numbers and nobody gave them to you. |
| 5 | **What changes for stateful.** | It's in the role family's prompts repeatedly. Pinning is the answer and it has consequences everywhere. |
| 6 | **Scale.** | Pick, state, derive. |
| 7 | **Multi-tenancy model** — do tenants share machines? | Determines the entire isolation and density story. |
| 8 | **What survives the control plane being down.** | For a platform, this is a product property, not an implementation detail. |

6+ established with reasons is a Staff signal. Fewer than 4 caps at Senior.

---

## Back-of-envelope

| Quantity | Defensible value | Working |
| --- | --- | --- |
| Running containers | **~200,000** | 500k services × 40% awake |
| Density | **~67 containers/machine** | 200k ÷ 3,000 |
| RAM at declared size | **~267 GB/machine** | 67 × 4 GB — requires large machines *and* overcommit |
| Container creations | **~23/s average, ~100/s peak** | 2M/day. Modest rate — **the scheduler is not the bottleneck**, and a candidate who spends the whole design on scheduler throughput has misread the problem |
| If every creation pulled | **~800 TB/day** of registry egress | 2M × 400 MB |
| **Image pull at 1 Gbps** | **3.2 s** | **Exactly blows the 3 s wake budget** |
| Image pull at 10 Gbps | **0.32 s** | Fits — so NIC speed is a design input, not a given |
| Image pull at 25 Gbps | **0.13 s** | |
| Cache hit rate needed | **high** | The p95 budget only holds if the common case is a cache hit; derive it from the budget |
| Overcommit ratio needed | **several x** | Declared 4 GB vs observed p50 usage a fraction of that |

**Staff:** computes the wake breakdown hop by hop, identifies **image pull
as the dominant controllable term**, and notices that **application boot time
is not theirs** — a customer's Rails app taking 8 seconds to boot blows the
budget no matter what the platform does. That honesty is a strong signal, and
it leads somewhere: the platform can only promise *container ready*, and the
product needs to either measure from the right place or offer
snapshot/restore of a booted process.

Also notices that **23 placements/second is not a hard scheduling problem** —
the difficulty is in the constraints (volume affinity, tenant separation,
fragmentation), not the throughput.

**Red flags:** No wake breakdown. Designing a distributed scheduler for
23 placements/s. Not pricing image distribution. Treating overcommit as
free.

---

## Design

### Architecture

**Senior:** control plane with a desired-state store, agents on machines,
scheduler, registry, router. Correct components.

**Staff:** the **data plane survives the control plane**. Running containers
keep serving with the control plane down; only *changes* stop. This is the
same property as interviews 08·A9 and for a PaaS it is a product guarantee —
a control-plane outage that stops customer traffic is an outage of the
entire company.

The wake path is annotated with its budget. The router's view of where a
service runs is addressed explicitly, including staleness during moves.
Machine agents are treated as semi-autonomous rather than as remote hands.

**Red flags:** A design where the control plane is on the request path.
No account of how the router learns placement. Magic "scheduler" box.

### The runtime decision

Must be made and defended. The strongest answers recognise this is not one
decision but a *policy*:

- Hostile/anonymous tenants → hardware boundary (microVM). The memory floor
  and the density cost are real and should be quantified.
- The argument that microVMs are now fast enough to boot that latency is not
  the objection — the objection is the memory floor and the loss of
  page-cache sharing between containers of the same image, which is a real
  density loss at 67/machine.
- gVisor as a middle option, with its syscall overhead being workload-
  dependent and its compatibility gaps being a real support burden.

A candidate who picks runc for anonymous internet tenants needs to defend it
hard, and probably can't.

### Cold start

The strong moves, in rough order of impact:

- **Image caching on machines**, with the scheduler *preferring machines
  that already have the image* — placement and caching become one problem,
  which is the elegant insight.
- **Lazy / on-demand image loading** (stargz, nydus, or equivalent) so the
  container starts after pulling the few MB it actually needs rather than
  400 MB. This is the single biggest lever for cold start and a candidate
  who names it is showing current knowledge.
- **P2P distribution** between machines so a popular image doesn't hammer
  the registry and pulls come from a neighbour at LAN speed.
- **Pre-warmed sandboxes** — a pool of started-but-empty runtimes so
  container create is not on the critical path.
- **Snapshot/restore** of a booted process, which is the only technique that
  addresses *application* boot time — the term the platform otherwise
  cannot control.
- **Thundering herd**: 500 simultaneous wakes must not stampede the registry
  or a single machine. Needs admission control and spread.

### Overcommit and its failure

The design must state the ratio and what happens when it's called. Strong
answers:
- Schedule on *observed* usage with headroom, not on declared requests.
- Memory is incompressible (interviews 05·A7, 08·A2) — when overcommit is
  called someone dies. Pick the victim deliberately: by tier, by whether the
  service is over its own declared request, by restartability. Killing a
  paying production service to protect a free-tier one is the wrong answer
  and the design should make that impossible.
- Use `memory.high` for throttling before `memory.max` for killing where
  the runtime allows it — degrade before you kill.
- Live migration or fast eviction of a low-priority workload to relieve
  pressure.

**Red flags:** Overcommit assumed with no policy for when it's called. No
distinction between compressible and incompressible resources.

### Stateful placement

Must be addressed. A container with a volume runs where the volume is;
therefore:
- The scheduler has a hard constraint it cannot optimise away.
- Machine maintenance now requires *data movement*, not just rescheduling.
- Fragmentation gets worse over time as pinned workloads accumulate.
- A machine failure with stateful tenants is a different and much worse
  incident than one with only stateless tenants.

A candidate who schedules stateful workloads like stateless ones has missed
what the prompt kept telling them.

---

## The live extension (25%)

Highest-value pushes:

1. *Wake budget is 300 ms.* Forces snapshot/restore or keeping things warm;
   tests whether they know what's actually possible.
2. *A customer's service has a 40 GB volume.* Tests the pinning consequence.
3. *Why Kubernetes / why not?* Tests whether the choice was made or
   defaulted.
4. *Someone is mining crypto on 400 machines right now.* Tests operational
   reality — detection, response, and whether the design allows a fast
   fleet-wide action.
5. *A machine dies with 67 containers, 12 stateful.* Tests whether stateless
   and stateful recovery were designed separately.
6. *You need 3x the density.* Tests whether they know which knob to turn and
   what it costs.

---

## Grading calibration

| Grade | Profile |
| --- | --- |
| **Below Senior** | No scale. "Use Kubernetes" with no justification or cost. Containers for anonymous tenants. No wake-path breakdown. Stateful treated identically to stateless. |
| **Senior** | Establishes scale and latency contracts. Sensible control/data plane split. Image caching for cold start. microVMs for isolation. Recognises overcommit is needed. Extends live but reaches for components rather than numbers. Doesn't price image distribution or state an overcommit policy. |
| **Borderline Staff** | Wake budget broken down *or* the density/isolation tension quantified, not both. Stateful pinning noticed but its scheduling consequences not followed through. Live extension holds in two or three dimensions. |
| **Staff** | Wake path budgeted hop by hop with image pull identified as the dominant controllable term and app boot identified as *not theirs*. Placement and image caching solved as one problem. Runtime decision made with the density cost quantified. Overcommit ratio derived with an explicit victim-selection policy. Data plane survives the control plane. Stateful pinning followed through into maintenance and failure. 6+ scoping decisions. Extends cleanly in four dimensions. |
| **Strong Staff** | All of the above, plus: names lazy image loading and/or snapshot-restore and knows what each buys; treats abuse as a daily operational reality with a designed response, not a risk to mitigate; notices that 23 placements/s means the scheduler is *not* the hard part and says what is; makes the free-tier sleeping-service cost explicit because it determines whether the business model works; and can say what they'd cut to ship in a quarter. |

---

## Review guidance

**Round 1:** Grade the scoping first. Recompute the wake budget, image pull
time at each NIC speed, and the density arithmetic. Then run three live
extensions in the review.

The single best probe: *"walk me through the three seconds between a request
arriving for a sleeping service and the response going out — with a number
on each step."* Almost everything is visible in the answer: whether they
budgeted, whether they know where the time goes, and whether they noticed
that the customer's own application boot is in the budget and out of their
control.

Second best: *"a machine with 67 containers on it just went unresponsive —
what happens, and what's different for the 12 stateful ones?"*

**Round 2+:** Commonly dodged: what happens when overcommit is called, how
the router stays correct during moves, and the abuse response. Escalate
those.

**Final:** The highest-leverage improvement is usually one of: "you never
budgeted the wake path, so the 3-second contract is an aspiration," "your
scheduler treats a pinned stateful workload like a free one," or "your
density depends on overcommit and you have no policy for when it's called."
