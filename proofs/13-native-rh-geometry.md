<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Multiplicity and RH Geometry Proofs

## Dependencies and boundary

T-AMULT-1 uses the oriented-scalar field isomorphism T-CX-2 and D-AMULT-1.
T-RH-3D-EQ-1 is a logical equivalence using C-XI-REF-1,
L-XI-ZETA-ZERO-1, C-RH-3D-1, and the conventional definition of RH. Neither
theorem proves RH or constructs the completed xi function operations-first.

### T-AMULT-1 -- native and conventional zero multiplicity agree [Proved]

**Claim.** Let $F$ satisfy D-AMULT-1 at $\rho$ with native multiplicity
$m$. Then $f=\kappa\circ F\circ\kappa^{-1}$ has a conventional zero of
multiplicity $m$ at $r=\kappa(\rho)$, and conversely.

**Proof.** Apply the additive and multiplicative field isomorphism $\kappa$
to the local native factorization:

$$
\begin{aligned}
F(s)&=(s\boxminus\rho)^{\boxtimes m}\boxtimes G(s),\\
f(z)&=(z-r)^m g(z),
\end{aligned}
$$

where $g=\kappa\circ G\circ\kappa^{-1}$ and
$g(r)=\kappa(G(\rho))\neq0$. Analyticity is preserved under the constant
J-to-i complex-linear field isomorphism, exactly as specified by the native
analytic pullback definition in D-AMULT-1. Applying $\kappa^{-1}$ proves
the converse. Uniqueness of the least local factor depth on either side gives
the same $m$. $\square$

**Boundary.** This proves that Native Space can carry analytic multiplicity as
MULTIPLY depth. It does not prove where any xi zeros occur or what their
multiplicities are.

### T-CENTERED-RE-PERSPECTIVE-1 -- the RE perspective has origin one half [Proved]

The exact Native Space definition is runnable with
`native-space derive --source examples/math-functions.ns centered_re_perspective`:

```ns
let exact_half = () =>
ADD()
MULTIPLY()

let centered_re_perspective = () =>
axis_subtract(classic_p, exact_half)
```

The current source expands to five primitive operations, in order:

```text
ORIENT(0)
ADD()
MULTIPLY()
ORIENT(2)
ADD()
```

The report also contains eight function-trace entries for navigation. Those
eight entries are not eight primitive operations. Expansion carries no proof
status; the quantified proof below establishes the camera equation.

**Claim.** For $\kappa(s)=\sigma+it$,

$$
\kappa(\mathrm{RE}_{1/2}(s))
=\sigma-\frac{1}{2},
$$

so

$$
\mathrm{RE}_{1/2}(s)=\mathbf0
\Longleftrightarrow
\mathrm{Re}\kappa(s)=\frac{1}{2}.
$$

The critical reflection negates this perspective coordinate:

$$
\mathrm{RE}_{1/2}(\mathcal R(s))
=-\mathrm{RE}_{1/2}(s).
$$

Moreover, $1/2$ is the unique translation of the real input axis for which
reflection acts by negation.

**Proof.** T-CLASSIC-P-1 maps `classic_p(s)` to $\sigma\mathbf1$.
T-QOS-1 supplies the exact native rational $\frac{1}{2}\mathbf1$, and
T-AXIS-SUBTRACT-1 makes the camera output
$(\sigma-\frac{1}{2})\mathbf1$. Uniqueness of the native number-line
coefficient from T-RJ-1 proves its zero equivalence. D-RH-REFL-1 changes
$\sigma$ to $1-\sigma$, so the centered output becomes
$(1-\sigma)-\frac{1}{2}=-(\sigma-\frac{1}{2})$, proving the reflection law.
For uniqueness, suppose the translated coordinate is $h_c(\sigma)=\sigma-c$
and reflection negates it for every $\sigma$. Then

$$
1-\sigma-c=-(\sigma-c)=-\sigma+c,
$$

so $1-c=c$ and therefore $c=1/2$. The one in this equation is the native
multiplicative identity appearing in D-RH-REFL-1. Thus the half is the unique
reflection center, not an offset chosen from the zeta output.
$\square$

**Boundary.** This proves the perspective and its origin. It does not assert
that a zeta zero has zero output under this perspective.

### L-XI-ZETA-ZERO-1 -- xi and zeta have the same strip zeros [Reproduced]

**Claim.** If $0<\mathrm{Re}z<1$, then

$$
\xi(z)=0\quad\Longleftrightarrow\quad\zeta(z)=0.
$$

**Proof.** Write C-XI-REF-1 as

$$
\xi(z)=F(z)\zeta(z),
\qquad
F(z)=\frac{1}{2}z(z-1)\Gamma(z/2)\pi^{-z/2}.
$$

