# Observability, SLOs & On-Call — Questions

The topic where SRE interviews decide whether you've *operated* systems or
merely built them. Expect the SLO and alerting questions at any company with
a mature SRE function, and the cardinality questions anywhere that runs
Prometheus at scale.

18 questions.

---

## Tier 1 — Recall

### Q1. Metrics, logs, traces, profiles: what does each answer that the others can't?
*Tags: telemetry, fundamentals*

### Q2. Define SLI, SLO, SLA, and error budget. How do they relate?
*Tags: slo, reliability*

### Q3. Explain the RED and USE methods. When is each the right frame?
*Tags: monitoring, method*

### Q4. Why can't you average percentiles? What should you do instead?
*Tags: statistics, percentiles* · *[very commonly asked]*

### Q5. What is cardinality in a metrics system, and why is it the thing that kills you?
*Tags: prometheus, cardinality*

---

## Tier 2 — Explain / compare

### Q6. Compare pull-based and push-based metrics collection. What does Prometheus's pull model give you, and where does it break?
*Tags: prometheus, architecture*

### Q7. Explain Prometheus histograms. What is the quantile estimation error, and what do native/exponential histograms change?
*Tags: prometheus, histograms, statistics*

### Q8. Compare federation, remote write, Thanos, Cortex/Mimir, and VictoriaMetrics for scaling Prometheus across a large fleet.
*Tags: prometheus, scale* · *[on your resume]*

### Q9. Explain multi-window multi-burn-rate alerting. Why is it better than "alert when error rate > 1%"?
*Tags: slo, alerting* · *[infra-heavy]*

### Q10. What makes a good alert? Compare symptom-based and cause-based alerting, and page vs ticket.
*Tags: alerting, oncall*

### Q11. Explain head-based and tail-based trace sampling. What does each cost, and what can tracing never tell you?
*Tags: tracing, sampling*

### Q12. Design a log pipeline for a 3000-node fleet. What are the retention tiers and where does the money go?
*Tags: logging, elasticsearch, cost* · *[on your resume]*

### Q13. How do you pick an SLI for a service? Walk through a concrete example and say what you'd reject.
*Tags: slo, measurement*

---

## Tier 3 — Scenario / debug

### Q14. Your dashboards are all green and customers say the product is broken. What is wrong with your monitoring, and how do you fix it structurally?
*Tags: observability, gaps, method* · *[classic]*

### Q15. A metrics cardinality explosion takes down your Prometheus during an incident, blinding you. Walk through detection, immediate mitigation, and prevention.
*Tags: prometheus, cardinality, incident*

### Q16. You're paged at 3am for a service you've never seen. Walk me through your first ten minutes.
*Tags: oncall, incident-response, method*

### Q17. Write the postmortem structure for an outage where a routine config change caused a 40-minute total outage. What separates a good postmortem from a bad one?
*Tags: postmortem, culture*

### Q18. Design the observability for a data pipeline: Kafka → stream processor → ClickHouse, 500k events/s, where the failure you most fear is silent data loss.
*Tags: design, pipelines, data-quality* · *[on your resume]*
