# Kubernetes & Containers — Questions

Your deepest operational territory, which means interviewers will push past
"what is a Deployment" into etcd behaviour, bare-metal differences, and
stateful workloads. The Tier 3 questions here are the ones that separate
people who use Kubernetes from people who run it.

18 questions.

---

## Tier 1 — Recall

### Q1. You run `kubectl apply -f deployment.yaml`. Trace what happens, component by component, until a container is running.
*Tags: control-plane, reconciliation*

### Q2. Explain requests and limits, and the three QoS classes they produce.
*Tags: scheduling, qos, resources*

### Q3. What is the difference between liveness, readiness, and startup probes?
*Tags: probes, health*

### Q4. What actually is a Service? What happens to a packet sent to a ClusterIP?
*Tags: networking, kube-proxy, services*

### Q5. What is a PodDisruptionBudget, and which kinds of disruption does it protect against?
*Tags: availability, deploys*

---

## Tier 2 — Explain / compare

### Q6. Why is a liveness probe that checks a database a bad idea, and what is the specific cascading failure it causes?
*Tags: probes, cascading-failure* · *[classic]*

### Q7. Compare kube-proxy in iptables mode, IPVS mode, and an eBPF datapath like Cilium. What breaks at scale in each?
*Tags: networking, scale, ebpf* · *[on your resume]*

### Q8. Compare overlay networking (VXLAN/Geneve) with native routing for a CNI. What are the costs of each?
*Tags: cni, networking, mtu*

### Q9. Explain what etcd is doing for the cluster, how it fails, and what happens to a running cluster when etcd is unavailable.
*Tags: etcd, control-plane, failure* · *[infra-heavy]*

### Q10. Compare HPA, VPA, Cluster Autoscaler, and Karpenter. Why is HPA on CPU usually the wrong signal?
*Tags: autoscaling*

### Q11. Is a namespace a security boundary? If not, what is?
*Tags: security, multi-tenancy*

### Q12. What makes running Kafka, ClickHouse, or Elasticsearch on Kubernetes hard? What does a StatefulSet actually give you?
*Tags: stateful, storage* · *[on your resume]*

### Q13. Walk through a rolling update. What do `maxSurge` and `maxUnavailable` do, and how do you get a zero-error deploy?
*Tags: deploys, availability*

### Q14. Compare Ingress, the Gateway API, and a service mesh. What problem does each solve that the previous one doesn't?
*Tags: ingress, mesh*

---

## Tier 3 — Scenario / debug

### Q15. A pod is stuck in `Pending`. Then a different one is stuck in `Terminating` for 20 minutes. Then a third is in `CrashLoopBackOff`. Give me your systematic approach to each.
*Tags: debugging, method*

### Q16. Nodes in your cluster periodically go `NotReady` for 30–60 seconds and then recover. Pods get evicted. Nothing is obviously wrong. Investigate.
*Tags: debugging, kubelet, control-plane* · *[infra-heavy]*

### Q17. What changes when you run Kubernetes on bare metal instead of a managed cloud service? Design the gaps you have to fill.
*Tags: bare-metal, design* · *[on your resume]*

### Q18. Design multi-tenancy for a platform where ~200 internal teams deploy to a shared cluster fleet, with a hard requirement that no team can degrade another.
*Tags: design, multi-tenancy, isolation*
