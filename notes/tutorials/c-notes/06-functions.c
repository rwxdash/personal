// Lesson 06: Functions
//
// C functions are simpler than Rust's: no closures, no generics, no Result.
// All parameters are pass-by-value (copies). To modify the caller's data,
// pass a pointer. This is how BPF helper functions work.
//
// BPF-specific:
//   - __always_inline forces the compiler to inline (BPF has no function calls pre-5.10)
//   - static restricts visibility to the current file (translation unit)
//   - Functions must be declared before use (forward declarations)
//
// Build: make 06-functions

#include <stdio.h>
#include <stdint.h>
#include <string.h>

// ── Forward declaration ──
// C reads top-to-bottom. If foo() calls bar(), bar must be declared first.
// You can either define bar above foo, or declare it like this:
int  add(int a, int b); // declaration (prototype)
void fill_buffer(char *buf, int size, char c);

// ── Basic function ──
int add(int a, int b) // definition
{
    return a + b;
}

// ── Void return — no return value (like Rust -> ()) ──
void say_hello(void) // "void" in params means "no arguments"
{
    printf("hello!\n");
}

// ── Pass by value — C copies the argument ──
void try_modify(int x)
{
    x = 999; // modifies the LOCAL copy, not the caller's
}

// ── Pass by pointer — to actually modify caller's data ──
void actually_modify(int *x)
{
    *x = 999; // dereferences and modifies the real value
}

// ── Array parameters ──
// Arrays decay to pointers when passed. You MUST pass the length separately.
// void foo(int arr[5]) is identical to void foo(int *arr) — the 5 is ignored!
int sum(const int *arr, int len)
{
    int total = 0;
    for (int i = 0; i < len; i++)
        total += arr[i];
    return total;
}

// ── Struct parameters ──
typedef struct {
    uint32_t pid;
    char     name[16];
} process_info;

// Pass struct by value = copy entire struct (expensive for large structs!)
void print_info_copy(process_info info)
{
    printf("  [copy] pid=%u name=%s\n", info.pid, info.name);
}

// Pass struct by pointer = no copy, just pass the address (cheap, preferred)
void print_info_ptr(const process_info *info)
{
    printf("  [ptr]  pid=%u name=%s\n", info->pid, info->name);
}

// Modify struct through pointer — common BPF pattern
void init_info(process_info *info, uint32_t pid, const char *name)
{
    memset(info, 0, sizeof(*info)); // sizeof(*info) = sizeof(process_info)
    info->pid = pid;
    strncpy(info->name, name, sizeof(info->name) - 1);
}

// ── static functions ──
// "static" means this function is only visible in this file.
// In BPF, all helper functions are static __always_inline.
static int helper(int x)
{
    return x * 2;
}

// ── Filling a buffer (BPF-like pattern) ──
void fill_buffer(char *buf, int size, char c)
{
    for (int i = 0; i < size; i++)
        buf[i] = c;
}

// ── Output parameters ──
// C has no tuples or multiple return values. Use output pointers instead.
// This is like Rust's fn foo(out: &mut i32) pattern.
int divide(int a, int b, int *quotient, int *remainder)
{
    if (b == 0)
        return -1; // error (C convention: negative = error, 0 = success)
    *quotient  = a / b;
    *remainder = a % b;
    return 0; // success
}

int main(void)
{
    printf("add(3, 4) = %d\n", add(3, 4));

    say_hello();

    // Pass by value vs pointer
    int x = 42;
    try_modify(x);
    printf("\nAfter try_modify: x = %d (unchanged)\n", x);
    actually_modify(&x);
    printf("After actually_modify: x = %d (modified)\n", x);

    // Arrays
    int nums[] = {1, 2, 3, 4, 5};
    printf("\nsum = %d\n", sum(nums, 5));

    // Structs
    printf("\nStruct passing:\n");
    process_info info;
    init_info(&info, 1234, "bash");
    print_info_copy(info); // copies 20 bytes
    print_info_ptr(&info); // passes 8-byte pointer

    // Static helper
    printf("\nhelper(21) = %d\n", helper(21));

    // Output parameters
    int q, r;
    if (divide(17, 5, &q, &r) == 0)
        printf("\n17 / 5 = %d remainder %d\n", q, r);

    if (divide(10, 0, &q, &r) != 0)
        printf("division by zero caught (returned error)\n");

    // ── BPF pattern recap ──
    // In BPF code, you'll see patterns like:
    //
    //   static __always_inline int
    //   get_process_info(struct process_event *evt) {
    //       evt->pid = bpf_get_current_pid_tgid() >> 32;
    //       bpf_get_current_comm(evt->name, sizeof(evt->name));
    //       return 0;
    //   }
    //
    // - static: file-local
    // - __always_inline: must inline (BPF requirement for older kernels)
    // - Takes pointer to struct: modifies in place
    // - Returns int: 0 = success, negative = error

    // ── Exercises ──
    // 1. Write a swap(int *a, int *b) function that swaps two ints
    // 2. Write a function that takes char buf[16] and fills it with a process name
    //    (just hardcode it). Return 0 on success.
    // 3. Why does BPF use static __always_inline for all helper functions?
    //    (hint: BPF programs can't do function calls on older kernels)
    // 4. Write a function that takes a process_info* array and its length,
    //    and prints all of them.

    return 0;
}
