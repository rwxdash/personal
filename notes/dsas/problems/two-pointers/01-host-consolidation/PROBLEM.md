# Host Consolidation

**Difficulty:** Medium
**ID:** `two-pointers-01-host-consolidation`

## Scenario

You are consolidating a fleet of virtual machines onto as few physical hosts as
possible before a datacentre migration.

Each VM has a memory footprint. Each host has the same memory capacity `cap`,
and because of the hypervisor licence tier your company bought, **a host may
run at most two VMs** — regardless of how small they are. Two VMs fit on one
host only if their footprints sum to at most `cap`.

Some VMs are **pinned**: compliance rules say they must not share hardware with
anything else, so a pinned VM always occupies a host by itself, even when there
is room to spare.

Every VM fits on a host by itself. Minimise the number of hosts.

## Task

Implement:

```python
min_hosts(footprints: List[int], pinned: List[bool], cap: int) -> int
```

- `footprints[i]` — memory footprint of VM `i`.
- `pinned[i]` — whether VM `i` must occupy a host alone.
- `cap` — memory capacity of every host.

Return the minimum number of hosts needed to place all VMs.

## Input

| Name | Type | Constraints |
| --- | --- | --- |
| `footprints` | `List[int]` | `0 <= n <= 200_000`; each value in `1 ..= 10^9` |
| `pinned` | `List[bool]` | same length as `footprints` |
| `cap` | `int` | `1 <= cap <= 10^9` |

Guaranteed: `footprints[i] <= cap` for every `i`, so a solution always exists.
Footprints may repeat.

## Output

`int` — the minimum host count. Zero VMs need zero hosts.

## Required complexity

- **Time:** `O(n log n)`.
- **Space:** `O(n)`.

At `n = 200_000`, any approach that considers VM pairs individually performs on
the order of 2 × 10^10 operations.

## Examples

### Example 1

```
footprints = [1, 2]
pinned     = [False, False]
cap        = 3
```

**Answer:** `1`

Both VMs are unpinned and `1 + 2 = 3`, exactly the capacity, so they share a
single host.

### Example 2

```
footprints = [3, 2, 2, 1]
pinned     = [False, False, False, False]
cap        = 3
```

**Answer:** `3`

The VM of size 3 fills a host on its own — pairing it with anything, even the
size-1 VM, would need capacity 4. The size-1 VM pairs with one of the size-2
VMs for exactly 3. The remaining size-2 VM cannot join it (2 + 2 = 4 > 3), so
it takes a third host. Hosts: `{3}`, `{1, 2}`, `{2}`.

Note that pairing the two size-2 VMs together is not possible either, so 3 is
genuinely the minimum — there is no arrangement using 2 hosts.

### Example 3 — pinning

```
footprints = [1, 1, 1]
pinned     = [True, False, False]
cap        = 2
```

**Answer:** `2`

The pinned VM takes a host alone even though two more size-1 VMs would fit
alongside it. The two unpinned VMs then pair up on a second host, since
`1 + 1 = 2`.

### Example 4 — nothing pairs

```
footprints = [4, 4, 4, 4, 4, 4]
pinned     = [False] * 6
cap        = 7
```

**Answer:** `6`

Any two VMs sum to 8, which exceeds the capacity of 7, so no pair fits together
and every VM needs its own host. Had `cap` been 8, the answer would be 3.

---

Stuck? Open `HINTS.md` — hints are graduated, read one level at a time.
Solved it (or surrendered)? The editorial is in
`editorials/two-pointers/01-host-consolidation/EDITORIAL.md`.
