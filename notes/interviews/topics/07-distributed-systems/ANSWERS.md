# Distributed Systems — Answers

---

## Tier 1 — Recall

### A1. CAP, precisely, and PACELC

**Answer.**
CAP, stated properly: **in the presence of a network partition, a
distributed system must choose between consistency (linearizability) and
availability (every non-failing node answers every request).** That's it.
The common phrasing "pick two of three" is wrong — partitions are not
something you choose, they are something that happens to you. You only ever
choose between C and A, and only during a partition.

Two more precision points:
- The "C" in CAP is **linearizability specifically**, not the C in ACID and
  not "consistency" in any looser sense. A system can be ACID-compliant and
  CAP-available.
- The "A" is *every non-failing node responds* — a very strong definition.
  A system that returns errors from a minority partition is CP, even if it's
  99.99% available in the ordinary sense.

**PACELC** (Abadi) extends it: **if there is a Partition, choose between
Availability and Consistency; Else (normal operation), choose between
Latency and Consistency.**

Why it's more useful: **partitions are rare; the else-branch is where you
live.** Every day, in the absence of any failure, you are trading latency
against consistency — every synchronous cross-region replication decision,
every quorum read, every "should this read hit a replica" question. CAP says
nothing about the 99.9% of time when nothing is broken; PACELC names the
tradeoff that actually shapes your architecture.

Classifications: Spanner is **PC/EC** — consistent under partition and in
normal operation, paying latency (commit wait) for it. Cassandra with
`LOCAL_QUORUM` is **PA/EL** — available and low-latency, consistency is
your problem. DynamoDB is PA/EL by default, with a PC/EC option per read.
MongoDB with majority write concern is PC/EC-ish.

**Weak answers miss.** That C means linearizability, and the "else" branch —
which is the whole reason to prefer PACELC.

**Follow-ups to expect.**
- Is CAP still useful? (As a sanity check on claims, yes: any vendor
  claiming consistency *and* availability under partition is either
  redefining a term or wrong. As a design tool, PACELC is better.)

---

### A2. Consistency models, ordered

**Answer.**
Strongest to weakest:

1. **Linearizability** — every operation appears to take effect at a single
   instant between its invocation and response, and that order is consistent
   with real time. If write W completes before read R begins (in wall-clock
   terms), R sees W. It's a *single-object* guarantee — linearizability plus
   serializability across multiple objects is what Spanner calls **external
   consistency**. Cost: requires coordination on every operation; a read must
   confirm it isn't stale, so you pay a round trip or a lease.
2. **Sequential consistency** — all nodes see operations in the *same* total
   order, and each process's operations appear in its program order — but
   that order need not match real time. So a read can return a stale value
   as long as everyone is stale in the same order. Rarely implemented
   directly; useful conceptually because it isolates "agreement on order"
   from "agreement with wall clock."
3. **Causal consistency** — operations that are causally related (one could
   have influenced the other, per happens-before) are seen in the same order
   by everyone; concurrent operations may be seen in different orders. This
   is the strongest model achievable while remaining **available under
   partition**, which is a genuinely important result and worth stating.
   Implemented with version vectors or dependency tracking.
4. **Session guarantees** (read-your-writes, monotonic reads/writes, writes-
   follow-reads) — not a total order at all, but per-client properties that
   make eventual consistency tolerable to a human. See A3.
5. **Eventual consistency** — if writes stop, replicas converge. Says
   nothing about when, and nothing about what you observe meanwhile. It is a
   liveness property with no safety content, which is why it's much weaker
   than it sounds.

The practical point: most systems that say "strongly consistent" mean
linearizable *reads within one region*, and most systems that say "eventually
consistent" would be far more usable if they offered session guarantees.
Knowing where a given store sits is more valuable than the taxonomy itself.

**Weak answers miss.** That causal is the strongest available-under-partition
model, and that eventual consistency provides no safety guarantee at all.

**Follow-ups to expect.**
- Is serializability comparable to linearizability? (Different axes:
  serializability is about *transactions* over multiple objects; it permits
  any order, including one that ignores real time. Linearizability is about
  single objects with real-time ordering. Strict serializability is both.)

---

### A3. Session guarantees

**Answer.**
- **Read-your-writes (read-after-write)**: once you've written, your
  subsequent reads see that write or something newer.
  *Failure*: user updates their profile photo, the page reloads from a
  replica, and the old photo is back. They update it again. Now you have a
  support ticket and possibly duplicate data.
- **Monotonic reads**: successive reads by the same client never go
  backwards in time.
  *Failure*: user refreshes and sees 5 notifications, refreshes again and
  sees 3, because the two reads hit replicas with different lag. Reads as
  data loss to the user — worse than staleness, because it destroys trust.
- **Monotonic writes**: a client's writes are applied in the order the
  client issued them.
  *Failure*: user renames a file then deletes it; the delete is applied to
  the replica before the rename arrives, so the rename recreates it. Or a
  "set status to X then Y" sequence lands as Y then X and the final state is
  wrong.
- (Fourth, often listed: **writes-follow-reads** — if you read a value and
  then write, your write is ordered after the value you read. *Failure*: you
  reply to a comment and the reply appears before the comment on another
  replica.)

The reason to know these by name: they're the vocabulary for saying *what
you actually need* instead of demanding "strong consistency," which is
expensive and usually more than the product requires. Most user-facing
staleness complaints are one of these four, and each has a cheap fix:
- Read-your-writes: route post-write reads to the primary for a window, or
  pass the write's LSN/version and have the replica wait for it.
- Monotonic reads: pin the session to one replica, or track the highest
  version seen per client and never serve older.
- Monotonic writes / writes-follow-reads: version vectors, or route a
  session's writes through one path.

**Weak answers miss.** Giving concrete user-visible failures. The models are
only useful as a diagnostic vocabulary, and reciting definitions without the
symptom means you can't apply them.

**Follow-ups to expect.**
- Where do you implement these — client, proxy, or store? (Cleanest at the
  routing/proxy layer with a version token, because it works for every client
  and doesn't need application changes. Some drivers do it natively.)

---

### A4. R + W > N

**Answer.**
With N replicas, writing to W and reading from R, the condition R + W > N
guarantees the read set and the write set **overlap in at least one
replica**. So a read is guaranteed to touch at least one replica that has
the latest *completed* write.

That's a real and useful property. What it is **not**:

1. **It is not linearizability.** Overlap tells you a replica in your read
   set *has* the newest completed write; it doesn't tell you how to identify
   it among conflicting values, and without a total order you may pick
   wrong. Dynamo-style systems resolve by timestamp (last-writer-wins),
   which is not a real order — clock skew means the "newer" value can be the
   one written first. Real linearizability needs consensus or read-repair
   with a coordination step, not just overlap.
2. **It says nothing about concurrent or failed writes.** A write that
   reaches some replicas and then fails is neither committed nor rolled
   back — there's no rollback in a leaderless system. A subsequent read may
   or may not see it, and a later read may see it *again* after read repair
   propagates it. So a failed write can become visible later, which
   surprises people badly.
3. **Sloppy quorums break it entirely.** If the system accepts writes on
   substitute nodes when the preferred replicas are unavailable (hinted
   handoff), the "W replicas" that acknowledged may not be members of the
   read set at all, and the overlap guarantee evaporates. Dynamo and
   Cassandra both do this by default in some configurations — it's an
   availability feature that quietly weakens the consistency claim.
4. It doesn't survive **membership changes** without care — N changing
   during a write/read pair breaks the arithmetic.

Common configurations: `R=1, W=N` (fast reads, slow and fragile writes),
`R=N, W=1` (the reverse), `R=W=⌈(N+1)/2⌉` (balanced; N=3 → R=W=2, the usual
default). And `LOCAL_QUORUM` in a multi-DC Cassandra deployment gives quorum
*within* the local datacentre — low latency, and explicitly no cross-DC
guarantee, which is a tradeoff people make without realising it.

**Weak answers miss.** That R+W>N ≠ strong consistency. This is one of the
most common misconceptions in the field and the question exists to test it.

**Follow-ups to expect.**
- How do you get linearizable operations in such a system? (Consensus per
  operation — Cassandra's lightweight transactions use Paxos. Correct but
  several times more expensive, and mixing LWT and normal writes on the same
  partition voids the guarantee.)
- What's read repair and anti-entropy? (Repair on read when replicas
  disagree; background Merkle-tree comparison to converge. Both are
  convergence mechanisms, not consistency guarantees.)

