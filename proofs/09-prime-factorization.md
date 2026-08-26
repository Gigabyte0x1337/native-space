<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Prime Factorization and Native Integer Encoding

## Dependencies

The conventional factorization proof uses only A-INT-1 and D-PRIME-1. The
native encoding proof then uses the finite index/native ring laws and the
definitions in `../theory/07-native-prime-system.md`.

### L-BEZ-1 -- Bézout identity for integer gcd [Proved]

**Claim.** For integers $a,b$, not both zero, there exist $x,y\in\mathbb{Z}$
such that

$$
ax+by=\gcd(a,b).
$$

**Proof.** Consider the nonempty set of positive integer combinations
$ax+by$ and let $d$ be its least member by A-INT-1 well-ordering. Divide
$a=qd+r$ with $0\leq r<d$. Since both $a$ and $d$ are integer
combinations of $a,b$, so is $r$. Minimality forces $r=0$, hence
$d\mid a$. The same argument gives $d\mid b$. Every common divisor of
$a,b$ divides every integer combination and therefore divides $d$. Thus
$d=\gcd(a,b)$, and its defining combination supplies $x,y$. $\square$

### L-EUCLID-1 -- Euclid's lemma [Proved]

**Claim.** If prime $p$ divides $ab$, then $p\mid a$ or $p\mid b$.

**Proof.** If $p\mid a$, the claim holds. Otherwise primality gives
$\gcd(p,a)=1$. L-BEZ-1 supplies $x,y\in\mathbb{Z}$ with

$$
xp+ya=1.
$$

Multiply by $b$. The prime $p$ divides $xpb$ and, by hypothesis,
$yab$, so it divides their sum $b$. $\square$

### T-FTA-1 -- fundamental theorem of arithmetic [Proved]

**Claim.** Every $n\in\mathbb{N}_{>0}$ is a finite product of primes, unique
up to factor order; $1$ is the empty product.

**Proof.** For existence, use strong induction. The claim is immediate for
$1$. Let $n>1$. If $n$ is prime, it is already a one-factor product. If
not, D-PRIME-1 supplies a positive divisor $a$ other than $1,n$. Writing
$b=n/a$ gives $n=ab$ with $1<a<n$ and $1<b<n$. Strong induction
factors both $a$ and $b$, and concatenating the finite products factors
$n$.

For uniqueness, suppose

$$
p_1\cdots p_r=q_1\cdots q_s.
$$

Euclid's lemma applied repeatedly shows $p_1\mid q_j$ for some $j$.
Because both are prime, $p_1=q_j$. Reorder the right product, cancel this
nonzero factor in the integer integral domain, and repeat. Induction leaves the
same multiset of prime factors on both sides. The empty product handles
$n=1$. $\square$

### T-PRIME-INF-1 -- the prime-value observation is always defined [Proved]

**Claim.** Values satisfying D-PRIME-1 are unbounded. Consequently the
recursive observation camera $p(k)$ in D-PRIME-2 is defined for every
positive input $k$.

**Proof.** If all primes were $p_1,\ldots,p_r$, the integer

$$
N=p_1\cdots p_r+1>1
$$

would have a prime factor by T-FTA-1. No listed $p_j$ divides $N$, because
division leaves remainder one. This is a contradiction. Well-ordering then
selects the least prime, followed recursively by the least larger prime,
giving the next value of the recursive observation camera. Induction on the
input therefore defines $p(k)$ for every positive $k$. No native range is
constructed. $\square$

### T-PI-1 -- prime count equals pattern observation index [Proved]

**Claim.** D-PI-1 is finite for every $x\in\mathbb{N}_0$, and

$$
\pi_N(p(k))=q(p(k))=k.
$$

More generally, with the camera convention $p(0):=1$,

$$
p(k)\leq x<p(k+1)
\quad\Longleftrightarrow\quad
\pi_N(x)=k.
$$

**Proof.** Only the finitely many integers from $2$ through $x$ can
contribute to D-PI-1. By D-PRIME-2, each new observation $p(k+1)$ is the
least larger value satisfying D-PRIME-1. Hence the recurrence increments
exactly once between $p(k)$ and $p(k+1)$, at $p(k+1)$. Induction gives
$\pi_N(p(k))=k$; D-PRIME-3 gives $q(p(k))=k$. The same recurrence proves
the displayed camera interval equivalence. This reasons about one recursive
pattern law, not a native collection of primes. $\square$

### L-VAL-1 -- finite support and valuation addition [Proved]

**Claim.** For every positive integer $n$, only finitely many
$v_k(n)$ are nonzero. For positive $m,n$,

