<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Multiplicity and the Three-Dimensional RH Geometry

Native Space already has enough algebraic structure to state the Riemann
Hypothesis as an exact coordinate-operation problem. The flat direct-zeta route
does not require a new geometry primitive or completed xi. The cone remains an
optional quadratic camera of the same native coordinates.

This document separates six statuses:

- native zero multiplicity is defined and proved compatible with conventional
  analytic multiplicity;
- the centered input perspective is proved to be
  `classic_p(s) - 1/2` and reflection negates it;
- native reflected equality (RE) is separately proved: its symbolic mismatch
  pattern has the same critical-line zero locus;
- the same-state camera zero equivalence is proved for two explicitly wrapped
  views of one legal classical-source value;
- the within-classical-pattern implication from zeta zero to native RE zero is
  the separate classical-transfer obligation and is open;
- the completed-xi and 3D formulations remain optional alternative cameras;
- classical RH is K-RH-1;
- the future operations-first completed-xi exclusion is the distinct
  K-NATIVE-XI-1 transfer target.

## Canonical centered-RE chain

The repository uses three related native objects and does not collapse their
names:

$$
\begin{aligned}
\delta(s)
&:=\mathrm{centered\_re\_perspective}(s)
=\mathrm{classic}_p(s)-\frac{1}{2},\\
\Delta(s;k)
&:=\mathrm{centered\_re\_pattern}(s;k),\\
R_{\mathrm{cl}}(s;k)
&:=\mathrm{re\_classical\_pattern}(s;k)
=\mathrm{classical\_perspective}(\Delta(s;k)).
\end{aligned}
$$

Their proved zero chain is

$$
R_{\mathrm{cl}}(s;k)=0
\Longleftrightarrow
\Delta(s;k)=0
\Longleftrightarrow
\delta(s)=0
\Longleftrightarrow
\mathrm{Re}(s)=\frac{1}{2}.
$$

The first equivalence is T-RE-OUTPUT-FRAME-1, the second is the zero-locus
content of T-NATIVE-RE-1 together with T-CENTERED-RE-PERSPECTIVE-1, and the
third is T-CENTERED-RE-PERSPECTIVE-1. The values of $\Delta$ and $\delta$
need not be equal away from zero; they have the same proved zero locus.

The direct-zeta route asks for one additional implication into this already
proved chain:

$$
\mathrm{zeta\_classical\_pattern}(s)=0
\Longrightarrow
R_{\mathrm{cl}}(s;k)=0.
$$

That implication is O-ZETA-RE-CLASSICAL-PATTERN-1. It is not part of the
centered-perspective or native-RE proofs.

## Proved native theorem and classical caveat

For one legal state $\Gamma$ on the one-dimensional classical source axis,
keep both camera wrappers:

$$
Z_N(\Gamma)=P_Z\Gamma,
\qquad
R_N(\Gamma)=P_R\Gamma.
$$

T-ZETA-RE-CLASSICAL-AXIS-ROTATION-1 proves

$$
R_N(\Gamma)=R_{-90}Z_N(\Gamma),
$$

and therefore T-NATIVE-PERSPECTIVE-ZERO-1 proves

$$
\boxed{Z_N(\Gamma)=0\Longleftrightarrow R_N(\Gamma)=0.}
$$

This is the proved same-state camera zero theorem. The perspective is part of
every term; deleting it changes the statement. It is not itself RH.

The one-dimensional source does not retain the recursive multiplicative prime
pattern. T-ZETA-RE-GLOBAL-FACTOR-NO-1 proves that interpreting its two rotated
views globally as actual zeta and centered RE is false: it would make every
point on the critical line a zeta zero. The corrected source must retain INDEX,
MULTIPLY-recursion, ORIENT, and analytic gain before applying two generally
lossy projections. O-ZETA-RE-CLASSICAL-PATTERN-1 records the still-open one-way
zero-fiber inclusion on that richer source.

