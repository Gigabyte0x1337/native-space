<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# RH Projection Factorization Audit

## The one-dimensional proposal

The attempted full proof treated zeta and centered RE as two invertibly
rotated projections of one value $\Gamma(s)$:

$$
P_R\Gamma(s)=R_{-90}P_Z\Gamma(s).
$$

T-NATIVE-PERSPECTIVE-ZERO-1 correctly proves that this relation makes the two
camera zeros equivalent. The question is whether the two outputs can globally
decode as

$$
P_Z\Gamma(s)=0\Longleftrightarrow\zeta(s)=0,
\qquad
P_R\Gamma(s)=0
\Longleftrightarrow
\mathrm{Re}(s)-\frac{1}{2}=0.
$$

## T-ZETA-RE-GLOBAL-FACTOR-NO-1 -- the proposed global interpretation is false [Proved negative result]

**Claim.** No source $\Gamma(s)$ on the whole open critical strip can satisfy
all three displayed equivalences above.

**Proof.** Set $s=\tfrac{1}{2}$. The centered RE coordinate is zero, so the
proposed RE decoder gives

$$
P_R\Gamma\!\left(\frac{1}{2}\right)=0.
$$

The inverse quarter-turn then forces

$$
P_Z\Gamma\!\left(\frac{1}{2}\right)=0,
$$

and the proposed zeta decoder would give $\zeta(\tfrac{1}{2})=0$.

But C-ZETA-STRIP-1 expresses this value using the paired pattern. Its numerator
is

$$
\mathcal E_N\!\left(\frac{1}{2}\right)
=\sum_{m\ge1}
\left(
\frac1{\sqrt{2m-1}}-\frac1{\sqrt{2m}}
\right).
$$

Every summand is strictly positive, so the convergent numerator is strictly
positive. The denominator is

$$
1-2^{1-1/2}=1-\sqrt{2}\ne0.
$$

T-ZETA-STRIP-1 therefore gives

$$
\zeta\left(\frac{1}{2}\right)
=\frac{\mathcal E_N(1/2)}{1-\sqrt{2}}\ne0,
$$

contradicting the proposed decoders. Thus the global one-dimensional
interpretation is false. $\square$

## Why the idea remains useful

The failure identifies the missing structure. The source was flattened to one
value before projection, so it did not retain the recursive multiplicative
prime pattern. A faithful candidate must instead preserve a structured object
of the form

$$
\Gamma_{\mathbb{P}}(s)
\mathrel{=}
(\mathrm{INDEX},\mathrm{MULTIPLY\ recursion},
  \mathrm{ORIENT},\mathrm{analytic\ gain}).
$$

Zeta and RE should then be separate projections

$$
C_Z(\Gamma_{\mathbb{P}}(s)),
\qquad
C_R(\Gamma_{\mathbb{P}}(s)),
$$

not invertible rotations of one flattened scalar. The desired statement is the
restricted zero-fiber inclusion

$$
\ker(C_Z|_{\Gamma_{\mathbb{P}}})
\subseteq
\ker(C_R|_{\Gamma_{\mathbb{P}}}).
$$

This is the correct operations-first form of the RH target: a zero of the zeta
projection must force zero of the RE projection because of the retained
recursive prime structure.

## Exact status

- **Proved true:** two invertibly rotated cameras of one value have the same
  zero.
- **Proved false:** those two camera zeros cannot globally decode as zeta zero
  and centered-RE zero on the whole critical strip.
- **Proved structural fact:** the native flat stack retains INDEX and ORIENT
  before scalar aggregation; the aggregate camera is generally noninjective.
- **Conjectured explanation:** the complete recursive prime pattern cannot be
  reconstructed from the flattened classical camera without adding equivalent
  native structure back.
- **Open RH step:** prove the restricted zero-fiber inclusion for the actual
  recursively generated prime-pattern image.

The current source derivations expose the boundary. `zeta_classical_pattern`
constructs a paired integer-character aggregate and denominator;
`re_classical_pattern` constructs a centered input and per-prime reflected
mismatch. Both expand successfully, but they are not derived from one retained
recursive source function.
