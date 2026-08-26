<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Space Language Semantics and Compiler Correctness

## Scope and proof status

This document proves, on paper, that the Native Space 1.0 direct
interpreter, stack-bytecode compilation scheme, and theorem-gated optimizer
preserve the denotational meaning specified here. The proof concerns the
algorithms and supported AST nodes defined in `../language/SPEC.md`.

The Rust implementation is tested against these constructions but is not a
machine-checked formalization. Passing tests is implementation evidence, not a
substitute for the proofs below.

## Definitions

### D-LANG-1 -- well-formed program

A program is well formed when its final form is `output E` or `L = R`, every
function, operator, and binding name is unique, no declaration shadows a core
operation or grammar word, every call names a source-defined function with
matching arity, function bodies reference only their parameters, every
reference in a binding names an earlier binding, every result reference names a binding,
the exact-state function-call graph is acyclic, every `Add` and `Multiply` has
at least two operands, and every `Index` direction and multiplicity is
positive. All
scalar coordinates are exact rational elements of the real-field substrate.

### D-LANG-IMPORT-1 -- relative function-library import [Definition]

`import "path.ns"` loads a function library relative to the importing file.
The transitive graph must be acyclic. Each canonical file is merged once,
imports precede local definitions, all merged function names are unique, paths
are relative `.ns` paths, and every function retains its defining file.

### T-LANG-IMPORT-1 -- import resolution is deterministic [Proved]

**Statement.** Every finite well-formed import graph resolves to one ordered
function catalog independent of repeated diamond edges, while every cycle or
duplicate name is rejected.

**Dependencies.** D-LANG-IMPORT-1 and finite map/set semantics.

**Proof.** Depth-first traversal keeps an active path and a loaded set. An edge
to the active path is exactly a cycle and is rejected. An edge to the loaded
set contributes nothing, so repeated diamond edges cannot duplicate content.
Every other node recursively contributes its imports before its local
functions and then enters the loaded set. Inserting each function into the
global name set rejects exactly the first duplicate. Relative path resolution
and traversal order are deterministic, so the resulting catalog and each
retained source location are deterministic. $\square$

**Executable evidence.** `cargo test --manifest-path language/runtime/Cargo.toml`
covers relative loading,
cycle rejection, duplicate rejection, original-file traces, operator
precedence, reserved-name rejection, and core-operation shadowing. These tests
check the Rust implementation; the arguments above are the paper proofs.

### D-LANG-UTF8-1 -- exact string encoding

For Unicode string $s$ with unique UTF-8 bytes
$(b_0,\ldots,b_{m-1})$, define

$$
\mathrm{UTF8}(s)=
\bigoplus_{j=0}^{m-1}
\mathrm{INDEX}_{257+j}
\left(\mathrm{INDEX}_{b_j+1}(\mathsf1)\right).
$$

The empty fold is $\mathsf0$. Directions $1,\ldots,256$ carry byte
identity inside this encoding, while direction $257+j$ identifies byte
position $j$. Parsing lowers a string to core `Index`, `Add`, `One`, or
`Zero`; no string node reaches denotation or bytecode.

### T-LANG-UTF8-1 -- string lowering is injective and decodable [Proved]

**Statement.** D-LANG-UTF8-1 is injective, and `decode_utf8` is its left
inverse.

**Dependencies.** D-LANG-UTF8-1, D-NS-2, L-SEP-2, uniqueness of UTF-8.

**Proof.** Each nonempty term has coefficient $\mathbf1$, one byte direction
in $1,\ldots,256$, and one position direction $257+j$. The two ranges are
disjoint. Distinct positions create distinct multi-indices, so ADD cannot merge
terms. Sorting by position and subtracting one from each byte direction
recovers $(b_0,\ldots,b_{m-1})$. UTF-8 decoding then recovers $s$. Hence
`decode_utf8(UTF8(s)) = s`, which also proves injectivity. $\square$

**Boundary.** This is an exact data camera inside the untyped native carrier.
A manually constructed state may have the same shape, and arbitrary native
operations need not preserve that shape. String algorithms and
programming-language types require separate definitions.

### D-LANG-2 -- expression denotation

For an environment

$$
\rho:\text{Name}\rightharpoonup\mathcal N_{\mathcal A},
$$

define the denotation of an expression recursively:

