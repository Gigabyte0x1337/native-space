<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Ground-Up Proof of the Native Space Core

**Proof status:** Complete paper proof relative to A-RF-1 and A-N-1  
**Research stage:** 3 -- Prove the native system from the ground up  
**Machine-checked:** No

## 1. Permitted dependencies

This proof uses only:

- A-RF-1: the ordinary real-field laws;
- A-N-1: the ordinary nonnegative-integer semiring laws and finite counting;
- definitions D-OS-1 through D-NS-12.

It does not use a complex-number camera, matrix camera, polar coordinates, the
PSD/cone cameras, transforms, primes, or application evidence.

After L-OS-1 proves oriented-scalar ADD associative and commutative, finite
$\boxplus$-folds may be reordered and reparenthesized. Before that point,
the canonical fold convention from D-OS applies.

## 2. Oriented-scalar algebra

Throughout this section let

$$
z=(x,y),\qquad w=(u,v),\qquad t=(p,q)
$$

be arbitrary members of $\mathbb{O}=\mathbb{R}^2$.

### L-OS-1 -- Scalar ADD laws [Proved]

**Claim.** $\boxplus$ is closed, associative, and commutative.

**Proof.** Closure follows because $x+u,y+v\in\mathbb{R}$, so
$(x+u,y+v)\in\mathbb{R}^2$.

Associativity follows coordinatewise:

$$
(z\boxplus w)\boxplus t
=((x+u)+p,(y+v)+q)
=(x+(u+p),y+(v+q))
=z\boxplus(w\boxplus t).
$$

Commutativity also follows coordinatewise:

$$
z\boxplus w=(x+u,y+v)=(u+x,v+y)=w\boxplus z.
$$

Only the corresponding real-field laws were used. $\square$

### L-OS-2 -- Scalar ADD identity and inverse [Proved]

**Claim.** $\mathbf0$ is the ADD identity and $\boxminus z$ is the
additive inverse of $z$.

**Proof.** Directly,

$$
z\boxplus\mathbf0=(x+0,y+0)=(x,y)=z
$$

and

$$
z\boxplus(\boxminus z)=(x-x,y-y)=(0,0)=\mathbf0.
$$

Commutativity from L-OS-1 gives the corresponding left identities. $\square$

### L-OS-3 -- Scalar MULTIPLY laws [Proved]

**Claim.** $\boxtimes$ is closed, associative, and commutative.

**Proof.** Closure follows because $xu-yv,xv+yu\in\mathbb{R}$.

For associativity, expand the left association:

$$
\begin{aligned}
(z\boxtimes w)\boxtimes t
={}&((xu-yv)p-(xv+yu)q,\\
   &(xu-yv)q+(xv+yu)p)\\
={}&(xup-yvp-xvq-yuq,\\
   &xuq-yvq+xvp+yup).
\end{aligned}
$$

Expand the right association:

$$
\begin{aligned}
z\boxtimes(w\boxtimes t)
={}&(x(up-vq)-y(uq+vp),\\
   &x(uq+vp)+y(up-vq))\\
={}&(xup-xvq-yuq-yvp,\\
   &xuq+xvp+yup-yvq).
\end{aligned}
$$

The coordinates agree after reordering real sums and products. Hence the
operation is associative.

For commutativity,

$$
z\boxtimes w=(xu-yv,xv+yu)
=(ux-vy,uy+vx)=w\boxtimes z.
$$

Thus all three claims hold. $\square$

### L-OS-4 -- Scalar MULTIPLY identity [Proved]

**Claim.** $\mathbf1=(1,0)$ is the identity for $\boxtimes$.

**Proof.**

$$
z\boxtimes\mathbf1=(x\cdot1-y\cdot0,x\cdot0+y\cdot1)=(x,y)=z.
$$

The left identity follows from L-OS-3 commutativity. $\square$

### L-OS-5 -- Scalar distributivity [Proved]

**Claim.** $\boxtimes$ distributes over $\boxplus$.

**Proof.** Expand:

$$
\begin{aligned}
z\boxtimes(w\boxplus t)
&=(x(u+p)-y(v+q),\ x(v+q)+y(u+p))\\
&=(xu-yv+xp-yq,\ xv+yu+xq+yp)\\
&=(z\boxtimes w)\boxplus(z\boxtimes t).
\end{aligned}
$$

Right distributivity follows from L-OS-3 commutativity. $\square$

### L-OS-6 -- Scalar zero absorption [Proved]

**Claim.** $\mathbf0\boxtimes z=\mathbf0$.

**Proof.** Direct substitution gives

