# Cloud Networking, VPN & IPsec — Answers

Cloud provider limits and defaults change. Where a number appears below it is
an order-of-magnitude aid or a documented behaviour at time of writing —
verify against current docs before quoting it in an interview as exact.

---

## Tier 1 — Recall

### A1. Public vs private subnet

**Answer.**
There is no "public" flag on a subnet. The distinction is **entirely in the
route table**:

- **Public subnet**: its associated route table has a default route
  `0.0.0.0/0 → internet gateway (IGW)`. Instances in it can reach the
  internet directly, and can be reached from it *if* they have a public or
  elastic IP.
- **Private subnet**: no route to an IGW. Typically has
  `0.0.0.0/0 → NAT gateway` (which itself lives in a public subnet), giving
  outbound-only internet access, or no default route at all for a fully
  isolated subnet.

Every route table has an implicit `local` route for the VPC's CIDR, which is
why anything in a VPC can reach anything else in the same VPC by default,
subject to security groups.

Two details worth adding because they show you've built this:
- An instance in a public subnet with **no public IP** cannot reach the
  internet regardless of the route, because the IGW performs 1:1 NAT and has
  nothing to translate.
- The IGW is not a device you scale or a bottleneck you manage — it's a
  logical construct implemented in the fabric. The NAT gateway is a real
  managed appliance with real limits and real cost (A3), which is why the
  public/private choice has an economic dimension, not just a security one.

**Weak answers miss.** That it's purely routing, and the "public IP
required" nuance.

**Follow-ups to expect.**
- Why put anything in a public subnet at all? (Load balancers, NAT gateways,
  bastions — things that must be internet-reachable. Everything else goes
  private.)
- What's an egress-only internet gateway? (The IPv6 equivalent of a NAT
  gateway's outbound-only property — IPv6 addresses are globally routable so
  you need something to block inbound, but there's no address translation.)

---

### A2. Security groups vs NACLs

**Answer.**
| | Security group | NACL |
| --- | --- | --- |
| Attaches to | ENI (instance) | Subnet |
| State | **Stateful** | **Stateless** |
| Rules | Allow only | Allow and deny |
| Evaluation | All rules, any match allows | Numbered, first match wins |
| Can reference | Other security groups | CIDRs only |

**Stateful** means: if you allow an inbound connection, the return traffic is
automatically permitted regardless of outbound rules — the connection is
tracked. **Stateless** means each packet is evaluated independently, so for a
NACL you must write **both** directions: inbound on port 443, and outbound on
the **ephemeral port range** (1024–65535) for the responses. Forgetting the
ephemeral return rule is the single most common NACL misconfiguration, and
it produces a maddening symptom: the connection establishes and then hangs,
or works in one direction only.

Why it matters practically:
- Security groups are the primary control and where 95% of your policy
  should live, because they can reference *other security groups* — "allow
  from the app tier" rather than "allow from 10.0.3.0/24". That's identity
  based on membership rather than address, and it survives resizing and
  re-addressing.
- NACLs are a coarse, subnet-wide backstop. The main legitimate uses are
  explicit **deny** (which security groups cannot express — blocking a
  specific malicious CIDR) and a defence-in-depth boundary that a
  compromised instance-level change can't undo.

Also worth knowing: security group state is connection tracking, and it has
capacity. Very high connection counts per instance can hit tracking limits;
some traffic is exempt from tracking when rules are fully open in both
directions. If you're pushing millions of flows through an instance, that's a
real consideration, and it's the kind of detail an infra interviewer will
probe.

**Weak answers miss.** The ephemeral-port return rule for NACLs, and
security-group-referencing-security-group as the reason SGs are the better
tool.

**Follow-ups to expect.**
- Can a security group deny? (No. If you need deny, you need a NACL, a
  firewall appliance, or policy at another layer.)
- How does this map to Kubernetes NetworkPolicy? (Similar model — default
  allow until a policy selects a pod, then default deny; selectors are
  label-based, which is the SG-referencing-SG idea. Note that NetworkPolicy
  is enforced by the CNI, so its semantics depend on which CNI you run.)

---

### A3. NAT gateway, and how it breaks

**Answer.**
It performs source NAT for outbound traffic from private subnets: rewrites
the source IP to its own elastic IP and tracks the flow so responses come
back. It's a managed, horizontally-scaled appliance — you don't size it, but
you do pay for it.

**Problem 1: cost.** You pay both an hourly charge *and* a per-GB data
processing charge for every byte through it — on top of any egress charge.
This surprises people repeatedly, and the classic version is: private
instances pulling from S3 or ECR through the NAT gateway, paying per-GB
processing on traffic that never needed to leave the AWS network. The fix is
**VPC endpoints** — a gateway endpoint for S3/DynamoDB (free) or interface
endpoints (PrivateLink, hourly + per-GB but usually cheaper than NAT for
volume). Auditing NAT data-processing charges and adding endpoints is one of
the highest-ROI cost exercises in a typical AWS account.

**Problem 2: port exhaustion.** The NAT gateway has a limit on simultaneous
connections **to a single destination IP:port** — documented at 55,000, and
it's the ephemeral-port constraint from topic 01 A17 in managed form.
Everything behind the NAT shares one source IP, so if you have 500
instances all talking to one third-party API endpoint, they share that
budget. Exceeding it produces `ErrorPortAllocation` and connection failures
that look like the third party is broken. Fixes: multiple NAT gateways
(one per AZ is standard anyway and also avoids cross-AZ charges), connection
pooling so you open fewer connections, or contacting the destination about
additional IPs.

Third thing worth naming: **idle timeout**. Flows idle beyond the gateway's
timeout (documented at 350 seconds) are dropped, and the gateway does not
send a RST to the instance — so the instance's socket stays open and the
next write hangs until TCP gives up. This is precisely the case for TCP
keepalives set below the timeout (topic 01, A13). "Long-lived connections
mysteriously die after ~6 minutes idle" is this, essentially always.

**Weak answers miss.** The per-GB processing charge on internal AWS traffic
and VPC endpoints as the fix, plus the idle-timeout/no-RST behaviour.

