<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Pure Function Expansion and Variadic Application

### D-FAPP-1 -- concrete source application [Definition]

For a source function `F` and function arguments `G_1, ..., G_n`, expansion
executes `F`'s body against the operation recorder. Every core operation adds
one step. Every nested call adds a navigation event and recursively expands the
called source body. A call to a function already active on that path adds a
pattern reference and returns to the containing finite body without adding a
primitive operation.

The argument list is dynamic in length. Each definition declares an exact
arity or one final variadic parameter. Fixed-arity bodies reject invalid
applications; variadic bodies accept one or more supplied functions.

### T-FAPP-TRACE-1 -- variadic apply preserves operation order [Proved]

**Claim.** If each concrete `G_j` application emits finite trace `E_j`, then
`apply(G_1, ..., G_n)` emits, in order,

$$
(\mathrm{call}G_1,E_1,\ldots,
  \mathrm{call}G_n,E_n).
$$

Its used-operation set is the ordered subset of

$$
\{\mathrm{ADD},\mathrm{MULTIPLY},\mathrm{ORIENT},\mathrm{INDEX}\}
$$

that occurs in the concatenated trace.

**Proof.** The source variadic body visits supplied arguments once in argument
order. At position `j`, expansion appends the call event and recursively appends
`E_j` before position `j+1`. The report computes its used-operation set by
deterministic filtering of that same trace. $\square$

### D-OP-INLINE-1 -- authoritative primitive expansion [Definition]

Given the finite source-graph trace $E$, define

$$
\mathrm{PrimitiveSteps}(E)
$$

by deleting only function-expansion markers. Retain every core operation and
its explicit parameters in original order. Function names remain only in the
separate navigation trace.

### T-OP-INLINE-1 -- function erasure preserves primitive operations [Proved]

**Claim.** `PrimitiveSteps(E)` contains no function call and preserves exactly
the ordered parameterized core-operation subsequence of $E$.

**Proof.** Expansion writes a function marker immediately before entering its
body. A pattern reference is also navigation structure and contributes no core
operation. D-OP-INLINE-1 removes precisely these non-operation entries and no
other entries. Every visited operation therefore remains once and in source
traversal order. $\square$

**Boundary.** These are implementation theorems about source expansion, not
proofs of the mathematics represented by a function. Mathematical truth is
decided by an executable zero equality, a finite Boolean proof, or the paper
dependency ledger.
