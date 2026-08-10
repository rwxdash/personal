# Kubernetes & Containers — Answers

---

## Tier 1 — Recall

### A1. What happens on `kubectl apply`

**Answer.**
1. **kubectl** resolves the manifest, computes a patch (three-way merge
   against the last-applied annotation, or server-side apply), and POSTs/
   PATCHes to the **API server**.
2. **API server**: authenticates (certs, tokens, OIDC), authorises (RBAC),
   runs **admission** — mutating webhooks and defaulting first (this is
   where a sidecar injector or a defaulting policy modifies the object),
   then validation, then validating webhooks and any policy engine. Then it
   **persists to etcd**. The API server is the only component that talks to
   etcd.
3. **Deployment controller** (in kube-controller-manager) watches
   Deployments, sees the new spec, and creates or updates a **ReplicaSet**
   with a hash of the pod template in its name.
4. **ReplicaSet controller** sees a replica count it hasn't satisfied and
   creates **Pod** objects — with `nodeName` empty.
5. **Scheduler** watches for unscheduled pods. For each: **filter**
   (predicates — does the node have enough allocatable resources, does it
   match node selectors/affinity, are taints tolerated, are volumes
   attachable) then **score** (priorities — spread across zones, image
   locality, least/most allocated) and picks the highest. It writes a
   **binding**, which sets `nodeName`.
6. **kubelet** on that node is watching for pods bound to itself. It:
   - calls the **CRI** (containerd/CRI-O) to pull images and create the
     sandbox (the "pause" container that owns the network and IPC
     namespaces),
   - calls the **CNI** plugin to set up networking — create a veth pair,
     move one end into the pod netns, assign an IP, program routes,
   - calls the **CSI** driver to attach/mount volumes,
   - starts init containers in order, then app containers,
   - begins running probes and reporting status back to the API server.
7. **kube-proxy** (or the eBPF equivalent) on every node watches Endpoints/
   EndpointSlices and programs the datapath so Service traffic reaches the
   new pod once it's Ready.

**The framing that matters**: this is not a pipeline, it's a set of
independent **controllers running level-triggered reconciliation loops**
against declarative state in etcd. Nobody calls anybody. Each controller
watches the API server, compares desired to observed, and acts. That's why
the system is resilient — a controller that was down for ten minutes catches
up by observing current state, not by replaying missed events — and it's
why "why isn't my pod running" is answered by asking *which loop is stuck*.

**Weak answers miss.** The level-triggered/reconciliation point, and
admission webhooks (which are a common source of "apply hangs" and of
cluster-wide outages when a webhook's backing service is down and its
failure policy is `Fail`).

**Follow-ups to expect.**
- What if a mutating webhook's service is down? (With `failurePolicy: Fail`,
  every matching create/update is rejected — potentially cluster-wide. This
  is one of the most common self-inflicted total outages in Kubernetes; scope
  webhooks narrowly with `namespaceSelector` and exclude `kube-system`.)
- Where does the pod IP come from? (The CNI's IPAM — per-node CIDR block, or
  a cluster-wide pool, or the cloud VPC's address space depending on the
  plugin.)

---

### A2. Requests, limits, QoS

**Answer.**
**Request** = what the scheduler reserves. It's the number used for bin
packing and it's the guaranteed floor for CPU (via `cpu.weight`/shares).
**Limit** = the enforced ceiling at runtime (`memory.max`, CFS quota).

| QoS class | Condition | Consequence |
| --- | --- | --- |
| **Guaranteed** | Every container has requests == limits for both CPU and memory | Last to be evicted under node pressure; eligible for exclusive CPUs under the static CPU manager policy |
| **Burstable** | At least one request set, but not equal to limits | Evicted after BestEffort, ordered by how far usage exceeds requests |
| **BestEffort** | No requests or limits at all | Evicted first |

The behavioural differences that actually matter:

- **Memory is incompressible.** Exceeding the memory limit means the cgroup
  OOM killer kills the container — `SIGKILL`, no graceful shutdown, no
  draining (topic 05, A7). You cannot "throttle" memory.
- **CPU is compressible.** Exceeding the CPU limit means **throttling**, not
  killing — and throttling is what wrecks tail latency at low average
  utilisation (topic 05, A8). This is why the standard advice is: **always
  set memory requests == limits, and consider omitting CPU limits** for
  latency-sensitive services while keeping CPU requests.
- Requests drive **scheduling**; limits drive **runtime enforcement**. A
  node can be 100% requested and 20% used (waste), or 40% requested and
  overloaded (if limits >> requests and everyone bursts).
- Eviction under node pressure uses QoS class *and* usage-over-request, not
  limits.

**Weak answers miss.** The compressible/incompressible distinction, which
explains every other behaviour here.

**Follow-ups to expect.**
- How do you pick request values? (From observed usage — p95-ish for CPU,
  peak plus headroom for memory. VPA in recommendation mode is the practical
  tool. Say that guessing produces either waste or evictions, and that
  reviewing them is ongoing work, not a one-time task.)
- What is `Allocatable` vs `Capacity`? (Capacity minus kube-reserved,
  system-reserved, and eviction thresholds. Pods are scheduled against
  Allocatable, and forgetting the reservation is how you get a node that
  OOMs the kubelet itself.)

---

### A3. Probe types

**Answer.**
- **Liveness** — "is this container wedged?" On failure the kubelet
  **restarts the container**. Should test only that the process is
  responsive and internally functional. It must never test a dependency,
  because restarting your process cannot fix someone else's database (A6).
- **Readiness** — "should this pod receive traffic?" On failure the pod is
  **removed from Service endpoints**; the container keeps running. This is
  the one for temporary, recoverable conditions: warming up, overloaded,
  local queue full, draining before shutdown.
- **Startup** — "has this container finished starting?" While it's failing,
  liveness and readiness probes are **suppressed**. It exists for slow-
  starting applications (a JVM loading a large cache, a database replaying a
  log). Before startup probes, you had to set a liveness
  `initialDelaySeconds` long enough for the worst-case start, which meant
  your liveness detection was that slow forever. A startup probe with a
  generous `failureThreshold` lets you have a slow start *and* a fast
  liveness check afterwards.

The distinction that matters in one line: **readiness controls traffic;
liveness controls the process's life.** Getting them backwards — a readiness
check that should be liveness, or vice versa — is the most common probe
misconfiguration after A6.

Also worth knowing: readiness failure removes the pod from endpoints but
does **not** stop existing connections, which is why graceful shutdown still
needs a `preStop` hook (topic 03, A3).

**Weak answers miss.** Startup probes entirely, and that readiness affects
endpoints rather than the container.

**Follow-ups to expect.**
- What if a readiness probe flaps? (Endpoints churn, connections are
  redistributed repeatedly, and load balancers see constant topology change.
  Use `failureThreshold`/`successThreshold` asymmetrically — quick to remove,
  slow to re-add.)

---

### A4. What a Service is, and the packet's journey

**Answer.**
A Service is **not a process**. It is an API object that (a) allocates a
stable virtual IP from the service CIDR, (b) defines a label selector, and
(c) causes the endpoints controller to maintain an **EndpointSlice** listing
the ready pod IPs. There is no proxy listening on the ClusterIP; nothing
answers ARP for it.

A packet sent to a ClusterIP, in the classic iptables mode:

1. The pod sends to `10.96.0.10:80`. The route sends it toward the node's
   networking stack.
2. In the **`nat` PREROUTING/OUTPUT** chain, kube-proxy has installed rules:
   a `KUBE-SERVICES` chain matches the ClusterIP+port and jumps to a
   per-service chain, which uses `statistic --mode random --probability` to
   pick one of the endpoint chains — that's the load balancing, and it's
   **random per connection**, not round robin.
3. The chosen endpoint chain **DNATs** the destination to the pod IP and
   port.
4. **conntrack** records the translation so return packets are un-DNATed
   consistently for the life of the connection.
5. The packet is routed to the pod, locally via the veth or across nodes via
   the CNI's mechanism (overlay encapsulation or a native route).

