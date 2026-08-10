# TCP/IP & the Wire — Answers

Grade yourself on three things per question: the mechanism, the failure mode,
and whether you could have handled the follow-ups. An answer with no failure
mode is a junior answer.

---

## Tier 1 — Recall

### A1. The three-way handshake, and why three messages

**Answer.**
Client sends `SYN` with its initial sequence number (ISN) and its options —
MSS, window scale factor, SACK-permitted, timestamps. Server replies
`SYN-ACK`: its own ISN plus an acknowledgement of the client's, and its own
option set. Client sends `ACK`. Both sides now have a sequence number they
know the peer has seen.

Three rather than two because the sequence-number agreement has to happen in
*both* directions, and each direction's ISN needs to be acknowledged. Two
messages would establish the client→server direction only. The deeper reason
is duplicate-detection: ISNs are randomised (originally clock-based, now
cryptographically derived from the 4-tuple plus a secret) so that a delayed
duplicate `SYN` from an old incarnation of the same 4-tuple cannot be
mistaken for a new connection. Random ISNs are also what makes off-path
connection spoofing hard.

Note the option negotiation is one-shot: window scaling and SACK are only
negotiable in the handshake. If a middlebox strips the window-scale option
from the `SYN`, you are stuck at a 64 KB receive window for the life of the
connection — which is the mechanism behind a large class of "fast link, slow
transfer" bugs.

**Weak answers miss.** That the handshake carries option negotiation, and
that this is where window scaling gets decided. Most candidates recite
SYN/SYN-ACK/ACK and stop, which tells the interviewer nothing.

**Follow-ups to expect.**
- What is in the backlog queue, and what is the difference between the SYN
  queue and the accept queue?
- What happens if the accept queue is full? (Depends on
  `tcp_abort_on_overflow`: silently drop the ACK — the client thinks it's
  connected and the server retransmits SYN-ACK — or send `RST`.)
- How does TCP Fast Open change this, and why isn't it widely used?

---

### A2. MTU vs MSS

**Answer.**
MTU is a **link-layer** property: the largest IP packet, headers included,
that an interface will transmit. Classic Ethernet is 1500 bytes; jumbo frames
are typically 9000; tunnels subtract their own overhead.

MSS is a **TCP** property: the largest *payload* a segment may carry,
excluding IP and TCP headers. Over IPv4 Ethernet with no options that's
1500 − 20 (IP) − 20 (TCP) = 1460.

MTU is configured or negotiated on the interface. MSS is advertised by each
side in its `SYN` — and it's not a negotiation, it's each side telling the
other "do not send me segments larger than this." Each side then uses
min(peer's advertised MSS, its own path-derived MSS).

The reason to know the distinction cold: every tunnel you add (IPsec, GRE,
VXLAN, WireGuard) eats bytes from the payload budget, and the standard fix is
**MSS clamping** on the tunnel endpoint — rewriting the MSS option in
transiting SYNs so both ends pick a size that fits — precisely because you
cannot rely on PMTUD working (see A9).

**Weak answers miss.** That MSS is advertised per-direction and is not a
negotiated minimum, and that clamping is a SYN-time intervention — it does
nothing for connections already established, and nothing for UDP at all.

**Follow-ups to expect.**
- Your VPN drops large POSTs but pings fine. Why? (See A9.)
- Where would you clamp, and what does the iptables/nftables rule look like
  conceptually? (`TCPMSS --clamp-mss-to-pmtu` on FORWARD, tcp flags SYN.)
- Do jumbo frames help you? (Only if every hop in the L2 path agrees;
  mismatched MTU inside a broadcast domain is a nasty intermittent failure.)

---

### A3. TIME_WAIT

**Answer.**
The side that initiates the close — sends the first `FIN` — enters
`TIME_WAIT` after it has both sent its FIN and acknowledged the peer's. It
stays there for 2×MSS (Maximum Segment Lifetime); Linux hardcodes this at
60 seconds and it is not tunable via sysctl.

Two jobs. First, if the final `ACK` is lost, the peer retransmits its `FIN`
and someone has to be around to re-ACK it — otherwise the peer receives a
`RST` and may treat a clean close as an error. Second and more important:
it prevents a delayed segment from the old connection being delivered into a
*new* connection that reuses the same 4-tuple.

The operational consequence: `TIME_WAIT` accumulates on whichever side closes
first, and it holds the 4-tuple. On a busy client or proxy that closes
connections, you can exhaust the ephemeral port range against a single
destination (see A17). On a *server* that closes first, tens of thousands of
`TIME_WAIT` sockets is normal and mostly harmless — they cost a small amount
of memory, not a port, because the local port is the listening port and the
4-tuple varies by client.

Correct fixes: use keepalive/connection pooling so you close far less often;
enable `tcp_tw_reuse` (safe for *outbound* connections, relies on TCP
timestamps); add source IPs or destination ports to widen the tuple space.
The wrong fix is `tcp_tw_recycle`, which broke NAT'd clients and was removed
from Linux in 4.12.

**Weak answers miss.** That TIME_WAIT is only a scaling problem for the side
that *initiates* the close, and that server-side TIME_WAIT is usually a
non-issue. Candidates who say "we set tcp_tw_recycle" are telling you they
last read this in 2013.

**Follow-ups to expect.**
- Why is `tcp_tw_reuse` safe but `tcp_tw_recycle` not?
- How do you decide which side should close? (Whichever side you'd rather
  pay the cost on; HTTP servers closing idle keepalives is the common design.)
- What does `SO_REUSEADDR` actually do, and is it related? (It lets you
  *bind* to a port with sockets in TIME_WAIT — different problem.)

---

### A4. CLOSE_WAIT

**Answer.**
`CLOSE_WAIT` means the peer sent a `FIN` — the kernel ACKed it and the
connection is half-closed — and **your application has not called `close()`
on the file descriptor**. The socket sits in `CLOSE_WAIT` indefinitely; there
is no timeout that clears it, because as far as TCP is concerned you may
still have data to send.

So it is essentially always a bug on the side showing `CLOSE_WAIT`: a leaked
descriptor. Typical causes are an exception path that skips the close, a
connection pool that doesn't reap, or a library where the caller must close a
response body and doesn't.

The symptom that follows is FD exhaustion: `accept()` starts returning
`EMFILE`, and the service stops taking new connections while looking
otherwise healthy. Growing `CLOSE_WAIT` count is one of the highest-signal,
lowest-noise metrics you can alert on.

**Weak answers miss.** That it never self-heals and that the fault is
local, not remote. Many candidates blame the peer.

**Follow-ups to expect.**
- How do you find the leaking code path from a running process?
  (`ss -tanp` to get the PID, `ls -l /proc/<pid>/fd`, then the stack — or a
  bpftrace probe on `close()`/`tcp_close`.)
- What's the difference between `close()` and `shutdown()`?
- Can you have a half-open connection that's useful? (Yes — `shutdown(WR)`
  to signal end-of-input while still reading, e.g. old-style HTTP.)

