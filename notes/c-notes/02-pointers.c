// Lesson 02: Pointers
//
// In Rust, references are safe (&T, &mut T). In C, you have raw pointers (*T).
// No borrow checker. No lifetime tracking. You're on your own.
//
// Key syntax:
//   int *p;       — p is a pointer to int (Rust: *const i32 or *mut i32)
//   &x            — address of x (same as Rust &x, but no safety)
//   *p            — dereference p (same as Rust *p, but no null check)
//   p = NULL;     — null pointer (Rust: Option<&T>::None)
//
// Build: make 02-pointers

#include <stdio.h>
#include <stdlib.h> // NULL

int main(void)
{
    // A pointer stores the memory address of another variable
    int  x = 42;
    int *p = &x; // p points to x

    printf("x value:   %d\n", x);
    printf("x address: %p\n", (void *) &x); // %p prints addresses
    printf("p value:   %p (same as &x)\n", (void *) p);
    printf("*p value:  %d (dereferencing p gives x)\n", *p);

    // Modifying through a pointer
    *p = 99;
    printf("\nAfter *p = 99:\n");
    printf("x is now:  %d\n", x);          // x changed!
    printf("p is:      %p\n", (void *) p); // unchanged
    printf("*p is:     %d\n", *p);

    // Pointer to pointer (like **T in Rust FFI)
    int **pp = &p;
    printf("\nPointer to pointer:\n");
    printf("**pp = %d\n", **pp); // dereference twice to get x

    // NULL pointers — C has no Option, you just get NULL
    int *bad = NULL;
    printf("\nbad is NULL: %p\n", (void *) bad);
    // Dereferencing NULL = SEGFAULT (like a crash, no nice panic message)
    // Uncomment to see:
    // printf("%d\n", *bad);

    // Pointer arithmetic — pointers know the size of what they point to
    int  arr[] = {10, 20, 30, 40, 50};
    int *ap    = arr; // arrays decay to pointers (lesson 03)

    printf("\nPointer arithmetic:\n");
    printf("ap[0] = %d, *ap = %d\n", ap[0], *ap);           // same thing
    printf("ap[1] = %d, *(ap+1) = %d\n", ap[1], *(ap + 1)); // same thing
    printf("ap[2] = %d, *(ap+2) = %d\n", ap[2], *(ap + 2));

    // ap+1 doesn't add 1 byte — it adds sizeof(int) = 4 bytes
    printf("\nAddresses (notice 4-byte gaps):\n");
    for (int i = 0; i < 5; i++)
        printf("  &arr[%d] = %p\n", i, (void *) &arr[i]);

    // void* — the "any pointer" type (like *const () in Rust)
    // You can't dereference void*, must cast first
    void *generic  = &x;
    int  *specific = (int *) generic; // explicit cast required
    printf("\nvoid* cast: %d\n", *specific);

    // const pointer vs pointer to const
    const int *cp = &x; // can't modify *cp, but can move cp
    // *cp = 5;            // ERROR: can't modify through const pointer
    int y = 7;
    cp    = &y;              // OK: can point to something else

    int *const fixedp = &x;  // can modify *fixedp, but can't move fixedp
    *fixedp           = 100; // OK
    // fixedp = &y;         // ERROR: can't change where it points

    printf("x after *fixedp = 100: %d\n", x);

    // ── Exercises ──
    // 1. Create two ints, swap their values using pointers (write a swap function)
    // 2. What does sizeof(p) return? Is it the size of int or the size of a pointer?
    // 3. Create an int array of 3 elements and print each using pointer arithmetic (no [])
    // 4. What happens if you do: int *p = (int *)0x1234; printf("%d", *p); ?

    return 0;
}