K-PRIME-FLATTENING-1 records the separate structural hypothesis motivating
this representation: the complete recursive multiplicative prime pattern is
not reconstructible from the flattened classical camera unless equivalent
native structure is added back. This hypothesis concerns representability,
not the already proved projected zero theorem. It remains conjectural until a
non-factorization result is proved on the actual generative prime-pattern
image.

## Analytic multiplicity as native depth

### D-AMULT-1 -- native analytic multiplicity [Definition]

For this analytic camera layer, call $F:D\to\mathbb{O}$ **native analytic**
when

$$
f:=\kappa\circ F\circ\kappa^{-1}
$$

is conventionally holomorphic on the corresponding coordinate domain. This is
a typed pullback definition through the proved J-to-i field isomorphism; it is
not the still-open operations-first construction of every analytic function.

A zero of native-analytic $F$ at $\rho$ has **native multiplicity**
$m\in\mathbb{N}_{>0}$ when
$m$ is the unique depth for which

$$
F(s)=(s\boxminus\rho)^{\boxtimes m}\boxtimes G(s)
$$

on some neighborhood of $\rho$, where $G$ is analytic and
$G(\rho)\neq\mathbf0$.

The exponent is repeated native MULTIPLY. The number $m$ is therefore a
multiplicative depth. It is not a prime birth index and not a wrapped
orientation.

### T-AMULT-1 -- native and conventional zero multiplicity agree [Proved]

The proof is in `../proofs/13-native-rh-geometry.md`. It uses only the field
isomorphism $\kappa$ and the local factor definition of analytic
multiplicity.

## Completed-xi reference boundary

### C-XI-REF-1 -- completed-xi reference lift [Definition]

For the sole purpose of specifying the reconstruction target, let

$$
\xi(z)
=\frac{1}{2} z(z-1)\Gamma(z/2)\pi^{-z/2}\zeta(z)
$$

be the conventional entire Riemann xi function, and define its oriented-scalar
reference lift

$$
\Xi_{\mathrm{ref}}(s):=\kappa^{-1}(\xi(\kappa(s))).
$$

This lift proves only that the oriented-scalar carrier can hold each value. It
does **not** count as the required operations-first native construction because
it imports $\xi$ through the coordinate camera.

### L-XI-ZETA-ZERO-1 -- xi and zeta have the same strip zeros [Reproduced]

For $0<\mathrm{Re}z<1$,

$$
\xi(z)=0\quad\Longleftrightarrow\quad\zeta(z)=0.
$$

Indeed, every factor multiplying $\zeta(z)$ in C-XI-REF-1 is finite and
nonzero in the open critical strip: $z\neq0$, $z-1\neq0$,
$\pi^{-z/2}\neq0$, and $\Gamma(z/2)$ is finite and nonzero because
$\mathrm{Re}(z/2)>0$. This is a reproduced classical analytic fact,
not an operations-first construction of $\xi$. The complete argument is in
`../proofs/13-native-rh-geometry.md`.

### O-XI-NATIVE-1 -- operations-first completed-xi obligation [Open]

Construct an indexed completed flat stack
$\widetilde{\mathfrak X}_N(s)$ from native states, flows, sums or integrals,
and justified limits. Define its scalar projection only afterward by

$$
\Xi_N(s)
:=
\mathrm{Agg}_{\Xi}
\big(\widetilde{\mathfrak X}_N(s)\big),
$$

then prove

$$
\kappa(\Xi_N(s))=\xi(\kappa(s))
$$

without using the right-hand formula as a premise. This obligation requires at
least a native theta/Mellin continuation mechanism, the gamma camera or an
equivalent construction, and a native derivation of the reflection law. The
indexed stack must be retained through the construction; it may not be
recovered by attempting to invert the generally lossy scalar aggregation.
The construction must define and justify $\mathrm{Agg}_{\Xi}$ on its
actual domain; absolute summability from the half-plane is not assumed in the
critical strip.

**Completion gate.** O-XI-NATIVE-1 closes only when the repository contains
the source-defined native stack and aggregate, a complete proof that they are
defined on the stated critical-strip domain, and derivations—without importing
classical $\xi$ as a premise—of both
$\kappa(\Xi_N(s))=\xi(\kappa(s))$ and the required reflection law. A
successful operation trace or agreement at finitely many inputs does not close
the obligation.

