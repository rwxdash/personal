# Load Balancing & Proxies — Answers

---

## Tier 1 — Recall

### A1. L4 vs L7, NLB vs ALB

**Answer.**
An **L4 load balancer** makes its decision on the TCP/UDP 4-tuple. It does
not parse the payload, so it cannot see HTTP at all. It picks a backend at
connection time and every byte on that connection goes to the same backend
for the connection's lifetime.

An **L7 load balancer** terminates the connection, parses the application
protocol, and makes a decision **per request**. It can route on path, host,
header, method, or cookie; it can retry a failed request against a different
backend; it can rewrite, compress, authenticate, and buffer.

Concretely with AWS:

| | NLB (L4) | ALB (L7) |
| --- | --- | --- |
| Decision granularity | Per connection | Per request |
| Protocols | TCP, UDP, TLS | HTTP/1.1, HTTP/2, gRPC, HTTP/3 (verify current support) |
| Routing | 4-tuple / target group | Host, path, header, method, query, source IP |
| Client IP | **Preserved** by default | Replaced; original in `X-Forwarded-For` |
| Static IP | Yes, one per AZ; supports Elastic IPs | No — DNS name only, IPs change |
| Latency added | Very low (~tens of µs order) | Higher (~single-digit ms order) — it's a full proxy |
| Retries / circuit breaking | No | Yes |
| TLS | Terminate or pass through | Terminates (can re-encrypt to targets) |
| WebSockets / long-lived | Natural fit | Supported, but idle timeouts apply |

**Where each breaks — this is the actual question:**

*NLB breaks* when your traffic is many requests over few connections. With
HTTP/2 or gRPC, one client opens one connection and multiplexes thousands of
requests over it; an L4 balancer pins that whole connection to one backend,
so load distributes by *connection count*, not request count. A handful of
heavy clients will hot-spot single backends while others idle. It also can't
retry a failed request, can't do header-based routing (no canary by header),
and gives you no HTTP-level observability — your LB metrics are bytes and
connections, not status codes and latency per route.

*ALB breaks* when you need a static IP (customer firewall allowlists), a
non-HTTP protocol, extremely low added latency, or very high connection
churn where proxy overhead and per-connection cost dominate. It's also a
place where request buffering can surprise you with slow-client behaviour,
and it's more expensive per unit of traffic.

The pattern worth naming: **NLB in front of a mesh/ingress**, so you get a
static IP and cheap L4 entry, with L7 decisions made by an ingress
controller or Envoy behind it. That's how most Kubernetes-on-AWS setups are
actually built, and saying it shows you've deployed this rather than read
about it.

**Weak answers miss.** The HTTP/2 multiplexing problem — that's the single
best discriminator on this question. Also missed: that "preserves client IP"
on NLB is a genuine architectural difference, not a convenience.

**Follow-ups to expect.**
- gRPC behind an NLB: what happens? (Long-lived H2 connections pinned per
  backend; you get poor balance, and scaling up doesn't rebalance existing
  connections. Fix: L7 balancer that understands H2 streams, client-side LB,
  or periodic server-initiated `GOAWAY` to force reconnects.)
- How does NLB preserve client IP if it's not a proxy? (It rewrites the
  destination, not the source — flow-hash based forwarding with connection
  state, so the backend sees the client's address directly. Note the
  hairpinning gotcha when a target calls the NLB it sits behind.)