Inside the open strip, $z\neq0$ and $z-1\neq0$. Also
$\pi^{-z/2}=\exp(-(z/2)\log\pi)\neq0$. Since
$\mathrm{Re}(z/2)>0$, $z/2$ is not a pole of $\Gamma$, and the
classical gamma function has no zeros. Hence $F(z)$ is finite and nonzero.
Multiplication by a nonzero complex number preserves and reflects zero, so
$F(z)\zeta(z)=0$ exactly when $\zeta(z)=0$. $\square$

**Boundary.** This reproduces the conventional xi-zeta zero correspondence in
the strip. It does not derive gamma or xi from native operations.

### T-RH-3D-EQ-1 -- RH is the native 3D axis-exclusion statement [Reproduced]

**Claim.** The conventional Riemann Hypothesis is equivalent to

$$
x(s)=y(s)=0\Longrightarrow\delta(s)=0
$$

for every $s$ whose conventional coordinate lies in the critical strip.

**Proof.** By C-XI-REF-1 and bijectivity of $\kappa$,

$$
x(s)=y(s)=0
\Longleftrightarrow
\Xi_{\mathrm{ref}}(s)=\mathbf0
\Longleftrightarrow
\xi(\kappa(s))=0.
$$

Because $\kappa(s)$ lies in the open critical strip,
L-XI-ZETA-ZERO-1 continues this equivalence to
$\zeta(\kappa(s))=0$.

By C-RH-3D-1,

$$
\delta(s)=0
\Longleftrightarrow
\mathrm{Re}\kappa(s)=\frac{1}{2}.
$$

The conventional RH statement says exactly that every nontrivial zero in the
critical strip has this real part. Substitution proves both implications.
$\square$

**Boundary.** Logical equivalence transfers the problem into native 3D
geometry. It does not prove the axis-exclusion implication. That implication
is K-RH-1 and remains conjectural.

### T-AXIS-CANCEL-1 -- 2D zero is two signed 1D cancellations [Proved]

**Claim.** D-AXIS-PROJ-1 has coordinates

$$
P_{\!R}(x,y)=(x,0),
\qquad
P_{\!I}(x,y)=(y,0),
$$

so $z=\mathbf0$ exactly when both projections vanish. For any finite or
absolutely summable family $g_k\mathbf J^{\boxtimes k}$, those projections
are the two signed residue-class balances stated in the theory.

**Proof.** D-OS-11 gives $(x,y)^\dagger=(x,-y)$. Native ADD, subtraction,
and multiplication by $1/2$ therefore give

$$
\frac{1}{2}(z\boxplus z^\dagger)=(x,0),
\qquad
\frac{1}{2}(z\boxminus z^\dagger)=(0,y).
$$

L-OS-10 gives $\mathrm{ORIENT}_3(0,y)=(y,0)$. Both outputs vanish
exactly when $x=y=0$, proving the first claim.

For the family claim, T-BIRTH-1 gives the four-cycle

$$
\mathbf J^{\boxtimes k}
\in\{\mathbf1,\mathbf J,\boxminus\mathbf1,\boxminus\mathbf J\}
$$

according to $k\bmod4$. The first projection retains the $\mathbf1$ and
$\boxminus\mathbf1$ classes with opposite signs. The second annihilates
those classes, rotates $\mathbf J$ to $\mathbf1$, and rotates
$\boxminus\mathbf J$ to $\boxminus\mathbf1$. Finite linearity follows
from the ring laws; absolute-family linearity follows from A-CNS-1. This gives
the two displayed signed balances. $\square$

**Boundary.** This proves the exact cancellation reduction for any 2D native
value. Applying the residue-class formulas to completed xi still requires an
operations-first transform that proves the relevant prime-oriented normal form
and preserves the zero set.

### T-RH-FLAT-1 -- operational form of the default RH transform [Proved]

**Claim.** The first two coordinates of C-RH-3D-1 are exactly
$P_{\!R}(\Xi_{\mathrm{ref}}(s))$ and
$P_{\!I}(\Xi_{\mathrm{ref}}(s))$, and they reconstruct the xi value by

$$
\Xi_{\mathrm{ref}}(s)
=P_{\!R}(\Xi_{\mathrm{ref}}(s))
\boxplus
\mathrm{ORIENT}_1(P_{\!I}(\Xi_{\mathrm{ref}}(s))).
$$

**Proof.** Write $\Xi_{\mathrm{ref}}(s)=(x,y)$. T-AXIS-CANCEL-1 gives
$P_{\!R}(x,y)=(x,0)$ and $P_{\!I}(x,y)=(y,0)$. L-OS-10 gives
$\mathrm{ORIENT}_1(y,0)=(0,y)$. Native ADD then returns
$(x,0)\boxplus(0,y)=(x,y)$. $\square$

**Boundary.** This proves the transform is a lossless coordinate change. It
does not prove the off-plane zero exclusion.

