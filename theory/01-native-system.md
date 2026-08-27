<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Space 1.0: Core Definition

**Document status:** Native Space 1.0 definition
**Research stage:** 1 -- Define the native system  
**Proof status:** Core laws proved in `../proofs/01-core-algebra.md`; this file
remains the authoritative definition source

## 1. Design requirement

The core must give precise, typed meanings to:

$$
\mathrm{ADD},\qquad
\mathrm{MULTIPLY},\qquad
\mathrm{ORIENT},\qquad
\mathrm{INDEX}.
$$

It must keep these three facts distinct:

1. a coefficient has scale and orientation;
2. a primitive direction has an identity or birth order;
3. a state may take a repeated number of steps along that direction.

The core contains only finite algebraic objects. Infinite sums, limits,
calculus, transforms, cone geometry, primes, and application-specific semantics
are outside 1.0 and may enter only through later definitions.

### D-PATTERN-ONTOLOGY-1 -- observations instead of ranges [Definition]

Native Space has no range carrier and no infinity element. A **pattern** is an
intensional rule

$$
P:K\longrightarrow X
$$

whose input selects one observation. The notation $P(k)$ denotes that one
observation, never a stored prefix $P(0),\ldots,P(k)$. Every core observation
and every core state is finite.

A pattern is called **unbounded** or **infinite** only when its rule is defined
for every finite constructed input and has no final observation. Here
“infinite” is a property of the rule's domain, not a value in $X$, a native
number, or a materialized native state. Quantification over all finite inputs
belongs to the surrounding proof language.

### D-SELF-PATTERN-1 -- finite self-modeled recurrence [Definition]

For a native seed $S$ and a native-state-induced action
$T_F\in\{\mathsf A_F,\mathsf M_F\}$ from D-NS-11, define

$$
P_{S,F}(0):=S,
\qquad
P_{S,F}(k+1):=T_F(P_{S,F}(k)).
$$

The finite pair $(S,F)$ represents the seed and step rule in the same native
carrier on which the step acts. T-SELF-PATTERN-1 proves that every finite
observation is native and that finite step composition is itself represented
by a native state. This theorem concerns the two internal actions. The generic
source language separately represents arbitrary function self-reference as a
finite pattern graph under D-FLANG-2 and T-FLANG-SELF-1.

Native Space therefore does not add an infinity element when a pattern has no
last observation. Repeated classical unfolding is a camera presentation of the
finite seed, rule, and self-reference. Establishing such a rule for a specific
classical object is still part of that object's camera proof.

## 2. Substrate and notation

### A-RF-1 -- Real-field substrate [Axiom]

Assume a real field $\mathbb{R}$ with its usual operations $+$, $\cdot$,
elements $0$ and $1$, and the field laws.

### A-N-1 -- Natural-number substrate [Axiom]

Assume the nonnegative integers $\mathbb{N}_0$, their usual addition and
multiplication, and the semiring laws for finite counting. The positive integers
are $\mathbb{N}_{>0}=\mathbb{N}_0\setminus\{0\}$.

These are the only numerical substrates assumed by the core. Ordinary
finite/countable set and function notation belongs to the surrounding logical
language rather than to Native Space.

**Verification profiles.** T-CLASSICAL-MODEL-1 is the default practical
soundness proof: it models the complete finite four-operation algebra in one
classical finite monoid algebra. The optional strict profile instead constructs
the exact rational slice from A-IND-1 in D-QOS-1 through T-QCORE-1. A-RF-1 and
A-N-1 remain the declared substrates of this broader real-valued 1.0 paper
presentation; they are not hidden as native theorems.

### Notational separation

Substrate operations retain ordinary symbols. Native operations use distinct
symbols so a proof cannot silently move between layers:

| Layer | Addition-like operation | Multiplication-like operation |
|---|---:|---:|
| real substrate | $+$ | $\cdot$ |
| oriented scalar | $\boxplus$ | $\boxtimes$ |
| index | -- | $\oplus_I$ |
| native state | $\oplus$ | $\star$ |

### Finite-fold convention

Until associativity and commutativity are proved, a displayed finite aggregate
$\mathop{\boxplus}$ means a left-associated fold in canonical order. Multi-
indices use lexicographic order by the least label position at which they
differ; pairs of multi-indices use the corresponding pairwise lexicographic
order. The empty fold is $\mathbf0$.

Stage 3 will prove that the resulting value is independent of order and
parenthesization. This convention makes every Stage 1 definition unambiguous
without assuming that theorem in advance.

## 3. Oriented scalars

### D-OS-1 -- Carrier [Definition]

The oriented-scalar carrier is

