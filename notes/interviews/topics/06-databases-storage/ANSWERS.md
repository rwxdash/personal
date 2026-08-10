# Databases & Storage — Answers

---

## Tier 1 — Recall

### A1. Isolation levels

**Answer.**
| Level | Dirty read | Non-repeatable read | Phantom read |
| --- | --- | --- | --- |
| Read Uncommitted | possible | possible | possible |
| Read Committed | prevented | possible | possible |
| Repeatable Read | prevented | prevented | possible (per the standard) |
| Serializable | prevented | prevented | prevented |

- **Dirty read**: you see another transaction's uncommitted data.
- **Non-repeatable read**: you read a row twice in one transaction and get
  different values because someone committed in between.
- **Phantom read**: you run the same *range* query twice and get a different
  set of rows because someone inserted into the range.

The important caveats, which are what the question is actually testing:

- **The names are standard; the implementations are not.** PostgreSQL's
  Read Uncommitted behaves as Read Committed (it never allows dirty reads).
  PostgreSQL's Repeatable Read is snapshot isolation and **does prevent
  phantoms**, unlike the standard's requirement. Oracle's Serializable is
  snapshot isolation, not true serializability.
- **Snapshot isolation still permits write skew**, which is the anomaly the
  standard's three-anomaly taxonomy doesn't name. Two transactions each read
  an overlapping set, each check an invariant, and each write a disjoint row
  — both commit, invariant violated. The canonical example: two doctors both
  going off-call because each sees the other is on-call. This is the reason
  Serializable exists as a distinct level even where phantoms are prevented,
  and it's the thing to mention.
- PostgreSQL's Serializable is **SSI** (serializable snapshot isolation):
  optimistic, so it doesn't block, but it can abort your transaction at
  commit with a serialization failure — meaning **every application using it
  must have retry logic**. That practical consequence is what separates a
  read-about-it answer from a used-it answer.
- Defaults differ: PostgreSQL and Oracle default to Read Committed;
  MySQL/InnoDB defaults to Repeatable Read. Knowing your default matters more
  than knowing the table.

**Weak answers miss.** Write skew, and that Repeatable Read means different
things in different engines.

**Follow-ups to expect.**
- How would you fix write skew without going to Serializable? (`SELECT ...
  FOR UPDATE` to take explicit locks on the rows you read, or materialise
  the conflict into a row you can lock, or a unique constraint that makes
  the invariant a database-level fact.)
- What's the cost of Serializable? (In SSI: tracking overhead and abort
  rate under contention. In lock-based implementations: reduced concurrency.
  Measure the abort rate before adopting it broadly.)

---

### A2. MVCC and long transactions

**Answer.**
**MVCC** gives each transaction a consistent snapshot by keeping multiple
versions of each row rather than blocking readers. The core property:
**readers don't block writers and writers don't block readers**.

PostgreSQL's implementation: an `UPDATE` writes a **new row version** (a new
tuple) and marks the old one as deleted at the current transaction id. Every
tuple carries `xmin`/`xmax`; a transaction's snapshot determines which
versions it can see. Nothing is overwritten in place.

The consequence is **dead tuples**: old versions that no transaction can
still see. `VACUUM` reclaims them, and autovacuum runs it automatically.

**What a long-running transaction does**: it holds a snapshot, and vacuum
cannot remove any tuple version that might still be visible to *any* open
transaction. So a single idle-in-transaction session — a connection that ran
a `BEGIN` and then went to lunch, or an analytics query running for hours —
**blocks cleanup across the entire database**. Dead tuples accumulate:

- **Table and index bloat.** Tables physically grow, sequential scans get
  slower, indexes get bigger, and the buffer cache holds dead data. Reclaiming
  it later needs `VACUUM FULL` (takes an `ACCESS EXCLUSIVE` lock — an outage)
  or `pg_repack` (online, but needs disk space and time).
- **Query plans degrade** as statistics and physical layout drift.
- **Transaction ID wraparound risk.** Postgres transaction ids are 32-bit
  and wrap. Vacuum is also responsible for freezing old tuples; if it can't
  keep up, the database approaches wraparound and will eventually **refuse
  writes** to protect itself. This is a genuine, well-documented outage
  mode at large scale, and the emergency fix (a forced vacuum) can take
  hours on a big table.

Note MySQL/InnoDB does it differently — old versions live in the **undo
log** rather than in the table — so the symptom is undo log growth and
history-list length rather than table bloat, but the same root cause: a long
transaction pins history.

**What to do**: alert on `idle in transaction` duration and on the age of
the oldest transaction, set `idle_in_transaction_session_timeout`, monitor
`n_dead_tup` and `pg_stat_progress_vacuum`, tune autovacuum to be more
aggressive on hot tables (the defaults are conservative for large tables),
and keep analytical workloads on a replica with `hot_standby_feedback`
consciously chosen — because turning it on makes the *replica's* long
queries block vacuum on the *primary*, which surprises people.

**Weak answers miss.** Transaction ID wraparound and that a long transaction
on a replica can block vacuum on the primary.

**Follow-ups to expect.**
- Why is `VACUUM FULL` dangerous? (Exclusive lock, rewrites the whole table,
  needs 2x disk. Use `pg_repack`.)
- How does this interact with connection poolers? (A pooler in session mode
  can leave transactions open across application idle time. Transaction-mode
  pooling with a timeout is safer — A8.)

---

### A3. Composite index `(a, b, c)`

**Answer.**
A B-tree composite index is sorted by `a`, then `b` within equal `a`, then
`c`. That single sentence answers every case:

- **Filter on `a`** — ✅ Uses the index. `a` is the leading column, so the
  matching entries are contiguous.
- **Filter on `b` only** — ❌ Not usefully. Entries with a given `b` are
  scattered across every `a` value. PostgreSQL *can* do a full index scan if
  the index is much narrower than the table, but it's not a seek and it
  doesn't scale. This is the **leftmost prefix rule**.
- **Filter on `a` and `c`** — ⚠️ Partially. It seeks on `a`, then must scan
  all entries under that `a` and filter `c` as a residual predicate. Useful
  if `a` is selective, wasteful if not. In Postgres you'd see the `c`
  condition as a `Filter` rather than an `Index Cond`, which is exactly how
  you diagnose this in `EXPLAIN`.
- **Range on `a`, then equality on `b`** — ⚠️ Only `a` is used for the seek.
  Once you have a *range* on a column, everything after it in the index is no
  longer sorted usefully within the result. This is the more subtle half of
  the rule and the one people miss: **equality columns first, range column
  last**. An index on `(status, created_at)` works for
  `status = 'x' AND created_at > y`; an index on `(created_at, status)` does
  not.

The general statement: an index supports a seek on a leading prefix of
equality predicates plus at most one range predicate on the next column.
Everything after that is filtering, not seeking.

Also worth adding: column order affects **sorting** too. `ORDER BY a, b`
can be satisfied by the index without a sort step; `ORDER BY b, a` cannot.
And the index can serve `ORDER BY a DESC, b DESC` by scanning backwards,
but not `ORDER BY a ASC, b DESC` unless you declare the index that way.

**Weak answers miss.** The range-column rule. Reciting "leftmost prefix"
without it means you'll design `(created_at, tenant_id)` and wonder why it's
slow.

**Follow-ups to expect.**
- How would you verify? (`EXPLAIN (ANALYZE, BUFFERS)` — look at `Index Cond`
  vs `Filter`, and at `Rows Removed by Filter`, which quantifies exactly how
  much of the index scan was wasted.)
- Would a hash index or a BRIN index help anywhere? (Hash: equality only, no
  ordering, rarely worth it. BRIN: tiny index for naturally-ordered large
  tables — time-series appended in timestamp order — where it's dramatically
  smaller than a B-tree for range queries.)

---

### A4. Covering index and index-only scan

**Answer.**
A **covering index** contains all the columns a query needs, so the engine
can answer entirely from the index without visiting the table heap. That
turns two I/Os per row (index lookup + heap fetch, the latter effectively
random) into one sequential-ish index scan. On a large table this is
frequently a 10x+ improvement, and it's the highest-leverage indexing move
after having the right leading columns.