**Follow-ups to expect.**
- NAT gateway vs NAT instance? (Managed and scaled vs a box you run; the NAT
  instance is cheaper at low volume and gives you control, but it's a
  single point of failure and a bandwidth ceiling you own. Rarely worth it
  now.)
- Do you need a NAT gateway per AZ? (Yes, for both availability — a single
  gateway is an AZ-scoped resource and its AZ failing takes out egress for
  everyone routed to it — and cost, since cross-AZ traffic to reach it is
  billed.)

---

### A4. IKE phase 1 and phase 2, and SAs

**Answer.**
A **Security Association (SA)** is a simplex (one-directional) agreement
between two peers: which encryption and integrity algorithms, which keys,
which SPI (Security Parameter Index — the identifier carried in each packet
so the receiver knows which SA to use), and lifetime. Because it's
one-directional, a working tunnel has **at least two** IPsec SAs, one per
direction.

**IKE Phase 1** — establish a secure, authenticated channel *for the
negotiation itself*. Peers perform a Diffie-Hellman exchange, derive keys,
and authenticate each other (pre-shared key or certificate). The output is
the **IKE SA** (bidirectional, unlike the IPsec SAs). In IKEv1 this is
main mode (6 messages, identity protected) or aggressive mode (3 messages,
faster, identity exposed). Phase 1 lifetimes are typically hours (8–24 h
is common).

**IKE Phase 2** — using the protected channel from phase 1, negotiate the
actual **IPsec SAs** that will carry data traffic: the ESP parameters, the
traffic selectors (which source/destination prefixes this SA covers), and
optionally a fresh DH exchange for **Perfect Forward Secrecy** (PFS), so a
compromise of the phase 1 keys doesn't expose the data keys. Phase 2
lifetimes are shorter — commonly ~1 hour or a byte count — and the SAs are
re-keyed on expiry.

**IKEv2** collapses this: 4 messages total (IKE_SA_INIT then IKE_AUTH),
built-in dead peer detection and liveness checks, NAT traversal by default,
MOBIKE for address changes, and much better rekey behaviour. It's what you
should be using; IKEv1 exists in interviews and in old partner equipment.

The operationally important part: **mismatched proposals are the #1 cause of
tunnels not coming up**, and phase 1 vs phase 2 failure tells you where to
look. If phase 1 never completes, it's authentication, DH group, or
encryption proposal mismatch — or UDP 500/4500 blocked. If phase 1 is up and
phase 2 isn't, it's the traffic selectors (one side configured with a
different subnet pair) or the phase 2 proposal/PFS group. Both ends must
agree exactly; there's no negotiation-down.

**Weak answers miss.** That SAs are unidirectional, and the "phase 1 up but
phase 2 down means check your traffic selectors" diagnostic — that's the
thing you can only know from having debugged one.

**Follow-ups to expect.**
- What is Dead Peer Detection and why does it matter? (Liveness probes;
  without it a peer that reboots leaves the other side sending into a
  black hole until the SA expires.)
- Route-based vs policy-based VPN? (Policy-based: traffic selectors decide
  what's encrypted, one SA pair per selector pair — inflexible, and the
  cause of most interop pain with legacy firewalls. Route-based: a virtual
  tunnel interface with `0.0.0.0/0` selectors and the *routing table*
  decides — supports dynamic routing via BGP over the tunnel, which is what
  you want. AWS VGW/TGW VPNs are route-based.)
- How do you get more than one tunnel's worth of bandwidth? (ECMP across
  multiple tunnels with BGP — see A15.)

---

### A5. ESP vs AH, tunnel vs transport

**Answer.**
**AH (Authentication Header)** provides integrity and authentication for the
packet *including most of the outer IP header* — but **no encryption**.
Because it covers the IP header, it breaks under NAT: NAT rewrites addresses,
which invalidates the integrity check. That, plus the lack of
confidentiality, is why AH is essentially unused today.

**ESP (Encapsulating Security Payload)** provides encryption *and*
integrity, but only over the payload and the ESP header — not the outer IP
header. That's exactly what makes it NAT-compatible (with NAT-T, which wraps
ESP in UDP 4500 because ESP is IP protocol 50 and has no ports for a NAT
device to translate). Use ESP. Always.

**Transport mode**: the original IP header is kept; only the payload is
encrypted. Used host-to-host, where the two endpoints are the two
communicating machines. Lower overhead, but the original addresses are
visible and it can't carry traffic on behalf of other hosts.

**Tunnel mode**: the entire original IP packet is encrypted and encapsulated
inside a new IP packet addressed to the tunnel endpoints. This is what
site-to-site VPNs use, because the gateways are forwarding traffic for
networks behind them, and it hides the internal addressing. Cost: a full
extra IP header (20 bytes for IPv4) on top of the ESP overhead — which is
where the MTU problem comes from (A7).

Summary line: **site-to-site is ESP in tunnel mode**; transport mode is for
the rarer host-to-host case (and for encapsulating another tunnel protocol,
e.g. GRE-over-IPsec in transport mode, which is a common enterprise pattern
because GRE handles multicast and dynamic routing while IPsec handles
encryption).

**Weak answers miss.** *Why* AH is dead — the NAT incompatibility — which is
the actual content of the question.

**Follow-ups to expect.**
- What's the ESP overhead? (SPI + sequence number + IV + padding + pad
  length + next header + ICV. Order of 50–60 bytes with AES-GCM in tunnel
  mode, more with NAT-T's extra UDP header. Do the arithmetic in A7.)
- Why does IPsec use UDP 4500 sometimes? (NAT traversal — ESP has no port
  numbers, so a NAT device can't demultiplex multiple inside hosts.
  Encapsulating in UDP gives it ports.)

---

### A6. AS, eBGP vs iBGP

**Answer.**
An **autonomous system** is a network under a single administrative routing
policy, identified by an AS number (16- or 32-bit, allocated by the RIRs).
The internet is the graph of ASes and the peering/transit relationships
between them, and BGP is the protocol they use to tell each other which
prefixes they can reach.

**eBGP** runs between routers in *different* ASes. Default TTL of 1 (peers
are expected to be directly connected), and — crucially — the router
**prepends its own AS** to the AS_PATH when advertising, which is both the
loop-prevention mechanism (reject any route whose AS_PATH already contains
your AS) and the primary path-selection metric.

