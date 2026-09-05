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
matching fixed or trailing-variadic arity, function bodies reference only
their parameters, every variadic parameter is spread only inside an argument
or operand list, every
reference in a binding names an earlier binding, every result reference names a binding,
every function path executed by the exact-state result is acyclic, every
`trace(function)` target names a source function, every `untrace(value)` or
`untrace(value, rank)` has one finite value expression and an exact rational
rank from zero through one, every `rank_descent(value)` has one finite value
expression, every optional static target rank is exact and greater than zero
through one, every optional minimum agreement is exact from zero through one,
every `apply` has a direct rank-descent pattern and a positive integer
position, every `Add` and `Multiply` has
at least two explicit operands or contains a variadic spread, every `Concat`
has a positive direction and at least one source value, every `Fold` names one
nonvariadic two-parameter source function and has a finite value list, every
`Camera` has a positive source direction and nonnegative destination direction,
every `Orient` turn is one canonical member of $\mathbb{Z}_4=\{0,1,2,3\}$,
and every `Index` direction and multiplicity is positive. All
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

### D-LANG-CONCAT-1 -- variadic pack and concat elaboration [Definition]

The final parameter of an exact-state function may be marked `...`. At a call,
its finite remaining argument expressions form an ordered source pack. A spread
`p...` substitutes those expressions, in order, into an argument or operand
list. It is not a native value.

For positive direction $d$ and nonempty expanded values
$(v_1,\ldots,v_n)$, define

$$
\mathrm{CONCAT}_d(v_1,\ldots,v_n)
=\bigoplus_{j=1}^{n}\mathrm{INDEX}_d^j(v_j).
$$

The elaborator performs pack substitution and this rewrite before bytecode
generation. An empty ADD spread lowers to ZERO and an empty MULTIPLY spread to
ONE. Empty concat is rejected.

### T-LANG-CONCAT-1 -- concat generates pure native source [Proved]

**Statement.** Every finite well-formed variadic call and concat expression
elaborates deterministically to a finite expression containing no spread or
concat node. Its concat result equals the D-LANG-CONCAT-1 ADD/INDEX expression
under both direct evaluation and compiled execution. If direction $d$ is
absent from every $v_j$, each source position occupies a distinct INDEX depth.

**Dependencies.** D-LANG-1, D-LANG-CONCAT-1, D-NS-2, D-NS-5,
T-LANG-INTERP-1.

**Proof.** A finite call supplies a finite ordered argument list. Binding the
fixed prefix consumes a unique prefix; the remaining ordered suffix binds the
single trailing pack. Replacing each spread with that suffix strictly removes
one spread occurrence and inserts only finitely many already finite
expressions. Concat then maps source position $j$ to the explicitly written
term $\mathrm{INDEX}_d^j(v_j)$ and joins the finite list with ADD. Thus the
result is finite, deterministic, and contains only ordinary expression nodes.
Direct evaluation applies the same position map. Compilation first performs
the rewrite and therefore emits only existing constant, ADD, and INDEX
instructions. When $d$ is absent from every value, the resulting depths are
exactly $1,\ldots,n$, so distinct positions cannot merge. $\square$

**Boundary.** This is source elaboration, not arbitrary parameter compression.
A generated parameter function may represent a large repeated pattern, but
unrelated information must still occur in that function or its exact residual.
If a value already uses direction $d$, INDEX depths add normally and concat is
not guaranteed to remain position-injective.

### D-LANG-FOLD-1 -- finite source fold [Definition]

For one ordinary binary source function $f$, initial value $z$, and finite
ordered values $(v_1,\ldots,v_n)$, define

$$
\mathrm{FOLD}(f,z,v_1,\ldots,v_n)
=f(\cdots f(f(z,v_1),v_2)\cdots,v_n).
$$

The empty fold equals $z$. A trailing source spread supplies its finite values
in argument order. The elaborator substitutes the nested calls before
bytecode generation.

### T-LANG-FOLD-1 -- fold is exactly nested source application [Proved]

**Statement.** Every well-formed finite fold returns the same state as the
D-LANG-FOLD-1 nested call expression, and its closed expansion contains no
fold node or fold bytecode.

**Dependencies.** D-LANG-1, D-LANG-FOLD-1, D-LANG-2.

