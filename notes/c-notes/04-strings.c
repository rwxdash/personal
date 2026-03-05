// Lesson 04: Strings
//
// C has NO String type. A "string" is just a char array ending with '\0' (null byte).
// This is exactly how BPF handles process names and filenames.
//
// Rust equivalent:
//   C string (char[16])  ≈  [u8; 16] with null terminator
//   "hello"              →  {'h','e','l','l','o','\0'}
//   strlen("hello")      →  5 (doesn't count the null)
//   sizeof("hello")      →  6 (includes the null)
//
// Build: make 04-strings

#include <stdio.h>
#include <string.h> // strlen, strcmp, strncpy, memset, memcmp

int main(void)
{
    // String literal — stored as char array with automatic \0
    char greeting[] = "hello";                   // compiler makes char[6]: {'h','e','l','l','o','\0'}
    printf("greeting: \"%s\"\n", greeting);
    printf("strlen:   %zu\n", strlen(greeting)); // 5 (no null)
    printf("sizeof:   %zu\n", sizeof(greeting)); // 6 (with null)

    // Fixed-size buffer — like BPF's char name[16]
    char name[16];                           // uninitialized — contains garbage!
    memset(name, 0, sizeof(name));           // zero it first
    strncpy(name, "bash", sizeof(name) - 1); // safe copy (reserves space for \0)
    printf("\nname: \"%s\"\n", name);

    // Show the raw bytes — see the null terminator and zero padding
    printf("bytes: ");
    for (int i = 0; i < 16; i++)
        printf("%02x ", (unsigned char) name[i]);
    printf("\n");
    // Output: 62 61 73 68 00 00 00 00 00 00 00 00 00 00 00 00
    //         b  a  s  h  \0 ...

    // String comparison — NO == for strings (that compares pointers!)
    char a[] = "hello";
    char b[] = "hello";
    printf("\na == b?      %d (WRONG — compares addresses)\n", a == b);
    printf("strcmp(a,b)? %d (CORRECT — 0 means equal)\n", strcmp(a, b));

    // strcmp returns: 0 = equal, <0 = a comes first, >0 = b comes first
    printf("strcmp(\"abc\",\"def\"): %d\n", strcmp("abc", "def")); // negative
    printf("strcmp(\"def\",\"abc\"): %d\n", strcmp("def", "abc")); // positive

    // memcmp — compare raw bytes (works for non-string data too)
    // Used in BPF for comparing comm names
    char comm[16] = "node";
    if (memcmp(comm, "node", 4) == 0)
        printf("\ncomm starts with \"node\"\n");

    // ── Building strings manually (like BPF does) ──
    char filename[128];
    memset(filename, 0, sizeof(filename));

    // snprintf — safe printf into a buffer (like format!() in Rust)
    snprintf(filename, sizeof(filename), "/usr/bin/%s", "python3");
    printf("\nfilename: %s\n", filename);

    // ── Why null termination matters ──
    char broken[8] = {'h', 'e', 'l', 'l', 'o'};        // remaining bytes are 0 (in this case)
    printf("\nbroken: \"%s\"\n", broken);              // works because rest is 0

    char really_broken[5] = {'h', 'e', 'l', 'l', 'o'}; // NO null terminator, NO room for one
    // printf("%s", really_broken); // DANGER: printf reads past the buffer looking for \0

    // ── Char array vs char pointer ──
    char        arr[] = "hello"; // copies "hello" onto the stack — mutable
    const char *ptr   = "hello"; // ptr points to read-only string in binary — immutable data

    arr[0] = 'H';                // OK: modifying stack copy
    // ptr[0] = 'H'; // CRASH: modifying read-only memory (undefined behavior)

    printf("\narr: %s\n", arr);
    printf("ptr: %s\n", ptr);

    // ── BPF pattern: reading comm name ──
    // In BPF, you call bpf_get_current_comm(name, sizeof(name))
    // It fills char name[16] with the process name, null-terminated.
    // If name is "bash", bytes are: 'b','a','s','h','\0',garbage...
    // The \0 at position 4 tells printf/strlen where the string ends.

    char comm2[16] = "python3.11"; // 10 chars + \0 + 5 zeros
    printf("\ncomm: \"%s\" (len=%zu, buf=%zu)\n", comm2, strlen(comm2), sizeof(comm2));

    // Truncation — what if the name is longer than the buffer?
    char tiny[4];
    memset(tiny, 0, sizeof(tiny));
    strncpy(tiny, "longprocessname", sizeof(tiny) - 1);
    printf("truncated: \"%s\"\n", tiny); // "lon" — truncated, but null-terminated

    // ── Exercises ──
    // 1. Create char args[3][64]. Put "ls", "-la", "/tmp" in them. Print each.
    // 2. Compare two char[16] buffers using memcmp. What if one is "bash\0garbage"
    //    and the other is "bash\0zeros"? Does memcmp(a, b, 4) work? What about
    //    memcmp(a, b, 16)?
    // 3. What does strlen return for an empty string ""?
    // 4. Write a function that takes char buf[16] and returns 1 if the string
    //    in it matches "node" (hint: use strncmp or memcmp)

    (void) really_broken; // suppress unused warning
    (void) ptr;
    return 0;
}
