# Observability, SLOs & On-Call — Answers

---

## Tier 1 — Recall

### A1. Metrics, logs, traces, profiles

**Answer.**
- **Metrics** — pre-aggregated numeric time series. Answer: *is something
  wrong, and how much?* Cheap and constant-cost per series regardless of
  traffic volume, so they're what you alert on and what you keep for a year.
  Their limitation is that the dimensions are fixed at instrumentation time
  — you can only ask questions you anticipated, and adding a dimension
  multiplies cost (A5).
- **Logs** — discrete, timestamped, high-detail events. Answer: *what
  exactly happened to this specific request/entity?* Arbitrary detail,
  queryable after the fact without redeploying. Cost scales linearly with
  traffic, which is why they're the most expensive telemetry per unit of
  insight at high volume.
- **Traces** — a causally-linked set of spans across services. Answer:
  *where did the time go, and which service in the call graph is
  responsible?* Uniquely capable of attributing latency across a
  distributed call path — metrics can tell you service B is slow, only a
  trace tells you that B is slow *because* it's waiting on D, and only for
  requests originating from A. Cost is managed by sampling, which introduces
  its own problems (A11).
- **Profiles** — statistical samples of where a process spends CPU/memory/
  time. Answer: *which code is responsible?* This is the layer below traces:
  a trace tells you the span took 300 ms inside service B, a profile tells
  you it was in JSON serialisation. Continuous profiling makes this available
  retrospectively (topic 05, A14).

The framing that matters: they form a **drill-down chain with increasing
cost and decreasing volume**. Alert on metrics → find the affected requests
via exemplars/traces → read the logs for one of them → profile if the time
is unattributed. Each layer should link to the next (trace IDs in logs,
exemplars in metrics), and **the links are worth more than any individual
signal**. A system where you can't get from a metric spike to an example
request in one click will have a long MTTR no matter how much data it
collects.

**Weak answers miss.** The drill-down chain and correlation IDs. Listing
the four pillars without saying how you traverse them is the shallow answer.

**Follow-ups to expect.**
- Where do events/audit logs fit? (Deployment markers, config changes,
  scaling events — low-volume, high-value context. "What changed?" is the
  first question in most incidents, and it's answered by this, not by any
  of the four.)
- What's OpenTelemetry's role? (Vendor-neutral instrumentation and a
  collector that can transform, sample, and route. The practical value is
  decoupling instrumentation from backend choice — you can change vendors
  without re-instrumenting.)

---

### A2. SLI, SLO, SLA, error budget

**Answer.**
- **SLI** (indicator) — a *measurement* of service behaviour, expressed as a
  ratio of good events to valid events. "Proportion of HTTP requests served
  in under 300 ms." It must be measurable and should reflect user
  experience.
- **SLO** (objective) — a *target* for the SLI over a window. "99.9% of
  requests under 300 ms over a rolling 28 days." Internal.
- **SLA** (agreement) — a *contract* with a customer, with financial or
  contractual consequences. Always looser than the SLO — you want to breach
  the SLO and fix it long before you breach the SLA. If your SLO equals your
  SLA, you have no margin to act.
- **Error budget** — `1 − SLO`. At 99.9% over 28 days, that's 0.1% of
  requests, or about **40 minutes** of total unavailability. It is the
  *permitted* amount of unreliability.

The relationship, and the reason any of this exists: the error budget turns
reliability from an argument into an **arithmetic decision**. Budget
remaining → ship features, take risks, do risky migrations. Budget exhausted
→ the team's priority shifts to reliability work until it recovers. It gives
product and engineering a shared, non-emotional mechanism for the "move fast
vs be stable" argument, which is otherwise decided by whoever is loudest.

The corollary people miss: **100% is the wrong target.** It's unachievable
(your dependencies aren't 100%, the network isn't), and pursuing it is
enormously expensive for value users can't perceive — their ISP and their
phone are less reliable than your service. An *unused* error budget is also
a signal: you're being too conservative and could be shipping faster.

Two useful refinements: SLOs should be measured from the **user's**
perspective where possible (client-side or at the edge, not at the
application) — a server-side SLI can't see the requests that never arrived.
And the window matters: a rolling window is better than a calendar month,
because a calendar reset forgives a bad month arbitrarily on the 1st.

**Weak answers miss.** The error budget as a *decision-making mechanism*
rather than a metric, and that 100% is the wrong target.

**Follow-ups to expect.**
- What do you do when the budget is exhausted? (A pre-agreed policy — freeze
  feature deploys, reliability work only, until it recovers. The policy must
  be agreed *before* it's needed, or it will be negotiated away in the
  moment.)
- 99.9% vs 99.99% — what's the real difference? (40 min/month vs 4 min/month.
  4 minutes is less than most human response times, so 99.99% means
  automated remediation and no single points of failure — an
  order-of-magnitude cost increase. Make sure the extra nine is worth it to
  someone.)

---

### A3. RED and USE

**Answer.**
**RED** (Weaveworks/Tom Wilkie) — for **request-driven services**:
- **R**ate: requests per second
- **E**rrors: failed requests per second (and as a proportion)
- **D**uration: latency distribution

Three signals, per service, per endpoint. It's a *user-facing* view — it
describes what callers experience. This is the right frame for microservices,
APIs, and anything with a request/response shape, and it's what your service
dashboards should lead with.

**USE** (Brendan Gregg) — for **resources**:
- **U**tilisation: percentage of time the resource was busy
- **S**aturation: how much work is queued waiting for it
- **E**rrors: error events

Applied per resource: CPU, memory, disk, network interface, and also
software resources like thread pools and connection pools. It's a
*system* view — it describes whether the machine can keep up.

**The key distinction**: USE's **saturation** is the leading indicator that
utilisation misses. A CPU at 100% utilisation with no run queue is fine —
fully used, nothing waiting. A CPU at 80% with a run queue of 20 is in
trouble. Utilisation saturates at 100% and stops carrying information;
saturation keeps growing and is what actually predicts latency (topic 07,
A11). Most dashboards show utilisation and not saturation, which is why they
look fine right up until they don't.

**Google's Four Golden Signals** — latency, traffic, errors, saturation —
are essentially RED plus saturation, which is the sensible synthesis.

How I'd use them: RED for every service (it's what your SLOs are built
from and what you alert on), USE for every resource underneath (it's what
you look at when RED goes bad). RED tells you *that* users are affected;
USE tells you *why*. And an important refinement on RED: measure duration as
a **histogram**, split by success and failure — mixing fast errors with slow
successes produces a latency metric that improves during an outage.

**Weak answers miss.** Saturation as the leading indicator, and that latency
should be split by outcome.

**Follow-ups to expect.**
- What's the saturation metric for a thread pool? (Queue depth and queue
  wait time. For a connection pool, time spent waiting to acquire — which is
  almost never instrumented and is a common hidden latency source, topic 05,
  A16.)

---

### A4. Averaging percentiles

**Answer.**
**Because percentiles are not linear.** A percentile is a property of a
*distribution*, not a value you can arithmetically combine. The 99th
percentile of a union of two datasets is not the average of their 99th
percentiles, and there is no general way to recover it from them.

Concrete counterexample: instance A serves 1 request with latency 1000 ms —
its p99 is 1000. Instance B serves 999 requests all at 10 ms — its p99 is
10. The average of the two p99s is 505 ms. The **true** p99 of the combined
1000 requests is 10 ms. You've reported a number 50x too high, and in the
reverse scenario you'd hide a real problem.

