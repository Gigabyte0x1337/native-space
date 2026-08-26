<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Projection, Axis ADD, and Reflected-Axis Subtraction

## 1. Classical axis cameras

### D-CLASSIC-P-1 -- signed coefficient cameras [Definition]

The language declarations are:

```ns
let classic_p = () =>
ORIENT(0)

let classic_i = () =>
ORIENT(1)
```

Every native oriented scalar has a unique form

$$
z=x\mathbf1+y\mathbf J.
$$

Define

$$
\mathrm{classic}_p(z):=x,
\qquad
\mathrm{classic}_i(z):=y.
$$

The second output is the imaginary coefficient rotated onto a signed real
line. Under the classical coefficient camera
$\kappa(x\mathbf1+y\mathbf J)=x+iy$, these definitions give the exact
commuting identities

$$
\mathrm{classic}_p(z)=\mathrm{Re}(\kappa(z)),
\qquad
\mathrm{classic}_i(z)=\mathrm{Im}(\kappa(z)).
$$

### T-CLASSIC-P-1 -- native and classical projections agree [Proved]

**Proof.** Substitute $\kappa(z)=x+iy$. Its real and imaginary coefficients
are $x$ and $y$, exactly D-CLASSIC-P-1. In particular,

$$
\mathrm{Re}(1)=1=\mathrm{classic}_p(\mathbf1).
$$

Because $\mathbf1,\mathbf J$ are a basis, this proves the equality for every
native oriented scalar, not only the four unit orientations. $\square$

### T-RE-OUTPUT-RESIDUAL-1 -- automatic RE residual detects RE zero [Proved]

**Claim.** For the reflected mismatch perspective,

$$
\Delta(s;k)=\mathbf0
\quad\Longleftrightarrow\quad
\mathrm{Residual}_{\mathrm{cl}}(\Delta(s;k))=\varnothing.
$$

**Proof.** Apply T-CLASSIC-P-1 to the one oriented scalar
$\Delta(s;k)$. T-CAMERA-RESIDUAL-1 automatically combines the coordinates
on the classical axes from C-RE-CLASSICAL-COORDS-1 and leaves the empty residual exactly when that
scalar is zero. $\square$

**Boundary.** This theorem stays inside the shared classical perspective. It
does not yet relate the RE coordinate values to the zeta coordinate values.

### T-RE-OUTPUT-FRAME-1 -- total RE frame detects RE zero [Proved]

**Claim.** The two reflected-mismatch coordinate maps exist on their declared
domain, and

$$
\Delta(s;k)=\mathbf0
\quad\Longleftrightarrow\quad
\mathrm{Frame}_{\mathrm{cl}}(\Delta(s;k))=(0,0)=0.
$$

**Proof.** C-RE-CLASSICAL-COORDS-1 and T-CLASSIC-P-1 define the unique coordinates
$R_R,R_I$, proving that both declared axes exist. T-RE-OUTPUT-RESIDUAL-1
identifies RE zero with the empty sparse residual. T-CAMERA-ZERO-FILL-1 extends
that residual over $\mathrm{Ax}(C_{\mathrm{cl}})=\{\mathbf1,\mathbf J\}$ and makes its empty
residual exactly the total zero frame. $\square$

**Boundary.** The source proof explicitly calls `classical_perspective` before
comparison with zero. The classical axes are shared with zeta.

## 2. Four orientation functions

### D-FOUR-ORIENT-FUNCTIONS-1 -- ordered comparison block [Definition]

```ns
let p_minus_i = () =>
ORIENT(3)

let p_i = () =>
ORIENT(1)

let p_one = () =>
ORIENT(0)

let p_minus_one = () =>
ORIENT(2)

let classical_four_projection = () =>
apply(p_minus_i, p_i, p_one, p_minus_one)
```

For number-line gains $a_{-i},a_i,a_1,a_{-1}$, define

$$
\begin{aligned}
P_{-i}(a_{-i})&=-a_{-i}\mathbf J,\\
P_i(a_i)&=a_i\mathbf J,\\
P_1(a_1)&=a_1\mathbf1,\\
P_{-1}(a_{-1})&=-a_{-1}\mathbf1.
\end{aligned}
$$

The explicit classical comparison order is

$$
(-i,i,1,-1).
$$

This order does not alter pattern identity $q(p(k))=k$, multiplicative depth,
or wrapped birth orientation $\mathbf J^k$.

### T-CLASSICAL-FOUR-CAMERA-1 -- projection table [Proved]

For unit gains, T-CLASSIC-P-1 gives

| function | $\mathrm{classic}_p$ | $\mathrm{classic}_i$ |
|---|---:|---:|
| $P_{-i}$ | $0$ | $-1$ |
| $P_i$ | $0$ | $1$ |
| $P_1$ | $1$ | $0$ |
| $P_{-1}$ | $-1$ | $0$ |

Thus ordinary `Re` does not see the $-i/i$ pair. The rotated imaginary
camera maps $-i\mapsto-1$ and $i\mapsto1$, while not seeing the
$1/-1$ pair. Adding the four classical camera coordinates axis by axis gives

$$
\sum \mathrm{classic}_p=0+0+1-1=0,
\qquad
\sum \mathrm{classic}_i=-1+1+0+0=0.
$$

Thus the classical axes cancel directly. $\square$

## 3. Signed-axis ADD

