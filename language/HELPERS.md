<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Source-Defined Functions

[functions.ns](functions.ns) is the small generic standard library. Every name
and full body is ordinary source:

```ns
let opposite = (value) =>
value()
ORIENT(2)
```

There is no duplicate Rust catalog and no duplicate documentation table.

The invariants are:

1. Rust recognizes only the four core operations, never mathematical names;
2. expansion erases ordinary calls and records active self-references as finite
   pattern edges;
3. authoritative operation steps contain only ADD, MULTIPLY, ORIENT, and INDEX;
4. the only executable mathematical proof is a zero equality;
5. mathematical proof status exists only in the dependency ledger.

Use:

```powershell
native-space check language/functions.ns
native-space derive axis_subtract identity_orientation identity_orientation
native-space derive --source examples/math-functions.ns pi
native-space derive --source examples/math-functions.ns centered_re_perspective
native-space derive --source examples/math-functions.ns zeta_classical_pattern
native-space derive --source examples/recursive-pattern.ns quarter_turn_pattern
```

Without `--source`, `derive` loads only the bundled generic
`language/functions.ns` library. A function defined in an example or another
module always requires `--source` with that library path.

For the current mathematical examples,
`centered_re_perspective` expands to five primitive steps:
`ORIENT(0)`, `ADD()`, `MULTIPLY()`, `ORIENT(2)`, `ADD()`. Its report also has
eight function-trace entries. Trace entries are navigation records, not extra
primitive operations and not proof steps.

The `exact_half` part is present because the native reflection is
`1 - conjugate(s)`: it pairs real positions `sigma` and `1 - sigma`. Half of
the multiplicative identity is their unique center, so subtracting `1/2`
turns reflection into sign reversal. This centers the RE input coordinate;
the zeta value remains centered at the ADD identity `0`.

The recursive-pattern example reports one `ORIENT` operation and one pattern
reference. The reference keeps the source line, arguments, and closing
function path. It is part of the finite source graph and never enters the
authoritative primitive-operation list.

Standalone derivation libraries reuse this catalog with:

```ns
import "../language/functions.ns"
```

The relative path is resolved from the importing file. Imports only reuse source
definitions and preserve their original locations in the expanded report.

## Mathematical examples

[../examples/math-functions.ns](../examples/math-functions.ns) contains the
prime-counting camera named `pi`, plus prime, Fourier, derivative, zeta, and
perspective examples. Here `pi` means the conventional counting notation
$\pi(n)$, not the circle constant. These functions have no special runtime
status. For example:

```ns
let pi = () =>
INDEX()
ADD()
```

The statement that this function represents the classical prime-counting
camera is documented and proved separately; its name and operation trace are
not evidence of that theorem.
