<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Analytic Completion and the Zeta Camera

This Stage 7 extension supplies the analytic definitions that were deliberately
absent from the formal completion. The native object is one generative zeta
pattern. Two scalar cameras are derived from it: the direct absolute camera on
$\mathrm{Re}(s)>1$, and an odd/even paired camera on
$\mathrm{Re}(s)>0$.

The construction is operations-first:

$$
\text{formal native state}
\longrightarrow
\text{prime-log flow character}
\longrightarrow
\text{absolutely summable camera}
\longrightarrow
\text{scalar coordinate}.
$$

The Riemann zeta function appears as a coordinate camera of the formal state;
it is not introduced as a primitive native operation.

## Analytic substrate

### A-CNS-1 -- complete scalar-series substrate [Axiom]

The conventional complex numbers form a complete normed commutative field.
For countable families, the standard theory of absolute summability is
available: comparison, linearity, invariance under bijective reindexing,
convergence over increasing exhausting subsets, and Tonelli/Fubini and Cauchy
product rules when the corresponding family of norms is summable.

Through the proved field isomorphism $\kappa:\mathbb{O}\to\mathbb{C}$, these
facts may be transported to oriented scalars. This axiom supplies analytic
completion and rearrangement. It does not supply any zeta identity.

### A-LOG-1 -- positive logarithm and real/complex powers [Axiom]

The conventional positive-real logarithm is inverse to the positive-real
exponential; both are strictly increasing on their real domains. The logarithm
obeys

$$
\log(xy)=\log x+\log y
\qquad(x,y>0).
$$

For $x>0$ and $z\in\mathbb{C}$, define

$$
x^z:=\exp(z\log x).
$$

The standard modulus law

$$
|x^z|=x^{\mathrm{Re}z}
$$

and the usual real-exponent order laws are available. This axiom supplies the
coordinate vocabulary needed to identify the flow character with $n^{-s}$;
it does not supply convergence or an Euler product.

### A-HOL-1 -- finite complex-analysis substrate [Axiom]

Use the standard finite-dimensional complex-analysis consequences needed here:
the real-path mean-value estimate for a continuously differentiable
complex-valued function, compact-uniform convergence under a summable majorant,
holomorphicity of the resulting sum, and uniqueness of analytic continuation
on a connected domain.

This substrate does not supply the zeta continuation formula. The native pair
rule, convergence bound, half-plane identity, and nonzero denominator are
derived below and in `../proofs/12-analytic-zeta.md`.

## Summable completed cameras

### D-SUM-1 -- absolute camera domain [Definition]

Let $F\in\widehat{\mathcal N}_{\mathcal A}$ and let
$\chi:\mathbb{I}_{\mathcal A}\to\mathbb{O}$ be multiplicative. Define

$$
(F,\chi)\in\mathrm{Dom}_{\mathrm{abs}}
\quad\Longleftrightarrow\quad
\sum_{\alpha\in\mathbb{I}_{\mathcal A}}
\left|\kappa(F_\alpha\boxtimes\chi(\alpha))\right|<\infty.
$$

This condition is ordering-independent by A-CNS-1.

### C-AFLAT-1 -- default analytic weighted flat stack [Definition]

For $(F,\chi)\in\mathrm{Dom}_{\mathrm{abs}}$, write

$$
F_\alpha\boxtimes\chi(\alpha)=(x_\alpha,y_\alpha)
$$

and define the absolutely summable indexed stack

$$
\mathrm{Flat}_\chi(F)
:=
\big((\alpha,x_\alpha,y_\alpha)
  :F_\alpha\boxtimes\chi(\alpha)\neq\mathbf0\big).
$$

This is the analytic extension of the default C-FLAT-STACK-1 camera. It keeps
every weighted INDEX coefficient separate. Its symbolic serialization name is
`analytic-flat-stack-v1`; the finite runtime continues to use
`flat-stack-v1`.

Define the final aggregation projection by

$$
\mathrm{Agg}_{\mathrm{abs}}
\big(\mathrm{Flat}_\chi(F)\big)
:=
\mathop{\boxplus}_{\alpha}^{\mathrm{abs}}
F_\alpha\boxtimes\chi(\alpha).
$$

### C-COMP-EVAL-1 -- completed analytic camera [Definition]

For $(F,\chi)\in\mathrm{Dom}_{\mathrm{abs}}$, define

$$
\widehat{\mathcal M}_{\chi}(F)
:=
\mathrm{Agg}_{\mathrm{abs}}
\big(\mathrm{Flat}_\chi(F)\big).
$$

The superscript records that this is the ordering-independent sum supplied by
A-CNS-1. On finite states it agrees exactly with C-MELLIN-1 for the same
character. The weighted flat stack is the default retained representation;
the oriented scalar is its explicitly aggregated projection.

## Prime-log character

Work over the prime directions $\mathcal A_{\mathbb{P}}$ from D-PRIME-3.

### D-PLOG-1 -- logarithmic prime weight [Definition]