- What is the GCP/Azure equivalent? (GCP: passthrough Network LB vs
  Application LB — GCP's global Application LB is anycast-fronted, which is
  a genuinely different model from ALB's regional DNS.)

---

### A2. Load balancing algorithms

**Answer.**
- **Round robin** — even rotation. Right when backends are homogeneous and
  requests are uniform in cost. Wrong the moment either assumption fails,
  which is most of the time: it will happily send a request to a backend
  that's stuck at 100% CPU.
- **Least connections** (or least outstanding requests) — send to the
  backend with the fewest in-flight. Right when request cost varies widely,
  because a slow backend accumulates in-flight requests and naturally sheds
  new ones. This is the best general-purpose default. Watch for the
  "least-connections stampede": a freshly added backend has zero
  connections and receives everything at once (see A16).
- **Weighted / EWMA-latency** — score backends by an exponentially weighted
  moving average of observed latency. Right for heterogeneous fleets (mixed
  instance types, mixed AZ distance) and for gradually shifting away from a
  degrading host. Costs you a feedback loop that can oscillate if the
  smoothing is wrong.
- **Consistent hashing** on a key — same key always goes to the same
  backend. Right when the backend holds state for the key: a cache, a
  session, a per-tenant in-memory index. Costs you: load imbalance follows
  key popularity, so one hot key is one hot backend (see A6).
- **Random / power-of-two-choices** — see A7. Right for distributed
  balancers where no single component sees global state.

The framing that matters: the algorithm is choosing what to optimise —
fairness of *requests*, fairness of *work*, or *affinity*. You cannot have
affinity and even work distribution at the same time; that's the tradeoff to
state.

**Weak answers miss.** Naming the assumption each one relies on. A list
without "right when / wrong when" is a memorised answer.

**Follow-ups to expect.**
- Which would you pick for a fleet where p99 request cost is 100x p50?
  (Least outstanding requests — say why: it's the only one that
  self-corrects without an explicit health signal.)

---

### A3. Connection draining

**Answer.**
When you remove a target — deploy, scale-in, health check failure — there
are two populations to worry about: new connections and in-flight requests.
Draining means the LB stops sending *new* connections to the target
immediately, but allows existing ones to finish for a configured grace
period before forcibly closing.

Without it, a deploy kills in-flight requests. Users see 502s and reset
connections proportional to your deploy frequency, which is exactly the sort
of low-grade error rate that gets normalised and never fixed.

The part people get wrong: **draining only works if the application
cooperates.** The full sequence is
1. Fail the readiness/health check (or call the deregister API) so the LB
   stops sending new traffic.
2. **Keep serving** for a period at least as long as the LB's health-check
   detection interval — because the LB doesn't know yet.
3. Then stop accepting new connections, finish in-flight requests, close
   keepalive connections gracefully (`Connection: close`, or HTTP/2
   `GOAWAY`).
4. Exit.

Step 2 is the one that's skipped, and it's why "we enabled draining and
still see 502s during deploys" is such a common complaint: the process died
the instant it got `SIGTERM`, before the LB had noticed. In Kubernetes the
equivalent is a `preStop` sleep long enough to cover endpoint propagation,
plus `terminationGracePeriodSeconds` longer than your longest request.

Also: the drain timeout must exceed your longest legitimate request. If you
have a 60-second export endpoint and a 30-second drain, you cut those in
half every deploy.

**Weak answers miss.** The "keep serving after failing the health check"
step, and that the whole mechanism is a race between LB detection and
process exit.

**Follow-ups to expect.**
- Long-lived connections (WebSocket, gRPC streams) — how do you drain those?
  (You can't wait them out. Send an application-level "reconnect" signal or
  `GOAWAY`, and make the client reconnect with jitter. Design the client for
  it from the start.)
- What if the pod is being evicted for node pressure and has 5 seconds?
  (You lose. Which is an argument for PDBs and for surge-before-terminate.)

---

### A4. Preserving client IP

**Answer.**
**Behind an L7 proxy**, the proxy makes its own TCP connection to the
backend, so the backend sees the proxy's IP. The original travels in a
header: `X-Forwarded-For` (a comma-separated chain, appended by each hop) or
the standardised `Forwarded` header. Modern practice on AWS also exposes it
in the ALB's access logs and via `X-Forwarded-For`.

The security trap: `X-Forwarded-For` is **client-supplied data**. A client
can send `X-Forwarded-For: 1.2.3.4` and if your app naively reads the first
entry, you've handed it IP spoofing — which breaks IP allowlists, rate
limits, and audit logs. The rule is: only trust the entries appended by
proxies you control. Configure a trusted-proxy count or CIDR list and read
the Nth-from-right entry, not the leftmost. Every mature framework has this
setting and it's misconfigured constantly.

**Behind an L4 balancer** there's no application protocol to put a header
into. Options:
- **The LB doesn't NAT the source at all** — AWS NLB with IP targets
  preserves the client IP natively, and the backend just sees it. This is
  the cleanest option and a real reason to choose NLB.
- **PROXY protocol** — a small plaintext (v1) or binary (v2) header sent
  once at the start of the TCP connection carrying the original 4-tuple,
  before any application bytes. Both sides must agree: a backend expecting
  PROXY protocol that receives a raw connection will mis-parse the request,
  and vice versa. Protocol-agnostic, so it works for anything, not just
  HTTP.
- **Direct Server Return** — responses bypass the LB entirely, so the client
  IP is inherently intact (A12).

**Weak answers miss.** The spoofing hazard and the trusted-hop rule. That's
the part that's a security bug rather than a config detail.

**Follow-ups to expect.**
- Your rate limiter is keyed on `X-Forwarded-For` and someone bypasses it.
  What did you do wrong?
- What happens when you enable PROXY protocol on the LB but not the backend?
  (Every request fails — the first line of the stream isn't valid HTTP.
  There is no graceful rollout, so this is a two-step change: enable
  optional/dual acceptance on the backend first.)

---

### A5. Active vs passive health checks

**Answer.**
**Active**: the balancer periodically probes each backend out-of-band — an
HTTP GET, a TCP connect — and marks it up or down based on consecutive
successes/failures. Predictable, gives you a clear signal, and detects a
fully dead backend even with zero traffic. Costs: it's a synthetic path that
may not resemble real traffic, detection latency is
`interval × unhealthy_threshold` (often 15–30 s), and at fleet scale the
probe traffic itself is non-trivial (N balancers × M backends).

**Passive** (outlier detection / "ejection"): the balancer watches *real*
request outcomes and ejects a backend that produces consecutive failures or
5xx above a threshold, usually for a growing cooldown. Zero extra traffic,
uses the exact path users use, and reacts much faster — within a few
requests rather than a few intervals. Costs: it needs traffic to work
(a backend receiving nothing is never evaluated), it can eject on failures
that were actually the *request's* fault rather than the backend's, and
without a cap it can eject too many at once.

They're complementary, and the standard configuration is both: active checks
for liveness and returning ejected hosts to service, passive detection for
fast reaction to gray failure. Envoy's model — active health checks plus
outlier detection with a `max_ejection_percent` — is the one to describe.

The cap matters: if the *dependency* is broken, every backend fails, and
unlimited ejection removes your entire fleet. `max_ejection_percent` (or the
LB's "minimum healthy" / fail-open behaviour) is the safety valve. AWS target
groups have a similar behaviour — when all targets are unhealthy they route
to all of them rather than none, on the theory that the health check is more
likely wrong than the whole fleet.

**Weak answers miss.** Fail-open, and the ejection cap. "What happens when
everything is unhealthy" is the question behind the question.

**Follow-ups to expect.**
- Detection latency vs flapping — how do you tune the thresholds? (Asymmetric:
  fast to eject, slow to return. And require several consecutive successes
  to re-admit.)

---

## Tier 2 — Explain / compare

### A6. Consistent hashing and bounded loads

**Answer.**
**Why modulo fails**: with `hash(key) % N`, changing N remaps essentially
every key. Going from 10 to 11 backends moves roughly 10/11 of all keys —
for a cache that's a near-total cache miss storm, and for a stateful shard
it's a full data reshuffle. Since N changes on every scale event and every
node failure, modulo is unusable for anything with per-key state.

**Consistent hashing**: map both keys and nodes onto the same circular hash
space (say 0..2³²). A key belongs to the first node clockwise from it. Add or
remove a node and only the keys in that node's arc move — expected **K/N**
keys instead of K. That's the whole point: disruption proportional to the
change, not to the whole keyspace.

With one point per node the arcs are wildly uneven, so real implementations
use **virtual nodes**: each physical node is hashed to many points (100–1000
is typical), which smooths the arc lengths. More vnodes means better balance
and more memory/lookup cost.

**Bounded loads** (Google's "consistent hashing with bounded loads")
addresses what plain consistent hashing still can't fix: **key popularity**.
Even perfectly even arcs don't help if one key gets 30% of traffic — that key
has exactly one home. The algorithm adds a capacity cap: each node may hold
at most `c × average_load` (c slightly above 1, e.g. 1.25). When a key hashes
to a node already at capacity, it walks clockwise to the next node with room.
You get a hard bound on imbalance while keeping the "only K/N keys move"
property on topology changes. The cost is that a key's placement now depends
on current load, so it isn't purely deterministic and you need the load view
to be reasonably fresh.

Where this shows up in practice: cache fleets (the original memcached
motivation), sharded stateful services, Envoy's `ring_hash` and
`maglev` policies, DynamoDB/Cassandra partitioning, and any per-tenant
routing scheme.

**Weak answers miss.** That consistent hashing does **not** solve hot keys —
it solves *rebalancing*. Conflating the two is the most common error, and
bounded loads is the answer to the second problem.

**Follow-ups to expect.**
- Rendezvous (HRW) hashing — how does it compare? (Hash (key, node) for every
  node, pick the max. Same minimal-disruption property, no vnodes or ring to
  maintain, but O(N) per lookup instead of O(log N). Simpler; fine for small
  N.)
- How do you handle a hot key that exceeds one node's capacity entirely?
  (Split it: key + random suffix across R replicas for reads, or a small
  front cache in the client. Both cost consistency or memory — say which.)
- Maglev hashing — what's different? (Builds a fixed-size lookup table for
  O(1) lookups and better balance than a ring, with minimal disruption. Used
  in L4 balancers where per-packet lookup cost matters — see A12.)

---

### A7. Power of two choices

**Answer.**
Pick two backends uniformly at random, query or consult their current load,
send to the lesser-loaded one. That's it.

The result is dramatic: with purely random placement of n items into n bins,
the maximum bin load is Θ(log n / log log n). With two random choices it
drops to Θ(log log n) — an *exponential* improvement from one extra sample.
More choices (d = 3, 4) buy only a constant factor beyond that, so two is
the sweet spot.

**Why it beats round robin**: round robin is blind to actual load. If
requests have variable cost, or backends have variable capacity, round robin
keeps feeding a struggling backend at exactly the same rate as a healthy
one.

**Why it beats true least-connections in a distributed setting** — this is
the real insight. Global least-connections requires a single point that sees
all in-flight requests. In practice you have many independent balancers
(many Envoy sidecars, many LB nodes, many client instances) each with a
partial view. If each independently picks "the least-loaded backend," they
all pick the *same* one simultaneously and stampede it — the herd effect.
The load information is stale by the time everyone acts on it, and the
correlation is what kills you.

Two random choices breaks the correlation: the two samples are independent
per balancer, so no global synchronisation forms, while still avoiding the
worst backends. It's a distributed-systems answer, not just an algorithms
one.

This is why it's the default in real systems — NGINX's `random two
least_conn`, Envoy's `LEAST_REQUEST` policy (which is p2c under the hood),
Finagle, and gRPC client-side balancers.

**Weak answers miss.** The herd effect from stale global state. Reciting the
log log n bound without it is answering an algorithms question when this is
an operations question.

**Follow-ups to expect.**
- What load signal do you compare? (In-flight request count is cheap and
  local to the balancer — no backend cooperation needed. Backend-reported
  CPU/queue depth is better but adds staleness and a feedback channel.)
- How does this interact with a slow backend that's *accepting* but not
  completing? (In-flight count rises, so p2c naturally sheds it — one of the
  main practical benefits over round robin.)

---

### A8. Arguing against a deep `/health` endpoint

**Answer.**
The design couples every backend's liveness to a shared dependency, which
turns a partial degradation into a total outage.

Concretely: the database has a blip — a failover, a brief connection-pool
exhaustion, a slow query saturating it. Every instance's health check fails
**simultaneously**, because they all check the same thing. The load balancer
now believes 100% of the fleet is down and removes it all. Traffic goes to
zero. Then, if the LB fails closed, you have a full outage caused by a
degradation that the fleet could have partially served through — some
requests don't touch the database at all, and cached reads would have
worked.

Worse, it's *self-sustaining*: when the database recovers, all instances go
healthy at once and receive the full restored traffic instantly, which can
re-break it.

There's a second failure: a health check that opens a database connection
consumes a connection. N instances × probe frequency × M balancers can be a
meaningful fraction of your connection pool, and under stress the health
check competes with real traffic for the resource it's checking.

**What I'd do instead.** Separate the two questions the system is asking:

- **Liveness** ("should this process be restarted?") — shallow. Is the
  process responsive, is the event loop running. Nothing about dependencies.
  A dependency failure must *never* restart your process; restarting doesn't
  fix someone else's database and it destroys your warm state.
- **Readiness** ("should this instance receive traffic?") — checks things
  that are *instance-local* and would make this instance worse than its
  peers: still warming up, out of file descriptors, local queue saturated.
  A shared dependency is not instance-local, so it doesn't belong here.
- **Dependency health** — expose it as a *metric and an alert*, not as a
  health check. That's what the information is for: paging a human, not
  removing capacity.

If a dependency is genuinely fatal to all functionality, degrade explicitly:
serve cached or partial responses, return 503 for the routes that need it
while still serving the ones that don't. That's a per-route decision, which
is exactly the granularity a binary health check can't express.

Two guardrails regardless: cap the fraction of the fleet the LB may remove
(`max_ejection_percent`), and make the LB fail open when everything is
"unhealthy."

**Weak answers miss.** The correlated-failure argument — that the problem
is *simultaneity*, not depth per se. Also missed: the liveness/readiness
distinction, and that a dependency check in liveness causes restart storms.

**Follow-ups to expect.**
- Isn't a healthy-looking instance that can't serve worse than no instance?
  (Sometimes — say when: if it returns fast errors it's arguably better,
  because it lets the client fail fast and lets you keep capacity for the
  routes that work. If it returns slow errors it's worse, because it
  consumes the client's deadline. Design for the former.)
- How would you handle a *partial* dependency failure — one shard down?
  (Route-level or key-level readiness; shed only the affected keyspace. Cell
  architecture makes this natural — see the distributed systems topic.)

---

### A9. TLS termination vs passthrough vs re-encryption

**Answer.**
**Termination at the LB**: the LB holds the certificate and private key,
decrypts, and talks plaintext to backends.
- Pro: centralised certificate management and rotation (one place, not N),
  L7 routing and observability become possible, offloads handshake CPU from
  backends, enables WAF/compression/caching at the edge.
- Con: plaintext on the internal network — acceptable only if you actually
  trust that segment, which under any zero-trust posture you don't. The LB
  becomes a high-value key-holding target and a compliance boundary.

**Passthrough**: the LB forwards encrypted bytes without decrypting;
backends terminate.
- Pro: true end-to-end encryption, client certificate/mTLS reaches the
  application, LB never touches key material — often the deciding factor for
  regulated workloads.
- Con: the LB is now blind. No L7 routing (except SNI peeking on the
  ClientHello, which is why SNI-based routing exists), no per-request
  retries, no HTTP metrics, no header manipulation. Certificate management
  is now distributed across every backend.

**Re-encryption** (terminate, inspect, re-encrypt to the backend):
- Pro: full L7 capability *and* encryption on the wire everywhere. This is
  the default choice for most serious deployments and the one to name first.
- Con: two handshakes per request path — CPU and latency — and you still
  have key material at the LB. Certificate management in two places. The
  backend leg's certificate validation is often configured to skip
  verification, which quietly reduces it to "encrypted but unauthenticated"
  — worth calling out because it's extremely common.

Decision rule I'd state: default to re-encryption; use passthrough when the
application must see client certificates or when policy forbids the LB
holding keys; use plain termination only inside a trust boundary you can
defend, and expect to have to justify it.

**Weak answers miss.** SNI-based routing as the middle ground under
passthrough, and the "re-encrypt with verification disabled" anti-pattern.

**Follow-ups to expect.**
- Where does a service mesh fit? (mTLS between sidecars is re-encryption
  formalised, with automated certificate lifecycle — the operational answer
  to "certificate management in two places.")
- How do you rotate a certificate at the LB with zero downtime? (Add the new
  one alongside, shift, remove — and confirm the LB serves per-SNI so both
  can coexist.)

---

### A10. Proxy LB vs client-side LB vs sidecar vs eBPF

**Answer.**
**Dedicated proxy** (hardware LB, ALB, a shared Envoy tier). One place to
configure; language-agnostic; clean operational boundary. Costs: an extra
network hop and its latency, a component that must scale with your traffic,
and a shared failure domain — the proxy tier going down takes everything.

**Client-side LB** (the client library holds the backend list and picks —
gRPC's default, Finagle, Ribbon). No extra hop, so lowest latency; the
client sees real per-backend latency and can do p2c well. Costs: you need a
service-discovery mechanism the client can consume, and every policy change
(retry budgets, load balancing algorithm, timeouts) requires a **library
upgrade across every service in every language**. That last cost is what
kills it in polyglot organisations — it's a deployment problem, not a
technical one.

**Sidecar mesh** (Envoy per pod). Gets client-side LB's no-shared-tier
property and the proxy's language-independence, with policy delivered from a
control plane and updatable without touching applications. Adds mTLS, retry
budgets, circuit breaking, and per-hop telemetry uniformly. Costs: a
container per pod — memory and CPU multiplied by pod count, which at
thousands of pods is real money — added latency on both the outbound and
inbound legs (two proxy traversals per request), and a control plane that is
now a critical dependency with its own failure modes and upgrade risk.

**eBPF datapath** (Cilium replacing kube-proxy; and ambient/L7-optional mesh
designs). Two distinct things worth separating:

1. **Service load balancing in the kernel.** kube-proxy in iptables mode
   builds a linear chain of rules per service; rule evaluation and update
   time grow with service count, and large clusters see multi-second
   `iptables-restore` times and measurable per-packet cost. Cilium replaces
   this with eBPF maps — O(1) lookup, updates that touch one map entry, and
   it can attach at the socket layer so the connection is load-balanced
   *before* it hits the network stack, eliminating per-packet DNAT for
   pod-to-service traffic. It also does DSR and Maglev-style consistent
   hashing for north-south traffic. This is a real, measurable win at scale
   and it's the part of the answer your resume makes you accountable for.
2. **Replacing L7 sidecars.** Partially. eBPF is excellent at L3/L4 and at
   observability; full L7 processing (HTTP parsing, retries, complex policy)
   still needs a userspace proxy — the verifier constrains program
   complexity, there are no unbounded loops, and TLS termination is not a
   kernel job. The ambient/node-proxy designs move that proxy from
   per-pod to per-node, cutting the resource multiplier and one hop, at the
   cost of a weaker isolation boundary — a shared node proxy handles
   multiple workloads' traffic, so a compromise or a crash has a larger blast
   radius than a per-pod sidecar.

The summary line: eBPF removes the *L4* datapath tax and gives you kernel-level
visibility; it doesn't remove the need for an L7 proxy, it changes where that
proxy lives and how many of them you run.

**Weak answers miss.** The library-upgrade problem with client-side LB (the
real reason meshes won), and being honest that eBPF doesn't replace L7
proxying.

**Follow-ups to expect.**
- What does Cilium's socket-level LB break? (Anything expecting to see the
  service VIP on the wire, some NAT-visible tooling, and it interacts with
  network policy ordering — plus host-namespace edge cases. Know that it's
  not free.)
- What's the memory cost of a sidecar fleet? (Do the arithmetic: even a
  modest per-sidecar footprint × several thousand pods is a substantial
  fraction of a cluster. That number is the business case for ambient.)

---

### A11. Sticky sessions

**Answer.**
Binding a client to a specific backend for the duration of a session.
Implementations:
- **Cookie-based** (L7): the LB injects its own cookie naming the target, or
  hashes an application cookie. Survives client IP changes; requires an L7
  balancer and a cookie-capable client.
- **Source IP hash** (L4): cheap and protocol-agnostic. Breaks badly behind
  carrier-grade NAT or a corporate proxy, where thousands of users share one
  address — you get both poor distribution and a large blast radius when
  that backend dies.
- **TLS session ID / SNI-based** for L4 with TLS.

Costs:
1. **Load imbalance.** You've replaced "balance requests" with "balance
   sessions," and sessions have wildly different lifetimes and intensities.
   A new backend gets no traffic until sessions naturally churn.
2. **Deploys become disruptive.** Every restart drops sessions, so state is
   lost or users are logged out. You cannot roll a deployment invisibly.
3. **Autoscaling barely works.** New capacity doesn't relieve existing hot
   backends because the load is pinned.
4. **Failure is user-visible.** Losing one backend loses its users' state,
   rather than shifting load.
5. It hides the real problem, which is that you put state in the wrong
   place.

The right answer to "we need sticky sessions" is usually: externalise the
session (a shared store — Redis, a signed cookie carrying the state, a
database) so any backend can serve any request. Then stickiness becomes a
*performance optimisation* (better local cache hit rate) rather than a
correctness requirement, and you can drop it at any time without breaking
anything. That distinction — stickiness as an optimisation vs as a
dependency — is the whole answer.

Legitimate remaining uses: long-lived stateful connections (WebSocket rooms,
video sessions, an in-memory game state), and cache-affinity routing where
the cost of a miss is high.

**Weak answers miss.** Reframing it as optimisation-vs-requirement, and the
CGNAT problem with source-IP hashing.

**Follow-ups to expect.**
- Signed cookie vs server-side session store: tradeoffs? (Cookie: no shared
  store, scales trivially; but size limits, you can't revoke before expiry,
  and rotating the signing key invalidates everything. Store: revocable,
  unlimited size; but it's a new critical dependency on every request.)

---

### A12. Maglev/Katran-style L4 balancing with ECMP

**Answer.**
The architecture, from the outside in:

1. **Anycast VIP announced via BGP.** A pool of balancer machines each
   announce the same service VIP. Upstream routers see multiple equal-cost
   paths to it.
2. **ECMP** at the router hashes each packet's 5-tuple and picks one
   next hop — i.e. one balancer machine. This is how you scale the balancer
   tier horizontally with no coordination: the router does the first level of
   distribution for free, in hardware, at line rate.
3. **The balancer** hashes the 5-tuple again to select a backend, using a
   consistent-hashing scheme (Maglev's table, or Katran's), and forwards the
   packet — typically encapsulated (IPIP/GRE) or via MAC rewrite.
4. **Direct Server Return**: the backend responds *directly to the client*,
   not back through the balancer. The balancer only sees the request
   direction.

**Why DSR matters**: for typical traffic the response is many times larger
than the request. Taking the balancer out of the response path means it
handles a small fraction of total bytes, so a modest balancer fleet fronts
an enormous amount of throughput. The backend must be configured with the
VIP on a loopback interface (to accept packets addressed to it) while not
ARPing for it.

**Why consistent hashing is essential here** — this is the crux. There are
two independent sources of churn:
- **ECMP rehashing.** When a balancer machine is added, removed, or its BGP
  session flaps, the router's ECMP group changes and packets from an
  *existing* connection can suddenly land on a **different balancer**.
- **Balancer fleet changes** for maintenance or scaling.

If each balancer chose backends by simple hashing or round-robin over a
mutable list, a connection that moves to a different balancer would be sent
to a different backend — which has no TCP state for it and replies `RST`.
Every ECMP change would reset a large fraction of connections.

Consistent hashing makes the backend selection a **pure function of the
5-tuple and the backend set**, identical across all balancer machines. Any
balancer receiving a packet independently computes the same backend, so
ECMP rehashing is harmless. Maglev's specific contribution is a lookup table
that gives near-perfect balance *and* minimal disruption when backends
change, with O(1) per-packet lookup.

The remaining hole: when the *backend* set changes, some connections do get
remapped. Maglev additionally keeps a local connection-tracking table as a
fast path, so established flows are pinned even across backend changes — the
consistent hash is the fallback when the local table doesn't have the entry
(e.g. right after an ECMP move). Belt and braces: the table handles the
common case, the hash handles the machine-change case.

**Weak answers miss.** That there are *two* levels of hashing (ECMP then
balancer) and that consistent hashing exists to make the second level
stateless across the first level's churn. Candidates who only mention
"consistent hashing spreads load evenly" have missed the point entirely.

**Follow-ups to expect.**
- How do you drain a balancer machine? (Withdraw its BGP announcement; ECMP
  stops sending it packets; existing flows relocate to other balancers,
  which compute the same backend. Graceful precisely because of the design
  above.)
- Why not just use stateful connection tracking on every balancer? (State
  must then be replicated or the flow must be pinned, reintroducing the
  coordination you were avoiding. The hash is what makes it shared-nothing.)
- How does this compare to an NLB? (Same family of ideas — flow hashing,
  cross-zone considerations, client IP preservation. Knowing the mechanism
  explains NLB's observable behaviours, e.g. why changing target counts can
  disturb existing flows.)

---

### A13. Rate limiting across a fleet

**Answer.**
**Token bucket**: a bucket of capacity B refills at rate R. Each request
takes a token; empty bucket means reject (or queue). Allows **bursts** up to
B while enforcing average R. This is the right default for APIs — real
clients are bursty and a limiter that forbids bursts is a limiter that
rejects legitimate traffic.

**Leaky bucket**: requests enter a queue that drains at a fixed rate R.
Output is perfectly smooth — no bursts pass through, ever. Right when the
thing you're protecting cannot absorb bursts at all (a fixed-capacity
downstream, a hardware device, a third-party API with a hard rate contract).
Costs latency: requests wait rather than being rejected.

**Fixed window**: count per calendar window, reset at the boundary. Trivial
to implement, but allows **2x the limit** across a boundary — 100 requests at
0:59 and 100 at 1:01 is 200 in two seconds against a 100/min limit.

**Sliding window log**: store a timestamp per request, count those within the
last window. Exact, but memory is O(requests) per key — untenable at scale.
**Sliding window counter** approximates it by weighting the previous
window's count by the fraction of it still in view. Bounded memory, small
bounded error, and it's what most production limiters use.

**Distributing it across 200 edge nodes.** The naive approach — every node
does an `INCR` against a central Redis on every request — gives exact global
limits and is what most people build first. It costs a network round trip on
every request (adding latency to *everything* to enforce a limit that
affects almost nothing) and makes Redis a hard dependency on your request
path with 200 nodes' worth of QPS against it.

Better, in the order I'd consider:

1. **Local buckets with a global budget allocation.** Each node holds a
   local token bucket. A control plane periodically (say every 1–10 s)
   redistributes the global rate across nodes in proportion to the traffic
   each is actually seeing. Enforcement is local and free; the shared path
   is off the request path. This is roughly how large edge platforms do it.
2. **Local enforcement + async aggregation.** Nodes enforce optimistically
   locally and stream counters to a central aggregator that corrects the
   allocation. Same shape, simpler control loop.
3. **Central store with batched/leased tokens.** A node leases a block of N
   tokens from Redis and spends them locally, refetching when low. Cuts the
   central round trips by N× while keeping a global view. Good middle
   ground.
4. **Sticky routing by limit key** so all requests for one tenant land on one
   node, making local state authoritative. Elegant, but reintroduces the
   hot-key and stickiness problems from A6/A11.

The tradeoff to state plainly: **exactness costs latency and availability.**
With local enforcement you will overshoot — worst case up to N × (per-node
allowance) during a redistribution interval — and you must decide whether
your limit is a safety mechanism (approximate is fine; overshoot by 20% is
harmless) or a billing/contractual boundary (needs exactness, and you should
enforce it at a chokepoint rather than at 200 edges). Most rate limits are
the former and are needlessly engineered as the latter.

Also decide the failure mode explicitly: if the central store is unreachable,
do you fail open (allow, risking overload) or closed (reject, guaranteeing an
outage)? For protection limiters, fail open with a conservative local cap.

**Weak answers miss.** The exactness-vs-latency tradeoff and the fail-open
decision. Also missed: that the burst behaviour is the actual difference
between token and leaky bucket, not the implementation.

**Follow-ups to expect.**
- What do you key on? (Tenant/API key, not IP, wherever you can — IP is
  shared by CGNAT and spoofable through XFF. Layered limits: per-key,
  per-IP, and global, with the global one as the last-resort protection.)
- What do you return? (429 with `Retry-After`; and make sure clients honour
  it — otherwise your limiter creates the retry storm from topic 02, A16.)
- How does this interact with autoscaling? (Adding nodes must not multiply
  the effective limit — which is exactly the failure mode of naive per-node
  static limits.)

---

### A14. Circuit breaking, load shedding, backpressure

**Answer.**
Three different controls, distinguished by *who* is protected and *where*
the decision is made.

**Circuit breaking** — the **caller** protects itself and its downstream.
After a threshold of failures/timeouts to a dependency, the caller stops
sending entirely for a cooldown (open), then allows a trickle of probes
(half-open), then closes if they succeed. What it buys: the caller stops
burning its own threads and deadlines on calls that will fail, and — more
importantly — the failing dependency gets the quiet interval it needs to
recover. Without it, a struggling service is kept struggling by the load it
can't serve. The tuning hazard is a breaker that trips on a transient blip
and turns a 1% error rate into a 100% one; thresholds should be based on
sustained rates and there should be partial-open behaviour rather than
all-or-nothing.

**Load shedding** — the **server** protects itself. When the server is over
capacity it rejects work *quickly and cheaply* rather than accepting it and
degrading everything. Two things make it work: the rejection must be far
cheaper than the request (otherwise you shed and still fall over), and it
must be **prioritised** — shed batch and retry traffic before interactive
user traffic, shed by request criticality. The signal to shed on should be
*queue latency*, not CPU: CoDel-style "how long has the oldest item been
waiting" is a much better overload indicator than utilisation, because it
directly measures whether you're keeping up. A server without shedding
doesn't stay up longer; it fails in a worse way — with everyone's timeouts
expiring after the work was already done.

**Backpressure** — the **system** propagates the constraint upstream so the
producer slows down, rather than anyone dropping anything. TCP's receive
window is backpressure. A bounded queue that blocks the producer is
backpressure. gRPC/HTTP-2 stream flow control is backpressure. The key
property: no work is lost, the pressure travels to the source, and the source
decides what to do with it. It only works when the producer *can* slow down —
which is true for a pipeline (Kafka consumer, batch job) and false for
inbound user traffic, where you have no control over the source and must
shed instead.

The way to organise the answer: **backpressure where the producer is
controllable, shedding where it isn't, circuit breaking at every call site
regardless.** And all three need to exist — they cover different points of
the graph, and a system with only one of them fails at the others.

**Weak answers miss.** That backpressure is unavailable for uncontrollable
producers, and that shedding should key on queue delay rather than CPU.

**Follow-ups to expect.**
- What's an unbounded queue and why is it the worst option? (It converts a
  throughput problem into a latency problem plus an OOM, and every item you
  eventually serve is past its deadline anyway. Bounded queues with rejection
  are strictly better — this is Little's law applied: queue depth ÷ service
  rate is your added latency.)
- How do these interact with retries? (Badly, if uncoordinated: a breaker
  that opens while clients retry hard just moves the load. Retry budgets
  and shedding must be designed together.)

---

## Tier 3 — Scenario / debug

### A15. Sporadic 502s, worse on idle connections

**Answer.**
This is the **idle timeout mismatch**, and the diagnostic detail is right
there in the question: worse on idle connections, and the backend never sees
the request.

The mechanism. The LB maintains a pool of keepalive connections to the
backend. Both the LB and the backend have an idle timeout. If the
**backend's** idle timeout is *shorter* than the **LB's**, then after a
period of inactivity the backend closes the connection — sends `FIN`. If a
request arrives at the LB in the window between the backend deciding to close
and the LB processing the `FIN`, the LB writes the request onto a connection
that is already gone. It gets a `RST` (or an already-closed socket), has no
response, and synthesises a 502.

The backend logs nothing because from its point of view the connection was
closed cleanly and no request ever arrived. That asymmetry — LB error,
backend silence — is the fingerprint.

**The fix**: make the backend's keepalive/idle timeout **strictly greater**
than the load balancer's. The idle-connection close should always be
initiated by the LB, which knows whether a request is in flight. Rule of
thumb: backend idle timeout ≥ LB idle timeout + a few seconds of margin.
This is a documented, recurring problem for ALB in front of Node.js
(`server.keepAliveTimeout` historically defaulted to 5 s, ALB's idle timeout
to 60 s), Go's `IdleTimeout`, nginx `keepalive_timeout`, and gunicorn — the
exact defaults vary by version, so check yours rather than assuming.

Related mechanisms worth ruling out, and how to tell them apart:
- **Backend closing due to `max_requests`/`MaxConnsPerChild`** (gunicorn,
  php-fpm recycling workers) — same race, but it correlates with request
  count rather than idle time.
- **Deploy/scale-in without draining** (A3) — correlates with deploys, not
  idleness.
- **Request or header size limits** at the LB — correlates with payload, and
  the backend also sees nothing.
- **Backend crash mid-request** — but then the backend logs *something*, and
  you'd see restarts.
- Real 502 from an application-layer error would appear in backend logs.

**What I'd actually check**, in order: (1) compare the two idle timeout
settings — this takes 60 seconds and is the answer most of the time;
(2) correlate 502 timestamps against connection age, which your LB access
logs can give you; (3) `tcpdump` on a backend to catch a `FIN` from the
backend immediately followed by an inbound request on the same 4-tuple —
that's conclusive proof.

**Weak answers miss.** Which side should close. Candidates often propose
"increase the backend timeout" as a guess without articulating the ordering
invariant, which means they'll get it backwards next time. Say the invariant:
*the client of a keepalive connection must time out before the server does.*

**Follow-ups to expect.**
- Why doesn't a retry fix it? (It does mitigate it — an idempotent request
  retried on a fresh connection succeeds. Which is a good argument for
  enabling LB-level retries on connection-establishment failures
  specifically, since those are unambiguously safe. But it's masking, not
  fixing.)