**Proof.** Pack substitution produces one unique finite ordered value list.
Beginning with $z$, the elaborator replaces the accumulator after item $j$ by
the ordinary call $f(a_{j-1},v_j)$. Induction on $j$ gives exactly the displayed
nested expression; for $n=0$ no replacement occurs and the result is $z$.
Every step strictly consumes one item, so expansion terminates and introduces
only ordinary calls. Existing call expansion removes those calls before
bytecode. Direct host evaluation performs the same induction over the same
ordered list. $\square$

### D-LANG-CAMERA-1 -- indexed direction camera [Definition]

Let $F=\sum_\alpha c_\alpha e_\alpha$ be a finite native state, let $a>0$, and
let $b\geq0$. For every $\alpha$ with depth $d=\alpha(a)>0$, let
$\rho_{a\to b}(\alpha)$ remove direction $a$ and, when $b>0$, add depth $d$ to
direction $b$. Define

$$
C_{a\to b}(F)=
\sum_{\alpha:\alpha(a)>0}c_\alpha e_{\rho_{a\to b}(\alpha)}.
$$

Canonical ADD combines equal output indices and removes zero coefficients.
Destination zero therefore unwraps the selected direction. Terms without the
source direction are outside the selected perspective and are discarded.

### T-LANG-CAMERA-1 -- camera selection and remapping are exact [Proved]

**Statement.** $C_{a\to b}$ is a deterministic finite linear camera. A closed
camera expression lowers to one ordinary finite state with exactly the same
denotation under direct evaluation and bytecode. If $b>0$ is absent from every
selected input term, the map on selected multi-indices is injective.

**Dependencies.** D-LANG-CAMERA-1, D-NS-2, D-LANG-2,
T-LANG-INTERP-1.

**Proof.** The support of $F$ is finite, and filtering it cannot enlarge it.
Each retained multi-index has one uniquely determined depth $d$ and one unique
image under $\rho_{a\to b}$, so the resulting finite coefficient fold is
deterministic. Coefficients are unchanged before canonical ADD; distributing
over two finite supports proves linearity. Closed staging evaluates this same
finite definition and serializes the resulting canonical terms as exact
scalar, ADD, and INDEX expressions. D-LANG-2 and T-LANG-INTERP-1 then give the
same state under the VM. If $b$ is fresh, its output depth recovers $d$ and
replacing $b$ by $a$ recovers the input index, proving injectivity on the
selected support. $\square$

**Boundary.** The camera is explicitly lossy when it discards unselected terms
or when destination removal/remapping merges distinct indices. It is generic
coordinate infrastructure, not a learned feature, ranking rule, or AI model.

### D-LANG-TRACE-1 -- reflective operation strand [Definition]

For one parsed source function $f$, `trace(f)` traverses the finite graph of
source functions reachable from $f$. It writes each definition once and keeps
every call as an explicit edge. Its result is the nested native strand

$$
\mathrm{Node}(h,t)=
\mathrm{ADD}
\left(
\mathrm{INDEX}_{H}(h),
\mathrm{INDEX}_{C}(t)
\right),
$$

where $h$ is one complete instruction coordinate and $t$ is the continuation.
Instruction coordinates have disjoint fields for kind, opcode, names, numeric
arguments, UTF-8 bytes, byte positions, and source span. ADD, MULTIPLY, ORIENT,
and INDEX use opcode orientations 0, 1, 2, and 3 respectively. Trace, length,
untrace, rank descent, pattern application, spread, concat, fold, and camera syntax have their own
non-opcode instruction kinds;
the function-start coordinate records whether its final parameter is variadic.
Prefix arities preserve the expression tree. Repeated continuation indexing
preserves exact strand order. Zero is the unique terminal continuation.

`trace` is a reflective language function, not an algebra operation. Its
result expression contains only exact constants, ADD, ORIENT, and INDEX. A
recursive source call remains a call edge; it is never repeatedly executed by
the trace construction.

### T-LANG-TRACE-1 -- trace preserves the finite source graph [Proved]

**Statement.** `trace(f)` terminates for every finite well-formed source graph
reachable from $f$, and its operation-strand coordinate uniquely reconstructs
that parsed graph, including direct and mutual self-reference.

**Dependencies.** D-LANG-TRACE-1, D-NS-2, L-SEP-2, uniqueness of UTF-8,
finite map and set semantics.