$$
(0,0)\boxtimes(x,y)=(0x-0y,0y+0x)=(0,0).
$$

The other order follows from commutativity. $\square$

### L-OS-7 -- Zero-size characterization [Proved]

**Claim.** $\nu(z)=0$ if and only if $z=\mathbf0$.

**Proof.** If $z=\mathbf0$, then $\nu(z)=0^2+0^2=0$.

Conversely, $x^2\geq0$ and $y^2\geq0$ in the ordinary real field. If
$x^2+y^2=0$, neither square can be positive; hence both are zero. A real
square is zero only when its base is zero, so $x=y=0$, and
$z=\mathbf0$. $\square$

### L-OS-8 -- Nonzero scalar inverse [Proved]

**Claim.** If $z\neq\mathbf0$, then the D-OS-6 candidate satisfies
$z\boxtimes z^{[-1]}=\mathbf1$.

**Proof.** L-OS-7 gives $\nu(z)\neq0$, so the candidate is defined. Then

$$
\begin{aligned}
z\boxtimes z^{[-1]}
&=(x,y)\boxtimes
\left(\frac{x}{x^2+y^2},-\frac{y}{x^2+y^2}\right)\\
&=\left(
\frac{x^2+y^2}{x^2+y^2},
\frac{-xy+yx}{x^2+y^2}
\right)\\
&=(1,0)=\mathbf1.
\end{aligned}
$$

Commutativity gives the inverse in the other order. $\square$

### L-OS-9 -- Multiplicativity of squared size [Proved]

**Claim.** $\nu(z\boxtimes w)=\nu(z)\nu(w)$.

**Proof.** Expand the left side:

$$
\begin{aligned}
\nu(z\boxtimes w)
&=(xu-yv)^2+(xv+yu)^2\\
&=x^2u^2-2xuyv+y^2v^2
  +x^2v^2+2xvyu+y^2u^2\\
&=x^2(u^2+v^2)+y^2(u^2+v^2)\\
&=(x^2+y^2)(u^2+v^2)\\
&=\nu(z)\nu(w).
\end{aligned}
$$

The mixed terms cancel in the real field. $\square$

### L-OS-10 -- Four-step orientation cycle [Proved]

**Claim.**

$$
\mathbf J^{\boxtimes2}=\mathbf{-1},\qquad
\mathbf J^{\boxtimes3}=\mathbf{-J},\qquad
\mathbf J^{\boxtimes4}=\mathbf1.
$$

**Proof.** From D-OS-3 and D-OS-4,

$$
(0,1)\boxtimes(0,1)=(-1,0)=\mathbf{-1}.
$$

Multiplying once more by $\mathbf J$ gives

$$
(0,1)\boxtimes(-1,0)=(0,-1)=\mathbf{-J},
$$

and once more gives

$$
(0,1)\boxtimes(0,-1)=(1,0)=\mathbf1.
$$

These are exactly the recursive powers from D-OS-7. $\square$

### T-ORIENT-ZERO-1 -- the four native orientations cancel [Proved]

**Claim.**

$$
\mathbf1
\boxplus\mathbf J
\boxplus\mathbf{-1}
\boxplus\mathbf{-J}
=\mathbf0.
$$

**Proof.** D-OS-1 and L-OS-10 identify the four states as

$$
(1,0),\quad(0,1),\quad(-1,0),\quad(0,-1).
$$

Using native ADD from D-OS-2 and the real-field additive inverse law,

$$
(1+0-1+0,\ 0+1+0-1)=(0,0)=\mathbf0.
$$

Equivalently, associativity and commutativity pair
$\mathbf1\boxplus\mathbf{-1}=\mathbf0$ and
$\mathbf J\boxplus\mathbf{-J}=\mathbf0$. No classical complex-number or
prime model is used. $\square$

### L-OS-11 -- Orientation action law [Proved]

**Claim.** For $r,s\in\mathbb{Z}_4$,

$$
\mathrm{orient}_{r+s}
\mathrel{=}
\mathrm{orient}_r\circ\mathrm{orient}_s,
$$

where the subscript sum is reduced modulo four.

**Proof.** Associativity gives the ordinary power rule

$$
\mathbf J^{\boxtimes r}\boxtimes
\mathbf J^{\boxtimes s}
\mathrel{=}
\mathbf J^{\boxtimes(r+s)}
$$

for nonnegative representatives. L-OS-10 gives
$\mathbf J^{\boxtimes4}=\mathbf1$, so removing any factor of four leaves the
power unchanged. Therefore

$$
\begin{aligned}
\mathrm{orient}_r(\mathrm{orient}_s(z))
&=\mathbf J^{\boxtimes r}\boxtimes
  (\mathbf J^{\boxtimes s}\boxtimes z)\\