---

### A5. RST vs FIN

**Answer.**
`FIN` is the graceful path: "I have no more data to send." It's part of the
ordered byte stream, so data sent before it is delivered first, and the peer
ACKs it. `RST` is abortive: it tears the connection down immediately,
discards anything queued in both directions, and is not acknowledged.

Situations that produce `RST`:
1. **Connecting to a port with no listener** — the classic
   `ECONNREFUSED`. (You get this only if nothing is filtering; a firewall
   that `DROP`s gives you a timeout instead, which is how you tell them apart.)
2. **Writing to a socket the peer has already closed** — the peer's kernel
   has no state for the 4-tuple, so it resets. Application-visible as
   "connection reset by peer" on the next write, often one request *after*
   the connection actually died. This is the mechanism behind idle-timeout
   502s (see the load balancing topic).
3. **`close()` on a socket with unread data in the receive buffer**, or a
   socket configured with `SO_LINGER` timeout 0 — the kernel sends `RST`
   rather than `FIN` because the application is discarding data.
4. Also: a middlebox or firewall injecting a forged `RST` to terminate a
   flow, and a half-open connection surviving a peer reboot (peer has no
   state, resets on the next segment).

**Weak answers miss.** Case 2, which is the one that actually shows up in
production, and the fact that firewall DROP vs REJECT is diagnosable from
timeout-vs-refused.

**Follow-ups to expect.**
- Your client sees "connection reset by peer" only on the first request
  after an idle period. Explain.
- Can an off-path attacker inject a RST? (Needs to guess the 4-tuple and a
  sequence number inside the window — the window check and randomised ISNs
  are what make it hard; RFC 5961 tightened this further.)

---

### A6. Flow control vs congestion control

**Answer.**
**Flow control** protects the *receiver*. The receiver advertises a window
(`rwnd`) saying how much buffer space it has; the sender may not have more
than that unacknowledged in flight. It's end-to-end and explicit — the number
is literally in every ACK. Window scaling (negotiated in the handshake)
extends the 16-bit field so you can exceed 64 KB.

**Congestion control** protects the *network*. There is no explicit signal,
so the sender maintains its own estimate, `cwnd`, and adjusts it based on
inferred congestion — loss, delay, or ECN marks. Slow start ramps
exponentially until a threshold or a loss event; congestion avoidance then
grows conservatively.

The sender is limited by `min(rwnd, cwnd)`. That's the sentence to say — it
tells the interviewer you know they're separate limits that compose.