Consequences of this design worth stating:
- Load balancing is **per connection**, in the kernel, with no health
  awareness beyond readiness. Long-lived connections (HTTP/2, gRPC,
  database pools) pin to one backend — the same problem as an L4 load
  balancer (topic 03, A1).
- It depends on **conntrack**, so connection churn consumes conntrack table
  entries and can exhaust `nf_conntrack_max` (topic 05, A17). There's also a
  well-known UDP/conntrack race that causes intermittent DNS failures.
- The ClusterIP is **not routable outside the cluster** — it only exists as
  iptables rules on nodes.
- `headless` Services (`clusterIP: None`) skip all of this: DNS returns the
  pod IPs directly, which is what StatefulSets and client-side load
  balancing use.

**Weak answers miss.** That there's no proxy process in the data path and
that balancing is random-per-connection via iptables probability rules.

**Follow-ups to expect.**
- How does NodePort/LoadBalancer differ? (Additional rules on a node port;
  `externalTrafficPolicy: Local` preserves the client source IP and avoids a
  second hop but means only nodes with a pod get traffic — and requires the
  external LB's health checks to reflect that.)
- Why do gRPC clients need special handling? (One long-lived H2 connection
  pins to one pod; scale-out doesn't rebalance. Use a headless service with
  client-side LB, or an L7 proxy/mesh.)

---

### A5. PodDisruptionBudget

**Answer.**
A PDB declares how much of a workload may be *voluntarily* disrupted at
once: `minAvailable: 80%` or `maxUnavailable: 1`. The **Eviction API**
(`/eviction`) checks the PDB and refuses an eviction that would violate it.

The critical distinction: PDBs only protect against **voluntary
disruptions** — things that go through the eviction API:
- `kubectl drain` (node maintenance, upgrades)
- Cluster Autoscaler / Karpenter scaling down a node
- The descheduler rebalancing pods

They do **not** protect against **involuntary** disruptions:
- Node hardware failure, kernel panic, or the VM being terminated
- Node OOM or the kubelet evicting for resource pressure
- Preemption of a lower-priority pod by a higher-priority one (which uses
  the eviction path but is *permitted* to violate PDBs)
- The pod being deleted directly, or the container crashing
- A spot/preemptible instance being reclaimed (though some providers give a
  drain signal, which then does respect PDBs)

So a PDB is a **maintenance-safety** mechanism, not an availability
guarantee. Getting this wrong leads teams to believe a PDB gives them an
SLO, which it doesn't — availability under involuntary disruption comes from
replica count, zone spread (topology spread constraints), and the app's
tolerance for losing an instance.

The operational trap: a **too-strict PDB blocks node drains indefinitely**.
`minAvailable: 100%`, or a PDB on a single-replica Deployment, means
`kubectl drain` hangs forever and cluster upgrades stall. This is one of the
most common causes of a stuck cluster upgrade, and the fix is to require
that every PDB permits at least one disruption.

**Weak answers miss.** The voluntary/involuntary distinction — it's the
whole question.

**Follow-ups to expect.**
- What's a sensible PDB for a 3-replica service? (`maxUnavailable: 1`.
  Expressing it as `maxUnavailable` rather than `minAvailable` means it
  stays correct as you scale, which is the better habit.)
- How do PDBs interact with a cluster upgrade? (Node-by-node drain, each
  waiting on PDBs — so total upgrade time is a function of your PDBs and
  your pods' termination grace periods. Worth computing before you start.)

---

## Tier 2 — Explain / compare

### A6. Liveness probe checking a database

**Answer.**
The mechanism, precisely:

1. The database has a blip — a failover, a connection storm, a slow query
   saturating it, a brief network partition.
2. **Every replica's liveness probe fails simultaneously**, because they all
   check the same shared thing. This is correlated failure by construction.
3. The kubelet on each node restarts every container.
4. Restarting doesn't fix the database. The probe fails again after the
   restart. `CrashLoopBackOff` begins, and the backoff grows exponentially
   (10s, 20s, 40s… capped at 5 minutes).
5. Meanwhile, each restarting pod **reconnects to the database on startup** —
   opening a fresh connection pool. So the recovering database is hit with a
   connection storm from the entire fleet, repeatedly, which prevents it from
   recovering (topic 03, A16 and topic 07, A16 — this is a metastable loop).
6. Even after the database is healthy, the fleet is spread across long
   backoff intervals, so recovery is slow and staggered. You have a total
   outage of a service that could have served cached reads and
   database-independent endpoints throughout.

The compounding factor: you also lost all in-process state — warm caches,
JIT compilation, connection pools — across the entire fleet at once, so even
when things recover, you're in the cold-start scenario from topic 03, A16.

**What to do instead:**
- **Liveness**: shallow and local only. "Is the process responsive?" — an
  HTTP handler that returns 200 if the event loop is running. Ideally it
  should only fail for conditions a restart actually fixes: deadlock,
  unrecoverable internal state.
- **Readiness**: instance-local conditions that make *this pod* worse than
  its peers — warming up, local queue saturated, out of file descriptors.
  A shared dependency is not instance-local. If every pod would fail
  readiness for the same reason, readiness is the wrong mechanism, because
  removing 100% of endpoints means the Service has nowhere to send traffic.
- **Dependency health**: a **metric and an alert**, not a probe. That's what
  the signal is for — telling a human, not removing capacity.
- **Degrade explicitly**: serve cached responses, return 503 only for the
  routes that need the dependency, keep serving the ones that don't.

If a pod genuinely cannot function at all without a dependency, the least-bad
option is readiness with the understanding that the Service will empty —
plus a caller-side circuit breaker so callers fail fast instead of timing
out.

**Weak answers miss.** The reconnection storm that prevents recovery, and
that the failure is *correlated*, which is what turns a degradation into an
outage.

**Follow-ups to expect.**
- Is there ever a case for a dependency check in liveness? (Essentially no.
  A defensible edge case is a client library with a known unrecoverable
  connection state that only a restart clears — and the right fix is the
  library, not the probe.)
- How would you catch this before production? (A chaos test that takes the
  database offline for 60 seconds and asserts the fleet doesn't restart. Say
  that this is a testable property.)

---

### A7. kube-proxy: iptables vs IPVS vs eBPF

**Answer.**
**iptables mode** (the long-time default). kube-proxy writes a chain per
service and per endpoint into netfilter's `nat` table.
- Matching is a **linear traversal of rules**. With S services and E
  endpoints each, the chain count is O(S×E) and packet matching cost grows
  with it. At a few thousand services the per-packet cost is measurable.
- The bigger problem is **update cost**. Historically kube-proxy rewrote
  large portions of the ruleset via `iptables-restore`, taking a global
  netfilter lock. At scale (thousands of services, high endpoint churn) this
  takes seconds — during which the datapath is stale and rule updates queue.
  A rolling deploy of a large service can cause cluster-wide dataplane update
  latency. Later versions added partial/incremental sync, which helps a lot,
  but the model's cost curve is unchanged.
- Load balancing is random per connection via probability rules; no health
  awareness beyond readiness.

**IPVS mode**. Uses the kernel's L4 load balancer (built on netfilter's
connection tracking but with hash tables rather than rule chains).
- **O(1) lookup** regardless of service count, and updates touch a single
  entry rather than rewriting chains. Scales to tens of thousands of
  services cleanly.
- Real scheduling algorithms: round robin, least connections, source
  hashing, shortest expected delay — meaningfully better than random.
- Still uses conntrack, still needs iptables for some functions (masquerade,
  NodePort edge cases), so it's a partial escape rather than a clean break.
  Adoption has been limited by edge-case bugs and the fact that it's still
  a per-packet NAT path.

**eBPF datapath (Cilium)**. Replaces kube-proxy entirely.
- Service→backend mapping lives in **eBPF maps**: O(1) hash lookup, and an
  update is a single map write with no global lock. Update cost is
  independent of cluster size.
- **Socket-level load balancing**: for pod-to-service traffic, Cilium attaches
  at the socket layer (`connect()`/`sendmsg` cgroup hooks) and rewrites the
  destination **before the packet is created**. There's no per-packet DNAT
  and no conntrack entry for the service translation at all — which removes
  both the per-packet cost and the conntrack table pressure.
