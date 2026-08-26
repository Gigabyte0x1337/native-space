<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Shifted Zero-Center Classification

## Typed statement

Let $s=\sigma+it$, let $0_Z$ be zeta's additive-output zero, and for a
fixed real center $c$ define the RE residual

$$
R_c(s):=\sigma-c.
$$

Its zero $0_{R,c}$ is owned by the centered RE camera. It is not zeta's
zero. Define the center-dependent implication

$$
H_c:
\qquad
\zeta(s)=0_Z
\Longrightarrow
R_c(s)=0_{R,c}
$$

for every nontrivial zeta zero in the critical strip.

## T-REFLECTION-CENTER-UNIQUE-1 -- reflection forces one half [Proved]

The multiplicative reflection is

$$
\rho(\sigma)=1-\sigma,
$$

where $1$ is the multiplicative identity. Require reflection to reverse the
centered RE residual:

$$
R_c(\rho(\sigma))=-R_c(\sigma).
$$

Expanding both sides gives

$$
1-\sigma-c=-\sigma+c.
$$

Canceling $-\sigma$ gives $1-c=c$, hence

$$
\boxed{c=\frac{1}{2}.}
$$

Conversely, at $c=\tfrac{1}{2}$,

$$
R_{1/2}(1-\sigma)
=1-\sigma-\frac{1}{2}
=-\left(\sigma-\frac{1}{2}\right)
=-R_{1/2}(\sigma).
$$

Thus one half is the unique fixed center of multiplicative reflection. The
closed Native Space witness in `../examples/reflection-center.ns` checks the
fixed point and one exact reflected pair using only ADD, MULTIPLY-generated
ORIENT, and INDEX.

## T-REVERSE-HALF-NO-1 -- RE zero does not imply zeta zero [Proved negative result]

At $s=\tfrac{1}{2}$,

$$
R_{1/2}\!\left(\frac{1}{2}\right)=0_{R,1/2}.
$$

However, the paired zeta camera gives

$$
\zeta\left(\frac{1}{2}\right)
=
\frac{
\displaystyle\sum_{m\ge1}
\left(
\frac1{\sqrt{2m-1}}-\frac1{\sqrt{2m}}
\right)
}{1-\sqrt{2}}.
$$

Every numerator term is positive and the denominator is nonzero. Therefore

$$
\zeta\left(\frac{1}{2}\right)\ne0_Z.
$$

Hence

$$
R_{1/2}(s)=0_{R,1/2}
\centernot\Longrightarrow
\zeta(s)=0_Z.
$$

The typed zero sets are not equal.

## T-OFFCENTER-RH-NO-1 -- every other fixed center is false [Reproduced consequence]

Classical zeta theory proves that infinitely many nontrivial zeta zeros lie on
the critical line. Let $s_0$ be any such zero, so

$$
\zeta(s_0)=0_Z,
\qquad
\mathrm{Re}(s_0)=\frac{1}{2}.
$$

For a fixed $c\ne\tfrac{1}{2}$,

$$
R_c(s_0)=\frac{1}{2}-c\ne0_{R,c}.
$$

Therefore $H_c$ is false for every fixed $c\ne\tfrac{1}{2}$.

## The half-centered forward case

At the unique reflection-compatible center, the statement is

$$
H_{1/2}:
\qquad
\zeta(s)=0_Z
\Longrightarrow
R_{1/2}(s)=0_{R,1/2}.
$$

Because $R_{1/2}(s)=0$ exactly when
$\mathrm{Re}(s)=\tfrac{1}{2}$, this is precisely classical RH. Shifting
the zero center identifies the only possible fixed center; it does not prove
the forward implication at that center.

## Moving centers are tautological

If the center is allowed to depend on the input and one defines

$$
c(s):=\mathrm{Re}(s),
$$

then $R_{c(s)}(s)=0$ for every input, whether or not zeta is zero. This makes
the implication true by definition but removes all RH content. A valid camera
must therefore declare one fixed center independently of the point being
tested.

## Complete classification

| Center and direction | Status | Reason |
|---|---|---|
| Fixed $c\ne\tfrac{1}{2}$, zeta zero $\Rightarrow R_c=0$ | False | Known critical-line zeros give $R_c\ne0$ |
| Fixed $c=\tfrac{1}{2}$, zeta zero $\Rightarrow R_c=0$ | Open | Exactly classical RH |
| Fixed $c=\tfrac{1}{2}$, $R_c=0\Rightarrow$ zeta zero | False | $s=\tfrac{1}{2}$ is a counterexample |
| Moving $c(s)=\mathrm{Re}(s)$ | Trivially true | The conclusion is inserted into the camera |

This proves every shifted-center case except the one case that is exactly RH.