The reference lift and the aggregate of the future native construction have
the same target coordinate but different evidential status. Classical xi is a
function of $s$ and the final camera image of this native projection
pipeline.

## Native reflection

### D-RH-REFL-1 -- critical reflection [Definition]

For $s\in\mathbb{O}$, define

$$
\mathcal R(s):=\mathbf1\boxminus s^\dagger.
$$

If $\kappa(s)=\sigma+it$, then

$$
\kappa(\mathcal R(s))=(1-\sigma)+it.
$$

Thus $\mathcal R$ negates the centered real coordinate

$$
\delta(s):=\sigma-\frac{1}{2}
$$

while preserving $t$. Its fixed set is exactly the critical line
$\delta=0$.

The value $1/2$ is forced by the multiplicative identity in the reflection.
The real positions $\sigma$ and $1-\sigma$ are a reflected pair whose ADD
midpoint is

$$
\frac{\sigma+(1-\sigma)}2=\frac{1}{2}.
$$

More intrinsically, let $h_c(\sigma)=\sigma-c$ be any translated real-axis
camera. Requiring reflection to become orientation reversal,

$$
h_c(1-\sigma)=-h_c(\sigma),
$$

gives $1-c=c$, hence the unique center $c=1/2$. The centered coordinate is
therefore a signed height above or below the reflection plane. It is not a
translation of the zeta value.

### C-CENTERED-RE-PERSPECTIVE-1 -- RE perspective centered at one half [Definition]

The native definition is authoritative:

```ns
let exact_half = () =>
ADD()
MULTIPLY()

let centered_re_perspective = () =>
axis_subtract(classic_p, exact_half)
```

The RE perspective itself is the affine classical-axis camera

$$
\mathrm{RE}_{1/2}(s)
:=
\mathrm{classic}_p(s)
\boxminus
\frac{1}{2}\mathbf1.
$$

Its origin is therefore $1/2$ on the classical real axis, not classical
zero. Under $\kappa(s)=\sigma+it$, its one coordinate is

$$
\kappa(\mathrm{RE}_{1/2}(s))=\sigma-\frac{1}{2}=\delta(s).
$$

The two zero origins must remain separate. Zeta is a value coordinate and its
zero is the native ADD identity $\mathbf0$. RE is the extra input-position
coordinate and its zero is the symmetry plane obtained after subtracting
$\mathbf1/2$. Centering zeta would instead test $\zeta(s)=1/2$, which is
not the Riemann-zero condition.

The two camera poses are proved independently in
`../proofs/23-zeta-re-perspectives.md`:

$$
P_Z=\frac{1}{2}\begin{pmatrix}1&1\\1&1\end{pmatrix},
\qquad
P_R=\frac{1}{2}\begin{pmatrix}1&-1\\-1&1\end{pmatrix}.
$$

Their directions are 90 degrees apart, and each quadratic camera places the
classical multiplicative identity at exactly one half. This quadratic camera
position is distinct from translating a zeta value; zeta remains tested at
its ADD zero.

The source function `centered_re_perspective` constructs this camera from
`classic_p`, the exact rational `1/2`, ORIENT$_2$, and ADD. The later
reflected-prime pattern is a representation of this centered perspective; it
does not define the perspective's origin.

Its current derivation contains five primitive-operation steps and eight
function-trace entries. The trace explains how functions were expanded; the
camera equation and zero law come from T-CENTERED-RE-PERSPECTIVE-1, not from
the trace length.

The conventional xi identities imply the reference symmetry

$$
\Xi_{\mathrm{ref}}(\mathcal R(s))
=\Xi_{\mathrm{ref}}(s)^\dagger.
$$

For $\Xi_N$, deriving this relation from native operations belongs to
O-XI-NATIVE-1; it is not imported as a native theorem.

## The flat three-dimensional zero geometry

### C-RH-3D-1 -- default centered xi flat transform [Definition]

Write

$$
\Xi_{\mathrm{ref}}(s)=(x(s),y(s))
$$