Define

$$
u_{\log}(a_k):=\log p(k).
$$

This weight uses pattern identity $k=q(p(k))$ to select the prime-value
camera output $p(k)$. It does not identify identity with multiplicative depth.

### C-PLOG-HELIX-1 -- analytic prime-log helix camera [Definition]

Using the orientation coordinates $(u_k,v_k)$ from C-BIRTH-HELIX-1, define

$$
H_{\log p}(k):=(u_k,v_k,u_{\log}(a_k))
=(u_k,v_k,\log p(k)).
$$

This is the analytic scale view of the same flat prime helix. It keeps birth
orientation in the first two coordinates and applies the logarithmic scale
camera only to the prime-value axis.

### L-PLOG-HELIX-1 -- prime-log helix remains injective [Proved]

The proof in `../proofs/12-analytic-zeta.md` shows that this camera is an
injective monotone axial deformation. The statement is a camera
classification, not a claim about prime distribution.

### D-ZCHAR-1 -- zeta character [Definition]

For $s\in\mathbb{O}$, define

$$
\chi_s(\alpha)
:=\chi_{u_{\log},s}(\alpha)
=\mathcal E_s\!\left(-U_{u_{\log}}(\alpha)\right).
$$

The character is an existing native multiplication-generated flow, specialized
by the prime-log weight. It remains distinct from the wrapped birth orientation
$\mathbf J^{\boxtimes k}=i^k$.

## Zeta series and Euler cameras

### C-ZETA-1 -- zeta series camera [Definition]

For parameters $s$ such that
$(\mathfrak Z_{\mathrm{pattern}},\chi_s)\in
\mathrm{Dom}_{\mathrm{abs}}$, define

$$
\mathcal Z_N(s)
:=
\widehat{\mathcal M}_{\chi_s}
  (\mathfrak Z_{\mathrm{pattern}}).
$$

The subscript $N$ marks the native construction. Its identification with the
conventional Dirichlet series is a theorem.

The complete projection pipeline is

$$
\mathfrak Z_{\mathrm{pattern}}
\xrightarrow{\ \mathrm{Flat}_{\chi_s}\ }
\text{indexed weighted orientation stack}
\xrightarrow{\ \mathrm{Agg}_{\mathrm{abs}}\ }
\mathcal Z_N(s)
\xrightarrow{\ \kappa\ }
\zeta(s)
$$

on the proved half-plane. Thus classical zeta is a function of $s$ and,
simultaneously, the final coordinate image of a native projection pipeline.
Projection does not make a classical proof logically impossible; the research
hypothesis is that retaining the pre-aggregation INDEX structure exposes an
invariant hidden by the scalar view.

## Paired critical-strip camera

### C-ZETA-INPUT-FRAME-1 -- zeta parameter perspective [Definition]

Write $s=\sigma+it$. The zeta input frame has axes
$(C_{\mathrm{in}},\sigma)$ and $(C_{\mathrm{in}},t)$. These parameter
axes are not the real and imaginary axes of the zeta value.

### C-ZETA-PAIR-1 -- odd/even observation pair [Definition]

For symbolic positive input $m$, use the integer-value camera and define

$$
B_s(m)
:=
\chi_s(\alpha_{2m-1})
\boxplus
\mathrm{ORIENT}_2\!\left(\chi_s(\alpha_{2m})\right).
$$

In conventional coordinates this is the single pattern law

$$
\kappa(B_s(m))
=(2m-1)^{-\kappa(s)}-(2m)^{-\kappa(s)}.
$$

This is one function of $m$, not a stored integer range. INDEX retains the
two multiplicative observations, ORIENT$_2$ supplies the minus orientation,
and ADD closes one pair.

Once T-ZETA-PAIR-CONV-1 establishes absolute summability, define the paired
scalar camera

$$
\mathcal E_N(s)
:=
\mathop{\boxplus}_{m\geq1}^{\mathrm{abs}} B_s(m)
\qquad
(\mathrm{Re}\kappa(s)>0).
$$

### C-ZETA-STRIP-1 -- paired zeta camera [Definition]

Let $\underline2:=\mathbf1\boxplus\mathbf1$. Wherever its denominator is
nonzero, define

$$
\mathcal Z_N^{\mathrm{pair}}(s)
:=
\mathcal E_N(s)
\boxtimes
\left(
\mathbf1
\boxminus
\underline2\boxtimes\chi_s(\alpha_2)
\right)^{\boxtimes-1}.
$$

Under $\kappa$, the denominator is $1-2^{1-\kappa(s)}$. The construction
uses only the existing pattern, character, INDEX selection, ORIENT, ADD,
MULTIPLY, and a proved convergent scalar camera.

T-ZETA-PAIR-IDENTITY-1 proves on $\mathrm{Re}(s)>1$ that

$$
\mathcal E_N(s)
\mathrel{=}
\left(
\mathbf1\boxminus
\underline2\boxtimes\chi_s(\alpha_2)
\right)
\boxtimes\mathcal Z_N(s).
$$

