<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Pure Function and Zero-Proof Semantics

## Scope

This replaces the former typed analytic claim subsystem. Rust no longer assigns
types or proof rules to zeta, RH, convergence, axis, or geometry names.

### D-FLANG-1 -- source function [Definition]

An exact function is `let name = (parameters) => expression`. An operation function
has the same declaration followed by generic operation/call steps until the
next `let` or end-of-file. Neither form has a function keyword, end marker,
summary field, or mandatory semicolon.

### D-FLANG-2 -- expansion [Definition]

Expansion binds supplied arguments to parameters, replaces each call by its
source body, and continues along the finite source graph. If a call refers to
a function already active on the current path, expansion records that call as
a **pattern reference** and does not enter the target again. The reference
retains its arguments, source line, and closing function path. It is graph
structure, not a primitive operation.

### T-FLANG-SELF-1 -- recursive source is a finite pattern graph [Proved]

**Claim.** Derivation of a finite well-formed function library terminates in a
finite report even when functions refer directly or mutually to themselves.
Every repeated active reference appears exactly once at that traversal
position as a pattern reference.

**Proof.** A library contains finitely many functions and every body contains
finitely many instructions. Along one traversal path, an ordinary call enters
a function not yet active, so path depth can increase at most by the number of
functions. A call to an active function records one edge and does not increase
depth. Each finite body is visited in source order, including instructions
after a recorded edge. Therefore the traversal terminates and its events and
pattern references are finite. The recorded edge contains the call target and
the active path, so it preserves exactly the source self-reference at that
position. $\square$

### T-FLANG-PURE-1 -- expanded operations are pure [Proved]

Every primitive step in a successful trace is ADD, MULTIPLY, ORIENT, or
INDEX. This follows by structural induction: source operations are accepted
only from that set; calls emit no authoritative operation and are replaced by
bodies satisfying the induction hypothesis; pattern references emit no
operation. $\square$

### T-FLANG-NAME-1 -- names have no proof force [Proved]

Consistently renaming a function and all call sites selects the same bodies in
the same positions. The fully expanded operation sequence is therefore
unchanged. $\square$

### T-FLANG-LOC-1 -- locations survive expansion [Proved]

Each parsed instruction retains its source span. Expansion copies that span and
the active function path to the emitted event, so every primitive operation
and pattern reference points to its `.ns` line. $\square$

### T-ZERO-ONLY-1 -- mathematical state proofs have one goal [Proved]

The only exact mathematical proof statement is `left = right`. Parsing replaces
it by `ADD(left, ORIENT(2, right))` with a zero goal. The checker independently
evaluates and compiles that expression, rejects disagreement, and accepts only
canonical zero. `output` makes no proof claim. No specialized mathematical
claim variant exists. $\square$

## Proof status

Function expansion carries no proof status. The paper ledger and, where finite,
the executable zero equality remain authoritative.