**iBGP** runs between routers *within* the same AS, to distribute
externally-learned routes internally. It does **not** prepend the AS number —
so AS_PATH can't prevent loops internally. Instead the rule is: **a router
must not re-advertise an iBGP-learned route to another iBGP peer.** That
requirement forces a **full mesh** of iBGP sessions (n(n−1)/2), which
doesn't scale, and the two standard solutions are **route reflectors** (a
designated router is allowed to reflect, with originator-id/cluster-list for
loop prevention) and **confederations**.

Why an SRE should care, beyond trivia:
- BGP is what makes **anycast** work (topic 02, A11 and topic 03, A12), and
  "withdraw the announcement" is your fastest global failover lever.
- BGP path selection is policy-driven, not latency-driven — shortest AS_PATH
  is not shortest RTT. That's why anycast can route a user to a "far" PoP.
- **Route leaks and hijacks**: BGP has historically had no built-in
  authentication of who may announce a prefix, so a misconfigured or
  malicious announcement of someone else's prefix propagates. RPKI + ROV
  (route origin validation) is the deployed mitigation for origin
  validation, with ASPA and BGPsec addressing path validation; adoption is
  partial, so verify current state rather than asserting the internet is
  fixed.
- In cloud: BGP over your VPN or Direct Connect is how routes are exchanged
  dynamically, and **AS path prepending / local preference / MED** are the
  knobs you use to steer traffic between redundant links.

**Weak answers miss.** The iBGP full-mesh requirement and *why* it exists
(no AS_PATH loop prevention internally). Also missed: connecting BGP to
anycast failover, which is what makes this an SRE question rather than a
networking-exam question.

**Follow-ups to expect.**
- How do you prefer one Direct Connect link over another? (Inbound to AWS:
  AS path prepending or a more specific prefix on the preferred link.
  Outbound from AWS: local preference / BGP communities that the provider
  honours. Say that inbound and outbound are steered by different knobs and
  that asymmetry is the usual source of confusion.)
- What's the difference between transit and peering? (Transit: you pay for
  reachability to everything. Peering: settlement-free exchange of your and
  your customers' routes only. Affects both cost and path.)

---

## Tier 2 — Explain / compare

### A7. IPsec and MTU

**Answer.**
Tunnel-mode ESP adds a new outer IP header plus ESP overhead to every
packet. Rough arithmetic for IPv4 with AES-GCM: 20 bytes outer IP + 8 bytes
ESP header (SPI + sequence) + 8–16 bytes IV + ICV (16 bytes) + padding and
trailer — call it **~55–60 bytes**, plus another 8 if NAT-T is wrapping it
in UDP. So on a 1500-byte path, the effective payload MTU inside the tunnel
drops to roughly **1400–1440**.

Now the failure. The host inside sends a 1500-byte packet with DF set (which
everything does, because PMTUD). The tunnel gateway can't fit it, so it
either fragments (bad, topic 01 A15) or drops it and sends back ICMP
"fragmentation needed." If ICMP is filtered anywhere — and it very often is,
either by a security policy or by the far end's firewall — the sender never
learns and you have a **PMTUD black hole**.

The symptom is unmistakable and worth being able to recite: **the tunnel
comes up, ping works, SSH connects and then hangs at the banner, small HTTP
requests succeed and large POSTs or file transfers time out.** Anything
small works; anything that fills a segment dies.

**The standard fix is MSS clamping.** On the tunnel endpoint, rewrite the
MSS option in transiting TCP SYNs down to (tunnel MTU − 40). Both ends then
negotiate a segment size that fits, and it works without depending on ICMP
at all. Conceptually `iptables -t mangle -A FORWARD -p tcp --tcp-flags
SYN,RST SYN -j TCPMSS --clamp-mss-to-pmtu` (or an explicit
`--set-mss 1360`); every VPN appliance has an equivalent setting, and AWS
documents a specific MSS for its VPN.

Secondary measures: lower the tunnel interface MTU so the stack knows;
permit ICMP type 3 code 4 through the firewalls so real PMTUD can work;
enable `tcp_mtu_probing` (PLPMTUD) so hosts infer the MTU without ICMP.

The important limitation to state: **clamping only fixes TCP.** UDP has no
handshake to intercept, so UDP applications over a tunnel must handle path
MTU themselves — which is why QUIC does its own probing, and why a
UDP-based protocol you write needs to care.

**Weak answers miss.** The diagnostic signature (ping fine, big transfers
dead) and the TCP-only limitation of clamping. A candidate who just says
"set a lower MTU" hasn't explained why the ICMP path failed.

**Follow-ups to expect.**
- The same problem in Kubernetes with an overlay? (VXLAN adds 50 bytes; pod
  MTU must be node MTU minus encapsulation. Mismatched MTU between CNI and
  node config is a very common cluster bug with the exact same signature.)
- How would you confirm it in 2 minutes? (`ping -M do -s` sweep to find the
  largest surviving size, from a host behind the tunnel.)

---

### A8. VPN vs Direct Connect vs peering vs Transit Gateway

**Answer.**
These aren't alternatives to each other across the board — two are
*cloud-to-onprem* and two are *cloud-to-cloud*. Say that first; it reframes
the question correctly.

**Cloud ↔ on-premises:**

- **Site-to-site VPN.** IPsec over the public internet. Provisioned in
  minutes, cheap, encrypted by default. Costs: bandwidth is capped per
  tunnel (AWS documents ~1.25 Gbps per tunnel — verify current figures) and
  you scale by adding tunnels with ECMP; latency and jitter are whatever the
  internet gives you today, with no SLA on the path; and you inherit the
  MTU problem in A7. Right answer for: getting started, low-to-moderate
  bandwidth, backup path for a dedicated link, and anything where the
  internet's variability is acceptable.
- **Direct Connect / Cloud Interconnect.** A dedicated physical circuit from
  your colo/carrier into the provider's edge. Consistent latency, committed
  bandwidth (1/10/100 Gbps), and materially cheaper egress rates — the cost
  case alone often justifies it at volume. Costs: lead time in weeks to
  months, a port charge whether you use it or not, and a single circuit is a
  single point of failure, so a real deployment means two circuits at two
  locations, ideally two providers. Note it is **not encrypted** by
  default — it's a private circuit, not a secure one, so regulated traffic
  usually runs IPsec or MACsec over it anyway. Right answer for: sustained
  high bandwidth, latency-sensitive hybrid workloads, and large egress
  volumes.

