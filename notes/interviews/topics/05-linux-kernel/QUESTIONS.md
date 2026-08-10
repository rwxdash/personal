# Linux, Kernel & Performance — Questions

The topic where an SRE resume with eBPF on it gets tested hardest. Expect
interviewers to skip the basics and go straight to "what does the verifier
reject and why" or "walk me through diagnosing a latency problem you can't
see in metrics."

18 questions.

---

## Tier 1 — Recall

### Q1. What does `fork()` actually copy? Explain copy-on-write and what happens on `exec()`.
*Tags: processes, memory*

### Q2. `free -m` shows almost no free memory on a healthy box. Explain what's going on.
*Tags: memory, page-cache*

### Q3. What is Linux load average actually measuring, and why is it different from other Unixes?
*Tags: scheduler, metrics*

### Q4. What is a zombie process, what is an orphan, and why does this matter in a container?
*Tags: processes, containers, pid1*

### Q5. Name the Linux namespaces and say which resource each isolates.
*Tags: namespaces, containers*

### Q6. What is the difference between a voluntary and an involuntary context switch, and what does a high rate of each tell you?
*Tags: scheduler, performance*

---

## Tier 2 — Explain / compare

### Q7. Explain how the OOM killer chooses a victim. How does this differ under cgroup v2, and what is the difference between `memory.high` and `memory.max`?
*Tags: memory, cgroups, oom* · *[infra-heavy]*

### Q8. Explain CPU limits in cgroups. Why can a container at 30% average CPU utilisation have terrible tail latency?
*Tags: cgroups, cfs, throttling, kubernetes* · *[very commonly asked]*

### Q9. Compare `strace`, `perf`, and eBPF/bpftrace for investigating a production process. What is the cost of each?
*Tags: tracing, ebpf, perf*

### Q10. Where can eBPF programs attach, and what are the main constraints the verifier imposes?
*Tags: ebpf, kernel* · *[on your resume — expect this]*

### Q11. Compare blocking I/O with threads, epoll-based event loops, and io_uring. What problem does each solve?
*Tags: io, async, performance*

### Q12. Explain the difference between IOPS, throughput, and latency for a storage device, and what queue depth does to all three.
*Tags: storage, performance*

### Q13. What does `fsync()` guarantee, and what lies between your `write()` and the platter/flash?
*Tags: storage, durability*

---

## Tier 3 — Scenario / debug

### Q14. A process is using 100% of one CPU. Walk me through finding out what it's doing, from cheapest tool to most invasive.
*Tags: debugging, method, profiling*

### Q15. Your fleet shows high `iowait` on some nodes and high `steal` on others. Explain what each means and what you'd do about it.
*Tags: cpu, virtualisation, storage*

### Q16. An application's p99 latency is 40x its p50, with no obvious CPU or memory pressure. Give me the hypotheses you'd work through.
*Tags: latency, tail, method* · *[infra-heavy]*

### Q17. A service handling many short-lived connections is dropping requests at the accept path under load, well below the CPU limit. Where do you look?
*Tags: networking, kernel-tuning, queues*

### Q18. You need to know which processes on a 3000-node fleet are writing to a specific file path, continuously, with negligible overhead. How do you build it?
*Tags: ebpf, observability, fleet* · *[on your resume]*