**Proof.** The traversal keeps a visited function-name set. A function not in
the set contributes its finite parameter list and finite expression tree, then
enters the set. A function already in the set contributes only the call edge
already present in its caller. Therefore at most the finite number of reachable
definitions is traversed, so construction terminates even on a cycle.

For reconstruction, continuation depth selects exactly one strand position.
Within that position, the head, kind, opcode, text, number, and span directions
are disjoint. Text byte position is an INDEX depth and its positive coefficient
is the byte plus one, so no byte disappears as zero. The instruction kind and
arity determine the unique prefix-tree boundary. Function-start and
function-end coordinates delimit each definition, while named call coordinates
recover every graph edge. Reading increasing continuation depth therefore
recovers the same parsed reachable graph. Hence the encoding is injective on
that graph and preserves self-reference without unfolding it. $\square$

**Boundary.** Comments and whitespace are discarded by parsing and are not
instructions. Executing a recursive exact-state call remains invalid; this
theorem concerns finite source observation, not unbounded computation or
optimization performance.

### D-LANG-LENGTH-1 -- operation-strand length camera [Definition]

In a canonical D-LANG-TRACE-1 strand with $n$ coordinates, the unique kind
marker of coordinate $j$ occurs at continuation depth $j$ for
$0\leq j<n$. Define

$$
L(S)=\mathrm{INDEX}_1^n(\mathrm{ONE}).
$$

`length(S)` is defined only for that canonical consecutive operation-strand
shape. It is a reflective camera and is lowered to ONE and INDEX before
bytecode generation.

### T-LANG-LENGTH-1 -- native length composition [Proved]

**Statement.** `length(trace(f))` is the unary native strand whose INDEX depth
equals the number of coordinates in the complete finite trace of $f$. For
canonical strands $P,Q$ and positive integer $k$,

$$
L(P)\mathbin{\mathrm{MULTIPLY}}\mathrm{INDEX}_1^k(\mathrm{ONE})=L(Q)
$$

holds exactly when the coordinate length of $Q$ exceeds that of $P$ by $k$.

**Dependencies.** D-LANG-TRACE-1, D-LANG-LENGTH-1, D-NS-5, D-NS-6.

**Proof.** D-LANG-TRACE-1 nests every successive coordinate under one
additional continuation INDEX, so its kind-marker depths are exactly
$0,\ldots,n-1$. The camera checks that consecutive set and emits depth $n$.
Native MULTIPLY composes monomials by adding their multi-indices. Hence depths
$a$ and $k$ produce depth $a+k$, which equals depth $b$ exactly when
$b=a+k$. Requiring $k\geq1$ makes the witness strict. Both direct evaluation
and compilation lower the camera to the same ONE/INDEX expression. $\square$

**Boundary.** This measures canonical trace-coordinate count. It does not
measure serialized bytes, execution time, mathematical complexity, semantic
equivalence, or global program minimality. Provenance and spans remain in the
trace but do not create instruction positions.

### D-LANG-UNTRACE-1 -- deterministic-or-relationship discovery [Definition]

Let the decoded ordered observations be complete canonical native states
$X_0,\ldots,X_{n-1}$. A CLI JSON or `NSBATCH` root supplies that sequence
directly and is retained in memory as one continuous synthesis state. In the
compact scalar source layout, consecutive one-depth INDEX directions are the
observation positions and missing directions inside the retained span are zero
states. In the structured source layout, INDEX direction 1 is the sequence
axis, its depth is the 1-based position, and every remaining coordinate is the
payload of that state.

For a candidate order $r$ with $2r<n$, the deterministic grammar is

$$
X_j=a_1\star X_{j-1}\oplus\cdots\oplus a_r\star X_{j-r},
$$

where $a_1,\ldots,a_r$ are exact native scalars and scalar multiplication acts
on every coordinate of the complete state. The same coefficients must satisfy
every coordinate; coordinates are neither flattened nor fitted independently.

The first $r$ states are seeds. Exact Gaussian elimination over native complex
rationals uses every coordinate equation at positions $r$ through $2r-1$ to
determine the leftmost-pivot solution, setting free coefficients to zero.
Every later supplied state is held out from coefficient construction. Starting
from the seeds, the candidate recursively generates
$\widehat X_r,\ldots,\widehat X_n$ without replacing a generated state by a
supplied observation. It is accepted exactly when

