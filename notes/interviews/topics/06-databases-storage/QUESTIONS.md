# Databases & Storage — Questions

Where the "senior engineer" bar is set for backend and SRE roles alike. The
migration questions (Q15, Q16) are the ones you were actually asked; treat
them as the centrepiece of this topic.

19 questions.

---

## Tier 1 — Recall

### Q1. Name the four SQL isolation levels and the anomaly each one permits.
*Tags: acid, isolation, sql*

### Q2. What is MVCC, and what problem does a long-running transaction cause in PostgreSQL?
*Tags: postgres, mvcc, vacuum*

### Q3. You have an index on `(a, b, c)`. Which of these queries can use it, and why: filter on `a`; filter on `b`; filter on `a` and `c`; filter on `a` with a range, then `b`?
*Tags: indexes, sql*

### Q4. What is a covering index, and what is an index-only scan?
*Tags: indexes, performance*

### Q5. What is write-ahead logging, and what does it buy you?
*Tags: durability, wal*

### Q6. What is replication lag, and name three ways a user notices it.
*Tags: replication, consistency*

---

## Tier 2 — Explain / compare

### Q7. Compare B-tree and LSM-tree storage engines. What does each optimise, and what is read/write amplification in each?
*Tags: storage-engines, lsm, btree* · *[infra-heavy]*

### Q8. Why does PostgreSQL need a connection pooler when MySQL is less sensitive to connection count? Explain transaction vs session pooling.
*Tags: postgres, pooling, pgbouncer*

### Q9. Explain synchronous vs asynchronous replication. What exactly do you lose in a failover with each?
*Tags: replication, rpo, failover*

### Q10. What is split brain in a database cluster, and what is fencing? Why is automated failover dangerous without it?
*Tags: failover, consensus, safety*

### Q11. How do you choose a shard key? Give me an example of a bad one and explain the failure.
*Tags: sharding, partitioning*

### Q12. Compare the consistency and transaction models of DynamoDB, Cassandra, and Spanner. What does each actually guarantee?
*Tags: nosql, consistency, cap*

### Q13. Explain cache-aside vs write-through vs write-behind. Then explain cache stampede and three ways to prevent it.
*Tags: caching, patterns*

### Q14. Why is ClickHouse fast for analytical queries, and what are the three things it's genuinely bad at?
*Tags: olap, clickhouse, columnar* · *[on your resume — expect this]*

### Q15. Walk me through adding a `NOT NULL` column with a default to a 500 GB table in production, with no downtime.
*Tags: migrations, ddl, postgres*

---

## Tier 3 — Scenario / debug

### Q16. Migrate a 4 TB production PostgreSQL database from AWS RDS to GCP Cloud SQL with zero downtime. Design the whole programme: cutover, verification, and rollback.
*Tags: migration, replication, cutover* · *[asked verbatim in your interviews]*

### Q17. A query that ran in 20 ms yesterday takes 8 seconds today. Nothing was deployed. Walk me through the diagnosis.
*Tags: debugging, query-planner, postgres*

### Q18. Your service writes to a database and publishes an event to Kafka. Sometimes the database commit succeeds and the event never appears. Explain why, and design the fix.
*Tags: consistency, outbox, cdc*

### Q19. Design the storage layer for a system ingesting 500k events/second that must serve (a) sub-second dashboard queries over the last 24 hours and (b) ad-hoc analytical queries over 13 months. State what you'd use and what you'd give up.
*Tags: design, olap, tiering, retention*