- Same problem one layer down: your backend to the database. Same rule?
  (Yes — connection pool max-idle must be below the database's/proxy's
  timeout. This is why "first query after lunch fails" is a classic.)

---

### A16. Adding capacity makes latency worse

**Answer.**
Several mechanisms, and they compound. Worth naming that the general shape
is: *new capacity is not equivalent capacity*, and balancers that assume it
is will actively harm you.

1. **Least-connections stampede.** A new backend has zero in-flight
   requests, so a least-connections or least-request balancer sends it
   *everything* until its count catches up. A cold instance receives a burst
   far above its share at the exact moment it's least able to serve it.
   Fix: **slow start / warm-up** — ramp the new target's weight from 0 to
   full over a configured period (ALB has this, Envoy and NGINX Plus have
   it). Also p2c (A7) is less pathological here than strict least-conn.
2. **Cold caches.** The new instance has an empty local cache, empty
   connection pools, and cold JIT/page cache. Its per-request cost is
   materially higher for the first seconds-to-minutes. If it's a cache tier,
   worse: adding a node *moves* keys to it (A6), so it also causes misses on
   traffic that was previously hitting warm nodes elsewhere. Net effect,
   briefly, is a fleet-wide hit-rate drop.
3. **Downstream connection storm.** N new instances each open a full
   connection pool to the database/cache. If your pool is 50 connections and
   you add 40 instances, that's 2000 new connections arriving at once — which
   can push the database past its connection limit or cause a latency spike
   for everyone. This is the one people forget, and it's why "scale out the
   app tier" sometimes takes down the data tier.