- **No conntrack dependency** for that path, which eliminates a class of
  failures including the UDP DNAT race behind intermittent DNS timeouts.
- North-south: XDP-based load balancing with Maglev consistent hashing and
  DSR (topic 03, A12), so backend changes don't reset unrelated connections.
- Also gives you identity-based network policy, L7 visibility, and the
  observability layer (Hubble) as a side effect of already being in the
  datapath.

**What breaks / costs in each:**
- iptables: rule count and sync time at scale; conntrack exhaustion;
  debugging via `iptables-save` output that is thousands of lines long.
- IPVS: fewer people run it, so you're on a less-travelled path; still
  conntrack-bound; some feature edge cases lag.
- eBPF: requires a modern kernel with the right features (a real constraint
  on older bare-metal fleets); socket-LB means the service VIP is never on
  the wire, which confuses tooling and some legacy assumptions about seeing
  the VIP; debugging requires new tools (`cilium monitor` rather than
  tcpdump alone); and it's a large, fast-moving component that owns your
  entire dataplane — upgrades are consequential.

**Weak answers miss.** That the dominant iptables problem is **update/sync
time**, not per-packet lookup, and the socket-level LB detail for eBPF.
Given Cilium is on your resume, expect this to go deeper.

**Follow-ups to expect.**
- How would you measure the problem before switching? (kube-proxy sync
  duration metrics, `iptables-save | wc -l`, conntrack utilisation, and
  endpoint-propagation latency measured end to end — the time from a pod
  becoming Ready to it receiving traffic on every node.)
- What did Cilium's kube-proxy replacement break for you in practice?
  (Have a real answer — host-network access to services, NodePort edge
  cases, and interaction with other iptables-based tooling are the usual
  candidates.)

---

### A8. Overlay vs native routing

**Answer.**
**Overlay (VXLAN, Geneve, IP-in-IP)**: pod traffic is encapsulated in an
outer packet addressed node-to-node. The underlying network only ever sees
node IPs and knows nothing about pod addressing.
- Pro: works on **any** network. No cooperation from the physical fabric, no
  BGP peering, no route injection, no coordination with a network team. Pod
  CIDRs can overlap with the underlay's address space. This is why it's the
  default for most CNIs — it works everywhere.
- Con: **encapsulation overhead** — VXLAN adds 50 bytes, so pod MTU must be
  node MTU minus that (1450 on a 1500 underlay). Get this wrong and you get
  the exact PMTUD black hole from topic 01, A9 / topic 04, A7: small packets
  work, large ones vanish. It's the single most common overlay bug.
- Con: **CPU cost** — encap/decap per packet, and it historically defeated
  NIC offloads (checksum, TSO/GRO), though VXLAN offload is now common. Real
  throughput penalty on high-bandwidth workloads.
- Con: **opacity**. The physical network can't see pod addresses, so
  firewalls, flow logs, and ECMP hashing all operate on node-to-node tunnels.
  Debugging requires decapsulating, and per-flow load balancing across the
  fabric is degraded because many pod flows share one tunnel 5-tuple (unless
  the encapsulation varies the source port, which modern implementations do).

**Native routing**: pod IPs are real, routable addresses in the underlay.
Packets go out unencapsulated.
- Pro: **no overhead, full MTU, full offload**, line-rate performance.
- Pro: the network sees real pod IPs, so flow logs, firewalls, and
  monitoring are meaningful, and ECMP hashes per pod flow.
- Pro: pods are directly reachable from outside the cluster, which some
  architectures want.
- Con: the underlay must **know the routes**. Either the CNI peers with the
  fabric via **BGP** (Calico, Cilium BGP), or the cloud programs routes
  (AWS VPC CNI hands pods real VPC IPs from ENIs; GKE uses alias IP ranges).
  That means coordination with the network — a political as well as a
  technical dependency on bare metal.
- Con: **address space consumption**. Pod IPs come from routable space, so a
  large cluster consumes a lot of it. AWS VPC CNI's per-node ENI/IP limits
  are a hard pod-density constraint, and IP exhaustion in a shared VPC is a
  real and common problem.
- Con: in AWS, native mode means each node's pod capacity is bounded by
  instance type ENI limits — a scheduling constraint that surprises people.

**How I'd choose:** on a cloud with a native-mode CNI (VPC CNI, GKE), use
native unless address space is scarce — you get performance and observability
for free. On bare metal where you control the fabric and can run BGP, native
routing with BGP is excellent (and is what Calico/Cilium do well). On
anything where you can't influence the network, overlay, and **get the MTU
right on day one**.

**Weak answers miss.** The MTU arithmetic and the address-space cost of
native mode. Also missed: that "overlay is slower" is now mostly about
offload and MTU rather than raw encap CPU.

**Follow-ups to expect.**
- What MTU do you set for VXLAN on a 9000-byte jumbo underlay? (8950. And
  verify every hop supports jumbo — one 1500 hop reintroduces the problem.)
- Cilium can do both — when would you pick each? (Same answer; note that
  Cilium's native routing needs either BGP or a fabric that already routes
  the pod CIDRs, and its `tunnel: disabled` mode assumes nodes are on the
  same L2 or the routes exist.)

---

### A9. etcd

**Answer.**
**What it does**: etcd is the cluster's only persistent state. Every object
— pods, services, secrets, configmaps, leases — lives there. It's a
Raft-replicated (topic 07, A5) consistent key-value store, and it provides
two things Kubernetes depends on absolutely: **linearizable reads/writes**
(so the API server can do compare-and-swap on `resourceVersion` for optimistic
concurrency) and **watches** (so every controller can be notified of changes
rather than polling).

Only the API server talks to it. Everything else goes through the API server.

**How it fails:**

1. **Disk latency.** etcd fsyncs every Raft log entry before acknowledging.
   Its performance is bounded by **fsync latency**, and it is extremely
   sensitive — the documented guidance is a p99 backend commit and WAL fsync
   in the low milliseconds. Put etcd on network storage with variable
   latency, or on a disk shared with something else, and the whole control
   plane becomes slow and leader elections start flapping. **Dedicated
   local SSD/NVMe for etcd is not optional.** This is the number one cause
   of etcd problems in the field.
2. **Database size.** There's a default 2 GB quota (`--quota-backend-bytes`,
   commonly raised to 8 GB; larger is discouraged). Exceeding it puts etcd
   into a **read-only alarm state (NOSPACE)** — the cluster stops accepting
   writes, and clearing it requires compaction, defragmentation, and
   explicitly disarming the alarm. Causes: too many objects, large objects
   (ConfigMaps/Secrets used as data stores), high churn without compaction,
   and Events (which is why Events have a short TTL and often a separate
   etcd).
3. **Compaction and defragmentation.** etcd keeps every revision until
   compacted (auto-compaction is configurable); compaction frees revisions
   but doesn't return disk space — **defrag** does, and defrag **blocks the
   member** while it runs. So defrag must be done one member at a time,
   deliberately.
4. **Quorum loss.** With 3 members, losing 2 means no quorum: no writes, no
   leader. Recovery is from a snapshot, and it's a genuine restore
   operation, not a restart.
5. **Watch/range load from the API server.** A controller doing expensive
   list operations (a `LIST` of all pods every few seconds) generates
   enormous load. This is why well-behaved controllers use informers with
   watches, and why a badly-written custom controller can destabilise a
   whole cluster.
6. **Network latency between members.** Raft commits require a round trip to
   a majority, so etcd members must be close — same region, ideally same
   low-latency network. Stretching etcd across regions is a common and bad
   idea.

**What happens when etcd is unavailable** — the important part:

- **Running workloads keep running.** Kubelets continue managing the
  containers they already know about; kube-proxy keeps the datapath it has
  programmed; pods serve traffic. The **data plane survives a control plane
  outage**, and this is the single most important architectural property to
  state.