**Cloud ↔ cloud (within a provider):**

- **VPC peering.** A direct, non-transitive connection between two VPCs.
  No bandwidth bottleneck (it uses the underlying fabric), no additional
  hourly charge beyond data transfer, and low latency. Costs: **non-transitive**
  (A11) so N VPCs need N(N−1)/2 peerings, route tables must be maintained on
  both sides, and CIDRs must not overlap. Right answer for: a handful of
  VPCs with a stable topology.
- **Transit Gateway.** A hub-and-spoke router. Each VPC attaches once and
  the TGW routes between them, and it also terminates VPNs and Direct
  Connect gateways, so it becomes the single connectivity hub for
  cloud+onprem. Supports multiple route tables for segmentation (prod can't
  reach dev). Costs: an hourly charge per attachment *and* a per-GB data
  processing charge on everything crossing it — which is real money at
  volume and is the reason people keep high-throughput paths on peering —
  plus it's another hop of latency and a shared failure domain. Right answer
  for: more than a handful of VPCs, hybrid connectivity, or when you need
  routing segmentation.

The decision heuristic I'd give: peering until the mesh becomes unmanageable
(roughly 5–6 VPCs) or you need on-prem, then Transit Gateway; VPN until
bandwidth or latency consistency forces Direct Connect, and keep the VPN as
the DR path afterwards.

**Weak answers miss.** That Direct Connect isn't encrypted, and the TGW
per-GB processing charge — both are the kind of thing that only bites people
who've deployed it.

**Follow-ups to expect.**
- How do you make Direct Connect highly available? (Two connections, two
  DX locations, ideally two providers; plus a VPN backup with BGP so
  failover is automatic. Test it — a DR path you've never failed to is a
  hypothesis.)
- What's a Direct Connect Gateway? (Lets one DX connection reach VPCs in
  multiple regions/accounts — the piece that makes DX work at organisational
  scale.)

---

### A9. WireGuard vs IPsec

**Answer.**
**WireGuard** gets right:
- **Simplicity.** ~4k lines of kernel code vs IPsec's sprawling stack. That's
  a security argument (auditability, tiny attack surface) and an operational
  one — the config is a handful of lines and there is essentially nothing to
  misconfigure.
- **No negotiation.** One fixed, modern cryptographic suite (Noise
  framework, Curve25519, ChaCha20-Poly1305, BLAKE2s). There are no cipher
  proposals to mismatch, which eliminates the entire class of "the tunnel
  won't come up" problems that dominate IPsec operations. The flip side:
  no crypto agility — if the suite needs replacing, it's a protocol version
  change, which is a deliberate design choice, not an oversight.
- **Stateless-ish, roaming-friendly.** Peers are identified by public key,
  not by IP. A peer that changes address is recognised on its first
  authenticated packet, so roaming across networks just works.
- **Performance.** In-kernel, less per-packet work, generally faster than
  IPsec in benchmarks — though modern kernel IPsec with AES-NI is
  competitive, so I'd frame this as "at least as fast, with far less
  tuning" rather than claiming a large fixed win.
- **Silence.** Doesn't respond to unauthenticated packets, so it doesn't
  announce itself to scanners.

**IPsec** gets right:
- **Interoperability.** It's the standard every firewall, router, and
  enterprise partner speaks. If the other end of the tunnel is a partner's
  Cisco/Fortinet/Palo Alto or a cloud provider's managed VPN endpoint,
  IPsec is frequently the only option. This is usually the deciding factor
  and it's not a technical one.
- **Certificate-based authentication and integration with enterprise PKI**;
  WireGuard is raw public keys, so key distribution and revocation are your
  problem (which is why Tailscale/Netbird exist — they're control planes
  layered on WireGuard to solve exactly this).
- **Mature features**: dynamic routing over the tunnel, rich policy,
  hardware offload on appliances, DPD, and the whole IKE ecosystem.
- **Auditability/compliance checkboxes**: FIPS validation and similar, which
  matter in regulated environments regardless of engineering merit.

The honest summary: WireGuard for infrastructure you control on both ends
(node-to-node mesh, admin access, cloud-to-cloud), IPsec when you must
interoperate with someone else's equipment or satisfy a compliance
requirement. Key management is WireGuard's real gap at scale, and the answer
is a control plane on top.

**Weak answers miss.** That WireGuard's lack of crypto agility is a
deliberate tradeoff, and that key distribution/revocation is the operational
gap.

**Follow-ups to expect.**
- How do you revoke a WireGuard peer? (Remove its public key from the peers
  list on every node it can reach — which is exactly why you want a control
  plane rather than config management for this.)
- Does WireGuard have forward secrecy? (Yes — it rekeys periodically via
  ephemeral Diffie-Hellman under the Noise handshake.)

---

### A10. Overlapping CIDRs after an acquisition

**Answer.**
This is the classic post-merger networking problem, and the first thing to
say is that **peering, Transit Gateway, and routing in general cannot handle
overlapping address space** — a route table can't have two different
next-hops for `10.0.0.0/16`. So there are only three families of solution.

**Option 1 — Re-address one side. The correct long-term answer.**
Renumber the acquired environment into a non-overlapping range from a
properly planned IPAM allocation. Everything afterwards is simple: peering,
TGW, DNS, security groups, all normal.
Cost: it is a large, disruptive project. Every hardcoded IP, every
allowlist, every partner firewall rule, every database connection string,
every certificate with an IP SAN. Doable incrementally by standing up new
subnets in the correct range within the same VPC and migrating workloads
subnet by subnet — which is the practical path, because you don't have to
move everything at once. Do this if the two environments will be jointly
operated for years, which after an acquisition they will be.

