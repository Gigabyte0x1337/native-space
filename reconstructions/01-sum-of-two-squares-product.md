<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# R-SOS-1: Sum of Two Squares under Multiplication

**Status:** Reproduced  
**Known result:** Brahmagupta--Fibonacci identity, two-square product form  
**Dependencies:** A-RF-1, D-OS-3, D-OS-5, L-OS-9, T-CX-2  
**Native invariant:** multiplicativity of squared coefficient size

## Conventional statement

For all $a,b,c,d\in\mathbb{R}$,

$$
(a^2+b^2)(c^2+d^2)
=(ac-bd)^2+(ad+bc)^2.
$$

Over the integers this shows that a product of two numbers representable as a
sum of two squares is again representable as a sum of two squares. The
polynomial identity itself is valid over the declared real-field substrate.

## Native construction

Let

$$
z=(a,b),\qquad w=(c,d)
$$

be oriented scalars. Their native squared sizes are

$$
\nu(z)=a^2+b^2,
\qquad
\nu(w)=c^2+d^2.
$$

Native MULTIPLY gives

$$
z\boxtimes w=(ac-bd,ad+bc).
$$

L-OS-9 was proved internally from the real-field substrate and states

$$
\nu(z\boxtimes w)=\nu(z)\nu(w).
$$

This is the entire reconstruction in native terms: coefficient MULTIPLY
preserves squared size multiplicatively.

## Back-translation

Expanding the left side of the invariant using D-OS-3 and D-OS-5 gives

$$
\nu(z\boxtimes w)
=(ac-bd)^2+(ad+bc)^2.
$$

Expanding the right side gives

$$
\nu(z)\nu(w)=(a^2+b^2)(c^2+d^2).
$$

Equating them reproduces the conventional identity. Under the complex camera
T-CX-2, the same statement is the familiar multiplicativity of squared
modulus, but that camera is a verification and interpretation here, not a
dependency of L-OS-9.

## Edge cases

- If either oriented scalar is zero, both sides vanish without an exception.
- Negative and non-integral real coordinates are allowed by the polynomial
  identity.
- The integer closure consequence uses the additional fact that products and
  sums of integers remain integers; it does not assert that every integer is a
  sum of two squares.

## Comparison

**Classification:** invariant-revealing and equivalent in total algebraic
content.

Once L-OS-9 exists, the native derivation is one invariant application and can
be reused without re-expanding four squares. However, the proof of L-OS-9
itself performs the same real-field cancellation that a direct proof of the
identity performs. Therefore this is not evidence that Native Space shortens
the one-off foundational proof. Its demonstrated value is organizational: the
expanded identity is recognized as one instance of multiplicative native
size.

## Executable cross-check

[`../language/runtime/tests/reconstructions.rs`](../language/runtime/tests/reconstructions.rs) checks
L-OS-9 on 200 deterministic exact-rational input pairs using the executable
coefficient model. This checks implementation agreement only; the deductive
argument above is the evidence for Reproduced status.

## What is not established

This reconstruction does not prove a new number-theory theorem, characterize
integers representable by two squares, or show a computational advantage.