&=(\mathbf J^{\boxtimes r}\boxtimes
   \mathbf J^{\boxtimes s})\boxtimes z\\
&=\mathbf J^{\boxtimes(r+s\bmod4)}\boxtimes z\\
&=\mathrm{orient}_{r+s\bmod4}(z).
\end{aligned}
$$

No angle or complex-number fact was used. $\square$

### T-REL-ORIENT-1 -- every native orientation locates every other [Proved]

**Claim.** For $r,s,t,u\in\mathbb{Z}_4$,

$$
\mathrm{Rel}(r,s)=O_{s-r\bmod4},
$$

and the complete relation table is

| reference \\ target | $\mathbf1$ | $\mathbf J$ | $\mathbf{-1}$ | $\mathbf{-J}$ |
|---|---:|---:|---:|---:|
| $\mathbf1$ | $\mathbf1$ | $\mathbf J$ | $\mathbf{-1}$ | $\mathbf{-J}$ |
| $\mathbf J$ | $\mathbf{-J}$ | $\mathbf1$ | $\mathbf J$ | $\mathbf{-1}$ |
| $\mathbf{-1}$ | $\mathbf{-1}$ | $\mathbf{-J}$ | $\mathbf1$ | $\mathbf J$ |
| $\mathbf{-J}$ | $\mathbf J$ | $\mathbf{-1}$ | $\mathbf{-J}$ | $\mathbf1$ |

Relative orientations compose and ignore a common global turn:

$$
\mathrm{Rel}(r,s)\boxtimes\mathrm{Rel}(s,t)
=\mathrm{Rel}(r,t),
$$

$$
\mathrm{Rel}(r+u,s+u)=\mathrm{Rel}(r,s).
$$

**Proof.** L-OS-8 supplies the inverse, while L-OS-10 and L-OS-11 identify it
as $O_r^{-1}=O_{-r\bmod4}$. Associativity therefore gives

$$
O_r^{-1}\boxtimes O_s
=O_{-r}\boxtimes O_s
=O_{s-r\bmod4}.
$$

Substituting the four possible differences gives the displayed table. For
composition,

$$
O_{s-r}\boxtimes O_{t-s}=O_{t-r}.
$$

For a common turn $u$,

$$
O_{(s+u)-(r+u)}=O_{s-r}.
$$

Every step uses only native MULTIPLY, inverse, and the proved four-cycle.
$\square$

### T-PERSPECTIVE-ZERO-1 -- cancellation is native-perspective invariant [Proved]

**Claim.** For every $r\in\mathbb{Z}_4$ and every finite oriented family
$z_1,\ldots,z_n$,

$$
\mathrm{View}_r
\left(\boxplus_{j=1}^{n}z_j\right)
\mathrel{=}
\boxplus_{j=1}^{n}\mathrm{View}_r(z_j),
$$

and

$$
\boxplus_{j=1}^{n}z_j=\mathbf0
\quad\Longleftrightarrow\quad
\boxplus_{j=1}^{n}\mathrm{View}_r(z_j)=\mathbf0.
$$

**Proof.** D-REL-ORIENT-1 defines every view as left MULTIPLY by the fixed
nonzero orientation $O_r^{-1}$. Distributivity gives the first identity. If
the original sum is zero, L-OS-6 makes its view zero. Conversely, multiplying
a zero viewed sum by $O_r$ cancels $O_r^{-1}$ by L-OS-8 and returns the
original sum, so it too is zero. $\square$

**Consequence.** T-ORIENT-ZERO-1 is true from all four native perspectives.
Changing the reference orientation changes every relative label but cannot
create or destroy cancellation.

### L-OS-12 -- Conjugation laws [Proved]

**Claim.** Conjugation preserves ADD, preserves MULTIPLY, fixes
$\mathbf0,\mathbf1$, and is an involution:

$$
\begin{aligned}
(z\boxplus w)^\dagger&=z^\dagger\boxplus w^\dagger,\\
(z\boxtimes w)^\dagger&=z^\dagger\boxtimes w^\dagger,\\
\mathbf0^\dagger&=\mathbf0,\quad
\mathbf1^\dagger=\mathbf1,\\
(z^\dagger)^\dagger&=z.
\end{aligned}
$$

**Proof.** The ADD statement is

$$
(x+u,y+v)^\dagger=(x+u,-y-v)
=(x,-y)\boxplus(u,-v).
$$

For MULTIPLY,

$$
(z\boxtimes w)^\dagger=(xu-yv,-xv-yu),
$$

while

