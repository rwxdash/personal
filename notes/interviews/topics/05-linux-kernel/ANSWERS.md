# Linux, Kernel & Performance — Answers

---

## Tier 1 — Recall

### A1. fork, copy-on-write, exec

**Answer.**
`fork()` creates a child that is a near-duplicate of the parent: same file
descriptor table (descriptors are *shared*, pointing at the same open file
descriptions with shared offsets), same memory contents, same signal
handlers. What it does **not** copy is the physical memory.

**Copy-on-write**: the kernel copies only the page tables, and marks every
writable page **read-only** in both processes. Both now reference the same
physical frames. When either writes, the MMU raises a page fault, the kernel
allocates a fresh frame, copies the 4 KB page into it, and remaps it
writable for the faulting process. So the cost of `fork()` is proportional
to the size of the page tables, not to the resident memory, and the copy
cost is paid lazily and only for pages actually modified.

`exec()` then discards the entire address space and replaces it with the new
program's — so a `fork()` immediately followed by `exec()` did all that page
table work for nothing. That's why `vfork()` and, better, `posix_spawn()` /
`clone()` with `CLONE_VM` exist, and why modern runtimes use
`posix_spawn` for subprocess launching.

Two consequences worth stating because they bite in production:
- **Fork of a large process can fail even with COW.** With
  `vm.overcommit_memory=2` (strict) or under a cgroup memory limit, the
  kernel may refuse to fork a 30 GB process because the *worst case* needs
  30 GB more. The classic form is Redis `BGSAVE`, and the mitigation is
  overcommit settings plus headroom.
- **File descriptors are shared, not copied.** `O_CLOEXEC` (or
  `FD_CLOEXEC`) is what stops a child inheriting descriptors it shouldn't —
  a real security issue when the child is less trusted, and a real
  descriptor-leak source otherwise.

**Weak answers miss.** That fd's are shared with a shared file offset, and
that the fork cost scales with page-table size (which is why huge pages
change the arithmetic).

**Follow-ups to expect.**
- Why is fork problematic in a multithreaded process? (Only the calling
  thread survives in the child. If another thread held a lock — including a
  malloc arena lock — it's held forever in the child. Between `fork` and
  `exec` you may only call async-signal-safe functions. This is the source
  of the classic "child hangs on the first `printf`" bug.)
- What does `clone()` add? (Fine-grained sharing flags — it's how threads
  and containers are both built.)

---

### A2. `free -m` shows no free memory

**Answer.**
Because the kernel uses otherwise-idle RAM as **page cache**, and free RAM
is wasted RAM. Every file read is cached; every buffered write lands in the
cache as a dirty page and is written back later. That memory shows in
`buff/cache`, not `free`.

The number to read is **`available`**, not `free`. `available` is the
kernel's estimate of how much a new workload could allocate without
swapping, and it accounts for the reclaimable portion of the page cache and
slab. A box with 200 MB `free` and 40 GB `available` is completely healthy.

The mechanics worth adding: dirty pages are written back by kernel
writeback threads, governed by `vm.dirty_background_ratio` (start writing
back in the background) and `vm.dirty_ratio` (writers **block** until
writeback catches up). Hitting `dirty_ratio` is a real production stall —
the application blocks in `write()` and it looks like the application is
slow. `/proc/meminfo`'s `Dirty` and `Writeback` are the counters.

Also: `vm.swappiness` controls the tradeoff between reclaiming page cache
and swapping anonymous memory. Setting it to 0 doesn't disable swap, it
makes the kernel avoid it until nearly out of memory, which can convert
"a bit of swapping" into "OOM kill." And on a database host, dropping page
cache to "free memory" will destroy your read performance — the cache *is*
the performance.

**Weak answers miss.** `available` vs `free`, and dirty-ratio write
stalls.

**Follow-ups to expect.**
- Should you disable swap on a Kubernetes node? (Historically required by
  kubelet; swap support has since been added in stages, so verify the
  current state for your version. The argument for keeping some swap is
  that it lets the kernel reclaim genuinely cold anonymous pages instead of
  OOM-killing; the argument against is unpredictable latency and that
  memory limits become fuzzy.)
- What's the difference between RSS, PSS, and WSS? (Resident, proportional
  — shared pages divided among sharers — and working set, the pages actually
  touched recently. Container memory accounting uses cgroup counters that
  include page cache attributable to the cgroup, which is why a container
  doing heavy file I/O can appear to approach its limit without leaking.)

---

### A3. Load average

**Answer.**
On Linux, load average is the exponentially-damped moving average (1, 5, 15
minute) of the number of tasks in state **`TASK_RUNNING` (running or
runnable) *plus* `TASK_UNINTERRUPTIBLE` (D state)**.

That D-state inclusion is the Linux-specific part. Other Unixes count only
runnable tasks, so their load average is a pure CPU-demand metric. Linux's
includes tasks blocked in uninterruptible sleep — typically waiting on disk
I/O, but also on NFS, on some kernel locks, or on a stuck device driver.

Consequences:
- A load average of 50 on a 16-core box might mean CPU saturation, or it
  might mean 50 processes stuck waiting on a hung NFS mount while the CPUs
  are idle. **The number alone doesn't tell you which**, and that's the
  entire point of the question.
- You must normalise by core count. Load 8 on 8 cores is fully utilised;
  on 64 cores it's nearly idle.
- The 1/5/15 averaging means it lags. A spike that started 20 seconds ago
  barely shows in the 1-minute figure.

What to use instead: **PSI (pressure stall information)**,
`/proc/pressure/{cpu,memory,io}`, which directly reports the fraction of
time tasks were stalled waiting for each resource. It separates the three
causes that load average conflates, and it's the metric to reach for on any
modern kernel. Failing that: run queue length (`vmstat` `r` column), `%iowait`,
and D-state process counts separately.

**Weak answers miss.** D state. It's the whole question.

**Follow-ups to expect.**
- How do you find the D-state processes? (`ps -eo state,pid,cmd | grep '^D'`,
  then `/proc/<pid>/stack` or `/proc/<pid>/wchan` for what they're blocked
  on.)
- Why does PSI distinguish "some" from "full"? ("Some" = at least one task
  stalled — a latency signal; "full" = *all* non-idle tasks stalled — a
  throughput-loss signal. Different alerting semantics.)

---

### A4. Zombies, orphans, and PID 1 in containers

**Answer.**
A **zombie** is a process that has exited but whose exit status hasn't been
collected. The kernel keeps a minimal entry (the PID and exit status) until
the parent calls `wait()`/`waitpid()`. A zombie consumes no memory or CPU —
only a PID slot. A few are normal and transient; a growing count means the
parent isn't reaping, and you eventually exhaust the PID space
(`kernel.pid_max`), at which point *nothing on the box can fork*.

An **orphan** is a process whose parent exited first. The kernel reparents
it — historically to PID 1, and since Linux 3.4 optionally to the nearest
ancestor marked as a "child subreaper" (`prctl(PR_SET_CHILD_SUBREAPER)`),
which is how process supervisors keep their descendants. PID 1's traditional
job is to reap these adopted orphans.

**Why containers care**: in a container, your application is usually PID 1.
Two things break if it wasn't written for that role:

1. **It doesn't reap.** Any orphaned grandchildren accumulate as zombies
   inside the container's PID namespace, and nothing cleans them up. A
   long-running container that spawns subprocesses (a shell wrapper, a
   sidecar that runs commands, a CI runner) will eventually fill the PID
   table.
2. **Signal handling changes.** PID 1 has special semantics: the kernel does
   **not** apply default signal dispositions to it. If your process has no
   explicit `SIGTERM` handler, the default "terminate" action does not
   apply, so `docker stop` / Kubernetes graceful shutdown sends `SIGTERM`,
   nothing happens, and 30 seconds later it's `SIGKILL`ed. Result: every
   shutdown is ungraceful, in-flight requests are dropped, and connection
   draining (topic 03, A3) never runs.

There's a third, related trap: `ENTRYPOINT ["sh", "-c", "myapp"]` makes the
shell PID 1, and many shells don't forward signals to their child at all.
Use exec form, or `exec myapp` in the script.

The fix is either to handle `SIGTERM` and reap children explicitly in your
app, or to use a minimal init as PID 1 — `tini`, `dumb-init`, or
`--init` / Kubernetes' `shareProcessNamespace` pause-container arrangement.

**Weak answers miss.** The PID 1 signal-disposition rule. That's the one
that causes real production impact, and it's the reason this is an SRE
question rather than an OS-trivia question.

**Follow-ups to expect.**
- How do you kill a zombie? (You can't — it's already dead. Kill or fix the
  parent; if the parent dies, init reaps the zombie.)
- What is `kernel.pid_max` and why might you raise it? (Default is often
  32768 on older systems, up to ~4 million on 64-bit. A high-density node
  running many containers can exhaust it, and the failure — `fork: retry:
  Resource temporarily unavailable` — looks like a memory problem.)

---

### A5. Namespaces

**Answer.**
| Namespace | Isolates |
| --- | --- |
| **mnt** | Mount table — the filesystem view |
| **pid** | Process IDs; the first process in a new PID ns is PID 1 |
| **net** | Network interfaces, routing tables, iptables rules, sockets, ports |
| **ipc** | System V IPC and POSIX message queues |
| **uts** | Hostname and NIS domain name |
| **user** | UID/GID mappings — enables rootless containers |
| **cgroup** | The cgroup root as seen by the process |
| **time** | `CLOCK_MONOTONIC` and `CLOCK_BOOTTIME` offsets (Linux 5.6+) |

Points worth making beyond the table:

- **Namespaces isolate *visibility*; cgroups limit *consumption*.** A
  container is namespaces + cgroups + (usually) a layered filesystem +
  seccomp/capabilities/LSM. Saying only "namespaces" is an incomplete
  definition of a container, and interviewers use that to separate people
  who've read a blog post from people who've built one.
- **The user namespace is the security-relevant one.** It maps UID 0 inside
  to an unprivileged UID outside, so a root process in the container is
  not root on the host. Without it, "root in container" is root on the host
  for anything not blocked by capabilities/seccomp — which is why the
  default Docker configuration drops capabilities rather than relying on
  isolation.
- **What is *not* namespaced**: the kernel itself, `/proc/sys` (mostly —
  some sysctls are namespaced, most are not), the clock (until the time
  namespace, and even then only monotonic offsets, not wall clock), kernel
  modules, and hardware. This is the fundamental reason containers are a
  weaker isolation boundary than VMs, and it's the right answer to "is a
  Kubernetes namespace a security boundary?" (topic 08).

**Weak answers miss.** The namespaces-vs-cgroups distinction and the "what
isn't namespaced" list.

**Follow-ups to expect.**
- How does a CNI plugin use the net namespace? (Creates a veth pair, moves
  one end into the pod's netns, configures addressing/routes — that's
  essentially the whole mechanism.)
- What's `nsenter` and when would you use it? (Enter an existing namespace
  from the host — the standard way to run `tcpdump` or `ss` "inside" a pod
  whose image has no tooling. Extremely practical, and knowing it signals
  hands-on experience.)

---

### A6. Voluntary vs involuntary context switches

**Answer.**
**Voluntary** (`voluntary_ctxt_switches` in `/proc/<pid>/status`): the task
gave up the CPU itself, because it blocked — waiting on I/O, a lock, a
condition variable, a socket read, a sleep. It called into the kernel and
the kernel descheduled it.

**Involuntary** (`nonvoluntary_ctxt_switches`): the scheduler took the CPU
away — the task's timeslice expired, or a higher-priority task became
runnable, or (in a container) CFS bandwidth throttling kicked in.

What each tells you:
- **High voluntary rate** = your workload is I/O- or lock-bound. If it's
  much higher than your I/O rate, suspect lock contention: threads
  repeatedly blocking on a mutex. The fix is in the application (reduce
  critical sections, shard the lock, use lock-free structures), not in the
  kernel.
- **High involuntary rate** = **CPU contention**. More runnable threads than
  cores, so the scheduler is time-slicing. Either the box is oversubscribed,
  or the process has far more runnable threads than available CPUs (a
  thread-per-request server with a 500-thread pool on 8 cores), or cgroup
  throttling is preempting it (A8).

The practical use: it's a cheap, always-available discriminator between
"waiting on something" and "competing for CPU," and both look like
"the service is slow" from outside. `pidstat -w`, `/proc/<pid>/status`, and
`perf sched` give you the data; scheduler latency (time from runnable to
running) via `perf sched latency` or a bpftrace `runqlat` is the direct
measurement of the second case.

**Weak answers miss.** That high voluntary switches with low I/O implies
lock contention — the inference is what makes the metric useful.

**Follow-ups to expect.**
- How expensive is a context switch? (Order of a few microseconds direct
  cost, but the real cost is cache and TLB pollution, which can be much
  larger and doesn't show in the switch count. Say that the indirect cost
  dominates.)

---

## Tier 2 — Explain / compare

### A7. OOM killer, cgroup v2, `memory.high` vs `memory.max`

**Answer.**
**Global OOM killer**: when the kernel cannot satisfy an allocation and
reclaim has failed, it picks a victim. The score is driven by
`oom_score_adj` (settable per process, −1000 to +1000) combined with the
process's memory footprint — roughly, it prefers to kill the process whose
death frees the most memory, adjusted by the tunable. `−1000` makes a
process OOM-immune (used for critical daemons like sshd or kubelet). The
kill is a `SIGKILL`: no cleanup, no flush, no graceful shutdown.

**Under cgroup v2**, the interesting behaviour is per-cgroup and the knobs
are different:

- **`memory.max`** — the hard limit. Allocation beyond it triggers reclaim;
  if reclaim can't free enough, the **cgroup OOM killer** kills something
  *inside that cgroup*. This is what a Kubernetes memory `limit` maps to,
  and it's why an OOMKilled container doesn't take down its neighbours.
- **`memory.high`** — the *throttling* threshold. Exceeding it doesn't kill
  anything; the kernel aggressively reclaims and **throttles the
  allocating process** by injecting sleeps proportional to the overage. The
  workload gets slow rather than dead. This is a genuinely better mechanism
  for most services — it converts a hard failure into backpressure and gives
  you time to react — and it's the main practical reason to care about
  cgroup v2.
