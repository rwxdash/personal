// Lesson 07: The Preprocessor
//
// C has a text-processing step BEFORE compilation: the preprocessor.
// It handles #include, #define, #ifdef, and macros. It literally does
// find-and-replace on your source code before the compiler sees it.
//
// BPF uses this heavily:
//   - #include "vmlinux.h" — paste 150K lines of kernel types
//   - #define MAX_ARG_LEN 128 — constants (no const like Rust)
//   - SEC("tp/...") — macro that expands to __attribute__((section(...)))
//   - BPF_CORE_READ() — macro that generates CO-RE relocations
//   - #ifndef guards — prevent double-inclusion of headers
//
// Build: make 07-preprocessor

#include <stdio.h>

// ── #define — simple constants ──
// NOT a variable — the preprocessor replaces every "MAX_SIZE" with "1024"
#define MAX_SIZE     1024
#define MAX_NAME_LEN 16
#define PI           3.14159

// ── #define — macros with arguments ──
// Like a function, but it's text replacement (no type checking!)
#define SQUARE(x)    ((x) * (x))
#define MIN(a, b)    ((a) < (b) ? (a) : (b))
#define MAX(a, b)    ((a) > (b) ? (a) : (b))

// Why all the parentheses? Without them:
//   SQUARE(1+2) expands to (1+2 * 1+2) = 5, not 9!
// With parens:
//   SQUARE(1+2) expands to ((1+2) * (1+2)) = 9

// ── Multi-line macros (backslash continues the line) ──
#define PRINT_VAR(name, val)                                                                                           \
    do {                                                                                                               \
        printf("  %s = %d\n", name, val);                                                                              \
    } while (0)
// The do { } while(0) trick makes the macro safe to use in if/else blocks

// ── Stringification (#) and token pasting (##) ──
#define STR(x)                 #x   // turns x into a string: STR(hello) → "hello"
#define CONCAT(a, b)           a##b // pastes tokens: CONCAT(my, var) → myvar
#define PRINT_FIELD(s, f)      printf("  " #f " = %d\n", (s).f)

// ── Conditional compilation ──
#define DEBUG_MODE             1
// #define VERBOSE 1  // uncomment to enable verbose output

// ── BPF-like macros ──
// SEC() macro from bpf_helpers.h:
#define SEC(name)              __attribute__((section(name), used))

// Simplified BPF_CORE_READ-like macro:
#define READ_FIELD(ptr, field) ((ptr)->field)

int main(void)
{
    printf("── Constants ──\n");
    printf("MAX_SIZE = %d\n", MAX_SIZE);
    printf("MAX_NAME_LEN = %d\n", MAX_NAME_LEN);
    printf("PI = %f\n", PI);

    // You can use them in array sizes (unlike variables in older C)
    char name[MAX_NAME_LEN];
    (void) name;

    printf("\n── Macros ──\n");
    printf("SQUARE(5) = %d\n", SQUARE(5));
    printf("MIN(10, 3) = %d\n", MIN(10, 3));
    printf("MAX(10, 3) = %d\n", MAX(10, 3));

    // DANGER: macros evaluate arguments multiple times!
    int x = 5;
    printf("SQUARE(x++) = %d (x is now %d — evaluated twice!)\n", SQUARE(x++), x);
    // x++ happens twice in the expansion: ((x++) * (x++))

    printf("\n── Multi-line macro ──\n");
    int a = 42, b = 99;
    PRINT_VAR("a", a);
    PRINT_VAR("b", b);

    printf("\n── Stringification ──\n");
    printf("STR(hello) = %s\n", STR(hello));
    printf("STR(MAX_SIZE) = %s\n", STR(MAX_SIZE)); // "MAX_SIZE", not "1024"!

    // Useful for debug printing — print the variable name AND value
    struct {
        int pid;
        int uid;
    } proc = {42, 1000};
    PRINT_FIELD(proc, pid);
    PRINT_FIELD(proc, uid);

    printf("\n── Conditional compilation ──\n");
#if DEBUG_MODE
    printf("Debug mode is ON\n");
#else
    printf("Debug mode is OFF\n");
#endif

#ifdef VERBOSE
    printf("Verbose output enabled\n");
#else
    printf("Verbose output disabled (define VERBOSE to enable)\n");
#endif

    // You can also check from command line: gcc -DVERBOSE main.c
    // This defines VERBOSE without changing the source

    printf("\n── #ifndef include guard ──\n");
    // Every header file starts with:
    //   #ifndef __MY_HEADER_H
    //   #define __MY_HEADER_H
    //   ... contents ...
    //   #endif
    //
    // This prevents double-inclusion. If two .c files both #include "header.h",
    // the second inclusion sees __MY_HEADER_H is already defined, skips everything.
    //
    // Your profiler.bpf.h starts with:
    //   #ifndef __PROFILER_BPF_H
    //   #define __PROFILER_BPF_H
    //   #endif
    printf("Include guards prevent double-inclusion of headers\n");

    printf("\n── How SEC() works ──\n");
    // SEC("tp/sched/sched_process_exit") expands to:
    //   __attribute__((section("tp/sched/sched_process_exit"), used))
    //
    // This tells the compiler to put the function in an ELF section named
    // "tp/sched/sched_process_exit" instead of the default .text section.
    // When aya loads the ELF, it finds functions by section name.
    printf("SEC(\"tp/...\") = __attribute__((section(\"tp/...\")))\n");

    // ── Exercises ──
    // 1. Define a macro CLAMP(val, lo, hi) that clamps val between lo and hi
    // 2. What does SQUARE(1+2) expand to? Verify by printing it.
    // 3. Define EVENT_EXEC 1 and EVENT_EXIT 2 as macros. Use them in an if/switch.
    // 4. Use gcc -E 07-preprocessor.c | head -50 to see the preprocessed output.
    //    Notice how all #defines are gone — replaced with their values.
    // 5. Write a macro DEBUG_PRINT(fmt, ...) that prints only if DEBUG_MODE is set.
    //    Hint: use __VA_ARGS__ for variable arguments.

    return 0;
}