in the native oriented-scalar coordinates and define

$$
\mathcal F_{\mathrm{RH}}(s)
:=
\left(x(s),y(s),\delta(s)\right)\in\mathbb{R}^3.
$$

The third coordinate is displacement from the critical symmetry plane. Under
the reference reflection, the geometry transforms as

$$
(x,y,\delta)\longmapsto(x,-y,-\delta).
$$

On the fixed plane $\delta=0$, the reflection relation forces $y=0$, so
the critical-line xi trajectory lies in the real oriented direction. Its zeros
are then signed crossings in that direction, with multiplicity represented by
D-AMULT-1 depth.

### T-RH-3D-EQ-1 -- RH is the 3D axis-exclusion statement [Reproduced]

Within the critical strip $0<\sigma<1$, the conventional Riemann Hypothesis
is logically equivalent to

$$
x(s)=0\ \wedge\ y(s)=0
\quad\Longrightarrow\quad
\delta(s)=0.
$$

Equivalently,

$$
\mathcal F_{\mathrm{RH}}(s)=(0,0,d)
\quad\Longrightarrow\quad d=0.
$$

The proof in `../proofs/13-native-rh-geometry.md` establishes the equivalence,
not the truth of either equivalent statement.

## Quadratic cone camera

The flat 3D geometry is the most economical formulation because it retains the
centered parameter coordinate. The existing quadratic cone remains an exact
value camera:

$$
\mathcal Q_\Xi(s):=Q(\Xi_{\mathrm{ref}}(s)).
$$

T-CONE-1 gives

$$
\mathcal Q_\Xi(s)=\text{cone apex}
\quad\Longleftrightarrow\quad
\Xi_{\mathrm{ref}}(s)=\mathbf0
\quad\Longleftrightarrow\quad
x(s)=y(s)=0.
$$

Thus RH is also the statement that every nontrivial cone-apex event occurs at
$\delta=0$. The cone detects vanishing quadratically; the flatter native
coordinates retain the orientation and the location needed to classify it.

## The two native prime projections

### D-AXIS-PROJ-1 -- two signed-axis projections [Definition]

For $z\in\mathbb{O}$, define

$$
P_{\!R}(z)
:=\frac{1}{2}\left(z\boxplus z^\dagger\right),
$$

$$
P_{\!I}(z)
:=\mathrm{ORIENT}_3\!\left(
\frac{1}{2}\left(z\boxminus z^\dagger\right)
\right).
$$

If $z=(x,y)$, then

$$
P_{\!R}(z)=(x,0),
\qquad
P_{\!I}(z)=(y,0).
$$

The second projection rotates the $i/-i$ axis onto the $1/-1$ axis. It
does not discard that axis.

### T-AXIS-CANCEL-1 -- 2D zero is two signed 1D cancellations [Proved]

The proof in `../proofs/13-native-rh-geometry.md` establishes

$$
z=\mathbf0
\quad\Longleftrightarrow\quad
P_{\!R}(z)=\mathbf0
\ \text{and}\ 
P_{\!I}(z)=\mathbf0.
$$

For a finite or absolutely summable birth-oriented family

$$
z=\mathop{\boxplus}_k g_k\mathbf J^{\boxtimes k},
\qquad g_k\in\mathbb{R},
$$

this is exactly

$$
\sum_{k\equiv0\ (4)}g_k-
\sum_{k\equiv2\ (4)}g_k=0,
$$

$$
\sum_{k\equiv1\ (4)}g_k-
\sum_{k\equiv3\ (4)}g_k=0.
$$

Thus the native four-orientation prime layout reduces a two-dimensional zero
to two signed $1/-1$ balances.

### T-RH-FLAT-1 -- operational form of the default RH transform [Proved]

Applying D-AXIS-PROJ-1 to the xi value gives the exact operations-first form

$$
\mathcal F_{\mathrm{RH}}(s)
=
\left(P_{\!R}(\Xi_{\mathrm{ref}}(s)),
      P_{\!I}(\Xi_{\mathrm{ref}}(s)),
      \delta(s)\right),
