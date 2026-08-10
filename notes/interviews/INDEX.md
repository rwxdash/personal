# Topic Index

172 questions across 10 topics. Every topic ships a spoiler-free
`QUESTIONS.md` and a matching `ANSWERS.md` with model answers, the traps a
weak answer falls into, and the follow-ups an interviewer asks next.

**New here? Read [README.md](README.md)** — how to drill, and why answering
out loud is the whole point.

**Fill in the Confidence column yourself.** It's blank on purpose. Log
attempts in [PROGRESS.md](PROGRESS.md).

## Topics

| # | Topic | Questions | T1 / T2 / T3 | Confidence |
| --- | --- | --- | --- | --- |
| 01 | [TCP/IP & the Wire](topics/01-tcp-ip/QUESTIONS.md) | 18 | 7 / 8 / 3 | |
| 02 | [DNS, TLS & HTTP](topics/02-dns-tls-http/QUESTIONS.md) | 17 | 6 / 8 / 3 | |
| 03 | [Load Balancing & Proxies](topics/03-load-balancing/QUESTIONS.md) | 17 | 5 / 9 / 3 | |
| 04 | [Cloud Networking, VPN & IPsec](topics/04-cloud-networking-vpn/QUESTIONS.md) | 16 | 6 / 7 / 3 | |
| 05 | [Linux, Kernel & Performance](topics/05-linux-kernel/QUESTIONS.md) | 18 | 6 / 7 / 5 | |
| 06 | [Databases & Storage](topics/06-databases-storage/QUESTIONS.md) | 19 | 6 / 9 / 4 | |
| 07 | [Distributed Systems](topics/07-distributed-systems/QUESTIONS.md) | 18 | 5 / 10 / 3 | |
| 08 | [Kubernetes & Containers](topics/08-kubernetes/QUESTIONS.md) | 18 | 5 / 9 / 4 | |
| 09 | [Observability, SLOs & On-Call](topics/09-observability-sre/QUESTIONS.md) | 18 | 5 / 8 / 5 | |
| 10 | [Cloud Platform, IaC, Migration & Cost](topics/10-cloud-platform-migration/QUESTIONS.md) | 17 | 5 / 8 / 4 | |

## Suggested first-pass order

Index order. The topics build on each other — TCP before load balancing,
load balancing before cloud networking, distributed systems before
Kubernetes multi-tenancy. Cross-references in the answers assume you've read
the earlier topics.

After the first pass, drill by weakness rather than by order.

## The questions you were actually asked

These appear verbatim or near-verbatim in your interview history. Be able to
give both a 3-minute verbal answer and a whiteboard version.

| Question | Where |
| --- | --- |
| Zero-downtime database migration between cloud vendors | [06 · A16](topics/06-databases-storage/ANSWERS.md) |
| Network LB vs Application LB (NLB vs ALB) | [03 · A1](topics/03-load-balancing/ANSWERS.md) |
| Site-to-site IPsec tunnel — IKE phases, SAs | [04 · A4](topics/04-cloud-networking-vpn/ANSWERS.md) |
| TCP/IP textbook questions | [01](topics/01-tcp-ip/QUESTIONS.md), Tier 1 |

## Questions your resume invites

An interviewer reading your CV will go deeper here than a generic bank
would. These are the ones to over-prepare, ideally with a specific story
attached.

| Area | Questions |
| --- | --- |
| eBPF | [05 · Q10](topics/05-linux-kernel/QUESTIONS.md) (attach points, verifier), [05 · Q18](topics/05-linux-kernel/QUESTIONS.md) (fleet-wide tool), [05 · Q9](topics/05-linux-kernel/QUESTIONS.md) (vs strace/perf) |
| Cilium / kube-proxy | [03 · Q10](topics/03-load-balancing/QUESTIONS.md), [08 · Q7](topics/08-kubernetes/QUESTIONS.md) |
| ClickHouse | [06 · Q14](topics/06-databases-storage/QUESTIONS.md), [06 · Q19](topics/06-databases-storage/QUESTIONS.md) |
| Kafka | [07 · Q10](topics/07-distributed-systems/QUESTIONS.md) (durability config), [09 · Q18](topics/09-observability-sre/QUESTIONS.md) (pipeline observability) |
| Elasticsearch / logging at scale | [09 · Q12](topics/09-observability-sre/QUESTIONS.md) |
| Prometheus at fleet scale | [09 · Q8](topics/09-observability-sre/QUESTIONS.md), [09 · Q15](topics/09-observability-sre/QUESTIONS.md) |
| Bare-metal Kubernetes | [08 · Q17](topics/08-kubernetes/QUESTIONS.md) |
| Terraform + AWS CDK | [10 · Q6](topics/10-cloud-platform-migration/QUESTIONS.md), [10 · Q7](topics/10-cloud-platform-migration/QUESTIONS.md) |
| Post-acquisition integration | [10 · Q15](topics/10-cloud-platform-migration/QUESTIONS.md), [04 · Q10](topics/04-cloud-networking-vpn/QUESTIONS.md) (overlapping CIDRs) |
| 3000-node fleet operations | [05 · Q18](topics/05-linux-kernel/QUESTIONS.md), [08 · Q18](topics/08-kubernetes/QUESTIONS.md), [09 · Q12](topics/09-observability-sre/QUESTIONS.md) |

## Cross-topic threads

Several ideas recur across topics. If you understand the thread, you can
answer questions in the bank that aren't written down.

- **Queueing and utilisation** — [07 · A11](topics/07-distributed-systems/ANSWERS.md) is the theory;
  it explains [05 · A12](topics/05-linux-kernel/ANSWERS.md) (queue depth),
  [03 · A14](topics/03-load-balancing/ANSWERS.md) (shedding), and
  [10 · A17](topics/10-cloud-platform-migration/ANSWERS.md) (capacity).
- **Retries and amplification** — [02 · A16](topics/02-dns-tls-http/ANSWERS.md) →
  [03 · A14](topics/03-load-balancing/ANSWERS.md) → [07 · A16](topics/07-distributed-systems/ANSWERS.md).
- **Idempotency** — [02 · A6](topics/02-dns-tls-http/ANSWERS.md) →
  [06 · A18](topics/06-databases-storage/ANSWERS.md) →
  [07 · A8](topics/07-distributed-systems/ANSWERS.md) → [07 · A18](topics/07-distributed-systems/ANSWERS.md).
- **MTU and tunnels** — [01 · A9](topics/01-tcp-ip/ANSWERS.md) →
  [04 · A7](topics/04-cloud-networking-vpn/ANSWERS.md) → [08 · A8](topics/08-kubernetes/ANSWERS.md).
- **Correlated failure and blast radius** — [03 · A8](topics/03-load-balancing/ANSWERS.md) →
  [07 · A14](topics/07-distributed-systems/ANSWERS.md) → [08 · A6](topics/08-kubernetes/ANSWERS.md) →
  [08 · A18](topics/08-kubernetes/ANSWERS.md).
- **Measuring from the wrong place** — [02 · A17](topics/02-dns-tls-http/ANSWERS.md) →
  [09 · A13](topics/09-observability-sre/ANSWERS.md) → [09 · A14](topics/09-observability-sre/ANSWERS.md).
- **Migrations are all the same shape** — [06 · A15](topics/06-databases-storage/ANSWERS.md) →
  [06 · A16](topics/06-databases-storage/ANSWERS.md) →
  [10 · A8](topics/10-cloud-platform-migration/ANSWERS.md) → [10 · A14](topics/10-cloud-platform-migration/ANSWERS.md).