$$
\begin{aligned}
\llbracket\texttt{zero}\rrbracket_\rho &= \mathsf0,\\
\llbracket\texttt{one}\rrbracket_\rho &= \mathsf1,\\
\llbracket\texttt{scalar}(a,b)\rrbracket_\rho &= \eta((a,b)),\\
\llbracket x\rrbracket_\rho &= \rho(x),\\
\llbracket\texttt{add}(E_1,\ldots,E_n)\rrbracket_\rho
  &= \llbracket E_1\rrbracket_\rho\oplus\cdots\oplus
     \llbracket E_n\rrbracket_\rho,\\
\llbracket\texttt{multiply}(E_1,\ldots,E_n)\rrbracket_\rho
  &= \llbracket E_1\rrbracket_\rho\star\cdots\star
     \llbracket E_n\rrbracket_\rho,\\
\llbracket\texttt{orient}(r,E)\rrbracket_\rho
  &= \mathrm{ORIENT}_r(\llbracket E\rrbracket_\rho),\\
\llbracket\texttt{index}(k,E)\rrbracket_\rho
  &= \mathrm{INDEX}_k(\llbracket E\rrbracket_\rho),\\
\llbracket\mathrm{IndexNode}(k,d,E)\rrbracket_\rho
  &= \mathrm{INDEX}_k^d(\llbracket E\rrbracket_\rho).
\end{aligned}
$$

String literals use D-LANG-UTF8-1. A call evaluates its arguments, binds the
resulting values to the called source function's parameters, and evaluates its
body. Function names have no denotational case of their own.

The displayed n-ary operations are left folds in source order. Associativity
from L-NS-2 and L-NS-5 makes the parenthesization immaterial, but no
commutative reordering is part of the language semantics.

For bindings $x_j=E_j$, begin with the empty environment and set

$$
\rho_j=\rho_{j-1}[x_j\mapsto\llbracket E_j\rrbracket_{\rho_{j-1}}].
$$

The program denotation is its final expression under the final environment.
`output E` requests that value. Surface equality `L = R` lowers to
`add(L, orient(2, R))` and requests the decidable closed judgment that this
residual equals $\mathsf0$.

### D-LANG-OPERATOR-1 -- derived binary operator [Definition]

An operator declaration

```text
operator "op" = (left, right) => body
```

defines an ordinary binary function named `op`. Infix `L op R` lowers to the
call `op(L, R)` before semantic analysis. Declaration order is the complete
precedence table: earlier declarations bind more tightly, equal precedence is
left-associative, and parentheses recurse before infix lowering. Operator names
are nonempty, whitespace-free lexical units. The one typed namespace registry
listed in `language/SPEC.md` reserves every core, exact, function, and Boolean
language-owned name. It also rejects collisions among functions, operators,
and bindings before execution.

### T-LANG-OPERATOR-1 -- operator lowering preserves denotation [Proved]

**Statement.** Every well-formed operator expression has the same denotation
as its lowered ordinary function-call expression.

**Dependencies.** D-LANG-1, D-LANG-2, D-LANG-OPERATOR-1.

**Proof.** Precedence climbing chooses the unique syntax tree determined by
declaration order, left associativity, and parentheses. Each infix node is
replaced by a `Call` with the same left and right subtrees. D-LANG-2 evaluates
that call by substituting those two denotations into the declared function
body. This is exactly the meaning assigned by D-LANG-OPERATOR-1. Structural
induction over the chosen infix tree proves the result. Because lowering emits
only existing `Call` nodes, interpreter and compiler semantics require no new
case. $\square$

### D-LANG-3 -- bytecode machine

A VM configuration consists of a program counter, a stack of native states,
and a finite slot vector. `PUSH_*` appends the corresponding state, `LOAD`
appends one initialized slot, `STORE` removes the stack top into one slot,
`ADD(n)` and `MULTIPLY(n)` replace the top $n$ states by their source-order
fold, while `ORIENT(r)` and `INDEX(k,d)` transform the stack top,
and `HALT` returns the unique stack value.

Malformed operands, invalid slots, stack underflow, invalid arity or direction,
and an invalid final stack are outside valid compilation and produce typed VM
diagnostics rather than a native value. Bytecode carries the final goal as
data. The VM returns the denotation without interpreting that goal; the checker
compares the direct and compiled results, then applies the goal.

### D-LANG-4 -- compilation

Source calls are recursively erased by parameter substitution before bytecode
generation. Expression compilation is then compositional:

