# C Notes — Zero to eBPF

Crash course in C aimed at getting up to speed for writing eBPF programs. Each lesson is a standalone `.c` file with heavy annotations, Rust comparisons, and exercises. The lessons progress from basic C to real BPF patterns.

## Lessons

| # | File | Topic |
|---|------|-------|
| 01 | `01-types-and-printf.c` | Types, printf format specifiers, Rust↔C type mapping |
| 02 | `02-pointers.c` | Raw pointers, `&`, `*`, NULL, pointer arithmetic |
| 03 | `03-arrays.c` | Fixed-size arrays, decay to pointers, no bounds checking |
| 04 | `04-strings.c` | Null-terminated strings, `strlen`/`strcmp`/`strncpy`/`memcmp` |
| 05 | `05-structs.c` | Struct layout, padding, `typedef`, packed structs |
| 06 | `06-functions.c` | Pass-by-value, pointer params, `static`, `__always_inline` |
| 07 | `07-preprocessor.c` | `#define`, `#include`, `#ifdef`, macros, `SEC()` |
| 08 | `08-bitwise.c` | Bit ops, masks, shifts, pid/tgid extraction, exit codes |
| 09 | `09-casting-and-void-ptr.c` | Casts, `void *`, type punning (BPF helpers return `void *`) |
| 10 | `10-memory-layout.c` | Stack vs heap, BPF's 512-byte stack limit, scratch maps |
| 11 | `11-headers.c` | Header files, include guards, BPF single compilation unit |
| 12 | `12-bpf-helpers-sim.c` | Simulated BPF helpers (`bpf_get_current_pid_tgid`, maps, ringbuf) |
| 13 | `13-bpf-real-patterns.c` | Real BPF C patterns — tracepoints, CO-RE, argv reading, filtering |

## Build & Run

```sh
# build everything
make

# build one lesson
make 05-structs

# build and run
make run-05-structs

# generate .clangd and compile_commands.json for editor support
make setup

# clean
make clean
```

Requires `gcc` and optionally `clangd` for in-editor diagnostics.

## How to work through them

1. Read each file top-to-bottom — the comments explain everything
2. Compile and run it to see the output
3. Do the exercises at the bottom of each file
4. Break things on purpose (remove a NULL check, overflow a buffer) to see what happens
5. Lessons 12-13 bridge into real eBPF — compare with your actual BPF code