$$
(x,-y)\boxtimes(u,-v)=(xu-yv,-xv-yu).
$$

The fixed points and involution follow immediately from
$(x,y)^\dagger=(x,-y)$. $\square$

### T-RJ-1 -- the native number line and J-system exist internally [Proved]

**Claim.** D-RJ-1 defines an injective field embedding
$\lambda:\mathbb{R}\to\mathbb{O}$, the distinguished orientation satisfies
$\mathbf J^{\boxtimes2}=\lambda(-1)$, and every oriented scalar has a
unique decomposition $\lambda(a)\boxplus\jmath(b)$.

**Proof.** For all $a,b\in\mathbb{R}$, D-OS-2 and D-OS-3 give

$$
\begin{aligned}
\lambda(a+b)&=(a+b,0)=(a,0)\boxplus(b,0),\\
\lambda(ab)&=(ab,0)=(a,0)\boxtimes(b,0),\\
\lambda(0)&=\mathbf0,\qquad \lambda(1)=\mathbf1.
\end{aligned}
$$

If $\lambda(a)=\lambda(b)$, coordinate equality gives $a=b$, so the
map is injective. If $a\neq0$, D-OS-6 gives

$$
\lambda(a)^{[-1]}=(a^{-1},0)=\lambda(a^{-1}),
$$

so the embedded line preserves nonzero inverses as well as ADD, MULTIPLY, and
the identities. It is therefore a field embedding from the declared real
substrate.

D-OS-3 and D-OS-4 give

$$
\mathbf J\boxtimes\mathbf J=(-1,0)=\lambda(-1).
$$

Also

$$
\jmath(b)=\lambda(b)\boxtimes\mathbf J=(b,0)\boxtimes(0,1)=(0,b).
$$

Hence every $z=(x,y)$ satisfies

$$
z=(x,0)\boxplus(0,y)=\lambda(x)\boxplus\jmath(y).
$$

If the same state also equals
$\lambda(a)\boxplus\jmath(b)=(a,b)$, coordinate equality forces
$a=x$ and $b=y$. The decomposition is unique. No complex camera or
complex-number law was used. $\square$

**Boundary.** The theorem constructs a real-line subfield and a square root of
native $-1$ inside the oriented carrier. Identifying the resulting field
with conventional $\mathbb{C}$ remains the separate camera theorem T-CX-2.

### T-OS-1 -- Oriented scalars form a commutative field [Proved]

**Claim.** $(\mathbb{O},\boxplus,\boxtimes,\mathbf0,\mathbf1)$ is a
commutative field.

**Proof.** L-OS-1 and L-OS-2 give an abelian group under ADD. L-OS-3 and
L-OS-4 give a commutative monoid under MULTIPLY. L-OS-5 gives distributivity.
The identities differ because $(0,0)\neq(1,0)$. L-OS-8 gives a
multiplicative inverse for every nonzero element. These are the field axioms.
$\square$

**Does not establish:** any identification with conventional complex numbers;
that is a separate Stage 4 theorem.

## 3. Index algebra

Let $\alpha,\beta,\gamma\in\mathbb{I}_{\mathcal A}$.

### L-IDX-1 -- Finite-support closure [Proved]

**Claim.** $\alpha\oplus_I\beta$ has finite support.

**Proof.** If a label lies outside
$\mathrm{supp}(\alpha)\cup\mathrm{supp}(\beta)$, both values are
zero, so their sum is zero. Hence

$$
\mathrm{supp}(\alpha\oplus_I\beta)
\subseteq
\mathrm{supp}(\alpha)\cup\mathrm{supp}(\beta).
$$

The union of two finite sets is finite. $\square$

### L-IDX-2 -- Index composition laws [Proved]

**Claim.** $\oplus_I$ is associative and commutative.

**Proof.** For every $a\in\mathcal A$,

$$
((\alpha\oplus_I\beta)\oplus_I\gamma)(a)
=(\alpha(a)+\beta(a))+\gamma(a)
=\alpha(a)+(\beta(a)+\gamma(a))
=(\alpha\oplus_I(\beta\oplus_I\gamma))(a).
$$

Pointwise equality gives associativity. Likewise,

$$
(\alpha\oplus_I\beta)(a)=\alpha(a)+\beta(a)
=\beta(a)+\alpha(a)=(\beta\oplus_I\alpha)(a),
$$

so composition is commutative. $\square$

### L-IDX-3 -- Zero-index identity [Proved]

**Claim.** $\mathbf0_I$ is the identity for $\oplus_I$.

**Proof.** For every label $a$,

$$
(\alpha\oplus_I\mathbf0_I)(a)=\alpha(a)+0=\alpha(a).
$$

