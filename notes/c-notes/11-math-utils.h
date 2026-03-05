// Example header file — like a mini .bpf.h
//
// Pattern: include guard + static inline functions + constants

#ifndef __MATH_UTILS_H
#define __MATH_UTILS_H

#define MAX_VAL 1000

// static inline — same pattern as BPF helper functions
// static: visible only in the file that includes this header
// inline: hint to compiler to paste the body at the call site

static inline int square(int x)
{
    return x * x;
}

static inline int cube(int x)
{
    return x * x * x;
}

static inline int clamp(int val, int lo, int hi)
{
    if (val < lo)
        return lo;
    if (val > hi)
        return hi;
    return val;
}

#endif // __MATH_UTILS_H
