// Lesson 11: Header Files and Multi-File Projects
//
// BPF projects use the "single compilation unit" pattern:
//   - One .bpf.c file (profiler.bpf.c) is compiled by clang
//   - It #includes multiple .bpf.h files (process.bpf.h, oom.bpf.h, etc.)
//   - Each .bpf.h has all its functions as static __always_inline
//   - The preprocessor pastes everything into one giant file before compiling
//
// This is different from normal C where you compile multiple .c files
// and link them together. BPF can't link, so everything must be in one unit.
//
// Build: make 11-headers

#include <stdio.h>
#include <stdint.h>

// ── Including our own header ──
// This is like "use" or "mod" in Rust, except it literally copy-pastes the file
#include "11-math-utils.h"

int main(void)
{
    printf("── Using header file functions ──\n");
    printf("  square(5) = %d\n", square(5));
    printf("  cube(3) = %d\n", cube(3));
    printf("  clamp(15, 0, 10) = %d\n", clamp(15, 0, 10));
    printf("  max_val = %d\n", MAX_VAL);

    printf("\n── How BPF headers work ──\n");
    printf("BPF uses the single compilation unit pattern:\n\n");
    printf("  profiler.bpf.c:\n");
    printf("    #include \"vmlinux.h\"       // 150K lines of kernel types\n");
    printf("    #include \"profiler.bpf.h\"  // maps + event structs\n");
    printf("    #include \"process.bpf.h\"   // exec/exit handlers\n");
    printf("    #include \"sched.bpf.h\"     // scheduler handlers\n");
    printf("    #include \"oom.bpf.h\"       // OOM handler\n");
    printf("    char LICENSE[] SEC(\"license\") = \"Dual MIT/GPL\";\n\n");
    printf("After preprocessing, clang sees ONE file with everything.\n");
    printf("Each .bpf.h has static functions — no symbol conflicts.\n");

    // ── Why static? ──
    // Without static, two .bpf.h files could define a helper with the same name
    // and you'd get "duplicate symbol" errors. static makes them file-scoped.
    // In BPF, static also means the function can be inlined.

    // ── Why __always_inline? ──
    // Older BPF (pre-5.10) doesn't support function calls between BPF functions.
    // __always_inline forces the compiler to paste the function body at each
    // call site. Modern kernels (5.10+) support BPF-to-BPF calls, but
    // __always_inline is still common practice.

    printf("\n── Include order matters ──\n");
    printf("1. vmlinux.h  (kernel types needed by everything)\n");
    printf("2. bpf_helpers.h, bpf_core_read.h  (BPF macros)\n");
    printf("3. profiler.bpf.h  (maps + structs, used by handlers)\n");
    printf("4. process.bpf.h, sched.bpf.h, etc.  (handlers use maps)\n");

    // ── Exercises ──
    // 1. Add a function to 11-math-utils.h and use it here
    // 2. What happens if you #include "11-math-utils.h" twice?
    //    Try it. Then check that the include guard prevents errors.
    // 3. Remove the #ifndef guard from the header and include it twice.
    //    What error do you get?
    // 4. Look at your profiler.bpf.c — does the include order match the
    //    pattern above?

    return 0;
}