- **`memory.low`** / **`memory.min`** — reclaim protection. Memory under
  `min` is never reclaimed (can trigger OOM instead); memory under `low` is
  reclaimed only under pressure. Use these to protect a cache or a critical
  service from being squeezed by noisy neighbours.
- **`memory.oom.group`** — kill the whole cgroup as a unit rather than one
  process, which avoids the state where the container survives with half its
  processes dead. Very often what you actually want.

Other v2 improvements worth naming: a **unified hierarchy** (v1 had separate
trees per controller, so a process could be in inconsistent positions),
proper **PSI** per cgroup, and much better accounting of page cache and
kernel memory to the cgroup that caused it.

The operational gap most teams have: **`memory.high` is rarely used**,
because Kubernetes maps `limits.memory` to `memory.max` only. Which means
the default experience is a hard kill with no warning. Recognising that gap
and knowing you can set `memory.high` (via a runtime class, a custom
runtime config, or directly) is a strong signal.

**Weak answers miss.** `memory.high` entirely. Also missed: that an
OOMKill is a `SIGKILL`, so no graceful shutdown, no draining, and any
in-flight work is lost — which matters for how you design shutdown.

**Follow-ups to expect.**
- Your container is OOMKilled but the app reports low heap usage. Why?
  (Off-heap: mmap'd files, thread stacks, JNI/native allocations, the JVM's
  metaspace and code cache — and page cache attributed to the cgroup from
  file I/O. Container memory ≠ heap, and the JVM's own `-Xmx` doesn't bound
  it. `MaxRAMPercentage` and container-awareness flags exist for this.)
- Why is setting a memory limit equal to the request the safest
  configuration? (Guaranteed QoS class in Kubernetes — the pod is last to be
  evicted under node pressure. Trade utilisation for predictability.)

---

### A8. CFS throttling and tail latency at low average CPU

**Answer.**
This is the highest-value question in the topic because almost every
Kubernetes user has been burned by it and most don't know why.

**The mechanism.** A CPU limit is implemented by CFS bandwidth control with
two parameters: `cpu.cfs_period_us` (default **100 ms**) and
`cpu.cfs_quota_us`. A limit of "1 CPU" means quota = 100 ms per 100 ms
period. The cgroup receives that quota at the start of each period; when it
is exhausted, **every thread in the cgroup is descheduled until the next
period boundary**.

Now the arithmetic that explains the symptom. Suppose a service has a limit
of 1 CPU, and it handles a request using 4 threads that each need 25 ms of
CPU. That's 100 ms of CPU consumed in 25 ms of wall clock — the quota for
the entire period, burned in a quarter of it. The container is then frozen
for **75 ms**. Average utilisation over the period: 100%... but average over
a minute, if requests arrive a few times a second, might be 30%.

Every request unlucky enough to land in a throttled window eats up to a full
period of added latency. p50 is fine because most requests don't hit it. p99
is catastrophic. **Average CPU utilisation is structurally incapable of
showing you this** — it's averaged over a window vastly longer than the
100 ms period where the damage happens.

**What to measure**: `container_cpu_cfs_throttled_periods_total` /
`container_cpu_cfs_periods_total` — the *fraction of periods in which
throttling occurred*. Not the total throttled seconds, which looks small and
reassuring. If more than a few percent of periods are throttled, you have a
latency problem regardless of what utilisation says. This single ratio
belongs on every service dashboard.

**Why it's worse than it sounds:**
- **Parallelism multiplies it.** A runtime that sees the host's core count
  (Go's `GOMAXPROCS`, a JVM sizing its thread pools, a thread-per-core
  framework) will spawn 64 workers on a 64-core node while limited to 1 CPU.
  All 64 threads burn the quota almost instantly. Fixes: `automaxprocs` for
  Go, `-XX:ActiveProcessorCount` / container awareness for the JVM, or
  explicit pool sizing. This is one of the most common and most impactful
  misconfigurations in Kubernetes.
- There were historical CFS bandwidth accounting bugs that caused throttling
  well below the quota, fixed in the 5.x series — worth knowing the kernel
  version matters here, but don't blame kernel bugs before checking your
  thread counts.

**What I'd do:**
1. Measure the throttled-period ratio before anything else.
2. Fix runtime parallelism to match the limit.
3. Consider **removing CPU limits** and relying on **requests** alone.
   Requests set `cpu.shares`/`cpu.weight`, which is *proportional* — under
   contention you get at least your share, and when the node is idle you can
   burst freely with no throttling. This is a legitimate and increasingly
   common recommendation for latency-sensitive services. The cost you accept:
   no hard isolation, so a runaway process can consume idle capacity and
   your performance becomes dependent on neighbours; and capacity planning
   gets harder because you can't reason about a fixed ceiling. State that
   tradeoff explicitly rather than presenting it as a free win.
4. If you must keep limits, set them generously above the p99 burst, not at
   the average.
5. For genuinely latency-critical workloads, CPU pinning (static CPU manager
   policy) removes both throttling and cross-core interference, at the cost
   of utilisation.

**Weak answers miss.** The 100 ms period and the burst arithmetic —
without those numbers the answer is a vague "throttling is bad." Also
missed: `GOMAXPROCS`/JVM core detection, which is usually the actual root
cause in the field.

**Follow-ups to expect.**
- Should you ever set CPU limits? (Yes — for batch/untrusted/multi-tenant
  workloads where you need a hard ceiling for isolation or billing. The
  question is whether the workload is latency-sensitive.)
- What does `cpu.weight` do when the node isn't contended? (Nothing — shares
  only bind under contention. Which is exactly why requests-without-limits
  works well on a node with headroom and degrades predictably without it.)
- How does this interact with the HPA? (Badly: if you scale on CPU
  utilisation-against-request while being throttled, the metric under-reports
  demand and you scale too late. Another argument for scaling on a
  work-related signal — queue depth, RPS, concurrency.)

---

### A9. strace vs perf vs eBPF

**Answer.**
**`strace`** — traces syscalls using `ptrace`. Every traced syscall causes
the target to **stop, context-switch to the tracer, and resume**: two extra
context switches plus signal delivery per syscall. Overhead is commonly
cited in the range of 100x+ slowdown for syscall-heavy processes; even for
moderate workloads it's severe. It also *stops the process* on attach, and
if `strace` dies at the wrong moment it can leave the target stopped.

Use it: on a development box, on a single low-traffic process, or when you
genuinely need the full syscall arguments and return values for one specific
call and can afford the hit. Never attach it to a busy production process
serving traffic — it is one of the few tools that can turn an investigation
into an outage.

**`perf`** — sampling profiler built on hardware performance counters and
kernel tracepoints. It periodically interrupts and records the stack, so
overhead is a function of sample rate (99 Hz is typical) and is generally
low single-digit percent. It gives you *where CPU time goes*, across kernel
and userspace, and it's the right tool for "the process is at 100% CPU"
(A14). Also does off-CPU analysis via scheduler tracepoints, PMU counters
(cache misses, branch mispredicts, IPC), and `perf trace` as a lower-overhead
strace alternative.
Limitations: sampling misses rare events by construction, symbols require
debug info or frame pointers (which are frequently compiled away — hence the
recent push to re-enable frame pointers in distributions), and the raw data
volume is large.