$$
v_k(mn)=v_k(m)+v_k(n).
$$

**Proof.** T-FTA-1 expresses $n$ with finitely many prime factors. The
maximum in D-VAL-1 is exactly each prime's multiplicity in that unique list,
so all other valuations vanish. Concatenating the unique factor lists for
$m$ and $n$ adds the multiplicity in each observed direction; uniqueness identifies
that list with the factorization of $mn$. $\square$

### T-ENC-1 -- integer multiplication is native monomial multiplication [Proved]

**Claim.** Encoding and decoding are mutually inverse bijections between
$(\mathbb{N}_{>0},\cdot,1)$ and
$(\mathcal M_{\mathbb{P}},\star,\mathsf1)$, and

$$
\mathrm{Enc}(mn)
=\mathrm{Enc}(m)\star\mathrm{Enc}(n).
$$

**Proof.** L-VAL-1 proves that $\alpha_n$ has finite support, so Enc is
defined. T-FTA-1 gives

$$
n=\prod_k p(k)^{v_k(n)}
=\mathrm{Dec}(\alpha_n),
$$

so decoding after encoding is the identity. Conversely, the displayed finite
product defining Dec has valuation $d_k(\alpha)$ in each observation direction by unique
factorization. Hence $\alpha_{\mathrm{Dec}(\alpha)}=\alpha$, and
encoding after decoding is the identity on $\mathcal M_{\mathbb{P}}$.

L-VAL-1 also gives

$$
\alpha_{mn}=\alpha_m\oplus_I\alpha_n.
$$

L-NS-4 and the coefficient identity $\mathbf1\boxtimes\mathbf1=\mathbf1$
then give the multiplication law. The empty multi-index represents one, so
identities are preserved. $\square$

**Boundary.** This is a commutative-monoid isomorphism for multiplication. It
does not preserve ordinary integer addition as native ADD.

### T-NPRIME-1 -- native prime monomials are exactly classical primes [Proved]

**Claim.** The native prime monomials of D-NPRIME-1 are exactly
$\mathsf X_k=\mathbf1[\varepsilon_k]$. Moreover,

$$
\mathrm{Enc}(p(k))=\mathsf X_k,
\qquad
\mathrm{Dec}(\varepsilon_k)=p(k),
$$

and, for every positive integer $n$,

$$
n\text{ is classically prime}
\quad\Longleftrightarrow\quad
\mathrm{Enc}(n)\text{ is a native prime monomial}.
$$

**Proof.** Suppose

$$
\mathsf X_k=\mathbf1[\beta]\star\mathbf1[\gamma].
$$

L-NS-4 gives $\varepsilon_k=\beta\oplus_I\gamma$. Taking total depth and
using L-IDX-5 gives

$$
1=\mathrm{depth}(\beta)+\mathrm{depth}(\gamma).
$$

Both depths are nonnegative integers, so one is zero. A nonnegative
multi-index has total depth zero exactly when it is $\mathbf0_I$; its
monomial is therefore the unit $\mathsf1$. Hence every $\mathsf X_k$ is a
native prime monomial.

Conversely, let $\mathbf1[\alpha]\neq\mathsf1$ and suppose
$\mathrm{depth}(\alpha)\geq2$. Choose a direction $a_k$ with
$d_k(\alpha)>0$, and subtract one in that coordinate to obtain the
nonnegative finite multi-index $\alpha'$. Then