4. **JIT/runtime warmup** (JVM especially): interpreted code for the first
   thousands of invocations, then compilation pauses. Minutes, not seconds.
5. **DNS/discovery propagation skew** — some balancers see the new targets
   before others, so distribution is briefly uneven in a way no single
   balancer can detect.
6. **Autoscaling oscillation.** If the scale-up signal is a lagging metric
   (average CPU over 5 minutes), you over-provision, then scale in, then the
   load returns. Each cycle repeats the cold-start cost.

**Prevention**, in the order I'd implement it:
- Slow start / warm-up ramp on the LB. Cheapest fix, biggest effect.
- Readiness probes that actually mean ready: pre-warm caches and connection
  pools *before* passing readiness, not after. This is the difference
  between readiness meaning "the process started" and "this instance can
  serve at parity."
- Cap the connection pool size and use pooling proxies (pgbouncer et al.) so
  app-tier scale-out doesn't linearly multiply database connections.
- Scale earlier and in smaller increments — a 20% step at the moment of
  crisis is worse than 5% steps starting earlier. Predictive/scheduled
  scaling for known patterns.
- Watch the right metric: scale on a leading indicator (queue depth,
  concurrency, RPS per instance) rather than a lagging one (CPU average).

**Weak answers miss.** The downstream connection storm and the cache-tier
key-movement effect. Those two are what separate someone who has scaled a
real system from someone reasoning about it.