$$
\widehat X_j=X_j\qquad(2r\leq j<n).
$$

Candidate ordering minimizes seed nodes plus recurrence-expression nodes;
source byte length, lower order, and lexical source order break ties. Search is
finite over $1\leq r\leq32$ with $2r<n$. An accepted result contains the exact
next-state function, recursive continuation, and seed entry function. `trace`
encodes that finite self-referential graph as an operation strand.

If no supported order satisfies every held-out equality, relationship mode is
used. Assign each distinct complete state its first-occurrence symbol id
$s(X_i)$. For every triple $(a,b,d)$ with $d>0$, define the exact frequency

$$
F(a,b,d)=|\{(i,j):i<j,\ s(X_i)=a,\ s(X_j)=b,\ j-i=d\}|.
$$

Let $m$ be the number of triples with nonzero frequency. `untrace(value)` has
rank $\rho=1$. `untrace(value,rho)` requires
$\rho\in\mathbb{Q}\cap[0,1]$ and retains
$q=\lceil\rho m\rceil$ channels, ranked by descending $F$ and then ascending
$(a,b,d)$. The dictionary entry for symbol $a$ has coefficient
$C(a)=|\{i:s(X_i)=a\}|$. The native result includes this complete counted
symbol dictionary and the retained channel coordinates. Six fresh INDEX directions greater than every
input direction distinguish dictionary entries, dictionary data, relation
terms, left ids, right ids, and distance. Relationship construction has an
explicit two-million-pair budget.

### T-LANG-UNTRACE-1 -- discovery is exact and deterministic [Proved]

**Statement.** `untrace(E,rho)` returns deterministic mode exactly when at
least one recurrence in the supported grammar regenerates every held-out state.
Otherwise it returns the exact frequencies of every ordered input pair, retains
exactly the rank-selected channel prefix, and preserves a dictionary for every
distinct complete input state together with its exact observation count.
Omitted rank equals one.

**Dependencies.** D-LANG-UNTRACE-1, D-LANG-TRACE-1, exact field elimination,
finite bounded search, finite map counting, D-NS-2.

**Proof.** For fixed order $r$, each training-state equality is equivalent to
the collection of its scalar coordinate equations, with absent coordinates
equal to exact zero. Row reduction applies invertible field row operations,
rejects an inconsistent row, chooses pivots left to right, and sets free
variables to zero. It therefore returns one deterministic shared coefficient
solution to all training coordinates when one exists. Recursive generation and
canonical-state equality accept precisely candidates with no held-out
mismatch. Since $2r<n$, at least one compared state was absent from coefficient
construction.

There are finitely many supported orders. Each accepted candidate has a finite
cost tuple, and total tuple ordering selects one unique minimum. Its function
graph has finitely many definitions and one recursive edge, so T-LANG-TRACE-1
encodes it as one finite injective operation strand.

If that set is empty, the two nested finite position loops visit each pair
$i<j$ once. Incrementing the corresponding map entry therefore produces
exactly $F(a,b,d)$. Exact rational ceiling computes $q$ without floating point,
and the stated total ordering selects one unique prefix. Fresh directions
cannot collide with input directions or with each other. Dictionary symbol-id
depth distinguishes complete input states, while the four relation fields
distinguish every retained $(a,b,d)$ channel. Canonical ADD combines exactly
equal channels into their exact counts. $\square$

**Boundary.** Exact recurrence agreement does not prove that an unseen
prediction is correct or that the grammar contains the source process.
Relationship frequencies describe the supplied finite observations; they do
not establish statistical generalization, causality, or an AI model. Rank is
channel retention, not confidence or tolerated deterministic error. Existing
operation strands are fixed points of `untrace`.

### D-LANG-RANK-1 -- finite relationship replay and rank search [Definition]

For one retained relationship pattern, let $y_0=1$, the first-occurrence id of
the first observed symbol. At generated position $t>0$, each retained channel
whose left id occurs at its recorded distance contributes its exact frequency:

$$
V_t(b)=\sum_{(a,b,d)\text{ retained}\atop d\leq t, y_{t-d}=a}F(a,b,d).
$$

If some vote is nonzero, $y_t$ is the smallest symbol id attaining maximum
$V_t$. If no retained channel applies, $y_t$ is the smallest id attaining
maximum dictionary count $C$. The generated native row is the complete state
named by $y_t$. Replay is explicitly finite and capped at one million rows.