**eBPF / bpftrace** — a sandboxed program running **in the kernel**,
attached to a probe point, doing aggregation in kernel space and exporting
only summaries. The overhead is per-event and small (typically tens to
hundreds of nanoseconds), and — critically — you never copy the raw events
to userspace. A histogram of syscall latency for the whole system costs a
few percent, where `strace` doing the same thing costs orders of magnitude
more.
It's also **programmable and safe**: the verifier guarantees the program
terminates and can't crash the kernel, which is what makes it acceptable to
run on production. And it's *always-on* capable — you can leave it running
across a fleet, which neither of the others can.
Limitations: needs a reasonably modern kernel and CO-RE/BTF for portability
across kernel versions (otherwise you're compiling per-kernel — the problem
libbpf CO-RE was built to solve); the verifier constrains what you can write
(A10); userspace stack unwinding is still awkward for JIT'd runtimes; and
there's a real skill floor.

**How I'd choose**: `perf` first for CPU-bound questions (cheap, no code to
write, immediately gives you a flame graph). `bpftrace` for anything about
latency distributions, off-CPU time, or a specific kernel event —
`biolatency`, `runqlat`, `execsnoop` answer most questions in one line.
`strace` last, on a single instance you can afford to slow down, when you
need exact syscall arguments.

**Weak answers miss.** *Why* strace is expensive (ptrace stop/resume per
syscall) rather than just "it's slow," and eBPF's in-kernel aggregation as
the reason it's cheap. Given eBPF is on your resume, this answer needs to be
specific.

**Follow-ups to expect.**
- How does `strace -f -e trace=openat` compare in cost? (Filtering with
  `-e` still uses ptrace stops for all syscalls in older versions; seccomp-bpf
  filtering (`--seccomp-bpf`) lets the kernel skip stopping for uninteresting
  syscalls, which is a large improvement. Knowing this distinguishes you.)
- What's CO-RE and why does it matter for a fleet? (Compile Once, Run
  Everywhere: BTF type information lets one compiled program relocate field
  offsets across kernel versions at load time, so you ship one binary to
  3000 heterogeneous nodes instead of compiling on each. This is the thing
  that made fleet-wide eBPF operationally viable.)

---

### A10. eBPF attach points and verifier constraints

**Answer.**
**Attach points**, grouped by what they observe:

*Kernel-side:*
- **kprobe / kretprobe** — any kernel function entry/return. Maximum
  coverage, but function names and signatures are unstable across kernel
  versions, so a probe on an internal function is a maintenance liability.
- **fentry / fexit** — the modern, faster replacement using BPF trampolines;
  lower overhead than kprobes and gives typed access to arguments and return
  values. Prefer these on kernels that support them.
- **tracepoints** — static, kernel-maintained instrumentation points with a
  stable ABI. Use these whenever one exists; they're the difference between
  a tool that works across your fleet and one that breaks on the next kernel
  upgrade.
- **raw tracepoints** — lower overhead than tracepoints, less argument
  processing.