**Option 2 — NAT at the boundary. The pragmatic bridge.**
Put a NAT layer between the two environments so each side sees the other
through a non-overlapping "translated" range.
- **1:1 (static) NAT** for the specific hosts that need to talk: allocate a
  spare range, e.g. `100.64.0.0/16` (RFC 6598 CGNAT space, chosen precisely
  because it's rarely used internally), and map `10.0.1.5` on their side to
  `100.64.1.5` as seen from yours. Bidirectional if both sides initiate.
- **Double NAT / twice-NAT** if both sides overlap and both initiate.
- On AWS: a **private NAT gateway**, a Network Firewall, or a self-managed
  NAT instance/appliance in a transit VPC, with TGW routing the translated
  ranges.
Cost: DNS becomes a mess (names must resolve to the translated address from
the other side — a split-horizon problem, A12), troubleshooting is much
harder because addresses differ depending on where you look, logs and audit
trails record translated addresses, and anything embedding IPs in the
payload (some legacy protocols, FTP, SIP, certain databases' cluster gossip)
breaks. It's a bridge, not a destination — but it gets you connected in days
rather than quarters.

**Option 3 — Don't route between them at all.**
Expose only the specific services that need to be shared, through mechanisms
that don't require IP routing:
- **PrivateLink / Private Service Connect** — an endpoint in your VPC backed
  by a service in theirs. Explicitly designed to work with overlapping
  CIDRs because it's a proxied, one-directional service exposure, not a
  route (A11). This is often the best answer for the "we just need to call
  their API" case, and it's the one candidates most often miss.
- **Public endpoints with mTLS/authentication**, or an API gateway.
- A **service mesh** spanning both, where identity and routing are at L7.
- Application-level integration: shared Kafka, shared object storage,
  message queues.

**How I'd actually sequence it:** PrivateLink or public-with-mTLS for the
2–3 integrations needed in the first month; NAT bridge if broader
connectivity is genuinely required for migration tooling; a funded
re-addressing project for the acquired side with a target of retiring the
NAT within a year. And immediately: an IPAM discipline and a registry of
allocated ranges, because the reason you're in this position is that nobody
had one — and the next acquisition will do it again.

**Weak answers miss.** PrivateLink as the overlap-immune option, and RFC
6598 space as the conventional translation range. Also missed: that the NAT
bridge's real cost is DNS and observability, not the packets.

**Follow-ups to expect.**
- How do you handle DNS across a NAT bridge? (Conditional forwarding plus
  DNS translation — the answer must resolve to the translated address from
  the caller's perspective. Route 53 Resolver rules, or a resolver that
  rewrites. Genuinely painful, and worth saying so.)
- What if both environments run Kubernetes with overlapping pod CIDRs too?
  (Now you have two layers of overlap. Cluster mesh solutions handle this
  with per-cluster identity and gateway-based translation, but the honest
  answer is that this is the point where re-addressing becomes cheaper than
  the workaround.)

---

### A11. Peering vs Transit Gateway vs PrivateLink

**Answer.**
**VPC peering is non-transitive** because it is implemented as routes, and a
route says "for this prefix, send to this peering connection" — there is no
mechanism for the peer VPC to forward the packet onward to a third VPC. A
routes to B and B routes to C does not give A a path to C; B would have to
act as a router, and the peering fabric explicitly does not forward transit
traffic. This is a deliberate design decision that keeps blast radius and
policy simple: a peering connection grants access between exactly two VPCs.

Consequence: N VPCs fully connected need N(N−1)/2 peering connections, each
with route table entries on both sides. At 10 VPCs that's 45 connections and
a maintenance burden nobody keeps correct.

**Transit Gateway** solves that by being an actual router: every VPC attaches
once, the TGW holds route tables, and it forwards between attachments —
transitively, by design. It also terminates VPN and Direct Connect gateway
attachments, so it becomes the one place hybrid connectivity lives. Multiple
route tables give you segmentation (prod attachments in one table, dev in
another, with no route between). Costs: per-attachment hourly + per-GB
processing, an extra hop, and a shared failure domain and blast radius —
a bad route in the TGW affects everything.

**PrivateLink** is a different shape entirely, and the distinction is the
part worth getting right: peering and TGW connect **networks**; PrivateLink
exposes a **service**.

You create an interface endpoint in your VPC — an ENI with an IP from *your*
subnet — that fronts a specific service (an NLB) in someone else's VPC. What
it gives you:
- **No routing relationship.** Their CIDR and yours never meet. Overlapping
  address space is fine, which is the killer feature (A10).
- **One direction only.** The consumer can reach the service; the service
  provider gets no path into the consumer's VPC. Under peering, routes are
  bidirectional and you're relying on security groups to constrain it.
- **Granularity.** One service exposed, not a whole network.
- Works across accounts and organisations without any trust relationship
  beyond an acceptance handshake — which is why it's how SaaS vendors offer
  private connectivity to customers.

Costs: it's per-service, so it doesn't scale to "connect these two
environments generally"; hourly per-endpoint plus per-GB charges; and the
service side must be behind an NLB (L4), which constrains the architecture.

The one-liner: **peering for a few networks, TGW for many networks plus
hybrid, PrivateLink when you want to share a service rather than a network —
especially across an organisational or address-space boundary.**

**Weak answers miss.** The unidirectionality and CIDR-independence of
PrivateLink, which is the entire reason it exists.

**Follow-ups to expect.**
- Can you reach a PrivateLink endpoint from on-prem? (Yes, via DX/VPN into
  the VPC containing the endpoint — a common pattern for giving datacentre
  workloads private access to a cloud service.)
- What's the GCP equivalent? (Private Service Connect; the same
  service-not-network model.)

---

### A12. Hybrid DNS

**Answer.**
The problem: you have names that only resolve inside the cloud VPC (private
hosted zones, internal service names), names that only resolve on-premises
(Active Directory, legacy internal zones), and public names — and clients on
both sides need to resolve all three.

**Inside a VPC**, instances point at the VPC's `.2` resolver
(the base of the VPC CIDR plus two — the "Amazon-provided DNS" / metadata
resolver). It answers for: private hosted zones associated with the VPC,
internal AWS names, and recurses to the public internet for everything else.
It's reachable only from inside the VPC, which is the constraint that makes
hybrid DNS non-trivial.

**The two directions you must solve:**

1. **Cloud → on-prem.** Instances need `db.corp.internal` to resolve to an
   on-prem address. Mechanism: a **Route 53 Resolver outbound endpoint**
   (ENIs in your VPC) plus **forwarding rules** — "queries for
   `corp.internal` go to these on-prem DNS servers." Traffic goes over your
   VPN/DX. Before Resolver endpoints existed people ran their own BIND/
   Unbound forwarders on EC2, and plenty still do; it's the same design with
   more toil.
2. **On-prem → cloud.** On-prem clients need `svc.aws.internal` (a private
   hosted zone) to resolve, but they can't reach the `.2` resolver.
   Mechanism: a **Route 53 Resolver inbound endpoint** — ENIs with real IPs
   in your VPC that accept DNS queries — and conditional forwarding on your
   on-prem DNS servers pointing the cloud zones at those IPs.

**Split-horizon** is the related concept: the same name resolving
differently depending on who asks — `api.example.com` returning a private
address internally and a public one externally. Implemented with a private
hosted zone for the same domain as a public one; the private zone wins for
queries from associated VPCs. Useful (internal traffic stays internal, no
NAT/egress cost, no internet exposure) and dangerous (an engineer debugging
from a laptop gets a different answer than production; the two zones drift).
If you use it, generate both from one source of truth.

**Failure modes worth naming:**
- The forwarder becomes a hard dependency on your VPN. If the tunnel drops,
  DNS for on-prem names fails, and because DNS failures manifest as
  application timeouts everywhere, the blast radius is much wider than the
  actual dependency. Cache aggressively and have a second path.
- Query volume: every instance's resolver traffic funnels through the
  endpoints, and there are per-ENI query rate limits — a chatty fleet with
  no local caching can hit them, producing intermittent SERVFAILs that look
  like random application failures.
- Conditional forwarding loops: on-prem forwards `example.com` to cloud,
  cloud forwards `example.com` back to on-prem. Easy to build, hard to spot.

**Weak answers miss.** That the two directions need two different mechanisms
(inbound vs outbound endpoints), and that DNS forwarding makes the VPN a
critical dependency for far more than on-prem traffic.

**Follow-ups to expect.**
- How does this work in Kubernetes? (CoreDNS with a `forward` plugin per
  zone, or `stubDomains`. Same conditional forwarding idea; note NodeLocal
  DNSCache to cut load and latency, and the `ndots:5` search-path problem
  from topic 01, A16.)
- What happens to hybrid DNS during a region failure?

---

### A13. Cloud network cost model

**Answer.**
The mental model, in the order that costs matter:

1. **Egress to the internet is the expensive one.** Order of cents per GB,
   with volume tiers. Ingress from the internet is generally free. That
   asymmetry drives architecture: it's why CDNs pay for themselves, why you
   compress responses, and why "just serve it from the origin" gets
   expensive at scale.
2. **Cross-region transfer** is charged, both directions in effect (you pay
   egress from the source region). This prices multi-region replication and
   is often the dominant line item in an active-active design.
3. **Cross-AZ transfer within a region** is charged in AWS, typically per GB
   in **each** direction. This is the one that quietly dominates internal
   bills, because it's invisible in application design — a service mesh that
   load balances uniformly across AZs sends ~2/3 of its traffic cross-AZ by
   default. Chatty microservices, replicated data stores (Kafka replication,
   Cassandra, ClickHouse), and shuffle-heavy pipelines are the big
   consumers. Mitigations: topology-aware routing / zone-aware load
   balancing so traffic prefers same-AZ endpoints, rack/zone-aware replica
   placement, and follower fetching in Kafka so consumers read from a
   same-AZ replica.
4. **Same-AZ, private IP traffic** is generally free. That's the target.
5. **Data processing charges on managed middleboxes.** NAT gateway per-GB,
   Transit Gateway per-GB, ALB/NLB capacity units, VPC endpoint per-GB,
   PrivateLink per-GB. Every hop through a managed network appliance has a
   toll, and a design that chains three of them pays three times for the
   same bytes.
6. **Fixed hourly charges** on endpoints, gateways, attachments, DX ports.
   Small individually; large when multiplied by accounts × AZs × VPCs. A
   common finding is dozens of interface endpoints provisioned per account
   by a module default.

**The design heuristics that follow:**
- Keep bytes in the same AZ where correctness allows; make AZ-awareness a
  first-class routing concern rather than an afterthought.
- Put a CDN in front of anything internet-facing that serves volume.
- Use gateway VPC endpoints for S3/DynamoDB (free) and audit for traffic
  needlessly traversing NAT.
- For sustained high-volume hybrid traffic, Direct Connect's lower egress
  rate is frequently the justification on its own.
- Attribute cost per team/tenant early — VPC flow logs plus tagging — or the
  conversation about who's generating the traffic is unwinnable.

The framing to state: cloud networking is priced to make **movement**
expensive and **locality** cheap, so the architectural lever is data
gravity — move computation to data, not data to computation.

**Weak answers miss.** Cross-AZ charges, which is the biggest surprise in
most real bills, and the "every managed hop has a per-GB toll" observation.

**Follow-ups to expect.**
- How would you find where the money goes? (Cost and Usage Report with
  usage-type breakdown for the transfer types, plus VPC flow logs
  aggregated by src/dst AZ and by prefix. Flow logs are the only way to
  attribute cross-AZ traffic to a workload.)
- Is cross-AZ traffic worth eliminating if it costs availability? (No — say
  so plainly. Single-AZ to save transfer cost is a bad trade for anything
  with an availability SLO. The right fix is topology-aware routing that
  prefers local *and* fails over cross-AZ.)

---

## Tier 3 — Scenario / debug

### A14. Private instance can't reach an external API

**Answer.**
Work outward from the instance, and at each step name what the result
eliminates. The value of this answer is the ordering discipline, not the
tool list.

1. **Is it DNS or connectivity?** `dig api.example.com` vs
   `curl -v https://<resolved-IP>`. If DNS fails, the problem is the
   resolver path (`/etc/resolv.conf`, the VPC `.2` resolver, a DNS firewall,
   a conditional forwarding rule) and nothing downstream matters. If DNS
   resolves but the connect fails, DNS is eliminated. *This step first
   because it splits the problem in half.*
2. **Does the packet leave the instance?** `curl -v` distinguishes
   "connection refused" (something answered — a RST — so routing works and a
   firewall is rejecting, or the port really is closed) from "connection
   timed out" (nothing answered — a DROP, a routing black hole, or the
   destination is gone). This is the single highest-information observation
   available and it's free. Confirm with `tcpdump -i any host <ip>` on the
   instance: SYNs going out with no reply localises it to the path or the
   return.
