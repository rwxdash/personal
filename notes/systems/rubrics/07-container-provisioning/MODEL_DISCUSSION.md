# Model Discussion — 07 The Container Provisioning Engine

> **Spoiler.** Do not open until the final review round is written.

---

## 1. Scoping, out loud

> "I'm designing the engine that takes a service definition and an image
> reference and keeps the right containers running in the right places on a
> fleet we own. In scope: placement, lifecycle, wake-from-sleep, rollout,
> and the machine agent. Out of scope: building source into images, the
> HTTP router itself, and volume storage — I'll define the interfaces to
> each. Assumptions: 500k services, 60% asleep, 3,000 machines, 2M container
> creations a day, anonymous tenants running hostile code, and a 3-second
> wake budget. The wake budget and the hostility are what drive most of the
> decisions."

---

## 2. The numbers that decide it

**The wake path, budgeted:**

| Stage | Cold | Warm | Controlled by |
| --- | --- | --- | --- |
| Edge identifies sleeping service | ~5 ms | ~5 ms | us |
| Placement decision | ~10 ms | ~10 ms | us |
| **Image available on machine** | **3.2 s @ 1 Gbps** | **~0 ms** | **us — the dominant term** |
| Sandbox create + start | 50–500 ms | ~0 ms (pre-warmed) | us |
| **Application boot** | **0.1–10 s** | 0.1–10 s | **the customer — not us** |
| First byte | | | |

Two conclusions, and they're the whole design:

1. **Image pull is the dominant term we control**, and 400 MB at 1 Gbps is
   3.2 seconds — the entire budget, gone, on one hop. Either the image is
   already there, or we don't make the number.
2. **Application boot is not ours.** A Rails app that takes 8 seconds to
   boot blows the budget regardless of what the platform does. Any honest
   design says so, and it points at the only technique that addresses it:
   snapshot and restore a *booted* process rather than starting one.

**Density:** 200,000 running containers ÷ 3,000 machines = **67 per
machine**. At the declared 4 GB that's 267 GB before selling anything.
Overcommit is not an optimisation here — it is the business model.

**Scheduling throughput:** 2M creations/day is **~23/second**. That is not a
hard distributed-systems problem. The difficulty is entirely in the
*constraints* — volume affinity, tenant separation, image locality,
fragmentation — not in the rate. A candidate who builds a sharded scheduler
for 23 placements/second has solved a problem they don't have.

---

## 3. The design

```
   git push / API                     inbound request for a sleeping service
        │                                          │
        ▼                                          ▼
 ┌──────────────────┐                    ┌──────────────────────┐
 │ CONTROL PLANE     │                   │ EDGE ROUTER           │
 │ · desired state   │                   │ · holds the request   │
 │ · scheduler       │◄──── wake ────────┤ · asks for a wake     │
 │   (constraint     │                   │ · retries/streams     │
 │    solver, not a  │                   └──────────────────────┘
 │    throughput one)│
 │ · rollout ctrl    │        NOT ON THE REQUEST PATH
 └────────┬──────────┘
          │ desired state (watch/lease)
          ▼
 ┌─────────────────────────────────────────────────────────────┐
 │ MACHINE AGENT  (×3,000) — semi-autonomous                    │
 │  · reconciles its own containers from cached desired state   │
 │  · KEEPS RUNNING WITH THE CONTROL PLANE DOWN                 │
 │  · local image store + lazy loader + P2P peer fetch          │
 │  · pre-warmed sandbox pool                                   │
 │  · reports observed usage (drives overcommit decisions)      │
 └─────────────────────────────────────────────────────────────┘
          │                        │
   ┌──────▼──────┐          ┌──────▼─────────────────┐
   │ microVM per │          │ IMAGE DISTRIBUTION      │
   │ tenant      │          │ · registry + regional   │
   │ workload    │          │   mirrors               │
   │             │          │ · P2P between machines  │
   └─────────────┘          │ · lazy/on-demand layers │
                            └────────────────────────┘
```

### 3.1 Placement and image caching are one problem