The error is worse in the direction that matters: averaging across instances
**dilutes** a single bad instance. If one pod in fifty has a p99 of 5
seconds and the rest are at 50 ms, the averaged p99 barely moves — so the
metric that's supposed to expose tail latency specifically hides the most
common cause of it (topic 05, A16).

**What to do instead:**

1. **Aggregate the histogram, not the quantile.** Export **bucket counts**
   from each instance, sum the buckets across instances (they're counters,
   so summing is valid), and compute the quantile from the merged histogram.
   In Prometheus:
   `histogram_quantile(0.99, sum by (le) (rate(http_duration_seconds_bucket[5m])))`
   — note `sum by (le)` **inside**, quantile **outside**. Doing it the other
   way round is exactly the mistake.
2. Use a **mergeable sketch** if you need accurate high quantiles with
   bounded error: t-digest or DDSketch. These are designed so that merging
   sketches from many sources gives a correct quantile of the union, which
   is precisely what naive percentile averaging fails at.
3. **Look at the max**, or at per-instance percentiles side by side, when
   hunting for a bad instance. `max by (pod)` on a latency metric finds the
   outlier that any aggregate hides.

Related traps worth naming: you can't average percentiles **over time**
either (averaging a p99 computed per minute over an hour is the same error),
and a percentile of a percentile is meaningless. Prometheus **summaries**
compute quantiles client-side and are therefore **not aggregatable across
instances** — this is the single biggest practical reason to prefer
histograms over summaries.

**Weak answers miss.** The concrete counterexample and the summaries-aren't-
aggregatable consequence. This question is asked because so many dashboards
in the wild do it wrong.

**Follow-ups to expect.**
- Can you average an average? (Only weighted by count. `sum(rate(sum_total))
  / sum(rate(count_total))` is a valid mean; averaging per-instance means is
  not, unless traffic is identical.)

---

### A5. Cardinality

**Answer.**
**Cardinality** is the number of distinct time series, and a series is
uniquely identified by its metric name plus the full set of label
key-value pairs. `http_requests_total{method="GET", status="200",
endpoint="/api/v1/users"}` is one series; change any label value and it's a
different series.

**Why it kills you: it multiplies.** Total series =
`metrics × ∏(distinct values per label)`. Adding one label with 100 values
multiplies your series count by 100. Adding `user_id` to a metric in a
system with a million users creates a million series *per metric*, per
instance.

The cost is not in the samples — samples are cheap and compress well. The
cost is **per-series overhead**:
- Prometheus keeps an in-memory index and an active chunk **per series**, so
  memory scales with *active series count*, not with sample volume. This is
  the resource that OOMs your Prometheus.
- Query cost scales with the number of series a query must touch. A query
  over a high-cardinality metric can take minutes or exhaust memory.
- The inverted index (label → series) grows, and lookups get slower.
- On restart, WAL replay time scales with series count — so a
  high-cardinality Prometheus takes a long time to come back, which is
  exactly when you need it (A15).

**The label values that cause it**, in order of how often I've seen it:
user IDs, request IDs, trace IDs, session IDs, full URL paths with embedded
IDs (`/api/users/12345/orders/67890` instead of a templated
`/api/users/:id/orders/:id`), timestamps, email addresses, container/pod
names in a high-churn environment (each new pod name is a new series —
"churn" cardinality, which is invisible in a point-in-time count but
accumulates in the index), error messages as labels, and full stack traces.

**The rule**: a label value must come from a **bounded, small set known at
design time**. If you can't enumerate the possible values, it's not a label.
High-cardinality identifiers belong in **logs or traces**, which are built
for them — and **exemplars** are the bridge: attach a trace ID to a histogram
bucket sample so you can jump from an aggregate metric to a specific slow
request without paying series cost.

**Weak answers miss.** That the cost is per-series memory rather than sample
volume, and pod-name churn as a hidden accumulator.

**Follow-ups to expect.**
- How do you find your worst offenders? (`topk(10, count by (__name__)
  ({__name__=~".+"}))` for series per metric; the TSDB status page in
  Prometheus shows top metrics and top label cardinality directly.)
- How do you limit it structurally? (`sample_limit` per scrape target,
  `metric_relabel_configs` to drop offending labels at ingest, per-tenant
  series limits in Mimir/Cortex, and — most effective — code review that
  treats a new label as a design decision.)

---

## Tier 2 — Explain / compare

### A6. Pull vs push

**Answer.**
**Pull (Prometheus)**: the server scrapes targets it discovers via service
discovery.
- **Target health is intrinsic.** The `up` metric tells you a target is
  unreachable — you get failure detection for free, and it's the same
  mechanism as your metrics collection. With push, a silent producer is
  indistinguishable from a healthy one that has nothing to report.
- **Service discovery is the source of truth.** You know what *should* exist
  (from Kubernetes, Consul, EC2 tags), so you can detect what's missing.
- **The collector controls the rate.** Targets can't overwhelm the monitoring
  system by pushing harder; back-pressure is inherent, and a misbehaving
  service can't take down monitoring for everyone.
- **Easy debugging**: `curl` the `/metrics` endpoint and see exactly what
  the server sees. Underrated in practice.
- **Idempotent and stateless targets** — no buffering, no delivery
  semantics, no retry logic in the application.