$$
\mathbb{O}:=\mathbb{R}\times\mathbb{R}.
$$

An element is written $z=(x,y)$. Equality is coordinatewise:

$$
(x,y)=(u,v)\quad\Longleftrightarrow\quad x=u\ \text{and}\ y=v.
$$

No complex-number interpretation is assumed.

### D-OS-2 -- Scalar ADD [Definition]

For $z=(x,y)$ and $w=(u,v)$, define

$$
z\boxplus w:=(x+u,\ y+v).
$$

Its candidate identity and additive opposite are

$$
\mathbf{0}:=(0,0),
\qquad
\boxminus(x,y):=(-x,-y).
$$

### D-OS-3 -- Scalar MULTIPLY [Definition]

Define

$$
(x,y)\boxtimes(u,v)
:=
(x u-y v,\ x v+y u).
$$

Its candidate identity is

$$
\mathbf{1}:=(1,0).
$$

The operation is defined directly by real-field arithmetic. Its familiar
interpretations, if any, are Stage 2 mapping questions.

### D-OS-4 -- Distinguished orientations [Definition]

Define

$$
\mathbf{J}:=(0,1),
\qquad
\mathbf{-1}:=(-1,0),
\qquad
\mathbf{-J}:=(0,-1).
$$

These are named native states. Statements such as
$\mathbf{J}\boxtimes\mathbf{J}=\mathbf{-1}$ are proof obligations, not
additional definitions.

### D-RJ-1 -- native number-line and quarter-axis embeddings [Definition]

Define the native line embedding and its quarter-oriented companion by

$$
\lambda(a):=(a,0),
\qquad
\jmath(b):=\lambda(b)\boxtimes\mathbf J.
$$

The candidate native number line and quarter axis are

$$
\mathbb{R}_N:=\lambda(\mathbb{R}),
\qquad
\mathbf J\mathbb{R}_N:=\jmath(\mathbb{R}).
$$

At this point these are definitions inside $\mathbb{O}$. Closure of the
line, the square law for $\mathbf J$, and unique decomposition into the two
axes are proof obligations. They are not imported from conventional complex
numbers.

### T-RJ-1 -- the native number line and J-system exist internally [Proved]

The proof in `../proofs/01-core-algebra.md` establishes that $\lambda$ is an
injective field embedding, that

$$
\mathbf J\boxtimes\mathbf J=\lambda(-1),
$$

and that every $z\in\mathbb{O}$ has a unique decomposition

$$
z=\lambda(a)\boxplus\jmath(b).
$$

This is the internal existence theorem for the signed number line plus the
quarter-oriented $J$-system. The later complex camera recognizes this
already-proved algebra; it does not create it.

### D-OS-5 -- Conjugate and squared size [Definition]

Define the maps $(\cdot)^\dagger:\mathbb{O}\to\mathbb{O}$ and
$\nu:\mathbb{O}\to\mathbb{R}$ by

$$
(x,y)^{\dagger}:=(x,-y)
$$

and

$$
\nu(x,y):=x^2+y^2.
$$

The codomain of $\nu$ is the nonnegative part of $\mathbb{R}$. Calling
$\nu$ a norm, or claiming it is multiplicative, requires later proof.

### D-OS-6 -- Candidate multiplicative inverse [Definition]

Let $\mathbb{O}^\times:=\{z\in\mathbb{O}:\nu(z)\neq0\}$. Define the partial
inverse candidate $(\cdot)^{[-1]}:\mathbb{O}^\times\to\mathbb{O}$. For
$z=(x,y)\in\mathbb{O}^\times$, set

$$
z^{[-1]}:=
\left(
\frac{x}{\nu(z)},
-\frac{y}{\nu(z)}
\right).
$$

This is a partial operation. It is undefined when $\nu(z)=0$. The claim that
it is an inverse under $\boxtimes$ is a Stage 3 proof obligation.

### D-OS-7 -- Quarter-orientation action [Definition]

For $n\in\mathbb{N}_0$, recursively define

$$
\mathbf{J}^{\boxtimes 0}:=\mathbf{1},
\qquad
\mathbf{J}^{\boxtimes(n+1)}
:=
\mathbf{J}\boxtimes\mathbf{J}^{\boxtimes n}.
$$

Let $\mathbb{Z}_4=\{0,1,2,3\}$ with addition modulo four. For the canonical
representative $r\in\{0,1,2,3\}$, define

$$
\mathrm{orient}_r(z)
:=
\mathbf{J}^{\boxtimes r}\boxtimes z.
$$

The action-composition law modulo four must be established from the
orientation-cycle proof; it is not assumed from a complex camera.