- constants emit their matching `PUSH` instruction;
- a reference emits `LOAD` for its earlier binding slot;
- n-ary operations compile operands from left to right and emit their arity;
- unary operations compile their operand and then emit the operation;
- every INDEX node emits one `INDEX` instruction carrying its direction and
  exact native multiplicity.

A binding compiles its expression followed by `STORE` into a fresh slot. The
final expression is followed by `HALT`, and the source goal and selected output
camera are copied into the bytecode artifact.

### D-LANG-5 -- optimizer

The optimizer recursively optimizes child expressions and may then apply only
the rewrites listed in `../language/SPEC.md`. Each emitted rewrite event names
the stable theorem IDs authorizing that rewrite. The executable allowlist is
checked against `00-dependency-ledger.md` by the test suite.

### D-LANG-6 -- canonical serialization

AST, bytecode, and runtime-state serialization are tagged, versioned where the
artifact can evolve independently, and encode exact rationals as numerator or
fraction strings and arbitrary-size INDEX depths as nonnegative decimal
strings. Decoding checks each constructor's required field types and
reconstructs immutable values. Runtime-state decoding restores canonical term
order, combines duplicate indices, and removes exact zero coefficients. The
runtime-state tag is the default camera and version `flat-stack-v1`; a missing
or different camera tag is rejected rather than interpreted as lossless state.

## Interpreter correctness

### T-LANG-INTERP-1 -- direct evaluation equals denotation [Proved]

**Statement.** For every well-formed program $P$, direct evaluation
terminates and returns $\llbracket P\rrbracket$.

**Dependencies.** D-LANG-1, D-LANG-2, D-NS-3 through D-NS-10.

**Proof.** Proceed by structural induction on expressions. Each base node is
returned using exactly its D-LANG-2 constructor. A reference is present in the
environment by D-LANG-1. In the induction cases, recursive calls return the
child denotations; the interpreter then applies the same native ADD,
MULTIPLY, ORIENT, or INDEX operation as D-LANG-2. The AST is finite, so the
recursion terminates.

Proceed next by induction over the finite binding sequence. Initially both the
semantic and interpreter environments are empty. Assuming equality through
binding $j-1$, expression correctness gives the same value for $E_j$, so
both environments extend with the same $x_j$ value. Expression correctness
then applies to the final result. $\square$

## Compiler correctness

### L-LANG-COMP-1 -- expression stack invariant [Proved]

**Statement.** Let the VM slots agree with environment $\rho$ for every
reference in well-formed expression $E$. Starting from any valid stack $S$,
executing the instructions compiled for $E$ terminates with the same slots
and stack

$$
S\mathbin{+\!+}[\llbracket E\rrbracket_\rho].
$$

**Dependencies.** D-LANG-1 through D-LANG-4.

**Proof.** By structural induction on $E$. Constant and reference
instructions append exactly their denotations. For an n-ary node, apply the
induction hypothesis to each operand in source order. The stack then ends in
the n child denotations; the arity instruction replaces precisely those values
with the D-LANG-2 fold and leaves the preceding stack untouched. For a unary
node, the induction hypothesis appends the child value and the following
instruction replaces it by the corresponding ORIENT or INDEX denotation.
`INDEX(k,d)` applies exact repeated INDEX multiplicity in one typed
instruction. No expression
instruction stores a slot. $\square$

### T-LANG-COMP-1 -- compiled execution equals direct evaluation [Proved]

**Statement.** For every well-formed program $P$, compilation followed by VM
execution returns the same canonical native state as direct evaluation:

$$
\mathrm{VM}(\mathrm{compile}(P))
=\mathrm{interpret}(P)
=\llbracket P\rrbracket.
$$

**Dependencies.** T-LANG-INTERP-1, L-LANG-COMP-1, D-LANG-4.

**Proof.** Induct over bindings. Before the first binding the environment and
slot prefix are both empty and the VM stack is empty. L-LANG-COMP-1 appends the
binding denotation; `STORE` removes it into the fresh matching slot, restoring
the empty stack. Thus slots and environment agree after every binding. Applying
L-LANG-COMP-1 to the final expression leaves exactly its denotation on the
stack, and `HALT` returns it. T-LANG-INTERP-1 identifies that denotation with
direct evaluation. $\square$

### T-LANG-ZERO-1 -- closed finite zero checking is sound and complete [Proved]

**Statement.** For every well-formed closed finite zero equality $P$, the
checker accepts exactly when
$\llbracket P\rrbracket=\mathsf0$.

