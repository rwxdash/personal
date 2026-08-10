# Rubric — 09 The Bare-Metal Fleet Engine

> **Spoiler.** Do not open until the final review round is written.

Weight: **scoping 25% · design 40% · live extension 25% · questions 10%.**

---

## The central tension

**You must commit capital 4–5 months before you know whether you need it,
and both errors are expensive in different currencies.**

This is the only problem in the set whose core tension is not technical. It
is capital allocation under uncertainty, expressed as infrastructure, and
the systems design exists to make the uncertainty survivable.

The arithmetic: at 8%/month growth, a 6-month horizon needs ~1,760 machines
ordered now. Forecast 3% when reality is 8% and you are ~1,180 machines
short at month 6 — you turn away revenue for a quarter and existing
customers hit capacity limits. Forecast 8% when reality is 3% and you have
~1,180 idle machines, **~$17.7M of capital sitting in racks doing nothing**,
plus the power and space to run them.

**Cloud makes this problem disappear and you don't have cloud.** Every
elasticity guarantee sold to customers must be manufactured from a fixed
fleet through density, overcommit, and headroom. That is the connection to
problem 07 and it should be made explicitly: the compute engine's overcommit
ratio is the lever that converts fixed capacity into apparent elasticity, and
the fleet engine's headroom is what absorbs the error.

**The insight the problem is built around: the levers that act faster than
the lead time are not hardware.** Within a 4-month window you cannot buy
your way out. What you *can* do:

- Raise density / overcommit (problem 07's dials)
- Reclaim capacity from free tiers and idle workloads
- Delay non-urgent maintenance, temporarily lowering headroom
- Rent cloud capacity as an expensive overflow valve
- Slow customer growth deliberately — waitlists, pricing, quotas

A candidate who designs only the provisioning pipeline has built the thing
that is *too slow to matter* in the case that hurts. The pipeline must
exist and be excellent; the design's *judgement* shows in the short-horizon
levers.

**Secondary tension:** every machine you take out of service is carrying
paying customers, some with volumes. Maintenance is therefore rationed
against availability, and a fleet that cannot be patched quickly is a fleet
with a permanent security exposure. The "how much of the fleet may be
non-serving at once" number is where this gets decided, and it is a real
percentage of purchased capital that is never earning.

---

## Scoping — what a strong candidate establishes unprompted

**25% of the grade.**

| # | Decision | Why it matters |
| --- | --- | --- |
| 1 | **Where the engine begins.** Purchase order, loading dock, or powered machine? | Including procurement makes it a supply-chain system; excluding it means someone else owns the hardest part, and that must be said. |
| 2 | **What "schedulable" means.** | Booted ≠ burned in ≠ trusted with customer data. Days apart, and the definition determines the pipeline. |
| 3 | **Is capacity planning in scope?** | The correct answer is yes, or "no, and here is whose it is and what I need from them." Silently omitting it means the design doesn't address the actual risk. |
| 4 | **Automated vs manual.** | There is always a human with a screwdriver. Pretending otherwise is the tell of someone who hasn't run hardware. |
| 5 | **Who owns failure-domain truth.** | Compute and storage schedulers both consume it; if nobody owns it, it is wrong within a month. |
| 6 | **Colo vs owned building.** | Different constraints, different lead times, different failure modes, different people. |
| 7 | **Scale.** | Pick, state, derive. |
| 8 | **Power as a constraint.** | Racks are limited by kilowatts, not rack units. Almost nobody raises this and it is the constraint that actually binds in a colo. |

6+ with reasons is a Staff signal. Fewer than 4 caps at Senior.

---

## Back-of-envelope

| Quantity | Defensible value | Working |
| --- | --- | --- |
| Fleet at month 4 / 6 / 12 | **~4,080 / ~4,760 / ~7,560** | 3,000 × 1.08^n |
| Machines to order for 6 months | **~1,760** | |
| Racks that represents | **~44** | at 40/rack |
| Power that represents | **~880 kW** | at 20 kW/rack — **often the binding constraint, not space** |
| Capital | **~$26M** | 1,760 × $15k |
| **Under-buy at 3% vs 8% actual** | **~1,180 machines short at month 6** | The revenue you cannot serve |
| **Over-buy at 8% vs 3% actual** | **~$17.7M idle** | 1,180 × $15k, plus power and space |
| Machines to provision per month | **~240 and rising** | 8% of 3,000; ~55/week |
| Machine failures | **~90/year ≈ 1.7/week** | 3% AFR |
| Drain time, stateless only | minutes | |
| Drain time, with volumes | **hours** | Data movement, not rescheduling — see problem 08 |
| Non-serving at steady state | **derive it** | Failures + in-repair + in-maintenance + burn-in. Typically a few percent, and it is capital that never earns |
| Headroom total | **derive it** | Failure absorption + maintenance + growth-between-orders + single-rack loss. 10–20% is defensible; the *derivation* is what's assessed |

**Staff:** computes both directions of the forecast error and converts
"short" into **revenue not served**, not machines. Notices that ~55 machines
must be provisioned per week and asks whether the process supports it —
a manual process that takes two hours per machine is ~14 hours/week of
someone's time and does not scale with 8% compounding.

Raises **power as the binding constraint** in a colo, which is the detail
that marks someone who has actually dealt with datacenter capacity.

**Red flags:** No forecast-error arithmetic. Treating growth as a smooth
input with no uncertainty. No headroom derivation — headroom asserted as a
percentage with no reasoning is the most common weak answer here.

---

## Design

### The provisioning pipeline

**Senior:** PXE/netboot, config management, join the cluster. Correct shape.

**Staff:** a *state machine* with explicit gates, and **burn-in as a
first-class stage**. A new machine is not trusted with customer workloads
until it has passed sustained load testing — memory, disk, CPU, network —
because infant mortality is real and a machine that fails in week one should
fail during burn-in, not while holding 67 customer containers. This is the
stage most candidates omit and it is what separates "I can install an OS
from the network" from "I have brought hardware into production."

Also: idempotent and re-runnable (a machine can always be rebuilt from
scratch), rack-at-a-time rather than machine-at-a-time, and a clear answer
to what happens when provisioning fails at 2am (it doesn't page — it goes
into a queue for the morning, because unprovisioned capacity is not an
incident).

### Capacity planning

Must exist, and must include the **short-horizon levers** (above). The
strong version:
- Forecast from a leading signal (signups, deploys, active services), not a
  lagging one (current utilisation), with an explicit uncertainty band
  rather than a point estimate.
- **Order on a rolling cadence** — monthly or quarterly tranches rather than
  one big annual order — so each decision is smaller and the error is
  correctable sooner. This is the single most effective structural mitigation
  and it costs a little in unit price.
- Asymmetric bias, stated: being short costs revenue and reputation; being
  long costs capital. Which you prefer depends on margin and cash position,
  and a Staff answer names the asymmetry rather than assuming.
- Cloud as a **priced overflow valve** for the case where you're short —
  expensive per unit, but it converts a stockout into a margin hit, and a
  design that has no answer to "we're short and hardware is 4 months away"
  is incomplete.

### Maintenance at fleet scale

3,000 machines, a security patch, and every machine carrying customers.
Must address:
- **Failure-domain-aware batching**: never drain two machines holding
  replicas of the same volume, never take more than one rack's worth of a
  failure domain.
- A derived cap on concurrent drains, with the derivation shown.
- **Total fleet cycle time**, computed. If a full patch cycle takes six
  weeks, then your worst-case exposure to a kernel CVE is six weeks, and
  that is a security posture someone should have agreed to.
- The stateful case: draining a machine with volumes is data movement
  measured in hours (problem 08), so the cycle time is dominated by the
  stateful population, not the machine count.
- Rollback: a firmware update that bricks machines is not theoretical, and
  the design should stage updates by hardware generation with a canary.

### Failure domains

The information originates here and is consumed by problems 07 and 08.
Strong answers address **how it stays true**: a model that says two racks
are independent when they share an upstream switch is worse than no model,
because both schedulers are relying on it. Verification — tracing actual
power and network paths, and periodically *testing* by pulling something —
is the part that distinguishes a real answer.

---

## The live extension (25%)

Highest-value pushes:

1. *Growth is 20%/month for two quarters.* Tests whether the short-horizon
   levers exist. The correct answer is not "order more machines."
2. *A supplier slips 10 weeks on 600 machines.* Same lever set, different
   trigger, and it tests whether supply risk was considered at all.
3. *Patch a kernel CVE across 3,000 machines this week.* Tests whether the
   maintenance cycle time was computed and whether it can be compressed.
4. *Your fleet database says a machine is serving and it's been unplugged
   for a month.* Tests reconciliation between the model and physical
   reality — a real and constant problem with owned hardware.
5. *Finance wants 30% less capital next quarter.* Tests whether they can
   trade density, headroom, and hardware lifetime consciously.
6. *Power in one hall is capped.* Tests whether power was ever a
   consideration.

---

## Grading calibration

| Grade | Profile |
| --- | --- |
| **Below Senior** | Describes PXE booting and config management. No growth or capital arithmetic. Capacity planning absent. No burn-in. Treats drain as instant. |
| **Senior** | Solid automated provisioning pipeline with a state machine. Computes fleet growth. Handles machine failure and maintenance batching. Headroom asserted. Doesn't quantify forecast error or identify levers faster than lead time. |
| **Borderline Staff** | Forecast error computed *or* maintenance cycle time derived, not both. Burn-in present. Capacity planning acknowledged but without the short-horizon levers. Failure domains modelled but their accuracy unaddressed. |
| **Staff** | Both directions of forecast error costed, with "short" converted to revenue. Short-horizon levers identified and ranked. Rolling order cadence. Burn-in as a trust gate. Failure-domain-aware maintenance with a derived concurrency cap and a computed fleet cycle time. Headroom derived from its four components. Power raised as a constraint. Provisioning throughput checked against growth. 6+ scoping decisions. |
| **Strong Staff** | All of the above, plus: states that the fastest levers in a shortage are density and demand-shaping rather than hardware, and connects fleet headroom to problem 07's overcommit ratio as one system; treats the fleet model's *accuracy* as a designed property with verification, not a database; prices cloud overflow as a deliberate valve; names the security exposure implied by the fleet cycle time and who signs off on it; and is explicit about how many humans the design requires and what they do. |

---

## Review guidance

**Round 1:** Ask for the capital arithmetic before reading the architecture.
Recompute growth, order quantity, both forecast-error costs, provisioning
throughput per week, and the maintenance cycle time.

The single best probe: *"growth doubles next month and hardware is four
months out — what do you actually do?"* It immediately separates candidates
who designed a provisioning pipeline from candidates who understood the
problem. Follow with *"and if you'd over-ordered instead, what does that
cost?"*

Second best: *"how long does it take to patch every machine in the fleet,
and what's your exposure window for a kernel CVE?"*

**Round 2+:** Commonly dodged: what happens when the fleet database is wrong,
the stateful drain cost, and how many people the design needs. Escalate
those.

**Final:** The highest-leverage improvement is usually one of: "you designed
the pipeline and not the decision — the pipeline is too slow to help in the
case that hurts," "you never costed being wrong about growth in either
direction," or "your maintenance cycle takes six weeks and nobody has agreed
to that as a security posture."