Pointwise equality gives the result; commutativity gives the other order.
$\square$

### L-IDX-4 -- Exact directional increment [Proved]

**Claim.** For $j,k\in K_{\mathcal A}$,

$$
d_j(\varepsilon_k\oplus_I\alpha)
=d_j(\alpha)+\delta_{jk}.
$$

**Proof.** By D-IDX-5, $\varepsilon_k(a_j)=1$ when $j=k$ and zero
otherwise. Therefore

$$
d_j(\varepsilon_k\oplus_I\alpha)
=\varepsilon_k(a_j)+\alpha(a_j)
=\delta_{jk}+d_j(\alpha).
$$

Natural addition is commutative. $\square$

### L-IDX-5 -- Additivity of total depth [Proved]

**Claim.**

$$
\mathrm{depth}(\alpha\oplus_I\beta)
\mathrel{=}
\mathrm{depth}(\alpha)+\mathrm{depth}(\beta).
$$

**Proof.** All values are nonnegative, so the support of the pointwise sum is
the union of the two supports. Extending each finite sum by zero over that union,

$$
\begin{aligned}
\mathrm{depth}(\alpha\oplus_I\beta)
&=\sum_{a\in\mathrm{supp}(\alpha)\cup
\mathrm{supp}(\beta)}(\alpha(a)+\beta(a))\\
&=\sum_a\alpha(a)+\sum_a\beta(a)\\
&=\mathrm{depth}(\alpha)+\mathrm{depth}(\beta).
\end{aligned}
$$

Only finite natural sums occur. $\square$

### T-IDX-1 -- Multi-indices form a commutative monoid [Proved]

**Claim.**

$$
(\mathbb{I}_{\mathcal A},\oplus_I,\mathbf0_I)
$$

is a commutative monoid.

**Proof.** Closure is L-IDX-1, associativity and commutativity are L-IDX-2,
and the identity is L-IDX-3. $\square$

**Does not establish:** additive inverses for nonzero indices. They do not exist
inside the nonnegative multi-index carrier.

## 4. Native-state algebra

Let $F,G,H\in\mathcal N_{\mathcal A}$.

### L-NS-1 -- ADD preserves finite support [Proved]

**Claim.** $F\oplus G$ is a finite native state.

**Proof.** Outside
$\mathrm{supp}(F)\cup\mathrm{supp}(G)$, both coefficients are
$\mathbf0$, and L-OS-2 gives
$\mathbf0\boxplus\mathbf0=\mathbf0$. Thus the support of the sum is a
subset of a finite union. $\square$

### L-NS-2 -- Native ADD group [Proved]

**Claim.** Native states form a commutative group under $\oplus$.

**Proof.** Closure is L-NS-1. At each coefficient, associativity and
commutativity follow from L-OS-1, so state equality D-NS-2 lifts both laws
pointwise. D-NS-3 and L-OS-2 give

$$
(F\oplus\mathsf0)_\alpha
=F_\alpha\boxplus\mathbf0=F_\alpha.
$$

D-NS-6 and L-OS-2 give

$$
(F\oplus(\ominus F))_\alpha
=F_\alpha\boxplus(\boxminus F_\alpha)=\mathbf0.
$$

Therefore every state has an additive inverse and all abelian-group laws hold.
$\square$

### L-NS-3 -- MULTIPLY preserves finite support [Proved]

**Claim.** $F\star G$ is a finite native state.

**Proof.** Every potentially nonzero output index belongs to

$$
S:=\{\alpha\oplus_I\beta:
\alpha\in\mathrm{supp}(F),
\beta\in\mathrm{supp}(G)\}.
$$

The Cartesian product of the two finite supports is finite, so its image $S$
is finite. Outside $S$, the defining coefficient fold is empty and equals
$\mathbf0$. Hence the product has finite support. $\square$

### L-NS-4 -- Single-term multiplication [Proved]

**Claim.**

$$
c[\alpha]\star d[\beta]
=(c\boxtimes d)[\alpha\oplus_I\beta].
$$

**Proof.** The two input supports each contain at most one index. The only
candidate pair in D-NS-7 is $(\alpha,\beta)$, and it contributes
$c\boxtimes d$ at $\alpha\oplus_I\beta$. Every other output coefficient
has an empty fold and is $\mathbf0$. State equality gives the claim.
$\square$

### L-NS-5 -- Native MULTIPLY laws [Proved]

**Claim.** $\star$ is associative and commutative.

**Proof.** Closure is L-NS-3.