- **LSM hooks** — for security enforcement (the basis of tools like
  Tetragon/Falco's newer backends).

*Userspace:*
- **uprobe / uretprobe** — a userspace function entry/return. How you trace
  a library or an application without modifying it. Higher overhead than
  kernel probes (they trap), and they need symbol resolution.
- **USDT** — statically defined tracepoints compiled into applications.

*Networking:*
- **XDP** — at the driver, *before* `sk_buff` allocation. The earliest and
  fastest point; used for DDoS filtering and L4 load balancing (Katran).
  Can `XDP_DROP`, `XDP_TX`, `XDP_REDIRECT` at line rate. Constrained: no
  full skb metadata yet, limited to ingress.
- **tc (traffic control) ingress/egress** — after skb allocation, so more
  context available, both directions; where most CNI datapath logic lives
  (Cilium).
- **socket filters, cgroup/skb, sock_ops, sockmap** — socket-level hooks;
  cgroup-attached programs are how per-container network policy and
  socket-level load balancing work.

*Other:* perf events (sampling), cgroup hooks, and `struct_ops` (e.g.
pluggable TCP congestion control, and sched_ext for schedulers).

**Verifier constraints** — the reason eBPF is safe to run in production:

1. **Termination must be provable.** Originally: no loops at all. Since
   Linux 5.3 there are **bounded loops** the verifier can prove terminate,
   and there are `bpf_loop`/iterator helpers for larger iteration. You
   still cannot write an unbounded `while`.
2. **Instruction and complexity limits.** 1 million instructions verified
   (raised from 4096 for privileged programs); the verifier explores all
   paths, so complexity, not just size, is bounded. Deeply nested branching
   fails verification even when small.
3. **Memory safety.** Every pointer dereference must be provably in bounds.
   Packet access requires an explicit `data + offset <= data_end` check
   *before* the read, and the verifier tracks that. You cannot read
   arbitrary kernel memory — you use `bpf_probe_read_kernel()` which handles
   faults.
4. **Bounded stack**: 512 bytes. This is a real constraint — any sizeable
   buffer must live in a map, not on the stack.
5. **Limited helper functions.** You can only call a fixed, allowlisted set
   of kernel helpers, and which ones are available depends on the program
   type. A tracing program and an XDP program have different helper sets.
6. **No unbounded recursion**; tail calls are limited (33 deep) and
   historically restricted how they compose with function calls.
7. **Type/state tracking**: the verifier tracks register types and rejects
   e.g. arithmetic on pointers that could escape bounds, and it enforces
   that a map value pointer is null-checked before use — the single most
   common thing that fails verification when you're learning.

The framing to state: the verifier is what converts "arbitrary code in the
kernel" from insane to routine. Everything it rejects is something that
could hang or crash the kernel, and the cost is that eBPF is not a general
programming environment — it's a constrained one you design around.

**Weak answers miss.** Bounded loops (many people still say "no loops",
which has been wrong for years), the 512-byte stack, and the packet-bounds
check requirement. Given this is on your resume, the interviewer will
expect you to have hit these personally — have a specific story about a
program the verifier rejected and how you restructured it.

**Follow-ups to expect.**
- Why is XDP faster than tc? (Runs in the driver before `sk_buff`
  allocation — you skip the most expensive part of receive processing. With
  a native-mode driver it's before almost the entire stack.)
- How do you get data out to userspace? (Maps for aggregation — the
  preferred approach, since you export summaries; perf buffer or the newer
  **ring buffer** (5.8+, single shared buffer with better ordering and
  memory efficiency) for events. Say that in-kernel aggregation is what
  makes it cheap.)
- How do you handle kernel version differences across a heterogeneous
  fleet? (CO-RE + BTF; prefer tracepoints over kprobes; `bpf_core_read` for
  field relocation; and feature-detect at load time with a fallback.)

---

### A11. Threads vs epoll vs io_uring

**Answer.**
**Blocking I/O with a thread per connection.** Simplest model — the code
reads top to bottom and the kernel scheduler handles concurrency. Costs:
each thread has a stack (default 8 MB virtual, typically much less resident)
and a kernel task struct; at ten thousand connections you have ten thousand
threads, and the scheduler is spending its time on context switches and
cache thrash rather than work. Fine to a few thousand connections; the
model that "C10K" was written about.

**epoll event loop.** One (or a few) threads call `epoll_wait()` to learn
which of N file descriptors are ready, then do non-blocking reads/writes.
Memory is O(connections) in small per-connection state rather than stacks,
and `epoll` itself is O(ready) rather than O(watched) — which is precisely
what `select`/`poll` got wrong (they rescan the whole set every call).
Costs: **two syscalls per I/O operation** (the readiness notification plus
the actual read/write), an inverted control flow that's harder to write and
harder to debug, and a hard rule that you must never block in the loop —
one blocking `getaddrinfo` or a synchronous disk read stalls every
connection. Also, `epoll` is a *readiness* interface, which works well for
sockets and badly for regular files: a regular file is always "ready," so
buffered file I/O in an event loop still blocks. That's why event-loop
servers historically used a thread pool for disk I/O.

**io_uring.** Two shared-memory ring buffers (submission and completion)
between userspace and kernel. You write submission queue entries and the
kernel writes completions; with `SQPOLL` a kernel thread polls the
submission queue so you can perform I/O with **zero syscalls** in the steady
state. Key differences from epoll:
- It's a **completion** interface, not a readiness one — you ask for the
  operation and get the result, so it works for regular files, not just
  sockets.
- **Batching**: submit many operations in one syscall.
- Broad operation coverage: read, write, accept, connect, send/recv,
  openat, statx, fsync, timeouts — and **linked operations** so you can
  express "accept then read" without a round trip to userspace.
- Registered buffers and files avoid per-operation reference counting and
  page pinning.

Costs of io_uring: relatively young, so it needs a modern kernel and the
feature set varies significantly by version; it has had a notable stream of
security issues, to the point that some environments (including Google and
some container platforms) restrict or disable it — worth knowing, because
"we'll use io_uring" can be blocked by policy; and the programming model is
harder still than epoll. The performance win is largest for
high-syscall-rate workloads (many small operations) and smaller when
per-operation work dominates.

Decision: threads until you have thousands of concurrent connections or a
measured scheduler problem; epoll (or a runtime built on it — Go's netpoller,
Tokio, libuv) as the default for network servers; io_uring when the syscall
rate itself is the bottleneck, or when you need true async file I/O, and the
kernel and security policy allow it.

**Weak answers miss.** The readiness-vs-completion distinction, which is
what makes io_uring qualitatively different rather than "epoll but faster,"
and epoll's regular-file blind spot.

**Follow-ups to expect.**
- Where does Go's runtime sit? (Goroutines over an epoll-based netpoller —
  it gives you the blocking programming model with event-loop efficiency.
  Note that blocking *syscalls* still consume an OS thread, which is why
  `GOMAXPROCS` and the thread count can grow under heavy file I/O.)
- What is thundering herd in epoll and how is it addressed? (`EPOLLEXCLUSIVE`,
  and `SO_REUSEPORT` for accept distribution across processes.)

---

### A12. IOPS, throughput, latency, and queue depth

**Answer.**
- **IOPS** — operations per second. Bounded by per-operation overhead, so it
  dominates for small random I/O.
- **Throughput** — bytes per second. Bounded by the device's and the bus's
  bandwidth, so it dominates for large sequential I/O.
- **Latency** — time for one operation to complete. This is the one users
  feel, and it's the one that gets left out of capacity planning.

They're related by `throughput ≈ IOPS × block_size`, so quoting IOPS without
block size is meaningless — a device advertising 100k IOPS at 4 KB is
claiming 400 MB/s, and it will not do 100k IOPS at 128 KB.

**Queue depth** is the number of operations outstanding at once, and it's
where the relationship gets interesting. Little's law:
`concurrency = throughput × latency`. So:
- At **QD=1**, throughput is purely `1 / latency`. A device with 100 µs
  latency gives you 10,000 IOPS and nothing you do to the device changes
  that — you're latency-bound.
- **Increasing queue depth increases throughput** by exploiting internal
  parallelism (an NVMe SSD has many channels and dies; it *needs* deep
  queues to be busy). This is why NVMe exposes up to 64k queues of 64k
  entries versus SATA's single 32-entry queue.
- **Past the saturation point, throughput flattens and latency rises
  linearly.** Adding queue depth beyond saturation buys you nothing but
  waiting. This is the knee, and operating past it is the most common
  storage performance mistake — the device shows high utilisation and high
  IOPS, and p99 latency is terrible.

So the practical statement: **throughput and latency are not independent
goals; queue depth is the dial that trades one for the other.** Benchmark
with the block size, access pattern (random vs sequential), read/write mix,
and queue depth that match your workload, or the numbers are fiction. `fio`
with those four parameters is the tool.

Extra facts worth having: cloud block storage (EBS gp3/io2, GCP PD) has
provisioned IOPS *and* throughput limits, and you can hit either — plus a
per-instance aggregate limit that's separate from the volume's, which is a
frequent surprise. Local NVMe is an order of magnitude lower latency than
network-attached block storage (tens of µs vs sub-millisecond to
milliseconds), which is why databases and ClickHouse-style workloads care
about it so much.

**Weak answers miss.** Little's law and the knee — without them you can't
explain why a device at "80% utilisation" has bad tail latency.

**Follow-ups to expect.**
- Why is write amplification a thing on SSDs? (Erase blocks are much larger
  than pages; garbage collection rewrites live data. Consequences: sustained
  random write performance is much worse than burst, over-provisioning helps,
  and TRIM matters. Connects directly to LSM-tree design in the databases
  topic.)
- How do you measure whether the disk is the problem? (`biolatency` /
  `iostat -x` — look at `await` and queue size, not `%util`, which is
  meaningless on devices with internal parallelism.)

---

### A13. What `fsync()` guarantees

**Answer.**
`write()` returns when the data is in the **page cache** — a dirty page in
RAM. It is not durable. A power loss loses it. The kernel writes it back
later based on `vm.dirty_*` settings, typically within tens of seconds.

`fsync(fd)` blocks until the file's data **and metadata** have been handed
to the storage device and the device reports them as persisted. `fdatasync()`
is the cheaper variant that skips metadata not needed to read the data back
(it will still flush a size change, but not e.g. mtime).

The layers between `write()` and persistent media, each of which can lose
or reorder data:

1. **Application buffer** (stdio, a language runtime's writer). `fflush()`
   moves it to the kernel; it does not make it durable. Confusing `flush`
   with `fsync` is a real and common bug.
2. **Page cache.** Cleared by `fsync`.
3. **Filesystem journal.** Ordering and metadata consistency depend on the
   mount options — ext4's `data=ordered` vs `data=writeback` change what
   guarantees you get after a crash.
4. **Block layer / I/O scheduler**, which reorders requests. `fsync` issues
   a **cache flush / FUA** to enforce ordering.
5. **Device write cache.** SSDs and disks have volatile DRAM caches. The
   flush command is what forces it to media. **Some consumer devices lie
   about completing the flush** — this is a documented, real problem and it
   is why enterprise drives with power-loss protection (a capacitor that
   flushes the cache on power failure) exist and cost more.
6. **Virtualisation / network storage.** In a VM or on network-attached
   storage, `fsync` semantics depend on the hypervisor's cache mode and the
   storage backend. `cache=writeback` at the hypervisor means the guest's
   fsync may not reach durable media.

Two more traps worth naming:
- **Directory entries.** Creating a file and fsyncing it does not guarantee
  the *directory entry* survives a crash. You must `fsync` the containing
  directory too. This is why the durable-rename pattern is: write temp file
  → `fsync` temp → `rename` → **`fsync` the directory**.
- **fsync error handling.** Historically, a failed writeback could clear the
  error flag such that a subsequent `fsync` returned success — the
  "fsyncgate" problem that affected PostgreSQL. Modern kernels report the
  error once per file description, and the accepted answer in database
  circles is that an `fsync` failure is unrecoverable: you must crash and
  recover from the WAL, not retry. Knowing this is a strong signal.

**Weak answers miss.** The directory fsync, and that the device may have a
volatile cache. Both are the difference between "I know the API" and "I've
built something durable."

**Follow-ups to expect.**
- Why is `fsync` per-commit expensive for a database, and what's the fix?
  (It's a round trip to durable media — order of hundreds of µs on NVMe
  with PLP, milliseconds otherwise. Fix: group commit — batch many
  transactions into one fsync, trading a little latency for a lot of
  throughput. Every serious database does this.)
- `O_DIRECT` — what does it change? (Bypasses the page cache, so the
  application manages its own buffering and alignment. Databases use it to
  avoid double-buffering and to control their own caching. It does *not* by
  itself guarantee durability — you still need the device cache flushed,
  hence `O_DSYNC`/fsync.)

---

## Tier 3 — Scenario / debug

### A14. A process pegging one CPU

**Answer.**
Order matters: cheapest and least invasive first, and stop as soon as you
have the answer.

**1. Confirm it's user time, not system time.** `top -H -p <pid>` (per
thread) or `pidstat -t -p <pid> 1`. `%usr` vs `%sys` splits the problem
immediately:
- High `%sys` → the kernel is doing the work. Suspect a syscall storm, page
  faults, or spinning in a kernel path. Go to step 3 with a syscall focus.
- High `%usr` → application code. Go to step 2.
Also note it's *one* CPU: single-threaded work, or one hot thread among
many. `top -H` tells you which thread, and the thread name often names the
subsystem outright.

**2. Sample the stack. `perf top -p <pid>`** for a live view, or
`perf record -F 99 -g -p <pid> -- sleep 30` then `perf report` /
a flame graph. 99 Hz, so ~3000 samples in 30 seconds — enough to find
anything taking more than a percent or two, at negligible cost. This
answers "what code is running" definitively and it is the single highest-value
step. If symbols are missing, that's a build issue (frame pointers, debug
info, or a JIT'd runtime needing a symbol map — `perf-<pid>.map` for JVM/
Node).

