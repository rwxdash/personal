// Lesson 03: Arrays
//
// C arrays are NOT like Rust slices. They are:
// - Fixed size (known at compile time, or variable-length on stack in C99)
// - No bounds checking (arr[100] on a 5-element array compiles fine, crashes at runtime)
// - "Decay" to pointers when passed to functions (you lose the length!)
//
// This is critical for BPF: args[MAX_ARG_COUNT][MAX_ARG_LEN] is how we store process arguments.
//
// Build: make 03-arrays

#include <stdio.h>
#include <string.h> // memset, memcpy

int main(void)
{
    // Stack-allocated array — size must be known at compile time (or C99 VLA)
    int nums[5] = {10, 20, 30, 40, 50};

    // Access by index — no bounds checking!
    printf("nums[0] = %d\n", nums[0]);
    printf("nums[4] = %d\n", nums[4]);
    // nums[5] would compile but read garbage or crash

    // Array length — C has NO .len() method. You track it yourself.
    // This trick only works for stack arrays, NOT pointers
    size_t len = sizeof(nums) / sizeof(nums[0]);
    printf("length: %zu\n", len); // 5

    // Partial initialization — rest is zeroed
    int partial[5] = {1, 2}; // {1, 2, 0, 0, 0}
    printf("\nPartial init: ");
    for (int i = 0; i < 5; i++)
        printf("%d ", partial[i]);
    printf("\n");

    // Zero-initialize entire array
    int zeroed[5] = {0}; // all zeros
    // Or with memset:
    int also_zeroed[5];
    memset(also_zeroed, 0, sizeof(also_zeroed)); // set all bytes to 0

    // ── Array decay ──
    // When you pass an array to a function, it becomes a pointer.
    // sizeof() no longer gives the array size — it gives pointer size!
    int *p = nums; // implicit decay, no & needed
    printf("\nsizeof(nums) = %zu (full array: 20 bytes)\n", sizeof(nums));
    printf("sizeof(p)    = %zu (just a pointer: 8 bytes)\n", sizeof(p));

    // arr[i] is literally *(arr + i)
    printf("nums[2] = %d, *(nums+2) = %d\n", nums[2], *(nums + 2));

    // ── Multi-dimensional arrays ──
    // This is what BPF args look like: char args[10][128]
    // It's a 2D array: 10 rows, 128 columns, laid out flat in memory
    char names[3][16] = {
        "hello", // names[0] = "hello\0\0\0\0..."
        "world", // names[1] = "world\0\0\0\0..."
        "foo",   // names[2] = "foo\0\0\0\0..."
    };

    printf("\n2D array:\n");
    for (int i = 0; i < 3; i++)
        printf("  names[%d] = \"%s\"\n", i, names[i]);

    // Total size = 3 * 16 = 48 bytes, all contiguous
    printf("sizeof(names) = %zu\n", sizeof(names));

    // Accessing individual characters
    printf("names[1][0] = '%c'\n", names[1][0]); // 'w'
    printf("names[1][1] = '%c'\n", names[1][1]); // 'o'

    // ── memcpy — the C way to copy data ──
    int src[3] = {100, 200, 300};
    int dst[3];
    memcpy(dst, src, sizeof(src)); // memcpy(dest, src, num_bytes)
    printf("\nmemcpy result: %d %d %d\n", dst[0], dst[1], dst[2]);

    // memcpy for partial copy
    char buf[16];
    memset(buf, 0, sizeof(buf));
    memcpy(buf, "hello", 5); // copy 5 bytes
    printf("buf = \"%s\"\n", buf);

    // ── Memory layout ──
    // 2D array char x[3][4] in memory:
    // | x[0][0] x[0][1] x[0][2] x[0][3] | x[1][0] x[1][1] x[1][2] x[1][3] | x[2][0] ...
    // All contiguous. Row-major order (C convention).
    // This matters for BPF because we do: args[i] to get a pointer to row i

    printf("\nMemory layout of names (first 48 bytes):\n");
    unsigned char *raw = (unsigned char *) names;
    for (int i = 0; i < 48; i++) {
        if (i > 0 && i % 16 == 0)
            printf("  | row %d\n", i / 16 - 1);
        printf("%02x ", raw[i]);
    }
    printf("  | row 2\n");

    // ── Exercises ──
    // 1. Create char args[5][64] and fill the first 3 with "ls", "-la", "/tmp"
    //    Print each one.
    // 2. What's the total sizeof(args) for the above?
    // 3. Create a function that takes an int array and its length, returns the sum.
    //    Why can't you use sizeof inside the function to get the length?
    // 4. What happens if you memcpy more bytes than the destination holds? Try it.

    return 0;
}
