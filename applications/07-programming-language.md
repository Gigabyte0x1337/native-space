<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# A Programming Language in Native Space

**Status:** the exact-state parser, evaluator, compiler, VM, source strings,
bindings, and finite checks work now. Typed data cameras, control flow, effects,
decoding, and self-hosting remain planned.

The purpose is not to add many primitives. It is to compile familiar language
constructs into the existing native state and operations.

## What translates today

| Familiar construct | Native Space form | Current status |
|---|---|---|
| Source text or identifier `name` | `"name"` | Exact UTF-8 byte-and-position INDEX lowering |
| Exact number `42` | `scalar(42, 0)` | Exact |
| Oriented number | `scalar(real, imag)` | Exact |
| False / true | typed camera over `zero` / `one` | Encoding choice; Boolean proof mode already separate |
| Ordered item at position $j$ | $\mathrm{INDEX}_{j}(\text{item})$ | Exact construction; reversible sequence camera still to prove |
| Record field $j$ | $\mathrm{INDEX}_{j}(\text{value})$ inside ADD | Exact construction; field-tag namespace still to define |
| AST node | ADD of a string tag and indexed child states | Candidate camera |
| Unary/binary operation | ORIENT, INDEX, ADD, or MULTIPLY over child states | Exact when it is one of the four native operations |
| Equality assertion | native difference followed by `= 0` | Exact for a closed finite instance |
| Source-function expansion | `derive` / `apply` over ordinary functions | Implemented generically |
| Conditional, loop, closure, memory, I/O | no current translation | Requires explicit semantics |

The [programming-language-data.ns](../examples/applications/programming-language-data.ns)
witness constructs a small `let answer = 42`-like node from two strings and
one scalar. It proves that literal/escaped UTF-8 and a reordered ADD of the same
tagged fields produce one exact native state.

## T-LANG-UTF8-1 -- strings are exactly recoverable [Proved]

If string $s$ has UTF-8 bytes $b_0,\ldots,b_{m-1}$, its literal lowers to

$$
\mathrm{UTF8}(s)=
\bigoplus_{j=0}^{m-1}
\mathrm{INDEX}_{257+j}
\left(\mathrm{INDEX}_{b_j+1}(\mathsf1)\right).
$$

**Proof.** Byte directions $1,\ldots,256$ and position directions $257+j$
are disjoint. Every position occurs once, so no two terms merge. Reading terms
by increasing position and subtracting one from each byte direction recovers
the unique UTF-8 byte sequence, which decodes to $s$. Therefore
`decode_utf8(UTF8(s)) = s`, and the map is injective. $\square$

## T-PL-CORE-1 -- finite AST field assembly is deterministic [Proved]

For finite field states $F_1,\ldots,F_n$ and any permutation $\sigma$,

$$
\mathrm{ADD}(F_1,\ldots,F_n)
=\mathrm{ADD}(F_{\sigma(1)},\ldots,F_{\sigma(n)}).
$$

**Proof.** L-NS-2 gives associativity and commutativity of ADD. Any finite
permutation is a finite sequence of swaps, so both folds are equal. Their
two-turn-ORIENT residual is native zero. $\square$

**Boundary:** deterministic assembly is not reversible AST decoding. Payloads
can already use arbitrary INDEX directions, so a production language needs a
typed, collision-free tag camera and a proof of its inverse.

## H-PL-1 / E-PL-1 [Hypothesis / Planned build]

Build the smallest useful language in evidence-gated stages:

1. **Done:** exact strings, exact scalar/state values, comments, bindings,
   `output`, zero equality, parser, AST, bytecode, and VM.
2. Define typed cameras for booleans, sequences, records, symbols, and AST
   variants; prove every Encode/Decode round trip.
3. Define lexical scope, environments, calls, conditionals, and failure as
   native data and transformations.
4. Compile a tiny expression language to existing bytecode and compare direct
   evaluation with compiled execution.
5. Represent its parser/compiler as native states and actions.
6. Claim self-hosting only when the native representation compiles its own
   source and the produced binary passes the same conformance suite.

The first go/no-go test is a calculator plus immutable bindings, strings,
booleans, records, and functions. It must reject malformed programs with exact
source locations and pass differential tests against a simple specification
interpreter.

## What cannot be claimed yet

- Strings alone do not provide concatenation, slicing, Unicode grapheme rules,
  parsing, or I/O.
- The current commutative INDEX algebra does not by itself encode execution
  order, mutable memory, control flow, or lexical scope.
- T-ACT-1 proves limited self-representation of additive and multiplicative
  actions, not arbitrary programs or cameras.
- A compiler written in Rust is not a self-hosted Native Space compiler.

## Reference specifications

- [Latest Unicode Standard](https://www.unicode.org/versions/latest/)
- [Rust language reference](https://doc.rust-lang.org/reference/)
- [WebAssembly core specification](https://webassembly.github.io/spec/core/)
