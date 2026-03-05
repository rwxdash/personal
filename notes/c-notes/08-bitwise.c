// Lesson 08: Bitwise Operations
//
// BPF code uses bit manipulation constantly:
//   - Extract PID from bpf_get_current_pid_tgid() (upper 32 bits)
//   - Extract UID from bpf_get_current_uid_gid() (lower 32 bits)
//   - Check flags, set bits, mask values
//   - Kernel exit_code encoding: signal in lower 8 bits, status in upper 8 bits
//
// Operators:
//   &   AND    (like Rust)
//   |   OR     (like Rust)
//   ^   XOR    (like Rust)
//   ~   NOT    (like Rust !)
//   <<  left shift   (like Rust)
//   >>  right shift  (like Rust)
//
// Build: make 08-bitwise

#include <stdio.h>
#include <stdint.h>

// Helper to print binary representation
void print_bits(const char *label, uint32_t val)
{
    printf("  %-12s = 0x%08x = ", label, val);
    for (int i = 31; i >= 0; i--) {
        printf("%d", (val >> i) & 1);
        if (i % 8 == 0 && i > 0)
            printf("_");
    }
    printf("\n");
}

int main(void)
{
    // ── AND (&) — mask out bits ──
    printf("── AND (masking) ──\n");
    uint32_t val     = 0xABCD1234;
    uint32_t lower16 = val & 0xFFFF;     // keep lower 16 bits
    uint32_t upper16 = val & 0xFFFF0000; // keep upper 16 bits
    print_bits("val", val);
    print_bits("& 0xFFFF", lower16);
    print_bits("& 0xFFFF0000", upper16);

    // ── OR (|) — set bits ──
    printf("\n── OR (setting bits) ──\n");
    uint32_t flags = 0;
    flags |= (1 << 0); // set bit 0
    flags |= (1 << 3); // set bit 3
    flags |= (1 << 7); // set bit 7
    print_bits("flags", flags);

    // ── XOR (^) — toggle bits ──
    printf("\n── XOR (toggle) ──\n");
    flags ^= (1 << 3); // toggle bit 3 off
    print_bits("after ^bit3", flags);
    flags ^= (1 << 3); // toggle bit 3 back on
    print_bits("after ^bit3", flags);

    // ── Shifts ──
    printf("\n── Shifts ──\n");
    uint32_t one = 1;
    print_bits("1", one);
    print_bits("1 << 4", one << 4);   // 16
    print_bits("1 << 31", one << 31); // 0x80000000

    uint32_t big = 0xFF000000;
    print_bits("0xFF000000", big);
    print_bits(">> 24", big >> 24); // 0xFF = 255

    // ═══════════════════════════════════════════════
    // ── BPF PATTERN: pid/tgid extraction ──
    // ═══════════════════════════════════════════════
    printf("\n── BPF: pid/tgid from bpf_get_current_pid_tgid() ──\n");

    // bpf_get_current_pid_tgid() returns a u64:
    //   upper 32 bits = tgid (what userspace calls "pid")
    //   lower 32 bits = tid  (thread id, kernel's internal pid)
    uint64_t pid_tgid = 0x00001234ABCD5678ULL; // fake value

    uint32_t tgid = pid_tgid >> 32;            // shift upper 32 down
    uint32_t tid  = (uint32_t) pid_tgid;       // cast truncates to lower 32
    // Or equivalently: tid = pid_tgid & 0xFFFFFFFF;

    printf("  pid_tgid = 0x%016llx\n", (unsigned long long) pid_tgid);
    printf("  tgid     = 0x%08x (%u)\n", tgid, tgid); // 0x00001234
    printf("  tid      = 0x%08x (%u)\n", tid, tid);   // 0xABCD5678

    // In your BPF code, this becomes:
    //   __u32 pid = bpf_get_current_pid_tgid() >> 32;

    // ── BPF PATTERN: uid/gid extraction ──
    printf("\n── BPF: uid/gid from bpf_get_current_uid_gid() ──\n");

    uint64_t uid_gid = 0x000003E8000003E8ULL; // uid=1000, gid=1000

    uint32_t gid = uid_gid >> 32;
    uint32_t uid = (uint32_t) uid_gid;

    printf("  uid = %u, gid = %u\n", uid, gid);

    // ── BPF PATTERN: exit code decoding ──
    printf("\n── Kernel exit_code encoding ──\n");

    // The kernel's task_struct.exit_code packs two things:
    //   - Lower 8 bits: signal number (if killed by signal)
    //   - Upper bits (shifted by 8): exit status (if exited normally)
    //
    // Normal exit(42):  exit_code = 42 << 8 = 0x2A00, signal = 0
    // Killed by SIGKILL: exit_code = 9, status = 0

    uint32_t exit_normal = 42 << 8; // exit(42)
    uint32_t exit_killed = 9;       // killed by SIGKILL

    printf("  exit(42):  raw=0x%x  status=%d  signal=%d\n", exit_normal, (exit_normal >> 8) & 0xFF, exit_normal & 0x7F);
    printf("  SIGKILL:   raw=0x%x  status=%d  signal=%d\n", exit_killed, (exit_killed >> 8) & 0xFF, exit_killed & 0x7F);

    // ── Checking if a bit is set ──
    printf("\n── Bit testing ──\n");
    uint32_t perms = 0x755; // rwxr-xr-x in octal
    printf("  perms = 0%o (0x%x)\n", perms, perms);
    printf("  owner execute: %s\n", (perms & 0x40) ? "yes" : "no");
    printf("  group write:   %s\n", (perms & 0x10) ? "yes" : "no");
    printf("  other read:    %s\n", (perms & 0x04) ? "yes" : "no");

    // ── Exercises ──
    // 1. Given uint64_t pid_tgid = 0x00002710_00006789, extract pid and tid.
    //    What are they in decimal?
    // 2. Pack two uint32_t values (pid=100, tid=200) into one uint64_t.
    //    Hint: ((uint64_t)pid << 32) | tid
    // 3. The kernel exit code 0x0900 — what's the exit status? What's the signal?
    // 4. Write a function is_bit_set(uint32_t val, int bit) that returns 1/0.

    return 0;
}