For commutativity, the map $(\alpha,\beta)\mapsto(\beta,\alpha)$ bijects the
finite contributing pairs for $(F\star G)_\gamma$ and
$(G\star F)_\gamma$. L-IDX-2 preserves the output index under this swap, and
L-OS-3 preserves each coefficient product. L-OS-1 makes finite fold order
irrelevant. Thus every output coefficient agrees.

For associativity, fix an output index $\delta$. Repeatedly applying D-NS-7
and distributing finite coefficient sums using L-OS-5 gives

$$
((F\star G)\star H)_\delta
\mathrel{=}
\mathop{\boxplus}_{(\alpha\oplus_I\beta)\oplus_I\gamma=\delta}
(F_\alpha\boxtimes G_\beta)\boxtimes H_\gamma.
$$

Similarly,

$$
(F\star(G\star H))_\delta
\mathrel{=}
\mathop{\boxplus}_{\alpha\oplus_I(\beta\oplus_I\gamma)=\delta}
F_\alpha\boxtimes(G_\beta\boxtimes H_\gamma).
$$

L-IDX-2 says the two finite triple-index conditions are identical. L-OS-3 says
the paired coefficient products are identical. L-OS-1 permits the same finite
terms to be folded in either order. Therefore the coefficients agree for every
$\delta$, and state equality gives associativity. $\square$

### L-NS-6 -- Native MULTIPLY identity [Proved]

**Claim.** $\mathsf1$ is the identity for $\star$.

**Proof.** The only nonzero coefficient of $\mathsf1$ is $\mathbf1$ at
$\mathbf0_I$. For each $\gamma$, L-IDX-3 makes the only possible
contributing pair $(\mathbf0_I,\gamma)$, and L-OS-4 gives

$$
(\mathsf1\star F)_\gamma
=\mathbf1\boxtimes F_\gamma
=F_\gamma.
$$

State equality proves the left identity; L-NS-5 gives the right identity.
$\square$

### L-NS-7 -- Native distributivity [Proved]

**Claim.** $\star$ distributes over $\oplus$.

**Proof.** Fix $\gamma$. D-NS-7 and L-OS-5 give

$$
\begin{aligned}
(F\star(G\oplus H))_\gamma
&=\mathop{\boxplus}_{\alpha\oplus_I\beta=\gamma}
F_\alpha\boxtimes(G_\beta\boxplus H_\beta)\\
&=\mathop{\boxplus}_{\alpha\oplus_I\beta=\gamma}
\big((F_\alpha\boxtimes G_\beta)
\boxplus(F_\alpha\boxtimes H_\beta)\big)\\
&=(F\star G)_\gamma\boxplus(F\star H)_\gamma\\
&=((F\star G)\oplus(F\star H))_\gamma.
\end{aligned}
$$

All folds are finite, so regrouping is justified by L-OS-1. State equality
gives left distributivity; L-NS-5 gives right distributivity. $\square$

### L-NS-8 -- Native zero absorption [Proved]

**Claim.** $\mathsf0\star F=\mathsf0$.

**Proof.** The zero state's support is empty, so every coefficient fold in
D-NS-7 is empty and equals $\mathbf0$. Thus the product is $\mathsf0$.
Commutativity gives the other order. $\square$

### T-NS-1 -- Native states form a commutative ring [Proved]

**Claim.**

$$
(\mathcal N_{\mathcal A},\oplus,\star,\mathsf0,\mathsf1)
$$

is a commutative ring with identity.

**Proof.** L-NS-2 supplies the abelian group under ADD. L-NS-5 and L-NS-6
supply the commutative monoid under MULTIPLY. L-NS-7 supplies distributivity.
L-NS-8 supplies zero absorption. The two identities differ because their
coefficients at $\mathbf0_I$ are $\mathbf0$ and $\mathbf1$, which differ
by T-OS-1. These are the commutative-ring axioms. $\square$

**Does not establish:** that every nonzero native state is invertible. The
theorem is a ring theorem, not a field theorem.

## 5. Separation laws

### L-SEP-1 -- ORIENT preserves indices [Proved]

**Claim.** `ORIENT` changes coefficients only and preserves every term's
multi-index.

**Proof.** D-NS-8 defines

$$
(\mathrm{ORIENT}_rF)_\alpha
=\mathrm{orient}_r(F_\alpha)
$$

at the same $\alpha$. No index function occurs on the right. A zero
coefficient may remain zero or a coefficient may be changed, but no coefficient
is assigned to a different index. $\square$

### L-SEP-2 -- INDEX increments exactly one depth [Proved]

**Claim.** `INDEX(k)` preserves coefficients and increments only directional
depth $k$ by one.