- **Nothing can change.** No new pods scheduled, no deployments, no scaling,
  no Service endpoint updates. Which means: if a pod dies, nothing replaces
  it; if a pod becomes unready, it is **not removed from Service endpoints**,
  so traffic keeps going to it. Degradation is gradual, not immediate — and
  it gets worse the longer it lasts.
- `kubectl` fails for writes; reads may work briefly from the API server's
  cache/watch cache, then fail.
- Anything depending on **leases** breaks: leader election for controllers,
  node heartbeats (so nodes eventually go `NotReady`, which after the outage
  ends triggers mass eviction — a nasty secondary effect).

**Weak answers miss.** That the data plane keeps serving, and the fsync
sensitivity. Also missed: that endpoints stop updating, so a control plane
outage silently degrades load balancing.

**Follow-ups to expect.**
- How do you back up and restore? (`etcdctl snapshot save` on a schedule,
  stored off-cluster, and — the part that matters — **tested restores**. A
  snapshot you've never restored is a hypothesis. Note the restore rewrites
  member identity, so it's a documented procedure, not a file copy.)
- Why 3 or 5 members and not 7? (Every commit needs a majority round trip;
  more members means more latency and more disk fsyncs for no additional
  practical fault tolerance. 3 for most, 5 when you want to tolerate 2
  failures or do rolling maintenance without losing headroom.)
- How would you reduce etcd load in a big cluster? (Separate etcd for
  Events; limit object counts and sizes; audit controllers for LIST-heavy
  behaviour; enable API priority and fairness to stop one client starving
  others.)

---

### A10. HPA, VPA, Cluster Autoscaler, Karpenter

**Answer.**
- **HPA** — horizontal pod autoscaler. Changes **replica count** based on a
  metric (CPU/memory utilisation against requests, or custom/external
  metrics). The right tool for stateless services that scale with load.
- **VPA** — vertical pod autoscaler. Changes **requests and limits** for a
  pod. Historically required a pod restart to apply (in-place resize is a
  newer capability — verify support for your version). Best used in
  *recommendation* mode as a right-sizing tool, because automatic vertical
  scaling of a live service is disruptive.
- **Cluster Autoscaler** — changes **node count** by scaling node groups
  (ASGs/MIGs) when pods are unschedulable, and scaling in when nodes are
  underutilised and their pods can be moved. Constrained to the node groups
  you predefined.
- **Karpenter** — provisions **individual nodes** directly from the pending
  pods' actual requirements: it picks instance type, size, zone, and
  capacity type (spot/on-demand) to fit the pending pods. Faster (no ASG
  round trip), better bin packing, and it consolidates by actively replacing
  underutilised nodes with cheaper ones. Costs: it's much more dynamic, so
  nodes churn, which is disruptive to workloads that don't handle it (and
  requires solid PDBs and graceful shutdown); and it's AWS-centric in
  practice.

**Combining them**: HPA + Cluster Autoscaler/Karpenter is the standard pair
— HPA adds pods, the node autoscaler adds room for them. **HPA and VPA on
the same metric conflict** (VPA raises requests, which lowers utilisation
against requests, which makes HPA scale down) — don't do it; use VPA on
memory and HPA on a different signal, or VPA in recommendation mode only.

**Why HPA on CPU is usually the wrong signal:**

1. **It's a lagging indicator.** CPU rises *after* the queue has built and
   latency has degraded. By the time you scale, add a node, pull an image,
   and warm up, you're minutes past the point where users noticed. Autoscaling
   is not a response to a sub-minute spike (topic 03, A16).
2. **Many services aren't CPU-bound.** An I/O-bound service waiting on a
   database has flat CPU while its latency triples. CPU tells you nothing
   about whether it needs more replicas.
3. **CFS throttling corrupts the metric** (topic 05, A8). A throttled
   container's utilisation-against-request can look moderate while it's
   badly latency-degraded. You scale late, or not at all.
4. **Utilisation is measured against *requests*, not capacity**, so a
   badly-set request makes the target meaningless — halve the request and
   the same workload reports double the utilisation.
5. **It doesn't map to user experience.** Nobody has an SLO on CPU.

