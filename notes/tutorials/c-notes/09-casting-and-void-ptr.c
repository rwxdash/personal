// Lesson 09: Casting and void*
//
// BPF code is full of casts because:
//   - BPF helpers return void* or long (you cast to your type)
//   - Tracepoint ctx is void* (you cast or read at offsets)
//   - bpf_map_lookup_elem returns void* (you cast to your map value type)
//   - CO-RE reads need casts through struct pointers
//
// C casts have NO runtime checks — it's just "trust me, this is type X now".
// Like Rust's `as` keyword but with zero safety.
//
// Build: make 09-casting-and-void-ptr

#include <stdio.h>
#include <stdint.h>
#include <string.h>

int main(void)
{
    // ── Numeric casts — same as Rust `as` ──
    printf("── Numeric casts ──\n");

    int64_t big       = 0x123456789ABCDEFLL;
    int32_t truncated = (int32_t) big; // keeps lower 32 bits
    printf("  i64 0x%llx → i32 0x%x\n", (long long) big, truncated);

    float f = 3.14;
    int   i = (int) f; // truncates decimal, like Rust `as`
    printf("  3.14 → int: %d\n", i);

    int      neg = -1;
    uint32_t u   = (uint32_t) neg; // reinterprets bits: 0xFFFFFFFF
    printf("  -1 → u32: %u (0x%x)\n", u, u);

    // ── void* — the universal pointer ──
    printf("\n── void* ──\n");

    // void* can point to anything, but you can't dereference it directly
    int   x       = 42;
    void *generic = &x;

    // Must cast before dereferencing
    int *ip = (int *) generic;
    printf("  *(int*)generic = %d\n", *ip);

    // This is how bpf_map_lookup_elem works:
    //   void *val = bpf_map_lookup_elem(&my_map, &key);
    //   struct my_type *typed = (struct my_type *)val;
    //   if (!typed) return 0;  // NULL check required!
    //   typed->field = 123;

    // ── Pointer arithmetic on void* and char* ──
    printf("\n── Pointer offsets ──\n");

    // You can't do arithmetic on void* (GCC allows it as extension, but it's not standard)
    // Use char* for byte-level pointer arithmetic

    typedef struct {
        uint32_t a; // offset 0
        uint32_t b; // offset 4
        uint64_t c; // offset 8
    } my_struct;

    my_struct s = {.a = 111, .b = 222, .c = 333};

    // Reading a field by offset (like BPF tracepoint ctx access)
    char *base = (char *) &s;

    uint32_t *pa = (uint32_t *) (base + 0); // offset 0 → field a
    uint32_t *pb = (uint32_t *) (base + 4); // offset 4 → field b
    uint64_t *pc = (uint64_t *) (base + 8); // offset 8 → field c

    printf("  at offset 0: %u (field a)\n", *pa);
    printf("  at offset 4: %u (field b)\n", *pb);
    printf("  at offset 8: %llu (field c)\n", (unsigned long long) *pc);

    // This is exactly what BPF tracepoint handlers do:
    //   void *ctx = ...; // tracepoint context
    //   __u32 pid = *(__u32 *)((char *)ctx + 4);  // read pid at offset 4
    //   const char *argv = *(const char **)((char *)ctx + 16);

    // ── Casting structs — reinterpreting memory ──
    printf("\n── Struct casting ──\n");

    // Raw bytes that happen to be a struct
    char raw_bytes[16];
    memset(raw_bytes, 0, sizeof(raw_bytes));

    // Write values at specific offsets
    *(uint32_t *) (raw_bytes + 0) = 0xDEADBEEF;
    *(uint32_t *) (raw_bytes + 4) = 42;
    *(uint64_t *) (raw_bytes + 8) = 123456789ULL;

    // Cast the whole thing to our struct
    my_struct *sp = (my_struct *) raw_bytes;
    printf("  a = 0x%x, b = %u, c = %llu\n", sp->a, sp->b, (unsigned long long) sp->c);

    // This is how BPF ring buffer events work:
    // BPF writes raw bytes → Rust reads and casts to the event struct.
    // Both sides must agree on the exact memory layout.

    // ── Function pointer casting (less common in BPF) ──
    printf("\n── sizeof with casts ──\n");
    printf("  sizeof(int) = %zu\n", sizeof(int));
    printf("  sizeof(int*) = %zu\n", sizeof(int *));
    printf("  sizeof(void*) = %zu\n", sizeof(void *));
    // All pointers are the same size (8 bytes on 64-bit)

    // ── BPF pattern: reading from ctx ──
    printf("\n── BPF tracepoint ctx pattern ──\n");

    // Simulating a tracepoint context buffer
    // sys_enter_execve after common header (skipped by SEC("tp/...")):
    //   offset 0: __syscall_nr (4 bytes) + 4 pad
    //   offset 8: filename ptr (8 bytes)
    //   offset 16: argv ptr (8 bytes)
    //   offset 24: envp ptr (8 bytes)
    char fake_ctx[32];
    memset(fake_ctx, 0, sizeof(fake_ctx));
    *(int32_t *) (fake_ctx + 0)   = 59;         // __syscall_nr for execve
    *(uint64_t *) (fake_ctx + 8)  = 0xAAAA0000; // fake filename ptr
    *(uint64_t *) (fake_ctx + 16) = 0xBBBB0000; // fake argv ptr

    // Reading fields (what your BPF code does):
    int      syscall_nr   = *(int32_t *) (fake_ctx + 0);
    uint64_t filename_ptr = *(uint64_t *) (fake_ctx + 8);
    uint64_t argv_ptr     = *(uint64_t *) (fake_ctx + 16);

    printf("  syscall_nr = %d\n", syscall_nr);
    printf("  filename   = 0x%llx\n", (unsigned long long) filename_ptr);
    printf("  argv       = 0x%llx\n", (unsigned long long) argv_ptr);

    // In real BPF:
    //   const char *const *argv;
    //   bpf_probe_read_kernel(&argv, sizeof(argv), ctx + 16);
    // bpf_probe_read_kernel is needed because ctx memory requires special access

    // ── Exercises ──
    // 1. Create a char buffer[24], write three uint64_t values at offsets 0, 8, 16.
    //    Then cast to a struct with three uint64_t fields and verify.
    // 2. What happens if you cast a char* to uint64_t* when the address isn't
    //    8-byte aligned? (hint: on x86 it works, on ARM it might crash)
    // 3. Write a function read_u32(void *ctx, int offset) that reads a uint32_t
    //    from an arbitrary offset in a buffer.
    // 4. Why does BPF use bpf_probe_read_kernel() instead of just *(type*)ptr?
    //    (hint: BPF verifier doesn't trust raw pointer dereferences on kernel memory)

    return 0;
}