For rank descent, rank one first generates one fixed reference
$R_0,\ldots,R_{N-1}$; by default $N$ is the number of supplied observations.
Every lower candidate is constructed from that same reference. It is successful
exactly when its independently generated rows satisfy

$$
\widehat R_t=R_t\qquad(0\leq t<N).
$$

The report records the first mismatch, longest exact prefix, and total matching
rows. In exact mode, only a candidate satisfying all $N$ equalities may replace
the selected pattern.
Adaptive search first tests $1/2$: success replaces the upper tested bound and
failure replaces the lower tested bound; their midpoint is tested next. Search
ends when the bounds retain adjacent channel counts. Linear search subtracts
one exact positive step and tests every positive resulting rank.
`rank_descent(E)` selects adaptive exact search. `rank_descent(E,r)` tests one
static rank $r$ with exact agreement. `rank_descent(E,r,a)` accepts that static
candidate exactly when $M/N\geq a$, where $M$ is the number of matching rows
across the complete fixed reference. The implementation compares exact integers
by cross multiplication. When $a<1$, this is a declared lossy policy, not an
equality proof. The CLI also retains a linear exact schedule for experiments.
Every form is staged and replaced by the selected finite native pattern before
bytecode generation.

`apply(rank_descent(E),p)` requires $p\geq1$. It runs the selected pattern's
same deterministic replay through position $p$ and returns the complete native
state at one-based position $p$. The form is staged and replaced by that state
before bytecode generation.

### T-LANG-RANK-1 -- selected rank replay is exact [Proved]

**Statement.** Relationship replay is deterministic. Rank descent always
returns a pattern that reproduces its complete finite rank-one reference when
minimum agreement is one. Its reported mismatch position is the first unequal
row. Adaptive branching moves
lower after complete success and higher after failure; linear search tests each
positive scheduled step. Pattern application returns the unique replay row at
its requested positive position.

**Dependencies.** D-LANG-RANK-1, T-LANG-UNTRACE-1, finite map counting,
canonical native-state equality.

**Proof.** At each replay position, the retained channel set and finite history
determine one finite exact vote map. Total ordering by vote and then inverse id
selects one unique winner. When the map is empty, the finite counted dictionary
selects one unique fallback by the same tie rule. Induction on the requested
finite row count therefore gives one unique replay.

Applying at position $p$ executes precisely this induction for $p$ rows and
returns its final row, so the result is unique and one-based.

Comparison visits reference and generated rows in order and stops at the first
unequal canonical state, so the recorded prefix and one-based mismatch are
exact. Rank one initializes the selected pattern because it generated the
reference. A lower candidate replaces it only after all $N$ comparisons are
equal. Induction over the finite tested schedule therefore preserves the
invariant that the selected pattern reproduces every reference row. Adaptive
bound updates and linear subtraction are their stated exact rational
operations. $\square$

**Boundary.** This theorem applies only when minimum agreement is one. It proves
finite replay equality, not correctness beyond the checked reference. Adaptive output is the lowest fully successful candidate it
tested, not a globally minimal rank: success under channel removal has not been
proved monotone. Relationship replay is a deterministic host interpretation of
the emitted native frequencies, not a fifth core algebra operation.

### T-LANG-RANK-POLICY-1 -- static agreement policy is exact [Proved]

**Statement.** For total channel count $C$ and exact target rank $r$, static
selection retains $\lceil Cr\rceil$ strongest channels. For $N>0$ compared
rows, a candidate is accepted exactly when its matching-row count $M$ satisfies
$M/N\geq a$ for the declared exact agreement $a$.

**Proof.** The retained count is integer ceiling division of the exact rational
product. Channel order is deterministic. Acceptance compares
$M\,\mathrm{den}(a)$ with $N\,\mathrm{num}(a)$, so no floating-point
rounding enters either decision. For $a=1$, acceptance is equivalent to
$M=N$, recovering T-LANG-RANK-1. For $a<1$, the report labels the result lossy
and retains both $M$ and $N$. $\square$

**Boundary.** A lossy accepted candidate has only the measured agreement stated
in its finite report. It is not semantically equivalent to rank one and says
nothing about unseen continuation.

### D-LANG-DATA-1 -- complete ordered data invocation [Definition]