**Better signals**, in rough order of preference: **concurrency / in-flight
requests per pod** (directly related to latency via Little's law — topic 07,
A11), **queue depth** for consumers (Kafka consumer lag is the canonical
one, and it's a genuine leading indicator), **requests per second per pod**
when service time is stable, and **latency SLI** as a last resort (it's the
thing you care about but it's the most lagging of all).

Plus: **predictive/scheduled scaling** for known patterns (daily peaks,
campaign launches), and headroom for anything faster than your scale-up
time. State the scale-up latency explicitly — metric window + HPA sync +
node provisioning + image pull + warm-up is often 2–5 minutes, and no signal
choice fixes that.

**Weak answers miss.** The lagging-indicator argument and the
throttling-corrupts-the-metric interaction. Also missed: that HPA and VPA
conflict.

**Follow-ups to expect.**
- How do you autoscale a Kafka consumer? (Consumer lag via KEDA. Note that
  replicas beyond the partition count do nothing, which caps horizontal
  scaling at the partition count — a design constraint that must be set at
  topic creation.)
- How do you avoid flapping? (Stabilisation windows, asymmetric scale-up/
  scale-down policies — fast up, slow down — and a sensible min replica
  count.)

---

### A11. Is a namespace a security boundary?

**Answer.**
**No.** A namespace is an **organisational and policy scope**, not an
isolation mechanism. It gives you a name for grouping objects, a target for
RBAC rules, a scope for ResourceQuotas and NetworkPolicies, and a DNS
subdomain. It does not, by itself, isolate anything at runtime.

What crosses namespaces freely by default:
- **The network.** Pod-to-pod traffic is unrestricted across namespaces
  unless a NetworkPolicy says otherwise — and NetworkPolicy is default-allow
  until a policy selects a pod. So with no policies, every pod can reach
  every other pod in the cluster.
- **The node.** Pods from different namespaces run on the same kernel, share
  the same node resources, and can contend for CPU, memory, disk, PIDs, and
  network. A noisy neighbour in another namespace degrades you.
- **The kernel.** A container escape (a kernel vulnerability, a privileged
  container, a hostPath mount) compromises the node and therefore every pod
  on it, regardless of namespace.
- **Cluster-scoped resources**: nodes, PersistentVolumes, CRDs,
  ClusterRoles, webhooks, StorageClasses. RBAC on these is cluster-wide.

**What actually provides isolation**, layered:

| Concern | Mechanism |
| --- | --- |
| API access | RBAC, scoped tightly; separate ServiceAccounts per workload; no wildcard verbs |
| Network | NetworkPolicy with **default-deny** ingress and egress per namespace, then explicit allows. This is the single highest-value control and it must be default-deny to mean anything |
| Resource contention | ResourceQuota + LimitRange per namespace; requests/limits per pod; priority classes |
| Kernel attack surface | Pod Security Admission (`restricted` profile), seccomp, AppArmor/SELinux, dropping capabilities, no privileged containers, no hostPath/hostNetwork/hostPID |
| Container escape | **User namespaces** (rootless), and for real isolation, **sandboxed runtimes** — gVisor (userspace kernel) or Kata Containers (lightweight VM per pod) |
| Hard isolation | **Separate node pools** with taints, or **separate clusters** |
| Supply chain | Image provenance, admission policy on registries and signatures |

**The decision rule I'd give:** namespaces are sufficient for separating
*cooperating* teams inside one trust domain — teams that could, in principle,
be trusted not to attack each other, where the goal is preventing accidents
and enabling policy. They are **not** sufficient for hostile multi-tenancy —
untrusted customer code, or a regulatory boundary. For those, the answer is
separate clusters, or sandboxed runtimes with dedicated nodes, and you should
be honest that the industry's consensus is "Kubernetes is not a hard
multi-tenancy boundary."

**Weak answers miss.** That NetworkPolicy is default-allow until a policy
exists, and naming sandboxed runtimes as the answer for hostile tenancy.

**Follow-ups to expect.**
- What's the cost of gVisor/Kata? (gVisor: syscall interception in userspace
  — meaningful overhead for syscall-heavy workloads, and incomplete syscall
  coverage breaks some applications. Kata: a VM per pod — higher memory
  floor and slower start, but a genuine hardware isolation boundary. Both
  are real tradeoffs, not free wins.)
- How would you prevent a team consuming the whole cluster? (ResourceQuota
  per namespace, priority classes with preemption so critical workloads win,
  and node pools with taints for workloads that need guaranteed capacity.)

---

### A12. Stateful workloads on Kubernetes

**Answer.**
**What a StatefulSet actually gives you** — and it's less than people
assume:
1. **Stable, ordinal network identity**: `kafka-0`, `kafka-1`, resolvable
   via a headless Service, surviving reschedule.
2. **Stable storage**: each ordinal gets its own PVC, and a rescheduled pod
   reattaches the *same* volume.
3. **Ordered, sequential operations**: pods created 0→N, deleted N→0, and
   rolling updates one at a time waiting for Ready — unless you set
   `podManagementPolicy: Parallel`.

That's it. It is a *naming and volume-affinity* primitive. **It knows nothing
about your data.** It will not: understand quorum, avoid restarting the
Kafka broker that currently holds the only in-sync replica for a partition,
wait for a rebalance to finish, know that ClickHouse needs its replicas
consistent, or back anything up.

**What makes these workloads hard:**

1. **Rescheduling is data movement.** A stateless pod moving to another node
   is free. A ClickHouse replica moving means either the volume follows it
   (network storage — slower, and zone-locked) or terabytes get re-replicated
   (local disk). The scheduler doesn't know or care about that cost, so the
   default behaviours (bin packing, descheduling, node consolidation by
   Karpenter) are actively harmful.
2. **Local storage vs network storage.** These workloads want local NVMe for
   latency and throughput (topic 05, A12). But local PVs are **pinned to a
   node** — if the node dies, the data is gone and the pod is unschedulable
   until you intervene. Network storage (EBS/PD) survives node loss and is
   zone-locked, at 2–10x the latency. There's no good answer; you're picking
   a failure mode. For Kafka and ClickHouse, which already replicate
   internally, local disks plus application-level replication is usually
   right — and then you must **not** let the platform reschedule casually.
3. **Rolling updates need application awareness.** Restarting brokers one at
   a time isn't enough: you must wait for under-replicated partitions to
   reach zero before touching the next one. StatefulSet's readiness gate
   doesn't know that. This is precisely what **operators** exist for
   (Strimzi, the ClickHouse operator, ECK) — they encode "safe to proceed"
   as domain logic. Running these workloads on Kubernetes without an operator
   is choosing to hand-hold every upgrade.
4. **Node lifecycle fights you.** Node autoscaling, spot reclamation, kernel
   upgrades, and automated node rotation all assume pods are movable.
   Mitigations: PDBs that reflect quorum (`maxUnavailable: 1` on a 3-node
   quorum), taints and dedicated node pools, disabling consolidation for
   those nodes, and long termination grace periods.
5. **Resource behaviour is hostile to the defaults.** Memory limits plus a
   JVM (Elasticsearch) or a page-cache-dependent engine (ClickHouse,
   Kafka) interact badly: page cache is charged to the cgroup, so a
   cache-hungry process approaches its memory limit and gets OOMKilled for
   doing exactly what it should. CPU limits cause throttling on latency-
   critical paths. These need Guaranteed QoS, generous memory, and usually
   no CPU limit.
6. **Ordering and PVC lifecycle.** Deleting a StatefulSet doesn't delete its
   PVCs (usually what you want, occasionally a surprise). Scaling down
   leaves orphaned PVCs. Scaling *up* a quorum system without telling the
   application is a rebalance, not a scale.
7. **Backups are still yours.** Volume snapshots are crash-consistent, not
   application-consistent. You need application-level backup (Kafka's data
   is the log; ClickHouse has `BACKUP`; ES has snapshot repositories) and a
   *tested* restore.

**Is it worth it?** My view: yes when you already have a strong Kubernetes
platform and a good operator exists, because you get consistent deployment,
monitoring, and RBAC for free. No when you have neither — a bare
StatefulSet for Kafka on network storage is worse than three VMs, and the
failure modes will find you during an incident.

**Weak answers miss.** That StatefulSet doesn't understand quorum, and the
page-cache-vs-memory-limit interaction. Given ClickHouse/Kafka/Elasticsearch
are all on your resume, expect a specific war story to be requested.

**Follow-ups to expect.**
- How do you upgrade a 3-node quorum safely? (`maxUnavailable: 1` PDB,
  operator-gated readiness that checks under-replicated partitions, one at a
  time, verify between each. And never drain two nodes at once — which is
  what a naive cluster upgrade does.)
- Local PV and a node dies — what's the recovery? (Delete the PVC and pod,
  let a new replica be built from peers. That must be a *documented,
  practised* procedure, and it must be safe to do under pressure.)

---

### A13. Rolling updates and zero-error deploys

**Answer.**
The Deployment controller creates a new ReplicaSet and shifts replicas
between old and new, bounded by:
- **`maxSurge`** — how many pods above the desired count may exist during
  the update (default 25%). Higher = faster, needs more capacity.
- **`maxUnavailable`** — how many below desired may be unavailable (default
  25%). `maxUnavailable: 0` with `maxSurge: 1` means new capacity always
  arrives before old is removed — the safest, slowest configuration.

A pod counts as available only after it's Ready for `minReadySeconds`, so
that field is your guard against "Ready but not actually warm."

**Getting to zero errors requires four things, and the deployment
configuration is only one of them:**

1. **Readiness that means ready.** Not "the process started" — the pod must
   be able to serve at parity: caches warmed, connection pools established,
   JIT settled. If readiness passes early, you route traffic to a cold pod
   and see latency spikes and timeouts (topic 03, A16). Add
   `minReadySeconds` and a startup probe.
2. **Graceful shutdown, properly sequenced.** This is where almost all
   deploy errors actually come from. On termination the kubelet does two
   things **concurrently**: sends `SIGTERM` to the container, and removes the
   pod from EndpointSlices. Endpoint removal then has to propagate to every
   kube-proxy/CNI on every node and to every load balancer — which takes
   time. So there is a window where the pod is shutting down and **still
   receiving new connections**.
   The fix is a **`preStop` hook that sleeps** (5–15 seconds, sized to your
   propagation time) so the container keeps serving while endpoints
   propagate, *then* the application handles `SIGTERM` by draining: stop
   accepting new work, finish in-flight requests, close keepalive
   connections gracefully, exit. `terminationGracePeriodSeconds` must exceed
   preStop sleep + longest in-flight request. (Topic 03, A3.)
3. **The application must handle `SIGTERM`** — and must be PID 1 or behind an
   init that forwards signals (topic 05, A4). A container that ignores
   `SIGTERM` gets `SIGKILL`ed after the grace period, dropping every
   in-flight request, on every deploy.
4. **Client-side retries on connection failures**, which are unambiguously
   safe to retry (the request never reached the server). This covers the
   residual race that no amount of sequencing eliminates.

Plus: a **PDB** so that node drains during the deploy don't compound the
unavailability, and enough replicas that `maxUnavailable` is a small
fraction.

**Beyond rolling**: a rolling update is a poor *risk* control because it
exposes real users to the new version immediately and there's no gate. For
risk, layer on **canary** (a small percentage of traffic to the new version,
with automated analysis of error rate and latency before proceeding — Argo
Rollouts/Flagger) or **blue-green** (full parallel environment, flip
traffic, instant rollback at the cost of double capacity). Rolling controls
*capacity* during the change; canary controls *blast radius*. Say that
distinction — it's the point most candidates miss.

**Weak answers miss.** The concurrent SIGTERM/endpoint-removal race and the
`preStop` sleep. That's the actual cause of "we get 502s on every deploy,"
and it's not fixable by tuning maxSurge.

**Follow-ups to expect.**
- How do you roll back? (`kubectl rollout undo` — but it only reverts the
  pod template. Schema changes, feature flags, and anything with a data
  migration are not covered, which is why expand/contract migrations matter
  — topic 06, A15.)