Diagnostically they look different: an `rwnd`-limited connection has a small
advertised window and a slow receiver (application not reading fast enough);
a `cwnd`-limited connection shows retransmits and a small congestion window.
`ss -ti` shows you both.

**Weak answers miss.** The composition, and the diagnostic difference. Also
often missed: a receiver that doesn't `read()` fast enough is a *flow
control* problem that looks like a network problem to the app team.

**Follow-ups to expect.**
- What is ECN and why isn't it universally on? (Middlebox mangling,
  historically; increasingly deployed now, especially L4S work.)
- What is a "zero window" and how does the sender recover? (Window probes.)

---

### A7. Bandwidth-delay product

**Answer.**
BDP = bandwidth × round-trip time. It's the amount of data that can be "on
the wire" in flight at any moment, and therefore the minimum in-flight
window you need to keep a link saturated.

Worked example: 1 Gbps between US-East and Frankfurt at ~80 ms RTT.
1e9 bits/s × 0.08 s = 8e7 bits = 10 MB. To use the full gigabit you need a
10 MB window. Without window scaling you're capped at 64 KB, giving
64 KB / 0.08 s ≈ 800 KB/s ≈ 6.5 Mbps — under 1% of the link.

That's the whole reason to know BDP: it explains "we bought a fat pipe and
a single transfer still crawls." The fixes are window scaling (must be
negotiated in the SYN, so a middlebox stripping it is fatal), adequate
socket buffers on both ends (`tcp_rmem`/`tcp_wmem` autotuning limits), and
parallel streams as a blunt workaround.

**Weak answers miss.** Doing the arithmetic. Say the numbers out loud —
this is a question where the interviewer wants to see you compute, and
"you need a bigger window" without the multiplication is a thin answer.

**Follow-ups to expect.**
- Buffer bloat: what happens if intermediate buffers are much larger than
  the BDP? (Loss-based CC fills them, RTT inflates, latency collapses for
  everyone sharing the queue — the motivation for BBR, CoDel, fq_codel.)
- Why do people use parallel TCP streams for bulk transfer, and what's the
  downside? (More aggregate cwnd; unfair to other flows, and doesn't fix
  the underlying tuning.)

---

## Tier 2 — Explain / compare

### A8. CUBIC vs BBR

**Answer.**
**CUBIC** (Linux default) is **loss-based**. It grows `cwnd` along a cubic
curve, treats packet loss as the congestion signal, and multiplicatively
backs off on loss. Its weakness follows directly: it needs loss to find the
limit, so on a path with deep buffers it fills them before backing off
(bufferbloat, inflated RTT), and on a path with *non-congestive* loss —
lossy wireless, a link with a small random error rate — it interprets that
loss as congestion and collapses throughput far below what the path can
carry.

**BBR** (Google) is **model-based**. It continuously estimates the path's
bottleneck bandwidth and minimum RTT, and paces sending at roughly
bandwidth × min-RTT. It doesn't need loss to find the operating point, so it
performs far better on long-fat paths with sporadic loss — the canonical win
is intercontinental transfers and lossy last miles.

Where BBR is worse: **fairness**. BBRv1 was well documented as aggressive
against CUBIC flows sharing a bottleneck, and could sustain meaningful
standing loss in shallow-buffered links because it doesn't respond to loss
the way its neighbours do. BBRv2/v3 work has specifically targeted loss and
ECN responsiveness and inter-flow fairness; if you assert current fairness
properties, verify the version — this has moved a lot and is worth checking
against current material rather than asserting.

Judgment call, mine: BBR is the right default for long-haul traffic you
control end-to-end (inter-region replication, CDN origin pull). For traffic
sharing a bottleneck with third parties you don't control, the fairness
question is real and I'd measure before switching fleet-wide.

**Weak answers miss.** *Why* loss-based breaks — conflating congestion with
corruption — and that BBR's advantage comes with a fairness cost rather than
being a free win.

**Follow-ups to expect.**
- What does pacing require from the sender? (fq qdisc / TSO-aware pacing.)
- How would you A/B test a congestion control change safely?
- Where do you set it, and can you set it per-route? (`net.ipv4.tcp_congestion_control`,
  or per-route via `ip route ... congctl`.)

---

### A9. Path MTU Discovery and the black hole

**Answer.**
Classic PMTUD (IPv4): the sender sets the **Don't Fragment** bit on all
packets. If a router along the path has an MTU smaller than the packet, it
drops it and returns **ICMP Type 3 Code 4** — "fragmentation needed and DF
set" — which carries the next-hop MTU. The sender caches that per-destination
and reduces its segment size.

The whole mechanism depends on that ICMP message arriving. The failure mode
is that a firewall or security group blocks ICMP wholesale — a depressingly
common "hardening" default. Now the large packets are dropped and the sender
never learns why. This is a **PMTUD black hole**.

