// Lesson 13: Real BPF Code Patterns (Annotated)
//
// This file WON'T compile with gcc — it's BPF C with annotations.
// Read it as reference. Each pattern maps to what you'll write in process.bpf.h.
//
// To "run" this, just read it: gcc will fail because there's no vmlinux.h etc.
// Instead, compare it to your profiler.bpf.h and the bpf.old/ reference.

#if 0 // everything below is wrapped in #if 0 so gcc doesn't try to compile it

// ═══════════════════════════════════════════════
// PATTERN 1: Tracepoint handler skeleton
// ═══════════════════════════════════════════════

// SEC() tells the linker to put this function in a named ELF section.
// aya finds programs by section name and attaches them to tracepoints.
//
// "tp/" prefix = tracepoint (the ctx skips the 8-byte common header)
// "raw_tracepoint/" prefix = raw tracepoint (ctx has bpf_raw_tracepoint_args)

SEC("tp/sched/sched_process_exit")
int handle_sched_process_exit(void *ctx)
{
    // ctx is a raw pointer to the tracepoint data (after common header).
    // For sched_process_exit:
    //   offset 0:  char comm[16]
    //   offset 16: pid_t pid       (4 bytes)
    //   offset 20: int prio        (4 bytes)
    //
    // But we usually don't read from ctx for this tracepoint —
    // we use bpf helpers and CO-RE instead.

    return 0; // must return int
}


// ═══════════════════════════════════════════════
// PATTERN 2: Get current process info
// ═══════════════════════════════════════════════

SEC("tp/sched/sched_process_exit")
int example_get_info(void *ctx)
{
    // pid/tgid — single helper, two values packed in u64
    __u64 pid_tgid = bpf_get_current_pid_tgid();
    __u32 pid  = pid_tgid >> 32;    // what userspace calls "pid"
    __u32 tid  = (__u32)pid_tgid;   // thread id (less useful for us)

    // uid/gid — same pattern
    __u64 uid_gid = bpf_get_current_uid_gid();
    __u32 uid = (__u32)uid_gid;
    __u32 gid = uid_gid >> 32;

    // process name (comm) — fills char[16]
    char name[16];
    bpf_get_current_comm(name, sizeof(name));

    // timestamp
    __u64 ts = bpf_ktime_get_ns();

    return 0;
}


// ═══════════════════════════════════════════════
// PATTERN 3: CO-RE read from task_struct
// ═══════════════════════════════════════════════

SEC("tp/sched/sched_process_exit")
int example_core_read(void *ctx)
{
    // bpf_get_current_task_btf() returns struct task_struct *
    // with BTF type info — enables CO-RE reads
    struct task_struct *task = (struct task_struct *)bpf_get_current_task_btf();

    // BPF_CORE_READ reads a field with CO-RE relocation.
    // The compiler emits a relocation record: "I want field exit_code from task_struct"
    // At load time, aya patches the offset for the running kernel.
    __u32 exit_code = BPF_CORE_READ(task, exit_code);

    // Chained read: task->real_parent->tgid
    // Each -> is a separate kernel memory read
    __u32 ppid = BPF_CORE_READ(task, real_parent, tgid);

    // Without CO-RE (old aya-rs approach — BROKEN across kernels):
    // __u32 exit_code = *((__u32 *)((char *)task + 2292)); // hardcoded offset!

    return 0;
}


// ═══════════════════════════════════════════════
// PATTERN 4: Scratch map for large structs
// ═══════════════════════════════════════════════

SEC("tp/sched/sched_process_exec")
int example_scratch(void *ctx)
{
    // process_event is 1600 bytes — won't fit on 512-byte BPF stack.
    // Use per-CPU array map as scratch space.
    __u32 zero = 0;
    struct process_event *evt = bpf_map_lookup_elem(&SCRATCH, &zero);
    if (!evt)          // NULL check is MANDATORY — verifier rejects without it
        return 0;

    // Now evt points to map memory (not stack). Fill it in:
    __builtin_memset(evt, 0, sizeof(*evt)); // zero the scratch — OK for some sizes
    // NOTE: __builtin_memset fails for very large sizes on BPF backend.
    // For your 1600-byte struct, you might need to zero field-by-field.

    evt->time_ns = bpf_ktime_get_ns();
    evt->event_type = EVENT_EXEC;
    evt->pid = bpf_get_current_pid_tgid() >> 32;

    // Submit to ring buffer
    bpf_ringbuf_output(&EVENTS, evt, sizeof(*evt), 0);
    // 0 = flags (BPF_RB_NO_WAKEUP or BPF_RB_FORCE_WAKEUP optional)

    return 0;
}


// ═══════════════════════════════════════════════
// PATTERN 5: Map stash and lookup (exec → exit pairing)
// ═══════════════════════════════════════════════