$$

where each of the first two outputs lies on the signed native number line and
is identified with its real coordinate in C-RH-3D-1. The value portion is
lossless because

$$
\Xi_{\mathrm{ref}}(s)
=P_{\!R}(\Xi_{\mathrm{ref}}(s))
\boxplus
\mathrm{ORIENT}_1(P_{\!I}(\Xi_{\mathrm{ref}}(s))).
$$

This follows immediately from the coordinate formulas in T-AXIS-CANCEL-1.
For the future $\Xi_N$, the same algebraic transform and inverse apply once
O-XI-NATIVE-1 constructs the input state.

### D-DUAL-CHAR-1 -- reflected prime-log projections [Definition]

For a native parameter $s$ with $\kappa(s)=\sigma+it$, use the same
formal prime state with the two characters

$$
\chi_s^+:=\chi_s,
\qquad
\chi_s^-:=\chi_{\mathcal R(s)}.
$$

These are two projections generated inside the same oriented-scalar and
prime-index system. They do not use two unrelated representations.

For one symbolic observation input $k$, define the native mismatch law

$$
\Delta(s;k)
:=
\chi_s^+(\varepsilon_k)
\boxminus
\chi_s^-(\varepsilon_k).
$$

### C-DUAL-FLAT-1 -- default flat pattern-comparison camera [Definition]

Apply the default signed-axis transform to the symbolic mismatch:

$$
\mathcal D_{\mathrm{flat}}(s;k)
:=
\left(k,P_{\!R}(\Delta(s;k)),P_{\!I}(\Delta(s;k))\right).
$$

This is the default alignment camera of the one generative prime pattern. It
retains observation INDEX, both signed cancellation coordinates, and therefore
the complete oriented mismatch. Its pattern-law zero means that the symbolic
residual is identically zero:

$$
P_{\!R}(\Delta(s;k))=P_{\!I}(\Delta(s;k))=\mathbf0.
$$

Here $k$ is a free observation input in one formula. The definition does not
materialize a stack, range, or prime-by-prime computation.

### C-RE-CLASSICAL-COORDS-1 -- reflected mismatch local coordinates [Definition]

The reflected mismatch is another oriented scalar using the same local
coefficient convention as zeta. Its coordinate values on the local
$\mathbf1,\mathbf J$ axes are

$$
R_R(s;k):=\mathrm{classic}_p(\Delta(s;k)),
\qquad
R_I(s;k):=\mathrm{classic}_i(\Delta(s;k)).
$$

They are not separate RE-owned axes. The pairs $(Z_R,Z_I)$ and
$(R_R,R_I)$ are coordinate values of different patterns on the same two
classical axes. C-CAMERA-RESIDUAL-1 reduces the RE pattern on those axes;
T-RE-OUTPUT-RESIDUAL-1 proves the resulting zero equivalence.

Both RE coordinate maps exist because $\Delta(s;k)$ is an oriented scalar and
T-CLASSIC-P-1 supplies its unique readouts. The source construction is:

```ns
let re_classical_pattern = () =>
classical_perspective(centered_re_pattern)
```

C-CAMERA-ZERO-FILL-1 then supplies the following classical verification
coordinate:

$$
\mathrm{Frame}_{\mathrm{cl}}(\Delta(s;k))
=(R_R(s;k),R_I(s;k)),
$$

reading an absent residual axis as exact zero while keeping the two axes
separate.

### C-DUAL-CONE-1 -- optional dual-prime mismatch cone [Definition]

Apply the existing quadratic cone camera to the same symbolic mismatch:

$$
\mathcal A(s;k):=Q(\Delta(s;k)).
$$

This is the optional quadratic view of the default flat pattern camera. A cone
apex means the two projections agree at the selected observation, but the cone
is not the default representation.

### T-DUAL-ALIGN-1 -- the reflected pattern aligns exactly on the critical line [Proved]

The proof in `../proofs/13-native-rh-geometry.md` establishes

$$
\kappa(\chi_s^+(\varepsilon_k))
=p(k)^{-\sigma}e^{-it\log p(k)},
$$