For a source function $f$ and a finite decoded data file
$(x_1,\ldots,x_n)$, complete-data invocation is the ordinary call
$f(x_1,\ldots,x_n)$. Every $x_j$ is the complete exact native state produced by
the existing JSON/NSBATCH decoder. The host introduces no chunk, reset,
projection, feature, or model update.

### T-LANG-DATA-1 -- complete-data invocation preserves every argument [Proved]

**Statement.** Complete-data execution passes exactly the decoded root items to
the selected source function in file order and returns D-LANG-2's ordinary call
denotation.

**Dependencies.** D-LANG-DATA-1, D-LANG-2, D-LANG-BATCH-1's exact decoder.

**Proof.** The decoder produces one finite ordered vector of exact states. The
host clones that vector in iteration order and supplies it to the existing
arity-checked exact-function boundary. Fixed parameters consume the unique
prefix and a trailing pack receives the unique remaining suffix. That boundary
then evaluates the ordinary function body under D-LANG-2. No other transform is
applied, proving the statement. $\square$

**Boundary.** This proves transport and invocation equality. It does not prove
that a source-defined model predicts unseen data, compresses its input, or runs
faster than a Rust implementation.

### D-LANG-BATCH-1 -- pointwise stepped execution [Definition]

For a unary exact function $f$, ordered inputs $x_0,\ldots,x_{m-1}$, and a
finite nonnegative step count $k$, define

$$
B_f^k(x_0,\ldots,x_{m-1})=
\left(f^k(x_0),\ldots,f^k(x_{m-1})\right).
$$

The $k$ applications inside one $f^k(x_j)$ are sequential. Distinct $j$
positions have no data dependency and may be evaluated concurrently. Results
retain input order. Batch selection, data loading, step count, and execution
backend are host parameters, not Native Space expression forms.

Let $A$ be a nonempty rectangular host array of rank $r\leq64$. For its leaf at
the 1-based position $p=(p_1,\ldots,p_r)$, define

$$
I_p(A_p)=\mathrm{INDEX}_1^{p_1}\circ\cdots\circ
\mathrm{INDEX}_r^{p_r}(A_p).
$$

The array lowers to the finite native ADD state

$$
L(A)=\boxplus_p I_p(A_p).
$$

For example, `[x,y]` lowers to
`add(index(1,x),index(1,index(1,y)))`. A rank-2 leaf at position $(2,1)$
uses INDEX direction 1 at depth 2 and direction 2 at depth 1. Therefore it is
distinct from position $(1,2)$ even though INDEX composition is commutative.
Empty, ragged, and mixed-rank arrays have no lowering. Thus an accepted array
is one ordinary native state, not an array primitive. Retaining its shape for
readable output does not alter that state's denotation.

JSON is the readable host encoding. The version-1 `NSBATCH` binary encoding
stores the same optional shape and sparse native terms with explicit lengths.
Packing followed by binary decoding is defined to reconstruct that exact pair.
Unsupported versions, malformed or noncanonical terms, shape mismatches, and
trailing bytes are outside the format and are rejected.

The exact GPU domain is the subset whose inputs, constants, and every
intermediate result are real signed-32-bit integers and whose expanded function
contains only ADD, MULTIPLY, and even ORIENT turns. GPU addition detects a sign
change inconsistent with its addend signs. GPU multiplication compares unsigned
magnitudes with the signed result limit before multiplying. GPU negation rejects
the least signed integer. An overflow flag aborts the complete result. Other
native values and operations are outside this GPU target and are rejected.

### T-LANG-BATCH-1 -- CPU and accepted GPU batches are pointwise exact [Proved]

**Statement.** Array lowering $L$ is injective in leaf position, JSON-to-binary
packing preserves every accepted input state and host shape, and CPU batch
execution returns $B_f^k$ under D-LANG-2. Whenever the GPU backend accepts the
same supported inputs without an overflow flag, it returns the same ordered
integer scalars as CPU batch execution.

**Dependencies.** D-LANG-BATCH-1, T-LANG-INTERP-1, exact two's-complement
signed-32-bit arithmetic within range.

**Proof.** The native multi-index of $I_p(A_p)$ is the finite map $a\mapsto p_a$.
If positions $p$ and $q$ differ, then $p_a\neq q_a$ on at least one axis $a$,
so their multi-indices differ. Hence no two array positions collide. Shape and
the nonzero sparse terms reconstruct all leaves, with omitted terms read as
exact zero. The binary encoder writes those same fields; its decoder reads each
field without a coordinate conversion, so accepted packing round-trips the
same input pair.