**Follow-ups to expect.**
- How long should the warm-up ramp be? (Long enough to cover the slowest
  warm-up component — usually cache fill or JIT. Measure it: plot per-request
  latency by instance age. If you can't answer this from data, that's the
  first thing to build.)
- What if the spike is faster than your warm-up? (Then you need headroom, not
  faster scaling. Say it plainly: autoscaling is not a response to a
  sub-minute spike; over-provisioning and shedding are.)

---

### A17. Global API request path with 30-second regional failover

**Answer.**
Walk the layers and name where the failover decision lives at each one. The
30-second requirement is the constraint that eliminates most options, and I'd
say so first: **DNS cannot meet a 30-second RTO** (topic 02, A12), so the
failover decision cannot live in DNS as the primary mechanism.

**Layer 1 — Entry: anycast VIP at edge PoPs.**
Announce one anycast prefix from PoPs worldwide, including Singapore. The
user's packets reach the Singapore PoP because BGP routes them there — no
DNS decision, no client state. TLS terminates at the PoP, which saves the
user 3 setup RTTs against a us-east-1 origin (topic 02, A17).
*Failover decision here*: withdraw the BGP announcement from a failed PoP.
Propagation is seconds to low tens of seconds within a region's upstreams.
Cost: in-flight connections at that PoP are cut, so clients must retry;
acceptable for an API with idempotent retries, and it's the fastest lever
available.