The scheduler's job is not "find a machine with capacity." It is "find a
machine with capacity **that already has this image**," because that
constraint is worth 3 seconds and every other constraint is worth
milliseconds.

So image locality is a first-class scheduling input with a heavy weight,
alongside the hard constraints (volume affinity, tenant anti-affinity,
machine class). A service that has run before has a set of machines that
have its layers cached, and the scheduler strongly prefers them. This is the
single most valuable idea in the design and it falls straight out of the
budget arithmetic.

Supporting mechanisms:
- **Lazy image loading.** Start the container after fetching the handful of
  megabytes it actually touches, streaming the rest on demand. Turns a
  400 MB pull into a small one for the critical path. This is the largest
  single lever on cold start.
- **P2P layer fetch** between machines so a popular base image comes from a
  neighbour at LAN speed and the registry isn't a bottleneck at 800 TB/day
  of notional pulls.
- **Regional mirrors**, so a cold layer is at most a site-local fetch.
- **Pre-warmed sandbox pool** per machine, so create-and-start is off the
  critical path.

### 3.2 The runtime: microVM, and pay for it honestly

Tenants are anonymous, arrive with a card, and some are hostile by
intention. A shared kernel is not a boundary you can defend to a customer
whose database is on the same machine (interviews 08·A11).

So: **a microVM per tenant workload**. The costs, stated rather than
waved at:
- A per-VM memory floor, which at 67 workloads per machine is a real density
  loss.
- Loss of page-cache sharing between containers of the same image — with
  many services on the same base image, that sharing was worth something,
  and it's gone.
- More moving parts in the agent.

What it buys, which is why it's worth it: an escape reaches a VM, not the
host and not the 66 neighbours. For a platform whose product is "run
strangers' code next to each other," that boundary is the product.

### 3.3 The data plane survives the control plane

Machine agents hold their cached desired state and keep reconciling.
Control plane down means **no new deploys and no wakes** — but every
already-running customer service keeps serving traffic.

This is a product guarantee, not an implementation detail: a control-plane
outage that takes customer traffic down is an outage of the entire company,
and the difference between "customers couldn't deploy for 30 minutes" and
"customers were down for 30 minutes" is existential.

It has a cost: agents must be able to act on stale state, which means the
reconciliation logic has to be safe under staleness, and it means a wake
cannot depend on the control plane. I'd let the **router trigger a wake by
talking to a machine agent directly** using a cached placement hint, with
the control plane as the slow path — so even wakes degrade rather than fail.

### 3.4 Overcommit, and who dies when it's called

Schedule on **observed** usage plus headroom, not on declared requests.
Agents report real consumption; the scheduler's view of a machine's free
capacity is empirical.

When a machine's tenants collectively demand more than exists, memory is
incompressible and somebody gets killed. The policy must be deliberate:

1. Workloads **over their own declared request** are killed before workloads
   within theirs. The customer who declared 4 GB and is using 1 GB should
   never be the victim of the customer using 12.
2. **Tier ordering** — free-tier and preview environments before production.
3. Use `memory.high` to throttle before `memory.max` kills (interviews
   05·A7), so the machine degrades before it evicts.
4. Evict *and reschedule* rather than kill, where the workload is stateless.

And the machine-level protection: a headroom reservation that is never
allocated, so a spike has somewhere to go while eviction happens.

**Never allowed:** a paying production service killed to protect a free-tier
one. If the policy can produce that, the policy is wrong.

### 3.5 Stateful pins everything

A container with a volume runs where the volume is. Consequences, all of
which have to be followed through:

- **Hard scheduling constraint**, not a preference. The scheduler has no
  freedom for these, so they should be placed *first*, with stateless
  workloads filling around them.
- **Fragmentation accumulates.** Pinned workloads can't be moved to defrag,
  so machines drift toward a state where they have CPU but not contiguous
  memory, or vice versa. Needs a defrag process that only moves stateless
  workloads, and an accepted level of waste.
- **Maintenance becomes data movement.** Draining a machine with stateless
  workloads is minutes; with 12 stateful ones it is a replication or restore
  operation measured in hours. Maintenance windows must be planned around
  the stateful population, not the container count.