3. **Local host firewall.** `iptables -L -n -v` / `nft list ruleset`,
   `firewalld`. Cheap to check, occasionally the answer, and it's on the
   instance so nothing else can rule it out.
4. **Security group — egress.** People check inbound and forget that SGs
   have egress rules too; a locked-down egress rule is a common cause. It's
   stateful, so if egress is allowed the return is automatically fine.
5. **NACL — both directions.** Stateless: verify outbound on the destination
   port *and* inbound on the ephemeral range (A2). This is where the "works
   halfway" symptoms come from.
6. **Route table.** Does the subnet have `0.0.0.0/0` pointing at a NAT
   gateway (or the right appliance)? Is the NAT gateway in a *public*
   subnet with a route to the IGW, and does it have an elastic IP? A NAT
   gateway in a private subnet is a classic self-inflicted wound.
7. **NAT gateway health and limits.** CloudWatch `ErrorPortAllocation`,
   `PacketsDropCount`. If you're at port exhaustion (A3) the failure is
   intermittent and load-correlated, which the symptom description should
   have hinted at.
8. **Beyond your control:** the destination blocking your NAT's egress IP
   (rate limiting, geo-blocking, an allowlist you were never added to), or
   the destination genuinely being down. Test from a different source IP —
   another NAT gateway, another region, a laptop — to separate "our network"
   from "their service."
