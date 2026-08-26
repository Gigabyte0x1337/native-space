<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Prime Pattern and Integer Encoding

Native Space contains one generative prime pattern. A positive $k$ selects an
observation of that pattern; it does not construct a native range, list, or
collection of independent primes. Conventional prime values enter through a
camera after the generic four-operation algebra is defined.

The same layer models positive-integer multiplication as native monomial
MULTIPLY. Ordinary integer addition belongs to the exact number-line camera,
not to ADD of INDEX monomials.

## Conventional substrate and value camera

### A-INT-1 -- integer substrate [Axiom]

Use the ordinary ordered integral domain $\mathbb{Z}$, induction,
well-ordering, and the division algorithm. Unique prime factorization is not
included; it is reconstructed in the proof layer.

### D-PRIME-1 -- conventional prime predicate [Definition]

A positive integer $n$ is prime when $n>1$ and its only positive divisors
are $1$ and $n$.

### D-PRIME-2 -- prime-value observation camera [Definition]

After prime infinitude is proved, define

$$
p(1):=2,
\qquad
p(k+1):=\min\{n>p(k):n\text{ satisfies D-PRIME-1}\}.
$$

Well-ordering and infinitude make the next observation defined. This recursive
camera returns one conventional value for an input $k$; it does not add a
native prime range.

### D-PRIME-3 -- pattern birth index [Definition]

$$
q(p(k)):=k.
$$

Thus $q$ is the inverse index on prime-value observations. Index $k$,
value $p(k)$, logarithmic weight, orientation, and multiplicative depth are
different typed quantities.

## One generative native pattern

### D-BIRTH-1 -- wrapped orientation projection [Definition]

$$
\mathrm{BirthOrient}(k)
:=\mathbf J^{\boxtimes k}
=\mathrm{ORIENT}_{k\bmod4}(\mathbf1).
$$

Its classical coefficient camera is $i^k$.

### D-PRIME-PATTERN-1 -- combined pattern observation [Definition]

$$
\mathrm{PrimePattern}(k)
:=
\mathrm{ORIENT}_k\!\left(\mathrm{INDEX}_k(\mathsf1)\right)
=\mathbf J^{\boxtimes k}[\varepsilon_k].
$$

This one function combines the two projections

$$
\mathrm{PrimeBirth}(k)=\mathrm{INDEX}_k(\mathsf1),
\qquad
\mathrm{BirthOrient}(k)=\mathrm{ORIENT}_k(\mathsf1).
$$

It has observation identity $k$, orientation $k\bmod4$, and
multiplicative depth one. The orientation repeats after four turns while the
INDEX identity remains visible. Applying $\mathrm{INDEX}_k^d$ changes
depth without changing observation identity or wrapped orientation.

Native Space 1.0 exposes these views as
`prime_pattern(k)`, `prime_birth(k)`, `birth_orient(k)`, and
`prime_power(k,d)`. Each lowers completely to ORIENT and INDEX.

### T-PRIME-PATTERN-1 -- pattern law [Proved]

The proof in `../proofs/09-prime-factorization.md` establishes

$$
\mathrm{PrimePattern}(k)
=\mathbf J^{\boxtimes k}[\varepsilon_k],
$$

$$
\mathrm{BirthOrient}(k+4)=\mathrm{BirthOrient}(k),
$$

while $\varepsilon_{k+4}\neq\varepsilon_k$. It also proves that increasing
multiplicative depth leaves the first two typed quantities unchanged.

## Finite prime-count camera

### D-PI-1 -- recurrence [Definition]

$$
\pi_N(0):=0,
\qquad
\pi_N(x+1):=\pi_N(x)+
\begin{cases}
1,&x+1\text{ satisfies D-PRIME-1},\\
0,&\text{otherwise}.
\end{cases}
$$

This finite recurrence observes primality successively through input $x$.
Its implementation may iterate to evaluate the recurrence; iteration is an
execution strategy, not a native prime range. The proof obligations are

$$
\pi_N(p(k))=q(p(k))=k
$$

and agreement of the finite implementation with the recurrence.

## Multiplicative depth and integer encoding

### D-VAL-1 -- valuation [Definition]

For positive $n$ and a pattern observation $k$, let

$$
v_k(n):=\max\{e\in\mathbb{N}_0:p(k)^e\mid n\}.
$$

### D-ENC-1 -- native integer monomial [Definition]

Define $\alpha_n(a_k):=v_k(n)$. After finite support is proved,

$$
\mathrm{Enc}(n):=\mathbf1[\alpha_n].
$$

### D-DEC-1 -- value decoding [Definition]

For finite-support $\alpha$,

$$
\mathrm{Dec}(\alpha)
:=\prod_{a_k\in\mathrm{supp}(\alpha)}
p(k)^{d_k(\alpha)}.
$$