Each CPU worker invokes the same exact unary evaluator $k$ times on one
immutable input and stores the result at that input's position. Worker
partitioning neither shares a state nor changes position order, so every output
is $f^k(x_j)$.

For GPU ADD, the unsigned-bit addition is the two's-complement encoding of
integer addition whenever the sign test reports no overflow. For MULTIPLY, the
magnitude bound is exactly the positive limit $2^{31}-1$ or negative limit
$2^{31}$; absence of its overflow flag therefore makes the encoded product the
integer product. Even ORIENT turns are identity or exact negation, with
$-2^{31}$ rejected before negation. Structural induction on the expanded step
expression gives the same integer result as D-LANG-2 whenever no flag is set.
Induction on the sequential step counter gives $f^k(x_j)$. One invocation owns
one $j$, and reconstruction reads invocation positions in order. Hence every
accepted GPU result equals the CPU result and both equal $B_f^k$. $\square$

**Boundary.** The retained shape is host metadata, not native state. This proves
the specified lowering and encoding equality, not that the Rust or shader
implementation is formally verified and not that binary parsing or a GPU is
faster. Unsupported values, malformed data, unavailable adapters, more than
1,000,000 GPU steps, and overflow are diagnostics; there is no approximate
conversion or backend fallback.

### D-LANG-2 -- expression denotation

For an environment

$$
\rho:\text{Name}\rightharpoonup\mathcal N_{\mathcal A},
$$

define the denotation of an expression recursively, with every orientation
literal $r\in\mathbb{Z}_4$:

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
  &= \mathrm{INDEX}_k^d(\llbracket E\rrbracket_\rho),\\
\llbracket\texttt{camera}(a,b,E)\rrbracket_\rho
  &= C_{a\to b}(\llbracket E\rrbracket_\rho),\\
\llbracket\texttt{trace}(f)\rrbracket_\rho
  &= \mathrm{OperationStrand}(f),\\
\llbracket\texttt{untrace}(E,r)\rrbracket_\rho
  &= \mathrm{DiscoverPattern}(\llbracket E\rrbracket_\rho,r),
  \qquad r\in\mathbb{Q}\cap[0,1].
\end{aligned}
$$

String literals use D-LANG-UTF8-1. A call evaluates its arguments, binds the
resulting values to the called source function's parameters, and evaluates its
body. Function names have no denotational case of their own. Finite fold uses
D-LANG-FOLD-1 and therefore denotes only its nested ordinary calls.
`DiscoverPattern` is partial only for incompatible input, invalid rank,
exhausted finite budgets, or unrepresentable generated coordinates. Lack of an
exact recurrence selects relationship mode rather than producing a diagnostic.

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

Malformed operands, noncanonical orientation turns, invalid slots, stack
underflow, invalid arity or direction, and an invalid final stack are outside
valid compilation and produce typed VM diagnostics rather than a native value.
Bytecode carries the final goal as
data. The VM returns the denotation without interpreting that goal; the checker
compares the direct and compiled results, then applies the goal.

### D-LANG-4 -- compilation

`trace(f)` is first replaced by its D-LANG-TRACE-1 expression, so reflection
observes the original source graph. Source calls and finite folds are then
recursively erased by parameter substitution; variadic packs are substituted
in source order, and concat is rewritten by D-LANG-CONCAT-1. `length(S)`, each
closed `camera(a,b,E)`, each `untrace(E,rank)`, and each
`rank_descent(E,rank,agreement)` and each rank-pattern `apply` are evaluated in the exact environment of their earlier
bindings and replaced by their finite core-state expressions. Consequently the
VM has no trace, length, untrace, rank-descent, pattern-application, pack, concat, fold, or camera
opcode.
Only then may D-LANG-5 optimizer rewrites alter the executable expression.
Expression compilation is compositional:

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

**Statement.** Direct evaluation of every finite well-formed program $P$
terminates. If no defined partial construction emits a diagnostic, it returns
$\llbracket P\rrbracket$.

**Dependencies.** D-LANG-1, D-LANG-2, T-LANG-TRACE-1,
T-LANG-UNTRACE-1, T-LANG-RANK-1, D-NS-3 through D-NS-10.