// EXEC handler: stash start time
SEC("tp/sched/sched_process_exec")
int example_exec(void *ctx)
{
    __u32 pid = bpf_get_current_pid_tgid() >> 32;
    __u64 ts = bpf_ktime_get_ns();

    // Save start time in EXEC_START hash map
    bpf_map_update_elem(&EXEC_START, &pid, &ts, BPF_ANY);
    // BPF_ANY = create or update
    // BPF_NOEXIST = create only (fail if exists)
    // BPF_EXIST = update only (fail if doesn't exist)

    return 0;
}

// EXIT handler: look up start time, compute duration
SEC("tp/sched/sched_process_exit")
int example_exit(void *ctx)
{
    __u32 pid = bpf_get_current_pid_tgid() >> 32;

    // Look up when this process started
    __u64 *start = bpf_map_lookup_elem(&EXEC_START, &pid);
    if (!start)
        return 0; // never saw the exec — ignore

    __u64 duration = bpf_ktime_get_ns() - *start;

    // ... build event, submit to ring buffer ...

    // Clean up — remove from map (prevent leak)
    bpf_map_delete_elem(&EXEC_START, &pid);

    return 0;
}


// ═══════════════════════════════════════════════
// PATTERN 6: Reading argv from sys_enter_execve
// ═══════════════════════════════════════════════

SEC("tp/syscalls/sys_enter_execve")
int handle_sys_enter_execve(void *ctx)
{
    __u32 pid = bpf_get_current_pid_tgid() >> 32;

    // Read the argv pointer from the tracepoint context.
    // sys_enter_execve layout (after common header, which tp/ skips):
    //   offset 0:  __syscall_nr (int, 4 bytes)
    //   offset 4:  (padding, 4 bytes)
    //   offset 8:  filename (const char *, 8 bytes)
    //   offset 16: argv (const char *const *, 8 bytes)
    //   offset 24: envp (const char *const *, 8 bytes)
    const char *const *argv;
    bpf_probe_read_kernel(&argv, sizeof(argv), (void *)ctx + 16);
    // Why ctx + 16 and not ctx + 24?
    // Because tp/ skips the 8-byte common header.
    // Kernel offset 24 - 8 = tp offset 16.

    // Get scratch for args (10 * 128 = 1280 bytes — too big for stack)
    __u32 zero = 0;
    struct exec_args *scratch = bpf_map_lookup_elem(&ARGS_SCRATCH, &zero);
    if (!scratch)
        return 0;

    // Read each argument string from userspace
#pragma unroll
    for (int i = 0; i < MAX_ARG_COUNT; i++) {
        const char *arg;
        // First: read the pointer itself (argv[i])
        if (bpf_probe_read_user(&arg, sizeof(arg), &argv[i]) || !arg)
            break; // no more args or read failed
        // Then: read the string that pointer points to
        bpf_probe_read_user_str(scratch->args[i], MAX_ARG_LEN, arg);
    }

    // Stash in EXEC_ARGS map — the exec handler will copy these later
    bpf_map_update_elem(&EXEC_ARGS, &pid, scratch, BPF_ANY);

    return 0;
}

// ═══════════════════════════════════════════════
// PATTERN 7: Filtering (ignore known short-lived processes)
// ═══════════════════════════════════════════════

SEC("tp/sched/sched_process_exec")
int example_filter(void *ctx)
{
    __u32 pid = bpf_get_current_pid_tgid() >> 32;

    // Check if pid is in the ignore list
    if (bpf_map_lookup_elem(&IGNORED_PIDS, &pid))
        return 0; // skip

    // Check if name is in the ignore list
    char name[16];
    bpf_get_current_comm(name, sizeof(name));
    if (bpf_map_lookup_elem(&IGNORED_NAMES, name))
        return 0; // skip (e.g., "cat", "grep", "ls")

    // Process is not filtered — continue with event emission
    return 0;
}

#endif // #if 0

// This file is reference-only. Compile the real BPF code with:
//   clang -target bpf -O2 -g ...
// Or via: cd profiler && cargo build

#include <stdio.h>

int main(void)
{
    printf("This file is reference-only — read the source code.\n");
    printf("It shows the 7 key BPF patterns:\n\n");
    printf("  1. Tracepoint handler skeleton (SEC + return int)\n");
    printf("  2. Get current process info (pid, uid, comm)\n");
    printf("  3. CO-RE reads from task_struct (ppid, exit_code)\n");
    printf("  4. Scratch maps for large structs (>512 bytes)\n");
    printf("  5. Map stash/lookup for exec→exit pairing\n");
    printf("  6. Reading argv from sys_enter_execve\n");
    printf("  7. Filtering by pid and comm name\n\n");
    printf("Compare with: profiler/profiler/src/bpf.old/\n");
    printf("Apply to:     profiler/profiler-ebpf/process.bpf.h\n");
    return 0;
}