$$
\kappa(\chi_s^-(\varepsilon_k))
=p(k)^{-(1-\sigma)}e^{-it\log p(k)}.
$$

Thus, for symbolic observation $k$, the two projections have the same
orientation. Their gains agree exactly when

$$
\sigma=\frac{1}{2}.
$$

Equivalently,

$$
\delta(s)=0
\Longleftrightarrow
\Delta(s;k)=\mathbf0
\Longleftrightarrow
\mathcal D_{\mathrm{flat}}(s;k)=\text{pattern-law zero}.
$$

The optional cone corollary is

$$
\delta(s)=0
\Longleftrightarrow
\mathcal A(s;k)=\text{cone apex}.
$$

The equivalence is independent of the selected positive observation input, so
one symbolic rule proves the critical-line locus without constructing a native
range. This is a proved native characterization of the critical line, not
merely a visual analogy.

## Direct coordinate-function route

No new geometry carrier is required to compose the RH reduction. Every
coordinate is a source-defined function over native operations. None of these
names belongs to the language kernel.

The documented source route is:

```ns
let zeta_classical_pattern = () =>
classical_perspective(zeta_strip_coordinate)

let centered_re_pattern = () =>
centered_re_perspective()
dual_prime_alignment()

let re_classical_pattern = () =>
classical_perspective(centered_re_pattern)

let zeta_re_classical_pattern = () =>
zeta_classical_pattern()
re_classical_pattern()
```

The source expander records each pattern's local coefficient camera. The
proved wrapper then applies $P_Z$ only to zeta and $P_R$ only to RE. In
native names, the remaining relation says: zero of `zeta_classical_pattern`
implies zero of `re_classical_pattern`. Its local coefficient-camera check is

$$
\mathrm{Frame}_{\mathrm{cl}}(\mathcal Z_N^{\mathrm{pair}}(s))=0
\Longrightarrow
\mathrm{Frame}_{\mathrm{cl}}(\Delta(s;k))=0
$$

is O-ZETA-RE-CLASSICAL-PATTERN-1. T-ZETA-RE-ROTATION-1 and
T-ZETA-RE-POSITION-1 prove the exact camera constants, orientations, and
identity positions. The remaining statement is then a relation between the
two separately wrapped patterns. T-CAMERA-ZERO-FILL-1 makes every local output
axis explicit and preserves the equivalent sparse-residual formulation.

The offset is not part of that remaining work. `centered_re_perspective`
constructs the exact half-offset directly, and `centered_re_pattern` adds the
reflected-prime realization whose zero locus is proved by T-DUAL-ALIGN-1 and
T-NATIVE-RE-1.

The complete direct target is therefore written in one chain:

$$
\boxed{
\zeta(s)=0
\Longrightarrow
\Delta(s;k)=0
\Longleftrightarrow
\mathrm{Re}(s)-\frac{1}{2}=0
}.
$$

The right equivalence is proved; the left implication is
O-ZETA-RE-CLASSICAL-PATTERN-1.
T-ZERO-FIBER-FACTOR-1 and T-TWO-PROJECTION-FIBER-1 prove that neither
strand-dependent reorientation nor adding a reflected scalar-zero projection
supplies that invariant as a generic algebra law.

T-RH-DIRECT-EQ-1 proves that O-ZETA-RE-CLASSICAL-PATTERN-1 is equivalent to K-RH-1 by
T-ZETA-STRIP-1 and T-DUAL-ALIGN-1. This is an exact reformulation, not a
reduction in difficulty. The obligation stays open in the dependency ledger.

### Finite pattern strategy

The finite-pattern strategy uses the one generative pattern directly. For symbolic
observation input $k$, the value camera supplies $p(k)>1$. After removing
the shared analytic phase from the reflected mismatch, the remaining real gain
law is

$$
g(\sigma;k)=p(k)^{-\sigma}-p(k)^{-(1-\sigma)}.
$$