In PostgreSQL this is an **index-only scan**, and there's a wrinkle worth
knowing: the index doesn't store visibility information, so Postgres must
check the **visibility map** to confirm the page is all-visible. If the
table has lots of recent writes and the visibility map isn't current
(i.e. vacuum hasn't run), the "index-only" scan falls back to heap fetches
— you'll see `Heap Fetches: <n>` in `EXPLAIN ANALYZE`. So index-only scans
depend on vacuum health, which ties back to A2.

`INCLUDE` columns (Postgres 11+, and SQL Server's equivalent) let you add
payload columns to the index leaf without making them part of the key —
smaller index, no effect on ordering or uniqueness, still covering.

The tradeoff: every added column makes the index larger, which means more
memory, slower writes (every insert/update maintains it), and more WAL.
Covering indexes are a targeted optimisation for a specific hot query, not a
default.

**Weak answers miss.** The visibility map dependency, which is Postgres-
specific and exactly the kind of thing an interviewer uses to check depth.

**Follow-ups to expect.**
- Why doesn't InnoDB have this problem? (Its secondary indexes store the
  primary key and it uses undo-based MVCC — different tradeoffs; a secondary
  index lookup costs a PK lookup unless covering.)
- When is a partial index better? (`WHERE status = 'pending'` on a table
  where 99% of rows are 'done' — a tiny index over exactly the rows you
  query. Often better than a covering index and much cheaper to maintain.)

---

### A5. Write-ahead logging

**Answer.**
Before any change is applied to the data pages, a record describing the
change is written to a sequential log and **fsynced**. Only then may the
change be made in the buffer pool; the dirty data pages themselves are
written back lazily by a checkpointer.

What it buys:

1. **Durability without random writes.** A committed transaction may touch
   pages scattered across the disk. Flushing those synchronously would mean
   many random writes per commit. Instead you do **one sequential write and
   one fsync** to the log. Sequential I/O plus a single flush is orders of
   magnitude cheaper than random I/O, and this is what makes commit latency
   acceptable at all.
2. **Crash recovery.** After a crash, replay the log from the last
   checkpoint: redo committed transactions whose pages hadn't been written,
   undo uncommitted ones. Recovery time is bounded by checkpoint frequency,
   which is a direct tunable tradeoff — frequent checkpoints mean fast
   recovery and more steady-state write I/O.
3. **Atomicity.** The commit record is the single point at which a
   transaction becomes durable. Everything before it can be rolled back.
   It also solves the torn-page problem (Postgres's `full_page_writes`
   writes whole pages to WAL after a checkpoint, because a partial 8 KB page
   write during a crash would be unrecoverable otherwise).
4. **Replication, for free.** The log is an ordered stream of every change,
   so shipping it to another node reproduces the database exactly. Postgres
   streaming replication, MySQL binlog, and by extension **CDC** all fall out
   of this. That's the point most worth making: the WAL is the reason change
   data capture is possible at all.
5. **Point-in-time recovery**: base backup + WAL replay to an arbitrary
   moment.

The cost: **write amplification** — every change is written twice (log and
data pages), plus full-page images. And the log is on the critical path for
commit latency, so the storage it lives on determines your commit throughput.
This is why fast, low-latency devices for WAL matter disproportionately, and
why **group commit** (batching many transactions into one fsync) is a
universal optimisation (topic 05, A13).

**Weak answers miss.** That replication and CDC are downstream consequences
of WAL, and the sequential-vs-random argument that motivates it.

**Follow-ups to expect.**
- What happens if WAL can't be archived or a replication slot stops
  consuming? (WAL accumulates and fills the disk — a genuine and common
  Postgres outage. An inactive replication slot will retain WAL forever;
  `max_slot_wal_keep_size` bounds it at the cost of breaking the replica.)
- What is `synchronous_commit = off`? (Commit returns before the WAL fsync;
  you keep atomicity and crash consistency but can lose the last few
  hundred milliseconds of committed transactions. A legitimate,
  well-defined RPO tradeoff — unlike `fsync = off`, which risks corruption.)

---

### A6. Replication lag

**Answer.**
The delay between a change committing on the primary and being visible on a
replica. Sources: network transfer, replica apply speed (often
single-threaded in older setups, so a write burst on a multi-core primary
outpaces the replica), replica CPU/IO saturation from serving reads, long
queries on the replica blocking apply (Postgres: apply can conflict with
running queries), and lock waits.

How a user notices — this is the useful part:

1. **Read-your-writes violation.** User posts a comment, the write goes to
   the primary, the subsequent read goes to a replica, and the comment isn't
   there. The single most common user-visible symptom, and it's a
   *correctness* bug from the user's perspective even though the system is
   working as designed.
2. **Monotonic-read violation.** Two consecutive reads hit different
   replicas with different lag, so data appears to move backwards — an item
   count goes 5, 6, 5. Worse than staleness because it looks like corruption.
3. **Causal violation across services.** Service A writes and calls service
   B; B reads from a replica and can't find the row A just created. Presents
   as a mysterious "record not found" race, usually in an async job that
   processes a message faster than replication.
4. Stale dashboards/reports, and — importantly — **a failover that loses
   data** equal to the lag if replication is async (A9).

Mitigations, in increasing order of cost: route reads that follow a write to
the primary for a window (sticky-to-primary after write, keyed on session);
pass a **write LSN/token** with the request and have the replica wait until
it has applied at least that position (the correct solution, and what
"read-your-writes" support in modern drivers does); pin a session to one
replica for monotonic reads; or use synchronous replication for the reads
that need it.

And monitor it properly: measure lag in **seconds behind** and in **bytes**,
alert on both, and — the part people miss — measure it from the *replica's*
perspective with a heartbeat row written on the primary, because
`pg_stat_replication`'s byte lag reads 0 on an idle primary even if apply is
broken.

**Weak answers miss.** Naming read-your-writes and monotonic reads as
consistency models rather than "the data is stale," and the LSN-token
solution.

**Follow-ups to expect.**
- How much lag is acceptable? (It's an SLO decision tied to which reads you
  route where. Have a number and route by requirement, not by convenience.)

---

## Tier 2 — Explain / compare

### A7. B-tree vs LSM-tree

**Answer.**
**B-tree** (Postgres, InnoDB, most OLTP engines). A balanced tree of pages,
updated **in place**. A write locates the leaf page and modifies it.
- **Read amplification**: low and predictable — O(log n) page reads, and the
  upper levels are cached, so a point lookup is typically 1 I/O.
- **Write amplification**: a single-row update dirties a whole page (8–16 KB)
  which must eventually be written; plus WAL, plus full-page images. So
  writing 100 bytes can cost tens of KB of I/O. Random writes, which is the
  expensive pattern for any storage device.
- **Space amplification**: pages are typically not full (fill factor,
  splits), so ~1.3x is normal. Fragmentation grows over time.
- Strengths: excellent read latency, cheap range scans in key order,
  straightforward transactions and secondary indexes.

**LSM-tree** (RocksDB, Cassandra, ScyllaDB, HBase, and the family
ClickHouse's MergeTree belongs to). Writes go to an in-memory **memtable**
(plus a WAL for durability); when full, it's flushed as an immutable,
sorted **SSTable**. Background **compaction** merges SSTables into larger
ones, discarding overwritten and deleted entries.
- **Write amplification**: the write path is *sequential* and the initial
  write is cheap — but each record is rewritten every time it participates
  in a compaction, so total amplification over its lifetime is roughly the
  number of levels times the fanout factor. In leveled compaction that's
  commonly 10–30x; size-tiered is lower write amplification but higher space
  and read amplification. **This is the central tuning tradeoff of an LSM
  and you should name it as such** (the RUM conjecture: read, update, memory
  — pick two).
- **Read amplification**: a point lookup may have to check the memtable and
  one file per level. Bloom filters make negative lookups cheap, so the real
  cost lands on **range scans**, which must merge across levels and can't
  skip via a filter.
- **Space amplification**: obsolete versions live until compacted.
- Strengths: very high write throughput, excellent compression (sorted
  immutable blocks compress far better than half-full B-tree pages), and
  cheap deletes-as-tombstones.

**Picking:**
- Write-heavy, high ingest, compression matters, reads are mostly by key or
  recent-range: **LSM**. Time series, event stores, metrics, key-value at
  scale.
- Read-heavy, latency-sensitive point and range reads, complex transactions
  and many secondary indexes: **B-tree**.

**The operational costs people forget**, and what an interviewer is fishing
for: an LSM's compaction is background I/O and CPU that competes with
foreground traffic, so **p99 latency is spiky and correlates with compaction
schedules**. Tombstones make deletes cheap to write and expensive to read
until compacted — a Cassandra table with heavy deletes and a range query is
a well-known pathology. And "the write was fast" hides the fact that you
deferred the work, so you must have the sustained I/O budget for compaction
or you fall permanently behind (compaction backlog is a real outage mode).

**Weak answers miss.** That LSM write amplification over a record's lifetime
can *exceed* a B-tree's, and the compaction-driven p99 spikes. "LSM is
better for writes" without those caveats is the shallow answer.

**Follow-ups to expect.**
- What does a Bloom filter give you and what's the cost? (Probabilistic set
  membership: no false negatives, tunable false positives. ~10 bits/key for
  ~1% FPR. Turns "check every level" into "check the levels that might have
  it." Doesn't help range scans.)
- Leveled vs size-tiered compaction? (Leveled: lower read and space
  amplification, higher write amplification — good for read-heavy. Tiered:
  the reverse — good for write-heavy. Being able to state the direction of
  each is the answer.)

---

### A8. Connection pooling in PostgreSQL

**Answer.**
**Why Postgres is sensitive**: it uses a **process per connection**. Each
backend is a full OS process with its own memory (work_mem allocations,
catalog caches, prepared statement state) — commonly several MB resident
each. So 1000 connections is 1000 processes. The costs:
- Memory: gigabytes just to exist.
- **Context switching and scheduler pressure** with hundreds of runnable
  backends.
- Contention on shared structures — lock manager, buffer mapping — which
  grows superlinearly. Throughput **decreases** past a point; the classic
  shape is a throughput curve that peaks around a small multiple of core
  count and then declines.
- Connection establishment is expensive (fork + setup), so short-lived
  connections are especially bad.

MySQL uses a **thread per connection** with a much smaller per-connection
footprint, and has had thread pool implementations, so it tolerates higher
connection counts — though it isn't immune, and the underlying truth is the
same for both: **useful concurrency is bounded by cores and disk, not by
connection count.** Little's law again — beyond the saturation point, more
connections only add queueing.

**PgBouncer pooling modes:**

- **Session pooling** — a client gets a server connection for the whole
  duration of its client connection. Safe (all features work) but gives you
  almost no multiplexing, so it only helps with connection *churn*, not
  connection *count*.
- **Transaction pooling** — a server connection is assigned per transaction
  and returned to the pool at commit. This is the mode that matters: a few
  dozen server connections can serve thousands of clients, because most
  clients are idle most of the time. It's what everyone actually deploys.
- **Statement pooling** — per statement; forbids multi-statement
  transactions. Rare.

**What transaction pooling breaks** — the part that must be in the answer:
anything with *session state*, because your client may get a different
backend for the next transaction.
- Session-level `SET` (e.g. `SET search_path`, `SET timezone`), unless
  wrapped in the transaction or handled by `server_reset_query`.
- `LISTEN`/`NOTIFY`.
- Advisory locks held across transactions.
- `WITH HOLD` cursors.
- **Server-side prepared statements** — a long-standing pain point;
  PgBouncer added support in recent versions, but many drivers still need
  configuring (e.g. disabling prepared statement caching or using a protocol
  mode that avoids it). Check this specifically when introducing PgBouncer;
  it's the most common breakage.
- Temp tables.

Also worth knowing: **connections-per-app-instance multiplies with
autoscaling** (topic 03, A16). 100 pods × a 20-connection pool = 2000
connections. A pooler in front is the only thing that decouples app scaling
from database connection count, which is a *scalability architecture*
decision, not a tuning detail.

**Weak answers miss.** The list of what transaction pooling breaks, and the
"throughput declines past saturation" shape — most people assert more
connections is merely wasteful rather than actively harmful.

**Follow-ups to expect.**
- How do you size the pool? (Start from cores and disk parallelism. A common
  rule of thumb is a small multiple of core count — measure the throughput
  curve rather than trusting a formula. Then size the *client-side* pools so
  their sum doesn't exceed the pooler's server pool.)
- PgBouncer is single-threaded — implication? (It can become a CPU
  bottleneck; run multiple instances with `SO_REUSEPORT` or shard by
  database. Also it's a new SPOF on the critical path, so it needs its own
  HA story. PgCat and Odyssey are multithreaded alternatives.)

---

### A9. Synchronous vs asynchronous replication

**Answer.**
**Asynchronous**: the primary commits and acknowledges the client
immediately; WAL is shipped to replicas in the background.
- Commit latency: unaffected by replica or network.
- **On failover you lose everything the replica hadn't received** — i.e.
  **RPO > 0**, equal to the lag at the moment of failure. Under load, lag is
  often larger precisely when you're most likely to fail over.
- Availability: a slow or dead replica never affects the primary.

**Synchronous**: the primary waits for at least one replica to confirm before
acknowledging the commit.
- **RPO = 0** for the confirmed replicas — no acknowledged transaction is
  lost.
- Commit latency now includes a network round trip to the replica. Same AZ:
  sub-millisecond, acceptable. **Cross-region: 70–90 ms US-East↔Europe as an
  order-of-magnitude figure — which means every write costs that, and a
  transaction doing 10 sequential writes costs a second.** This is the number
  that kills naive "just replicate synchronously to another region" designs,
  and you should say it out loud.
- Availability: if the sync replica is down and you require confirmation,
  **writes stop**. This is the crucial part — naive synchronous replication
  *reduces* availability, because you've added a component whose failure
  blocks the primary.

The nuance that separates a good answer: **what does "confirmed" mean?**
Postgres `synchronous_commit` has levels — `remote_write` (the replica's OS
has it), `on`/`remote_flush` (the replica has fsynced it), `remote_apply`
(the replica has applied it, so it's visible to reads there). Each is
progressively safer and slower, and `remote_apply` is the only one that
gives you read-your-writes on the replica.

**The configuration that's usually right**: synchronous replication to a
replica in **another AZ in the same region** (RPO 0, latency cost
sub-millisecond, survives an AZ failure), plus **asynchronous** replication
to another region (bounded RPO for regional disaster, no latency cost). And
critically, **more than one candidate sync replica** with quorum-style
configuration (`ANY 1 (r1, r2)` in Postgres, or semi-sync with multiple
candidates in MySQL) so that losing one replica doesn't halt writes.
Alternatively a timeout that degrades to async — but be explicit that this
silently converts your RPO 0 guarantee to RPO > 0 exactly when you're having
an incident, which many teams discover only afterwards.

**Weak answers miss.** That sync replication reduces availability unless you
have redundant sync candidates, and the cross-region latency arithmetic.

**Follow-ups to expect.**
- MySQL semi-synchronous: what does it actually guarantee? (The replica has
  received and logged the event, not necessarily applied it. And there's a
  documented window — `rpl_semi_sync_master_wait_point` — where behaviour
  differs between `AFTER_SYNC` and `AFTER_COMMIT`; the latter can expose a
  transaction to readers on the primary before the replica has it.)
- How does this relate to consensus replication? (Raft-based systems replicate
  to a majority quorum, giving RPO 0 with tolerance for a minority failing —
  you get both properties instead of choosing. That's the argument for
  Patroni-with-etcd style setups and for Spanner/CockroachDB-class systems.)

---

### A10. Split brain and fencing

**Answer.**
**Split brain**: two nodes both believe they are the primary and both accept
writes. Once that happens you have two divergent histories, and there is no
automatic way to merge them — you will lose data or need manual
reconciliation. It is the worst outcome in database operations, strictly
worse than being down, because being down is recoverable.

How it happens: the failover mechanism can't distinguish "the primary is
dead" from "I can't reach the primary." A network partition isolates the
primary from the monitoring system while it's still up and still serving
clients on its side of the partition. The monitor promotes a replica.
Now there are two.

**Fencing**: guaranteeing the old primary cannot accept writes before the
new one starts. Mechanisms:

- **STONITH** ("shoot the other node in the head") — forcibly power off or
  reset the old primary out of band: IPMI/BMC, hypervisor API, cloud API to
  stop the instance. The only mechanism that works when the old primary is
  unreachable *and* uncooperative.
- **Network fencing** — revoke its network access: remove it from the load
  balancer's target group, revoke a security group rule, change routing. In
  cloud environments this is often more practical than STONITH.
- **Storage fencing** — SCSI reservations, or detaching the network volume.
- **Fencing tokens** — every write carries a monotonically increasing epoch
  number issued at promotion; the storage layer rejects writes with a stale
  token. This is the software-level solution and it's strictly the most
  robust, because it doesn't depend on reaching the old node at all — it
  only requires the *storage* to enforce the ordering. Same idea as a lease
  with a version number in distributed systems generally.
- **Quorum**: require a majority to agree before promoting, so a minority
  partition can never elect a leader. This prevents *electing* a second
  primary; it does not by itself stop the *old* primary from continuing to
  serve, which is why you still need the old primary to self-demote on
  losing quorum. Patroni does this: it holds a leader key in etcd/Consul
  with a TTL, and if it can't renew, it demotes itself.

**Why automated failover without fencing is dangerous**: you've automated
the creation of split brain. A brief network blip now triggers a promotion,
and the old primary, still healthy, keeps serving whichever clients can
reach it. The classic manifestation is a DNS or VIP-based failover where
some clients have the old address cached (topic 02, A12) — those clients
write to the old primary for minutes.

The honest framing: automated failover trades **RTO for the risk of split
brain**, and the exchange is only worth it if fencing is genuinely reliable.
Plenty of serious operations teams run manual or semi-automatic failover for
their most critical database precisely for this reason, and saying that is
a mature answer rather than a timid one.

**Weak answers miss.** Fencing tokens, and the point that quorum prevents
electing a second primary but doesn't stop the first one from serving.

**Follow-ups to expect.**
- How does Patroni prevent it? (Leader key with TTL in a DCS; the leader
  must renew, and demotes itself if it can't. Plus optional watchdog —
  hardware or `softdog` — that reboots the node if the process stops
  renewing, which is self-STONITH and is the belt-and-braces answer.)
- What if the fencing action itself fails? (Then you must not promote. A
  failover that can't fence must block and page a human. Systems that
  promote anyway are choosing availability over data integrity — state it
  as an explicit decision, not an accident.)

---

### A11. Choosing a shard key

**Answer.**
A shard key must satisfy four things, and they're frequently in conflict:

1. **High cardinality** — enough distinct values to spread across shards and
   to keep spreading as you add them.
2. **Even distribution of both data and *traffic*.** These are different:
   `user_id` distributes rows evenly and traffic unevenly if some users are
   1000x more active.
3. **Present in the majority of queries.** If your dominant query doesn't
   include the shard key, every query becomes a scatter-gather across all
   shards, and your p99 becomes the max over N shards (topic 05, A16). You
   have built a slower database with more operational cost.
4. **Keeps related data together**, so transactions and joins stay within
   one shard. Cross-shard transactions need 2PC or sagas and are the thing
   you're trying to avoid.

**A bad shard key: monotonically increasing timestamp (or auto-increment
id) with range partitioning.** All new writes go to the newest range, which
means one shard takes 100% of the write load while the rest sit idle — a
**hot spot** that no amount of adding shards fixes, because the problem is
that the key ordering concentrates recent activity. You've bought N shards
and are using one. (Hash partitioning on the same key fixes the write
distribution but destroys efficient time-range queries, which is usually
why you chose time in the first place. That tension is the real content of
the question.)

**Another bad one: a low-cardinality field like `country` or `status`.**
Cardinality caps your shard count, and the distribution follows real-world
skew — a "US" shard with 60% of the data that you can never split.

**Another: `tenant_id` in a multi-tenant system with a power-law tenant size
distribution.** Works beautifully until one tenant outgrows a single shard,
at which point you have an unsplittable partition. This is the most common
real-world version, and the mitigation is a **composite key**
(`tenant_id` + a bucket, or `tenant_id` + entity id) so a large tenant can
span shards, plus the ability to give the largest tenants dedicated shards.

**Good patterns:** hash of a natural entity id for even distribution;
composite keys that put the tenant first (for locality) and a
high-cardinality component second (for splittability); and
**time-bucketed + hashed** keys for time series — e.g. `(hash(series_id),
time_bucket)` — so writes spread but a query for one series over a range is
still local.

**And the meta-point**: the expensive part isn't picking the key, it's
**changing it**. Resharding a live system means rewriting and re-routing
everything. So design for it up front: use more logical shards than physical
ones (e.g. 4096 virtual shards mapped onto 16 machines) so growing means
moving whole virtual shards rather than rehashing, and keep the mapping in a
lookup table you can change rather than a hash function you can't.

**Weak answers miss.** Traffic distribution vs data distribution, and the
virtual-shards trick that makes resharding tractable.

**Follow-ups to expect.**
- How do you handle the one tenant that's 100x everyone else? (Dedicated
  shard, or split its keyspace with a composite key. Say that you also need
  per-tenant rate limiting or it will hurt everyone regardless of placement.)
- How would you actually reshard live? (Same playbook as A16: dual-write or
  CDC to the new layout, backfill, verify, flip reads, flip writes, keep
  rollback. Resharding is a migration.)

---

### A12. DynamoDB vs Cassandra vs Spanner

**Answer.**
**DynamoDB** — a managed, partitioned key-value/document store.
- Reads: **eventually consistent by default**; you can request a
  *strongly consistent* read per operation at 2x the read cost, which reads
  from the leader replica within the region. Global tables (multi-region) are
  **not** strongly consistent — they're async multi-master with
  **last-writer-wins** conflict resolution, so concurrent writes in two
  regions silently discard one.
- Writes: single-item writes are atomic and conditional writes
  (`ConditionExpression`) give you compare-and-set, which is the primitive
  for optimistic concurrency and idempotency.
- Transactions: `TransactWriteItems`/`TransactGetItems` provide
  ACID across multiple items **within one region and one account**, with a
  documented item limit (100 items at time of writing — verify) and no
  interactive transactions. Serializable in effect, but you cannot read,
  think, and then write inside a transaction.
- The operational model to know: partition-level throughput limits mean a
  hot partition throttles even when the table has capacity — the same shard
  key problem as A11, surfaced as `ProvisionedThroughputExceeded`.

**Cassandra** — leaderless, Dynamo-style replication with tunable quorums.
- Consistency is **per query**: you choose R and W (ONE, QUORUM, LOCAL_QUORUM,
  ALL). `R + W > N` gives you read-your-writes/overlap of replica sets —
  but note this is **not linearizability**: without a consensus protocol
  there's no total order, concurrent writes can be reordered, and a failed
  write may or may not be visible (there's no rollback). Say this explicitly;
  "R+W>N means strong consistency" is a very common and incorrect claim.
- Conflict resolution is **last-writer-wins by cell timestamp**, so clock
  skew directly causes silent data loss, and there's no read-modify-write
  safety.
- Lightweight transactions (`IF NOT EXISTS`) use Paxos for real linearizable
  compare-and-set — correct, but several times more expensive, and mixing
  LWT and non-LWT writes to the same partition breaks the guarantee.
- No multi-partition transactions. (Newer Cassandra work on Accord aims at
  this — verify current state.)
- Strength: multi-datacentre with `LOCAL_QUORUM` gives low-latency local
  operations and async cross-DC, which is genuinely hard to beat for
  write-heavy geo-distributed workloads.

**Spanner** — externally-consistent distributed SQL.
- Provides **external consistency** (linearizability for transactions, the
  strongest practical guarantee): if T1 commits before T2 starts, every
  observer sees T1 before T2. Achieved with Paxos groups per split, 2PC
  across splits, and **TrueTime** — a bounded-uncertainty clock backed by
  GPS and atomic clocks. Commit waits out the uncertainty interval, which is
  why Spanner writes have a latency floor tied to clock uncertainty
  (single-digit milliseconds order) and why the whole design depends on
  Google's clock infrastructure.
- Gives you real SQL, real multi-row multi-table transactions across
  regions, and strongly consistent secondary indexes.
- Costs: money, write latency (especially multi-region — you're paying
  cross-region consensus per commit), and the need to design schemas around
  splits and interleaving to avoid hotspots. A monotonically increasing
  primary key is as bad here as anywhere (A11).
- CockroachDB and YugabyteDB are the open equivalents; they use HLCs with a
  configured maximum clock offset instead of TrueTime, which shifts the
  assumption from "hardware guarantees the bound" to "NTP had better hold,
  and if it doesn't we may violate the guarantee."

**The framing**: DynamoDB and Cassandra give you availability and
partition-tolerance with a consistency model you must reason about at every
call site; Spanner gives you a consistency model you can stop thinking about,
paid for in latency and cost. Choose based on whether your correctness
depends on cross-entity invariants.

**Weak answers miss.** That R+W>N is not linearizability, and that DynamoDB
global tables are LWW. Both are the specific misconceptions this question
exists to expose.

**Follow-ups to expect.**
- What is PACELC? (Else-latency: even when there's no Partition, you trade
  Latency against Consistency. Spanner is PC/EC — consistent in both cases,
  paying latency. Cassandra with LOCAL_QUORUM is PA/EL. It's a strictly more
  useful framing than CAP because the "else" case is what you live in 99.9%
  of the time.)
- When would you choose eventual consistency deliberately? (When the
  business operation is commutative or has a compensating action — inventory
  with oversell tolerance, view counts, feeds. Name the compensation, not
  just the tolerance.)

---

### A13. Cache patterns and stampede

**Answer.**
**Cache-aside (lazy loading)**: application checks the cache, and on a miss
reads the database, populates the cache, and returns. The default and the
right choice most of the time. Only requested data is cached; cache and
database can't be atomically updated, so there's always a window where they
disagree. On write, you **invalidate** rather than update, because updating
creates a race (two concurrent writers can leave the cache holding the older
value).

**Write-through**: writes go to the cache, which synchronously writes to the
database. Cache is never stale, but every write pays both costs, and you
cache data that may never be read. Usually paired with cache-aside reads.

**Write-behind (write-back)**: writes go to the cache and are flushed to the
database asynchronously. Fast writes and write coalescing (many updates to
one key become one database write), but you can **lose acknowledged writes**
if the cache dies, and it's the only pattern where the cache is authoritative
— which makes it a durability component, not a cache. Legitimate for
counters and metrics; dangerous for anything you'd be sad to lose.

**Cache stampede** (dogpile / thundering herd): a hot key expires, and every
concurrent request misses simultaneously and hits the database at once. With
1000 req/s on one key, expiry means 1000 concurrent identical database
queries. The database slows, requests pile up, and you can lose the database
to a *cache expiry* — with the cache still perfectly healthy.

Three (four) preventions:

1. **Request coalescing / single-flight.** Only one request per key is
   allowed to recompute; the others wait for its result. In-process
   (Go's `singleflight`, a per-key mutex) handles the per-instance case; a
   distributed lock in the cache (`SET key NX EX`) handles it across
   instances. This is the most direct fix and it bounds the load to one
   query per key regardless of traffic.
2. **Probabilistic early expiration.** Store the value with its computation
   time and TTL; on read, recompute early with a probability that rises as
   the expiry approaches (the XFetch approach). One request refreshes the
   value *before* it expires, so no one ever sees a miss. Elegant, no locks,
   no coordination.
3. **TTL jitter.** Never use a fixed TTL for a set of keys populated at the
   same time — a cache warmed at deploy with a uniform 1-hour TTL produces a
   synchronised mass expiry an hour later. Randomise: `ttl * (1 + rand(0,
   0.1))`. This addresses the *correlated* stampede across many keys, which
   is the version that takes down databases.
4. **Serve stale while revalidating.** Keep the old value past its logical
   expiry, return it immediately, and refresh in the background — the
   `stale-while-revalidate` idea from topic 02, A13. Users never wait, and
   the load is exactly one refresh.

Also worth naming: **negative caching** (cache the "not found" too, or a
lookup for a nonexistent key hits the database every time — the basis of
cache-penetration attacks; a Bloom filter over existing keys is the
scalable defence), and the fact that **a cache you can't survive losing is
not a cache, it's an undocumented database**. Test cold-start: if your
system can't come up with an empty cache, that's an outage waiting for a
Redis failover.

**Weak answers miss.** TTL jitter (the correlated case) and the cold-start
survivability point.

**Follow-ups to expect.**
- Invalidate or update on write? (Invalidate. Updating races; invalidating
  is idempotent and the next reader repopulates. The exception is when the
  recompute is so expensive that a miss is worse than the race.)
- How do you invalidate across regions? (You mostly can't do it atomically.
  Options: short TTLs, a pub/sub invalidation bus with best-effort delivery,
  or versioned keys so a version bump invalidates implicitly. Versioned keys
  are the most robust and cost you memory.)

---

### A14. ClickHouse: why it's fast, and what it's bad at

**Answer.**
**Why it's fast** — several independent things compounding:

1. **Columnar storage.** A query touching 3 of 200 columns reads only those
   3. For wide analytical tables that alone is a 50x I/O reduction.
2. **Compression that actually works.** Values in a column are homogeneous,
   so specialised codecs apply: `Delta`/`DoubleDelta` for timestamps and
   counters, `Gorilla` for floats, `T64`, then LZ4 (fast) or ZSTD (smaller).
   10x compression on real telemetry is routine, which multiplies effective
   disk throughput and cache capacity.
3. **Vectorised execution.** Processes columns in blocks (~65k rows) rather
   than row-at-a-time, which keeps data in cache, amortises virtual call
   overhead, and lets the compiler use SIMD. This is the single biggest CPU
   win.
4. **Sparse primary index + data skipping.** MergeTree sorts data by the
   `ORDER BY` key and keeps a sparse index (one mark per ~8192 rows), plus
   optional skip indexes (minmax, set, bloom filter). Queries with a
   predicate on the sort key read a small fraction of granules. **Partition
   pruning** on the `PARTITION BY` expression (usually a month or day)
   eliminates whole directories.
5. **Parallelism everywhere** — across cores, across parts, across shards —
   and a query engine willing to use every core on the box.

**What it's genuinely bad at:**

1. **Point updates and deletes.** Data is stored in immutable parts.
   `ALTER TABLE ... UPDATE/DELETE` is a **mutation**: it rewrites every
   affected part in the background. It is asynchronous, expensive, and not
   transactional. On a large table a single-row delete can rewrite gigabytes.
   Lightweight deletes improve the ergonomics (marking rows and filtering
   them at read time) but the physical cleanup still happens via merges, so
   the cost doesn't disappear. **Design so you never update**: append-only,
   and use `ReplacingMergeTree`/`AggregatingMergeTree` to collapse versions
   during merges — with the critical caveat that collapsing happens
   **eventually and non-deterministically**, so a query must use `FINAL`
   (expensive) or aggregate defensively (e.g. `argMax`) to get correct
   results. Teams get burned by assuming ReplacingMergeTree deduplicates
   immediately. It does not.
2. **High-concurrency small queries / OLTP.** Each query is designed to use
   many cores, so hundreds of concurrent queries thrash. There's no MVCC for
   readers in the OLTP sense, no real transactions across statements, and no
   enforced foreign keys or unique constraints. Point lookups by a
   non-sort-key column are slow. It is an analytical engine; putting a
   user-facing per-request query on it needs care and usually a strict
   concurrency limit plus caching.
3. **Joins**, especially large-to-large. The default hash join builds the
   right table in memory on the initiating node; a join between two large
   distributed tables either broadcasts or fails on memory. The idiomatic
   answer is denormalisation and dictionaries (`dictGet`) for lookup data —
   which is a real design constraint, not a tuning issue. (The join engine
   has improved substantially in recent versions, so verify current
   behaviour, but the design bias toward wide denormalised tables remains.)

Two more operational realities worth having ready, given your background:
**too many small parts** is the classic failure — inserting frequently in
small batches creates parts faster than merges can consolidate them,
producing `Too many parts` errors and degraded queries. The fix is batching
(large inserts, seconds apart, not per-row) or async inserts. And
**ZooKeeper/ClickHouse Keeper** is the coordination dependency for replicated
tables; its health is your cluster's health, and it's a common source of
incidents at scale.

**Weak answers miss.** The mutation cost and the eventual/non-deterministic
nature of ReplacingMergeTree. Given ClickHouse is on your resume, an
interviewer will expect the "too many parts" story and a specific incident.

**Follow-ups to expect.**
- How do you handle deletes for GDPR? (Partition by something that lets you
  drop whole partitions, or use lightweight deletes and accept the merge
  cost, or `ALTER TABLE DELETE` batched and scheduled. State that "delete a
  user's rows on demand" is a design requirement that must shape the schema
  from day one.)
- ClickHouse vs a time-series database for metrics? (CH wins on
  cardinality tolerance, arbitrary-dimension queries, and compression; a
  purpose-built TSDB wins on ingestion ergonomics, downsampling/retention
  policy, and the ecosystem. Say what you'd pick for what.)

---

### A15. Adding a NOT NULL column with a default to a 500 GB table

**Answer.**
The first question is **which version of which engine**, because the correct
answer changed materially.

**PostgreSQL 11+**: `ADD COLUMN ... NOT NULL DEFAULT <constant>` is
**metadata-only**. The default is recorded in the catalog and materialised
lazily as rows are updated; the existing 500 GB isn't rewritten. It takes an
`ACCESS EXCLUSIVE` lock but holds it for milliseconds. So on modern
Postgres, with a *constant* default, this is a non-event — **and the correct
senior answer is to say so rather than describing a complex migration you
don't need.**

Two caveats that keep it dangerous:
- A **volatile** default (`now()`, `gen_random_uuid()`) still forces a full
  table rewrite. Only constants get the fast path.
- The `ACCESS EXCLUSIVE` lock is brief but it **queues behind existing
  transactions and blocks everything behind it**. A long-running query holds
  a lock; your DDL waits; every subsequent query queues behind your DDL. A
  "millisecond" migration becomes a full outage. **Always set
  `lock_timeout` (e.g. 2s) and retry**, so you fail fast instead of building
  a queue. This is the single most important operational detail and it
  applies to *every* DDL statement on a busy table.

**Pre-11 Postgres, or a volatile default, or MySQL without instant DDL** —
the safe pattern is expand/contract:

1. `ADD COLUMN` **nullable, no default**. Fast, metadata-only.
2. Add the default for *new* rows: `ALTER TABLE ... ALTER COLUMN ... SET
   DEFAULT`. Also metadata-only.
3. **Backfill in batches** — a few thousand rows per transaction, with a
   pause between batches, tracking progress by primary key range. Small
   transactions so you don't hold locks, don't blow up WAL, and don't create
   a long-running transaction that blocks vacuum (A2). Monitor replication
   lag and back off if it grows; a fast backfill is a classic way to push a
   replica hours behind.
4. Deploy application code that writes the column for new rows (this can
   come before or after the backfill; it must come before you rely on it).
5. Add the `NOT NULL` constraint **without a full-table validation lock**:
   in Postgres, `ADD CONSTRAINT ... CHECK (col IS NOT NULL) NOT VALID`
   (instant), then `VALIDATE CONSTRAINT` (takes only a `SHARE UPDATE
   EXCLUSIVE` lock, doesn't block reads/writes). Postgres 12+ can then
   convert that to a real `SET NOT NULL` cheaply because the check proves it.

**MySQL**: `ALGORITHM=INSTANT` handles adding a column with a default in
8.0+ for many cases. Otherwise `ALGORITHM=INPLACE` avoids a rebuild-with-
copy but still does work, and for anything that requires a copy the
production answer is an **external online schema change tool** —
`gh-ost` or `pt-online-schema-change`. They build a shadow table, copy in
throttled batches while capturing ongoing changes (gh-ost from the binlog,
pt-osc via triggers), then do an atomic rename. gh-ost's advantage is no
triggers on the production table and pausability/throttling driven by
replica lag.

**General rules I'd state regardless of engine:**
- Always `lock_timeout` + retry on DDL.
- Never combine a fast operation and a slow one in one statement or one
  transaction; the fast one inherits the slow one's lock duration.
- Backfill in bounded batches with monitoring of replication lag and dead
  tuples.
- Expand/contract as the general shape: add the new thing, dual-write,
  backfill, switch reads, then remove the old thing — with each step
  independently deployable and reversible.
- Test on a production-sized copy. A migration that's instant on a 1 GB
  staging table tells you nothing about 500 GB.

**Weak answers miss.** The `lock_timeout` point — describing a
"metadata-only, instant" migration while ignoring that lock queuing turns it
into an outage is the mistake that actually happens in production.

**Follow-ups to expect.**
- How do you drop a column safely? (Postgres `DROP COLUMN` is metadata-only
  and fast, but the application must stop referencing it first — including
  any `SELECT *` and any cached prepared statement plan. Contract step comes
  a full deploy cycle after expand.)
- How do you add an index without blocking? (`CREATE INDEX CONCURRENTLY` —
  two table scans, much slower, doesn't block writes; can leave an
  `INVALID` index if it fails, which you must drop and retry. MySQL:
  online DDL or gh-ost.)

---

## Tier 3 — Scenario / debug

### A16. Zero-downtime 4 TB PostgreSQL migration, AWS RDS → GCP Cloud SQL

**Answer.**
Open by scoping, because "zero downtime" is the ambiguous part and the
interviewer wants to see you interrogate it: does it mean zero *write*
downtime, or is a 30-second write freeze at cutover acceptable? Those lead
to materially different designs, and the second is dramatically simpler and
safer. I'd propose targeting a **brief, controlled write pause (seconds)**
and state why: it lets you guarantee no data loss and no split-brain
writes, which a truly-zero-pause design cannot without dual-write
reconciliation. I'd design for that and note where the extra work goes if
the requirement is genuinely zero.

**Phase 0 — Inventory and constraints (the part people skip).**
- **Extension and version audit.** Cloud SQL supports a specific extension
  set; RDS supports a different one. `pg_stat_statements`, `postgis`,
  `pg_partman`, `pgvector`, custom extensions — enumerate and verify each is
  available on the target *at a compatible version*. A missing extension
  discovered at cutover is a programme failure.
- **What logical replication does not carry**: DDL, sequences (current
  values), large objects, and — depending on version —
  `TRUNCATE`/generated columns behaviour. Postgres 10+ logical replication
  replicates DML only. Sequence values must be handled explicitly at
  cutover. This is the single most common source of post-cutover corruption:
  the new primary starts issuing ids from 1.
- **Replica identity.** Tables without a primary key need
  `REPLICA IDENTITY FULL` for updates/deletes to replicate, which is
  expensive. Find them now.
- Roles/users/grants, `search_path` assumptions, connection limits, timezone
  and collation (a **collation version difference between source and target
  can change index ordering** — a real correctness hazard; verify the
  glibc/ICU collation versions match or plan to reindex).
- Network path: how does the target reach the source? Cloud Interconnect,
  VPN, or public endpoint with TLS and IP allowlisting (topic 04). Bandwidth
  determines the initial copy time: 4 TB over 1 Gbps is ~9 hours at line
  rate, so realistically half a day to a day. Over a saturated VPN tunnel
  (topic 04, A15) it could be far worse. Size this before promising a
  timeline.
- Downstream consumers: read replicas, ETL jobs, BI tools, CDC pipelines,
  anything with the hostname hardcoded. Each is a cutover task.

**Phase 1 — Establish replication.**
The mechanism is **logical replication** (not physical/streaming, which
requires identical major versions and byte-compatible storage and can't
cross providers). Options:
- **Native logical replication** (`pgoutput`): create a publication on RDS,
  a subscription on Cloud SQL. RDS supports this with
  `rds.logical_replication = 1` (requires a parameter group change and a
  **reboot** — plan it, it's your first outage window). Cloud SQL has a
  documented external-primary/migration path (Database Migration Service)
  built on this.
- **GCP Database Migration Service**, which wraps the above with tooling for
  the initial dump and monitoring. Prefer it if it supports your version
  combination — less bespoke machinery to operate.
- **Debezium/pglogical** if you need transformation or fan-out.

Sequence: initial snapshot (parallel `pg_dump`/`COPY` or DMS's copy phase),
then stream changes from the snapshot's LSN. Watch:
- **Replication slot WAL retention on the source.** If the target falls
  behind or the slot stalls, WAL accumulates on RDS and can fill the disk.
  Alert on `pg_replication_slots.restart_lsn` age and on free storage. This
  is the most likely way to take down the *source* during the migration.
- Indexes: build them after the bulk copy, not during, or the copy takes
  many times longer.
- Long-running transactions on the source delaying the snapshot.

**Phase 2 — Verification (do not skip; allocate real time to this).**
- Row counts per table, continuously, and reconciled at a consistent point.
- Checksums: aggregate hashes per table, or per primary-key range for large
  tables so you can localise a mismatch. Something like
  `md5(string_agg(...))` over ordered chunks, run on both sides against a
  consistent snapshot.
- Sequence values compared.
- Constraint and index inventory compared (`pg_indexes`, `pg_constraint`) —
  logical replication does not create them.
- **Shadow reads**: point a copy of read traffic at the target and diff the
  results against the source. This finds collation differences, planner
  differences, and extension behaviour differences that checksums won't.
- **Performance verification**: run the real query workload against the
  target under production-like load. Cloud SQL's instance class, disk type,
  and default parameters differ from RDS. Discovering at cutover that a hot
  query is 5x slower because `work_mem` or the storage tier differs is a
  rollback. Compare `pg_stat_statements` top queries between the two.

**Phase 3 — Cutover.**
This is a scripted, rehearsed sequence with a stopwatch, not an improvised
one:

1. Announce and freeze schema changes and deploys.
2. **Stop writes** at the application layer — put the app into a read-only
   or maintenance mode, or stop the writer deployments. Better than stopping
   the database, because it's reversible in one step and the app can serve
   reads throughout.
3. **Wait for replication lag to reach zero.** Confirm from the target's
   perspective, not the source's, and confirm the LSN matches.
4. **Advance sequences** on the target: `setval` to the source's current
   value plus a safety margin. Scripted, for every sequence.
5. Run the checksum verification one final time on the now-static data.
6. **Repoint the application.** Ideally via a DNS CNAME or a proxy/PgBouncer
   whose backend you change — not by redeploying every service with a new
   connection string, which is slow and hard to reverse. Whatever mechanism
   you use, it must be reversible in seconds. Note the DNS caching problem
   (topic 02, A12): use a low pre-lowered TTL, or better, a connection
   proxy that you flip server-side.
7. **Set up reverse replication** target → source *before* re-enabling
   writes, if you want a real rollback path. Without it, every write after
   cutover is stranded on the target and rollback means data loss.
8. Re-enable writes. Watch error rates, latency, connection counts.
9. Keep the source running, read-only, for a defined period (days).

Realistic write-pause: **1–5 minutes** dominated by lag drain and sequence
work, and it's mostly deterministic if you've rehearsed. If the requirement
is genuinely zero, the additional machinery is dual-writes with conflict
detection or a proxy that queues writes during the flip — both add
significant risk, and I'd argue hard that a 2-minute planned write pause at
3am is a better engineering trade than a system that can silently diverge.

**Phase 4 — Rollback plan** (must exist before you start Phase 3).
- Trigger criteria defined in advance and *numeric*: error rate above X,
  p99 above Y, any checksum mismatch, any replication failure. Not "if it
  feels bad."
- The rollback action is: flip the proxy/DNS back, confirm reverse
  replication has drained, re-enable writes on the source. Achievable in
  minutes *only if* reverse replication was established.
- A time limit on the decision: if you can't confirm health within N
  minutes, roll back. Rolling back is cheap; discovering corruption a week
  later is not.
- Point of no return: the moment you drop reverse replication or accept
  writes you can't replay. Name it explicitly on the runbook.

**Phase 5 — Rehearsal.** Do the entire thing at least twice against a
restored copy, with the clock running, including the rollback. Most of the
value of this plan is in what the rehearsal teaches you about your own
environment.

**Risks I'd flag unprompted:**
- Cross-cloud egress cost for 4 TB plus ongoing change stream — real money,
  and it should be in the plan.
- Latency during the transition if the app is in AWS and the database moves
  to GCP: **cross-cloud RTT of 10–50 ms per query** turns a 20-query request
  into a timeout. If the application isn't moving at the same time, this is
  the thing that kills the migration, and it means the app cutover and
  database cutover may need to be simultaneous — which is a much bigger
  event. **Raise this in the first five minutes of the answer**; it's the
  constraint most candidates miss entirely.
- Extension/collation mismatches producing subtly different query results.
- Anything reading the WAL on the source (existing CDC pipelines) must be
  re-established against the target.

**Weak answers miss.** Sequences, reverse replication for rollback, and the
application-locality latency problem. A candidate who describes DMS and a
DNS switch has described 20% of the work.

**Follow-ups to expect.**
- What if a table has no primary key? (`REPLICA IDENTITY FULL`, which makes
  every update/delete replicate the whole old row — expensive and slow. Add
  a key first if you can.)
- How would this differ for MySQL? (Binlog-based; `gh-ost`-adjacent tooling,
  GTIDs make the position tracking cleaner. Same programme shape.)
- What if it were 400 TB instead of 4? (Different problem: you can't
  snapshot-and-stream in one pass. Per-table or per-tenant phased migration,
  moving shards independently, with a routing layer that knows where each
  tenant lives. Say that the answer changes shape, not just scale.)

---

### A17. Query went from 20 ms to 8 s with no deploy

**Answer.**
"No deploy" points away from the query and toward its inputs: data,
statistics, plan, or contention. Work through them in cost order.

**1. Is it this query, or everything?** Check overall database latency and
CPU/IO first. If everything is slow, the query is a victim, not a cause —
go look for a vacuum, a backup, a replica rebuild, a noisy neighbour, or
storage degradation (topic 05, A15). One command's worth of information that
eliminates half the possibilities.

**2. Get the actual plan now, and compare it to the good one.**
`EXPLAIN (ANALYZE, BUFFERS)` on the slow execution. The decisive question is
whether the **plan changed**. If it did, you're looking at a planner
decision; if it didn't, the same plan is now doing more work.

**Plan changed — likely causes:**
- **Stale statistics.** The table grew or its distribution shifted, and
  `ANALYZE` hasn't run. The classic manifestation: the planner still thinks
  a table has 1000 rows, chooses a nested loop, and it's now 10 million.
  Fix: `ANALYZE` the table; check `pg_stat_user_tables.last_autoanalyze`.
- **Crossed a cost threshold.** Data growth flipped the planner from an
  index scan to a sequential scan (or vice versa) — the estimated
  selectivity crossed the point where a bitmap or seq scan looked cheaper.
  Small changes in row counts produce discontinuous plan changes; this is
  normal planner behaviour and it's why "nothing changed" is never true.
- **Parameter sniffing / bad plan cache.** A prepared statement's generic
  plan was built for an unrepresentative parameter value. Postgres switches
  from custom to generic plans after 5 executions; a generic plan chosen for
  a skewed column can be catastrophic. Test by running the query directly
  with literals vs prepared.
- **Index bloat or an invalid index.** A `CREATE INDEX CONCURRENTLY` that
  failed leaves an `INVALID` index the planner won't use. Check
  `pg_index.indisvalid`.
- **Correlation the planner can't see.** Two columns are correlated
  (`city` and `postcode`); the planner multiplies selectivities and
  underestimates by orders of magnitude. Fix: extended statistics
  (`CREATE STATISTICS`).

**Plan unchanged — the same plan is doing more work:**
- **Data volume grew** past what fits in cache. The `BUFFERS` output tells
  you directly: a jump in `read` (disk) vs `hit` (cache) is the signal. A
  query whose working set just exceeded `shared_buffers`/page cache falls off
  a cliff with no plan change at all.
- **Bloat** from dead tuples (A2), so the same scan reads far more pages.
  Check `n_dead_tup` and table size vs row count.
- **Lock contention.** The query is waiting, not working. `pg_stat_activity`
  `wait_event_type`/`wait_event`, and `pg_locks` for blockers. A long
  transaction holding a lock produces exactly this — sudden, no deploy.
- **Connection pool saturation**: the "query" time measured by the
  application includes waiting for a connection. Verify where the
  application measures from.
- Underlying storage: burst credit exhaustion on gp2, an EBS volume
  degradation, or a noisy neighbour.

**3. Check what else changed in the environment**, since "no deploy" only
covers application code: a data backfill or bulk import, a new report or ETL
job hitting the same table, a failover to a replica with a cold cache, a
parameter group change, an autovacuum that hasn't kept up, or a
minor-version upgrade.

**What I'd actually do first**, in order: `pg_stat_activity` to see if it's
waiting or working; `EXPLAIN (ANALYZE, BUFFERS)` to compare plans;
`last_autoanalyze` and `n_dead_tup`. Three checks, under five minutes, and
they discriminate between every branch above.

**Mitigation while diagnosing**: `ANALYZE` is cheap and safe and fixes a
large share of these. If it's plan instability, pinning behaviour with
`pg_hint_plan`, disabling a specific plan type for the session, or
restructuring the query are options — but treat plan pinning as a temporary
measure, since it hides future changes.

**Weak answers miss.** The plan-changed vs plan-same branch, which is the
organising question. Also missed: checking whether the query is *waiting*
before analysing how it *runs*.

**Follow-ups to expect.**
- How do you catch this proactively? (`pg_stat_statements` with alerting on
  mean/total time regressions per query id — the only way to notice before
  users do. `auto_explain` with a duration threshold logs the actual plan for
  slow executions, which gives you the evidence retroactively instead of
  needing to reproduce.)

---

### A18. Database commit succeeds, Kafka event never appears

**Answer.**
**Why it happens**: two independent systems, two independent writes, no
atomicity across them. The code is

```
tx.commit()          // durable in Postgres
producer.send(event) // separate network call
```

Anything between those two lines — process crash, pod eviction, OOM kill,
network partition to Kafka, broker unavailability, a retry budget exhausted
— leaves the database updated and the event lost. There is no ordering of
the two calls that fixes it: swap them and you get phantom events for
transactions that rolled back, which is usually worse.

**Distributed transactions (2PC) are not the answer** here. Kafka supports
transactions but not XA with an arbitrary database; even where 2PC is
available it's blocking — a coordinator failure between prepare and commit
leaves participants holding locks indefinitely — and it couples your
availability to the product of both systems'. Say this explicitly, because
"use 2PC" is the trap answer.

**The fix: the transactional outbox pattern.**

1. In the **same database transaction** as the business change, insert a row
   into an `outbox` table: `(id, aggregate_id, event_type, payload,
   created_at, published_at NULL)`. Now the event's existence is atomic with
   the state change — one commit, one durability decision.
2. A **separate relay process** reads unpublished outbox rows and publishes
   them to Kafka, marking them published (or deleting them) after the broker
   acknowledges.

Two ways to build the relay:

- **Polling publisher.** Query for unpublished rows, publish, mark. Simple,
  works anywhere, no special database features. Costs: polling latency,
  load on the database, and you must handle concurrency (multiple relay
  instances) with `SELECT ... FOR UPDATE SKIP LOCKED`, which is the right
  primitive here.
- **CDC / log tailing** — Debezium reading the WAL. No polling load on the
  database, low latency, and it captures the outbox inserts in commit order.
  This is the better production answer at volume. Cost: operating Debezium
  and Kafka Connect, managing replication slots (and the WAL-retention
  hazard from A16), and schema evolution of the change events.

**What the outbox does and does not give you:**
- It gives **at-least-once** delivery. The relay can crash after publishing
  and before marking, so the event is published twice.
- It does **not** give exactly-once. Nothing does, end-to-end, without
  consumer cooperation. **Consumers must be idempotent** — dedupe on the
  event id, or make the effect naturally idempotent (upsert rather than
  insert, set rather than increment). This is the sentence that must be in
  the answer.
- Ordering: within a partition, Kafka preserves order. Key the message by
  aggregate id so all events for one entity land in one partition and stay
  ordered. Across aggregates there is no global order and you shouldn't
  design as though there is.

**Operational details worth naming:**
- The outbox table is high-churn; in Postgres that means bloat (A2). Delete
  published rows promptly, or partition by time and drop partitions.
- Monitor **outbox lag** — the age of the oldest unpublished row. It's the
  single best health metric for this pattern and it directly measures the
  thing your users would notice.
- Kafka producer config matters: `acks=all`, `enable.idempotence=true`,
  bounded retries, and `min.insync.replicas` ≥ 2 on the topic, or you can
  lose the event *after* the relay believed it was published (see the
  distributed systems topic).
- Payload versioning from day one — you will change the event schema.

**The alternative worth mentioning**: if the consumer only needs the state
change and not a semantic event, skip the outbox and use **CDC on the
business table directly**. Fewer moving parts. The reason to prefer an
explicit outbox is that it decouples your internal schema from your public
event contract — CDC on business tables makes every column rename a
breaking change for downstream consumers. That's a design argument, not a
technical one, and it's usually decisive.

**Weak answers miss.** That the outbox gives at-least-once and therefore
requires idempotent consumers, and the schema-coupling argument for outbox
over raw table CDC.

**Follow-ups to expect.**
- What if the relay publishes out of order after a crash? (Order is
  preserved if you publish in outbox insertion order per key and use a
  single ordered reader per partition key. `SKIP LOCKED` with multiple
  relays can reorder across keys — fine — but you must not let two relays
  handle the same key concurrently.)
- How do you handle a poison event the broker rejects? (Bounded retries,
  then park it in a dead-letter table and alert. Do not let one bad row
  block the whole outbox — head-of-line blocking in your own pipeline.)

---

### A19. Storage for 500k events/s with 24-hour dashboards and 13-month analytics

**Answer.**
**Start with the arithmetic**, because it determines everything.

- 500,000 events/s. Say each event is ~500 bytes raw (a handful of
  dimensions plus values) → **250 MB/s ingest**, ~21 TB/day raw.
- At 10x columnar compression (realistic for telemetry with delta/Gorilla
  codecs — A14): **~2.1 TB/day on disk**.
- 13 months ≈ 400 days → **~840 TB** compressed if everything is retained at
  full resolution. That is a large but not absurd number, and it is clearly
  the wrong answer to keep it all hot.
- 24 hours hot = ~2.1 TB compressed. That fits comfortably on local NVMe on
  a modest cluster and can be served with sub-second latency.

That split — 2 TB hot vs 840 TB cold — **is the design**. State it before
naming any technology.

**The architecture:**

```
producers → Kafka (buffer, replay, decoupling)
              ├→ stream aggregation (pre-computed rollups)  → hot store
              └→ raw sink → ClickHouse (hot, local NVMe, ~7-30d)
                                 └→ tiered storage → object storage (cold)
                                                          ↑
                                       ad-hoc queries via ClickHouse over
                                       S3/GCS, or a lakehouse table format
```

**Ingest buffer: Kafka.** Non-negotiable at this rate. It decouples producer
rate from store write rate, absorbs bursts, survives a store outage without
data loss, and gives you **replay** — which is what makes backfills,
schema changes, and adding a second consumer possible at all. Sizing:
250 MB/s at 3x replication is 750 MB/s of cluster write throughput; with a
retention of, say, 24 hours that's ~21 TB × 3 = 63 TB of Kafka storage.
Partition count driven by consumer parallelism — order of hundreds.
Partition key by something that gives even distribution *and* keeps a
series' events ordered (topic 06, A11 logic applies).

**Hot store: ClickHouse.** Why it fits: the query pattern is analytical
(aggregate over dimensions, filter by time), the ingest rate is exactly what
MergeTree is built for, and compression makes the hot window cheap.
- `PARTITION BY toDate(ts)` (or by hour if daily partitions are too coarse)
  so retention is a `DROP PARTITION` rather than a delete.
- `ORDER BY (tenant_id, metric, ts)` — put the highest-selectivity
  equality dimensions first, time last, per A3's logic. This decides your
  query performance more than anything else.
- **Batch inserts** — large batches every few seconds, never per-event, or
  you hit "too many parts" (A14). The stream consumer does the batching.
- Local NVMe for the hot partitions.

**Sub-second dashboards: don't query raw.** At 500k/s, a dashboard panel
scanning 24 hours of raw data reads ~2 TB. Even ClickHouse won't do that in
under a second reliably, and doing it for every panel refresh for every user
is a capacity disaster.
The answer is **pre-aggregation**: materialized views (`AggregatingMergeTree`)
maintaining per-minute rollups by the dimension combinations dashboards
actually use, populated automatically on insert. A dashboard query then
reads minute-granularity rows — orders of magnitude less data — and hits
sub-second easily. Raw data remains available for drill-down.
The cost you accept: **you must know the dimension combinations in advance**.
Novel groupings fall back to the raw table and are slow. That's the central
tradeoff and I'd state it as such: dashboards get a fixed, fast contract;
exploration gets a slower, general one.

**Cold tier: object storage.** Two credible options:
1. **ClickHouse tiered storage** — a storage policy that moves parts older
   than N days from NVMe to S3/GCS, with the table remaining queryable
   transparently. Simplest operationally; queries over cold data are much
   slower (object storage latency and bandwidth) but functional. Keeps one
   query interface for the whole 13 months.
2. **Export to an open table format** (Parquet + Iceberg/Delta) in object
   storage, queried by ClickHouse, Trino, Spark, or BigQuery/Athena. More
   moving parts, but decouples long-term data from the ClickHouse cluster's
   lifecycle, and lets other tools read it. Better if multiple teams and
   engines need the archive.
I'd pick (1) if the analytics audience is small and ClickHouse-native, (2)
if the archive is a shared company asset.
Either way: **downsample before archiving** where the use case permits.
Keeping 13 months at 1-minute resolution rather than raw cuts the cold tier
by 1–2 orders of magnitude. Whether you can depends on whether ad-hoc
queries need individual events — which is a requirements question I'd ask
explicitly.

**What I'd give up, stated plainly:**
- **Sub-second on cold data.** 13-month ad-hoc queries will take seconds to
  minutes. That is the correct trade; making 840 TB uniformly fast costs
  more than the business value.
- **Exactly-once.** Kafka → ClickHouse is at-least-once; duplicates are
  handled by `ReplacingMergeTree` with a deterministic event id, accepting
  that dedup is eventual (A14) and that queries needing exactness must use
  `argMax`-style aggregation. Alternatively, dedupe in the stream processor
  with a bounded window.
- **Updates and deletes.** Append-only. GDPR deletion must be designed for
  up front — partition or shard by a key that lets you drop whole partitions,
  or accept expensive mutations on a schedule.
- **Arbitrary low-latency slicing.** Pre-aggregation means the fast path is
  the anticipated path.
- **Strong consistency between ingest and query.** There's a seconds-scale
  lag from event to queryable. Fine for dashboards; state it as an SLO.

**Failure modes to name:** ClickHouse down → Kafka retains, consumer catches
up (so Kafka retention must exceed your worst realistic outage — this is why
24h retention, not 4h). Consumer lag → alert on it as the primary pipeline
health metric. Too-many-parts → back-pressure the consumer, don't shrink
batches. Cardinality explosion in a dimension (a tenant sending unique ids
as a label) → per-tenant cardinality limits at ingest, or one tenant
degrades the cluster for everyone.

**Weak answers miss.** Doing the arithmetic first, and pre-aggregation for
the dashboard requirement — proposing to query raw data for sub-second
dashboards at this rate is the fundamental error. Also missed: Kafka
retention sized to the outage you want to survive.

**Follow-ups to expect.**
- Why not Elasticsearch? (Viable for search and moderate analytics, but at
  this ingest rate the indexing cost and storage overhead are far higher
  than a columnar store, and aggregations over 24h of 43 billion documents
  are not sub-second. Right tool for full-text and per-document retrieval,
  wrong tool for this. Given your ES background, have a concrete comparison
  ready.)
- How would you handle multi-tenancy? (Tenant in the sort key for locality,
  per-tenant quotas at ingest, and query concurrency limits per tenant.
  If isolation must be hard, separate clusters or cells — see the
  distributed systems topic.)
