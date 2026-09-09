<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Source-defined functions

[functions.ns](functions.ns) is the small generic derivation library.
Names and bodies are ordinary source; mathematical names have no privileged
runtime behavior.

```ns
let opposite = (value) =>
value()
PHASE(2)
```

Expansion records ADD, MULTIPLY, PHASE, and INDEX operations. Calls to an
already active function are retained as finite pattern references. An
operation listing describes source structure; it does not establish a
mathematical theorem.

```sh
native-space check language/functions.ns
native-space derive axis_subtract identity_phase identity_phase
```

Without `--source`, derivation uses the bundled generic library.
Use `native-space derive --source FILE FUNCTION` for your own library.
Imports resolve relative to the importing file and preserve the original
source location of each operation.
