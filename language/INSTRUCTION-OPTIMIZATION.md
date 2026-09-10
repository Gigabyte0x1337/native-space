# Host-side instruction optimization

Instruction optimization is an explicit Rust tool, not a Native language
operation. The language core remains ADD, MULTIPLY, PHASE, INDEX, REFLECT.

## Why this boundary exists

REFLECT matches, binds, and rebuilds Native state. It does not silently search
for a better algorithm. Search policy and acceptance checks belong to the caller.

The host API `strand::optimize_operation_strand` decodes an unbound function's
Native graph and applies the compiler's exact rewrite allowlist. Rank must be
one. It returns a smaller expression when one is found, otherwise `None`.
Bound closures are rejected rather than silently losing their environment.

The optimization preserves classical results under the supported algebraic
identities, not the original construction history. Removing identity operations
deliberately changes that history. It is not a claim of universal optimization.

## User-authored rewrites

The host API `reflection::rewrite` performs structural source rewriting.
It does not prove equivalence or shortening. The caller must test or prove the
required behavior before using its result.

Both tools consume the same Native function representation used for execution.
Source reconstruction belongs only to these explicit host tools; ordinary calls
use the shared compiled graph and their binding environment.

Reconstruction is bounded to 1,024 nested nodes. Malformed data is rejected
instead of being interpreted as a partial program. Tests cover malformed records,
bound environments, helper dependencies, recursive references, and output
equivalence for the exact optimizer.

The accepted limitation is a finite rewrite allowlist, not general algorithm
discovery, lossy instruction removal, or hardware-specific optimization.