**Dependencies.** T-LANG-INTERP-1, T-LANG-COMP-1, D-NS-2, D-NS-3.

**Proof.** T-LANG-INTERP-1 says direct evaluation returns
$\llbracket P\rrbracket$. T-LANG-COMP-1 says compiled execution returns the
same state. The checker first requires those two states to be equal. By D-NS-2
and D-NS-3, the canonical state equals $\mathsf0$ exactly when it contains no
nonzero coefficient. The checker accepts exactly under that condition. Hence
acceptance is equivalent to $\llbracket P\rrbracket=\mathsf0$. $\square$

**Boundary.** This decision procedure proves a concrete closed finite
instance. It does not quantify over symbolic states, prove a paper theorem by
testing examples, or decide an infinite or analytic equality.

The canonical-state condition also implements C-CAMERA-RESIDUAL-1 for the
default perspective: opposite signed coordinates at one retained INDEX are
combined automatically, zero coefficients disappear, and only nonzero
residual terms remain. Distinct retained indices are not silently aggregated.
The generic source function `zero_fill_axes(parts...)` implements
C-CAMERA-ZERO-FILL-1 at the proof-language layer: a perspective passes its own
axis declaration and residual pattern immediately before comparing the
resulting frame with zero. No concrete perspective is privileged by the
language kernel.

## Optimizer correctness

### T-LANG-OPT-1 -- every admitted rewrite preserves denotation [Proved]

**Statement.** For every well-formed program $P$, the optimized program is
well formed and

$$
\llbracket\mathrm{optimize}(P)\rrbracket
=\llbracket P\rrbracket.
$$

**Dependencies.** D-LANG-5, L-NS-2, L-NS-5, L-NS-6, L-NS-8,
L-SEP-5.

**Proof.** Use structural induction. Child optimization preserves each child
denotation by the induction hypothesis. Flattening ADD uses associativity and
removing `zero` uses its identity law, both in L-NS-2. Flattening MULTIPLY uses
L-NS-5, removing `one` uses L-NS-6, and replacing a product containing `zero`
uses L-NS-8. Reducing turns modulo four and removing or combining nested
ORIENT nodes use the state-level cycle and composition law L-SEP-5.
INDEX only rebuilds its node around an equivalent child while preserving its
native multiplicity. No rewrite changes bindings or references, so well-formedness is
preserved. $\square$

## Serialization correctness

### T-LANG-SER-1 -- supported artifact round trips [Proved]

**Statement.** For every valid 1.0 core AST $A$, bytecode program $B$, and
canonical runtime state $F$, decoding the encoded artifact reproduces it:

$$
\mathrm{decode}(\mathrm{encode}(A))=A,\qquad
\mathrm{decode}(\mathrm{encode}(B))=B,\qquad
\mathrm{decode}(\mathrm{encode}(F))=F.
$$

**Dependencies.** D-LANG-6, D-NS-1, D-NS-2.

**Proof.** For AST expressions, structural induction applies because every
constructor has a distinct tag and its scalar fields, spans, and children are
encoded componentwise; exact rational string parsing is inverse to rational
string formatting. Bindings and programs follow componentwise. For bytecode,
each opcode has a unique string and each allowed operand is one of `null`, an
integer, a direction-depth pair, or a tagged scalar; instruction order, slots,
source name, spans, and
version, final goal, and output camera are retained. For a canonical state, the
`flat-stack-v1` camera tag,
every index pair with its arbitrary-size decimal depth, and every coefficient
round trip componentwise. Canonical
reconstruction cannot merge, remove, or reorder the already unique, nonzero,
sorted terms, so it returns $F$.
$\square$

## What these results do not establish

- They do not prove the Rust source correct by formal verification; the test
  suite checks conformance through examples, generated ASTs, and round trips.
- They do not establish performance, compression, novelty, or application
  usefulness.
- They do not cover malformed bytecode with a native denotation; malformed
  artifacts are required to fail diagnostically.
- They do not cover future approximate values, camera execution, conventional
  prime-value lookup, automatic factorization, loops, unbounded recursive
  execution, effects, or compiler targets other than schema-1 bytecode.
- Source-function self-reference **is** covered separately by D-FLANG-2 and
  T-FLANG-SELF-1 in `14-analytic-language-semantics.md`: an active repeated
  call becomes one finite pattern-reference edge. What is not covered is
  repeatedly unfolding that edge as an unbounded computation or treating it
  as a completed exact state.