- What about long-lived connections? (WebSockets/gRPC streams won't drain in
  30 seconds. You need application-level reconnect signalling (`GOAWAY`) and
  clients that reconnect with jitter — designed in from the start.)

---

### A14. Ingress vs Gateway API vs service mesh

**Answer.**
**Ingress** — the original L7 north-south API. Host/path routing to
Services, TLS termination. Its problems, which are why it's being replaced:
- **Underspecified.** It expresses so little that every controller added
  annotations for the real functionality — timeouts, rewrites, canary
  weights, auth, rate limits. So an Ingress manifest is not portable
  between nginx-ingress, Traefik, and an ALB controller; you're writing
  vendor config in an annotation.
- **No role separation.** One object holds the listener/TLS config (a
  platform concern) and the routing rules (an app-team concern), so RBAC
  can't split them cleanly.
- **HTTP only** in practice.

**Gateway API** — the successor, and a genuine redesign rather than a v2:
- **Role-oriented resource split**: `GatewayClass` (infrastructure provider),
  `Gateway` (the platform team's listener, ports, TLS), `HTTPRoute`/
  `GRPCRoute`/`TCPRoute` (the app team's routing). RBAC now maps to who owns
  what, and route attachment is explicitly permitted by the Gateway. This is
  the main point.
- **Expressive core spec**: header matching, traffic splitting by weight
  (so canaries are first-class, not an annotation), request mirroring,
  filters, and cross-namespace routing with an explicit `ReferenceGrant`
  handshake.
- Protocol coverage beyond HTTP.
- Portable across implementations, which was the original promise Ingress
  broke.

**Service mesh** — solves a different problem: **east-west**, service to
service. Adds, uniformly and without application changes: mTLS between
workloads (identity, not IP — topic 02, A10), retries with budgets, timeouts,
circuit breaking, outlier detection, traffic splitting for canaries at the
service level, and per-hop golden-signal telemetry.
Costs: a sidecar (or node proxy) per workload — CPU and memory multiplied by
pod count, plus latency on both legs — a control plane that becomes a
critical dependency, and a large operational surface. Ambient/sidecar-less
modes trade some isolation for the resource cost (topic 03, A10).

**What each adds that the previous doesn't:**
- Ingress → Gateway API: *governance and expressiveness* for north-south.
  Same job, done properly.
- Gateway API → mesh: *east-west*. An ingress of any kind sees traffic
  entering the cluster; it has no visibility into or control over the
  service-to-service calls that follow. The mesh is where zero-trust
  identity and per-hop resilience policy live.

Note the convergence: Gateway API now has a mesh-oriented profile (GAMMA),
and mesh implementations increasingly use Gateway API resources for their
ingress. So the right framing is "one config API, two traffic directions"
rather than three separate technologies.

**When you don't need a mesh** — worth saying, because it's the honest
answer for most teams: if you have a small number of services, consistent
libraries, and no zero-trust requirement, a mesh is a large tax for
capabilities you can get from a good RPC library. The strongest justifications
are polyglot environments (where you can't ship a library — topic 03, A10),
a mandated mTLS/zero-trust posture, and uniform telemetry across teams you
don't control.

**Weak answers miss.** The role-separation motivation for Gateway API, and
that mesh solves a different axis rather than being "a better ingress."

**Follow-ups to expect.**
- Would you adopt Gateway API today? (Yes for new clusters — it's GA for the
  core resources, controller support is broad. Migrate incrementally; both
  can coexist. Verify the maturity of the specific features you need, since
  some are still experimental.)

---

## Tier 3 — Scenario / debug

### A15. Pending, Terminating, CrashLoopBackOff

**Answer.**
The general method: `kubectl describe` on the object first — Events are the
single highest-information source and they name the failing loop directly.
Then work out **which controller is stuck**.

**`Pending` — nothing has bound this pod to a node.**
Two sub-cases, distinguished by whether `nodeName` is set:
- *Unscheduled.* `kubectl describe pod` shows the scheduler's rejection with
  per-node reasons ("0/12 nodes are available: 5 Insufficient cpu, 4 node(s)
  had untolerated taint, 3 node(s) didn't match pod affinity"). That message
  *is* the answer; read it carefully. Causes: insufficient allocatable
  resources (check requests against node Allocatable, not Capacity);
  taints without tolerations; node selectors or affinity that match nothing;
  topology spread constraints that can't be satisfied; unbound PVC (a
  `WaitForFirstConsumer` StorageClass with no node in the right zone, or no
  available PV); pod anti-affinity with too few nodes.
- *Scheduled but not started.* `nodeName` is set — now it's the kubelet's
  problem. Image pull (`ImagePullBackOff` — wrong tag, missing
  `imagePullSecret`, registry down, rate limited), volume attach/mount
  failures (CSI errors, a volume attached to another node in another zone),
  or the node being unhealthy.

**`Terminating` for 20 minutes — something is blocking deletion.**
Three causes, in order of likelihood:
1. **A finalizer.** `kubectl get pod -o yaml | grep -A5 finalizers`. The
   object won't be removed until every finalizer is cleared by its
   controller. If that controller is dead or broken (a deleted operator, a
   CRD whose controller is gone), it hangs forever. Fix the controller, or
   as a last resort patch the finalizer off — knowing you're skipping
   whatever cleanup it existed to perform.
2. **Termination grace period not expired**, or a container ignoring
   `SIGTERM` and running out the clock. `terminationGracePeriodSeconds` plus
   a preStop hook can legitimately be minutes.
3. **The node is gone.** If the kubelet is unreachable, nothing can confirm
   the container stopped. Kubernetes will not force-delete, because it
   cannot distinguish "node dead" from "node partitioned" — and force-
   deleting a StatefulSet pod whose node is merely partitioned can create
   **two pods with the same identity writing the same volume**. That's why
   `--force --grace-period=0` is dangerous and must be a deliberate decision,
   not a reflex.

**`CrashLoopBackOff` — the container keeps exiting; the kubelet is backing
off (10s, 20s, 40s… capped at 5 min).**
- `kubectl logs <pod> --previous` — the logs of the *crashed* instance. This
  is the first command and the one people forget; the current instance's
  logs are empty or partial.
- Check the **exit code**: 0 (the process completed — a command that isn't
  a long-running server, or an entrypoint that exits); 1/2 (application
  error); **137** = SIGKILL, usually OOMKilled — confirm with
  `kubectl describe` → `Last State: Terminated, Reason: OOMKilled`; **143** =
  SIGTERM.
- If OOMKilled: is the limit too low, is there a leak, or is it off-heap /
  page cache (topic 05, A7)? A JVM or Go runtime not aware of its container
  limit is the common cause.
- If it's config: missing ConfigMap/Secret key, bad env var, a dependency
  unreachable at startup, permissions on a mounted volume (fsGroup /
  runAsUser vs the volume's ownership).
- If it starts and then dies after ~30 seconds: suspect a **failing liveness
  probe** rather than the app — `describe` shows probe failure events. This
  misdiagnosis is common; the app is fine and the probe is wrong (A6).
- If the logs are empty and it exits instantly: the entrypoint is wrong,
  the binary is missing (architecture mismatch — an amd64 image on arm64
  nodes), or a shell script isn't executable.

**The meta-point**: for all three, `kubectl describe` + Events + previous
logs answers the majority within a minute. The systematic part is asking
*which component owns the current state* — scheduler, kubelet, a finalizer's
controller — because that tells you where to look next.

**Weak answers miss.** `--previous` logs, exit code 137, and the reason
force-deleting a Terminating StatefulSet pod is dangerous.

**Follow-ups to expect.**
- The scheduler says "Insufficient cpu" but the nodes look idle. Explain.
  (Scheduling is against *requests*, not usage. Over-requested and
  under-used is the most common cluster inefficiency.)

---

### A16. Nodes flapping NotReady

**Answer.**
A node goes `NotReady` when the **kubelet stops posting its status/lease** to
the API server within the node-monitor grace period (~40 s by default), or
posts a condition that isn't Ready. Flapping for 30–60 s and recovering
means the heartbeat is being *delayed*, not that the node is dying. So the
question is: what is delaying it? Three families.

**1. The kubelet can't talk to the API server, or is slow to.**
- **API server / etcd overload.** If etcd's fsync latency spikes (A9) or the
  API server is saturated, *every* kubelet's lease update slows and you see
  **many nodes flap simultaneously**. That correlation is the single most
  useful diagnostic: if nodes flap together, look at the control plane, not
  the nodes. Check API server request latency and 429s (priority-and-fairness
  rejections), etcd commit/fsync duration, and whether a controller is doing
  expensive LISTs.
- **Network path**: intermittent packet loss to the control plane, a
  saturated NAT gateway or LB in front of the API server, conntrack
  exhaustion on the node (topic 05, A17), or a control plane LB idle timeout
  killing the kubelet's watch connections.
- **Certificate/auth issues** — expiring kubelet client certs cause a
  distinctive "all nodes fail at once at the same timestamp."

**2. The node is resource-starved, so the kubelet doesn't get scheduled.**
- **CPU starvation.** The kubelet is a normal process; if the node is at
  100% CPU with many runnable threads, the kubelet's heartbeat goroutine is
  delayed past the grace period. Check whether flapping correlates with load
  spikes. Fix: `--kube-reserved`/`--system-reserved` (which reduce
  Allocatable so pods can't consume everything) and putting the kubelet in a
  protected cgroup slice with reserved shares.
- **Memory pressure**: reclaim stalls, direct reclaim, or swap thrash delay
  everything. PSI (topic 05, A3) is the right metric.
- **Disk pressure / iowait**: a full or slow disk stalls the kubelet's
  writes; the container runtime's disk (image layers, logs) filling is a
  classic. `DiskPressure` also triggers eviction independently.
- **PID exhaustion** (topic 05, A4).

**3. The kubelet or runtime is itself wedged.**
- **PLEG (Pod Lifecycle Event Generator)** — `PLEG is not healthy` in the
  kubelet log is the signature. The kubelet periodically lists all
  containers via the CRI; if the container runtime is slow (many containers,
  a hung `containerd` operation, a stuck image pull, a wedged mount), the
  relist exceeds its threshold and the kubelet marks itself unhealthy. Very
  common on high-density nodes, and the fix is usually reducing pod density
  or fixing the runtime, not the kubelet.
- **Hung mounts** — an unresponsive NFS or CSI volume blocks kubelet
  syscalls in D state (topic 05, A3), and one bad mount can wedge the whole
  kubelet.
- **containerd/runc bugs, or log rotation issues** filling the disk.

**What I'd do, in order:**
1. **Is it correlated across nodes or isolated?** This single question
   splits control plane (family 1) from node-local (2, 3). Check the
   timestamp distribution of the NotReady events.
2. `kubectl get events` and node conditions for the exact reason string
   (`MemoryPressure`, `DiskPressure`, `PIDPressure`, `KubeletNotReady`).
3. On an affected node: `journalctl -u kubelet` around the flap. PLEG
   messages, API server timeout errors, and lease update failures each point
   at a different family.
4. Node metrics at the moment of the flap: CPU (including steal — topic 05,
   A15), PSI cpu/memory/io, disk latency, conntrack usage.
5. Control plane metrics: API server p99 latency, etcd backend commit
   duration, request rejections.
6. Check what's *different* about the affected nodes — instance type, kernel
   version, AZ, workload mix, pod density. Nodes flapping in a pattern
   usually share an attribute.

**Why it matters beyond the flap:** each `NotReady` past the eviction
timeout triggers **pod eviction and rescheduling**, so you get churn,
cold starts, and — for stateful workloads — data movement, all caused by a
30-second measurement artifact. That secondary damage usually exceeds the
primary problem, and it's worth saying that you'd consider raising the
eviction tolerance while diagnosing.

**Weak answers miss.** PLEG, the correlated-vs-isolated split as the first
question, and kube-reserved as the structural fix for kubelet starvation.

**Follow-ups to expect.**
- How do you stop the eviction churn while investigating? (Tune
  `node-monitor-grace-period` / the default not-ready toleration seconds,
  or add explicit tolerations with longer `tolerationSeconds` to critical
  workloads. Say that this is masking, and why masking is nonetheless right
  during an investigation.)

---

### A17. Bare metal vs managed Kubernetes

**Answer.**
Managed Kubernetes gives you a small number of things that are each large
pieces of work. On bare metal you are building all of them. The gaps, and
how I'd fill each:

**1. Control plane.** EKS/GKE run and scale the API server, scheduler,
controller manager, and etcd, with backups and upgrades. On bare metal:
- 3 (or 5) control plane nodes across separate failure domains — different
  racks, different power, different top-of-rack switches.
- **etcd on dedicated local NVMe** (A9) — this is the single most important
  hardware decision, and the most common bare-metal mistake is co-locating
  etcd with anything else.
- A load balancer in front of the API server. Chicken-and-egg: it can't be
  an in-cluster service, so it's `kube-vip`/keepalived with VRRP, or an
  external HA proxy pair, or DNS with multiple A records plus client
  retry.
- Certificate lifecycle (kubeadm handles renewal on upgrade; expiring certs
  are a classic bare-metal outage), etcd snapshot backups off-node, and a
  **rehearsed restore**.
- Upgrades: yours to plan, sequence, and roll back.

**2. Load balancer services.** There is no cloud LB to provision, so
`type: LoadBalancer` does nothing by default.
- **MetalLB** in L2 mode (one node ARPs for the VIP — simple, but all
  traffic for a VIP hits one node and failover is an ARP update) or **BGP
  mode** (nodes peer with the top-of-rack switches and advertise the VIP;
  ECMP spreads traffic across nodes — the right answer, and it's exactly the
  anycast/ECMP model from topic 03, A12).
- Or Cilium's built-in BGP control plane and L2 announcements, which avoids
  running a second component.
- Requires cooperation from the network team: a BGP session, an AS number,
  and an agreed VIP range. That's an organisational dependency, not just a
  technical one.

**3. Storage.** No EBS/PD. Options and their real costs:
- **Local PVs** — best performance (topic 05, A12), zero HA. Data dies with
  the node; the workload must replicate (Kafka, ClickHouse, ES do). Needs a
  local static provisioner and disciplined node lifecycle.
- **Ceph/Rook** — real distributed block/object/file storage with
  replication. Powerful and a substantial operational commitment: it's a
  distributed storage system you now operate, with its own failure modes,
  rebalancing behaviour, and expertise requirement. Don't adopt it casually.
- **A NAS/SAN** via CSI, if you have one — simplest if the hardware already
  exists.
- Longhorn/OpenEBS as lighter-weight replicated block options.

**4. Node lifecycle.** No autoscaling, no instance replacement, no
"terminate and let the ASG rebuild."
- **Provisioning**: PXE/iPXE + a lifecycle tool (MAAS, Tinkerbell,
  Metal³/Ironic) or config management. Ideally immutable images rather than
  configuration drift.
- **Capacity is fixed.** You can't scale out in a minute, so headroom is a
  procurement decision made months ahead. This changes autoscaling
  philosophy entirely: HPA still works within the fleet, but there is no
  cluster autoscaler, so over-provisioning is the only burst absorber.
- **Hardware fails differently**: a failed disk, a bad DIMM, a flaky NIC, a
  degraded optic. Node problem detection, firmware management, and a repair
  workflow with a real inventory system are all yours. You need
  node-problem-detector and a way to cordon-and-ticket automatically.

**5. Networking.** No VPC CNI handing out routable IPs.
- Choose overlay (works anywhere, MTU care — A8) or native routing with BGP
  (better performance and observability, requires fabric cooperation).
- **MTU** end to end, including jumbo frames if the fabric supports them.
- No security groups — NetworkPolicy is your only network segmentation, and
  it must be default-deny to mean anything.
- DNS, NTP, and the container registry are now infrastructure you run.
  **NTP especially**: clock skew breaks certificates, leases, and anything
  timestamp-ordered (topic 07, A7), and nobody notices until it's causing
  bizarre failures.

**6. Identity and secrets.** No IRSA/Workload Identity, so pods can't
assume cloud roles. You need something else for workload identity —
SPIFFE/SPIRE, or Vault with Kubernetes auth — and a KMS equivalent for
encrypting etcd at rest.

**7. Observability of the layer below.** In the cloud, hardware is
someone else's problem. Here you need IPMI/Redfish metrics, disk SMART, NIC
error counters, PSU and thermal data, switch port stats — and you need them
correlated with Kubernetes events, because "the pod is slow" and "that
node's NIC is dropping frames" are the same incident.

**What you gain, and it's real:** dramatically better price/performance
(especially for data-heavy workloads — local NVMe and no egress charges),
no noisy neighbours or steal time, hardware you can specify (GPUs, high
core counts, lots of local disk), and no cross-AZ data transfer billing —
which for a Kafka/ClickHouse pipeline is a very large number (topic 04, A13).

**The honest framing:** bare metal trades cloud's *elasticity* and *managed
components* for *cost* and *control*. It's the right choice for stable,
predictable, data-heavy workloads at scale, and the wrong choice for spiky
or small ones. The hidden cost is not the technology — it's the headcount to
operate seven additional systems well, and that should be stated as the
first-order tradeoff.

**Weak answers miss.** The API server load balancer chicken-and-egg,
etcd disk requirements, and that fixed capacity changes the autoscaling
philosophy rather than just removing a feature.

**Follow-ups to expect.**
- Hybrid — bare metal and cloud in one cluster or federated? (One cluster
  stretched across both means etcd or worker-to-control-plane latency over
  a WAN, which is fragile. Prefer separate clusters with a common control
  plane above (GitOps, a mesh spanning both), and put the workload where its
  data is.)
- What would you *not* run on bare metal? (Anything spiky, anything needing
  fast horizontal burst, and anything where the team is too small to carry
  the operational load. Naming what you'd exclude is a staff signal.)

---

### A18. Multi-tenancy for 200 internal teams

**Answer.**
Start by pinning down "no team can degrade another," because it's the
ambiguous part and it drives the cost. There are three levels and they cost
very differently:
- *Resource* isolation (one team can't starve another of CPU/memory/IO) —
  achievable within a cluster.
- *Availability* isolation (one team's incident can't take down another's
  workload) — needs failure-domain separation.
- *Security* isolation (one team can't access another's data even if
  compromised) — for internal, cooperating teams, RBAC plus network policy
  is usually the accepted bar; hostile tenancy would need much more (A11).
I'd assume the first two are the requirement and say so.

**The core decision: how many clusters?**

Not one. A single cluster for 200 teams makes the control plane a shared
failure domain for the entire company — one team's LIST-heavy controller,
one bad CRD, one admission webhook, one etcd size blowout, and everyone is
down. Also, cluster upgrades become an event nobody can ever schedule.

Not 200 either — per-team clusters means 200 control planes, 200 upgrade
cycles, terrible utilisation, and a platform team that can't keep up.

**My answer: a fleet of ~10–20 clusters as failure domains** (cells — topic
07, A14), with teams assigned to clusters, and **namespaces within a cluster
as the unit of team isolation**. Sizing driven by: blast radius (a cluster
outage affects ~5–10% of teams), etcd/control-plane scaling limits, and the
platform team's ability to operate them uniformly. Separate clusters —
non-negotiable — for prod vs non-prod, and for any workload with a distinct
compliance boundary.

**Within a cluster, layered controls:**

1. **Namespace per team per environment**, provisioned by automation, never
   by hand. A "team onboarding" pipeline creates the namespace, quota,
   default network policies, RBAC bindings, and a default PDB/priority class.
   The critical property: **the safe defaults exist before the team deploys
   anything**, rather than being a checklist they might skip.

2. **Resource isolation.**
   - `ResourceQuota` per namespace, on CPU/memory requests *and* limits,
     plus object counts (pods, services, PVCs) and storage.
   - `LimitRange` to force every pod to have requests — a BestEffort pod is
     a pod that will be evicted at the worst moment and that the scheduler
     can't reason about.
   - **PriorityClasses** with preemption: platform-critical > production >
     batch > best-effort. Under contention, the right things survive.
   - **Node pools with taints** for workloads that genuinely need
     guaranteed capacity, so a team paying for isolation can get it.
   - Recommend requests==limits for memory; discourage CPU limits for
     latency-sensitive services (topic 05, A8) — the platform should encode
     this as policy guidance, not leave 200 teams to discover it.

3. **The isolation gaps quotas don't cover** — this is where a thoughtful
   answer separates itself, because most people stop at ResourceQuota:
   - **Node-level noisy neighbours.** A quota is namespace-wide; a single
     pod can still saturate a node's disk or network. Mitigate with cgroup v2
     `io.max`/`io.latency` per pod class, local-disk-heavy workloads on
     dedicated pools, and node-level monitoring per workload.
   - **API server abuse.** A team's operator doing a full LIST every second
     degrades the control plane for everyone. **API Priority and Fairness**
     with per-tenant flow schemas is the mechanism, plus monitoring
     API request rates *by service account* and treating a top-talker as an
     incident.
   - **etcd pressure.** Object count and size quotas; forbid using
     ConfigMaps/Secrets as data stores; separate etcd for Events.
   - **Shared infrastructure**: ingress controllers, DNS (CoreDNS is a
     shared, easily-saturated component — NodeLocal DNSCache and per-tenant
     query monitoring), the metrics pipeline (cardinality — see the
     observability topic), and the log pipeline. Each needs per-tenant
     limits or one team will consume all of it. **CoreDNS and the metrics
     backend are, in my experience, the two shared components that actually
     fall over.**
   - **Cluster-scoped objects**: CRDs, webhooks, ClusterRoles, and
     PriorityClasses are global. Teams must not create them directly —
     admission policy blocks it, and platform reviews additions. One
     misconfigured mutating webhook with `failurePolicy: Fail` is a
     cluster-wide outage (A1).

4. **Network isolation.** Default-deny ingress and egress NetworkPolicy per
   namespace, installed at onboarding, with an explicit allow to shared
   services (DNS, metrics, ingress). Teams add their own allows. Without
   default-deny, NetworkPolicy is decorative.

5. **Security baseline.** Pod Security Admission `restricted` by default,
   with exceptions requiring review. RBAC scoped to the namespace, no
   cluster-admin for teams, separate ServiceAccounts per workload, admission
   policy (Kyverno/Gatekeeper/VAP) enforcing image registries, signatures,
   required labels, resource requests, and forbidding hostPath/hostNetwork/
   privileged.

6. **Platform interface.** 200 teams cannot each learn all of the above.
   Give them a constrained abstraction — a golden Helm chart or a CRD that
   generates the Deployment, Service, HPA, PDB, NetworkPolicy, and probes
   with correct defaults. Escape hatches allowed but reviewed. This is
   the thing that makes the policy work in practice, because policy that
   teams route around is not policy.

7. **Chargeback/showback.** Cost attribution per namespace (requests are the
   billing unit) is the only sustainable mechanism for making teams
   right-size. Without it, everyone requests the maximum and utilisation
   collapses.

**What I'd explicitly not solve**: hostile tenancy. If any workload is
genuinely untrusted, it goes to a separate cluster with a sandboxed runtime
(A11), and I'd say that up front rather than pretending namespaces suffice.

**Weak answers miss.** The shared-component contention (CoreDNS, API server,
metrics pipeline) — quotas cover compute and nothing else — and the
cluster-count decision as the primary blast-radius control.

**Follow-ups to expect.**
- How do you upgrade 15 clusters? (Automated, canary-first — upgrade a
  non-prod cluster, then one prod cluster, then the rest in waves. The
  platform must be able to drain and rebuild a cluster, which means teams'
  workloads must be portable — which is itself an argument for the
  constrained abstraction.)
- How do you assign teams to clusters? (Bin-pack by resource profile and
  criticality, avoid putting all of one product's services in one cluster
  — a cluster failure should degrade many teams a little, not one product
  completely. And make the mapping changeable: it's a migration, so build
  the capability before you need it.)