**Proof.** Proceed by structural induction on expressions. Each base node is
returned using exactly its D-LANG-2 constructor. A reference is present in the
environment by D-LANG-1. In the induction cases, recursive calls return the
child denotations; the interpreter then applies the same native ADD,
MULTIPLY, ORIENT, or INDEX operation as D-LANG-2. The AST is finite, so the
recursion terminates. For `trace(f)`, T-LANG-TRACE-1 first constructs one
finite core expression and the same induction applies to that expression.
Variadic substitution and concat use the finite rewrites of T-LANG-CONCAT-1.
Finite fold performs exactly the left-associated call sequence proved by
T-LANG-FOLD-1. Camera evaluates its child and applies exactly the finite
coefficient selection and index remapping of T-LANG-CAMERA-1. For
`untrace(E,rank)`, T-LANG-UNTRACE-1 either returns a typed finite-budget
diagnostic or constructs the exact deterministic strand or relationship-state
expression assigned by D-LANG-2; the same induction applies to that expression.
For `rank_descent(E,rank,agreement)`, T-LANG-RANK-1 or
T-LANG-RANK-POLICY-1 either returns a located finite-budget
diagnostic or one selected finite native pattern, after which the same induction
applies.
Pattern application uses that theorem's unique finite replay row and likewise
reduces to one native state.

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
instruction. Calls, packs, concat, fold, trace, length, untrace, rank descent,
pattern application, and camera
nodes are absent here because D-LANG-4 lowers them to finite core expressions
before this induction. No expression instruction stores a slot. $\square$

### T-LANG-COMP-1 -- compiled execution equals direct evaluation [Proved]

**Statement.** For every well-formed program $P$ that compiles, compilation
followed by VM execution returns the same canonical native state as direct
evaluation:

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
uses L-NS-8. Combining nested canonical ORIENT nodes, reducing only their
derived sum modulo four, and removing the resulting identity use the
state-level cycle and composition law L-SEP-5.
INDEX only rebuilds its node around an equivalent child while preserving its
native multiplicity. No rewrite changes bindings or references, so well-formedness is
preserved. $\square$

## Exact vector output

### D-LANG-VECTOR-1 -- exact quadratic vector output [Definition]

For native zero define

$$
V(0)=(0,0,0).
$$

For one unindexed oriented scalar with exact rational coordinates $(x,y)$,
define the requested vector camera by

$$
V(x,y)=(x^2-y^2,2xy,x^2+y^2).
$$

The output is undefined for indexed or multi-term states. The runtime must
reject those states rather than erase their INDEX locations or combine their
distinct coefficients.

### T-LANG-VECTOR-1 -- vector output is the exact cone camera [Proved]

**Statement.** Every defined `as vector` output has exact rational coordinates
and satisfies

$$
V_1^2+V_2^2=V_3^2.
$$

Zero maps to the cone tip. Away from zero, two oriented scalars have the same
vector exactly when they are opposites.

**Dependencies.** D-LANG-VECTOR-1, T-CONE-1.

**Proof.** D-LANG-VECTOR-1 is exactly the map $Q$ of C-CONE-1, restricted to
the scalar states accepted by the output camera. T-CONE-1 proves the displayed
identity and proves that every nonzero fiber is the pair $\{z,-z\}$. The
explicit zero rule gives the cone tip. Because all input coordinates are exact
rationals and the formula uses only exact ADD and MULTIPLY, all three returned
coordinates are exact. $\square$

**Boundary.** This camera is a deliberate quotient, not the default state
representation. `as pattern` retains the complete flat stack, including INDEX
directions and oriented signs.

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
- They do not cover future approximate values, cameras other than the exact
  indexed camera and quadratic scalar vector camera defined here, conventional prime-value lookup, automatic
  factorization, general loops, unbounded recursive execution, effects, or
  compiler targets other than schema-1 bytecode.
- Source-function self-reference is covered by two finite observations.
  D-FLANG-2 and T-FLANG-SELF-1 retain an active repeated derivation call as a
  pattern-reference edge. D-LANG-TRACE-1 and T-LANG-TRACE-1 return the parsed
  reachable exact-function graph as native coordinates. Neither result
  repeatedly unfolds that edge as an unbounded computation or treats it as a
  completed recursive execution.