---

### A5. Raft leader election

**Answer.**
Time is divided into **terms**, each with at most one leader. Every node is
follower, candidate, or leader.

A follower that receives no heartbeat (`AppendEntries`) within its
randomised election timeout increments its term, becomes a **candidate**,
votes for itself, and sends `RequestVote` to all peers. A node grants its
vote if (a) it hasn't already voted in this term and (b) the candidate's log
is **at least as up to date** as its own — compared by last log term, then
last log index. A candidate receiving votes from a **majority** becomes
leader and immediately sends heartbeats to establish authority.

The randomised timeout (typically 150–300 ms range, randomised per node) is
what prevents perpetual split votes: nodes wake at different times, so one
usually gets there first.

**Why a majority:** two majorities of the same set must intersect. So at most
one leader can be elected per term, and the intersection property also
guarantees the new leader's log contains every committed entry — because an
entry is committed only when replicated to a majority, and the new leader
was voted for by a majority, and those two majorities share at least one
node that has the entry. The up-to-date-log vote condition is what turns
that intersection into a safety guarantee. **Majority is not a heuristic; it
is the mechanism.**

**Exactly half fail** (e.g. 2 of 4, or the 3–2 partition case with 5 nodes,
seen from the minority side): the remaining nodes cannot form a majority, so
**no leader can be elected and no writes can be committed**. The cluster is
unavailable for writes until a node returns. It remains *safe* — it will not
serve divergent writes — which is the intended trade. This is why cluster
sizes are odd: 4 nodes tolerate the same single failure as 3 while costing
more and having a worse split-vote profile.

Fault tolerance: `⌊(N−1)/2⌋`. N=3 tolerates 1, N=5 tolerates 2, N=7
tolerates 3. Larger clusters tolerate more failures but make every commit
slower (more replicas to hear from) — which is why 5 is the usual sweet spot
for etcd/consensus stores and why you don't run 11-node Raft groups.

**Weak answers miss.** The intersection argument, and the log-up-to-date
vote condition — without it, majority alone doesn't guarantee the leader has
all committed entries.

**Follow-ups to expect.**
- Can a Raft leader serve reads locally without contacting anyone? (Not
  safely by default — it might have been deposed and not know. Options:
  route reads through the log (correct, slow), **ReadIndex** (confirm
  leadership with a heartbeat round before answering), or **leader leases**
  (assume leadership for a bounded time — fast, but depends on bounded clock
  drift, so it trades a timing assumption for latency).
- How does Raft differ from Paxos? (Same guarantees; Raft constrains the
  design — strong leader, logs that must match — for understandability and
  implementability. Multi-Paxos is more flexible and much harder to get
  right.)
- What is a Raft membership change and why is it delicate? (Joint consensus
  or single-node-at-a-time; changing membership by more than one node at
  once can create two disjoint majorities and elect two leaders.)

---

## Tier 2 — Explain / compare

### A6. Distributed locks and fencing tokens

**Answer.**
The failure, step by step:

1. Client A acquires the lock (Redis `SET key NX PX 30000`, or a
   ZooKeeper/etcd lease).
2. Client A begins its critical section — say, writing to shared storage.
3. Client A **stalls**: a stop-the-world GC pause, a hypervisor pause, a
   scheduler preemption, a page fault storm, or it gets throttled by cgroup
   quota (topic 05, A8). Any of these can exceed 30 seconds; JVM full GCs on
   large heaps regularly do.
4. The lock's TTL expires. The lock service, correctly, releases it.
5. Client B acquires the lock and begins its critical section.
6. Client A wakes up. **It does not know it lost the lock.** It completes its
   write.

Now two clients wrote concurrently. The mutual exclusion the lock promised
was violated, and no component behaved incorrectly — this is the *designed*
behaviour of every TTL-based lock.

The root problem: a lock is a statement about the *past* ("I held it when I
checked"), and no amount of checking makes it a statement about the present.
You cannot close the gap by checking again before writing, because the
process can be paused between the check and the write. There is no
timing-based fix. This is Kleppmann's argument against Redlock, and the
important part is that it applies to **any** lease-based lock, not just
Redis.

**Fencing tokens** solve it by moving enforcement to the resource. The lock
service issues a **monotonically increasing token** with each grant. Every
write to the protected resource carries its token, and the **resource
rejects any write with a token lower than the highest it has seen**. In the
scenario above, A holds token 33 and B holds token 34; the storage system
has already accepted 34, so A's late write with 33 is rejected. Safety is
preserved regardless of how long A was paused.

The catch, which must be in the answer: **the resource must support it.**
If your storage is a plain S3 bucket or a filesystem with no conditional
write, you cannot fence, and the lock is advisory only. In practice this
means using a store with compare-and-set / conditional writes / optimistic
concurrency (a version column, a DynamoDB `ConditionExpression`, an S3
conditional PUT, a database `WHERE version = n`) — at which point you often
don't need the external lock at all, because the conditional write *is* the
mutual exclusion.

That's the punchline worth delivering: **if you can fence, you probably
didn't need a distributed lock; if you can't fence, the distributed lock
isn't safe.** Distributed locks are appropriate for *efficiency* (avoid
duplicate work, and duplicate work is merely wasteful) and inappropriate for
*correctness* (where duplicate work corrupts data) unless fenced.

**Weak answers miss.** That the failure is caused by process pauses rather
than network problems, and the "resource must enforce the token"
requirement. Saying "use Redlock" without the caveats is a red flag.

**Follow-ups to expect.**
- Does using etcd/ZooKeeper instead of Redis fix it? (No. They give you a
  *correct lock service* — linearizable, with proper consensus — which
  removes one class of bug (Redlock's reliance on unsynchronised clocks
  across independent Redis nodes), but the client-pause problem is
  unchanged. ZooKeeper's ephemeral nodes and `zxid` do give you a natural
  fencing token, which is the real advantage.)
- When is an unfenced lock acceptable? (When the worst case is doing work
  twice and that's idempotent or merely wasteful — e.g. cache warming,
  a cron job that's safe to run twice. Say the acceptance criterion
  explicitly.)

---

### A7. NTP vs PTP vs HLC vs TrueTime

**Answer.**
**NTP** — synchronises over the network with round-trip estimation.
Realistic accuracy: **single-digit to tens of milliseconds** over the
internet, sub-millisecond on a good LAN with local stratum-1 sources. It
can also step the clock backwards (unless slewing is configured), which
means `CLOCK_REALTIME` is **not monotonic** — a fact that breaks any code
measuring durations with wall-clock time. Use `CLOCK_MONOTONIC` for
durations, always.
Safe to use for: logging, dashboards, coarse expiry (a token valid for an
hour), roughly ordering events for humans.
Not safe for: ordering events for correctness, distributed locks, or
last-writer-wins conflict resolution — clock skew directly becomes data loss.

**PTP (IEEE 1588)** — hardware timestamping at the NIC plus
transparent/boundary clocks in the switches, removing the software stack and
queueing from the measurement. Accuracy: **sub-microsecond** on properly
provisioned hardware. Requires switch support end to end; a single
non-PTP-aware hop degrades it. Used in finance (regulatory timestamping),
telecoms, and increasingly in datacentres. It reduces uncertainty
dramatically but does not eliminate it, and it doesn't give you a *bound you
can program against* — which is the crucial difference from TrueTime.

**Hybrid Logical Clocks (HLC)** — a logical clock (Lamport-style) that
tracks physical time in its high bits. Each node keeps
`max(local physical time, highest timestamp seen) (+1)`. Properties: it
**captures causality** (if A happened-before B, then A's HLC < B's HLC), it
stays close to physical time so timestamps are human-meaningful and can be
used for TTLs, and it's **bounded** in how far it can run ahead. It does
*not* order concurrent events by real time — two independent writes can get
timestamps in the "wrong" real-time order.
Safe for: causal ordering, MVCC timestamps, conflict resolution where
causality is what matters. Used by CockroachDB, YugabyteDB, MongoDB.
Not safe for: claiming external consistency without an additional
assumption. CockroachDB assumes a **maximum clock offset** (default 500 ms)
and will *crash a node* whose clock drifts beyond it — that's the safety
mechanism, and it means an NTP failure becomes a node failure by design.

**TrueTime** (Spanner) — the key innovation is that it returns an
**interval** `[earliest, latest]` rather than a point, with a *guaranteed*
bound on uncertainty (single-digit milliseconds), backed by GPS receivers
and atomic clocks in every datacentre. Because the uncertainty is bounded
and *known to the program*, Spanner can do **commit wait**: after choosing a
commit timestamp, wait out the uncertainty interval before releasing locks,
which guarantees that any transaction starting later gets a strictly greater
timestamp. That's what buys external consistency.
The cost: every commit pays the uncertainty interval in latency, and the
whole thing depends on specialised hardware and a clock infrastructure most
organisations can't replicate. (AWS Time Sync Service and equivalents now
offer bounded-error clocks — verify current guarantees; this space is moving.)