Where pull breaks:
- **Short-lived jobs.** A batch job that runs for 8 seconds may never be
  scraped. That's what the **Pushgateway** is for — and it's a deliberate
  exception with known caveats (it becomes stale state that must be deleted,
  and it's a single point of failure).
- **Network reachability.** The server must reach every target. Behind NAT,
  in a customer's network, at the edge, on ephemeral serverless compute —
  pull is impossible without a tunnel or an agent.
- **Very large fleets** need scrape sharding, because one Prometheus can't
  scrape everything.
- **Event-driven or irregular data** doesn't fit a periodic sampling model.

**Push (StatsD, OTLP, Datadog agent, remote write)**:
- Works from anywhere, through NAT, from ephemeral compute.
- Naturally handles short-lived processes and event-driven emission.
- The collector can be scaled independently and fronted by a load balancer.
- Costs: you need explicit staleness/liveness detection ("this thing stopped
  reporting" is now a separate problem you must solve); the collector can be
  overwhelmed by a misbehaving producer, so you need per-tenant limits; and
  the client needs buffering and retry logic.

**In practice everyone runs both.** The common architecture is pull locally
(a Prometheus or an OTel collector agent per cluster/node, using service
discovery) and **push globally** via remote write to a long-term store.
That gets you pull's health semantics and discovery at the edge, and push's
reachability and central aggregation. The pull-vs-push argument is largely
settled by that hybrid, and saying so is better than defending one side.

**Weak answers miss.** That `up` is the real argument for pull — failure
detection as a free property — and that the hybrid is the actual answer.

**Follow-ups to expect.**
- How do you monitor a short-lived Kubernetes Job? (Pushgateway, or have it
  write to a collector via OTLP, or — often best — have the *controller*
  expose the outcome as a metric rather than the job itself.)

---

### A7. Prometheus histograms and estimation error

**Answer.**
A **classic Prometheus histogram** exposes a set of cumulative counters:
`_bucket{le="0.1"}`, `le="0.25"`, `le="0.5"`, `le="1"`, `le="+Inf"`, plus
`_sum` and `_count`. Each bucket counts observations **less than or equal
to** its bound. They're counters, so they're **aggregatable across
instances** by summing (A4) — which is the whole reason to use them.

`histogram_quantile()` computes a quantile by finding the bucket containing
the target rank and **linearly interpolating within it**, assuming
observations are uniformly distributed inside the bucket. They aren't —
latency distributions are heavily skewed — so the error is bounded by the
**bucket width**, and it can be large.

Worked example: buckets at 0.5 s and 1 s. Every observation actually lands
at 0.55 s. `histogram_quantile(0.99, ...)` will interpolate somewhere in
[0.5, 1.0] and can report ~0.99 s — nearly **2x** the truth. And if your
p99 falls in the **`+Inf`** bucket (i.e. above your highest finite bound),
Prometheus returns the highest bound, so your p99 is silently pinned at your
last bucket boundary and you cannot see how bad it actually is. That's the
failure to name: **badly chosen buckets don't produce noise, they produce
a confidently wrong number.**

So bucket selection is a real design decision: place boundaries around the
values you care about (your SLO threshold should be *exactly* a bucket
boundary, so the "proportion under 300 ms" query is exact rather than
interpolated), use roughly exponential spacing, and make sure the top
finite bucket is well above your worst realistic latency.

The other cost: each bucket is a separate time series. 20 buckets × your
label combinations is 20x the cardinality (A5), which is why people use too
few buckets and get bad estimates.

**Native (exponential) histograms** — the newer Prometheus feature — change
this fundamentally:
- One time series carries the whole histogram, with buckets defined
  implicitly by an exponential schema (bucket *i* covers
  `[base^i, base^(i+1))`). Resolution is a parameter, not a hand-picked list.
- **No bucket design.** They adapt to the observed range automatically, so
  you can't accidentally pin your p99 at the top bucket.
- **Dramatically lower cardinality** — one series instead of N, with a
  compact sparse encoding of populated buckets only.
- Much better relative accuracy (a bounded *relative* error, typically a few
  percent, rather than an unbounded absolute error from a badly-placed
  bucket).
- Still aggregatable, which preserves the essential property.

Costs: it's newer, so support across the ecosystem (exporters, storage
backends, Grafana, remote-write receivers) needs checking — verify current
state rather than assuming. But for new instrumentation at scale it's the
right direction, and the same idea underlies DDSketch and OTel's exponential
histograms.

**Weak answers miss.** The `+Inf` pinning failure, and putting the SLO
threshold on a bucket boundary.

**Follow-ups to expect.**
- Histogram vs summary? (Summaries compute quantiles client-side — exact for
  that instance, and **not aggregatable** (A4). Use them only when you have
  one instance or genuinely only care per-instance. Histograms otherwise.)
- How would you compute an SLO compliance ratio? (Don't use
  `histogram_quantile` — use the bucket directly:
  `sum(rate(bucket{le="0.3"})) / sum(rate(count))`. Exact, no interpolation,
  and it's why the threshold must be a bucket boundary.)

---

### A8. Scaling Prometheus

**Answer.**
The problem: a single Prometheus is a single machine with local storage. It
scales to a lot (millions of active series on a big host), but it gives you
no HA, no long-term retention, and no global view across clusters.

**Federation** — a central Prometheus scrapes `/federate` on leaf
Prometheus servers, pulling a *subset* of series (usually pre-aggregated
recording rules).
- Pro: simple, no new components.
- Con: it does not scale. The central server's scrape of all leaves is a
  single large request; you can only pull aggregates, so you lose drill-down;
  and it's fragile — one slow leaf delays everything. Fine for pulling a
  handful of top-level SLI series into a global view, wrong as a general
  architecture. Say that explicitly, because federation is what people reach
  for first and regret.

**Remote write** — the leaf Prometheus streams every sample to a remote
endpoint as it's ingested. This is the foundation of every modern option;
the question is what's on the receiving end.

**Thanos** — sidecar per Prometheus uploads completed TSDB blocks to object
storage; a Querier fans out to sidecars (recent data) and Store Gateways
(historical, from object storage) and **deduplicates** replicas at query
time; Compactor does downsampling and compaction in object storage.
- Pro: object storage for cheap unbounded retention; global query view;
  built-in dedup of HA pairs; you keep running normal Prometheus servers, so
  it's incremental.
- Con: query latency for historical data (object storage round trips);
  the Store Gateway needs a substantial index cache; more components; and
  the sidecar model means recent data availability depends on the leaf
  Prometheus being up.

**Cortex / Mimir** — a horizontally scalable, **multi-tenant** TSDB that
receives via remote write. Distributor → Ingester (with replication) →
object storage, with separate query, ruler, and compactor components.
- Pro: genuinely multi-tenant with **per-tenant limits** (series, ingestion
  rate, query concurrency) — which is the decisive feature for a platform
  serving many teams (topic 08, A18); horizontal scaling of every component;
  a single global write and query endpoint.
- Con: many components, a consistent hash ring to operate, and real
  operational complexity. Mimir simplified deployment considerably over
  Cortex, but it's still a distributed system you own.

**VictoriaMetrics** — a remote-write-compatible TSDB, single-binary or
clustered.
- Pro: substantially lower resource usage in most published comparisons,
  much simpler operationally, MetricsQL is a superset of PromQL, good
  compression.
- Con: not the CNCF-blessed path so ecosystem alignment is looser; the
  clustered version's open-source/enterprise split is worth checking; and
  benchmark claims are vendor-published, so verify against your own
  workload.

**How I'd choose:**
- Per-cluster Prometheus (or Agent mode) for local scraping and alerting —
  keep alerting local so it survives losing the global tier. **This is the
  most important architectural decision**: a monitoring system whose
  alerting depends on a central component fails silently when that component
  fails.
- Remote write to a central store for long-term retention and cross-cluster
  queries.
- Thanos if you already have many Prometheus servers and want incremental
  adoption. Mimir if multi-tenancy with hard per-tenant limits is the
  requirement. VictoriaMetrics if operational simplicity and cost are the
  priorities and you can accept the ecosystem tradeoff.
- HA by running **two identical Prometheus servers** per scrape target set
  and deduplicating downstream — the standard pattern, and it's why dedup is
  a first-class feature in all of these.

**Weak answers miss.** Keeping alerting local to the leaf, and that
federation doesn't scale. Both are the practical lessons from running this.

**Follow-ups to expect.**
- What's Prometheus Agent mode? (Scrape and remote-write only, no local
  querying or storage — much lower resource usage for the leaf when the
  central store is authoritative. Note the tradeoff: you lose local
  alerting and local query, so it's for architectures where the central tier
  is genuinely reliable.)
- How do you handle a cluster losing network to the central store?
  (Remote write buffers on disk with a bounded WAL; beyond that you drop
  data. Size the buffer against your worst realistic partition and alert on
  remote write queue depth and failures — those metrics are the ones nobody
  watches until they've lost a day of data.)

---

### A9. Multi-window multi-burn-rate alerting

**Answer.**
**Burn rate** = how fast you're consuming the error budget, relative to the
rate that would exactly exhaust it over the SLO window. Burn rate 1 = you'll
use exactly the budget by the end of the period. Burn rate 14.4 = you'll
exhaust a 30-day budget in about 2 days; burn rate 6 = in 5 days.

**Why "error rate > 1%" is bad**, concretely:
- **It has no notion of significance.** A 1.5% error rate for 30 seconds is
  noise; a 1.5% error rate for 6 hours consumes most of a 99.9% monthly
  budget. The same alert fires for both, so it's either too noisy or too
  slow — you cannot tune one threshold to be both.
- **It's disconnected from the SLO.** The threshold is a number someone
  picked. Nothing ties it to whether users are actually being harmed at a
  rate you've agreed is unacceptable.
- **It misses slow burns entirely.** A 0.5% error rate sustained for a week
  will blow a 99.9% budget completely — and never trips a 1% threshold. This
  is the failure mode that quietly destroys reliability.

**The multi-window multi-burn-rate approach** (the Google SRE workbook
formulation) alerts on burn rate, at several severities, each with two
windows:

| Severity | Burn rate | Long window | Short window | Budget consumed before firing |
| --- | --- | --- | --- | --- |
| Page | 14.4 | 1 hour | 5 min | 2% |
| Page | 6 | 6 hours | 30 min | 5% |
| Ticket | 3 | 1 day | 2 hours | 10% |
| Ticket | 1 | 3 days | 6 hours | 10% |

Two properties make this work:

1. **Multiple burn rates catch both fast and slow failures.** A total
   outage trips the 14.4 rule within minutes. A slow 0.5% degradation never
   trips it but does trip the 3-day rule as a ticket. Each severity is tuned
   to a different failure shape, and together they cover the space.
2. **The short window is the reset condition.** The long window gives
   confidence that the burn is real (not a blip); the short window ensures
   the alert **stops firing quickly once the problem is fixed**. Without it,
   an alert based on a 6-hour window keeps firing for hours after recovery,
   which trains people to ignore it. Requiring *both* windows to be over
   threshold is what gives you good detection time and good reset time
   simultaneously.

In PromQL, each rule is roughly:
```
(
  sum(rate(errors[1h])) / sum(rate(total[1h])) > 14.4 * (1 - 0.999)
and
  sum(rate(errors[5m])) / sum(rate(total[5m])) > 14.4 * (1 - 0.999)
)
```

**The bigger win**: every page is now, by construction, tied to
user-visible harm at a rate you've agreed matters. That's what makes it
possible to say "if this pages, it's worth waking someone" — which is the
only sustainable basis for an on-call rotation.

**Caveats to raise unprompted**: low-traffic services produce noisy ratios
(one error out of ten requests is a 10% error rate) — you need a minimum
traffic gate or you'll page on statistical noise. And burn-rate alerting
tells you the *symptom*; you still need diagnostic dashboards, because the
alert deliberately says nothing about cause.

**Weak answers miss.** The short window as the reset condition — most people
who've read about this remember the multiple burn rates and not why there
are two windows.

**Follow-ups to expect.**
- How do you handle a service with 10 requests/minute? (Aggregate over a
  longer window, alert on absolute error counts, or accept that you can't
  have a meaningful availability SLO and pick a different SLI. Saying "an
  SLO doesn't work here" is the honest answer.)

---

### A10. What makes a good alert

**Answer.**
**The bar**: every page must be **urgent, actionable, and user-impacting**.
If a human can't do anything useful in the next few minutes, it isn't a
page. If it can be fixed tomorrow, it's a ticket. If it needs no human at
all, it should be automation.

**Symptom-based** alerting fires on what users experience: error rate,
latency, availability, "orders per minute has dropped 40% below the
forecast." **Cause-based** fires on internal conditions: disk 85% full, CPU
above 90%, a pod restarting, replication lag above 30 s.

**Page on symptoms; ticket on causes.** The reasons:
- Symptom alerts have **complete coverage** for a bounded number of rules.
  There are countless ways for a service to break and only a few ways for
  users to notice. You cannot enumerate every cause, but you can measure
  every symptom, so a small set of symptom alerts catches failures you never
  anticipated — including the one that will actually happen.
- Cause alerts have terrible precision. High CPU is often fine. A restarting
  pod is often fine. Each cause alert that fires without user impact is
  training the on-call to ignore alerts, and that training is cumulative and
  hard to reverse.
- Cause alerts are *still valuable* — as tickets and as **diagnostic
  context**. "Disk will be full in 4 hours" is an excellent ticket and a
  terrible page. And when a symptom alert fires, the cause signals are what
  you look at next.

The exception where a cause deserves a page: an **imminent, unavoidable,
and irreversible** failure with a lead time short enough to require
immediate action — certificate expiring in 2 hours, disk full in 30 minutes
with no auto-remediation, a quota about to be exceeded. These are pages
because the impact is certain, not because the cause is interesting.

**Every alert also needs:**
- A **runbook link** that says what the alert means, how to confirm it, what
  to check, and what actions are safe. An alert without one is a puzzle
  handed to someone at 3am.
- A **severity** and a routing decision that reflects it.
- An **owner** — a team, not a person.
- To be **tested**: if you've never seen it fire, you don't know it works.

**Measuring the rotation**: track pages per shift, the proportion actionable,
and the proportion that led to a change. A common threshold is that more
than ~2 pages per on-call shift is unsustainable — people stop reading them,
and then a real one gets missed. Alert quality is a service you provide to
your colleagues, and reviewing every page in a weekly ops review (was it
actionable? should it exist?) is the mechanism that keeps it good.

**Weak answers miss.** That alert fatigue is a *safety* problem — noisy
alerts cause missed incidents, so deleting bad alerts is reliability work,
not tidying.

**Follow-ups to expect.**
- What do you do with an alert that fires every week and is always fine?
  (Delete it or fix the underlying condition. There is no third option; an
  alert nobody acts on is worse than no alert because it costs attention.)
- How do you alert on something with no traffic at 3am? (Synthetic probes,
  or don't — a service nobody uses at 3am doesn't need a 3am page. Match
  alerting to when the SLO matters.)

---

### A11. Trace sampling

**Answer.**
**Head-based**: the decision is made at the start of the trace, usually at
the first service, and propagated (a sampled flag in the trace context).
- Cheap and simple: unsampled traces cost nothing beyond the context
  propagation, and no buffering is needed anywhere.
- Consistent by construction — the whole trace is either kept or dropped,
  so you never get partial traces.
- **You cannot sample on outcome**, because you decide before you know it.
  So at 1% sampling you keep 1% of the errors and 1% of the slow requests —
  and those are the only ones you wanted. For a rare failure you may capture
  none at all.
- Mitigations: higher rates for specific endpoints, and a "debug" flag that
  forces sampling for a particular user or request.

**Tail-based**: buffer all spans, wait for the trace to complete, then
decide — keep everything that errored, exceeded a latency threshold, touched
a particular tenant, or hit an unusual code path, plus a random sample of
normal ones.
- You keep the traces that matter. This is a qualitative difference, not an
  efficiency gain.
- Costs: **every span must be transmitted and buffered** until the trace
  completes, so the network and collector cost is the full unsampled volume
  even though the storage cost is low. The collector needs memory
  proportional to (span rate × trace duration), and it needs all spans of one
  trace to reach the **same** collector instance — which means
  trace-ID-aware routing (a load-balancing exporter), a real operational
  constraint. Add a decision timeout, and long traces may be cut off.
- Practically: a two-tier collector setup, and it's substantially more
  infrastructure than head-based.

The pragmatic middle: head-based at a low base rate for baseline coverage,
**plus** forced sampling on error paths (set the sampled flag when an error
occurs, so at least the downstream portion is captured), plus tail-based on
the highest-value services only. And **exemplars** in metrics — attaching a
trace ID to a histogram bucket — give you a cheap path from "the p99 got
worse" to an actual slow trace without sampling everything.

**What tracing can never tell you** — the part of the question people miss:
- **Anything about the requests you didn't sample.** Traces are examples,
  not aggregates. Never compute a rate or a percentile from sampled traces
  and treat it as truth; that's what metrics are for.
- **Why a span was slow internally.** A span says "3 seconds in service B."
  It does not say whether that was GC, lock contention, or CPU. That's
  profiling (A1).
- **The requests that never arrived** — a client that couldn't connect, DNS
  failures, TLS failures. Server-side tracing is blind to them entirely,
  which is the same blind spot as server-side latency metrics (topic 02,
  A17).
- **Anything not instrumented**, and anything where context propagation
  breaks — a message queue, a thread pool handoff, an async boundary, a
  third-party library. Broken propagation produces traces that *look*
  complete and are silently truncated, which is worse than no trace.

**Weak answers miss.** The "tail-based still pays full network cost" point,
and the trace-ID-affinity requirement for collectors.

**Follow-ups to expect.**
- How do you propagate context through Kafka? (Inject W3C `traceparent`
  into message headers and extract on consume. Note the semantic question:
  is the consumer a child span of the producer, or a linked trace? For
  batch consumption, links are more honest than a single parent.)

---

### A12. Log pipeline for a 3000-node fleet

**Answer.**
**Start with the arithmetic**, because it determines the architecture. Say
3000 nodes averaging (conservatively) 500 KB/s of logs each → **1.5 GB/s**,
about 130 TB/day raw. Even at 10x compression that's 13 TB/day stored. A
year of that is unaffordable in a search index. So the design is
fundamentally about **tiering and reduction**, not about collection.

**Architecture:**

```
app (stdout, structured JSON)
  → node agent (Fluent Bit / Vector, DaemonSet)
      · parse, add k8s metadata, drop/redact, sample
  → buffer (Kafka)                    ← decouples, absorbs bursts, replay
  → routing/enrichment
      ├→ hot search index (Elasticsearch / OpenSearch)     7 days
      ├→ warm (searchable snapshots / Loki)             30–90 days
      └→ cold object storage (compressed JSON/Parquet)  1–7 years
```

**Component notes:**
- **Structured logs at the source.** JSON with consistent field names. This
  is the highest-leverage decision: unstructured logs mean grok parsing at
  ingest, which is CPU-expensive at this volume and brittle. Push the cost
  to the producer.
- **Node agent**: Fluent Bit or Vector rather than Logstash — dramatically
  lower per-node resource usage, which matters when multiplied by 3000. It
  must have a **disk buffer** so a backend outage doesn't lose logs or fill
  the node's disk uncontrolled. Watch its own resource usage: a log agent
  that consumes a core per node is 3000 cores.
- **Kafka as the buffer.** Decouples producers from the index, absorbs
  bursts (a crash loop can 100x log volume instantly), allows replay when
  the index breaks or you need to re-index, and lets multiple consumers
  (SIEM, analytics, index) read the same stream independently. At this
  volume it's not optional.
- **Elasticsearch for the hot tier.** ILM policies: hot (fast nodes, local
  NVMe, indexing) → warm (fewer replicas, force-merged, cheaper nodes) →
  cold/frozen (searchable snapshots backed by object storage) → delete.
  Index per day (or per tenant per day) so retention is index deletion, not
  document deletion — deleting documents in Lucene is expensive and doesn't
  reclaim space until merge.
- **Cold tier in object storage**, compressed, partitioned by
  date/service. Queried rarely via Athena/ClickHouse/S3 Select. Cost per TB
  is 1–2 orders of magnitude below the search index.

**Where the money goes**, in order:
1. **The search index.** Elasticsearch stores the original document *plus*
   the inverted index; with replicas, on-disk size can approach or exceed
   raw. It also needs memory and fast disks. This is the dominant cost and
   the reason hot retention should be days, not weeks.
2. **Compute for ingest** — parsing, enrichment, indexing. Indexing is
   CPU-bound and it's why hot nodes are expensive.
3. **Network egress**, especially **cross-AZ** (topic 04, A13). 1.5 GB/s
   crossing AZ boundaries is a very large monthly bill. Keep agents,
   Kafka, and indexers AZ-local where possible; use rack-aware placement.
4. Object storage — comparatively negligible, which is exactly why the cold
   tier exists.

**The reductions that actually pay** — and this is what separates an
answer from a diagram:
- **Sampling repetitive logs.** Successful requests at INFO are the bulk of
  the volume and carry almost no information; keep 1% and all errors. This
  alone often halves the bill.
- **Drop what nobody queries.** Audit which fields and which sources are
  ever searched. Health-check access logs, debug output left on in
  production, and framework startup noise are usually enormous and never
  read.
- **Move counting to metrics.** If you're grepping logs to count something,
  that should be a counter. Log lines emitted only to be aggregated are the
  most expensive metric you can build.
- **Field-level control** — index only the fields you filter on; store the
  rest without indexing (`index: false` in ES mappings). Indexing everything
  is the default and it's the wrong default.
- **Cap per-tenant/per-service ingestion** so one crash-looping service
  logging a stack trace per request can't consume the whole cluster's
  capacity. This is a reliability control as much as a cost one, and it's
  the one people add only after the first incident.
- **Redact at the node**, not centrally — PII that reaches storage is a
  compliance problem that's expensive to unwind.

**Failure modes to design for:** Elasticsearch backpressure → Kafka absorbs
it, and you must alert on consumer lag; disk-full on nodes from unrotated
container logs (a real cause of node NotReady — topic 08, A16); a log storm
from one service (per-source rate limits); and the index cluster becoming a
critical dependency for incident response — which argues for the node agents
degrading gracefully rather than blocking the application.

**Weak answers miss.** The arithmetic, per-source rate limits, and "move
counting to metrics." Also missed: cross-AZ transfer cost, which at this
volume can rival the compute bill.

**Follow-ups to expect.**
- Loki vs Elasticsearch? (Loki indexes only labels and stores compressed
  chunks — far cheaper, and queries are a brute-force scan over a
  label-selected subset. Excellent when you always know the service and time
  range; poor for wide-open full-text search across everything. A defensible
  choice for the warm tier specifically.)
- How do you keep this available during an incident? (It must not depend on
  the systems it monitors. Separate cluster, separate credentials, separate
  region if possible — and a documented fallback of `kubectl logs` /
  node-local access when the pipeline is the thing that's broken.)

---

### A13. Picking an SLI

**Answer.**
The criteria, then an example.

**A good SLI:**
1. **Reflects user experience.** If it improves, users are happier; if it
   degrades, they're unhappier. That's the test.
2. Is a **ratio of good events to valid events**, so it's naturally
   expressed as a percentage and composes with an error budget.
3. Is measured **as close to the user as practical** — at the edge or in the
   client, not deep in the application, because the application can't see
   the requests that never reached it.
4. Has a **clearly defined "valid"** denominator. What counts as a request
   you're responsible for?
5. Is **hard to game** and doesn't improve when the system gets worse.

**Worked example — an API service.**

*Availability SLI*: proportion of valid requests that return a non-5xx
response.

The design decisions that make it right, each of which I'd state:
- **Measured at the load balancer**, not in the app — the app can't count
  requests it never received (connection refused, timeouts, the process
  being down). Server-side application metrics report 100% availability
  during a total outage, which is the classic failure.
- **4xx excluded from the numerator's failures**, because a client sending a
  malformed request is not our failure — but **429 and 503 are counted as
  failures**, because those are us shedding load. That distinction matters
  and is frequently got wrong.
- **Health-check and synthetic traffic excluded** from the denominator, or
  they dilute the ratio: probes are high-volume and always succeed, so they
  make a real outage look smaller than it is.

*Latency SLI*: proportion of valid requests served in under 300 ms —
computed from a **histogram bucket boundary at exactly 300 ms** (A7), not
from an interpolated percentile.
- Threshold chosen from actual user-experience data, not from current
  performance. Setting the SLO at "whatever we do today" makes it
  meaningless.
- Split by **endpoint class**: a search endpoint and a health endpoint have
  different expectations, and one SLI over both is an average of unlike
  things.

**What I'd reject, and why:**
- **CPU utilisation, memory usage, pod restart count, queue depth.** These
  are causes, not symptoms (A10). Nobody has an experience of CPU.
- **Uptime / "the process is running."** A running process serving 500s is
  100% "up."
- **Average latency.** It hides the tail entirely; a service where 5% of
  requests take 10 seconds has a fine average.
- **An SLI computed only over successful requests.** During an outage,
  the failures are fast, so latency *improves* — a latency SLI over
  successes only will look great during your worst hour. Either include
  failures as violations or measure them separately and be explicit.
- **Internal-only measurement** for anything user-facing, per above.

**The last step**: choose *few* SLIs. Two or three per service. An SLO
document with fifteen indicators means nobody knows which one matters and
none of them drives a decision.

**Weak answers miss.** The measurement-point argument, excluding synthetic
traffic from the denominator, and the "errors are fast so latency improves
during an outage" trap.

**Follow-ups to expect.**
- SLI for a data pipeline? (Freshness — proportion of time the data is less
  than N minutes stale — and completeness — proportion of expected records
  present. Not "the job succeeded," which says nothing about correctness.
  See A18.)
- SLI for a batch job? (Proportion of runs that complete within the deadline
  with correct output. The user experience is "was the report there by 9am,"
  so measure that.)

---

## Tier 3 — Scenario / debug

### A14. Green dashboards, broken product

**Answer.**
Name the class of problem first: this is **differential observability** —
the system's view of itself diverges from the users' view (topic 07, A13).
The dashboards aren't lying, they're measuring the wrong thing or measuring
from the wrong place.

**The specific gaps, in the order I'd check them:**

1. **You're measuring the wrong side of the boundary.** Server-side metrics
   count requests that *arrived*. They are structurally blind to: DNS
   failures, TLS handshake failures, connection refused, load balancer
   errors, CDN failures, requests dropped before your app, and network
   problems between the user and you. If the entire failure is upstream of
   your application, every server metric is perfect. **This is the most
   common answer**, and it also covers the mobile-latency blind spot from
   topic 02, A17.
2. **Aggregation is hiding it.** A global success rate of 99.5% can mean one
   region is completely down, one tenant is entirely broken, one API version
   fails, or one client platform can't authenticate. Averages across
   dimensions destroy exactly the signal you need. Slice by region, tenant,
   endpoint, client version, and instance — and note that averaging
   percentiles (A4) makes this worse.
3. **The SLI doesn't capture the failure.** You measure HTTP 200s; the
   endpoint returns 200 with an empty result set, or wrong data, or a
   correctly-formatted error inside a 200 body (common with GraphQL, which
   returns 200 for errors by design). Correctness failures are invisible to
   availability metrics.
4. **Latency measured over successes only** — errors return fast, so a
   worsening error rate *improves* your latency chart (A13).
5. **Synthetic traffic dilutes the denominator.** High-volume health checks
   that always succeed make a real failure a rounding error.
6. **The failure is in a path you don't instrument** — an async job, a
   webhook delivery, an email, a scheduled export, a third-party
   integration. Users experience the product, which is larger than your
   request path.
7. **The monitoring itself is broken** — scrape failures, a stale
   dashboard, an exporter that died and is serving the last value, a metric
   that silently stopped being emitted after a refactor. A missing series
   renders as "no data," which many dashboards display as green or blank.
8. **Time windows too coarse.** A 5-minute average smooths a 90-second total
   outage into a barely-visible dip.

**The immediate move during the incident:** ask the customers what they're
doing, reproduce it yourself from outside your network, and look at raw
recent logs/traces for the affected users. Don't spend the incident
defending the dashboard.

**The structural fixes** — this is what the question is actually asking:

- **Measure from outside.** Synthetic probes from multiple regions and
  networks exercising real user journeys (not just `/health`), plus **real
  user monitoring** in the client — JS/mobile SDK reporting DNS, connect,
  TLS, TTFB, errors, and crashes. Client-side telemetry is the only thing
  that sees failures which never reach you. If I could add one thing to a
  system with this problem, it's this.
- **Build SLIs on user journeys, not components.** "Can a user log in and
  see their dashboard" rather than "the auth service returns 200s."
- **Alert on business metrics.** Orders per minute, signups per hour, events
  ingested — compared against a forecast or a week-over-week baseline. These
  catch *everything*, including failures you never imagined, because they
  measure the outcome rather than the mechanism. Slow to react and noisy on
  seasonality, so they complement rather than replace technical SLIs — but
  in my experience they catch the incidents that nothing else does.
- **Always slice by high-value dimensions**, with per-dimension alerting for
  large tenants.
- **Monitor the monitoring**: alert on absence of data (`absent()`), scrape
  failures, and stale exporters. "No data" must be a loud state, never a
  silent one.
- **Close the loop**: for every incident found by a customer rather than an
  alert, the action item is a detection improvement. Track the proportion of
  incidents detected by monitoring vs reported by users — that ratio is the
  single best measure of observability quality, and it's the number I'd
  report to leadership.

**Weak answers miss.** Client-side/synthetic measurement as the structural
fix, and the "detected-by-monitoring vs reported-by-users" metric.

**Follow-ups to expect.**
- Your synthetic probes are also green. Now what? (The probe doesn't
  exercise the broken path — probes test the journey you thought to write.
  Expand coverage, and consider that the failure may be tenant- or
  data-specific, which no generic probe will find.)

---

### A15. Cardinality explosion during an incident

**Answer.**
The compounding problem: your observability system fails at the moment you
need it, and the *cause* is often the incident itself (a service in a crash
loop emitting a new pod name every few seconds, an error path generating
labels from error messages, a retry storm producing new label values).

**Detection** — you want this before the OOM, not after:
- **Alert on active series count** per Prometheus, and on its **rate of
  change**. A step change is the signature, and a rate-of-change alert gives
  you minutes of warning that a threshold alert doesn't.
- Alert on Prometheus **memory usage** and on `prometheus_tsdb_head_series`.
- Alert on **scrape sample counts per target** — `scrape_samples_scraped`
  jumping for one job identifies the culprit immediately, which is the fastest
  path from "Prometheus is dying" to "this service is why."
- Alert on ingestion rejections / `sample_limit` hits, which are the
  *designed* early warning if you've configured limits.

**Immediate mitigation**, in order of speed:
1. **Identify the offender.** The TSDB status page shows top metrics by
   series count and top label cardinality. `topk(10, count by (job)
   ({__name__=~".+"}))` if the server is still answering. Do this first —
   everything else depends on knowing which job.
2. **Drop the offending series at ingest** with
   `metric_relabel_configs` — a `labeldrop` for the exploding label, or a
   `drop` action on the metric entirely. This is a config reload, not a
   restart, so it's fast and doesn't lose your existing data.
3. **Set or lower `sample_limit`** on that scrape job, so the target is
   rejected wholesale rather than poisoning the server. Blunt but immediate.
4. **Stop scraping the offending job** entirely if it's still growing.
   Losing one service's metrics beats losing all of them.
5. If Prometheus has already OOMed and is crash-looping: **it will replay
   the WAL on start, which takes longer with more series**, so it may not
   come back at all. Options: raise the memory limit temporarily to get it
   up, or — as a last resort — delete the WAL/data to get a working (empty)
   server back for the ongoing incident. Losing history to regain visibility
   is usually the right call mid-incident; say that you'd make it
   deliberately rather than accidentally.
6. **Fall back**: if you run a second Prometheus in an HA pair scraping the
   same targets, it's dying too (same input). The genuine fallback is the
   remote-write store (which has per-tenant limits and horizontal scaling)
   or a separate, minimal Prometheus scraping only critical SLI targets.

**Prevention** — the real answer:
- **`sample_limit` on every scrape job**, sized generously. This converts a
  cluster-wide outage into one broken target. It is the single most valuable
  control and most teams don't set it.
- **`label_limit`, `label_value_length_limit`, `target_limit`** as
  additional guards.
- **Per-tenant series limits** in Mimir/Cortex if you run a multi-tenant
  backend (topic 08, A18) — the platform enforces what teams won't.
- **A separate, minimal "meta" monitoring instance** scraping only the
  critical SLI endpoints and the monitoring system itself, with tight
  limits. It must be able to survive whatever kills the main one — this is
  the thing that gives you eyes during exactly this incident.
- **CI/review gates**: lint metrics for label names matching known-dangerous
  patterns (`*_id`, `user`, `path`, `url`, `email`), and treat adding a
  label as a design decision requiring a bounded-set justification (A5).
- **Reduce churn cardinality**: don't put pod names in metric labels where a
  deployment or service name will do; a crash-looping pod then doesn't
  generate new series.
- Test it: deliberately emit a high-cardinality metric in staging and verify
  the limits catch it. A guard you've never seen trigger is a guess.

**Weak answers miss.** `sample_limit` as the structural prevention, and the
WAL-replay problem that makes recovery slow precisely when you need speed.

**Follow-ups to expect.**
- How would a service accidentally do this? (Label from a URL path with an
  embedded ID; label from an error message; label from a customer-supplied
  header; a new deployment with pod-name labels and a crash loop. All four
  are common, and none looks alarming in code review unless you're looking
  for it.)

---

### A16. First ten minutes on an unfamiliar 3am page

**Answer.**
The governing principle: **mitigate before you diagnose.** Your job is to
restore service, not to understand the bug. Understanding is tomorrow's
work.

**Minute 0–1 — Acknowledge and orient.**
- Ack the page so it stops escalating and so others know it's owned.
- Read the alert itself: what fired, what threshold, what service, and
  **open the runbook link**. If there's a runbook, follow it — it was
  written by someone who understood this better than you do at 3am.
- Establish scope from the alert: is this user-facing? How many users?

**Minute 1–3 — Assess impact and declare.**
- Look at the top-level SLI dashboard: error rate, latency, traffic. Is it
  one service, one region, one tenant, or everything?
- **Decide whether to declare an incident.** If user-facing and non-trivial,
  declare early — it's cheap to close and expensive to escalate late. Open a
  channel, state the impact in one sentence, and name yourself incident
  commander until relieved.
- **Get help.** For a service you've never seen, escalate to the owning team
  *in parallel* with investigating. This is not failure; the fastest
  mitigation usually comes from someone who knows the system. Don't spend
  20 minutes learning the architecture before asking.

**Minute 3–6 — "What changed?"**
This resolves the majority of incidents and should be the first
investigative question, before any theorising:
- **Deploys** — to this service, to its dependencies, to the platform.
  Deploy markers on the dashboards make this a glance.
- **Config and feature flag changes**, which are deploys nobody calls
  deploys and are frequently the answer.
- **Infrastructure changes**: Terraform applies, cluster upgrades, node
  pool changes, certificate rotations, DNS changes.
- **Traffic changes**: a spike, a new large customer, a marketing event, a
  bot.
- **Provider status pages.**
If something changed shortly before the alert, **that's your first
hypothesis and probably your mitigation**: roll it back.

**Minute 6–10 — Mitigate.**
In rough order of preference, because they're ordered by speed and
reversibility:
1. **Roll back** the recent change. Fastest, safest, and you don't need to
   understand why it broke.
2. **Disable the feature flag** or the new code path.
3. **Fail over** — shift traffic to another region/cell/replica.
4. **Scale up**, if it's a capacity symptom.
5. **Shed load / rate limit**, if it's overload — and remember you may need
   to shed below the pre-incident level (topic 07, A16).
6. **Restart** the affected component. Often works, destroys evidence — so
   grab a profile/heap dump/log snapshot first if it takes ten seconds.
Then **verify the mitigation worked** against the user-facing SLI, not
against the internal metric you were staring at.

**Throughout — communicate.**
A status update every 15–30 minutes even when there's nothing new
("investigating, no change, next update at 03:45"). Silence makes people
join the channel and ask, which costs you more time than the updates. State
impact and ETA-to-next-update, never a fix ETA you can't keep.

**What I'd deliberately not do:** read source code, form elaborate theories,
try more than one change at a time (you won't know which worked), or
optimise for finding the root cause. Also: don't work alone on a serious
incident — get a second person for a sanity check even if you don't need
hands.

**Weak answers miss.** "What changed?" as the first investigative step, and
mitigate-before-diagnose as an explicit principle. Also missed: escalating
to the owning team immediately rather than after failing alone for 20
minutes.

**Follow-ups to expect.**
- What if there's no runbook? (Then the first action item afterwards is
  writing one. And in the moment: check dashboards, check recent changes,
  escalate. Say plainly that the absence of a runbook is a defect in the
  alert.)
- When do you wake someone else up? (Immediately, if you're not making
  progress within ~10 minutes on a user-facing incident, or if you need a
  decision you're not empowered to make. The cost of waking someone is
  always less than the cost of a prolonged outage — make that explicit,
  because reluctance to escalate is the most common on-call failure.)

---

### A17. Postmortem for a config-change outage

**Answer.**

**Structure:**

1. **Title and metadata** — date, duration, severity, authors, status.
2. **Impact** — first, and in *user* terms with numbers: "40 minutes of
   total unavailability; approximately 180,000 failed requests; 12 customers
   raised tickets; N% of the monthly error budget consumed." Not "the
   service was degraded." Impact framed as budget consumption connects it
   to the decisions that follow.
3. **Timeline** — timestamped, factual, including detection, escalation,
   each mitigation attempted, and resolution. Include **when the change was
   made** and **when it was detected** — the gap between those two is
   usually the most important number in the document.
4. **Root cause / contributing factors** — plural, deliberately. See below.
5. **What went well** — genuinely useful: it identifies the defences that
   worked and should be protected.
6. **What went poorly / where we got lucky.** "We got lucky" items are the
   highest-value content in most postmortems: near-misses that will become
   incidents.
7. **Action items** — each with an owner, a priority, and a tracking link.
8. **Lessons learned.**

**What separates a good postmortem from a bad one:**

**1. Blameless, and structurally so.** Not "we won't punish anyone" as a
sentiment, but written so the question is never "who." The standard framing:
*given the information, tools, and incentives that person had at that
moment, their action was reasonable* — so the fix is to change the
information, tools, or incentives. If the document reads as though the
outage would not have happened with a more careful engineer, it is a bad
postmortem, and it will make people hide the next one.

**2. Contributing factors, not "the root cause."** For this incident, "a bad
config change" is not a cause, it's the trigger. The real content is:
- Why did the change pass review? What would a reviewer have needed to see
  the problem?
- Why was there no validation that would have rejected it?
- Why did it apply to 100% of production at once, instead of one cell?
- Why did detection take N minutes? What would have caught it in one?
- Why was rollback slow, or why wasn't it obvious that rollback was the
  answer?
- Why was config not treated with the same rigour as code?
Each of those is an independent defence that failed, and each is separately
fixable. A postmortem that stops at "human error, we'll be more careful"
has produced nothing.

**3. Action items that are specific, owned, prioritised, and tracked.**
"Improve testing" is not an action item. "Add a validation webhook that
rejects config with an empty upstream list — @owner, P1, TICKET-123" is.
And the most important discipline: **the action items must actually be
completed**. Track the completion rate as a metric; an organisation with a
30% postmortem action-item completion rate is having the same incident
repeatedly and documenting it each time.

**4. Prioritise action items by *class of failure*, not by this instance.**
The best items generalise: not "validate this config field" but "all config
changes go through the same progressive rollout as code." For this specific
scenario, the highest-value items are almost certainly:
- **Progressive rollout for configuration** — canary to one cell, then
  percentage waves, with automated rollback on SLI regression. Config
  changes bypassing the deploy pipeline is the underlying structural
  problem.
- **Validation at admission** — schema and semantic checks that reject the
  bad state before it applies.
- **Fast, obvious, practised rollback**, with the time-to-rollback measured.
- **Detection**: an alert on the user-facing symptom that fires in under a
  minute at that error rate.

**5. Timeliness and circulation.** Written within days while memory is
fresh, reviewed in a forum where other teams learn from it, and readable by
people who weren't there.

**6. Honest about the things nobody wants to write down** — that the
runbook was wrong, that the alert had been ignored before, that a known risk
was deprioritised. Those are the entries that change behaviour.

**The bad postmortem to avoid**: names an individual; identifies exactly one
root cause; the action item is "be more careful" or "add documentation";
written to satisfy a process rather than to change the system; and never
referenced again.

**Weak answers miss.** Contributing-factors-plural, the "we got lucky"
section, and tracking action-item completion as a metric. Anyone can say
"blameless"; the structural version of it is the signal.

**Follow-ups to expect.**
- Should every incident get a postmortem? (No — define a threshold by
  severity or budget consumption, or the process becomes ritual. But
  near-misses above a certain severity should, because they're free
  lessons.)
- How do you make people write good ones? (A template that asks the right
  questions, a review forum with genuine engagement from senior people, and
  visibly funding the action items. If action items never get prioritised,
  people correctly conclude the exercise is theatre.)

---

### A18. Observability for a 500k events/s pipeline where silent loss is the fear

**Answer.**
The framing that drives everything: **"the job is running" and "the data is
correct" are unrelated statements.** Every component here can be green while
data is being dropped. So the design principle is **end-to-end accounting**
— reconcile counts at every boundary — rather than per-component health.

**1. The primary signal: end-to-end reconciliation.**
Count events at every stage and compare:
```
produced → kafka_in → consumed → transformed → written → queryable
```
Emit a counter at each boundary, tagged by source and by **event time
bucket** (not processing time). Then continuously compute the deltas. A
discrepancy that isn't explained by known filtering is data loss, and it's
the only signal that catches loss *nobody's health check noticed*.

Concretely: a job that, for each 1-minute event-time window, compares
producer count to rows-in-ClickHouse count once the window is closed plus a
grace period, and alerts on a delta beyond a tolerance. This is the single
most valuable thing to build and it is usually the thing nobody builds.

**2. Per-stage signals.**

*Producers:*
- Send success/failure counters, and — critically — **dropped-on-buffer-full
  counters**. A Kafka producer with a full buffer and `block.on.buffer.full`
  disabled drops silently. This is a top silent-loss source.
- Producer retry rate and `record-error-rate`.

*Kafka:*
- **Consumer lag per partition** (not just aggregate) — the primary health
  metric, and per-partition because one stuck partition hides in an
  aggregate.
- **Under-replicated partitions** and **ISR shrink/expand rate** — a
  shrinking ISR means you're one failure from data loss (topic 07, A10).
- Whether `unclean.leader.election` has ever occurred — that's silent loss by
  definition and should be a page.
- **Log-end-offset vs committed offset** per consumer group.
- Retention headroom: **time-to-oldest-message vs retention setting**. If
  consumers fall behind further than retention, data is deleted unread —
  silent loss, and the metric that predicts it is "lag in *time*", not in
  messages.

*Stream processor:*
- Records in / records out / records **dropped**, with an explicit
  categorised drop counter: malformed, schema mismatch, late-arriving beyond
  the window, deliberately filtered. **Every drop must be counted and
  categorised** — an uncounted drop is the definition of silent loss.
- Checkpoint success rate and duration; a processor that can't checkpoint
  will reprocess or lose on restart.
- Watermark lag and late-event counts — for event-time windowing, events
  arriving after the watermark are dropped by default, and this is a very
  common silent loss that looks like nothing.
- Dead-letter queue depth **and rate**, with an alert on any sustained
  non-zero rate. A DLQ nobody watches is a bin.

*ClickHouse:*
- Insert success/failure, rows inserted per second (compare to expected).
- **Parts count per table** and merge backlog — "too many parts" leads to
  rejected inserts (topic 06, A14), which is loss if the producer doesn't
  retry.
- Replication queue depth and `ReplicatedMergeTree` delay.
- Async insert buffer state, if used — buffered data not yet flushed is data
  at risk.
- Disk headroom, with a forecast, not just a threshold.

**3. Data quality, not just data flow.**
- **Freshness**: `now() − max(event_time)` in the destination. This is the
  SLI users actually care about, and it catches a stalled pipeline instantly.
- **Completeness by source**: expected sources reporting vs actual. A source
  that goes silent produces *no* error anywhere — nothing failed, data just
  stopped. Alert on absence per source (`absent()` semantics), with a
  learned baseline for volume so a 90% drop from one source pages even
  though it's still "reporting."
- **Volume anomaly detection** against a week-over-week baseline, since
  seasonality is strong. A 20% drop with no errors anywhere is the classic
  silent-loss signature.
- **Schema/validation failure rates** as a first-class metric, since a
  schema change upstream is a common cause of mass silent drops.
- **Duplicate rate**, since at-least-once delivery plus a dedup mechanism
  that isn't working is the mirror-image failure.

**4. Audit/canary events.**
Inject synthetic events with known identifiers at a known rate at the
producer, and verify they arrive and are queryable, end to end, with the
expected values. This is the only mechanism that tests the *whole* path
including the parts you didn't think to instrument, and it works even when
every component reports healthy. Cheap, and it's what I'd build second after
reconciliation.

**5. What to page on vs ticket:**
- **Page**: freshness SLO breached; reconciliation delta beyond tolerance;
  consumer lag *in time* approaching retention; under-replicated partitions
  sustained; producer drop counter non-zero; canary events missing; a source
  that's been silent beyond its expected interval.
- **Ticket**: parts count trending up, disk forecast, DLQ accumulating
  slowly, merge backlog.
- Explicitly **not** paged: individual task restarts, transient consumer
  rebalances, single insert failures that were retried. These are the noise
  that trains people to ignore the pipeline alerts.

**6. What I'd also do structurally**, because observability alone doesn't
prevent loss: make Kafka retention long enough to survive the worst
realistic downstream outage (so you can replay rather than lose), make the
consumer's commit happen *after* the write succeeds (at-least-once, not
at-most-once), make writes idempotent so replay is safe (topic 07, A8), and
verify that the producer blocks or errors rather than dropping when its
buffer fills. Observability tells you loss happened; those four make loss
recoverable.

**Weak answers miss.** End-to-end reconciliation and canary events —
proposing a dashboard of per-component health metrics is exactly the design
that produces "everything was green and we lost four hours of data." Also
missed: lag measured in *time* against retention, and counting every drop
category explicitly.

**Follow-ups to expect.**
- How do you reconcile counts when the pipeline aggregates? (Count at the
  boundary before aggregation, and separately assert the aggregation's
  invariants — sum of outputs' input-counts equals inputs. Emit the
  contributing record count alongside each aggregate so it's checkable.)
- What's your SLI for this pipeline? (Freshness — "p99 of events are
  queryable within 60 seconds of their event time" — and completeness —
  "99.99% of produced events are queryable within 5 minutes." Two numbers,
  both user-meaningful, both catching failures that component health
  doesn't. A13's principles applied to a pipeline.)