### T-DUAL-ALIGN-1 -- dual prime projections align exactly on the critical line [Proved]

**Claim.** Let $\kappa(s)=\sigma+it$. For a symbolic positive observation
input $k$, the two D-DUAL-CHAR-1 generator values have the same orientation
and

$$
\chi_s^+(\varepsilon_k)=\chi_s^-(\varepsilon_k)
\quad\Longleftrightarrow\quad
\sigma=\frac{1}{2}.
$$

Consequently the zero locus of the single parameterized C-DUAL-FLAT-1 pattern
camera is the critical line. The theorem is symbolic in $k$; it does not
construct a native range or stack of prime objects.

**Proof.** D-RH-REFL-1 gives

$$
\kappa(\mathcal R(s))=(1-\sigma)+it.
$$

L-ZETA-COORD-1, applied to $n=p(k)$, therefore gives

$$
\begin{aligned}
\kappa(\chi_s^+(\varepsilon_k))
&=p(k)^{-\sigma}e^{-it\log p(k)},\\
\kappa(\chi_s^-(\varepsilon_k))
&=p(k)^{-(1-\sigma)}e^{-it\log p(k)}.
\end{aligned}
$$

The orientation factor is identical and nonzero. The native comparison is the
derived axis subtraction

$$
\Delta(s;k)
:=\chi_s^+(\varepsilon_k)
\boxplus\mathrm{ORIENT}_2\!\left(\chi_s^-(\varepsilon_k)\right).
$$

By T-AXIS-SUBTRACT-1, $\Delta(s;k)=\mathbf0$ exactly when the two axes are
equal. Canceling their shared nonzero orientation then reduces this condition
to

$$
p(k)^{-\sigma}=p(k)^{-(1-\sigma)}.
$$

D-PRIME-2 gives $p(k)>1$, so the real exponential with base $p(k)$ is
injective by A-LOG-1. Hence

$$
-\sigma=-(1-\sigma)
\quad\Longleftrightarrow\quad
\sigma=\frac{1}{2}.
$$

The condition is independent of which positive observation input is selected.
L-MELLIN-2 extends generator equality multiplicatively to an arbitrary finite
multi-index, while a generator observation gives the converse.
T-AXIS-CANCEL-1 says the two flat camera coordinates are zero exactly when this
derived ADD residual $\Delta(s;k)$ is zero. $\square$

**Boundary.** This theorem identifies the critical line exactly from the two
native pattern projections. It does not prove that a completed-xi or direct-zeta
zero forces the pattern mismatch to vanish.

### L-DUAL-CONE-1 -- optional cone view has the same zero locus [Proved]

**Claim.** The symbolic C-DUAL-CONE-1 observation is the cone apex exactly when
the corresponding C-DUAL-FLAT-1 observation is zero. Therefore the cone
pattern law has the same critical-line locus.

**Proof.** T-CONE-1 gives
$Q(\Delta(s;k))=\text{apex}$ exactly when
$\Delta(s;k)=\mathbf0$. T-AXIS-CANCEL-1 gives the latter exactly when both
signed coordinates in the corresponding flat observation vanish.
T-DUAL-ALIGN-1 then proves the claim symbolically in $k$. $\square$

**Boundary.** This is an explicitly requested quadratic view of the default
flat stack. It is not used to recover orientation sign.

### T-RH-REDUCE-1 -- zero-to-alignment implies native-xi exclusion [Proved reduction]

**Claim.** Suppose O-XI-NATIVE-1 supplies the completed indexed flat stack and
its aggregate $\Xi_N$, and suppose the
O-ZERO-ALIGN-1 flat-transform law holds at a nontrivial zero in the
critical strip. Then K-NATIVE-XI-1 holds. Together with the coordinate
equivalence required by O-XI-NATIVE-1 and T-RH-3D-EQ-1, this also implies
K-RH-1.

**Proof.** Let $\Xi_N(s)=\mathbf0$ at a nontrivial point. O-ZERO-ALIGN-1
gives the pattern-law-zero flat alignment camera, hence
$\Delta(s;k)=\mathbf0$ as a symbolic observation law by T-AXIS-CANCEL-1.
T-DUAL-ALIGN-1 then gives
$\mathrm{Re}\kappa(s)=\tfrac{1}{2}$, equivalently $\delta(s)=0$.
This is exactly K-NATIVE-XI-1. The required O-XI-NATIVE-1 coordinate
equivalence identifies these native zeros with the completed classical zeros;
T-RH-3D-EQ-1 then yields K-RH-1. $\square$

**Boundary.** The implication is proved; O-ZERO-ALIGN-1 is not. This optional
completed-xi route is distinct from the direct open
O-ZETA-RE-CLASSICAL-PATTERN-1 implication.