**The organising insight**: NTP and PTP reduce error; TrueTime *bounds and
exposes* error; HLC sidesteps physical time for ordering. The design
question is never "how accurate is my clock" but "what happens when it's
wrong" — and a system that has no answer to that is unsafe regardless of how
good its NTP setup is.

**Weak answers miss.** That TrueTime's contribution is the *explicit bound*,
not the accuracy, and that `CLOCK_REALTIME` can go backwards.

**Follow-ups to expect.**
- Cassandra uses wall-clock timestamps for LWW. What's the risk? (Clock skew
  between coordinators means a later write can lose to an earlier one, and
  it fails silently — no error, just missing data. It's the strongest
  argument against LWW as a conflict strategy.)
- How would you detect clock problems in a fleet? (Monitor NTP offset and
  stratum per node, alert on offset above a threshold, and make the
  threshold smaller than any offset your software's correctness depends on.)

---

### A8. Exactly-once

**Answer.**
The impossibility, stated cleanly: a sender transmits a message and does not
receive an acknowledgement. It cannot distinguish between "the message was
lost" and "the message arrived and the ack was lost." It has exactly two
choices — resend (risking a duplicate: **at-least-once**) or not resend
(risking loss: **at-most-once**). There is no third option, because the
information needed to choose correctly does not exist at the sender. This is
the Two Generals problem and it is not an engineering limitation.

So "exactly-once **delivery**" is impossible. What people mean, and what is
achievable, is **exactly-once *processing* / effects**: the message may be
delivered many times, but its *effect* on the system happens once. That's
achieved by making the consumer idempotent, and there are three ways:

1. **Deduplication by identifier.** Every message carries a stable id; the
   consumer records processed ids and skips repeats. The dedup store must be
   updated **atomically with the effect** — otherwise you've just moved the
   problem. In practice that means the dedup record and the business write
   are in the same transaction, or the dedup key *is* a uniqueness
   constraint on the business write.
2. **Naturally idempotent operations.** `SET x = 5` instead of
   `x = x + 1`; upsert instead of insert; "state machine transition to state
   S if currently in state R" instead of "advance the state." This is by far
   the most robust approach — design the operation so repetition is
   meaningless — and it's an application design decision, not infrastructure.
3. **Transactional coupling of consume and produce.** Kafka's transactions
   (`transactional.id`, `isolation.level=read_committed`) make "read from
   topic A, write to topic B, commit offset" atomic *within Kafka*. That's
   real and useful and is what "Kafka exactly-once" means. Its scope is
   Kafka-to-Kafka. The moment your side effect is an external database, an
   HTTP call, or an email, the transaction cannot cover it, and you're back
   to idempotency.
   Also relevant: the **idempotent producer**
   (`enable.idempotence=true`), which uses a producer id and sequence
   numbers to dedupe retries at the broker — it removes duplicates caused by
   producer retries within a session, which is a narrower and often
   sufficient guarantee.

The sentence to say in an interview: **"Exactly-once delivery is impossible;
exactly-once processing is achievable, and it is always the consumer's
responsibility."** Anyone selling you exactly-once is either scoping it to a
closed system or relying on your idempotency.

**Weak answers miss.** That the dedup record must be atomic with the effect,
and that Kafka EOS is scoped to Kafka.

**Follow-ups to expect.**
- How long do you keep dedup state? (Longer than the maximum possible
  redelivery window — which for a system with replay capability could be
  the full retention. Bound it deliberately and accept duplicates beyond the
  window, or make the operation naturally idempotent so it doesn't matter.)
- Non-idempotent external effect, like charging a card? (Idempotency key
  passed to the provider — the provider dedupes. This is exactly why every
  payments API has one. Topic 02, A6.)

---

### A9. Two-phase commit and sagas

**Answer.**
**2PC**: a coordinator asks all participants to *prepare* — do the work,
acquire locks, write to durable log, and promise you can commit. If all vote
yes, the coordinator writes a commit decision and tells everyone to commit.
If any votes no, it aborts.

**The blocking failure**: a participant that has voted "prepared" has given
up its right to decide unilaterally. It holds locks and must wait for the
coordinator. If the **coordinator crashes after collecting votes but before
broadcasting the decision**, the prepared participants are stuck: they
cannot commit (the decision might have been abort), cannot abort (it might
have been commit), and cannot ask each other (a participant that hasn't
heard is in the same position). They hold locks — blocking other
transactions — until the coordinator recovers and reads its log.

That's the core weakness, and it's why 2PC is unsuitable for
loosely-coupled services: it converts an availability problem in one
component (the coordinator) into a liveness *and* throughput problem across
all participants. Its availability is the product of everyone's, and it
scales badly with participant count.

Mitigations exist but don't remove it: a durable coordinator log (mandatory),
coordinator replication via consensus (3PC's premise, and what Spanner
actually does — Paxos groups as participants and a replicated coordinator),
heuristic decisions by operators (which risk violating atomicity), and
timeouts that abort (which risk the opposite).

**Saga**: split the operation into a sequence of local transactions, each
committing independently, each with a **compensating transaction** that
semantically undoes it. If step 4 fails, run compensations for 3, 2, 1 in
reverse.

What a saga gives up:
- **Atomicity** in the ACID sense. Intermediate states are *visible* to
  other transactions. Someone can observe the order created but not yet
  paid.
- **Isolation** entirely. This is the big one, and the countermeasures are
  application-level: semantic locks (a "pending" status), commutative
  updates, pessimistic ordering of steps (do the reversible things first),
  and re-reading values before acting.
- **Clean rollback.** Compensation is not rollback — it's a *new*
  transaction with its own semantics. You cannot un-send an email; you send
  an apology. You cannot un-charge in all cases; you refund, which is a
  different financial event with different fees and different tax
  treatment. Designing correct compensations is the hard part and it is
  domain work, not infrastructure work.
- Compensations can themselves fail, so they must be retryable, idempotent,
  and eventually manual-intervention-able.

**When to use which**: 2PC within a trust and latency boundary you control,
with few participants, where you need real atomicity — e.g. across shards of
one database. Sagas across services and organisational boundaries, where
long-lived operations and independent availability matter more than
isolation. And most commonly: neither — restructure so the operation is
local to one transactional boundary, which is usually a signal that the
service decomposition was wrong.

**Weak answers miss.** That compensation is semantically different from
rollback, and the loss of isolation (which is what actually causes bugs in
saga implementations).

**Follow-ups to expect.**
- Orchestration vs choreography? (Central orchestrator: explicit, debuggable,
  a single place that knows the state, but a component to run and a coupling
  point. Choreography via events: decoupled, but the workflow exists only
  implicitly across services and nobody can answer "where is order 123
  stuck." I'd default to orchestration for anything with more than three
  steps, and say why.)
- How do you make a saga observable? (Persist the saga state machine with
  every step's status; that record *is* the answer to "what happened," and
  without it you're reconstructing from logs.)

---

### A10. Kafka durability configuration

**Answer.**
**ISR (In-Sync Replicas)**: the set of replicas the leader considers caught
up — those that have fetched within `replica.lag.time.max.ms`. A replica
that falls behind is removed from the ISR; when it catches up it rejoins. A
message is **committed** (and thus readable by consumers) once it's
replicated to all members of the ISR.

**`acks`** — the producer's durability request:
- `acks=0`: fire and forget. No delivery guarantee whatsoever.
- `acks=1`: the *leader* has written it to its log (not necessarily fsynced;
  Kafka relies on replication, not fsync, for durability). If the leader
  dies before followers replicate, the message is lost.
- `acks=all` (`-1`): all **in-sync** replicas have it.

**`min.insync.replicas`** — a topic/broker setting that only takes effect
with `acks=all`. It's the minimum ISR size for a write to be accepted; if
the ISR shrinks below it, the producer gets
`NotEnoughReplicasException` and the partition becomes **write-unavailable**.
This is the knob that makes `acks=all` meaningful.

**Unclean leader election** (`unclean.leader.election.enable`): if all ISR
members are unavailable, may an out-of-sync replica become leader? `true` =
yes, choosing availability and **accepting silent data loss** (the new
leader's log is shorter; committed messages beyond its offset vanish, and
consumers may see the log truncate). `false` = no, the partition stays
offline until an ISR member returns. Default is `false` in modern versions,
and it should stay false for anything you care about.

**The configuration that loses acknowledged data — the point of the
question.** The subtle one is:

> `acks=all` with `min.insync.replicas=1` and `replication.factor=3`.

Here's why it's dangerous despite looking safe. `acks=all` means "all
in-sync replicas." If followers are lagging and drop out of the ISR, the ISR
can shrink to just the leader. With `min.insync.replicas=1` that's still
acceptable, so the producer gets a successful ack for a write that exists on
**exactly one broker**. That broker's disk dies, or the instance
terminates — the acknowledged message is gone. The producer was told
"durably replicated to all in-sync replicas" and it was, technically, and it
still lost data.

The correct setting is `replication.factor=3`, `min.insync.replicas=2`,
`acks=all`. Then every acknowledged write is on at least two brokers, and
you can lose one broker with no loss. You accept that losing two brokers in
a partition's replica set makes it write-unavailable — which is the correct
trade, and the reason `min.insync.replicas=RF` is wrong (it makes a single
broker failure block writes).

Other loss configurations to name:
- `acks=1` + leader failure before replication → lost. Common default in
  older clients and a frequent silent data-loss source.
- `unclean.leader.election.enable=true` → committed data lost on failover.
- Producer with `retries=0` or an application that ignores the send
  callback/future → loss on any transient failure. **The producer is
  asynchronous by default**; not checking the result is the most common
  application-level loss bug.
- Consumer with `enable.auto.commit=true` committing offsets *before*
  processing completes → messages skipped on crash. This is loss at the
  other end, and it's a default.

**Weak answers miss.** The `acks=all` + `min.insync.replicas=1` trap. That
specific combination is what the question is designed to surface, and it's
worth knowing cold given Kafka is on your resume.

**Follow-ups to expect.**
- Does Kafka fsync per message? (No — `flush.messages`/`flush.ms` default to
  relying on replication rather than fsync, on the theory that N replicas'
  page caches are safer than one disk. Correlated failure — a whole rack
  losing power — breaks that assumption, which is why rack-aware replica
  placement matters.)
- What does rack awareness give you? (`broker.rack` + the replica assigner
  spreads a partition's replicas across racks/AZs, so an AZ failure doesn't
  take the whole ISR. Also relevant to cross-AZ cost — topic 04, A13 — and
  follower fetching lets consumers read from a same-AZ replica.)
- What happens to a consumer group during a rebalance? (Stop-the-world in
  the eager protocol — all consumers stop, partitions are reassigned.
  Frequent rebalances (from `max.poll.interval.ms` violations) can prevent
  progress entirely. Cooperative/incremental rebalancing and static group
  membership (`group.instance.id`) are the fixes.)

---

### A11. Little's law and the utilisation curve

**Answer.**
**Little's law**: `L = λ × W` — the average number of items in a system
equals the arrival rate times the average time each spends there. It's
distribution-free: it holds for any stable system regardless of arrival
pattern or service-time distribution, which is what makes it so useful.

Rearranged for practical use: `W = L / λ`. If you observe a queue of 500
items and a throughput of 100/s, items are waiting 5 seconds — no
instrumentation of individual requests required. Or: to serve 1000 req/s at
50 ms each, you need 50 concurrent workers. That's your thread pool, your
connection pool, and your capacity plan in one line.

**The utilisation curve.** For an M/M/1 queue, the expected wait is
proportional to `ρ / (1 − ρ)` where ρ is utilisation. Plug in numbers:

| Utilisation | Queueing multiplier `ρ/(1−ρ)` |
| --- | --- |
| 50% | 1.0 |
| 70% | 2.3 |
| 80% | 4.0 |
| 90% | **9.0** |
| 95% | **19.0** |
| 99% | **99.0** |

So going from 70% to 90% utilisation — a 29% increase in throughput —
roughly **quadruples** queueing delay. From 90% to 95% doubles it again.
The curve is hyperbolic: it's nearly flat until it isn't, and there is no
warning in the utilisation metric itself. A dashboard showing 85% CPU looks
"fine" and is one traffic increment away from a latency cliff.

That's the answer to "why is 90% so different from 70%": you're not on a
linear part of the curve, and the *variance* is worse than the mean —
tail latency degrades much faster than average latency, because queueing
delay's variance grows even faster than its mean.

Consequences worth stating:
- **Headroom is not waste.** Running at 50–70% is buying latency
  predictability, and the correct target depends on how bursty your arrivals
  are and how fast you can add capacity.
- Real systems are worse than M/M/1 because arrivals are bursty (not
  Poisson) and service times are variable — variability in either shifts the
  knee left. The Kingman approximation makes this explicit: wait scales with
  `(C_a² + C_s²)/2`, so *reducing variance* is as effective as adding
  capacity. Which is an argument for smoothing traffic (rate limiting,
  batching) and for reducing service-time variance (caching, timeouts,
  isolating slow paths).
- **Autoscaling on utilisation is scaling on the wrong signal** — by the
  time utilisation moves meaningfully, latency has already gone. Scale on
  queue depth or concurrency, which are leading indicators (and are exactly
  what Little's law relates to latency).

**Weak answers miss.** The actual numbers. This is a question where reciting
"latency increases nonlinearly" is worth much less than saying "90% gives
you 9x the queueing of 50%."

**Follow-ups to expect.**
- How does this apply to a thread pool? (Pool size is L. If your pool is 200
  and service time is 100 ms, max throughput is 2000/s — and if arrivals
  exceed that, the *queue in front of the pool* grows without bound and every
  request's latency includes the whole queue. Bounded queue + shedding is
  the only stable configuration.)
- Where else does this bite? (Storage queue depth — topic 05, A12 — is
  literally the same curve.)

---

### A12. Failure detection

**Answer.**
**Fixed heartbeat timeout**: declare a node dead after missing N heartbeats.
Simple, and the tuning is a direct tradeoff — short timeout means fast
detection and **false positives**; long timeout means no false positives and
slow detection. There is no setting that is both, because the underlying
problem is undecidable: **you cannot distinguish a crashed node from a slow
node or a partitioned network.** That's the FLP-adjacent insight and it's
the "why is every timeout wrong" part of the question.

The costs of getting it wrong are asymmetric and situation-dependent:
- False positive on a database primary → unnecessary failover, potential
  split brain (topic 06, A10), a write outage during the transition.
- False negative → a dead node continues receiving traffic, and requests
  time out at the client instead of failing fast.
- False positives are also *correlated with load*: a GC pause, a CPU spike,
  or network congestion during peak traffic causes heartbeats to be late
  precisely when eviction hurts most. A naive detector will evict healthy
  nodes during your busiest minute, making the incident worse. This is a
  real and common cascading-failure mechanism.

**Phi-accrual** (Hayashibara et al., used in Cassandra and Akka): instead of
a boolean, output a continuous suspicion level φ. It maintains a sliding
window of recent heartbeat inter-arrival times, fits a distribution, and
computes φ = −log₁₀(probability that a heartbeat this late is normal).
Advantages: it **adapts** to the observed network — a link with naturally
variable latency gets a wider tolerance automatically — and it lets
different consumers apply different thresholds. A cheap action (stop routing
new requests) can trigger at φ=3; an expensive, dangerous one (promote a new
leader) at φ=12. That decoupling of "how suspicious" from "what to do" is
the real contribution.

**Lease-based liveness**: instead of the observer deciding the node is dead,
the **node itself** must periodically renew a lease from an authority. If it
cannot renew, it **self-demotes** — stops serving, steps down as leader —
*before* the authority hands the role to someone else. This inverts the
safety argument: correctness no longer depends on the observer's judgement,
only on the node's ability to observe its own clock and its own failure to
renew. It's what makes automated failover safe (Patroni's leader key with
TTL, Chubby/ZooKeeper session leases, Raft's leader lease). The residual
assumption is bounded clock drift on the node, plus the possibility that a
paused process (topic 07, A6) wakes up after its lease expired — which is
why leases pair with fencing tokens.

**How I'd choose:** phi-accrual or an adaptive detector for *routing*
decisions where a mistake is cheap and reversible; leases plus fencing for
*role* decisions where a mistake is expensive; and never a bare fixed
timeout on a critical role.

Two more practical points:
- **Detect from multiple observers.** A single observer's view conflates
  "the node is down" with "my path to it is down." Requiring agreement from
  several observers (or a quorum) distinguishes node failure from network
  failure — and gossip-based membership does this implicitly.
- **Cap the eviction rate.** Whatever the detector says, never remove more
  than X% of the fleet — the same fail-open logic as health checking (topic
  03, A5). If your detector says everything is dead, the detector is wrong.

**Weak answers miss.** Self-demotion (the lease inversion) and the
correlation between false positives and peak load.

**Follow-ups to expect.**
- SWIM protocol? (Gossip-based membership with indirect probing: if A can't
  reach B, it asks C and D to probe B. Distinguishes "B is down" from "the
  A–B path is down" for a small constant cost, and it's how Consul/Serf and
  many meshes do membership. Good answer to the multiple-observers point.)

---

### A13. Gray failure

**Answer.**
A **gray failure** is a component that is not down but is not working
correctly either: slow, intermittently erroring, corrupting a fraction of
responses, or working for some callers and not others. The defining property
(Huang et al.) is **differential observability** — the system's own health
checks say healthy while the actual users experience failure. The failure is
invisible to exactly the mechanism designed to catch it.

**Why "up but slow" is worse than "down":**

1. **Nothing removes it.** A crashed node fails its health check and is
   ejected in seconds. A node responding to `/health` in 2 ms while serving
   real requests in 30 seconds passes every check and keeps receiving its
   full share of traffic.
2. **It consumes the caller's resources.** A fast failure returns the
   caller's thread, connection, and deadline immediately, and the caller
   retries elsewhere. A slow response holds all of them for the full timeout.
   With enough traffic to a slow dependency, the *caller's* thread pool or
   connection pool fills, and the caller becomes unavailable to everyone —
   including for requests that never touch the slow dependency. This is how
   one degraded node takes down a whole service tier, and it's the single
   most important mechanism in the answer.
3. **Retries make it worse.** The client times out, retries, and the retry
   also lands on a node that is slow — or, worse, on the same one. Now the
   degraded node has more load than before (topic 02, A16).
4. **It's ambiguous, so humans hesitate.** "Is it broken?" takes far longer
   to answer than "it's down," so MTTR is dominated by diagnosis rather than
   repair.
5. **Load balancing may actively prefer it.** A node that errors *quickly*
   has low latency and few in-flight requests — so a least-connections or
   latency-based balancer will send it **more** traffic. A fast-failing node
   becomes a black hole that attracts traffic. This is a genuinely
   counterintuitive failure and worth naming.

**Defences:**
- **Measure what users measure.** Health checks should exercise the real
  request path (or better, use *passive* outlier detection on real request
  outcomes — topic 03, A5), not a synthetic endpoint.
- **Latency-aware ejection**, not just error-rate ejection. A node whose p99
  is 10x its peers' should be ejected even at a 0% error rate.
- **Aggressive client-side timeouts and bounded concurrency per
  dependency** (bulkheads), so a slow dependency cannot consume more than a
  fixed slice of the caller's resources. This is the highest-value
  structural defence.
- **Circuit breakers** to stop sending to a degraded dependency at all.
- **Hedged requests** to route around per-node variance for read paths.
- **Compare nodes against each other**, not against absolute thresholds. The
  strongest gray-failure signal is "this node is different from its
  identical peers," and it needs no threshold tuning.
- Instrument success rate and latency **per (client, server) pair** so you
  can see a failure that affects only some callers.

**Weak answers miss.** The caller-resource-exhaustion mechanism and the
"fast failures attract traffic" inversion.

**Follow-ups to expect.**
- How would you detect a node returning *wrong* answers rather than slow
  ones? (Much harder: checksums/canary queries with known answers, shadow
  comparison against peers, or invariant checks in the response. Say
  honestly that this is one of the hardest classes of failure and that
  detection usually comes from downstream data quality alarms, not
  infrastructure monitoring.)

---

### A14. Cells and shuffle sharding

**Answer.**
**Cell-based architecture**: partition the entire system — not just the
data, but the full stack: load balancers, services, databases, caches — into
independent **cells**, each serving a subset of customers. A cell is a
complete, self-sufficient instance of the service. Customers are assigned to
exactly one cell, and cells share nothing on the request path.

Blast-radius property: **a failure is bounded to one cell.** With 10 cells,
any single-cell failure — a bad deploy, a poison record, a database
saturation, a cache stampede — affects at most 10% of customers instead of
100%. And critically it bounds *unknown* failure modes: you don't need to
predict how it will fail for the bound to hold, which is what makes it
qualitatively different from other resilience measures.

Second-order benefits, which are often the real motivation: cells give you a
natural **deployment unit** (roll out cell by cell; a bad change is caught at
10% and rolled back), a **bounded scale unit** (you know a cell's capacity
because you've tested one, so growth means more cells rather than a
re-architecture), and a **testable failure story** (you can drain a cell as
a routine operation).

Costs, stated honestly: significant operational complexity — you're running
N of everything; a **routing layer** that maps customer → cell, which must
be highly available and is itself a shared component (the one thing that
isn't cellularised, so it must be extremely simple); poorer utilisation,
since each cell needs its own headroom; harder cross-customer operations and
analytics; and the largest single customer must fit in one cell.

**Shuffle sharding** solves a different problem: how to assign customers to
a *subset* of shared resources so that a "poison" customer damages as few
others as possible.

Instead of assigning each customer to one shard of N, assign each to a
**random subset of k shards** (say k=2 out of 8 workers). Two customers
collide completely only if they draw the *same* pair. The number of distinct
pairs from 8 choose 2 is 28, so a given customer shares its full assignment
with roughly 1/28 of the population; the rest overlap partially and still
have at least one healthy shard.

The arithmetic scales beautifully: with 100 workers and k=5, there are
`C(100,5) ≈ 75 million` combinations. A single abusive customer takes down 5
workers; the probability that another specific customer's entire set of 5 is
inside those same 5 is `1 / C(100,5)` — effectively zero. So **one bad
tenant degrades a vanishing fraction of others completely**, while most see
partial degradation they can retry through. You get near-per-customer
isolation from a shared pool, without paying for per-customer capacity.

Requirements for it to work: the client must be able to **retry against
another shard in its set** (otherwise partial availability doesn't help),
and shard assignment must be stable per customer.

**How they compose:** cells for coarse, hard isolation of the whole stack;
shuffle sharding for fine-grained isolation *within* a cell's shared
resources. AWS uses both — it's the architecture behind Route 53's and
several other services' resilience — and saying that they're complementary
rather than alternatives is the right framing.

**Weak answers miss.** That cells bound *unknown* failure modes (the
argument that makes them worth the cost), and the combinatorial arithmetic
for shuffle sharding.

**Follow-ups to expect.**
- How do you size a cell? (Small enough that losing one is acceptable, large
  enough to be economical and to hold your biggest customer. It's an explicit
  blast-radius-vs-cost decision — and you should be able to state the
  percentage of customers a cell represents as a deliberate number.)
- How do you migrate a customer between cells? (A migration, with the same
  shape as topic 06, A16 — which is why cell assignment should be a
  changeable mapping, not a hash.)
- What's the one thing that can't be cellularised? (The routing layer, and
  any global control plane. Keep them as simple and as static as possible —
  ideally the router is a lookup in a rarely-changing table, so it has almost
  no failure modes of its own.)

---

### A15. Multi-region conflict resolution

**Answer.**
**Last-writer-wins (LWW)**: each write carries a timestamp; on conflict, the
higher timestamp wins.
- Pro: trivially simple, no coordination, converges deterministically,
  constant metadata.
- Con: **it silently discards data**. Two users edit different fields of the
  same record in two regions; one entire version is thrown away with no
  error and no record. And "later" depends on **clock skew** (A7) — with NTP
  skew of tens of milliseconds, the write that happened second can lose. The
  failure is invisible: no conflict is reported, the data is just wrong.
- Use it when: the value is naturally a snapshot where only the newest
  matters (a status flag, a cached rendering, a last-seen timestamp), and
  losing an update is acceptable. DynamoDB global tables and Cassandra use
  it, so if you use those multi-region, you are choosing this whether you
  meant to or not.

**CRDTs** (conflict-free replicated data types): types whose merge operation
is commutative, associative, and idempotent, so replicas converge regardless
of the order updates arrive.
- Pro: **no data loss and no coordination** — you can accept writes in every
  region during a partition and still converge. Strong eventual consistency
  by construction.
- Con: only certain semantics are expressible. G-counters, PN-counters,
  OR-sets, LWW-registers, and sequence CRDTs for text (the basis of
  collaborative editors) are the practical vocabulary. Anything requiring a
  **global invariant** — "stock must not go below zero," "this username is
  unique," "the account balance must not go negative" — cannot be a CRDT,
  because enforcing it requires knowing about writes you haven't seen. Also:
  metadata overhead (tombstones, version vectors) that grows and must be
  garbage-collected, which is a real operational burden; and the merged
  result can be *semantically* surprising even when it's mathematically
  correct (concurrent add and remove on a set — which wins is a design
  choice you must make and explain to users).
- Use it when: collaborative editing, presence, shopping carts (Dynamo's
  original motivation — a merged cart that resurrects a removed item is
  better business than a lost cart), counters, feature flags.

**Per-region ownership (partitioned writes)**: every record has exactly one
region that may write it. Other regions read locally (async replica) and
forward writes to the owner.
- Pro: **no conflicts exist**, because there is only ever one writer. You
  keep normal single-region transactional semantics — real invariants, real
  uniqueness constraints, real transactions. Reads are local and fast
  everywhere.
- Con: writes from a non-owning region pay the cross-region round trip
  (70–90 ms US–EU order of magnitude), so write latency is unequal by
  geography. And if the owning region goes down, that data is **read-only**
  until you fail over ownership — which is a coordination problem in its own
  right and must be designed, not improvised (you need a fencing/epoch
  mechanism so the old owner can't resume writing — topic 06, A10).
- Use it when: data has natural geographic locality — users, accounts,
  tenants, inventory at a specific warehouse. Which is most business data.

**My default recommendation and why**: **per-region ownership** for anything
with invariants, because it's the only one of the three that lets you keep
ordinary transactional reasoning; CRDTs for the specific data types that
genuinely need concurrent multi-region writes and have no invariant; LWW
only where you've explicitly decided that losing an update is fine, and
documented it. The failure mode of the industry is picking LWW by accident —
by choosing a datastore whose multi-region mode is LWW — and discovering the
data loss months later.

Worth adding: a fourth option is **synchronous consensus across regions**
(Spanner, CockroachDB), which eliminates conflicts by ordering everything.
That's correct and costs you a cross-region round trip on every write. If
your write rate and latency budget can absorb it, it's the simplest thing to
reason about, and dismissing it without pricing it is a mistake.

**Weak answers miss.** That CRDTs cannot enforce global invariants, and that
LWW's data loss is silent. Also missed: the ownership-failover problem,
which is where per-region ownership gets hard.

**Follow-ups to expect.**
- How do you assign ownership? (By a stable attribute — user's home region,
  tenant's contracted region, warehouse location. Store the owner in the
  record so any region can route. Changing it is a small migration with a
  fencing step.)
- What about a global uniqueness constraint like usernames? (Needs a single
  coordination point — one region owns the namespace, or a consensus-backed
  registry. This is the case that proves you can't have everything, and the
  usual pragmatic answer is a global service for that one narrow thing.)

---

## Tier 3 — Scenario / debug

### A16. Metastable failure

**Answer.**
**What's happening**: the system has entered a **metastable failure state** —
a self-sustaining bad equilibrium. The original trigger created an overload;
the system's *response* to the overload generates enough additional work to
keep itself overloaded, so removing the trigger doesn't help. There are two
stable states (healthy and failed) at the same input load, and a large enough
perturbation moved you from one to the other.

The mechanism is always some **positive feedback loop where failure creates
work**:
- **Retries.** Each failure produces N more requests. Offered load rises
  with the error rate, which raises the error rate (topic 02, A16). The
  canonical case.
- **Cache collapse.** Load spike → timeouts → cache not being populated →
  hit rate falls → more backend load → more timeouts. Once the hit rate
  collapses, the backend can no longer serve even the *original* load,
  because it was sized assuming a 95% hit rate. This one is especially nasty
  because restoring the cache requires serving requests you can't serve.
- **Queue buildup past deadline.** The queue is full of requests whose
  clients have already given up. The system is spending 100% of its capacity
  producing responses nobody will read, so its *useful* throughput is zero
  while its utilisation is 100%.
- **Connection/thread exhaustion.** Slow responses hold resources, reducing
  concurrency, making responses slower.
- **Eviction cascades.** Health checks fail under load, capacity is removed,
  remaining nodes get more load, they fail health checks too.
- **GC death spiral**: memory pressure → more GC → less CPU for work → queue
  grows → more memory → more GC.

**How you get out.** The only reliable exit is to **break the feedback loop
by force**, which means shedding load below the level that sustains it — and
that level is *lower* than the load the system handled fine before the
incident, because you now have a cold cache and a full queue. That's the
key, counterintuitive point: restoring normal traffic will just re-enter the
bad state.

In order:

1. **Shed hard, immediately.** Drop a large fraction of traffic at the edge —
   or all of it. Prioritise by request class if you can; if you can't, drop
   uniformly. This is the step people delay because it feels like giving up,
   and delaying it extends the outage.
2. **Drain the queues.** Purge requests older than their deadline; there is
   no value in serving them and they're consuming the capacity you need.
   Same for in-flight retries.
3. **Disable retries** globally, or clamp retry budgets to zero, while you
   recover.
4. **Warm the cache** deliberately if cache collapse is the loop — with
   synthetic or replayed traffic under controlled concurrency, not by
   letting user traffic in.
5. **Ramp traffic back slowly**, in steps, watching the error rate and the
   cache hit rate at each level. If you re-enter the bad state you'll know
   which level is the tipping point, and that number is worth recording.
6. Add capacity if you have it — but note that adding capacity mid-metastable
   often doesn't help, because new instances arrive cold and cause the
   problems in topic 03, A16.

**What makes this possible before the incident** — and this is the part that
distinguishes a staff answer, because during the incident your options are
determined entirely by what you built beforehand:
- A **traffic control point** that can shed at a settable percentage, by
  class, without a deploy. If turning off traffic requires a code change,
  you cannot do step 1.
- **Bounded queues everywhere.** Unbounded queues are the substrate that
  metastability grows in.
- **Deadline propagation** so work past its deadline is dropped rather than
  completed.
- **Retry budgets**, so the amplification factor is bounded by construction
  and the loop cannot form.
- **A tested cold-start path**, so you know the system can come up with an
  empty cache.
- Load tests that specifically probe *recovery*: push the system into
  overload, remove the load, and verify it recovers on its own. Most load
  testing measures the peak and never tests the return path, which is
  exactly the property metastability violates.

**Weak answers miss.** That you must shed below the *pre-incident* level,
and that the ability to shed is a pre-built capability rather than an
in-incident decision.

**Follow-ups to expect.**
- How do you know you're in a metastable state rather than still-overloaded?
  (Remove the trigger and see. If load is back to normal and errors are not,
  the loop is self-sustaining. Practically: check whether *offered* load —
  including retries — is still elevated while *user* load is normal.)
- Which is the most common loop in practice? (Retries, by a wide margin, and
  it's the cheapest to prevent. Budgets first.)

---

### A17. 3–2 partition of a 5-node cluster

**Answer.**

**Raft-based system (etcd, Consul, a Patroni-managed Postgres, CockroachDB
range):**

*Majority side (3 nodes):*
- If the leader is here, it retains leadership — it can still reach a
  majority (itself plus 2), so it keeps committing entries. No interruption.
- If the leader is on the minority side, these 3 detect missed heartbeats,
  hold an election, and elect a new leader with a higher term. Unavailable
  for writes for roughly one election timeout plus the election round trip —
  typically hundreds of milliseconds to a couple of seconds.
- Serves reads and writes normally afterwards.

*Minority side (2 nodes):*
- Cannot elect a leader — no majority. They will campaign, fail, increment
  their terms, and retry.
- If the old leader is here, it **cannot commit** anything: it can't
  replicate to a majority. A correct implementation also **steps down** once
  it fails to hear from a majority within its lease/election timeout, which
  is essential — otherwise it might serve stale reads believing itself
  leader.
- Clients on this side get errors or timeouts. **This is correct
  behaviour**: the system is choosing consistency over availability (CP).

*Clients:* those routed to the minority see failures; those on the majority
side see a brief blip at most. If your client routing is DNS- or
VIP-based, some clients may keep hitting minority nodes — which is why
clients should be aware of the cluster and follow leadership, not a static
address.

*On heal:* minority nodes discover the higher term, revert to followers, and
the leader replicates the log to them. Any entries the old leader had
**appended but not committed** are **truncated and discarded** — they were
never acknowledged to clients, so no acknowledged data is lost. Convergence
is automatic and requires no operator action. That's the property that makes
consensus worth its cost.

**Leaderless quorum system (Cassandra/Dynamo-style, N=5):**

Behaviour depends entirely on the consistency level, which is a *per-query*
choice — that's the first thing to say.

*With QUORUM (3 of 5):*
- Majority side: reaches 3 replicas, so reads and writes succeed.
- Minority side: can only reach 2, so QUORUM operations **fail**. Same CP
  behaviour as Raft for those queries.

*With ONE or LOCAL_ONE:*
- **Both sides accept writes.** Each side is internally consistent-ish and
  globally divergent. This is the AP configuration, and it's where the
  interesting behaviour is.

*With hinted handoff / sloppy quorum enabled:* the majority side may accept
writes on behalf of the unreachable replicas, storing hints. This preserves
availability and further weakens the quorum-overlap guarantee (A4).

*Clients:* both sides may appear to work. Users on different sides see
different data. Nothing errors, which is the dangerous part — the failure is
silent.

*On heal:* replicas exchange data via **read repair** (on the next read of a
divergent key), **hinted handoff** replay, and **anti-entropy repair**
(Merkle-tree comparison, usually a scheduled operation). Conflicts are
resolved by **last-writer-wins on cell timestamp**, so **concurrent writes
to the same cell on both sides silently lose one** — and which one depends
on clock skew (A7, A15). There's no error, no conflict report, no operator
action. Divergent data that is never read may persist until a scheduled
repair runs, which is why repair schedules are an operational requirement,
not an optimisation.

**The contrast to draw:** the Raft system was unavailable on the minority
side and lost nothing. The leaderless system stayed available on both sides
and lost data silently. Neither is "better" — they're the two ends of the
CAP choice, made concrete. The mistake is running the AP system while
believing you have the CP one, which is exactly what "R+W>N gives strong
consistency" (A4) leads people into.

**Weak answers miss.** The old-leader step-down requirement, uncommitted-log
truncation on heal, and that the leaderless case depends on the per-query
consistency level rather than a cluster-wide setting.

**Follow-ups to expect.**
- What if the partition is 2–2–1? (Nobody has a majority; the Raft system is
  entirely write-unavailable. Worth knowing that multi-way partitions are
  worse than two-way, and that this is an argument for placing nodes across
  exactly 3 failure domains, not 5.)
- What about an asymmetric partition — A can reach B but B cannot reach A?
  (Much nastier: a node can win an election it shouldn't, or heartbeats
  arrive in one direction only, causing repeated leader churn. Raft's term
  mechanism keeps it *safe*, but liveness can suffer badly. Real networks do
  this, and it's a good example of why "partition" is a simplification.)

---

### A18. At-least-once, at-most-once-per-schedule job runner

**Answer.**
First, name the tension in the requirement: **"at least once" and "at most
once per scheduled time" together mean exactly-once execution per
occurrence**, and A8 says that's impossible to *deliver*. So the design must
make execution **idempotent per occurrence**, and everything else is about
minimising duplicate *work* rather than eliminating duplicate *delivery*.

**Core model.** The unit is an **occurrence**: `(job_id, scheduled_time)`.
That composite key is the idempotency key for the entire system, and every
mechanism below hangs off it. A retry of the 09:00 run is the *same*
occurrence; the 10:00 run is a different one.

**1. Scheduling — deciding what should run.**
A scheduler component computes due occurrences and inserts them into a
durable store with a **unique constraint on (job_id, scheduled_time)**. That
constraint is what makes scheduler duplication harmless: run three scheduler
replicas for availability and let them race; the second and third inserts
fail on the constraint. No leader election is needed for correctness, only
for efficiency.
Never derive "what should run now" from wall-clock triggers alone — a
scheduler that was down from 08:55 to 09:05 must, on recovery, look back and
materialise the missed occurrence. Compute from the schedule and the last
materialised occurrence, not from "now."

**2. Assignment — a lease, not a lock.**
A worker claims an occurrence with an atomic conditional update:

```sql
UPDATE occurrences
   SET status='running', owner=:worker, lease_expires=now()+interval '60s',
       attempt=attempt+1, fencing_token = fencing_token + 1
 WHERE (job_id,scheduled_time)=(:j,:t)
   AND (status='pending' OR (status='running' AND lease_expires < now()))
RETURNING fencing_token;
```

Single atomic statement, so exactly one worker wins. `SELECT ... FOR UPDATE
SKIP LOCKED` is the equivalent for batch claiming.
The worker **renews the lease** while running (heartbeat every ~20 s on a
60 s lease). If it dies, the lease expires and another worker reclaims. That
gives you at-least-once.

**3. The unavoidable hazard, and the fencing token.**
A worker can be paused (GC, cgroup throttling, hypervisor stall — A6) past
its lease expiry, have the occurrence reclaimed, and then wake up and write
its results. Two workers' output for one occurrence.

Mitigations, in order of strength:
- **Fencing token** issued with the lease and carried into every write the
  job makes. The target rejects writes with a stale token. Only works if the
  target supports conditional writes — which is the same caveat as A6, and
  it's why the job's *output* store matters as much as the scheduler.
- **Self-check before committing**: the worker re-reads its lease and
  verifies it still owns the occurrence, in the *same transaction* as the
  result write, if the results live in the same database. That makes the
  ownership check and the effect atomic, which is the strongest available
  option when you control the store.
- **Idempotent job effects** keyed on the occurrence: writes are upserts
  keyed by `(job_id, scheduled_time)`; external calls carry the occurrence
  as an idempotency key. This is the real answer, and everything else is an
  optimisation to avoid wasted work.
- **Self-abort**: the worker checks its remaining lease before each
  significant step and aborts if it's expired. Cheap, and it converts most
  pause cases into clean failures.

**4. Completion.**
Mark `status='succeeded'` with the result, conditioned on still holding the
lease and the fencing token. On failure: `status='failed'`, record the
error, and schedule a retry with **exponential backoff and jitter**, bounded
by a max attempt count, after which it goes to a dead-letter state and pages
someone. Retries of the same occurrence keep the same key, so idempotency
still holds.

**5. The failure modes I'd design for explicitly:**

| Failure | Behaviour |
| --- | --- |
| Worker crashes mid-job | Lease expires, another worker reclaims. Duplicate work; idempotency makes it safe. |
| Worker pauses past lease | Reclaimed; fencing token/conditional write rejects the zombie's output. |
| Scheduler down at trigger time | On recovery, backfill missed occurrences from the schedule. Decide policy: run all missed, or only the latest (**catch-up vs skip**) — this must be per-job configuration, since a report wants the latest only and a billing run wants all of them. |
| Database (state store) down | No claims, no progress. This is the single point of failure; it needs the HA story of a primary datastore. Workers already running should complete and retry their status write. |
| Clock skew between workers | Never compare wall clocks across workers. Lease expiry is evaluated **by the database**, using the database's clock, in the conditional update. This eliminates the entire class. |
| A job runs longer than its lease | Lease renewal handles it, provided renewal is on a separate thread from the work. If the work blocks the renewal thread, you get a false reclaim — a real bug, so renewal must be independent. |
| A job runs longer than its interval | Overlap policy per job: skip, queue, or allow concurrent. Must be explicit or you get pile-ups. |
| Poison job that kills every worker | Attempt counter with a cap; quarantine after N attempts. Without this, one bad job cycles through and crashes your entire worker fleet. |

**6. What I'd deliberately not build.** No distributed lock service — the
database's conditional update *is* the lock and it's atomic with the state I
care about, so introducing etcd/Redis would add a component and a
cross-system consistency problem for no gain. No exactly-once claim; I'd
document at-least-once with idempotent effects and make job authors sign up
to it, because a documented weaker guarantee that holds beats a stronger one
that doesn't.

**Scaling note if pressed:** the single state store is the bottleneck and
the SPOF. Shard occurrences by `hash(job_id)` across partitions, each with
its own workers, or move to a purpose-built system (Temporal, which
essentially productises this design with durable execution, or a
Kafka-partitioned assignment model). But I'd start with the database — a
single Postgres handles a very large number of occurrences per second, and
the operational simplicity is worth a lot.

**Weak answers miss.** The `(job_id, scheduled_time)` composite key as the
idempotency unit, evaluating lease expiry with the *database's* clock, and
the catch-up-vs-skip policy decision. Proposing a distributed lock without
fencing is the specific failure this question is designed to catch.

**Follow-ups to expect.**
- What if a job must *never* run twice, at any cost? (Then the effect must be
  transactional with the ownership check in one store, or you need the
  external system to enforce idempotency. If neither is possible, the honest
  answer is that you cannot guarantee it, and you should say so and design a
  detection/reconciliation process instead of pretending.)
- How do you handle time zones and DST? (Store schedules with an explicit
  time zone and materialise occurrences in UTC. DST transitions create
  occurrences that happen twice or not at all — a real correctness issue for
  daily jobs, and having thought about it is a good signal.)