### D-REL-ORIENT-1 -- relative native orientation [Definition]

Write $O_r:=\mathbf J^{\boxtimes r}$ for $r\in\mathbb{Z}_4$. Define the
orientation of target $O_s$ from reference perspective $O_r$ by

$$
\mathrm{Rel}(r,s)
:=
O_r^{[-1]}\boxtimes O_s.
$$

Equivalently, after the inverse and cycle laws are proved,

$$
\mathrm{Rel}(r,s)=O_{s-r\bmod4}.
$$

For any oriented scalar $z$, define its readout from perspective $r$ as

$$
\mathrm{View}_r(z):=O_r^{[-1]}\boxtimes z.
$$

These are native self-relations: reference, target, relative orientation, and
the perspective action all live in the same oriented-scalar algebra. Their
composition, frame-invariance, and zero-preservation are proof obligations.

## 4. Primitive labels and multi-indices

### D-IDX-1 -- Primitive-label set [Definition]

Let

$$
\mathcal A=\{a_k:k\in K_{\mathcal A}\},
$$

be a finite or countably infinite ordered set of formal primitive labels, where
$K_{\mathcal A}=\{1,\ldots,m\}$ in the finite case and
$K_{\mathcal A}=\mathbb{N}_{>0}$ in the countably infinite case. The labels
carry no scalar coefficient or externally assigned magnitude and are not
primes.

### D-IDX-2 -- Birth order [Definition]

Define the label-order bijection

$$
\mathrm{idx}:\mathcal A\to K_{\mathcal A},
\qquad
\mathrm{idx}(a_k):=k.
$$

This identifies a primitive direction. It does not say how many times that
direction occurs in a state and does not assign an orientation to it.

### D-IDX-3 -- Multi-index carrier [Definition]

Define

$$
\mathbb{I}_{\mathcal A}
:=
\left\{
\alpha:\mathcal A\to\mathbb{N}_0
\;\middle|\;
\mathrm{supp}(\alpha)\text{ is finite}
\right\},
$$

where

$$
\mathrm{supp}(\alpha)
:=\{a\in\mathcal A:\alpha(a)\neq0\}.
$$

Equality is pointwise.

### D-IDX-4 -- Zero multi-index [Definition]

Define $\mathbf{0}_I\in\mathbb{I}_{\mathcal A}$ by

$$
\mathbf{0}_I(a)=0
\quad\text{for every }a\in\mathcal A.
$$

### D-IDX-5 -- Elementary multi-index [Definition]

For each $k\in K_{\mathcal A}$, define
$\varepsilon_k\in\mathbb{I}_{\mathcal A}$ by

$$
\varepsilon_k(a_j)
\mathrel{=}
\begin{cases}
1,&j=k,\\
0,&j\neq k.
\end{cases}
$$

### D-IDX-6 -- Index composition [Definition]

For $\alpha,\beta\in\mathbb{I}_{\mathcal A}$, define pointwise

$$
(\alpha\oplus_I\beta)(a)
:=\alpha(a)+\beta(a).
$$

Finite support makes the result a candidate member of
$\mathbb{I}_{\mathcal A}$; closure is proved in Stage 3.

### D-IDX-7 -- Directional and total depth [Definition]

For each $k\in K_{\mathcal A}$, define
$d_k:\mathbb{I}_{\mathcal A}\to\mathbb{N}_0$ by

$$
d_k(\alpha):=\alpha(a_k)
$$

and define $\mathrm{depth}:\mathbb{I}_{\mathcal A}\to\mathbb{N}_0$ by

$$
\mathrm{depth}(\alpha)
:=\sum_{a\in\mathrm{supp}(\alpha)}\alpha(a).
$$

Thus $\mathrm{idx}(a_k)=k$ and
$d_k(\alpha)=\alpha(a_k)$ are different typed quantities by construction.

### D-IDX-8 -- Formal basis token [Definition]

For each $\alpha\in\mathbb{I}_{\mathcal A}$, write $[\alpha]$ for a
formal basis token. It has no numerical value. The notation

$$
[\alpha][\beta]:=[\alpha\oplus_I\beta]
$$

is shorthand for index composition and not substrate multiplication.

## 5. Native states

### D-NS-1 -- State carrier [Definition]

Define Native Space over $\mathcal A$ as

$$
\mathcal N_{\mathcal A}
:=
\left\{
F:\mathbb{I}_{\mathcal A}\to\mathbb{O}
\;\middle|\;
\mathrm{supp}(F)\text{ is finite}
\right\},
$$

where

