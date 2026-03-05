// Lesson 10: Memory Layout — Stack, Heap, and the BPF 512-byte Stack Limit
//
// Understanding memory layout is crucial for BPF because:
//   - BPF stack is only 512 bytes (not the usual 8MB)
//   - Large structs (your process_event is 1600 bytes!) don't fit on the stack
//   - You use per-CPU array maps as "scratch space" instead of the stack
//   - No malloc/free in BPF
//
// Build: make 10-memory-layout

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h> // malloc, free
#include <string.h>

int main(void)
{
    // ═══════════════════════════════════════════════
    // ── Stack memory ──
    // ═══════════════════════════════════════════════
    printf("── Stack ──\n");

    // Local variables live on the stack — automatic allocation and deallocation
    int  x = 42;  // 4 bytes on stack
    char buf[64]; // 64 bytes on stack
    memset(buf, 0, sizeof(buf));

    printf("  &x   = %p (stack)\n", (void *) &x);
    printf("  &buf = %p (stack, close to &x)\n", (void *) &buf);

    // Stack grows downward on x86 — later variables have lower addresses
    int y = 99;
    printf("  &y   = %p (lower than &x: stack grows down)\n", (void *) &y);

    // Normal program stack: ~8MB. BPF stack: 512 bytes!
    // This is fine in normal C:
    char big_array[4096]; // 4KB on stack — no problem
    (void) big_array;
    // But in BPF, this would fail verification:
    //   char big[1024]; // ERROR: stack size 1024 > 512 limit

    // ═══════════════════════════════════════════════
    // ── Heap memory ──
    // ═══════════════════════════════════════════════
    printf("\n── Heap (malloc/free) ──\n");

    // malloc allocates on the heap — you must free it yourself
    // Rust equivalent: Box::new() but without automatic Drop
    int *heap_int = (int *) malloc(sizeof(int));
    if (!heap_int) {
        printf("malloc failed!\n");
        return 1;
    }
    *heap_int = 42;
    printf("  heap_int = %p, value = %d\n", (void *) heap_int, *heap_int);

    // Dynamic array (like Vec::with_capacity())
    int  count = 10;
    int *arr   = (int *) malloc(count * sizeof(int));
    for (int i = 0; i < count; i++)
        arr[i] = i * 10;
    printf("  arr[5] = %d\n", arr[5]);

    // You MUST free when done — no garbage collector, no RAII
    free(heap_int);
    free(arr);
    // Using heap_int or arr after free = use-after-free bug (undefined behavior)
    // Rust prevents this at compile time. C doesn't.

    // NOTE: BPF has NO malloc. No heap at all.

    // ═══════════════════════════════════════════════
    // ── BPF pattern: per-CPU scratch maps ──
    // ═══════════════════════════════════════════════
    printf("\n── BPF scratch pattern (simulated) ──\n");

    // Since BPF stack is 512 bytes and process_event is 1600 bytes,
    // we use a BPF_MAP_TYPE_PERCPU_ARRAY as "scratch space":
    //
    //   // In BPF:
    //   struct {
    //       __uint(type, BPF_MAP_TYPE_PERCPU_ARRAY);
    //       __type(key, __u32);
    //       __type(value, struct process_event);  // 1600 bytes
    //       __uint(max_entries, 1);  // only 1 entry per CPU
    //   } SCRATCH SEC(".maps");
    //
    //   // In handler:
    //   __u32 zero = 0;
    //   struct process_event *evt = bpf_map_lookup_elem(&SCRATCH, &zero);
    //   if (!evt) return 0;  // verifier requires this null check!
    //
    //   // Now evt points to 1600 bytes of map memory — not the stack
    //   evt->pid = bpf_get_current_pid_tgid() >> 32;
    //   evt->time_ns = bpf_ktime_get_ns();

    // Simulating this in userspace:
    typedef struct {
        uint64_t time_ns;
        uint32_t pid;
        char     name[16];
        char     filename[256];
        char     args[10][128]; // 1280 bytes — way over 512!
    } process_event;

    printf("  sizeof(process_event) = %zu bytes\n", sizeof(process_event));
    printf("  BPF stack limit       = 512 bytes\n");
    printf("  Fits on stack?        NO — need scratch map\n");

    // The "map" is like a pre-allocated buffer that persists across calls
    process_event scratch; // in BPF, this comes from map lookup
    memset(&scratch, 0, sizeof(scratch));
    scratch.pid = 1234;
    snprintf(scratch.name, sizeof(scratch.name), "bash");
    printf("  scratch.pid = %u, name = %s\n", scratch.pid, scratch.name);

    // ═══════════════════════════════════════════════
    // ── What fits on the BPF stack ──
    // ═══════════════════════════════════════════════
    printf("\n── What fits on 512-byte BPF stack ──\n");

    // These are fine:
    printf("  __u32 key = 0;              // %zu bytes\n", sizeof(uint32_t));
    printf("  __u64 ts;                   // %zu bytes\n", sizeof(uint64_t));
    printf("  char comm[16];              // %zu bytes\n", sizeof(char[16]));
    printf("  const char *ptr;            // %zu bytes\n", sizeof(char *));
    // Total so far: 4 + 8 + 16 + 8 = 36 bytes — plenty of room

    // These are NOT fine:
    printf("\n  char args[10][128];          // %zu bytes — TOO BIG\n", sizeof(char[10][128]));
    printf("  struct process_event evt;   // %zu bytes — WAY TOO BIG\n", sizeof(process_event));

    // ═══════════════════════════════════════════════
    // ── Memory segments summary ──
    // ═══════════════════════════════════════════════
    printf("\n── Memory segments (normal program) ──\n");
    printf("  .text    — compiled code (read-only, execute)\n");
    printf("  .rodata  — string literals, const globals (read-only)\n");
    printf("  .data    — initialized global/static variables\n");
    printf("  .bss     — uninitialized globals (zeroed at start)\n");
    printf("  stack    — local variables, function call frames\n");
    printf("  heap     — malloc'd memory\n");

    printf("\n── BPF memory model ──\n");
    printf("  stack    — 512 bytes only, local vars\n");
    printf("  maps     — persistent storage (hash, array, ring buffer)\n");
    printf("  ctx      — tracepoint/kprobe context (read-only-ish)\n");
    printf("  NO heap  — no malloc, no dynamic allocation\n");
    printf("  NO globals (mutable) — use maps instead\n");

    // ── Exercises ──
    // 1. Calculate: how many uint32_t variables can fit on a 512-byte BPF stack?
    // 2. Your process_event is 1600 bytes. What's the max_entries you'd set
    //    for a PERCPU_ARRAY scratch map? (hint: just 1 — one per CPU)
    // 3. Why PERCPU_ARRAY instead of regular ARRAY? (hint: no locking needed
    //    because each CPU has its own copy)
    // 4. In BPF, why is the null check after bpf_map_lookup_elem mandatory?
    //    What happens if you skip it? (hint: BPF verifier rejects the program)

    return 0;
}