The product is finite because support is finite. The proof layer establishes
Encode/Decode inversion and

$$
\mathrm{Enc}(mn)
=\mathrm{Enc}(m)\star\mathrm{Enc}(n).
$$

### D-NPRIME-1 -- intrinsic native multiplicative atom [Definition]

Inside the coefficient-one monomial carrier, $P\neq\mathsf1$ is a native
prime atom when $P=A\star B$ implies $A=\mathsf1$ or $B=\mathsf1$.
T-NPRIME-1 proves these atoms are exactly

$$
\mathsf X_k=\mathbf1[\varepsilon_k].
$$

This is multiplicative structure, not a range of prime objects.

## Exact number-line camera

### C-PRIME-LINE-1 -- classical value view [Definition]

Let

$$
\mathbb{Z}_N:=\lambda(\mathbb{Z}),
\qquad
\mathbb{N}_N^+:=\lambda(\mathbb{N}_{>0}).
$$

Define

$$
V(\mathbf1[\alpha])
:=\lambda(\mathrm{Dec}(\alpha)).
$$

The proof layer establishes

$$
V(\mathsf X_k)=\lambda(p(k)),
$$

and proves that intrinsic native atoms, number-line prime values, and the
conventional prime predicate are exact cameras of the same multiplicative
observation. The number-line camera preserves ordinary integer ADD and
MULTIPLY; the INDEX monomial camera preserves positive multiplication,
divisibility, powers, and finite factorization.

## Flat and quadratic pattern cameras

Write

$$
(u_k,v_k):=\kappa(\mathrm{BirthOrient}(k)).
$$

### C-BIRTH-HELIX-1 -- flat indexed pattern camera [Definition]

$$
H_q(k):=(u_k,v_k,k)
=\left(\cos\frac{\pi k}{2},\sin\frac{\pi k}{2},k\right).
$$

This is the default flat 3D observation: orientation plus retained INDEX.

### C-PRIME-HELIX-1 -- prime-value camera [Definition]

$$
H_p(k):=(u_k,v_k,p(k)).
$$

It changes only the axial camera from observation index to conventional prime
value. The later logarithmic camera changes that axis to $\log p(k)$.

### C-PRIME-BRAID-1 -- exact three-track cable [Definition]

For three distinct track labels $r\in\{0,1,2\}$, choose distinct native
INDEX symbols $a_{r,k}$ and define the three source observations

$$
\mathrm{Track}_r(k)
:=\mathbf J^{\boxtimes(k+r)}[\varepsilon_{r,k}].
$$

The cable camera retains the common birth coordinate $k$, reads the signed
coefficient axes, and suppresses only the explicit track label:

$$
B_r(k):=
\left(
\cos\frac{\pi(k+r)}2,
\sin\frac{\pi(k+r)}2,
k
\right).
$$

Each coefficient pair is the conventional camera of the native quarter-turn
state $\mathbf J^{\boxtimes(k+r)}$. The third coordinate retains the common
prime birth identity. The track label is not a fourth native operation; the
three observations are distinguished by ordinary INDEX before the camera is
applied.

Define the classical flattening camera

$$
C_B(x,y,k):=(x,k).
$$

This camera deliberately removes the second signed orientation coordinate.
T-PRIME-BRAID-1 proves the three tracks are disjoint and have exactly three
endpoints on every transverse cut. T-PRIME-BRAID-PROJ-1 proves that $C_B$ is
noninjective on the cable and identifies an exact projected crossing.

### C-PRIME-CONE-1 -- optional quadratic camera [Definition]

$$
H_Q(k):=Q(\mathrm{BirthOrient}(k)).
$$

The cone identifies opposite orientations, so it cannot replace the signed
flat camera. T-PRIME-HELIX-1 proves the flat camera is injective through its
INDEX coordinate and classifies the cone fiber. These are observations of one
pattern law, not stored prime ranges or distribution theorems.

## Required separation

| Quantity | Meaning |
|---|---|
| $p(k)$ | conventional prime-value camera |
| $q(p(k))=k$ | pattern observation identity |
| $d_k(\alpha_n)$ | multiplicative depth |
| $\mathbf J^{\boxtimes k}$ | wrapped observation orientation |

Equal integer codomains do not make these operations interchangeable.

## Boundaries

- No native prime range, sequence object, or prime-by-prime execution is
  defined.
- The prime-value recurrence and conventional sums/products are camera-side
  constructions.
- Wrapped orientation alone is not an identifier; the flat camera retains
  INDEX.
- The pattern law does not predict prime values or prove a distribution
  theorem.
- The finite layer does not justify analytic zeta cameras. Formal completion
  and analytic convergence remain separate stages.

The arithmetic, factorization, pattern, and camera obligations are proved in
`../proofs/09-prime-factorization.md`.