**3. If it's system time, find the syscalls.** `perf trace -p <pid>` or
bpftrace `syscount -p <pid>` gives you a per-syscall count/latency histogram
at low cost. A spin on `futex` means lock contention; a flood of
`epoll_wait` returning immediately means a busy-loop bug; `clock_gettime` at
absurd rates means a hot logging path. Only if I need exact arguments for
one specific call do I reach for `strace -p <pid> -e trace=<call>` — and
briefly, knowing the cost (A9).

**4. Language-level tooling if the runtime has it.** `jstack`/async-profiler
for the JVM, `pprof` for Go (`/debug/pprof/profile` — already there if the
service exposes it), `py-spy` for Python. These give you semantic stacks
rather than native frames, and `py-spy`/async-profiler in particular are
low-overhead and safe to run in production. If the service already exposes
pprof, this is arguably step 1.5.

**5. Rule out the environment.** Is the process actually pegged, or is it
being throttled and appearing busy (A8)? Check the cgroup throttle counters.
Is another tenant stealing time (A15)? Check `%steal`. Is it in a spin
because of a dependency it's polling?

**What I'd say about method**: the flame graph from step 2 resolves the
majority of these in under a minute, so the discipline is to not skip it
in favour of guessing from logs. And I'd capture the profile *before*
restarting anything — a restart destroys the only evidence.

**Weak answers miss.** The usr/sys split as the first branch, and going to
`strace` before `perf`. Reaching for `strace` first on a busy production
process is a red flag.

**Follow-ups to expect.**
- What if the process is *not* on CPU but still slow? (Off-CPU analysis:
  `offcputime` from bcc, or `perf sched`. Different tool, and knowing that
  on-CPU profiling is blind to blocking is the point.)
- How do you profile something that only misbehaves for 5 seconds once an
  hour? (Continuous profiling — Parca/Pyroscope/Google-style always-on
  profiling with eBPF. Say that the answer is infrastructure, not a
  faster reaction time.)

---

### A15. iowait vs steal

**Answer.**
**`%iowait`** is the fraction of time a CPU was **idle with at least one
outstanding block I/O request on that CPU's run queue**. Three things follow
that people get wrong:
- It is a form of **idle** time. The CPU had nothing else to do. High
  iowait with a busy application means you're I/O-bound; high iowait on an
  otherwise-idle box means almost nothing.
- Conversely, **low iowait doesn't mean no I/O problem** — if the CPU has
  other runnable work, that time gets counted as user/system instead, and
  your I/O bottleneck is invisible in this metric.
- It's per-CPU and the aggregate is misleading on many-core boxes.

So `%iowait` is a hint, not a diagnosis. The real measurements are per-device:
`iostat -x` (`await`, `r_await`/`w_await`, average queue size — and
*not* `%util`, which is meaningless for devices with internal parallelism),
or `biolatency` for a proper distribution, plus `/proc/pressure/io` (PSI),
which is the cleanest signal available.

What to do about genuine I/O saturation: find the offender (`iotop`,
`biosnoop`, or per-cgroup I/O accounting via `io.stat` in cgroup v2), then
either reduce the I/O (indexing, caching, batching, compression), move to
faster storage (local NVMe vs network block), spread across more devices, or
throttle the offending workload with cgroup v2 `io.max`/`io.latency` — the
latter being the right tool for "a batch job is starving the database on a
shared node."

**`%steal`** is time the **hypervisor** ran something else while your vCPU
was runnable. It only exists on virtualised guests and it means you are not
getting the CPU you think you bought. Causes:
- **Oversubscription by the provider** — the physical host is overcommitted
  and neighbours are consuming it. Common on burstable/shared instance
  classes.
- **Your own credit exhaustion** on burstable instances (T-class): once
  CPU credits are gone you're throttled to a baseline, and it presents as
  steal. This is the most common cause in practice and it's self-inflicted.
- A noisy neighbour on a dedicated host you share.

What to do: check whether it's credits first (instance-level CPU credit
metrics) — if so, resize to a non-burstable class or accept the baseline. If
it's genuine host contention, the levers are: move the instance (stop/start
usually lands you on different hardware), move to a larger instance size
(larger sizes are frequently less oversubscribed), dedicated hosts/instances,
or a different instance family. Sustained double-digit steal on production
capacity is a reason to escalate to the provider with data.