$$
\mathbf1[\alpha]=\mathsf X_k\star\mathbf1[\alpha'].
$$

Both factors are nonunits because their total depths are $1$ and
$\mathrm{depth}(\alpha)-1\geq1$. Thus a native prime monomial must
have total depth one. The only nonnegative multi-indices of total depth one are
the $\varepsilon_k$, proving the intrinsic classification.

For the prime-value observation $p(k)$, its valuation is one in direction
$k$ and zero in every distinct observation direction, so D-ENC-1 gives
$\mathrm{Enc}(p(k))=\mathsf X_k$; D-DEC-1 gives the inverse formula.
Finally T-ENC-1 is a multiplicative bijection, so the atom classification is
preserved and reflected. The atoms of $(\mathbb{N}_{>0},\cdot,1)$ are exactly
the integers satisfying D-PRIME-1: a nontrivial factorization supplies a
nontrivial divisor, and any nontrivial divisor supplies such a factorization.
This proves the last equivalence. $\square$

**Boundary.** This theorem is exact for positive-integer multiplication. A
classical prime written as $(p(k),0)$ is prime only relative to the integer
number-line subring, not relative to the whole oriented-scalar field. Ordinary
integer addition is also not preserved by the INDEX-monomial camera.

### T-PRIME-LINE-1 -- the classical number line and INDEX prime camera commute [Proved]

**Claim.** The embedding

$$
\lambda:\mathbb{Z}\longrightarrow\mathbb{Z}_N
$$

is a ring isomorphism onto its image. It restricts to a multiplicative-monoid
isomorphism from $\mathbb{N}_{>0}$ to $\mathbb{N}_N^+$, and

$$
p\text{ is classically prime}
\quad\Longleftrightarrow\quad
\lambda(p)\text{ is number-line prime}.
$$

The C-PRIME-LINE-1 camera is the multiplicative bijection

$$
V=\lambda\circ\mathrm{Dec},
$$

and the diagram

$$
\begin{array}{ccc}
\mathcal M_{\mathbb{P}} & \xrightarrow{\ V\ } & \mathbb{N}_N^+\\
\mathrm{Dec}\downarrow\phantom{\mathrm{Dec}}
&&\phantom{\lambda^{-1}}\downarrow\lambda^{-1}\\
\mathbb{N}_{>0} & \xrightarrow{\ \mathrm{id}\ } & \mathbb{N}_{>0}
\end{array}
$$

commutes. In particular,

$$
\mathsf X_k
\xmapsto{\ V\ }
\lambda(p(k)),
$$

so intrinsic native primes, number-line primes, and classical primes are three
exact views of the same indexed object.

**Proof.** T-RJ-1 proves that $\lambda$ is injective and preserves ADD,
MULTIPLY, zero, one, and additive inverses. Restricting its domain and codomain
therefore gives the stated ring and positive-monoid isomorphisms. Because
positive divisibility is defined entirely by multiplication and identity, the
positive-monoid isomorphism preserves and reflects divisors. D-PRIME-1 and the
C-PRIME-LINE-1 definition then give the number-line prime equivalence.

T-ENC-1 proves that Dec is a multiplicative bijection, so composing it with
$\lambda$ proves the corresponding statement for $V$ and makes the diagram
commute by definition. T-NPRIME-1 gives
$\mathrm{Dec}(\varepsilon_k)=p(k)$; applying $\lambda$ gives
$V(\mathsf X_k)=\lambda(p(k))$. $\square$

**Boundary.** The number-line camera matches classical integer ADD and
MULTIPLY. The INDEX camera matches classical positive-integer MULTIPLY and
factorization. There is no claim that state ADD of INDEX monomials directly
produces the factorized INDEX monomial of the classical sum.

### T-BIRTH-1 -- wrapped observation orientation and depth separation [Proved]

**Claim.** For a symbolic positive observation input $k$,

$$
\kappa(\mathrm{BirthOrient}(k))=i^k,
$$

and

$$
\mathrm{BirthOrient}(k+4)
=\mathrm{BirthOrient}(k).
$$

Birth index/orientation is independent of multiplicative depth.

**Proof.** D-BIRTH-1 and T-CX-2 send $\mathbf J$ to $i$ and preserve
MULTIPLY, proving the first formula. L-OS-10 gives
$\mathbf J^{\boxtimes4}=\mathbf1$, proving four-periodicity.

For a fixed observation $k$, q and BirthOrient depend only on $k$. In
contrast,
L-VAL-1 gives

$$
d_k(\alpha_{n p(k)})=d_k(\alpha_n)+1.
$$

Thus multiplying by another copy changes depth while leaving birth identity
and wrapped birth orientation unchanged. Also primes with indices congruent
modulo four share an orientation, so wrapped orientation is not a unique prime
identifier. $\square$

### T-PRIME-PATTERN-1 -- one pattern combines identity and orientation [Proved]

**Claim.** For a symbolic positive observation input $k$,

$$
\mathrm{PrimePattern}(k)
=\mathbf J^{\boxtimes k}[\varepsilon_k].
$$

Its orientation is four-periodic, its INDEX identity is not, and its
multiplicative depth is one independently of both.

**Proof.** D-PRIME-PATTERN-1 applies `INDEX(k)` to the unit and then
`ORIENT(k)`. D-NS-10 places the unit coefficient at $\varepsilon_k$, while
L-SEP-3 rotates that coefficient without changing its INDEX location. D-BIRTH-1
therefore gives the displayed formula. L-OS-10 gives
$\mathbf J^{\boxtimes(k+4)}=\mathbf J^{\boxtimes k}$, while uniqueness of
finite multi-indices gives $\varepsilon_{k+4}\neq\varepsilon_k$. Finally
L-IDX-5 gives total depth one. Reapplying `INDEX(k)` changes only that depth;
it does not alter the direction label or the wrapped observation orientation.
$\square$

### T-PRIME-HELIX-1 -- flat prime cameras have helix form [Proved]

**Claim.** C-BIRTH-HELIX-1 is an injective sampling of a circular helix,
C-PRIME-HELIX-1 is an injective monotone axial deformation of it, and the
quadratic orientation camera identifies birth indices differing by two modulo
four.

**Proof.** T-BIRTH-1 gives

$$
(u_k,v_k)=i^k
=\left(\cos\frac{\pi k}{2},\sin\frac{\pi k}{2}\right)
$$

under the conventional coordinate camera. Therefore

$$
H_q(k)
=\left(\cos\frac{\pi k}{2},\sin\frac{\pi k}{2},k\right),
$$

which is the circular helix
$(\cos\theta,\sin\theta,2\theta/\pi)$ sampled at
$\theta=\pi k/2$. If $H_q(j)=H_q(k)$, equality of the third
coordinate gives $j=k$, so $H_q$ is injective.

For $H_p$, equality of third coordinates gives $p(j)=p(k)$, and the strict
recurrence in D-PRIME-2 then gives $j=k$. Its axial coordinate is strictly
increasing, so it is a monotone axial deformation of the indexed pattern
camera.

Finally, T-CONE-1 gives $Q(z)=Q(-z)$. L-OS-10 gives
$\mathbf J^{\boxtimes(k+2)}=\boxminus\mathbf J^{\boxtimes k}$, hence

$$
H_Q(k+2)=H_Q(k).
$$

The cone therefore retains the quadratic orientation axis but loses the sign
along it. $\square$

**Boundary.** These are exact camera and fiber statements. They neither insert
birth orientation into arithmetic coefficients nor establish a theorem about
prime gaps, zeta zeros, or computational complexity.

### T-PRIME-BRAID-1 -- the three quarter-turn tracks form a disjoint cable [Proved]

**Claim.** The three tracks in C-PRIME-BRAID-1 are pairwise disjoint. At every
fixed birth coordinate $k$, their pairwise distances are
$\sqrt{2},\sqrt{2},2$. Consequently every transverse cut has exactly three
distinct strand endpoints.

**Proof.** Fix $k$ and write $a=\pi k/2$. For labels $r\neq s$, the
squared distance between the two orientation-coordinate pairs is

$$
\begin{aligned}
&\left(\cos(a+\pi r/2)-\cos(a+\pi s/2)\right)^2\\
&\quad+\left(\sin(a+\pi r/2)-\sin(a+\pi s/2)\right)^2\\
&=2-2\cos\frac{\pi(r-s)}2.
\end{aligned}
$$

For the three unordered label pairs, $|r-s|$ is $1,1,2$. The squared
distances are therefore $2,2,4$, giving distances
$\sqrt{2},\sqrt{2},2$. None is zero. Because every track is a graph over the
same axial coordinate $k$, tracks with different axial coordinates cannot
meet either. Thus the three tracks are pairwise disjoint and each cut at fixed
$k$ contains exactly one endpoint from each label. $\square$

### T-PRIME-BRAID-PROJ-1 -- classical flattening creates an exact apparent crossing [Proved]

**Claim.** The classical flattening camera $C_B(x,y,k)=(x,k)$ is
noninjective on the three-track cable even though the native tracks are
disjoint.

**Proof.** At $k=1$, direct quarter-turn evaluation gives

$$
B_0(1)=(0,1,1),\qquad
B_1(1)=(-1,0,1),\qquad
B_2(1)=(0,-1,1).
$$

T-PRIME-BRAID-1 gives $B_0(1)\neq B_2(1)$, while

$$
C_B(B_0(1))=(0,1)=C_B(B_2(1)).
$$

Hence the projected crossing is an exact fiber of a noninjective camera, not
an intersection of the native strands. $\square$

**Boundary.** The cable theorem proves separation and identifies the lost
coordinate exactly. The three labels are three selected tracks, while each
track itself traverses the full four-orientation cycle. This camera fact does
not establish a new theorem about the values, gaps, or distribution of
$p(k)$.

## What these proofs do not establish

- The encoding gives no fast factorization algorithm; constructing
  $\alpha_n$ from $n$ still requires its prime valuations.
- Prime birth orientation is periodic and lossy.
- The helix form is a consequence of the chosen birth index and quarter-turn
  camera; it is not evidence of a new prime-distribution law.
- No ordinary-addition isomorphism is claimed.
- No infinite Euler product, zeta function, or statement about prime
  distribution follows from the factorization layer alone. The later analytic
  camera proves the first two on its restricted half-plane using additional
  convergence dependencies.