L-ZETA-PAIR-DENOM-1 proves the denominator is nonzero throughout
$0<\mathrm{Re}\kappa(s)<1$. Therefore T-ZETA-STRIP-1 identifies
$\mathcal Z_N^{\mathrm{pair}}$ as the operations-first direct-zeta camera in
the open critical strip.

### C-ZETA-CLASSICAL-COORDS-1 -- zeta value in the classical perspective [Definition]

The canonical source construction is:

```ns
let zeta_classical_pattern = () =>
classical_perspective(zeta_strip_coordinate)
```

The local coefficient camera has the already proved basis
$\mathrm{Ax}(C_{\mathrm{cl}})=\{\mathbf1,\mathbf J\}$. Its coordinate
values are

$$
Z_R(s):=\mathrm{classic}_p(\mathcal Z_N^{\mathrm{pair}}(s)),
\qquad
Z_I(s):=\mathrm{classic}_i(\mathcal Z_N^{\mathrm{pair}}(s)).
$$

These are local zeta-value coordinates. D-ZETA-RE-QUADRATIC-CAMERAS-1 wraps
the zeta pattern with $P_Z$, separately from RE's perpendicular $P_R$.
C-CAMERA-RESIDUAL-1 combines contributions separately on the local
$\mathbf1$ and $\mathbf J$ coefficient axes; T-ZETA-OUTPUT-RESIDUAL-1
proves that nothing remains exactly at zeta zero. Both coordinate maps exist
because the paired zeta value is an oriented scalar on this domain and T-CLASSIC-P-1
supplies its unique two coordinate readouts.

`classical_perspective` performs the generic residual normalization and exact
zero-fill before comparison with zero. C-CAMERA-ZERO-FILL-1 reads either
absent classical axis as exact zero. In camera notation this gives

$$
\mathrm{Frame}_{\mathrm{cl}}(\mathcal Z_N^{\mathrm{pair}}(s))
=(Z_R(s),Z_I(s)).
$$

The coordinates are used separately, but the classical axes are the same axes
used when any other oriented scalar, including the RE pattern, is viewed from
this perspective.

### C-EULER-AN-1 -- observation-ordered analytic Euler camera [Definition]

For $K\geq1$, let

$$
P_K(s)
:=
\mathop{\boxtimes}_{k=1}^{K}
\left(\mathbf1\boxminus\chi_s(\varepsilon_k)\right)^{\boxtimes-1},
$$

whenever the displayed inverses exist. Define the ordered analytic Euler
camera by

$$
\mathcal P_N(s):=\lim_{K\to\infty}P_K(s)
$$

only where that limit is proved to exist. This is a classical analytic camera
limit over successive symbolic observations; it is not a native prime range.
No convergence is built into the definition.

## Proved domains and boundary

The proofs in `../proofs/12-analytic-zeta.md` establish, for
$\sigma=\mathrm{Re}\kappa(s)>1$,

$$
\kappa(\mathcal Z_N(s))
=\sum_{n=1}^{\infty}n^{-\kappa(s)}
=\prod_{k=1}^{\infty}(1-p(k)^{-\kappa(s)})^{-1}
=\kappa(\mathcal P_N(s)).
$$

This is a reconstruction of the classical absolutely convergent zeta/Euler
identity in native operations. It is not claimed as a new theorem.

The paired proofs additionally establish

$$
\kappa(\mathcal Z_N^{\mathrm{pair}}(s))
\mathrel{=}
\frac{\sum_{m\geq1}
\left((2m-1)^{-\kappa(s)}-(2m)^{-\kappa(s)}\right)}
{1-2^{1-\kappa(s)}}
$$

throughout the open critical strip. This is the analytic continuation of the
half-plane zeta camera by T-ZETA-STRIP-1.

### T-ZETA-FLAT-1 -- the zeta stack is lossless before aggregation [Proved]

The proof in `../proofs/12-analytic-zeta.md` establishes that every zeta
character value is nonzero, so the weighted coefficient at each retained
INDEX location determines the original formal coefficient. The information
loss occurs only at the explicitly named aggregate projection.

## Explicit exclusions

- The paired camera supplies analytic continuation only on
  $\mathrm{Re}(s)>0$ where its denominator is nonzero; it does not
  construct the full meromorphic plane or the functional equation.
- The critical-strip zeta coordinate is now defined, but no claim that its zero
  forces native RE zero is inferred from that construction.
- Native RE is proved separately by T-NATIVE-RE-1; this construction does not
  prove the cross-camera zero-location implication
  `zeta_classical_pattern = 0 -> re_classical_pattern = 0` recorded as
  O-ZETA-RE-CLASSICAL-PATTERN-1.
- Absolute convergence is a domain condition, not a property of every
  completed state.
- The analytic camera is generally lossy and does not identify the formal state
  with its scalar coordinate.
- Native Space 1.0 can certify symbolic theorem instances but does not numerically
  materialize an infinite state or infinite sum.
