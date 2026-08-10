# DNS, TLS & HTTP — Answers

---

## Tier 1 — Recall

### A1. Cold DNS resolution of `api.example.com`

**Answer.**
The application calls `getaddrinfo()`, which consults `nsswitch.conf` —
`/etc/hosts` first on most systems, then DNS via `/etc/resolv.conf`. That
sends a **recursive** query to the configured resolver (your ISP's, a public
resolver, or in a container the cluster resolver).

The resolver then does **iterative** queries on your behalf:

1. Ask a **root** server for `api.example.com`. Roots don't know it, but
   they return a referral to the `.com` TLD nameservers.
2. Ask a `.com` nameserver. Referral to `example.com`'s authoritative
   nameservers, with glue records (their A/AAAA) so you don't need a
   second lookup to find them.
3. Ask an authoritative server for `example.com`. It returns the A/AAAA
   record — or a CNAME, in which case resolution restarts for the target.

The resolver caches each step for its TTL and returns the answer to you.
The distinction to state explicitly: **your stub resolver asks one recursive
question; the recursive resolver asks many iterative ones.** Root and TLD
answers are cached for a long time, so in practice step 1 and 2 almost never
happen.

The operational consequence: you control your TTL, but you do not control
whether anyone honours it. Enterprise resolvers, mobile carriers, JVMs with
`networkaddress.cache.ttl=-1` (cache forever, historically the default under
a security manager), and application-level caches all ignore or extend TTLs.

**Weak answers miss.** The recursive/iterative distinction — the single most
common thing candidates get backwards — and the fact that TTL is advisory in
practice.