9. **MTU**, if the connection *establishes* and then hangs on data (A7) —
   pattern-match this early if the symptom is "TLS handshake starts and
   stalls."

**The meta-point I'd state up front:** the two observations that carry the
most information are (a) DNS vs connect and (b) refused vs timed out. Those
two answers eliminate most of the list before you touch the console. Anyone
who starts by opening the AWS console and reading security groups is
searching linearly through the least likely causes.

**Weak answers miss.** The refused-vs-timeout distinction, testing from a
second source IP to exonerate your own network, and the NACL ephemeral-port
direction.

**Follow-ups to expect.**
- It works from one AZ and not another. What does that tell you? (Per-AZ
  route tables and per-AZ NAT gateways — you've localised it to one AZ's
  egress path in one step.)
- How would you make this debuggable *before* the incident? (VPC flow logs
  with reject records, which tell you exactly which rule dropped what;
  synthetic egress probes per subnet.)

---

### A15. VPN capped at 1 Gbps, and drops hourly

**Answer.**
Two independent symptoms with two independent causes. Say that first —
conflating them is the trap.

**Symptom 1: the ~1 Gbps ceiling.**

A single IPsec tunnel is limited by a single flow's processing path. On
AWS, a VPN tunnel is documented at roughly 1.25 Gbps and that is a hard
per-tunnel property (verify current figures) — but the same ceiling appears
on self-managed gateways for a structural reason: **an IPsec SA's packet
processing is largely serialised.** ESP sequence numbers and the anti-replay
window create ordering constraints, so a single SA is typically handled by
a single core. You cannot scale one tunnel by adding CPUs.

So "no matter what you do" is expected — as long as what you're doing is
tuning one tunnel. The fixes are all about **parallelism**:
- **Multiple tunnels with ECMP.** Run several IPsec tunnels between the same
  sites, advertise the same prefixes over all of them with BGP, and let
  ECMP hash flows across them. Aggregate throughput scales with tunnel
  count. (Note: *flows* are hashed, so a single TCP connection still lives
  in one tunnel and is still capped — an important caveat if the traffic is
  one big replication stream.) On AWS this is what a Transit Gateway VPN
  with ECMP enabled gives you, versus a VGW attachment which historically
  did not support ECMP across tunnels.
- **Split the traffic across more flows** at the application layer if a
  single stream is the bottleneck — parallel transfer streams.
- **Check crypto offload.** Confirm AES-NI is being used; a gateway falling
  back to software crypto is dramatically slower. On Linux, verify the
  kernel is doing the ESP work rather than a userspace implementation.
- **The real answer at sustained multi-gigabit: stop using a VPN.** Direct
  Connect / Interconnect with MACsec or IPsec over it. If you need
  10 Gbps consistently, the internet path is the wrong medium regardless of
  tunnel count.

Also verify it's actually the tunnel: run an iperf between the gateways
*outside* the tunnel. If the raw path is also ~1 Gbps, it's the circuit or
an intermediate policer and the tunnel is innocent. And do the BDP check
from topic 01 A7 — a high-RTT path with an untuned window will underperform
regardless of the tunnel, and that's a completely different fix.

**Symptom 2: hourly multi-second drops.**

An hourly period is a strong signal: **phase 2 rekey**. IPsec SA lifetimes
are commonly 3600 seconds, and a rekey that isn't seamless produces exactly
this — a brief gap while the new SA is established and traffic switches
over.

Causes of a *disruptive* rather than seamless rekey:
- **Mismatched lifetimes** between the two peers. When one side expires
  first and the other hasn't rekeyed, you get a window where one direction's
  SA is gone. Both ends should be configured with the same lifetime, and the
  initiator should rekey *before* expiry with a margin.
- **No overlap / make-before-break.** A correct implementation establishes
  the new SA before tearing down the old one and briefly accepts both.
  Older IKEv1 implementations and some appliances tear down first. IKEv2
  handles this much better, which is a concrete reason to migrate.
- **Traffic-selector or proposal mismatch that only manifests on rekey** —
  the tunnel came up under one negotiation and fails to renegotiate.
- **PFS mismatch** on the rekey DH group.
- If it's not exactly hourly, consider **DPD** killing the tunnel during a
  quiet period, or a NAT device on the path expiring the UDP 4500 mapping —
  in which case NAT-T keepalives are the fix and the interval is the NAT's
  timeout, not the SA lifetime.

