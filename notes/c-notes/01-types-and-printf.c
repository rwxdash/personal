// Lesson 01: Types and printf
//
// Rust vs C type mapping:
//   Rust     C              Size
//   i8       char           1 byte (signed by default!)
//   u8       unsigned char  1 byte
//   i32      int            4 bytes
//   u32      unsigned int   4 bytes
//   i64      long long      8 bytes
//   u64      unsigned long long  8 bytes
//   f32      float          4 bytes
//   f64      double         8 bytes
//   bool     _Bool / int    C has no real bool (0 = false, nonzero = true)
//   usize    size_t         pointer-sized unsigned
//
// C has NO: String, &str, Vec, Option, Result, match, traits, generics.
// C has: raw pointers everywhere, manual memory, preprocessor macros.
//
// Build: gcc -Wall -Wextra -g -std=c11 -o 01-types-and-printf 01-types-and-printf.c
// Run:   ./01-types-and-printf

#include <stdio.h> // printf, puts

int main(void)
{
    // Integer types — no type inference, you must declare everything
    int                x    = 42;             // i32
    unsigned int       y    = 100;            // u32
    long long          big  = 123456789012LL; // i64 (LL suffix)
    unsigned long long ubig = 999ULL;         // u64

    // printf uses format specifiers — NOT like Rust's {} or {:?}
    printf("int: %d\n", x);           // %d = signed decimal
    printf("uint: %u\n", y);          // %u = unsigned decimal
    printf("long long: %lld\n", big); // %lld = long long decimal
    printf("hex: 0x%x\n", x);         // %x = hex (lowercase)
    printf("hex: 0x%X\n", x);         // %X = hex (uppercase)
    printf("u64: %llu\n", ubig);      // %llu = unsigned long long

    // Characters — in C, char is just a tiny integer
    char c = 'A';
    printf("char: %c (value: %d)\n", c, c); // 'A' is 65

    // Floats
    double pi = 3.14159;
    printf("double: %f\n", pi);   // default 6 decimal places
    printf("double: %.2f\n", pi); // 2 decimal places

    // sizeof — returns size in bytes (like std::mem::size_of)
    printf("\nSizes:\n");
    printf("  char:      %zu bytes\n", sizeof(char));
    printf("  int:       %zu bytes\n", sizeof(int));
    printf("  long:      %zu bytes\n", sizeof(long));
    printf("  long long: %zu bytes\n", sizeof(long long));
    printf("  float:     %zu bytes\n", sizeof(float));
    printf("  double:    %zu bytes\n", sizeof(double));
    printf("  size_t:    %zu bytes\n", sizeof(size_t)); // %zu for size_t

    // C has no bool keyword (C99 added _Bool, stdbool.h adds bool)
    // Most C code just uses int: 0 = false, anything else = true
    int is_ready = 1; // "true"
    int is_done  = 0; // "false"
    if (is_ready)
        printf("\nis_ready is truthy (%d)\n", is_ready);
    if (!is_done)
        printf("is_done is falsy (%d)\n", is_done);

    // DANGER: C happily converts between types without warning
    int          negative    = -1;
    unsigned int as_unsigned = negative; // No error! Wraps to 4294967295
    printf("\n-1 as unsigned: %u\n", as_unsigned);

    // DANGER: Integer overflow is undefined behavior for signed types
    // (but defined wraparound for unsigned)
    unsigned char byte = 255;
    byte += 1; // Wraps to 0 (defined for unsigned)
    printf("255 + 1 as u8: %u\n", byte);

    // ── Exercises ──
    // 1. Print the hex value of 255 (should be 0xff)
    // 2. What's sizeof(int*) on your system? (hint: it's the pointer size)
    // 3. What happens if you printf("%d\n", 3.14)? Try it. Why?
    // 4. Use %08x to print 255 with leading zeros (8-digit hex)
    int mint = 255;
    printf("\nmy int in hex: 0x%x\n", mint);
    printf("size of int* %zu bytes\n", sizeof(int *));
    printf("%d\n", 3.14);
    printf("my int in hex: 0x%08x\n", mint);

    return 0; // 0 = success (like std::process::exit(0))
}