**Proof.** By D-NS-9, D-NS-10, and L-NS-4,

$$
\mathrm{INDEX}_k(F)
=\mathsf X_k\star F
=\mathop{\bigoplus}_{\alpha\in\mathrm{supp}(F)}
F_\alpha[\varepsilon_k\oplus_I\alpha].
$$

The coefficient remains
$\mathbf1\boxtimes F_\alpha=F_\alpha$ by L-OS-4. Natural-number
cancellation makes the shift $\alpha\mapsto\varepsilon_k\oplus_I\alpha$
injective, so distinct terms do not collide. L-IDX-4 gives

$$
d_j(\varepsilon_k\oplus_I\alpha)
=d_j(\alpha)+\delta_{jk}.
$$

Thus only depth $k$ increases. $\square$

### L-SEP-3 -- ORIENT is multiplication by a state [Proved]

**Claim.**

$$
\mathrm{ORIENT}_r(F)
\mathrel{=}
\eta(\mathbf J^{\boxtimes r})\star F.
$$

**Proof.** The embedded scalar has one coefficient,
$\mathbf J^{\boxtimes r}$, at $\mathbf0_I$. By L-NS-4 and L-IDX-3,
its product with each term is

$$
(\mathbf J^{\boxtimes r}\boxtimes F_\alpha)
[\mathbf0_I\oplus_I\alpha]
\mathrel{=}
\mathrm{orient}_r(F_\alpha)[\alpha].
$$

Collecting the finite terms gives exactly D-NS-8. $\square$

### L-SEP-4 -- ORIENT and INDEX commute [Proved]

**Claim.**

$$
\mathrm{ORIENT}_r(\mathrm{INDEX}_k(F))
\mathrel{=}
\mathrm{INDEX}_k(\mathrm{ORIENT}_r(F)).
$$

**Proof.** By L-SEP-3 and D-NS-10, the two sides are respectively

$$
\eta(\mathbf J^{\boxtimes r})\star(\mathsf X_k\star F)
$$

and

$$
\mathsf X_k\star(\eta(\mathbf J^{\boxtimes r})\star F).
$$

They agree by L-NS-5 associativity and commutativity. $\square$

### L-SEP-5 -- State ORIENT action law [Proved]

**Claim.** For native states $F$ and integer turn counts interpreted modulo
four,

$$
\mathrm{ORIENT}_r(\mathrm{ORIENT}_s(F))
=\mathrm{ORIENT}_{r+s}(F),
\qquad
\mathrm{ORIENT}_{4m}(F)=F.
$$

**Proof.** By D-NS-8, the coefficient at every multi-index $\alpha$ on the
left is

$$
\mathrm{orient}_r(\mathrm{orient}_s(F_\alpha)).
$$

L-OS-11 identifies this with
$\mathrm{orient}_{r+s}(F_\alpha)$, which is the coefficient of the
first right-hand side. L-OS-10 and L-OS-11 show that four turns act as the
scalar identity, so every coefficient of $\mathrm{ORIENT}_{4m}(F)$ equals
$F_\alpha$. Equality is coefficientwise by D-NS-2. $\square$

## 6. Self-action laws

### L-ACT-1 -- Additive action composition [Proved]

**Claim.**

$$
\mathsf A_F\circ\mathsf A_G=\mathsf A_{F\oplus G}.
$$

**Proof.** For arbitrary $H$,

$$
(\mathsf A_F\circ\mathsf A_G)(H)
=F\oplus(G\oplus H)
=(F\oplus G)\oplus H
=\mathsf A_{F\oplus G}(H)
$$

by L-NS-2 associativity. The transformations agree on every input. $\square$

### L-ACT-2 -- Multiplicative action composition [Proved]

**Claim.**

$$
\mathsf M_F\circ\mathsf M_G=\mathsf M_{F\star G}.
$$

**Proof.** For arbitrary $H$,

$$
(\mathsf M_F\circ\mathsf M_G)(H)
=F\star(G\star H)
=(F\star G)\star H
=\mathsf M_{F\star G}(H)
$$

by L-NS-5 associativity. $\square$

### T-ACT-1 -- Limited self-representation [Proved]

**Claim.** Native states represent additive and multiplicative actions on their
own carrier, and native composition represents action composition.

**Proof.** D-NS-11 assigns $\mathsf A_F$ and $\mathsf M_F$ to every
native state $F$. Both transformations map
$\mathcal N_{\mathcal A}$ to itself by L-NS-1 and L-NS-3. L-ACT-1 and
L-ACT-2 show their compositions are represented by $F\oplus G$ and
$F\star G$, which are states in the same carrier. Finally,