T-NATIVE-RE-1 proves symbolically that this gain is positive for
$\sigma<\tfrac{1}{2}$, zero at $\sigma=\tfrac{1}{2}$, and negative for
$\sigma>\tfrac{1}{2}$. Therefore the deterministic dephased prime pattern is
zero exactly on the critical line. This is one parameterized pattern theorem;
no prime range, next-prime loop, or infinite execution is involved.

The remaining obligation is O-ZETA-RE-CLASSICAL-PATTERN-1: derive, through
explicit native operations, that `zeta_classical_pattern = 0` forces
`re_classical_pattern = 0`. T-RH-PATTERN-CONSEQUENCE-1 proves the conditional
consequence. The implication itself remains open; determinism alone does not
establish it.

The wrapped birth orientation $i^k$ is not the phase removed here. Birth
orientation is indexed metadata from T-BIRTH-1; the common analytic phase is
$e^{-it\log p(k)}$, generated by the prime-log character.

T-AXIS-CANCEL-1 already proves the finite cancellation pattern of the four
wrapped orientations: the `1/-1` balance and the `i/-i` balance must each be
zero. Rotating the second balance onto a signed real line is lossless while its
lane label is retained. What is not automatic is removing every individual
analytic phase after scalar aggregation; the elementary cancellation
$1+i-1-i=0$ becomes $1+1+1+1=4$ if each orientation is independently
discarded. The zeta-specific pattern implication must therefore act before
aggregation or prove its zero-fiber law on the special zeta state.

The native observer construction does not change this boundary.
T-ALL-PERSPECTIVES-1 proves that the view from orientation $O_r$ is
$O_r^{-1}$ times the whole aggregate. All four observer equations are
therefore invertible rotations of one zero equation, not four independent
coefficient equations. T-OPPOSITE-PAIR-1 proves the proposed geometry exactly:
births $k$ and $k+2$ are opposite, but their weighted contributions cancel
only when their gains agree. The fully native proof and finite counterexample
are in `../proofs/19-zeta-zero-fiber-audit.md`.

### Classical-axis cameras and native cancellation

The corrected native construction uses four functions over one number-line
gain $a$:

$$
P_{-i}(a)=-a\mathbf J,
\quad
P_i(a)=a\mathbf J,
\quad
P_1(a)=a\mathbf1,
\quad
P_{-1}(a)=-a\mathbf1.
$$

The two coefficient cameras are

$$
\mathrm{classic}_p(z)=\mathrm{Re}(\kappa(z)),
\qquad
\mathrm{classic}_i(z)=\mathrm{Im}(\kappa(z)).
$$

On the ordered comparison block $(-i,i,1,-1)$, they return

$$
\mathrm{classic}_p=(0,0,1,-1),
\qquad
\mathrm{classic}_i=(-1,1,0,0).
$$

Adding the visible signed coordinates on each classical axis therefore gives

$$
(-1)+1=0,
\qquad
1+(-1)=0.
$$

The same statement before projection is exactly native ADD:

$$
P_{-i}(a)\boxplus P_i(a)=\mathbf0,
\qquad
P_1(a)\boxplus P_{-1}(a)=\mathbf0.
$$

The comparison order does not change pattern identity $q(p(k))=k$,
multiplicative depth, or wrapped birth orientation $\mathbf J^k$.

For two same-facing axes, subtraction is derived rather than primitive:

$$
\mathrm{axis\_subtract}(A,B)
=A\boxplus\mathrm{ORIENT}_2(B).
$$

For the reflected prime axes

$$
A(s;k)=p(k)^{-\sigma}e^{-it\log p(k)},
\qquad
B(s;k)=p(k)^{-(1-\sigma)}e^{-it\log p(k)},
$$

this derived subtraction is zero exactly when $\sigma=1/2$. The complete
camera, ADD, and reflected-axis proof is in
`../proofs/22-projection-and-axis-subtraction.md`.

For independently weighted functions, opposite orientations cancel only when
their gains agree. The remaining RH work is consequently still
O-ZETA-RE-CLASSICAL-PATTERN-1: prove that an operations-first critical-strip zeta zero is
carried by the stated projections to the reflected-axis zero. The four unit
orientations do not assume that zeta-specific gains are equal.

