# Load Balancing & Proxies — Questions

Asked at every infrastructure interview, and the source of one of the most
common real questions ("NLB vs ALB"). The tier-2 questions here are where
candidates most often give a correct-but-shallow answer — knowing what each
thing *is* is table stakes; knowing where each one breaks is the bar.

17 questions.

---

## Tier 1 — Recall

### Q1. What is the difference between a layer 4 and a layer 7 load balancer? Give the concrete example of AWS NLB vs ALB.
*Tags: lb, l4, l7* · *[asked verbatim in real interviews]*

### Q2. Name four load balancing algorithms and say when each is the right choice.
*Tags: lb, algorithms*

### Q3. What is connection draining / deregistration delay, and what goes wrong without it?
*Tags: lb, deploys*

### Q4. How does a backend behind an L7 load balancer learn the real client IP? What about behind an L4 one?
*Tags: lb, client-ip, proxy-protocol*

### Q5. What is the difference between active and passive health checking?
*Tags: lb, health-checks*

---

## Tier 2 — Explain / compare

### Q6. Why does `hash(key) % N` fail as a way to distribute keys across backends? Explain consistent hashing and what "bounded loads" adds.
*Tags: hashing, sharding, cache*

### Q7. Explain "power of two choices". Why does it beat both round robin and true least-connections in a distributed setting?
*Tags: lb, algorithms, distributed*

### Q8. Your health check is a `/health` endpoint that verifies the database connection. Argue against this design.
*Tags: health-checks, cascading-failure* · *[infra-heavy]*

### Q9. Compare TLS termination, passthrough, and re-encryption at the load balancer. When do you pick each?
*Tags: tls, lb, security*

### Q10. Compare a proxy load balancer, client-side load balancing, and a service mesh sidecar. What does eBPF-based datapath (e.g. Cilium replacing kube-proxy) change?
*Tags: lb, mesh, ebpf, kubernetes* · *[infra-heavy]*

### Q11. What are sticky sessions, how are they implemented, and what do they cost you?
*Tags: lb, state*

### Q12. Explain how a large-scale L4 load balancer (Maglev/Katran style) works with ECMP and BGP. Why is consistent hashing essential there?
*Tags: lb, ecmp, bgp, dsr* · *[infra-heavy]*

### Q13. Compare token bucket, leaky bucket, and sliding window rate limiting. How do you implement any of them across a fleet of 200 edge nodes?
*Tags: rate-limiting, distributed*

### Q14. Explain circuit breaking, load shedding, and backpressure. How do they differ and where does each belong?
*Tags: resilience, overload*

---

## Tier 3 — Scenario / debug

### Q15. Your service starts returning sporadic 502s from the load balancer. The backend logs show no errors at all — the requests never arrive. The rate is low but constant, and it's worse on connections that have been idle. What is happening?
*Tags: debugging, lb, timeouts* · *[classic]*

### Q16. You add 20% more backends during a traffic spike and latency gets *worse* for several minutes before improving. Explain the mechanisms that could cause this and how you'd prevent it.
*Tags: lb, autoscaling, cold-start*

### Q17. Design the request path for a global API: a user in Singapore, an origin in us-east-1, and a requirement that a regional failure be invisible within 30 seconds. Walk the layers and say where the failover decision is made.
*Tags: global-lb, anycast, failover, design*
