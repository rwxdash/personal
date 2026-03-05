// Lesson 12: Simulating BPF Helpers and Patterns
//
// BPF programs can't use libc (no printf, malloc, strcmp).
// Instead, they call "BPF helpers" — kernel-provided functions.
// This lesson simulates the key ones so you understand the API
// before writing real BPF code.
//
// Build: make 12-bpf-helpers-sim

#define _POSIX_C_SOURCE 199309L // needed for clock_gettime / CLOCK_MONOTONIC

#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <time.h> // clock_gettime

// ═══════════════════════════════════════════════
// Simulate BPF helper functions
// ═══════════════════════════════════════════════

// bpf_ktime_get_ns() — returns monotonic nanosecond timestamp
static uint64_t sim_bpf_ktime_get_ns(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t) ts.tv_sec * 1000000000ULL + ts.tv_nsec;
}

// bpf_get_current_pid_tgid() — returns (tgid << 32 | pid)
static uint64_t sim_bpf_get_current_pid_tgid(void)
{
    // Fake: pid=1234, tgid=1234 (single-threaded)
    uint32_t pid  = 1234;
    uint32_t tgid = 1234;
    return ((uint64_t) tgid << 32) | pid;
}

// bpf_get_current_uid_gid() — returns (gid << 32 | uid)
static uint64_t sim_bpf_get_current_uid_gid(void)
{
    uint32_t uid = 1000;
    uint32_t gid = 1000;
    return ((uint64_t) gid << 32) | uid;
}

// bpf_get_current_comm(buf, size) — fill buf with current process name
static int sim_bpf_get_current_comm(char *buf, uint32_t size)
{
    strncpy(buf, "fake-process", size - 1);
    buf[size - 1] = '\0';
    return 0;
}

// bpf_probe_read_user_str(dst, size, unsafe_ptr) — read string from userspace
static int sim_bpf_probe_read_user_str(char *dst, uint32_t size, const char *src)
{
    // In real BPF, src is a userspace pointer that needs special access
    strncpy(dst, src, size - 1);
    dst[size - 1] = '\0';
    return (int) strlen(dst) + 1; // returns bytes written including \0
}

// bpf_probe_read_kernel(dst, size, src) — read from kernel memory
static int sim_bpf_probe_read_kernel(void *dst, uint32_t size, const void *src)
{
    memcpy(dst, src, size);
    return 0;
}

// ═══════════════════════════════════════════════
// Simulate BPF maps
// ═══════════════════════════════════════════════

// Simulating a simple hash map (BPF_MAP_TYPE_HASH)
#define MAP_SIZE 64

typedef struct {
    uint32_t key;
    uint64_t value;
    int      occupied;
} map_entry;

typedef struct {
    map_entry entries[MAP_SIZE];
} hash_map;

static hash_map exec_start_map = {0};

static void *sim_bpf_map_lookup_elem(hash_map *map, const uint32_t *key)
{
    for (int i = 0; i < MAP_SIZE; i++) {
        if (map->entries[i].occupied && map->entries[i].key == *key)
            return &map->entries[i].value;
    }
    return NULL; // not found — caller MUST check!
}

static int sim_bpf_map_update_elem(hash_map *map, const uint32_t *key, const uint64_t *value)
{
    // Find existing or empty slot
    for (int i = 0; i < MAP_SIZE; i++) {
        if (!map->entries[i].occupied || map->entries[i].key == *key) {
            map->entries[i].key      = *key;
            map->entries[i].value    = *value;
            map->entries[i].occupied = 1;
            return 0;
        }
    }
    return -1; // map full
}

static int sim_bpf_map_delete_elem(hash_map *map, const uint32_t *key)
{
    for (int i = 0; i < MAP_SIZE; i++) {
        if (map->entries[i].occupied && map->entries[i].key == *key) {
            map->entries[i].occupied = 0;
            return 0;
        }
    }
    return -1; // not found
}

// ═══════════════════════════════════════════════
// Simulate a BPF ring buffer output
// ═══════════════════════════════════════════════

#define MAX_NAME_LEN     16
#define MAX_FILENAME_LEN 256
#define MAX_ARG_COUNT    10
#define MAX_ARG_LEN      128

#define EVENT_EXEC       1
#define EVENT_EXIT       2

typedef struct {
    uint64_t time_ns;
    uint8_t  event_type;
    uint32_t exit_code;
    uint64_t duration_ns;
    uint32_t uid;
    uint32_t gid;
    uint32_t pid;
    uint32_t tgid;
    uint32_t ppid;
    char     name[MAX_NAME_LEN];
    char     filename[MAX_FILENAME_LEN];
    char     args[MAX_ARG_COUNT][MAX_ARG_LEN];
} process_event;