### T-AXIS-ADD-CANCEL-1 -- opposite signed axes cancel [Proved]

```ns
let axis_add = (left, right) =>
left()
right()
ADD()
```

Native axis cancellation is ADD:

$$
(-a)\boxplus a=0.
$$

Consequently, for one shared gain $a$,

$$
P_{-i}(a)\boxplus P_i(a)=\mathbf0,
\qquad
P_1(a)\boxplus P_{-1}(a)=\mathbf0.
$$

This follows directly from the additive-inverse law L-OS-2. There is no fifth
operation, aggregate, limit, or repetition predicate. $\square$

### T-FOUR-AXIS-CANCEL-1 -- independently weighted residuals [Proved]

For four independent gains, finite ADD gives

$$
\begin{aligned}
P_{-i}(a_{-i})\boxplus P_i(a_i)
&=(a_i-a_{-i})\mathbf J,\\
P_1(a_1)\boxplus P_{-1}(a_{-1})
&=(a_1-a_{-1})\mathbf1.
\end{aligned}
$$

Therefore both axes cancel exactly when

$$
a_i=a_{-i}
\quad\text{and}\quad
a_1=a_{-1}.
$$

The source function `four_axis_cancellation` is the equal-gain specialization.
Unequal symbolic gains are covered by the paper formula above; Native Space
1.0 does not execute symbolic quantified gain proofs. $\square$

## 4. Axis subtraction is derived ADD

### D-AXIS-SUBTRACT-1 -- turn then add [Definition]

```ns
let axis_subtract = (left, right) =>
left()
right()
ORIENT(2)
ADD()
```

For two same-facing native axes, define

$$
\mathrm{axis\_subtract}(A,B)
:=
A\boxplus\mathrm{ORIENT}_2(B).
$$

The second axis is turned to its opposite orientation and then both axes are
added. Subtraction is therefore not primitive.

### T-AXIS-SUBTRACT-1 -- camera equivalence and zero fiber [Proved]

The classical camera preserves ADD and maps a half-turn to multiplication by
$-1$. Hence

$$
\kappa(\mathrm{axis\_subtract}(A,B))
=\kappa(A)-\kappa(B).
$$

Because the camera is injective,

$$
\mathrm{axis\_subtract}(A,B)=\mathbf0
\quad\Longleftrightarrow\quad
A=B.
$$

This proves both the operation and its zero condition. $\square$

## 5. Reflected prime axes

For $s=\sigma+it$ and prime $p$, the two proved prime-axis cameras are

$$
A_p(s)=p^{-\sigma}e^{-it\log p},
\qquad
B_p(s)=p^{-(1-\sigma)}e^{-it\log p}.
$$

Apply D-AXIS-SUBTRACT-1:

$$
\Delta_p(s)
=A_p(s)-B_p(s)
=\left(p^{-\sigma}-p^{-(1-\sigma)}\right)e^{-it\log p}.
$$

The common orientation factor is nonzero. Since $p>1$, A-LOG-1 makes the
real power injective. Therefore

$$
\begin{aligned}
\Delta_p(s)=0
&\Longleftrightarrow p^{-\sigma}=p^{-(1-\sigma)}\\
&\Longleftrightarrow \sigma=1-\sigma\\
&\Longleftrightarrow \boxed{\sigma=\tfrac{1}{2}}.
\end{aligned}
$$

This is T-DUAL-ALIGN-1 in its minimal operations-first form:

$$
\boxed{
\text{PROJECT TWO REFLECTED AXES}
\to
\text{TURN THE SECOND}
\to
\text{ADD}
\to
0
\Longleftrightarrow
\mathrm{Re}(s)=\tfrac{1}{2}.
}
$$

## 6. Exact status after the centered-RE proofs

The axis algebra and critical-line detector above are proved. In native names,
the remaining statement is that zero of `zeta_classical_pattern` implies zero
of `re_classical_pattern`. The classical camera verification is

$$
\mathrm{Residual}_{Z}(s)=\varnothing
\Longrightarrow
\mathrm{Residual}_{R}(s;k)=\varnothing.
$$

That is O-ZETA-RE-CLASSICAL-PATTERN-1. After automatic zero-fill in each local
coefficient camera, it is

$$
\mathrm{Frame}_{\mathrm{cl}}(\mathcal Z_N^{\mathrm{pair}}(s))=0
\Longrightarrow
\mathrm{Frame}_{\mathrm{cl}}(\Delta(s;k))=0.
$$

Both local outputs use the same classical coefficient convention, while
T-ZETA-RE-ROTATION-1 and T-ZETA-RE-POSITION-1 place them as separate cameras
under their perpendicular quadratic projectors. The remaining proof must
establish the stated wrapped-pattern relation.

Thus the exact status is:

- classical/native projection agreement: proved;
- half of the multiplicative identity is the unique midpoint of the reflected
  real positions `sigma` and `1 - sigma`: proved;
- `centered_re_perspective(s) = classic_p(s) - 1/2`, while zeta remains
  centered at ADD zero: proved;
- centered perspective zero exactly at `Re(s) = 1/2`: proved;
- signed-axis cancellation: proved;
- reflected RE mismatch and its classical output frame are zero exactly on the
  critical line: proved;
- `zeta_classical_pattern = 0` implies `re_classical_pattern = 0`: open and
  RH-equivalent.
