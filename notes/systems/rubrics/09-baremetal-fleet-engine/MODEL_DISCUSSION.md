# Model Discussion — 09 The Bare-Metal Fleet Engine

> **Spoiler.** Do not open until the final review round is written.

---

## 1. Open with the capital arithmetic

```
growth 8%/month, lead time 12–20 weeks → commit capital ~4–5 months ahead

fleet today            3,000
month 4                4,081       order ~1,080 by now
month 6                4,761       order ~1,760 by now   ≈ 44 racks
                                                        ≈ 880 kW
                                                        ≈ $26M capital

forecast 3%, actual 8% →  ~1,180 machines SHORT at month 6
forecast 8%, actual 3% →  ~1,180 machines IDLE  ≈ $17.7M of dead capital
```

That is the problem. Everything else in the design exists to make being
wrong survivable, because **you will be wrong** — nobody forecasts a
compounding growth rate accurately four months out.

And the reason this is hard in a way a cloud-native platform never
experiences: **the elasticity we sell customers has to be manufactured from
a fixed fleet.** When a customer scales up, there is no ASG. There is
density, overcommit, and headroom — which means this engine and problem
07's scheduler are two halves of one system.

---

## 2. The reframe: the pipeline is too slow to save you

The natural instinct is to design the provisioning pipeline, and it does
need to exist and be excellent. But in the case that actually hurts —
demand arrives and you're short — **the pipeline cannot help, because it
sits behind a 4-month lead time.**

So the design's judgement shows in the levers that act *faster* than
procurement:

| Lever | Timescale | What it costs |
| --- | --- | --- |
| **Raise overcommit ratio** (problem 07's dial) | hours | Higher risk of the overcommit being called; someone gets evicted |
| **Reclaim from free tier / idle** — sleep more aggressively, shorten idle timeouts, evict abandoned projects | days | Free-tier experience, some churn |
| **Defer maintenance**, spend the headroom | days | Security exposure and deferred failures — a debt with interest |
| **Rent cloud as overflow** | days–weeks | Expensive per unit; converts a stockout into a margin hit |
| **Shape demand** — waitlist, raise prices, quota new signups | weeks | Growth and goodwill, and it is a real option people forget is available |
| **Expedite hardware** | weeks | Premium pricing, partial relief |
| Order more | 4–5 months | Doesn't help this quarter |

Ranking those, and knowing that the top of the list is not hardware, is the
Staff move in this problem.

The structural mitigation on the ordering side is **rolling tranches** —
monthly or quarterly orders rather than one large annual commitment. Each
decision is smaller, each error is correctable one cycle later, and the
forecast horizon you must be accurate over shrinks. It costs a little in
unit price and it is worth it.

---

## 3. The design

```
 ┌──────────────────────────────────────────────────────────────┐
 │ CAPACITY PLANNER                                              │
 │  · forecasts from LEADING signals (signups, deploys, active   │
 │    services) with an uncertainty band, not a point estimate    │
 │  · rolling monthly tranches                                    │
 │  · owns the short-horizon lever list and who may pull each     │
 └────────────────┬─────────────────────────────────────────────┘
                  │ purchase orders
                  ▼
 ┌──────────────────────────────────────────────────────────────┐
 │ PROVISIONING PIPELINE   (state machine, rack-at-a-time)       │
 │                                                               │
 │  racked → discovered → firmware → network → OS → identity     │
 │        → BURN-IN → verified → joined → schedulable            │
 │                       ▲                                       │
 │                       └── the trust gate: no customer         │
 │                           workload before this passes          │
 └────────────────┬─────────────────────────────────────────────┘
                  ▼
 ┌──────────────────────────────────────────────────────────────┐
 │ FLEET STATE  (source of truth, continuously reconciled        │
 │               against physical reality — §3.3)                │
 │  · machine record, generation, capabilities                   │
 │  · FAILURE DOMAINS: rack / PDU / switch / hall / site         │
 │  · lifecycle state                                            │
 └──────┬────────────────────────────┬──────────────────────────┘
        │ constraints                │ health signals
        ▼                            ▼
 ┌──────────────┐          ┌──────────────────────────────────┐
 │ problem 07   │          │ HEALTH & MAINTENANCE ORCHESTRATOR │
 │ scheduler    │          │  · comparative health detection    │
 │ problem 08   │          │  · failure-domain-aware batching   │
 │ storage      │          │  · drain → update → verify → return│
 └──────────────┘          │  · cap on concurrent non-serving   │
                           └──────────────────────────────────┘
```

### 3.1 Burn-in is the trust gate

A new machine is not schedulable when it boots. It is schedulable when it
has survived sustained load — memory under pressure, all drives written and
verified, CPU at thermal load, network at line rate — for long enough to
shake out infant mortality.

This matters because the alternative is discovering a bad DIMM while the
machine holds 67 customer containers and 12 volumes. Burn-in moves that
discovery to a time when the machine holds nothing. It costs days of
capacity per machine and it is unambiguously worth it.

It also gives you a hardware-generation quality signal: if a batch fails
burn-in at an unusual rate, you know before it's in production, and you have
a conversation with the supplier while you still have leverage.

### 3.2 Maintenance is rationed against availability

The cap on how much of the fleet may be non-serving at once is derived, not
picked:

- Failure absorption (something is always broken)
- In-repair and awaiting RMA
- In burn-in
- In maintenance right now

Sum it, add the growth buffer and the single-rack-loss reserve, and that is
your headroom — call it 10–20%, but **the derivation is the answer, not the
number**. It is also capital you bought and are not earning on, which is the
honest framing when finance asks why utilisation isn't higher.

Batching must be **failure-domain aware**, and this is where the three
problems interlock: never drain two machines holding replicas of the same
volume (problem 08), never take enough of a rack to breach the compute
scheduler's spread constraints (problem 07). The maintenance orchestrator has
to ask both systems for permission, which means the contract between them is
real API, not a wiki page.

Then compute the **fleet cycle time**. If patching everything takes six
weeks, your worst-case exposure to a kernel CVE is six weeks. That is a
security posture, and someone — a named person, not "the team" — has to
agree to it. Most organisations have this number and have never said it out
loud.

The stateful population dominates: draining a machine with volumes is data
movement measured in hours (problem 08), so cycle time is set by the
stateful machines, not the machine count.

### 3.3 The fleet model must be verified, not just stored

With owned hardware, the database and physical reality diverge constantly —
a machine gets moved, a cable gets repatched, a rack gets rewired during a
power upgrade, someone swaps a drive and doesn't update the record.

This matters more than it sounds, because **problems 07 and 08 make safety
decisions from this data**. If the model says two racks are independent and
they share an upstream switch, then the storage engine has placed both
replicas of a volume behind one point of failure while believing it did the
opposite. The model being wrong is worse than having no model, because both
schedulers trusted it.

So: continuous reconciliation (LLDP neighbour discovery against the recorded
topology, PDU inventory against the recorded power domain, BMC inventory
against the recorded hardware), alerting on divergence, and — periodically —
*testing* a failure domain by actually failing it in a controlled way. A
failure domain you have never exercised is a hypothesis.

### 3.4 Power, not space

Racks are limited by kilowatts before rack units. At ~20 kW/rack, an
880 kW expansion is the number to negotiate with the colo, and power is
frequently the thing that is unavailable — a hall with free space and no
spare power capacity is a common and expensive surprise, and lead times on
additional power can exceed lead times on machines.

This also shapes hardware choice: a denser machine that draws more power may
not be deployable in an existing hall, which makes "buy bigger machines" a
constrained option rather than an obvious one.

---

## 4. Why the main alternative loses

**The alternative: don't own hardware. Run on cloud, autoscale, and delete
this entire engine.**

It should be taken seriously, because it makes the hardest part of this
problem — the capital commitment under uncertainty — *disappear*. No lead
time, no forecast error, no burn-in, no RMA process, no datacenter
relationships, and a much smaller team.

It loses for a platform company on:

1. **Unit economics.** A company whose product is selling compute cannot buy
   compute at retail and resell it at a margin. Owning hardware at 3,000
   machines is a large multiple cheaper per unit of capacity than renting
   it, and that difference *is* the business model.
2. **No egress and inter-machine transfer charges**, which for a platform
   moving customer traffic and replicating volumes is a very large recurring
   line item (interviews 04·A13).
3. **Control over the machine** — NVMe layout, NIC speed, kernel, firmware,
   microVM support, CPU generation. Several decisions in problems 07 and 08
   depend on being able to specify hardware.

**Where it genuinely wins**, and a good answer concedes it: cloud is
strictly better for *bursty, unpredictable, or new-region* capacity. The
mature answer is not either/or — own the predictable baseline, rent the
overflow and the experiments, and treat cloud as a deliberate, priced valve
for exactly the forecast-error case this whole design is built around.
A design that owns everything and has no overflow valve has no answer when
it's short.

---

## 5. Where reasonable engineers would disagree

**5.1 How much headroom.** I'd carry 10–20%, derived. Someone with a
tighter cash position would run at 5% and accept that a rack failure during
a growth spike becomes a customer-visible capacity event. Someone
availability-focused would carry 25%. It's a risk-appetite question and the
right response is to make the tradeoff visible rather than to have an
opinion about the number.

**5.2 Bias on the forecast error.** I'd bias toward over-buying, because
being short costs revenue you cannot recover and damages the growth that
justifies the whole company, while idle capital is recoverable — it gets
used two months later. But at $17.7M of exposure, a CFO may reasonably
prefer the stockout, and this is genuinely their call and not the
engineer's. The engineer's job is to produce the number and the levers, not
the decision.

**5.3 Whether capacity planning belongs in this engine.** I've included it.
A reasonable structure puts forecasting with finance or a capacity team and
leaves this engine purely operational. My objection is that separating the
forecast from the people who know the fleet's real headroom produces
forecasts that are wrong in avoidable ways — but it's an org design
argument, not an architecture one.

**5.4 Burn-in duration.** Days of burn-in per machine, at 8% monthly growth,
is a meaningful amount of capacity permanently in a non-serving state. A
shorter burn-in ships capacity faster and pushes more infant mortality into
production. I'd rather lose the days; someone racing a capacity crunch would
reasonably shorten it, and should do so knowingly rather than by
accident.

**5.5 What I'd cut to ship in a quarter.** Cut automated capacity
forecasting (do it in a spreadsheet with a monthly review — it's a judgement
call informed by data, not an algorithm). Cut failure-domain
auto-verification. Ship: the provisioning state machine with burn-in, the
fleet state model with manually-maintained failure domains, and the
drain/maintenance orchestrator with a hard concurrency cap. That's enough to
operate 3,000 machines safely, and the spreadsheet is honestly fine at this
size — which is worth saying, because knowing what *not* to automate at a
given scale is as much a Staff signal as knowing what to build.