**Layer 2 — Edge to origin: the PoP chooses the backend region.**
This is where the 30-second requirement is actually met. The PoP holds warm,
pooled connections to two or more origin regions and health-checks them
continuously (active checks every few seconds plus passive outlier
detection). When us-east-1 fails, the PoP routes to the secondary region on
the next request — detection in a few seconds, failover in zero additional
round trips because the connections are already warm.
*Failover decision here*: at the edge, per request, based on origin health.
This is the right place for it: the decision-maker is close to the user,
sees real request outcomes, and has no cache to invalidate. Nothing on the
client needs to change or expire.

**Layer 3 — Origin region internals.** Regional LB (NLB for static entry
→ ingress/Envoy for L7), backends across ≥3 AZs, health checking and
draining as in A3/A5.

**Layer 4 — Data.** The layer that actually determines whether 30 seconds is
achievable, and the honest answer is: it depends on what the API writes.
- Read-only or read-mostly: trivially met with cross-region read replicas.
- Writes with a single-region primary: your failover time is the *database's*
  failover time, which is typically longer than 30 s if it's safe, and less
  than durable if it's fast. If the requirement is truly 30 s for writes,
  either you accept potential data loss (async replication, RPO > 0) or you
  need a multi-region-writable store (Spanner-class, DynamoDB global tables
  with last-writer-wins semantics, or per-region ownership of key ranges) —
  and each has real consistency costs.