static void sim_bpf_ringbuf_output(const process_event *evt)
{
    // In real BPF: bpf_ringbuf_output(&EVENTS, evt, sizeof(*evt), 0);
    // Here we just print it like the Rust userspace would
    printf("  [RING] type=%s pid=%u ppid=%u name=%s file=%s", evt->event_type == EVENT_EXEC ? "EXEC" : "EXIT", evt->pid,
           evt->ppid, evt->name, evt->filename);
    if (evt->event_type == EVENT_EXIT)
        printf(" exit_code=%u duration=%llums", evt->exit_code, (unsigned long long) (evt->duration_ns / 1000000));
    printf("\n");
}

// ═══════════════════════════════════════════════
// Simulated BPF program handlers
// ═══════════════════════════════════════════════

// This mirrors what handle_sched_process_exec does in process.bpf.h
static void simulate_exec_handler(const char *filename, const char *argv[])
{
    printf("\n── Simulating sched_process_exec ──\n");

    // 1. Get IDs
    uint64_t pid_tgid = sim_bpf_get_current_pid_tgid();
    uint32_t pid      = pid_tgid >> 32;
    uint32_t tgid     = (uint32_t) pid_tgid;

    uint64_t uid_gid = sim_bpf_get_current_uid_gid();
    uint32_t uid     = (uint32_t) uid_gid;
    uint32_t gid     = uid_gid >> 32;

    // 2. Get scratch buffer (simulated — in BPF this is a map lookup)
    process_event evt;
    memset(&evt, 0, sizeof(evt));

    // 3. Fill in fields
    evt.time_ns    = sim_bpf_ktime_get_ns();
    evt.event_type = EVENT_EXEC;
    evt.pid        = pid;
    evt.tgid       = tgid;
    evt.uid        = uid;
    evt.gid        = gid;
    evt.ppid       = 1; // In BPF: BPF_CORE_READ(task, real_parent, tgid)

    sim_bpf_get_current_comm(evt.name, sizeof(evt.name));
    sim_bpf_probe_read_user_str(evt.filename, sizeof(evt.filename), filename);

    // 4. Copy args
    for (int i = 0; i < MAX_ARG_COUNT && argv[i]; i++)
        sim_bpf_probe_read_user_str(evt.args[i], MAX_ARG_LEN, argv[i]);

    // 5. Record start time in EXEC_START map
    uint64_t ts = evt.time_ns;
    sim_bpf_map_update_elem(&exec_start_map, &pid, &ts);

    // 6. Submit to ring buffer
    sim_bpf_ringbuf_output(&evt);
}

// This mirrors what handle_sched_process_exit does
static void simulate_exit_handler(uint32_t pid, int exit_status)
{
    printf("\n── Simulating sched_process_exit ──\n");

    // 1. Look up start time
    uint64_t *start = (uint64_t *) sim_bpf_map_lookup_elem(&exec_start_map, &pid);
    if (!start) {
        printf("  [SKIP] pid=%u not in EXEC_START map\n", pid);
        return;
    }
    uint64_t start_time = *start;

    // 2. Build exit event
    process_event evt;
    memset(&evt, 0, sizeof(evt));

    evt.time_ns     = sim_bpf_ktime_get_ns();
    evt.event_type  = EVENT_EXIT;
    evt.pid         = pid;
    evt.tgid        = pid;
    evt.exit_code   = exit_status;
    evt.duration_ns = evt.time_ns - start_time;

    sim_bpf_get_current_comm(evt.name, sizeof(evt.name));

    // 3. Submit and clean up
    sim_bpf_ringbuf_output(&evt);
    sim_bpf_map_delete_elem(&exec_start_map, &pid);
}

int main(void)
{
    printf("── BPF Helper Simulation ──\n\n");

    // Test individual helpers
    printf("  ktime_ns:  %llu\n", (unsigned long long) sim_bpf_ktime_get_ns());

    uint64_t pid_tgid = sim_bpf_get_current_pid_tgid();
    printf("  pid_tgid:  0x%016llx\n", (unsigned long long) pid_tgid);
    printf("  tgid:      %u\n", (uint32_t) (pid_tgid >> 32));
    printf("  pid:       %u\n", (uint32_t) pid_tgid);

    char comm[16];
    sim_bpf_get_current_comm(comm, sizeof(comm));
    printf("  comm:      %s\n", comm);

    // Simulate full exec → exit flow
    const char *argv[] = {"/usr/bin/gcc", "-O2", "-o", "hello", "hello.c", NULL};
    simulate_exec_handler("/usr/bin/gcc", argv);

    // Simulate process exit
    uint32_t pid = 1234;
    simulate_exit_handler(pid, 0); // normal exit

    // Try exit for unknown pid
    simulate_exit_handler(9999, 1);

    printf("\n── Key Takeaways ──\n");
    printf("1. Always check bpf_map_lookup_elem() return for NULL\n");
    printf("2. Use scratch maps for large structs (>512 bytes)\n");
    printf("3. Extract pid/uid from combined u64 return values\n");
    printf("4. BPF_CORE_READ for kernel struct fields (ppid, exit_code)\n");
    printf("5. bpf_probe_read_user_str for userspace strings (argv)\n");

    return 0;
}
