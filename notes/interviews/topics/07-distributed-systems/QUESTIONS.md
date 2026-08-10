# Distributed Systems — Questions

The theory that infrastructure interviews test through practical scenarios.
Nobody will ask you to prove FLP; they will ask what happens when the
coordinator partitions away, and expect you to reason from the same
principles.

18 questions.

---

## Tier 1 — Recall

### Q1. State CAP precisely. Then state PACELC and explain why it's the more useful framing.
*Tags: cap, theory*

### Q2. Define linearizability, sequential consistency, causal consistency, and eventual consistency. Order them by strength.
*Tags: consistency-models*

### Q3. What are read-your-writes, monotonic reads, and monotonic writes? Give a user-visible failure for each.
*Tags: session-guarantees*

### Q4. In a quorum system with N replicas, R readers and W writers, what does R + W > N give you — and what does it not give you?
*Tags: quorum, replication*

### Q5. Explain Raft leader election. Why does it need a majority, and what happens when exactly half the nodes fail?
*Tags: consensus, raft*

---

## Tier 2 — Explain / compare

### Q6. Why is a distributed lock without a fencing token unsafe? Walk through the failure.
*Tags: locks, leases, safety* · *[classic]*

### Q7. Compare NTP, PTP, hybrid logical clocks, and TrueTime. What can you safely do with each?
*Tags: clocks, time*

### Q8. Explain why "exactly-once delivery" is impossible and what people actually mean when they claim it.
*Tags: messaging, semantics*

### Q9. Explain two-phase commit and its blocking failure mode. What is a saga and what does it give up?
*Tags: transactions, 2pc, saga*

### Q10. In Kafka: explain ISR, `acks`, `min.insync.replicas`, and unclean leader election. Construct the exact configuration that loses acknowledged data.
*Tags: kafka, durability* · *[on your resume — expect this]*

### Q11. Explain Little's law and the utilisation-latency curve. Why does a queue at 90% utilisation behave so differently from one at 70%?
*Tags: queueing, capacity*

### Q12. Compare heartbeat timeouts, phi-accrual failure detection, and lease-based liveness. Why is every timeout wrong?
*Tags: failure-detection*

### Q13. What is a gray failure, and why is a node that is "up but slow" worse than one that is down?
*Tags: failure-modes, resilience* · *[infra-heavy]*

### Q14. Explain cell-based architecture and shuffle sharding. What blast-radius property does each give you?
*Tags: isolation, blast-radius, multi-tenancy*

### Q15. For a multi-region active-active system, compare last-writer-wins, CRDTs, and per-region ownership as conflict strategies.
*Tags: multi-region, conflicts*

---

## Tier 3 — Scenario / debug

### Q16. Your system recovered from the original trigger of an outage, but it will not return to healthy on its own. Explain what's happening and how you get out of it.
*Tags: metastable-failure, overload* · *[infra-heavy]*

### Q17. A network partition splits your 5-node cluster 3–2. Walk through what each side does, what clients see, and what happens on heal — for a Raft-based system and for a leaderless quorum system.
*Tags: partition, consensus, quorum*

### Q18. Design the failure handling for a job scheduler that must run each job at least once, at most once per scheduled time, across a fleet where any worker can die at any moment.
*Tags: design, idempotency, leases*