- **Machine failure is a different incident.** 55 stateless containers
  reschedule automatically; 12 stateful ones need their volumes recovered
  and are down until that happens. The customer-visible impact of one
  machine failure is dominated entirely by its stateful tenants.

This is the interface to problem 08 and the boundary should be stated: the
provisioning engine asks a storage engine "where can this volume be
attached," and the storage engine's answer is a constraint the scheduler
must honour.

---

## 4. Why the main alternative loses

**The alternative: run Kubernetes. One large cluster (or a few), containers
with a hardened security context, cluster autoscaler off since the fleet is
fixed, and a custom controller for sleep/wake.**

It is a completely reasonable starting point and many companies do it. It
loses here on:

1. **Churn.** 2M container creations/day against etcd — pod objects, status
   updates, events — is far outside the pattern Kubernetes is tuned for.
   etcd is the constraint (interviews 08·A9) and the workload is
   pathological for it: high create/delete rate on short-lived objects.
2. **Isolation.** The default container boundary is inadequate for anonymous
   hostile tenants, and retrofitting microVMs means a runtime class plus
   accepting that much of the ecosystem assumes shared-kernel semantics.
3. **The scheduler optimises for the wrong thing.** Kubernetes' scheduler
   knows nothing about image locality as a *latency* constraint, and image
   locality is worth 3 seconds here. You'd be writing a scheduler plugin
   for the single most important placement input.
4. **The control plane is on the critical path** for wake, and Kubernetes'
   availability model is not built around "the data plane must keep working
   and wakes must still happen."

**Where it genuinely wins**, and it should be conceded loudly: you get a
decade of other people's operational hardening, an enormous ecosystem, and
engineers who already know it. Building your own orchestrator is a
multi-year commitment and a permanent tax on hiring. If the fleet were 300
machines rather than 3,000, or the tenants trusted, Kubernetes would be
correct and building a scheduler would be the wrong use of a team.

The honest position: I'd expect a real company to start on Kubernetes and
grow out of it at exactly these numbers, and a candidate who says *"I'd use
Kubernetes today and here is the specific threshold at which I'd replace
it"* is giving a better answer than one who builds from scratch on day one.

---

## 5. Where reasonable engineers would disagree

**5.1 microVMs everywhere.** The density cost is real. An alternative is
containers-with-gVisor for the free tier and microVMs for paid, or
per-tenant machine pools for anyone who'll pay for it. My objection is that
a platform's isolation story shouldn't have a cheap tier — the escape you
suffer will be from the tier you economised on — but the density argument is
strong and the counter-position is legitimate.

**5.2 Whether to promise a 3-second wake at all.** Since application boot is
outside our control, the promise is partly unkeepable and partly measuring
the customer's code. A more honest product promise is "container ready in
under 500 ms; total time depends on your app," plus tooling that shows the
customer their own boot time. Someone in product will hate that. They're not
wrong that "3 seconds" sells better.

**5.3 Snapshot/restore.** It's the only technique that attacks application
boot, and it's genuinely hard — restoring a process with open sockets,
timers, and a clock that jumped is a source of subtle bugs, and it interacts
badly with anything that seeded randomness or cached a connection at boot. I
think it's worth it for the sleeping-hobby-project case, which is 60% of
services. A reasonable engineer would say ship without it and revisit.

**5.4 Scheduling on observed rather than declared usage.** It's what makes
the density work and it means a customer who is quiet for a month and then
gets traffic may find their machine full. The alternative — schedule on
declarations — is honest, predictable, and several times more expensive.
This is a business decision dressed as an engineering one, and it should be
made by someone who understands both.

**5.5 What I'd cut to ship in a quarter.** Snapshot/restore, P2P
distribution, and defragmentation. Ship: agent, scheduler with image
locality and volume affinity, microVM runtime, pre-warm pool, and lazy image
loading. That's a working platform; the rest are increments with numbers
attached, and being able to say which increment buys what is more valuable
in the interview than having designed all of them.