The signature is unmistakable once you've seen it: small requests succeed,
the TCP handshake succeeds (SYNs are small), pings succeed — and then any
request with a large body, or any response with a large payload, hangs and
eventually times out. "SSH connects but hangs on the banner." "Small GETs
fine, POSTs die." It is almost always a tunnel: IPsec, GRE, VXLAN, WireGuard,
or a cloud VPN.

Diagnosis: `ping -M do -s <size>` sweeping the payload size to find the
largest that survives (remember to add 28 bytes for ICMP+IP headers when
converting to MTU), or `tracepath`. Confirm by watching a `tcpdump` where you
see the same large segment retransmitted repeatedly with no ACK, while small
segments flow fine.

Fixes, in order of preference: **MSS clamping** on the tunnel endpoint (fixes
TCP for everyone, doesn't depend on ICMP), then allowing ICMP type 3 through
the firewall, then lowering interface MTU. Linux also has
`tcp_mtu_probing` (PLPMTUD, RFC 4821), which infers the MTU from black-holed
segments without needing ICMP — worth enabling on tunnel-heavy fleets.

**Weak answers miss.** Naming the specific ICMP type/code, and the
diagnostic signature ("handshake fine, payload dies") that makes this
identifiable in thirty seconds. Also missed: clamping only helps TCP; UDP
over a tunnel needs the application to handle it (this is why QUIC does its
own PMTU probing).

**Follow-ups to expect.**
- Why does clamping only work on SYN packets?
- How does QUIC handle this? (DPLPMTUD in the transport itself, since it
  can't rely on ICMP either.)
- Your Kubernetes pods can't talk across nodes for large payloads only.
  First guess? (Overlay encapsulation overhead vs node MTU — VXLAN costs
  50 bytes, so 1500 MTU nodes need ~1450 pod MTU.)

---

### A10. Nagle + delayed ACK

**Answer.**
**Nagle's algorithm** (sender side): if there is unacknowledged data
outstanding, buffer small writes and don't send until either a full MSS
accumulates or the outstanding data is ACKed. Purpose: stop a telnet session
from putting one byte in each 41-byte packet.

**Delayed ACK** (receiver side): don't ACK immediately; wait — up to 40 ms
on Linux, up to 200 ms on some stacks — hoping to piggyback the ACK on
response data or to ACK two segments at once.

Together they deadlock each other. The sender has a small write pending and
is waiting for an ACK before sending it. The receiver has nothing to send
back and is sitting on its delayed-ACK timer. Nothing moves until the timer
fires. The application sees a latency spike of tens to hundreds of
milliseconds, on some requests only, with no CPU load and nothing in the
logs to explain it.

The trigger pattern is a write-write-read sequence: an app that writes a
header, then writes a body as a separate `write()`, then blocks on the
response. The second write gets Nagled.

Fixes: set `TCP_NODELAY` (disables Nagle) — which is what essentially every
RPC library and HTTP server does by default today; or fix the application to
issue a single write (`writev`/vectored I/O, or buffer and flush once), which
is the better fix because it also cuts syscalls.

**Weak answers miss.** That the *application's* write pattern is the
trigger, and that `TCP_NODELAY` is treating the symptom. Also: candidates who
say "always disable Nagle" without knowing what it was for.

**Follow-ups to expect.**
- What's `TCP_CORK`/`TCP_NOPUSH` and when would you want the opposite
  behaviour? (Explicitly batch — sendfile-style responses.)
- How would you confirm this is what's happening from a packet capture?
  (A gap of ~40 ms between a small segment and its ACK, repeatedly.)

---

### A11. Choosing UDP

**Answer.**
Reach for UDP when TCP's guarantees are actively wrong for you, not just
when you want it faster. Concretely:

- **Latency-critical data where stale data is worthless.** Real-time voice
  and video: retransmitting a 300 ms-old audio frame is worse than
  concealing the loss. This is why LiveKit/WebRTC media flows over UDP.
- **You need your own loss/ordering semantics.** QUIC exists because
  head-of-line blocking and the ossified TCP handshake couldn't be fixed in
  TCP; it rebuilds reliability per-stream on top of UDP.
- **Request/response smaller than a handshake.** DNS. Paying 1 RTT of
  handshake for a 100-byte exchange doubles your latency.
- **Multicast/broadcast**, which TCP simply cannot do.

What you have to rebuild if you take it on: reliability (ACKs and
retransmission), ordering, connection state and liveness, **congestion
control** — the one people forget, and the one that makes a naive UDP
protocol a good network citizen or a menace — flow control, path MTU
discovery, and encryption (no TLS to lean on; you're using DTLS or rolling
QUIC).

The honest senior answer is usually: "unless one of those four cases
applies, use TCP, or use QUIC and get someone else's implementation of all
of the above." Also mention the operational tax: UDP is more likely to be
blocked or rate-limited by middleboxes and corporate firewalls, so anything
UDP-based needs a TCP fallback path.

**Weak answers miss.** Congestion control, and the middlebox/fallback
reality.

**Follow-ups to expect.**
- What's the amplification risk in a UDP service you expose? (Spoofed source
  + a response larger than the request = a DDoS reflector. Response must not
  exceed request size before the client is validated — QUIC enforces a 3x
  amplification limit.)
- Why is QUIC in userspace, and what does that cost? (Deployability and
  iteration speed; costs CPU — more syscalls, no equivalent of the mature
  kernel offload path, though GSO/GRO for UDP has narrowed the gap.)

---

### A12. Head-of-line blocking across HTTP versions

**Answer.**
Two distinct layers, and the answer must separate them.

**HTTP/1.1**: one request in flight per connection. A slow response blocks
everything behind it on that connection — *application-level* HOL. Browsers
worked around it with 6 parallel connections per host; pipelining was
specified but effectively undeployable because responses still had to return
in order and middleboxes broke it.

**HTTP/2**: multiplexes many streams over one TCP connection, which fixes
the application-level blocking. But all streams share one TCP byte stream, so
a single lost packet stalls delivery of *every* stream until it's
retransmitted — *transport-level* HOL. Net effect: H2 is better than H1 on a
clean network and can be **worse** on a lossy one, because H1's six
connections meant a loss only stalled one of six.

**HTTP/3**: runs over QUIC, which implements independent streams with
per-stream ordering directly over UDP. A lost packet stalls only the stream
whose data it carried. QUIC also folds the transport and crypto handshake
together (1-RTT establishment, 0-RTT on resumption) and supports connection
migration across IP changes via connection IDs — genuinely valuable on
mobile.

The honest caveat: 0-RTT data is replayable by design, so it must only carry
idempotent requests. And QUIC's userspace implementation costs more CPU per
byte than kernel TCP, which matters at CDN scale.

**Weak answers miss.** That H2 can be worse than H1 under loss — that's the
insight the question is fishing for.

**Follow-ups to expect.**
- Does H2 HOL blocking matter inside a datacentre? (Much less — near-zero
  loss, so H2 multiplexing is close to free. Which is why gRPC over H2 is
  fine internally and QUIC's benefit is mostly at the edge.)
- What is 0-RTT replay and how do you defend? (Restrict to idempotent
  methods; server-side single-use tickets/strike registers.)

---

### A13. TCP keepalive vs application heartbeat

**Answer.**
**TCP keepalive** is a kernel feature, off by default, and its defaults are
useless for failure detection: Linux waits `tcp_keepalive_time` = 7200 s
(2 hours) idle before the first probe, then 9 probes 75 s apart. Left alone
it detects a dead peer in over two hours. It must be enabled per-socket
(`SO_KEEPALIVE`) and tuned per-socket (`TCP_KEEPIDLE`, `TCP_KEEPINTVL`,
`TCP_KEEPCNT`) to be useful.

What it detects: the *connection* is gone — peer rebooted, NAT/firewall
dropped the flow's state, cable pulled. What it does **not** detect: the peer
process is alive at the TCP level but wedged — deadlocked, GC-stalled,
blocked on a dependency. The kernel happily ACKs keepalives while the
application does nothing.

**Application heartbeat** detects liveness of the thing you actually care
about — the application loop, and if you design it well, the application's
ability to *do work*. It also lets you carry information (load, drain
signals, protocol version) and to distinguish "slow" from "dead," which the
kernel cannot.

Practically both have a role. Keepalive with tuned timers is the cheap
defence against silently dropped NAT state on long-lived idle connections
(cloud NAT gateways commonly idle out flows in the minutes range — verify
the current value for your provider). Application heartbeats are what your
failure detector should key on.

The trap in the heartbeat design: heartbeat on the same connection and same
path as real traffic, or you're measuring a path you don't use. And set the
timeout with an eye on false positives — an aggressive heartbeat during a GC
pause evicts a healthy node and can cascade.

**Weak answers miss.** The 2-hour default (say the number), and the
"TCP-alive but application-dead" distinction, which is the actual point.

**Follow-ups to expect.**
- Your service sits behind a cloud NAT gateway and long-lived connections die
  after ~5 minutes idle. Fix? (Keepalive interval below the NAT idle timeout.)
- How do you pick a heartbeat timeout? (See phi-accrual / failure detection
  in the distributed systems topic — there is no timeout that is both fast
  and never wrong.)

---

### A14. SYN flood and SYN cookies

**Answer.**
The attack exploits the fact that a half-open connection costs the server
state. Attacker sends `SYN` packets with spoofed sources; the server
allocates a request-socket entry in the SYN queue, replies `SYN-ACK`, and
waits for an `ACK` that never comes, retransmitting the SYN-ACK several times
first. The SYN queue fills, and legitimate `SYN`s get dropped. It costs the
attacker one small packet per unit of server state — excellent asymmetry.

**SYN cookies** remove the state. Instead of storing the half-open
connection, the server encodes the necessary information into the ISN it
sends in the `SYN-ACK`: a slowly-changing timestamp counter, an MSS index,
and a MAC over the 4-tuple keyed with a server secret. It then *forgets* the
connection entirely. When a legitimate `ACK` arrives, its acknowledgement
number is cookie+1, so the server recomputes the MAC, validates it, and
reconstructs the connection from scratch. Spoofed sources never send that
ACK, so they cost nothing.

What you give up: the ISN has limited bits, so the server cannot preserve
the full option set from the original SYN. Historically that meant losing
window scaling, SACK and timestamps — a real throughput penalty for
legitimate connections. Linux mitigates this by encoding an MSS index and,
when TCP timestamps are available, stashing option bits in the timestamp
field. It also only engages cookies when the SYN queue actually overflows
(`net.ipv4.tcp_syncookies = 1`), so the cost is paid only under attack.
Setting it to `2` forces them always on, which you generally don't want.

**Weak answers miss.** That the tradeoff is TCP option loss, and that the
default mode is "only on overflow" — so "we turned on syncookies" is not a
throughput regression in normal operation.

**Follow-ups to expect.**
- What else defends? (Rate limiting and SYN proxying at the edge/scrubbing
  provider; increasing `tcp_max_syn_backlog` buys a little headroom;
  `tcp_synack_retries` reduces the retransmit amplification.)
- Why can't you just block the source IPs? (They're spoofed and drawn from
  the whole address space; you'd be blocking your users.)
- Does this help against a flood from real, non-spoofed clients? (No — those
  complete the handshake. That's a different problem, solved with connection
  limits and load shedding.)

---

### A15. IP fragmentation, and IPv6

**Answer.**
Why it's harmful:

- **All-or-nothing reassembly.** Lose one fragment and the entire original
  datagram is discarded, so the effective loss rate is multiplied by the
  fragment count. With TCP on top you then retransmit the whole segment.
- **Reassembly is stateful and attackable.** The receiver must buffer
  fragments until the set completes, with a timeout. That's a memory
  reservation an attacker can drive with incomplete fragment sets, and it's
  the basis of a family of overlapping-fragment evasion and DoS attacks.
- **Only the first fragment carries the L4 header.** Subsequent fragments
  have no port numbers, so firewalls, ECMP hashing, and load balancers can't
  classify them. Many middleboxes therefore drop non-initial fragments
  outright, and ECMP may hash fragments of one datagram down different paths.
- **It defeats hardware offload** on many NICs.

IPv6 changed the model: **routers do not fragment**. Only the source host
may fragment, using a Fragment extension header, and a router that can't
forward a packet returns ICMPv6 **Packet Too Big**. This makes working PMTUD
mandatory rather than optional — and it's why blocking ICMPv6 breaks IPv6
much more severely than blocking ICMP breaks IPv4. IPv6 also mandates a
minimum link MTU of 1280 bytes, so 1280 is the safe floor.

Practical upshot: design so you never fragment. Clamp MSS for TCP; for UDP,
keep datagrams under the path MTU (this is why DNS-over-UDP responses above
512/1232 bytes are a perennial problem and why EDNS0 buffer sizing matters,
and why QUIC does its own probing).

**Weak answers miss.** The "only the first fragment has ports" consequence
for firewalls and ECMP, and that IPv6 pushed fragmentation to the source.

**Follow-ups to expect.**
- Why is 1232 a common DNS EDNS0 buffer size? (1280 IPv6 minimum MTU minus
  IPv6 + UDP headers — chosen to never fragment.)
- What happens to a fragmented packet arriving at an ECMP-hashed LB?

---

## Tier 3 — Scenario / debug

### A16. Intermittent ~5 second stalls

**Answer.**
Lead with the observation that makes this tractable: **the number is
suspiciously round**. Almost-exactly-5 s is a *timeout* firing, not a
resource contention — contention produces a distribution, timeouts produce a
spike at a constant. So the question becomes "whose 5-second timer is this?"

Hypotheses, ordered by how often they're the answer:

1. **DNS.** The default resolver timeout on glibc is 5 seconds. A dropped
   query or a non-responding nameserver produces exactly this. The classic
   variant in containers is the parallel A/AAAA lookup race — a
   well-documented conntrack race in Linux DNAT caused one of the two UDP
   replies to be dropped, so the resolver waited out its timeout and retried.
   Check: is resolution happening per-request (no connection reuse / no DNS
   cache)? Does `search` domain expansion mean each lookup is really 4–5
   queries? Measure with a tcpdump on port 53 and correlate timestamps
   against the slow requests.
2. **TCP retransmission of the initial SYN.** Linux's initial RTO is 1 s,
   doubling — so a lost SYN costs 1 s, two lost costs 3 s, three costs 7 s.
   Not exactly 5, which argues against it here, but adjacent losses in the
   stream produce multi-second stalls at similar magnitudes. Look at
   `netstat -s` retransmit counters and `ss -ti` per-socket.
3. **Connection pool exhaustion with a 5 s acquire timeout** in the client
   library. Very common and often misattributed to the network. Check
   whether the stall is on connection acquisition or on the wire — those are
   different spans and your tracing should separate them.
4. **A downstream/middleware timeout set to 5 s** — a client library
   default, a service mesh sidecar, an LB idle setting.
5. **PMTUD black hole** (A9) — but that would correlate with payload size,
   not be random, so check whether the slow requests share a size profile.

Method I'd actually run: correlate the slow requests on one dimension at a
time — same client, same upstream, same payload size, same time-of-day. Then
capture packets on the affected host filtered to the slow request's 5-tuple
and look at where the gap lands: before the SYN (DNS or pool), between SYN
and SYN-ACK (SYN loss), or mid-stream (retransmit / delayed ACK / receiver
stall). The packet capture answers the question definitively in a way that
metrics cannot, because it timestamps the gap.

**Weak answers miss.** The "round number means timeout" reasoning — this is
the single most transferable idea in the answer. Also missed: distinguishing
where the gap sits in the packet timeline, which collapses five hypotheses to
one.

**Follow-ups to expect.**
- How would you eliminate DNS from the request path entirely? (Connection
  pooling with long-lived connections, a local caching resolver such as
  NodeLocal DNSCache, or resolving to IPs at startup and refreshing out of
  band — each has a failure mode; say which you'd accept.)
- If it's DNS in Kubernetes, what specifically? (ndots:5 causing search-path
  expansion — a lookup for an external name issuing several needless
  queries first; plus conntrack races on UDP DNAT.)
- Why doesn't your p99 dashboard show this clearly? (Small fraction of
  requests; you need p99.9 and a histogram, and this is a good place to
  mention that averaging percentiles across instances is invalid.)

---

### A17. Egress proxy running out of connections

**Answer.**
This is **ephemeral port exhaustion**, and the tell is that the failure is
immediate (`EADDRNOTAVAIL` / "cannot assign requested address") rather than
a timeout — timeouts mean the packet went out and nothing came back; an
immediate local error means the kernel couldn't even allocate a socket.

The arithmetic. A TCP connection is identified by the 4-tuple
(src IP, src port, dst IP, dst port). Outbound to a *fixed* destination
IP:port, the only thing that varies is your source port. Linux's default
ephemeral range (`net.ipv4.ip_local_port_range`) is 32768–60999 — about
28,000 ports. So one source IP to one destination IP:port caps at ~28k
concurrent connections.

And it's worse than concurrency, because of `TIME_WAIT` (A3): the proxy
initiates the close, so each finished connection holds its port for 60 s.
At a steady rate of *R* new connections per second to one destination, the
ports in use are roughly R × 60. At R = 500/s that's 30,000 — you exhaust the
range at around 470 connections/second even if none are concurrently active.
That's the calculation to say out loud; it's why "we only have 200 active
connections" doesn't save you.

Options, best first:

1. **Stop opening connections.** Keepalive + connection pooling to the
   upstreams. If you can serve 500 req/s over 50 pooled connections, the
   problem evaporates. This is nearly always the right answer and it's also
   cheaper on latency (no handshake, no TLS handshake).
2. **Widen the tuple.** More source IPs — additional IPs on the proxy,
   more proxy instances, or (in cloud) more NAT gateway addresses. Each
   source IP buys another ~28k. Or more destination IPs/ports if the upstream
   has them.
3. **Widen the range**: `ip_local_port_range` to 1024–65535 buys ~2.3x. A
   real but small win; don't let it be your only fix.
4. **`net.ipv4.tcp_tw_reuse=1`** lets the kernel reuse a TIME_WAIT socket for
   a new outbound connection when timestamps make it safe. This is the
   targeted fix for the TIME_WAIT-driven portion. Not `tcp_tw_recycle` —
   removed from the kernel, and it broke NAT'd peers.
5. **`SO_REUSEPORT` / explicit bind-before-connect** with your own port
   accounting, if you're writing the proxy — rarely worth it.

Cloud-specific note worth raising: a managed NAT gateway has its own
per-destination port limits and its own idle timeouts, and the failure
presents as connection errors that look like the upstream's fault. That's a
common and expensive surprise — mention it, since the resume context is
cloud fleets.

**Weak answers miss.** The R × 60 arithmetic — connecting TIME_WAIT to a
*rate* limit rather than a concurrency limit is the staff-level move here.

**Follow-ups to expect.**
- How would you monitor for this before it pages? (`ss -s` socket counts by
  state, `netstat -s` for "port unreachable"/bind failures, and a synthetic
  headroom metric: ports used / range size per destination.)
- Does TLS change the math? (Not the port math, but it raises the cost of
  *not* pooling — full handshakes are RTT- and CPU-expensive.)

---

### A18. Proving packet loss between two datacentres

**Answer.**
State the method before the tools: I want to (a) establish whether loss is
real, (b) locate it, (c) rule out the two things that masquerade as loss —
application slowness and reordering.

**Step 1 — Is the network even implicated?** Compare an application-level
latency measurement against a transport-level one. If `ss -ti` shows a
healthy RTT, no retransmits, and a full congestion window while the app is
slow, the network is exonerated and I stop here. This step saves the most
time and candidates skip it.

**Step 2 — Cheap continuous signal.** `mtr` (or `mtr --tcp --port <p>` to
follow the actual traffic's path, since ICMP may be policed differently) run
long, both directions. The crucial reading skill: **loss reported at an
intermediate hop that does not persist to subsequent hops is not loss** — it's
that router deprioritising its own ICMP responses. Only loss that continues
at every hop from that point onward, and at the final destination, counts.
Also: routing can be asymmetric, so run it from both ends; the return path
may be the broken one.

**Step 3 — Kernel counters, which are free and unambiguous.**
`nstat`/`netstat -s` deltas over an interval: `TcpRetransSegs`,
`TcpExtTCPLostRetransmit`, `TcpExtTCPSackRecovery`,
`TcpExtTCPDSACKRecv`. Interface-level: `ip -s link` and `ethtool -S` for
`rx_dropped`, `rx_missed_errors`, CRC errors, and NIC ring-buffer overruns —
this distinguishes "loss in the fabric" from "loss on this host because the
receive queue overflowed," which is a completely different fix (RSS/ring
sizing/IRQ affinity, not a network ticket).

**Step 4 — Distinguish loss from reordering.** This is the question's real
teeth. Both produce duplicate ACKs and can trigger fast retransmit, so
retransmit count alone doesn't separate them. Use **DSACK**: if the receiver
reports "I already had that segment," the retransmission was unnecessary,
which means the original arrived — late. High `TCPDSACKRecv` with low actual
loss is the signature of reordering, typically from ECMP spraying a flow
across unequal paths or from LAG rebalancing. In a capture, reordering shows
segments arriving out of sequence and then filling in; loss shows a gap that
only closes on retransmission.

**Step 5 — Congestion vs corruption vs policing.** Congestion loss
correlates with RTT inflation (queues filling before they drop) and with
time-of-day/utilisation. Corruption (bad optic, marginal cable) is roughly
constant-rate, uncorrelated with load, and shows up as CRC/FCS errors on an
interface counter. Policing shows as a hard cliff at a specific rate with no
RTT rise. Plot loss against link utilisation — if they correlate you have
congestion and the fix is capacity or QoS; if they don't, go look for
hardware.

**Step 6 — Capture, targeted.** `tcpdump -s 128` on both ends filtered to
one flow, then compare: a packet present in the sender's capture and absent
from the receiver's localises the loss to the path; present in both but the
app didn't see it localises it to the host. Keep the capture small and
snap-limited — a full capture on a busy host is itself an incident.

**Tools worth naming for continuous measurement rather than incident
response:** a mesh prober between sites (each site probes every other on a
schedule) so you have the baseline *before* the incident; that is the
difference between an argument with the network team and a ticket with
evidence.

**Weak answers miss.** Step 1 (ruling out the application), the ICMP-hop
reading rule in mtr, and DSACK for reordering. Any candidate who opens with
"I'd run ping" and never mentions kernel counters is describing something
they read, not something they did.

**Follow-ups to expect.**
- The loss is only on flows between two specific racks. Now what? (Suspect a
  specific ECMP path/LAG member; hash the flow differently by changing source
  port to test — a flow whose loss disappears when you change source port
  indicts one member of a bundle.)
- How much loss is acceptable? (Depends entirely on the CC algorithm and RTT:
  at 80 ms RTT, even 0.1% loss meaningfully caps CUBIC throughput —
  connect this back to A8.)
- What if it's in a cloud where you can't see the fabric? (Your evidence is
  host counters plus a same-region control flow; open the ticket with the
  5-tuples and timestamps, and design around it with retries/hedging while
  you wait.)