$$
\mathrm{supp}(F)
:=\{\alpha\in\mathbb{I}_{\mathcal A}:F(\alpha)\neq\mathbf0\}.
$$

The finite formal-sum notation

$$
F=\sum_{\alpha\in\mathrm{supp}(F)}F_\alpha[\alpha]
$$

means exactly the finite-support function above. It introduces no infinite
series or convergence assumption.

### D-NS-2 -- State equality [Definition]

For $F,G\in\mathcal N_{\mathcal A}$, define

$$
F=G
\quad\Longleftrightarrow\quad
F_\alpha=G_\alpha
\text{ for every }\alpha\in\mathbb{I}_{\mathcal A}.
$$

Two different formal expressions therefore denote the same state when all
collected coefficients agree.

### D-NS-3 -- State zero and one [Definition]

Define the zero state $\mathsf{0}\in\mathcal N_{\mathcal A}$ by

$$
\mathsf{0}_\alpha:=\mathbf0
\quad\text{for every }\alpha.
$$

Define the candidate multiplicative identity $\mathsf{1}$ by

$$
\mathsf{1}_{\mathbf0_I}:=\mathbf1,
\qquad
\mathsf{1}_\alpha:=\mathbf0
\text{ for }\alpha\neq\mathbf0_I.
$$

### D-NS-4 -- Coefficient and scalar embedding [Definition]

For $c\in\mathbb{O}$ and $\alpha\in\mathbb{I}_{\mathcal A}$, define
$c[\alpha]\in\mathcal N_{\mathcal A}$ as the state with coefficient $c$
at $\alpha$ and $\mathbf0$ elsewhere.

The scalar embedding $\eta:\mathbb{O}\to\mathcal N_{\mathcal A}$ is

$$
\eta(c):=c[\mathbf0_I].
$$

### D-NS-5 -- ADD [Definition]

For $F,G\in\mathcal N_{\mathcal A}$, define

$$
(F\oplus G)_\alpha
:=F_\alpha\boxplus G_\alpha.
$$

This combines alternatives occupying the same formal index.

### D-NS-6 -- Additive opposite [Definition]

Define

$$
(\ominus F)_\alpha:=\boxminus F_\alpha.
$$

### D-NS-7 -- MULTIPLY [Definition]

For $F,G\in\mathcal N_{\mathcal A}$, define the finite convolution

$$
(F\star G)_\gamma
:=
\mathop{\boxplus}_{\substack{
\alpha\in\mathrm{supp}(F),\\
\beta\in\mathrm{supp}(G),\\
\alpha\oplus_I\beta=\gamma
}}
F_\alpha\boxtimes G_\beta.
$$

An empty coefficient sum is $\mathbf0$. Because both supports are finite,
only finitely many pairs occur. Formal closure and independence from summation
order are Stage 3 proof obligations.

For single terms, the intended reduction is

$$
c[\alpha]\star d[\beta]
\mathrel{=}
(c\boxtimes d)[\alpha\oplus_I\beta].
$$

This equation follows directly from the definition but will still be recorded
as a lemma so later proofs can cite it.

### D-NS-8 -- ORIENT [Definition]

For $r\in\mathbb{Z}_4$, define

$$
(\mathrm{ORIENT}_r F)_\alpha
:=
\mathrm{orient}_r(F_\alpha).
$$

Equivalently, once the relevant multiplication lemma is proved,
$\mathrm{ORIENT}_r$ should be multiplication by the state
$\eta(\mathbf J^{\boxtimes r})$. The equivalence is not assumed here.

### D-NS-9 -- Primitive generator state [Definition]

For each $k\in K_{\mathcal A}$, define

$$
\mathsf{X}_k:=\mathbf1[\varepsilon_k].
$$

This is a formal primitive direction. It is not a prime and has no externally
assigned magnitude.

### D-NS-10 -- INDEX [Definition]

For each $k\in K_{\mathcal A}$, define the index-step operation

$$
\mathrm{INDEX}_k(F):=\mathsf X_k\star F.
$$

The expected effect is to move each term from $\alpha$ to
$\varepsilon_k\oplus_I\alpha$ without changing its coefficient. That
statement is a Stage 3 lemma.

### D-NS-11 -- Two state-induced actions [Definition]

Every $F\in\mathcal N_{\mathcal A}$ defines two transformations on the same
carrier:

$$
\mathsf A_F(G):=F\oplus G,
\qquad
\mathsf M_F(G):=F\star G.
$$

The expected composition laws

$$
\mathsf A_F\circ\mathsf A_G=\mathsf A_{F\oplus G},
\qquad
\mathsf M_F\circ\mathsf M_G=\mathsf M_{F\star G}
$$