**The connection worth drawing**: both metrics describe *time you didn't get
to run*, from two different causes — one below you (storage), one beside you
(the hypervisor). And both are why "CPU utilisation looks fine" is such an
unreliable statement on virtualised infrastructure. PSI and per-device
latency are what you should actually be alerting on.

**Weak answers miss.** That iowait is idle time, and that burst credits are
the usual cause of steal in the cloud rather than an evil neighbour.

**Follow-ups to expect.**
- You see steal on a bare-metal box. Explain. (You shouldn't — no
  hypervisor. If you do, something is virtualising you that you didn't know
  about, or it's a nested/container-runtime artifact. It's a good sanity
  check on your inventory.)
- How would you tell iowait from lock contention? Both look like "not
  running." (D-state count and PSI-io for the first, futex syscall rate and
  voluntary context switches for the second — A6.)

---

### A16. p99 at 40x p50 with no obvious pressure

**Answer.**
The framing first: a p99/p50 ratio of 40 means the slow requests are not
doing 40x more work — they're **waiting** on something the fast ones aren't.
So the question is "what queue are they in?" And "no obvious CPU or memory
pressure" is expected, because averages hide everything that happens on a
100 ms timescale.

Hypotheses, roughly in order of how often they're the answer:

1. **Garbage collection / runtime pauses.** A stop-the-world pause hits
   whatever requests happen to be in flight. Signature: latency spikes
   correlate exactly with GC events, and they hit *all* concurrent requests
   at once. Check GC logs, pause duration histograms, allocation rate.
   Fixes: reduce allocation, tune the collector, or move to one with
   bounded pauses.
2. **CFS throttling** (A8). The signature is periodic ~100 ms-scale stalls
   and a nonzero throttled-period ratio while average CPU looks low. This is
   the single most common cause in a Kubernetes environment and is
   *specifically invisible* to average-utilisation dashboards, which is why
   the question says "no obvious CPU pressure."
3. **Lock contention.** A few requests take a contended lock behind a slow
   holder. Signature: high voluntary context switches (A6), high futex
   rates, and the tail grows super-linearly with concurrency. Measure with
   off-CPU profiling / `offcputime`.
4. **Queueing at a downstream dependency.** Little's law: if a dependency is
   at 80% utilisation, queueing delay is already several times service time,
   and at 95% it's catastrophic. Signature: tail correlates with dependency
   utilisation, not with your own. Includes connection pool waits —
   check whether the p99 is spent *acquiring* a connection rather than
   using one, which requires the client to instrument those separately.
5. **Tail amplification from fan-out.** If a request fans out to 10 backends
   and waits for all, the request's latency is the *max* of 10 samples — so
   with a per-backend p99 of 100 ms, roughly 1 − 0.99¹⁰ ≈ 10% of requests
   see ≥100 ms. Fan-out converts a backend p99 into a frontend p90. This is
   the arithmetic to say out loud; it explains tails that no individual
   service seems responsible for.
6. **Cold caches / cache misses.** The 1% that miss pay the full backend
   cost. Signature: tail latency ≈ backend latency, and it tracks hit rate.
7. **Storage tail** — a single slow device, a compaction, an SSD garbage
   collection burst, or a network storage hiccup. `biolatency` shows a
   bimodal distribution.
8. **Network retransmissions** — a 200 ms+ RTO on a dropped packet lands
   squarely in the tail (topic 01).
9. **Head-of-line blocking** at the application: a single-threaded worker,
   an event loop with an occasional blocking call, or an unbounded queue in
   front of a fixed pool.
10. **Load imbalance.** Not all instances are equal — one hot instance
    serving 5% of traffic badly produces exactly this. Break the histogram
    down *by instance* before anything else; if one pod owns the tail,
    you've turned a hard problem into an easy one.

**Method I'd actually run**: (a) break the latency histogram down by
instance, endpoint, and tenant — one of those three dimensions usually
localises it immediately; (b) get a distributed trace of a slow request and
find which span holds the time, since that eliminates most of the list in
one look; (c) if the time is unattributed within a span, it's the runtime or
the scheduler, and I go to GC logs plus throttling counters plus off-CPU
profiling.

The thing I'd insist on: **never average percentiles across instances.**
Averaging p99s is mathematically meaningless and will hide a single bad
instance. You need histograms that can be merged (Prometheus histogram
buckets, or a mergeable sketch like t-digest/DDSketch), not pre-computed
quantiles.

**Weak answers miss.** The fan-out arithmetic and the "break down by
instance first" move. Also missed: connection-pool acquisition time, which
is invisible unless someone deliberately instrumented it.

**Follow-ups to expect.**
- How would hedged requests help, and what do they cost? (Send a duplicate
  after p95 elapses; cuts the tail dramatically when it's caused by random
  per-server variance. Costs extra load — cap it at a few percent — and
  requires idempotency.)
- What if the tail is caused by one tenant? (Now it's an isolation problem:
  per-tenant concurrency limits, shuffle sharding, or cells. Different topic,
  right instinct.)

---

### A17. Dropping connections at accept, below CPU limit

**Answer.**
The clue is "at the accept path" and "below CPU" — the application isn't the
bottleneck; something in the kernel's connection-establishment path is.
There are several distinct queues, and naming them separately is the answer.

**1. The SYN queue (`tcp_max_syn_backlog`).** Half-open connections awaiting
the final ACK. Overflow means SYNs are dropped, and clients see connection
timeouts (retried at 1 s, 3 s, 7 s — the multi-second stalls from topic 01
A16). Counter: `netstat -s | grep -i "SYNs to LISTEN"`. If this is
overflowing under legitimate load, raise `net.ipv4.tcp_max_syn_backlog`; if
it's an attack, syncookies (topic 01, A14).

**2. The accept queue (`somaxconn` and the `backlog` argument to
`listen()`).** Fully-established connections waiting for the application to
call `accept()`. **The effective size is `min(backlog, somaxconn)`** — so
raising `net.core.somaxconn` alone does nothing if the application passes a
small backlog, which is the single most common mistake here. Many frameworks
default to 128 or 511 and never expose it.
Overflow behaviour: by default the kernel **silently drops the ACK**, so the
client believes the connection is established and the server retransmits
SYN-ACK — producing a long stall rather than a clean error. With
`net.ipv4.tcp_abort_on_overflow=1` it sends a RST instead, which fails fast
but is usually worse for users. Counter: `netstat -s | grep -i "listen
queue"` / `ListenOverflows`, `ListenDrops`. `ss -lnt` shows `Recv-Q` (current
accept queue depth) and `Send-Q` (its maximum) for listening sockets — that
one command answers this question directly.
Root cause is usually that the application isn't accepting fast enough: a
single accept thread, an event loop blocked on something, or the accept
happening on the same thread as request handling.