**Follow-ups to expect.**
- What are glue records and when are they required? (When the nameserver for
  a zone lives inside that zone — otherwise you'd need to resolve
  `ns1.example.com` by asking `example.com`'s nameservers.)
- What does DNSSEC add, and why is adoption low? (Chain of signatures from
  the root; adds operational risk — an expired signature is an outage — plus
  packet size and reflection-amplification concerns. It authenticates, it
  does not encrypt.)
- DoH/DoT: what problem do they solve and what do they break? (Confidentiality
  from the network; break split-horizon resolution and enterprise/K8s
  resolvers when a browser bypasses the system resolver.)

---

### A2. CNAME at the apex

**Answer.**
A CNAME means "this name is an alias for another name, and it has no other
records." RFC 1034/2181 forbid a CNAME coexisting with any other record type
at the same name. The apex of a zone (`example.com` itself) is *required* to
have `SOA` and `NS` records — that's what makes it a zone. So a CNAME at the
apex is a contradiction, and a resolver hitting one has undefined behaviour.

Providers work around it with a synthetic record type — Route 53 `ALIAS`,
various providers' `ANAME`/`CNAME flattening`. These are not on the wire:
the authoritative server resolves the target itself and returns the resulting
A/AAAA records as if they were configured directly at the apex. Cost: it's
provider-specific and non-portable, and it ties your apex resolution to that
provider's ability to resolve the target.

The other workaround is redirecting the apex to `www` at the HTTP layer,
which requires something listening on the apex IP anyway.

**Weak answers miss.** That ALIAS is server-side synthesis, not a protocol
feature — which matters because it means the TTL behaviour and the failure
mode belong to your DNS provider.

**Follow-ups to expect.**
- What TTL does an ALIAS answer carry?
- Why is this a bigger problem now than in 2005? (Everything terminates at
  a CNAME-addressed CDN or cloud LB rather than a static IP.)

---

### A3. Negative caching

**Answer.**
When a resolver gets `NXDOMAIN` (or `NODATA` — name exists, no record of
that type), it caches the negative answer so it doesn't re-ask. The TTL for
that is **not** the record's TTL, since there is no record. It comes from the
**`SOA` record's `MINIMUM` field**, returned in the authority section of the
negative response — capped by the SOA record's own TTL (RFC 2308).

Why it matters operationally: if your SOA minimum is 1 hour and you create a
new hostname *after* something queried for it, that hostname is unreachable
for up to an hour from every resolver that asked. Classic incident during
automated provisioning — health checks probe the name before the record
exists, the negative answer gets cached, and the new service is "down" long
after the record is live.

The fix is process, not config: create DNS records before anything probes
them, and keep the SOA minimum low (order of minutes) if your workflow
creates names dynamically.

**Weak answers miss.** That it's the SOA MINIMUM, not the zone default TTL.

**Follow-ups to expect.**
- Difference between NXDOMAIN and NODATA, and why an app might treat them
  the same when it shouldn't?

---

### A4. Certificate chain validation

**Answer.**
Given a leaf certificate and the intermediates the server presented:

1. **Build a path** from the leaf up to a certificate in the local trust
   store. Each step: the child's `Issuer` must match the parent's `Subject`,
   and the child's signature must verify against the parent's public key.
   Multiple valid paths can exist — this is where cross-signed roots and
   path-building bugs bite.
2. **Verify each signature** in the chain cryptographically.
3. **Check validity windows** — `notBefore`/`notAfter` on every certificate
   in the chain, against the local clock. A wrong client clock is a common
   cause of "certificate invalid" on embedded devices.
4. **Check constraints**: `basicConstraints` CA:TRUE on every non-leaf,
   `pathLenConstraint`, `keyUsage` (must permit certificate signing on CAs),
   `extendedKeyUsage` (serverAuth on the leaf).
5. **Check the name**: the requested hostname must match a `subjectAltName`
   entry. The CN field has been deprecated for this for years; modern
   clients ignore it entirely.
6. **Check revocation** — in practice weakly or not at all (A9).
7. Optionally, **Certificate Transparency**: Chrome and others require SCTs
   proving the certificate was logged publicly.

The trust anchor is the root in your local store, which was never sent by
the server. That's the point to make: the server sends leaf + intermediates,
never the root, and a server that omits an intermediate produces the classic
"works in browsers, fails in curl/Java" bug — browsers often fetch the
missing intermediate via AIA or have it cached from another site, while
stricter clients don't.

**Weak answers miss.** The missing-intermediate failure mode and why it's
client-dependent. That is *the* real-world TLS bug this question is testing.

**Follow-ups to expect.**
- How do you check a served chain? (`openssl s_client -connect host:443
  -servername host -showcerts`.)
- What is a cross-signed root and why does it exist? (Compatibility with old
  trust stores during a root transition — the Let's Encrypt DST Root
  expiry in 2021 broke a lot of old Android/OpenSSL 1.0.2 clients.)

---

### A5. SNI

**Answer.**
Server Name Indication is a TLS extension in the `ClientHello` carrying the
hostname the client wants. It exists because of a chicken-and-egg problem:
the server must present a certificate before the client sends the HTTP
`Host` header (that's inside the encrypted stream), so without SNI a single
IP could serve only one certificate. SNI is what makes virtual hosting over
HTTPS and shared CDN/load-balancer IPs possible.

What it leaks: SNI is sent in **plaintext** in the ClientHello. Anyone on
path sees exactly which hostname you're connecting to, even though everything
else is encrypted. That's how national-scale filtering and corporate DPI do
hostname-based blocking, and it's the largest remaining plaintext metadata
leak in a TLS connection.

The mitigation is **Encrypted Client Hello** (ECH, the successor to
ESNI): the client encrypts the inner ClientHello to a public key published
in DNS (an HTTPS RR) and sends an outer ClientHello with a generic
"public name." Deployment is partial and it depends on DNS delivery of the
key, which itself needs DoH to be private — verify current deployment state
rather than asserting it.

**Weak answers miss.** The chicken-and-egg reason it exists. And note the
older-client hazard: a client that doesn't send SNI gets whatever default
certificate the server picks, which is a real source of "wrong certificate"
errors from legacy clients.

**Follow-ups to expect.**
- What breaks if a client doesn't send SNI?
- How does an L4 load balancer route by hostname without terminating TLS?
  (Peek at the plaintext SNI in the ClientHello — SNI-based routing/TLS
  passthrough. Requires SNI to be plaintext, which is exactly what ECH
  removes.)

---

### A6. Safe, idempotent, and why it matters

**Answer.**
**Safe** = no intended side effects: `GET`, `HEAD`, `OPTIONS`, `TRACE`.
**Idempotent** = executing N times has the same effect as executing once:
all the safe methods plus `PUT` and `DELETE`. `POST` and `PATCH` are
neither by default.

Why it matters: **retries**. Every layer between your client and your server
— the client library, the load balancer, the service mesh, the CDN — may
retry on failure. Retrying an idempotent request is free. Retrying a `POST`
means you may have created two orders, because the failure mode you cannot
distinguish is "request never arrived" from "request succeeded and the
response was lost."

That ambiguity is unfixable at the HTTP layer, so the answer is to make the
operation idempotent at the application layer with an **idempotency key**:
the client generates a unique key per logical operation, sends it as a
header, and the server records key → result. A repeat with the same key
returns the stored result instead of re-executing. This is what Stripe and
every serious payments API do, and it's the correct answer to "how do you
make POST retryable."

Note `DELETE` is idempotent in *effect* but not in *response*: the second
call returns 404. That's fine — idempotency is about state, not status codes
— but it trips up clients that treat 404 as failure.

**Weak answers miss.** That idempotency is a property you can *construct*
rather than one you're stuck with, and the "lost response" ambiguity that
makes it necessary.

**Follow-ups to expect.**
- Where do you store idempotency keys and for how long? (Durable store keyed
  by (key, endpoint, request-hash-to-detect-reuse); TTL longer than your
  maximum retry window; needs to be transactional with the effect, which
  points at the outbox pattern — see the distributed systems topic.)
- Which status codes are safe to retry automatically? (429 and 503 with
  `Retry-After`, and connection-level failures before the request was sent.
  500 is ambiguous — the request may have partially applied.)

---

## Tier 2 — Explain / compare

### A7. TLS 1.2 vs 1.3 handshake

**Answer.**
**TLS 1.2**: 2 round trips before application data. ClientHello →
ServerHello + Certificate + ServerKeyExchange + ServerHelloDone →
ClientKeyExchange + ChangeCipherSpec + Finished → ChangeCipherSpec +
Finished. Plus a TCP handshake underneath, so a fresh HTTPS connection costs
3 RTTs before the first byte of the request.

**TLS 1.3**: 1 round trip. The client guesses the group and sends its key
share in the ClientHello; the server responds with its share, and everything
after the ServerHello — certificate included — is already encrypted.
Total with TCP: 2 RTTs. (If the client guesses the wrong group, the server
sends HelloRetryRequest and you're back to 2 RTTs for TLS.)

What 1.3 removed, and why it matters more than the RTT:
- **Static RSA key exchange**, so all key exchange is (EC)DHE and
  **forward secrecy is mandatory** — a compromised server private key no
  longer decrypts recorded past sessions.
- **Renegotiation**, **compression** (CRIME), **CBC-mode ciphers** and
  the MAC-then-encrypt construction (BEAST, Lucky13), **RC4**, **custom DH
  groups** (Logjam), and export ciphers (FREAK). The cipher suite list went
  from hundreds to five AEAD suites, so misconfiguration is nearly
  impossible.
- Session resumption moved from session IDs/tickets to PSKs.

The honest framing: 1.3's security win is that it removed the ability to
configure it wrongly. That's a bigger deal than the round trip.

**Weak answers miss.** That the certificate is encrypted in 1.3 (a privacy
change — you can no longer passively identify the server's cert), and that
forward secrecy became mandatory.

**Follow-ups to expect.**
- At what RTT does saving one round trip matter? (Everywhere, but do the
  math: at 150 ms mobile RTT, one saved RTT is 150 ms off every cold
  connection — comparable to the entire server-side budget.)
- How much does TLS cost in CPU? (Handshake asymmetric crypto dominates;
  bulk symmetric encryption with AES-NI is close to free. So the fix for
  TLS CPU is session resumption and connection reuse, not weaker ciphers.)

---

### A8. 0-RTT and its risk

**Answer.**
On a resumed connection, the client already has a pre-shared key from the
previous session. TLS 1.3 lets it send application data **in the very first
flight**, alongside the ClientHello, encrypted under a key derived from the
PSK. Zero round trips of handshake before the request goes out.

The risk is **replay**, and it's structural, not an implementation bug. The
early data is encrypted under a key derived entirely from state the client
already had; the server contributes no freshness to it. So an on-path
attacker can capture the early-data flight and resend it — to the same
server or to a different server in the same cluster — and it will decrypt
and be accepted. There is no nonce from the server to bind it to a single
exchange.

Defences, all partial:
- Restrict 0-RTT to **idempotent requests only** (`GET` without
  side effects). This is what CDNs do, and it's the practical answer.
- **Single-use tickets / strike registers**: the server records which PSK
  tickets have been used. Works within one server; hard across a fleet
  without shared state, which is exactly the environment where you'd want
  0-RTT.
- Bound ticket lifetime and `max_early_data_size`.

Also worth stating: 0-RTT early data does **not** have forward secrecy at
the time it's sent — it's protected by the resumption key, not a fresh
ephemeral exchange.

My judgment: enable it at the CDN edge for static/idempotent traffic, never
for an API that mutates state, and never rely on the application layer to
sort it out afterwards.

**Weak answers miss.** That replay is inherent to the design rather than an
implementation weakness, and the forward-secrecy gap.

**Follow-ups to expect.**
- QUIC has the same property — how does HTTP/3 handle it? (Same rule:
  idempotent-only early data.)
- Would an idempotency key make 0-RTT POSTs safe? (It makes the *effect*
  safe against duplication, but the request is still replayable by an
  attacker who never saw the plaintext — a different threat. Weigh both.)

---

### A9. Revocation

**Answer.**
Three mechanisms:

**CRL** — the CA publishes a signed list of revoked serial numbers, clients
download it. Fails on size (large CAs have huge lists) and freshness
(published periodically). Effectively unused by browsers for public TLS.

**OCSP** — the client asks the CA's responder "is this serial revoked?" in
real time. Problems: it's a latency hit and an availability dependency on
the CA for every connection, and it leaks the client's browsing to the CA.
Worst of all, it's **soft-fail**: if the responder is unreachable, clients
proceed anyway, because hard-fail would mean a CA outage takes down the web.
An attacker who can present a revoked certificate can also block OCSP, so
soft-fail revocation provides essentially no security against a network
attacker. That's the sentence that answers "why is revocation broken."

**OCSP stapling** — the *server* fetches a signed, time-stamped OCSP
response and staples it into the handshake. Fixes latency and privacy, but
still soft-fail unless the certificate carries the `must-staple` extension,
which is rarely used because a stapling failure then becomes an outage.

What the industry actually does instead:
- **Short-lived certificates.** If a certificate lives 90 days — and the
  ecosystem is moving toward much shorter, with the CA/Browser Forum having
  agreed to a stepped reduction toward ~47 days by 2029, so verify the
  current schedule — then expiry substitutes for revocation. This is the
  dominant direction and it's why ACME automation is now mandatory in
  practice.
- **Browser-pushed aggregated revocation sets** (Chrome's CRLSets, Firefox's
  CRLite) — the browser vendor curates and pushes a compressed set,
  sidestepping the availability problem.

**Weak answers miss.** Soft-fail. Without it, the answer sounds like
revocation works.

**Follow-ups to expect.**
- What is your revocation story for an internal PKI? (You control both ends,
  so hard-fail OCSP or very short-lived certs — minutes/hours — issued by
  something like Vault or SPIFFE/SPIRE, are both viable. Short-lived is
  simpler and is what service meshes do.)
- Certificate expiry is one of the most common outage causes — how do you
  prevent it? (Automated issuance/renewal, plus alerting on *days remaining*
  from an external prober that checks what's actually served, not what's in
  your config. Those differ more often than you'd like.)

---

### A10. mTLS

**Answer.**
Plain TLS authenticates the *server* to the client and encrypts the channel.
**mTLS** adds a client certificate, so the server cryptographically
authenticates the client too — identity bound to a key, verified at the
transport layer, before any application code runs.

What that buys: a strong workload identity that doesn't depend on network
position. "This request came from the payments service" becomes a
cryptographic fact rather than an inference from a source IP that anyone
inside the VPC could spoof or reuse. That's the foundation of zero-trust
service-to-service authorization, and it's why service meshes (Istio,
Linkerd) and SPIFFE/SPIRE exist.

Operational costs, which is what the question is really asking:

1. **Certificate lifecycle for every workload.** Issuance at pod start,
   rotation without dropping connections, and revocation. At fleet scale
   this is a distributed system in its own right, and it becomes a hard
   dependency on your critical path — the CA being down means new workloads
   can't start.
2. **Clock sensitivity.** Short-lived certificates plus skewed clocks equals
   sporadic, hard-to-debug auth failures.
3. **Debuggability collapses.** You can no longer curl a service or read a
   packet capture without provisioning an identity and doing key material
   handling. Every break-glass procedure gets harder.
4. **CPU and connection cost.** Full handshakes are expensive; if your
   traffic pattern is many short connections you'll feel it. Mitigate with
   long-lived pooled connections, which then complicates rotation (a
   connection outlives the certificate that authenticated it — decide
   whether you care).
5. **Trust store distribution and CA rotation.** Rotating a root without an
   overlap window is a fleet-wide outage; you need dual-trust periods.

And the thing mTLS does **not** give you: authorization. It tells you *who*,
not *what they may do*. You still need policy on top, and conflating the two
is a common design error.

**Weak answers miss.** That the CA becomes a critical-path dependency, and
that mTLS is authentication only. Candidates who've only read about it list
benefits; candidates who've run it lead with rotation.

**Follow-ups to expect.**
- How do you rotate the root CA with zero downtime? (Distribute the new root
  to all trust stores first, run a dual-trust period longer than your maximum
  certificate lifetime, then switch issuance, then remove the old root.)
- How does SPIFFE identity differ from an IP or a Kubernetes service
  account? (A workload identity document with a URI SAN, attested at issue
  time by node + workload attestation — portable across platforms.)
- Where would you terminate mTLS: app, sidecar, or eBPF? (Sidecar is the
  common answer; note the per-pod resource tax and latency, and that
  ambient/eBPF-based approaches trade some isolation for it.)

---

### A11. GeoDNS vs anycast

**Answer.**
**GeoDNS**: the authoritative nameserver returns different A records based
on where the *query* came from. Steering happens at resolution time.
- Pro: you can return arbitrary answers per region and change weights
  instantly in your control plane; works with any protocol; can implement
  complex policy (weighted, latency-based, failover).
- Con: you see the **resolver's** location, not the client's — a user on a
  public resolver in another country gets steered to the resolver's region.
  (EDNS Client Subnet partially fixes this by passing a truncated client
  prefix, at a privacy cost, and not every resolver sends it.) And it
  inherits every DNS caching problem: TTLs you don't control, so steering
  changes propagate slowly and unevenly (A12).

**Anycast**: the same IP prefix is advertised via BGP from many locations;
the internet's routing fabric delivers each packet to the topologically
nearest announcement. Steering happens per-packet.
- Pro: no client-side state, no DNS TTL dependency, and failover is
  *withdrawing a BGP announcement* — fast and global. Excellent DDoS
  property: an attack is absorbed by whichever PoPs it reaches rather than
  concentrating.
- Con: BGP optimises for **AS-path length, not latency**, so "nearest" can
  be wrong. You have no per-user control. And the big one: **route changes
  can move a flow mid-connection**, breaking any TCP connection whose state
  lives at one PoP. That's fine for short DNS/UDP exchanges (why every
  major public resolver is anycast), manageable for HTTP with retries, and
  genuinely hard for long-lived stateful connections — which is why anycast
  deployments either keep connections short or push state to a shared tier.

In practice large services use both: anycast for the edge/PoP entry, DNS
steering for coarse policy and for products where per-tenant routing is
needed. Neither alone gives you both fast failover and precise control.

**Weak answers miss.** The resolver-vs-client location problem in GeoDNS,
and the mid-connection re-route hazard in anycast. Those are the two things
that decide real deployments.

**Follow-ups to expect.**
- How do you do maintenance on an anycast PoP? (Withdraw the announcement,
  wait for connections to drain, then work. "Drain by BGP" — and note that
  in-flight long connections still get cut, so you need graceful shutdown.)
- What's the third option? (Client-side steering: the app measures and
  picks, e.g. a mobile client probing several endpoints. Most control,
  requires you to own the client, and needs a bootstrap that isn't itself
  a single point of failure.)

---

### A12. DNS as failover

**Answer.**
It's poor because **you do not control the cache**. Your TTL is a request,
not a guarantee. Between you and the user sit: the recursive resolver (may
clamp minimum TTLs, may serve stale on failure), the OS stub cache, the
browser cache (Chrome has its own, historically ~60 s regardless of your
TTL), and application-level caches — the JVM's DNS cache is notorious,
having historically cached forever under a security manager, and plenty of
services resolve once at startup and never again.

So a 60-second TTL does not mean 60-second failover. It means "most traffic
moves in a few minutes, some fraction moves in an hour, and a long tail
never moves until it's restarted." During a real incident that long tail is
the traffic still hammering the dead endpoint.

Second problem: DNS gives you no feedback. You don't know who moved.

If you must use it:
- **Pre-lower the TTL.** A TTL change itself propagates at the *old* TTL, so
  dropping from 3600 to 60 during an incident does nothing for an hour. Run
  low TTLs (30–60 s) permanently on anything you intend to fail over, and
  accept the extra query load.
- **Health-checked DNS failover** from your provider, so the record change is
  automatic rather than a human under pressure.
- **Keep the old endpoint serving** if at all possible — draining, or
  proxying to the new one — so the stragglers aren't broken. This is the most
  effective single measure and it's the one people skip.
- **Multiple A records** so clients can fail over themselves; browsers and
  many libraries will try the next address. Behaviour varies by client, so
  test it rather than assuming.
- Prefer a mechanism that isn't DNS for the fast path: anycast withdrawal,
  an LB that reroutes behind a stable IP, or client-side steering.

**Weak answers miss.** That lowering the TTL during an incident is too late,
and that keeping the old endpoint alive beats making it fail faster.

**Follow-ups to expect.**
- What's the cost of a 30 s TTL? (Query volume on your authoritative
  servers, and more resolution latency on cold paths — usually cheap
  relative to the failover benefit, but say it's a tradeoff you priced.)

---

### A13. Cache-Control semantics

**Answer.**
- **`no-store`** — do not write this to any storage, anywhere. The only one
  that actually means "don't cache." Use for genuinely sensitive responses.
- **`no-cache`** — you *may* store it, but you must revalidate with the
  origin before serving it. Not "don't cache," despite the name. This is the
  most commonly misused directive.
- **`private`** — may be cached by the end-user's browser but not by shared
  caches (CDN, proxy). Use for per-user responses that are still worth
  caching client-side.
- **`public`** — explicitly cacheable by shared caches, even in cases where
  heuristics would say otherwise (e.g. a request with an `Authorization`
  header).
- **`must-revalidate`** — once the response is stale, do not serve it; go to
  the origin. Without it, caches are permitted to serve stale under some
  conditions (e.g. origin unreachable). `must-revalidate` explicitly forbids
  that.
- **`max-age`** / **`s-maxage`** — freshness lifetime; `s-maxage` overrides
  for shared caches only.

**`ETag`** is the validator. The origin returns `ETag: "abc"`; the cache
later sends `If-None-Match: "abc"` and gets a `304 Not Modified` with no
body if unchanged. It saves bandwidth, not round trips — you still pay one
RTT. `Last-Modified`/`If-Modified-Since` is the weaker, second-granularity
equivalent. Strong vs weak ETags (`W/"abc"`) differ in whether byte-identical
representation is guaranteed, which matters for range requests.

Two directives worth knowing because they're what CDNs actually run on:
**`stale-while-revalidate`** (serve stale immediately, refresh in the
background — turns a cache miss's latency cost into zero for the user) and
**`stale-if-error`** (serve stale when the origin is failing — a genuinely
powerful availability lever that costs you nothing until an incident).

**Weak answers miss.** `no-cache` ≠ don't cache, and
`stale-while-revalidate`/`stale-if-error`, which are the directives with real
operational value.

**Follow-ups to expect.**
- `Vary` — what does it do and why is `Vary: *` or `Vary: User-Agent`
  destructive? (Cache key expands per distinct header value; on User-Agent
  that's effectively per-client, destroying the hit rate.)
- How do you invalidate? (Purge APIs, or don't — use content-hashed URLs so
  nothing ever needs invalidating. The second is the right answer for
  assets.)

---

### A14. CORS preflight

**Answer.**
The browser's same-origin policy blocks JS from reading cross-origin
responses. CORS is the server's way to opt in.

A **preflight** is triggered when the request isn't "simple." Simple means:
method is GET/HEAD/POST, and the headers are limited to a small safelist
(`Accept`, `Accept-Language`, `Content-Language`, `Content-Type`), and
`Content-Type` is one of `text/plain`,
`application/x-www-form-urlencoded`, `multipart/form-data`. Anything else —
`PUT`/`DELETE`/`PATCH`, a custom header like `Authorization` or
`X-Request-Id`, or `Content-Type: application/json` — triggers preflight.
That last one is why practically every JSON API sees preflights.

The flow: browser sends `OPTIONS` to the same URL with
`Origin`, `Access-Control-Request-Method`, and
`Access-Control-Request-Headers`. The server responds with
`Access-Control-Allow-Origin`, `-Allow-Methods`, `-Allow-Headers`, and
optionally `-Max-Age`. If the response permits the intended request, the
browser sends the real one; otherwise it fails before the real request is
ever made.

For credentialed requests (cookies, TLS client certs) you must send
`Access-Control-Allow-Credentials: true`, and then
`Access-Control-Allow-Origin` may **not** be `*` — it must echo a specific
origin. That's the rule people hit and then "fix" by reflecting the `Origin`
header unconditionally, which turns CORS off entirely and is a real
vulnerability.

Operationally: `Access-Control-Max-Age` caches the preflight (browsers cap
it — Chrome at 2 hours, Firefox at 24, so verify current values). Without
it you're paying an extra RTT on every cross-origin API call, which on a
mobile network is a visible latency regression.

**Weak answers miss.** That CORS is enforced by the *browser*, not the
server — it protects users from malicious sites, and provides no protection
for your API against a non-browser client. And the credentials/wildcard
interaction.

**Follow-ups to expect.**
- Does CORS protect your API? (No. `curl` ignores it. Use real
  authorization; CSRF tokens or SameSite cookies for the browser threat.)
- Why did `Access-Control-Allow-Origin: *` stop working after you added
  cookies?

---

## Tier 3 — Scenario / debug

### A15. TLS failures from one country, subset of clients

**Answer.**
"Certificate is valid" from your machine tells you almost nothing — validity
is a property of the (chain, client trust store, client clock, client TLS
stack) tuple, not of the certificate. So structure the investigation around
what varies.

**First, characterise.** Get the failing clients' TLS stack and version from
whatever telemetry exists — user agent, SDK version, OS version. Then check
whether the failures cluster on: old Android, old OpenSSL, a specific
corporate network, or a specific mobile carrier. That single cut usually
solves it.

**Hypotheses, in order:**

1. **Missing intermediate.** You rotated the certificate and the new chain
   omits an intermediate, or presents them in the wrong order. Browsers
   paper over this via AIA fetching or cached intermediates; strict clients
   (older Java, OpenSSL 1.0.x, embedded, some mobile SDKs) don't. Presents
   exactly as "some clients, valid cert." Check with
   `openssl s_client -showcerts` from outside your network — and check
   against *every* edge node, because a partially-rolled deploy means only
   some terminators serve the bad chain.
2. **Trust store age.** A root the new chain depends on isn't in old
   clients' stores. The Let's Encrypt DST Root CA X3 expiry in 2021 is the
   canonical example — it broke old Android and OpenSSL 1.0.2 while
   everything modern was fine. If the failing population skews old-device,
   this is it. Fix is a cross-signed chain that old stores accept.
3. **TLS version / cipher floor.** You disabled TLS 1.0/1.1 or a legacy
   cipher suite as part of the change, and clients that only speak those now
   fail. Geographic clustering follows device-age distribution, which
   correlates with country — that explains the "one country" pattern
   without anything network-specific being wrong.
4. **On-path interception.** Corporate or national middleboxes that
   re-sign traffic. If the change altered anything the middlebox parses
   (certificate size, TLS 1.3 negotiation, ECH, a new extension), the
   middlebox may fail closed. This is the hypothesis that specifically
   explains country-scoped, subset-of-clients behaviour, and it's worth
   testing early given the framing of the question.
5. **Clock skew** on client devices against a `notBefore` that's in the
   future — happens right after issuance if there's skew.
6. **Anycast/CDN PoP-specific config.** Only the PoPs serving that country
   got the bad config. Check per-PoP, not globally.

**What I'd measure:** a synthetic prober from that region with several
distinct TLS stacks (modern OpenSSL, an old OpenSSL, an old Android API
level) hitting each edge node directly by IP with SNI set. That grid —
region × stack × edge node — isolates all six hypotheses in one pass.

**Weak answers miss.** That "valid certificate" is client-relative, and
running the test from the affected region *with the affected stack*. Testing
from a laptop with a current trust store reproduces nothing.

**Follow-ups to expect.**
- How would you have caught this pre-deploy? (Chain validation in CI against
  a matrix of trust stores; canary the certificate change to one PoP;
  external monitoring that alerts on chain composition, not just expiry.)
- Roll back or roll forward? (Rolling back a certificate is usually safe and
  fast — say so, and say that you'd do it before finishing the diagnosis.
  Mitigate first, diagnose second.)

---

### A16. Retry storms

**Answer.**
**What happens.** The system is at 2% errors because something downstream is
partially degraded — say a database at capacity. Each failed request now
generates 3 more, immediately. Offered load on the degraded component jumps
by up to 3x precisely when it has least headroom. It degrades further, error
rate climbs, and more requests enter the retry path. This is positive
feedback and it converges on total failure, not on recovery.

Then it gets worse in two ways. First, **retries multiply through layers**:
if the client retries 3x, the API gateway retries 3x, and the service retries
its dependency 3x, a single user request can become 27 requests at the
bottom. Nobody designs this; it accretes. Second, **synchronisation**:
without jitter, retries from thousands of clients that failed at the same
moment arrive at the same moment, producing a periodic thundering herd that
prevents the system from ever getting a quiet interval to recover in. This
is the mechanism behind outages that don't self-heal after the original
trigger is gone — a metastable failure.

**The policy I'd ship:**

1. **Exponential backoff with full jitter.** `sleep = random(0, min(cap,
   base * 2^attempt))`. Full jitter, not "exponential plus a little
   randomness" — AWS's own analysis of this is the standard reference and
   full jitter both decorrelates and reduces total work.
2. **Low attempt count.** 2–3 total attempts. Deep retry counts buy almost
   nothing; if two attempts failed, the dependency is down, not unlucky.
3. **Retry budget / token bucket.** Cap retries at a fraction of successful
   traffic — e.g. retries may not exceed 10% of successes on that route.
   This is the single most important control, because it makes the amplify
   factor bounded *by construction* regardless of how bad the error rate
   gets. gRPC and Envoy both implement this; use it.
4. **Retry at exactly one layer.** Pick the layer with the best context —
   usually the outermost client that knows the user's deadline — and make
   every other layer fail fast. Disable mesh/LB retries if the client
   retries.
5. **Only retry what's retryable.** Idempotent methods, or non-idempotent
   ones carrying an idempotency key (A6). Connection-refused and timeouts
   before the request was sent are always safe; a 500 mid-request is not.
   Honour `Retry-After` on 429/503.
6. **Deadline propagation.** Pass the remaining budget down as a deadline;
   a service must not retry past the caller's deadline, because that work
   is guaranteed to be discarded. This is where most of the wasted capacity
   in a degraded system goes.
7. **Circuit breaker** in front of the retry logic: after a threshold of
   failures, stop sending entirely for a cooldown and let a small number of
   probe requests test recovery. This is what actually gives the downstream
   the quiet interval it needs.
8. **Server-side load shedding** as the backstop, because you cannot trust
   every client. Shed at the edge based on queue depth, prioritising by
   request class, and return 429 fast — cheap rejection is what keeps the
   remaining capacity useful.

The framing that separates a staff answer: retries are a *capacity* decision,
not an error-handling detail. You are choosing to spend the system's scarcest
resource during its worst moment, and the budget is the mechanism that makes
that choice explicit.

**Weak answers miss.** The retry budget and deadline propagation. Backoff
plus jitter is the expected answer; those two are the ones that show
operational experience. Also missed: that the system may not recover on its
own even after the trigger clears.

**Follow-ups to expect.**
- How do you recover from a metastable failure once you're in it? (Shed
  load hard enough to break the feedback loop — often literally turning
  traffic off and ramping back in. Say that gracefully draining and
  re-admitting is a capability you need *before* the incident.)
- Where do hedged requests fit, and how are they different? (Send a second
  request after p95 elapses to cut tail latency; different intent from
  retries, and they *add* load, so they need their own budget — typically
  capped at a few percent of traffic.)

---

### A17. Cutting 200 ms p50 on a mobile API

**Answer.**
Budget the path first, then attack the biggest line item. Rough
order-of-magnitude numbers for mobile (state these as estimates): mobile RTT
50–150 ms on 4G/5G, worse on congested networks; intra-region server-side
RTT sub-millisecond.

**A cold request costs, in RTTs:**
- DNS: 1 RTT to the resolver if uncached, more if the resolver has to
  recurse or if the name is a chain of CNAMEs.
- TCP handshake: 1 RTT.
- TLS 1.3: 1 RTT (TLS 1.2: 2).
- Request/response: 1 RTT plus server time.

That's 4 RTTs before you've done any work — at 100 ms RTT, **400 ms of pure
network setup**. Which immediately tells you the answer: on mobile, p50 is
usually dominated by connection setup, not by your server.

**In priority order:**

1. **Connection reuse.** If the client opens a new connection per request,
   fixing that alone removes 3 of the 4 RTTs. Keepalive with a generous idle
   timeout, HTTP/2 or HTTP/3 so one connection multiplexes everything.
   Check the mobile SDK's default — many pool badly.
2. **HTTP/3 / QUIC.** Folds transport and crypto handshake (1 RTT cold,
   0-RTT resumed), and connection migration survives the WiFi↔cellular
   switch that otherwise forces a full re-handshake. This is the single
   biggest structural win on mobile.
3. **TLS 1.3 + session resumption.** If anything is still on 1.2, that's a
   free RTT.
4. **Terminate at the edge.** A PoP near the user turns the 3 setup RTTs
   from user↔origin (say 150 ms each) into user↔PoP (say 20 ms each), with
   a warm pooled connection PoP↔origin. Saves ~390 ms of setup on a cold
   connection without touching the application. Usually the highest
   leverage per unit of work.
5. **DNS.** Reduce CNAME chains (each is potentially another resolution),
   keep TTLs sane, and consider resolving at app start and reusing. Measure
   before optimising — often already cached.
6. **Payload.** Compression (Brotli over gzip for text), response shape —
   is the client making 4 sequential calls where 1 would do? Sequential
   dependent requests each cost a full RTT and this is frequently the real
   problem hiding behind "the API is slow."
7. **Server time.** Only now. If p50 server time is 20 ms out of 200, the
   entire backend optimisation budget is capped at 20 ms and everyone
   working on it is wasting their time.

**The measurement point I'd insist on:** instrument the *client* with a
breakdown — DNS, connect, TLS, TTFB, transfer — not just total. Server-side
metrics cannot see 3 of the 4 RTTs, which is why teams so often optimise the
one component they can see. If the client SDK can't do that, add it before
doing anything else.

**Weak answers miss.** Doing the RTT arithmetic, and the observation that
server-side metrics are structurally blind to most of the mobile latency
budget. Also missed: sequential client calls, which no server-side dashboard
will ever reveal.

**Follow-ups to expect.**
- p50 vs p99 — would your priorities change? (Yes. p99 on mobile is
  dominated by radio state transitions, packet loss and retransmission
  timeouts. Different fix set: loss tolerance, HTTP/3, smaller payloads,
  hedging.)
- What would you *not* do? (Micro-optimise server handlers; add a cache
  that doesn't address setup cost. Say what you're explicitly not solving
  and why — that's the staff signal.)