are the Stage 3 self-action target. They are not granted by definition.

### D-NS-12 -- Coefficient pairing candidate [Definition]

Define the oriented-scalar readout candidate
$\langle\cdot,\cdot\rangle:\mathcal N_{\mathcal A}\times
\mathcal N_{\mathcal A}\to\mathbb{O}$ by

$$
\langle R,F\rangle
:=
\mathop{\boxplus}_{\alpha\in
\mathrm{supp}(R)\cap\mathrm{supp}(F)}
R_\alpha^{\dagger}\boxtimes F_\alpha.
$$

This is included only to make the strongest self-representation question
precise: can a camera be selected by a state $R$ from the same carrier?
Its algebraic properties belong to Stage 3, and whether it is the correct or a
sufficient camera belongs to Stage 2. No full self-observation claim is made.

## 6. Public operation signatures

The four named operations have these exact signatures:

$$
\begin{aligned}
\mathrm{ADD}&:\mathcal N_{\mathcal A}\times
\mathcal N_{\mathcal A}\to\mathcal N_{\mathcal A},\\
\mathrm{MULTIPLY}&:\mathcal N_{\mathcal A}\times
\mathcal N_{\mathcal A}\to\mathcal N_{\mathcal A},\\
\mathrm{ORIENT}&:\mathbb{Z}_4\times
\mathcal N_{\mathcal A}\to\mathcal N_{\mathcal A},\\
\mathrm{INDEX}&:K_{\mathcal A}\times
\mathcal N_{\mathcal A}\to\mathcal N_{\mathcal A}.
\end{aligned}
$$

In formulas these are $\oplus$, $\star$,
$\mathrm{ORIENT}_r$, and $\mathrm{INDEX}_k$, respectively.

## 7. Domains, inverses, and singular cases

- `ADD`, `MULTIPLY`, `ORIENT`, and `INDEX` are intended as total operations on
  their stated domains. Closure remains to be proved.
- Every state has a defined additive opposite candidate $\ominus F$.
- Only oriented scalars with $\nu(z)\neq0$ have the inverse candidate from
  D-OS-6.
- A general native state is **not** assigned a multiplicative inverse in 1.0.
  Nonzero does not imply invertible.
- The zero state is expected to absorb multiplication, but that is a lemma.
- No division by a native state is part of the public core.
- No infinite-support state is part of the carrier. Analytic completion requires
  a separate topology, convergence rule, and versioned extension.

## 8. Separation invariants

The following are requirements to be proved and preserved:

1. **Index/orientation separation:** applying `ORIENT` changes coefficients and
   does not change multi-indices.
2. **Index/depth separation:** $\mathrm{idx}(a_k)=k$ identifies a
   direction; $d_k(\alpha)$ counts uses of it.
3. **Index-step behavior:** applying `INDEX(k)` increases only directional depth
   $k$ by one.
4. **Finite closure:** every public operation on finite states produces a finite
   state.
5. **State/action unity:** at least additive and multiplicative actions are
   represented by states on the same carrier.

These are not informal intentions. They correspond to named proof obligations
in `proofs/00-dependency-ledger.md`.

## 9. Explicit exclusions

The following do not belong to the Stage 1 core:

- complex numbers or the symbol $i$ as an axiom;
- angle, phase, trigonometric, exponential, or logarithmic functions;
- the quadratic / PSD camera and 3D cone;
- Fourier, Mellin, Laplace, or other transforms;
- derivatives, integrals, or infinite series;
- primes, factorization, pattern birth index $q(p(k))=k$, Möbius functions,
  Euler products, zeta functions, or RH;
- probability, physical units, learned parameters, or computational cost;
- claims of novelty, compression, speed, or scientific impact.

These exclusions prevent later interpretations from becoming circular premises.

## 10. Stage 1 acceptance checklist

- [x] Every carrier has a type and equality rule.
- [x] Every public operation has a domain and codomain.
- [x] Identity, zero, opposite, inverse candidate, and singular domains are
  explicit.
- [x] Primitive label, birth order, directional depth, and total depth are
  distinct.
- [x] Orientation is defined without importing complex arithmetic.
- [x] Infinite and analytic constructions are excluded.
- [x] Full self-observation is posed as a question, not claimed.
- [x] All definitions have passed an initial type and dependency consistency
  review.
- [x] Stage 3 has proved the required laws from A-RF-1 and A-N-1 in a complete
  paper proof.

The definitions and paper proof establish the finite algebraic core. They do not
constitute a machine-checked formalization or establish any later camera,
analytic, prime, language, or application claim.
