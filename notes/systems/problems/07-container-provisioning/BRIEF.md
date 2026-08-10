# 07 — The Container Provisioning Engine

**Difficulty:** Staff
**Format:** Take-home + 45-minute live extension — read
[TAKEHOME-FORMAT.md](../../TAKEHOME-FORMAT.md) first
**Roles this maps to:** Senior Infra Engineer — Platform / Compute /
Orchestration

---

## The prompt

This is the entire brief, as you would receive it.

> **Architect a Container Provisioning Engine to power something like
> Railway.**
>
> *Pre-work: complete your solution. 0–5m introduction. 5–50m building (or
> expanding) your solution. 50–60m questions.*

**Stop here if you're working in realistic mode.**

---

## What you have to invent

- **Where does the engine begin?** At a git push? At an image reference? Is
  *building* source into an image part of the provisioning engine, or a
  separate system you consume?
- **What does "provisioning" cover?** Scheduling, placement, starting,
  networking, health, rollout, scale, sleep/wake, termination — pick your
  boundary and defend it.
- **Are you building on an orchestrator or replacing one?** "Use Kubernetes"
  is an answer, and it needs a justification and an account of what it costs
  you. So does "don't."
- **What runs the container?** runc, gVisor, Firecracker, Kata,
  something else. This is a security *and* a latency *and* a density
  decision, and they conflict.
- **What is the latency contract?** How long from push to serving? How long
  from a request arriving for a sleeping service to a response? These are
  very different numbers.
- **Stateless and stateful.** The prompt for this family of roles keeps
  saying both. A stateful workload has a volume, and a volume is somewhere
  specific. What does that do to your scheduler?
- **What scale?** Pick, state, derive.

---

## Reference scale

*Use this in guided mode, or compare against your own numbers afterwards.
Plausible figures for a PaaS of this shape — not any real company's
published data.*

| | |
| --- | --- |
| Customers / projects | ~200,000 |
| Deployed services | ~500,000 |
| Of which asleep at any moment | ~60% (hobby projects, preview environments) |
| **Running containers** | **~200,000** |
| Fleet | ~3,000 bare-metal machines, 4 sites, heterogeneous generations |
| **Container creations per day** | **~2,000,000** (deploys, wakes, crashes, restarts, scale events) |
| Average image size | ~400 MB compressed |
| Typical request | 4 GB RAM, fractional CPU — but declared, not measured |
| Actual p50 usage | a small fraction of what is declared |

**Latency contracts you may assume**

| | |
| --- | --- |
| `git push` → serving traffic | p95 under 60 s including build |
| Redeploy of an existing image | p95 under 15 s |
| **Wake a sleeping service on an inbound request** | **p95 under 3 s** — the request is held open, and the user is watching |
| Scale-out of an existing service | p95 under 10 s |

**Two properties that are not negotiable**

1. **The code is hostile.** Tenants are anonymous, sign up with a card, and
   some fraction of them are there to mine cryptocurrency, scan the
   internet, or escape onto your hardware. This is not a hypothetical risk
   to mitigate later; it is a daily operational reality and it is the
   default assumption for every tenant.

2. **You own the hardware.** There is no autoscaling group. Capacity is a
   fixed fleet that changes on a purchasing timescale, so elasticity sold to
   customers must be manufactured out of density and overcommit, not out of
   someone else's spare capacity.

---

## What to produce

Produce **the artifact you would actually bring to the call** — see
[TAKEHOME-FORMAT.md](../../TAKEHOME-FORMAT.md).

Create `DESIGN.md` in this directory. Use [WORKSHEET.md](WORKSHEET.md) to
check coverage, and **do Part 10**, the live-extension drill.

When a round is ready, say so and a review will land in `reviews/round-1.md`.
Reviews include a simulated extension block.
