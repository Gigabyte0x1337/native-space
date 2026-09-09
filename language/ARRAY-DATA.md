<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Array data

Arrays are host input notation, not a Native Space value kind. The runtime
lowers every nonempty rectangular array to an ADD/INDEX graph of scalar inputs,
including zero leaves. Its classical camera is sparse; its native state retains
every input location.

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
- `run --data` passes all items to one source function;
- `untrace --input` treats their classical projections as one observation sequence.

Native graph objects are also valid items. A saved batch output supplies its
`states` field, not its displayed `results`, when read again. This permits
save/load/feedback without reconstructing from classical zero.

Each root item may itself be a rank-1 through rank-64 array. Empty, ragged, and
mixed-rank arrays are rejected. Exact real strings and exact `{ "real": ...,
"imag": ... }` scalar objects are valid leaves. Shape is retained by the host
only to reconstruct readable output, including locations whose coefficient
cancels to zero.

`untrace --input` reads the complete JSON or binary file into memory before
synthesis. It uses each item's complete indexed classical projection and keeps
one recurrence state across the full order. Native history is not an extra
numerical feature. There is no chunk reset or conversion through the lossy
floating-point frequency camera. Synthesis algorithms are unchanged.

## Binary form

`pack-data` validates JSON input and writes the same data in the versioned
`NSBATCH` binary form:

```text
native-space pack-data data.json data.nsb
native-space batch program.ns --function step --data data.nsb --steps 1 --backend cpu
```

Version 2 intentionally replaces the projection-only version 1 payload. This
does not change the language version, which remains 1.0. No compatibility
decoder invents missing history. All fixed-width integers are little-endian:

- Header: eight bytes `NSBATCH\0`, `u16` version 2, `u16` flags 0, `u64` item count.
- Item: `u16` shape rank (`65535` means no shape), optional `u64` extents,
  then `u64` native-node count. The last node is the root.
- Node tag `u8`: 0 scalar, 1 ADD, 2 MULTIPLY, 3 PHASE, 4 INDEX, 5 camera read.
- Scalar payload: squared magnitude text, `u8` ray presence (0 or 1), and
  real/imaginary ray text when present. Text has a `u32` UTF-8 byte length.
- PHASE payload: `u8` canonical quarter-turn count. INDEX payload: `u64`
  direction and `u64` depth. ADD/MULTIPLY have no extra payload.
- Camera payload: `u64` source direction and `u64` destination direction.
  Reads remain graph nodes rather than being replaced by projected constants.
- Every node ends with its operand edges and then its scope edges: each list
  has a `u64` count followed by `u64` backward node references. Sharing survives.

JSON and binary use one graph validator. Invalid operators, forward edges,
unreachable nodes, noncanonical exact coordinates, invalid shapes, unsupported
versions/flags, truncated fields, and trailing bytes are errors. The codec uses
typed records without JSON parsing; no parsing speedup is claimed.

Shape validation does not allocate a dense array. Readable output materializes
at most one million elements; larger shapes use sparse/scalar observations
while keeping their full native state and shape metadata. This display budget
does not truncate data or alter the computation. Keeping zero leaves and scope
edges costs storage deliberately; this is not a sparse-history compressor.
