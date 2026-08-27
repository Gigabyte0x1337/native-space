<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Array data

Arrays are host input notation, not a Native Space value kind. The runtime
lowers every nonempty rectangular array to one ordinary sparse state made only
from ADD and INDEX.

For a rank-$r$ array, array axis $a$ uses INDEX direction $a$. A 1-based
position $p$ on that axis applies INDEX $p$ times. Therefore a leaf at
position $(p_1,\ldots,p_r)$ is

```text
index(1 repeated p1, index(2 repeated p2, ... value))
```

For example:

```text
[x, y]
-> add(index(1, x), index(1, index(1, y)))

[[a, b], [c, d]]
-> add(
     index(1, index(2, a)),
     index(1, index(2, index(2, b))),
     index(1, index(1, index(2, c))),
     index(1, index(1, index(2, index(2, d))))
   )
```

This choice is intentional. INDEX composition is commutative, so using array
positions as direction names would make positions such as $(1,2)$ and $(2,1)$
collide. Axis-as-direction and position-as-depth keeps every array location
distinct while using only the existing native operation.

The JSON root is ordered. Its interpretation belongs to the host command:

- `batch` treats every root item as an independent data point;
- `untrace --input` treats all root items together as one observation sequence.

Each root item may itself be a rank-1 through rank-64 array. Empty, ragged, and
mixed-rank arrays are rejected. Exact real strings and exact `{ "real": ...,
"imag": ... }` scalar objects are valid leaves. Shape is retained by the host
only to reconstruct readable output, including locations whose coefficient
cancels to zero.

`untrace --input` reads the complete JSON or binary file into memory before
synthesis. It preserves every root item as one complete native observation and
keeps one recurrence state across the full order. There is no chunk reset,
coordinate flattening, or conversion through the lossy frequency camera.

## Binary form

`pack-data` validates JSON input and writes the same data in the versioned
`NSBATCH` binary form:

```text
native-space pack-data data.json data.nsb
native-space batch program.ns --function step --data data.nsb --steps 1 --backend cpu
```

The runtime detects the format from its magic bytes. Version 1 uses
little-endian fixed-width counts and stores, for every item:

1. optional host-array rank and extents;
2. the sparse native term count;
3. each INDEX direction and arbitrary-size decimal depth;
4. each exact rational real and imaginary coefficient as length-prefixed UTF-8.

The binary decoder rejects unsupported versions, flags, zero extents, zero or
duplicate INDEX entries, out-of-shape positions, truncated fields, invalid
UTF-8 or rationals, and trailing bytes. It avoids JSON syntax parsing; no
performance gain is claimed without a benchmark.

The format stores the native sparse state rather than a dense array. This keeps
the operation representation exact and compact when many input coefficients
are zero. The retained shape is presentation metadata and never participates
in evaluation.