**3. Ephemeral port / conntrack exhaustion.** If there's a NAT or a
stateful firewall in the path (including Docker's default bridge networking
or kube-proxy's iptables), `nf_conntrack_max` can fill. Symptom:
`nf_conntrack: table full, dropping packet` in dmesg, and dropped
connections with no application involvement. Very common with high
connection churn. Check `/proc/sys/net/netfilter/nf_conntrack_count` against
`_max`, and note that short-lived connections occupy conntrack entries for
`nf_conntrack_tcp_timeout_time_wait` after closing — the same
rate × timeout arithmetic as topic 01 A17.

**4. File descriptor limits.** `accept()` returns `EMFILE`, and a naive
accept loop that doesn't handle it spins. Check the process's
`ulimit -n` / systemd `LimitNOFILE` and `/proc/<pid>/limits` — not the
shell's limits, which are usually different. Also check for a descriptor
leak (`CLOSE_WAIT` growth — topic 01, A4).

**5. Ephemeral port exhaustion on the *outbound* side**, if the service also
makes connections. Different symptom, same investigation.

**6. Softirq / packet processing saturation on a single core.** If RSS isn't
distributing interrupts, one core handles all network softirq and saturates
while the box looks idle in aggregate. Check `/proc/softirqs` for imbalance,
`mpstat -P ALL` for one hot core, and `ethtool -S` for `rx_dropped` /
ring-buffer overruns. Fixes: RSS queue count, RPS/RFS, IRQ affinity, larger
ring buffers.

**What I'd check, in one pass**: `ss -lnt` (accept queue depth vs max),
`nstat`/`netstat -s` deltas for `ListenOverflows` and SYN drops, conntrack
count vs max, `/proc/<pid>/limits`, and `/proc/softirqs`. That's five
commands and it covers everything above.

And the structural fix behind most of these: **the application should
accept faster** — multiple accept threads or `SO_REUSEPORT` with one
listening socket per worker, which lets the kernel distribute incoming
connections across processes and removes the single-accept-queue bottleneck
entirely.

**Weak answers miss.** `min(backlog, somaxconn)` and the silent-drop
behaviour on accept-queue overflow. Also missed: conntrack, which is the
answer surprisingly often in containerised environments.

**Follow-ups to expect.**
- Why is a *large* accept queue not obviously good? (It converts a fast
  rejection into a long wait. A connection sitting in the accept queue for
  3 seconds will be served after the client has given up. Bounded queues
  plus shedding beat deep queues — the same argument as topic 03, A14.)
- How does `SO_REUSEPORT` change load distribution? (Kernel hashes the
  4-tuple to a listening socket. Note the classic gotcha: adding or removing
  a listener rehashes, and in-flight connections in a removed socket's queue
  are dropped — which matters during rolling restarts.)

---

### A18. Fleet-wide file-write attribution with eBPF

**Answer.**
The requirement — continuous, fleet-wide, negligible overhead — rules out
`strace`, auditd at volume, and anything that exports raw events per write.
The design has to do aggregation in the kernel and export summaries.

**Probe choice.** I want writes to a *path*, but the kernel's write path
deals in inodes and file structs, not paths — resolving a path on every
write is expensive and racy. Two options:

- **Attach at `vfs_write` / `vfs_writev`** (fentry, or a tracepoint if a
  suitable one exists on the target kernels) and filter on the file's
  **inode number and device** rather than the path. Resolve path → (dev,
  inode) once in userspace at startup, push it into a BPF map, and have the
  kernel program do an O(1) map lookup per write. This is cheap and exact.
  Handle the rename/recreate case by re-resolving periodically or by also
  watching the parent directory.
- If I need *prefix* matching over a directory tree rather than one file,
  walk the dentry parent chain in the probe — bounded loop, which the
  verifier permits with an explicit iteration cap — or filter on the
  superblock/mount to narrow first and do finer matching in userspace on a
  much smaller event stream.

**Aggregation.** A per-key hash map keyed by
`(pid, tgid, cgroup_id, dev, inode)` with values `{bytes, count}`. All
updates happen in-kernel; userspace reads and clears the map on an interval
(say 15–30 s). No per-event userspace copy at all, which is what keeps the
cost negligible — the marginal cost per write is a map lookup and an atomic
add, order of tens of nanoseconds.

**Identity.** Capture `bpf_get_current_cgroup_id()` alongside the PID, so
events survive process exit and map cleanly to a container/pod. Resolve
cgroup id → pod in userspace against the kubelet or the cgroup filesystem;
PID alone is useless in a container fleet because PIDs churn and are
namespaced. Also grab the comm and, if needed, the exe path once at
first-sight and cache it.

**Portability across 3000 nodes.** This is the part that separates a demo
from a fleet tool:
- **CO-RE with libbpf**, compiled once, relying on **BTF** for field
  relocation across kernel versions. Ship a single binary rather than
  compiling on each node (which is what BCC does and why BCC is a poor fleet
  choice — it needs clang and kernel headers on every node and burns
  memory/CPU at startup).
- Prefer **tracepoints/fentry over kprobes** on internal functions, because
  kprobe targets get renamed and inlined between versions.
- **Feature-detect at load** and degrade gracefully — a node whose kernel
  lacks BTF should log and disable rather than crash-loop.
- For nodes without BTF, BTFHub-style external BTF archives are the
  workaround; I'd rather set a minimum kernel version.

**Deployment.** A DaemonSet with `hostPID`, the required capabilities
(`CAP_BPF` + `CAP_PERFMON` on modern kernels, `CAP_SYS_ADMIN` on older
ones), and pinned maps if I want the program to survive agent restarts.
Export via Prometheus (a bounded number of series — see below) or push
aggregated records to a pipeline.

**The constraints I'd call out unprompted**, because they're where this goes
wrong:
1. **Cardinality.** Per-PID-per-inode metrics on 3000 nodes will destroy a
   Prometheus. Aggregate to (pod, path-of-interest) in the agent, cap the
   number of tracked keys with an LRU, and export a bounded series count.
   The eBPF part is the easy part; the cardinality budget is the design
   constraint (see the observability topic).
2. **Map size limits.** A hash map has a fixed max entries; decide the
   eviction policy deliberately rather than silently dropping.
3. **Verifier budget.** Keep the program simple — the dentry walk is the
   only risky part, and it needs a hard iteration bound.
4. **Overhead measurement.** I'd benchmark it: run a write-heavy synthetic
   with and without the probe and report the delta, because "negligible" has
   to be a measured claim on a fleet this size. A per-write map update
   should be well under a microsecond; if it isn't, something's wrong.
5. **Blast radius.** A bad eBPF program can't crash the kernel, but it can
   burn CPU on every write on every node simultaneously. Roll it out like
   any other fleet change — canary, then percentage rollout, with a kill
   switch.

**Weak answers miss.** In-kernel aggregation (proposing a perf-buffer event
per write is the wrong answer and the most common one), cgroup id for
container attribution, and the cardinality problem. Given this is your
resume territory, the interviewer is testing whether you've shipped one or
prototyped one.

**Follow-ups to expect.**
- Why not auditd? (It can do this, but the overhead and event volume at
  write granularity across a fleet are prohibitive, and the userspace
  pipeline becomes the bottleneck. It's an event log, not an aggregator.)
- What if the file is written via `mmap` rather than `write`? (You'd miss it
  — `vfs_write` isn't in that path. You'd need page-fault or writeback
  probes, which is meaningfully harder. Saying this unprompted shows you
  know the limits of your own instrumentation.)
