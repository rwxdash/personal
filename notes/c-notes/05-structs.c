// Lesson 05: Structs
//
// C structs are like Rust structs without methods, traits, or derive macros.
// They're just data layouts — and the layout matters a LOT for BPF because
// your C struct and Rust struct must have identical memory layout.
//
// Key differences from Rust:
//   - No methods (no impl block)
//   - No #[derive(Debug)] — you printf each field manually
//   - Padding/alignment happens automatically (like Rust's default repr)
//   - Use __attribute__((packed)) or #pragma pack to disable padding
//   - typedef lets you skip writing "struct" everywhere
//
// Build: make 05-structs

#include <stdio.h>
#include <stdint.h> // uint32_t, uint64_t — fixed-width types like Rust
#include <string.h> // memset
#include <stddef.h> // offsetof

int main(void)
{
    // Basic struct — must write "struct" before the name (unlike Rust)
    struct point {
        int x;
        int y;
    };

    struct point p = {10, 20};
    printf("point: (%d, %d)\n", p.x, p.y);

    // Access via pointer — use -> instead of .
    struct point *pp = &p;
    printf("via pointer: (%d, %d)\n", pp->x, pp->y);
    // pp->x is shorthand for (*pp).x

    // ── typedef — avoid writing "struct" everywhere ──
    typedef struct {
        uint32_t uid;
        uint32_t gid;
        uint32_t pid;
        uint32_t tgid;
    } process_id;

    process_id id = {.uid = 1000, .gid = 1000, .pid = 42, .tgid = 42};
    printf("\nprocess: uid=%u pid=%u\n", id.uid, id.pid);

    // Designated initializers — name the fields (like Rust struct init)
    // C99 feature. You can skip fields (they're zeroed).
    process_id partial = {.pid = 99}; // uid=0, gid=0, pid=99, tgid=0
    printf("partial: uid=%u pid=%u\n", partial.uid, partial.pid);

    // ── Struct padding and alignment ──
    // The compiler inserts padding bytes to align fields to their natural size.
    // This is CRITICAL for BPF — your C and Rust structs must match exactly.
    struct padded {
        char a; // 1 byte
        // 3 bytes padding (to align int to 4-byte boundary)
        int  b; // 4 bytes
        char c; // 1 byte
        // 7 bytes padding (to align uint64_t to 8-byte boundary)
        uint64_t d; // 8 bytes
    };
    // Total: 24 bytes (not 14!)

    printf("\n── Padding ──\n");
    printf("sizeof(struct padded) = %zu\n", sizeof(struct padded)); // 24
    printf("  offset of a: %zu\n", offsetof(struct padded, a));     // 0
    printf("  offset of b: %zu\n", offsetof(struct padded, b));     // 4
    printf("  offset of c: %zu\n", offsetof(struct padded, c));     // 8
    printf("  offset of d: %zu\n", offsetof(struct padded, d));     // 16

    // ── No padding (packed) — sometimes needed for wire formats ──
    struct __attribute__((packed)) tight {
        char     a;
        int      b;
        char     c;
        uint64_t d;
    };
    printf("\nsizeof(packed) = %zu\n", sizeof(struct tight)); // 14 (no padding)

    // ── BPF event struct example ──
    // This mirrors what you'll define in profiler.bpf.h
    typedef struct {
        uint64_t time_ns;    // 0: 8 bytes
        uint8_t  event_type; // 8: 1 byte
        // 3 bytes padding
        uint32_t exit_code;   // 12: 4 bytes
        uint64_t duration_ns; // 16: 8 bytes
        uint32_t uid;         // 24: 4 bytes
        uint32_t gid;         // 28: 4 bytes
        uint32_t pid;         // 32: 4 bytes
        uint32_t tgid;        // 36: 4 bytes
        uint32_t ppid;        // 40: 4 bytes
        char     name[16];    // 44: 16 bytes
        // 4 bytes padding (to align filename to... actually char has no alignment)
        char filename[256]; // 60: 256 bytes
    } process_event;

    printf("\n── BPF-like struct ──\n");
    printf("sizeof(process_event) = %zu\n", sizeof(process_event));
    printf("  offset time_ns:    %zu\n", offsetof(process_event, time_ns));
    printf("  offset event_type: %zu\n", offsetof(process_event, event_type));
    printf("  offset exit_code:  %zu\n", offsetof(process_event, exit_code));
    printf("  offset pid:        %zu\n", offsetof(process_event, pid));
    printf("  offset name:       %zu\n", offsetof(process_event, name));
    printf("  offset filename:   %zu\n", offsetof(process_event, filename));

    // Zero-init a struct (common BPF pattern)
    process_event evt;
    memset(&evt, 0, sizeof(evt));
    evt.pid        = 1234;
    evt.event_type = 1;
    snprintf(evt.name, sizeof(evt.name), "bash");
    snprintf(evt.filename, sizeof(evt.filename), "/usr/bin/bash");
    printf("\nevent: pid=%u type=%u name=%s file=%s\n", evt.pid, evt.event_type, evt.name, evt.filename);

    // ── Nested structs ──
    typedef struct {
        uint64_t time_ns;
        uint8_t  event_type;
    } event_header;

    typedef struct {
        event_header header; // embedded, not a pointer
        uint32_t     pid;
        int          exit_code;
    } exit_event;

    exit_event ex = {
        .header    = {.time_ns = 123456, .event_type = 2},
        .pid       = 42,
        .exit_code = 0,
    };
    printf("\nnested: time=%llu type=%u pid=%u exit=%d\n", (unsigned long long) ex.header.time_ns, ex.header.event_type,
           ex.pid, ex.exit_code);

    // ── Exercises ──
    // 1. Create a struct with fields: char a; char b; int c;
    //    What's the sizeof? Use offsetof to check padding.
    // 2. Reorder the fields to minimize padding. What's the new sizeof?
    // 3. Create a struct that matches your profiler.bpf.h process_event exactly.
    //    Verify sizeof and all offsets match the BTF dump you saw earlier.
    // 4. What's the difference between struct point p = {0} and memset(&p, 0, sizeof(p))?

    return 0;
}