- I would push back on the requirement here and ask whether 30 s applies to
  reads, writes, or both, because the answer changes the entire data
  architecture and its cost.

**Belt and braces — DNS as the slow backstop.** Keep health-checked DNS
failover with a 30–60 s TTL. It won't meet the RTO, but it covers the case
where the anycast/edge layer itself is the thing that failed, and it costs
nothing to have.

**What I'd explicitly not do**: client-side region selection as the primary
mechanism. It works (and is the lowest-latency option) but requires shipping
client changes to fix routing bugs, which is the slowest control loop you can
choose for an availability mechanism.

**Where the failover decisions live, summarised:**

| Failure | Decided by | Mechanism | Time |
| --- | --- | --- | --- |
| Single backend | Regional LB | Health check + ejection | seconds |
| AZ | Regional LB | Cross-AZ targets | seconds |
| Origin region | Edge PoP | Origin health check → alternate region | ~seconds, meets 30 s |
| Edge PoP | BGP / anycast | Withdraw announcement | seconds–tens of seconds |
| Everything | DNS | Health-checked record change | minutes (backstop) |

**Weak answers miss.** Putting the failover decision at the edge rather than
in DNS, and interrogating whether the 30 s applies to writes. A candidate who
designs the request path beautifully and never mentions that the database is
the binding constraint has answered the easy half.

**Follow-ups to expect.**
- How do you test this? (Regular game days that actually fail a region —
  a failover path that isn't exercised doesn't work. Say that you'd measure
  the achieved RTO, not assume the configured one.)
- What's the steady-state cost of the standby region? (Warm standby means
  paying for capacity you don't use, or accepting scale-up time during
  failover — which may blow the 30 s budget. Active-active costs more but is
  the only configuration that's continuously proven.)
