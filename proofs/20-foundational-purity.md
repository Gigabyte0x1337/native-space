<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Exact Rational Native Kernel Proof

## Scope

This file proves the finite exact Native Space carrier relative only to
A-IND-1. It does not yet construct real completion, limits, analytic flow,
zeta continuation, or the Riemann Hypothesis.

### T-QARITH-1 -- constructed rationals form an ordered field [Proved relative]

**Construction.** Primitive recursion gives natural ADD and MULTIPLY. Structural
induction proves their identity, associativity, commutativity, distributivity,
and cancellation laws. Strong induction, obtained from structural induction on
the bound, proves Euclidean division: for positive $d$, every $n$ has
unique $q,r$ with $n=qd+r$ and $r<d$. Repeated remainder strictly
decreases, so gcd normalization terminates.

Canonical sign-and-magnitude integers inherit ADD and MULTIPLY by finite sign
cases. The natural laws prove the commutative-ring laws and trichotomous order.

For canonical rationals $a/b$ and $c/d$, define

$$
\frac ab+\frac cd:=\mathrm{reduce}\left(\frac{ad+bc}{bd}\right),
\qquad
\frac ab\frac cd:=\mathrm{reduce}\left(\frac{ac}{bd}\right).
$$

Reduction divides numerator and denominator by their positive gcd and fixes
the denominator sign. Euclidean gcd properties prove that two reduced pairs
represent the same cross-product value only when their canonical pairs are
equal. Therefore reduction preserves value and is unique. The integer ring
laws then prove the rational field laws by cross multiplication. Every nonzero
$a/b$ has inverse $\mathrm{sign}(a)b/|a|$. Define order by the sign
of $ad-bc$ for positive $b,d$; integer trichotomy and multiplication by a
positive denominator prove ordered-field compatibility. No real or complex
number premise is used. $\square$

### T-QOS-1 -- exact rational oriented scalars form the native field [Proved relative]

**Claim.** D-QOS-1 satisfies T-OS-1, the four-cycle, the internal number line
and $\mathbf J$-axis theorem, and perspective invariance.

**Proof.** All polynomial ADD/MULTIPLY identities in `01-core-algebra.md` use
only commutative-field laws, now supplied by T-QARITH-1. For nonzero
$(x,y)\in\mathbb{Q}_P^2$, ordered-field positivity gives
$x^2+y^2>0$. Hence

$$
(x,y)^{-1}
\mathrel{=}
\left(\frac{x}{x^2+y^2},\frac{-y}{x^2+y^2}\right)
$$

is a constructed rational pair and direct MULTIPLY gives $\mathbf1$.
The computations $\mathbf J^2=\mathbf{-1}$ and $\mathbf J^4=\mathbf1$
are finite rational calculations. Distributivity by a common unit proves
T-PERSPECTIVE-ZERO-1 exactly as before. $\square$

### T-QCORE-1 -- the finite four-operation state carrier is pure [Proved relative]

**Claim.** D-QIDX-1 and D-QNS-1 are closed under ADD, MULTIPLY, ORIENT, and
INDEX and satisfy the finite core laws in T-NS-1 relative only to A-IND-1.

**Proof.** Every input sequence is finite by construction. Merge of sorted
index sequences terminates by structural recursion and preserves positive
depths. ADD is a finite sorted merge, using T-QOS-1 coefficient ADD and deleting
canonical zero. MULTIPLY visits a finite Cartesian product of supports, merges
each index pair, multiplies coefficients in T-QOS-1, and then uses finite ADD
to collect equal locations. ORIENT maps a finite support by one T-QOS-1 unit;
INDEX maps it by one finite index merge. Thus all four results are canonical
finite states.

The proofs of associativity, identities, inverses for ADD, distributivity,
commutativity, ORIENT action, and INDEX/MULTIPLY compatibility in
`01-core-algebra.md` are finite coefficientwise arguments. Replacing A-RF-1 by
T-QOS-1 and A-N-1 by D-PNAT-1 discharges every premise used in those arguments.
No camera or analytic law enters. $\square$

### E-QIMPL-1 -- implementation correspondence [Tested, not formally proved]

exact rational values and canonical finite `NativeState` values implement
the D-PRAT-1 and D-QNS-1 normal forms. Differential interpreter/VM tests,
serialization tests, random optimizer checks, and the tagged classical
identity example test implementation behavior.

This is strong executable evidence, not a foundational proof that Rust,
its arbitrary-precision integer implementation, or the host machine realizes
A-IND-1 correctly. A small proof-assistant kernel or verified extractor is the
remaining implementation-level trust reduction.