## Alternative completed-xi transform

### O-ZERO-ALIGN-1 -- completed-xi flat transform has the decisive zero fiber [Open obligation]

Construct one exact native completed-xi transform whose output is the default
flat stack $\widetilde{\mathfrak X}_N(s)$. O-XI-NATIVE-1 is its
construction/equivalence clause. The present obligation is its zero-fiber
clause across the final aggregate projection:

$$
\mathrm{Agg}_{\Xi}
\big(\widetilde{\mathfrak X}_N(s)\big)=\mathbf0
\quad\Longrightarrow\quad
\mathcal D_{\mathrm{flat}}(s;k)=\text{pattern-law zero}
$$

at nontrivial points in the critical strip.

This is now the exact missing transform law. T-AXIS-CANCEL-1 already turns the
aggregate zero into two real cancellation equations. The retained stack must
derive the zeta-specific signed prime balances and prove that simultaneous
cancellation maps to the pattern-law-zero alignment locus. Its required
property is zero-fiber preservation on completed-xi states; no global
injectivity claim for arbitrary completed states is needed.

The transform may prove the stronger symbolic identity
$\Delta(s;k)=0$, or directly prove that its two aggregate signed balances can
vanish simultaneously only when $\delta(s)=0$. These are two
certification strategies for one transform, not two independent RH steps.

**Completion gate.** O-ZERO-ALIGN-1 closes only when O-XI-NATIVE-1 is already
closed and a complete proof establishes the displayed zero-fiber implication
for every nontrivial critical-strip input in the constructed native domain.
Finite witnesses, a derivation trace, or a proof for arbitrary diagonal states
without the completed-xi specialization do not close it.

### T-RH-REDUCE-1 -- zero-to-alignment implies native-xi exclusion [Proved reduction]

If O-XI-NATIVE-1 and O-ZERO-ALIGN-1 are discharged, then K-NATIVE-XI-1 follows
immediately from T-DUAL-ALIGN-1. With the coordinate equivalence required by
O-XI-NATIVE-1, T-RH-3D-EQ-1 then transfers that exclusion to K-RH-1. This
conditional reduction is proved in `../proofs/13-native-rh-geometry.md`; it does
not claim that either open premise is already proved.

## Exact conjectures

### K-RH-1 -- classical Riemann Hypothesis [Conjecture]

For every nontrivial classical zeta zero (z) in the critical strip,

$$
\zeta(z)=0
\quad\Longrightarrow\quad
\mathrm{Re}(z)=\frac{1}{2}.
$$

This is the conjecture used by the direct-zeta equivalence
T-RH-DIRECT-EQ-1.

### K-NATIVE-XI-1 -- native completed-xi off-plane exclusion [Conjecture]

After O-XI-NATIVE-1 is discharged, prove for every nontrivial point in the
critical strip:

$$
\Xi_N(s)=\mathbf0
\quad\Longrightarrow\quad
\delta(s)=0.
$$

An acceptable proof may discharge O-ZERO-ALIGN-1 directly or derive a stronger
native invariant that implies it. The dual-projection alignment theorem itself
is already proved; only its connection to completed-xi zero cancellation is
open.

## Why the 3D reformulation is useful but not yet the classical transfer

The reformulation exposes the exact missing theorem as a geometric exclusion,
rather than treating RH as an opaque statement about a conventional function.
More strongly, T-DUAL-ALIGN-1 proves that the two prime projections share
orientation everywhere and become gain-aligned exactly on the critical line.
The remaining work is not to discover the critical geometry; it is to prove
O-ZERO-ALIGN-1 for the operations-first completed-xi transform.

## Verification references

- [NIST DLMF 25.4](https://dlmf.nist.gov/25.4) records the completed xi
  definition and reflection equation.
- [NIST DLMF 25.10](https://dlmf.nist.gov/25.10) records the critical strip,
  critical line, zero symmetries, and the conventional RH statement.

These sources specify the conventional target. They are not dependencies of a
future operations-first proof of O-XI-NATIVE-1 or K-NATIVE-XI-1.