$$
\mathsf A_F(\mathsf0)=F,
\qquad
\mathsf M_F(\mathsf1)=F
$$

by L-NS-2 and L-NS-6. Thus each state is recovered as its action on the
appropriate identity. $\square$

**Does not establish:** that every transformation on Native Space is represented
by a state, or that every camera is an internal action.

### T-SELF-PATTERN-1 -- Finite observations remain self-represented [Proved]

**Claim.** For every D-SELF-PATTERN-1 pair $(S,F)$, each observation
$P_{S,F}(k)$ at finite $k\in\mathbb{N}_0$ is a native state. The finite
composition of its step action is represented by a native state in the same
carrier.

**Proof.** At $k=0$, the observation is the declared native seed $S$.
Suppose $P_{S,F}(k)$ is native. T-ACT-1 says both
$\mathsf A_F$ and $\mathsf M_F$ map native states to native states, so
$P_{S,F}(k+1)=T_F(P_{S,F}(k))$ is native. Induction proves the first claim
for every finite constructed input.

For action composition, the zero state represents the identity additive
action because $\mathsf A_{\mathsf0}(H)=H$, and the unit state represents
the identity multiplicative action because $\mathsf M_{\mathsf1}(H)=H$.
L-ACT-1 and L-ACT-2 then show inductively that adjoining one more step produces
another representing state using native ADD or MULTIPLY. Thus every finite
step composition is represented in the same carrier. No range, completed
sequence, or infinity element is used. $\square$

**Boundary.** This theorem proves finite observations and their internal step
composition. Saying that a particular classical unbounded construction has
this form requires its own camera derivation.

## 7. Coefficient-pairing laws

### L-PAIR-1 -- Pairing additivity [Proved]

**Claim.** The D-NS-12 pairing is additive in each argument.

**Proof.** All relevant supports are finite. Using L-OS-12 and L-OS-5,

$$
\begin{aligned}
\langle R\oplus S,F\rangle
&=\mathop{\boxplus}_\alpha
(R_\alpha\boxplus S_\alpha)^\dagger\boxtimes F_\alpha\\
&=\mathop{\boxplus}_\alpha
(R_\alpha^\dagger\boxplus S_\alpha^\dagger)\boxtimes F_\alpha\\
&=\langle R,F\rangle\boxplus\langle S,F\rangle.
\end{aligned}
$$

Similarly,

$$
\begin{aligned}
\langle R,F\oplus G\rangle
&=\mathop{\boxplus}_\alpha
R_\alpha^\dagger\boxtimes(F_\alpha\boxplus G_\alpha)\\
&=\langle R,F\rangle\boxplus\langle R,G\rangle.
\end{aligned}
$$

Zero coefficients may be included in the folds because L-OS-6 makes their
contribution zero. $\square$

### L-PAIR-2 -- Embedded scalar behavior [Proved]

**Claim.** For $c\in\mathbb{O}$,

$$
\begin{aligned}
\langle\eta(c)\star R,F\rangle
&=c^\dagger\boxtimes\langle R,F\rangle,\\
\langle R,\eta(c)\star F\rangle
&=c\boxtimes\langle R,F\rangle.
\end{aligned}
$$

**Proof.** L-NS-4 and L-IDX-3 show that embedded scalar multiplication changes
each coefficient from $R_\alpha$ to $c\boxtimes R_\alpha$. L-OS-12 then
gives

$$
(c\boxtimes R_\alpha)^\dagger
=c^\dagger\boxtimes R_\alpha^\dagger.
$$

Factor $c^\dagger$ from the finite fold using L-OS-5 and L-OS-3 to obtain
the first identity. The second follows by factoring $c$ from

$$
R_\alpha^\dagger\boxtimes(c\boxtimes F_\alpha).
$$

$\square$

## 8. Dependency conclusion

All core obligations in Sections 2 through 7 have been derived without camera
dependencies. Therefore:

1. the oriented-scalar carrier is a commutative field;
2. the finite multi-index carrier is a commutative monoid;
3. finite native states form a commutative ring with identity;
4. orientation and index steps preserve their separation invariants;
5. states represent additive and multiplicative actions on the same carrier;
6. the coefficient pairing has the stated finite algebraic laws.

## 9. What this proof does not establish

This proof does not establish:

- novelty of the algebra;
- equivalence to complex numbers, matrices, polar coordinates, PSD matrices, or
  the cone;
- full self-observation;
- an analytic completion or native calculus;
- any result about primes or the Riemann hypothesis;
- any compression, runtime, physical, or machine-learning advantage;
- machine-checked correctness.

Those remain separate obligations in the governing research order.