**How to confirm:** correlate the drop timestamps against the SA
establishment times in the IKE daemon logs (`ipsec statusall` / `swanctl
--list-sas` for strongSwan, or the cloud console's tunnel logs). If the
gaps line up with rekey events, it's confirmed in one look. Then align
lifetimes on both ends, move to IKEv2 if you aren't, and verify the new SA
is created before the old one dies.

**Weak answers miss.** The single-SA serialisation reason for the throughput
cap — candidates usually say "it's an AWS limit" without knowing why the
limit exists, and therefore propose tuning instead of parallelism. On the
drops, missing "hourly = rekey" means missing the whole question.

**Follow-ups to expect.**
- Does ECMP across tunnels help a single large flow? (No — say this
  explicitly. Flow hashing pins a connection to one tunnel. If your workload
  is one replication stream, you need either application-level parallelism
  or a bigger single path.)
- What about the MTU interaction with all this? (A7 — and note that
  fragmentation inside the tunnel would also depress throughput, so check it
  as part of the same investigation.)

---

### A16. Multi-region AWS + colo + private customer access

**Answer.**
Four connectivity problems; I'd design them as separate layers with one hub.

**Layer 1 — Intra-cloud, multi-region.**
A Transit Gateway per region, each with segmented route tables (prod / non-
prod / shared services), and **TGW inter-region peering** between them. VPCs
attach to their regional TGW once. Why not full-mesh VPC peering: with three
regions and a realistic VPC count the mesh is unmaintainable, and TGW gives
routing segmentation as a first-class feature.
Cost accepted: per-attachment hourly plus per-GB processing on everything
crossing the TGW, and inter-region data transfer on top. For a small number
of very-high-volume paths (e.g. cross-region database replication) I'd
consider direct VPC peering *alongside* the TGW as a bypass, and I'd say so
explicitly as a cost/complexity tradeoff rather than pretending TGW is free.
Non-overlapping CIDRs from a central IPAM is a precondition, not a detail.

**Layer 2 — Colo (bare-metal GPU) to cloud.**
Direct Connect, and specifically:
- **Two circuits at two DX locations**, ideally different providers, to
  avoid a single facility or carrier being the failure domain.
- Terminate on a **Direct Connect Gateway** attached to the TGWs, so one
  pair of circuits reaches all three regions and all attached VPCs rather
  than one VPC per virtual interface.
- **BGP** over the circuits, with local preference / AS path prepending to
  set the primary and backup paths.
- A **site-to-site VPN as the backup path** over the internet, also
  BGP-attached to the TGW with a less-preferred path, so failover is
  automatic and tested. Cheap insurance.
- Encryption: DX is a private circuit but not encrypted. For GPU training
  traffic carrying customer data I'd run MACsec if the circuit supports it,
  or IPsec over DX — and note the throughput ceiling from A15, which means
  IPsec over a 10 Gbps DX needs multiple tunnels with ECMP or dedicated
  crypto hardware. Worth flagging as a real constraint, since GPU workloads
  are exactly the ones that move large datasets.
- MTU: DX supports jumbo frames (9001) on supported virtual interfaces —
  worth using for bulk dataset movement, and worth verifying end to end
  because a single 1500-byte hop in the middle reintroduces A7.

**Layer 3 — Customer private access to your API.**
**PrivateLink**: put the API behind an NLB in a dedicated services VPC,
create a VPC endpoint *service*, and grant specific customer accounts
permission to create endpoints against it.
Why this rather than peering or VPN: the customer's CIDR is unknown and may
overlap with yours (A10, A11); PrivateLink has no routing relationship at
all, so overlap is irrelevant. It's unidirectional — they reach your service,
you get no path into their VPC — which is both a security property and an
easier sell to their security team. And it's per-service, so you're exposing
one API, not a network.
Details that matter: it's regional, so a customer in another region needs
either an endpoint reachable via their own inter-region path or an endpoint
service in each region you serve; the NLB requirement means L7 features must
live behind it (ingress/Envoy inside your VPC); and you need DNS —
private DNS names on the endpoint service so the customer uses your normal
hostname.
Provide a documented alternative for customers who aren't on AWS: public
endpoint with mTLS and IP allowlisting, or a site-to-site VPN. Don't make
PrivateLink the only option or you've constrained your addressable market to
one cloud.

**Layer 4 — Egress and DNS.**
- Centralised egress through an inspection VPC attached to the TGW if there's
  a compliance requirement for outbound inspection; otherwise per-VPC NAT
  gateways, which is cheaper and has a smaller blast radius. State the
  tradeoff rather than defaulting.
- Gateway VPC endpoints for S3/DynamoDB everywhere, to keep that traffic off
  NAT (A3, A13).
- Route 53 Resolver inbound and outbound endpoints in a shared-services VPC
  (A12), with forwarding rules shared across accounts via RAM, so colo can
  resolve cloud names and vice versa.

**What I'd call out as the risks:**
1. The TGW is now the blast radius for everything. A bad route table change
   is a multi-region outage. That argues for infrastructure-as-code with
   plan review, separate route tables per environment, and change windows.
2. The DX pair is the physical single point of concern; the VPN backup must
   be *tested*, not merely configured.
3. Cost: TGW per-GB processing across three regions plus inter-region
   transfer is a large recurring line item. I'd instrument it from day one
   and expect to bypass the TGW for the top one or two flows.
4. IPAM discipline. Every one of these layers fails on overlapping CIDRs, so
   central address allocation is the foundational dependency.

**Weak answers miss.** PrivateLink for the customer requirement (many
propose VPN or peering, which fails on unknown/overlapping customer CIDRs),
the VPN-as-DX-backup with BGP, and that DX is unencrypted. A candidate who
doesn't mention IPAM has designed something that will break on the first
acquisition or the first customer with a `10.0.0.0/16`.

**Follow-ups to expect.**
- What's your failure story if a region goes away? (Connectivity is the easy
  part — TGW peering and DX Gateway already reach the others. The hard part
  is data and control plane, which is a different design question. Say
  where the boundary of this answer is.)
- Why not run everything through the internet with mTLS? (Defensible, and
  worth engaging honestly: it's simpler, cheaper, and cloud-agnostic. It
  loses on consistent latency for GPU dataset movement, on egress cost at
  volume, and on the customer's security requirement for a non-internet
  path. Those are the three things that justify the complexity.)
