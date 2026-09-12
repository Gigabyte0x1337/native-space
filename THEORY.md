<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Homogeneous Native state

The authoritative scalar is `P=(L,A,M)`, with exact rational components.
Signed and zero coordinates are allowed; readouts have explicit domains.
Raw equality compares all three coordinates. Retained equality also includes
operation provenance. Decoded equality compares both `R` and `x`.

```text
x = M/L
R = (A+M)/L
T = L+A+M
```

Finite decoding requires L != 0. The common scale is retained:
P and cP have identical (R,x) for nonzero c, but are different raw states.
Two decoded degrees of freedom plus retained computational scale make three
raw coordinates. This is not three independent decoded algebraic values.

## Projective meaning and domains

A nonzero raw triple represents the projective point [L:A:M].
Projective equality is tested without division:

```text
P ~ Q iff P != (0,0,0), Q != (0,0,0), and
    L1 A2 = L2 A1
    L1 M2 = L2 M1
    A1 M2 = A2 M1
```

These equations say the two nonzero vectors are proportional. For example,
choose a nonzero component of P; its counterpart in Q must be nonzero, and
the cross-products fix one common nonzero ratio for all three components.
Testing this relation never changes the stored scale.

The finite chart has L != 0. The projective boundary has L = 0 and a nonzero
triple. Neither a boundary point nor the raw all-zero triple has a finite
decode. Raw (0,0,0) remains valid storage but represents **no projective point**;
the projective comparison returns false even when comparing it with itself.
The additive identity (1,0,0) is a different, valid projective point.

The raw polynomial operations extend to the boundary, but they are not total
operations on projective points: for example,
add((0,1,0),(0,1,0)) = (0,0,0). Where the resulting triple is nonzero,
bilinearity makes the operation independent of the representatives' scales.
No boundary ring structure or invented finite infinity value is asserted.

## Arithmetic

For P=(L,A,M) and Q=(K,B,N):

```text
add(P,Q)      = (LK, AK+BL, MK+NL)
multiply(P,Q) = (LK, AB+AN+MB, MN)
negate(P)     = (L,-A,-M)
inverse(P)    = (M(A+M), -AL, L(A+M))
```

ADD and MULTIPLY decode to componentwise arithmetic on (R,x).
The inverse additionally requires M != 0 and A+M != 0.
Literals n embed as (1,0,n), so their decoded pair is (n,n).
The raw additive identity is (1,0,0); the multiplicative identity is (1,0,1).

### Exact finite representation theorem

Define Phi(L,A,M) = ((A+M)/L, M/L) = (R,x) for L != 0.
For Q=(K,B,N), K != 0, direct substitution gives:

```text
Phi(add(P,Q))
 = ((AK+BL+MK+NL)/(LK), (MK+NL)/(LK))
 = ((A+M)/L + (B+N)/K, M/L + N/K)
 = Phi(P) + Phi(Q)

Phi(multiply(P,Q))
 = ((AB+AN+MB+MN)/(LK), MN/(LK))
 = (((A+M)(B+N))/(LK), (M/L)(N/K))
 = Phi(P) * Phi(Q)
```

Both operations on the right are componentwise. Every rational pair (R,x)
has representative (1,R-x,x). Conversely, a finite P equals
L*(1,R-x,x), so two finite states decode equally exactly when they differ
by nonzero common scale. Thus Phi induces a bijective algebra homomorphism:

```text
finite Native algebra modulo nonzero homogeneous scale ~= Q x Q
```

This is an exact representation theorem, not a theorem about raw scale or
provenance. Negation decodes to (-R,-x). The stated inverse decodes to
(1/R,1/x) only when both R and x are nonzero.

Q x Q has zero divisors: the nonzero pairs (1,0) and (0,1) multiply to (0,0).
Their Native representatives (1,1,0) and (1,-1,1) multiply to (1,0,0).
Consequently a nonzero Native state need not be invertible.

Do not confuse decoded algebraic laws with raw-coordinate laws.
For P=(2,2,2) and Q=S=(1,1,1):

```text
P*(Q+S) = (2,12,4)
P*Q+P*S = (4,24,8)
```

Both decode to (R,x)=(8,2), but their raw scales differ by two.
An optimizer cannot replace one with the other while claiming raw-state
or retained-graph equality. No common-scale normalization is automatic.

## Half-refinement

```text
split(P,"add")      = (L/2, A+L/2, M)
split(P,"multiply") = (L/2, A, M+L/2)
```

Both preserve T. Decoded, they are (2R+1,2x) and (2R+1,2x+1).
They redistribute LOAD; they do not double total scale.
A separate uniform rescaling P -> 2P doubles T and preserves (R,x).

## Cameras

For T != 0 the simplex camera is (L/T,A/T,M/T), whose components sum to one.
It is a positive simplex only on the nonnegative cone with T>0.
Equal distribution is (1/3,1/3,1/3). Retaining T restores the missing scale.

An orthogonal numerical camera is:

```text
u = (A-M)/sqrt(2)
v = (A+M-2L)/sqrt(6)
w = T/sqrt(3)
```

Its inverse is:

```text
L = -2v/sqrt(6)+w/sqrt(3)
A = u/sqrt(2)+v/sqrt(6)+w/sqrt(3)
M = -u/sqrt(2)+v/sqrt(6)+w/sqrt(3)
```

The log-ratio camera requires L,A,M > 0:

```text
u = log2(A/M)/sqrt(2)
v = log2(L*L/(A*M))/sqrt(6)
q = log2(T)
```

Swap A and M and u negates. Scaling by positive c adds log2(c) to q.
The implementation evaluates numerical cameras only on request; floating-point
round trips are tolerance checks, not exact equality. Native storage is rational.
T=0 invalidates simplex normalization, not the raw state. L=0 invalidates
finite classical decoding. Neither case is silently clamped to a finite point.

## Program recurrence and computational frames

A Program contains seed S and one reusable function G. Observation k means G^k(S).
Index is metadata, not an arithmetic factor, radial distance, or source address.
Frequency means recurrence/orbit structure of that Program; fitting a finite
numerical window does not prove continuation.

A rational matrix T is a computational transform only after exact inversion.
A framed value carries both TP and T. Decoding recovers P exactly.
Changing the camera is not the same operation as mutating P to TP.
For an operation B with bilinear coefficients b[a,b,c], the local tensor is:

```text
B_T(u,v) = T B(T^-1 u, T^-1 v)
C[i,j,k] = sum(a,b,c) T[i,a] b[a,b,c] T^-1[b,j] T^-1[c,k]
B_T(u,v)[i] = sum(j,k) C[i,j,k] u[j] v[k]
```

The implementation compiles these exact rational coefficients by evaluating
the frozen bilinear map on inverse-transformed basis vectors. ADD and MULTIPLY
then run directly on local coordinates. Negation and the two splits are linear,
so their transported matrices are T H T^-1. Inverse retains the simpler exact
decode/domain-check/apply/encode reference route. These identities hold for raw
triples, including boundary and zero triples where the operation is defined.

For mixed frames, convert the second operand with T1*T2^-1 and use the first
explicit frame. Never treat coordinates in two bases as interchangeable.
Composition is compose(T2,T1)=T2*T1; its inverse is T1^-1*T2^-1.
Compiled coefficients are caches, not authoritative serialized meaning.

## Explicit scale movement and numerical inference

rescale_pow2(P,k) multiplies each raw coordinate by exactly 2^k, including
negative k. It preserves finite (R,x) and projective meaning, not raw equality.
No rescaling is automatic.

The exact balanced-frame policy makes the largest absolute local coordinate
one. Its dyadic option chooses 2^-ceil(log2(max_abs(P))) using integer bit lengths
and rational comparison, without f64 conversion. The largest local magnitude
is in (1/2,1]; raw zero uses the identity frame. Decoding restores the exact
original raw scale, even near 2^10000 or 2^-10000.

For numerical samples F(N), F(2N), ... a real-layer model is
limit + sum_j amplitude_j * ratio_j^k, with 0 < abs(ratio_j) < 1.
alpha_j = -log2(abs(ratio_j)); a negative ratio alternates sign each step.
The four-sample probe reports one attempted fit and its fourth-sample error.

The progressive probe fits increasing layer counts using only the training
prefix. A difference recurrence identifies candidate ratios; rational storage
is not involved in this numerical fit. All layers are jointly refitted at each
rank, then subtracted successively to expose each residual. The first fit sets
the error baseline; further ranks continue only while held-out maximum error
improves. The held-out suffix selects ranks but never fits their parameters.
A selected model must match training and held-out samples within explicit
tolerances. Failed validation, predictions, errors and unexplained residuals
remain in the report. Since validation selects the model, it is not an
independent final test set.

This numerical hypothesis class excludes complex ratios, repeated-root
polynomial factors and arbitrary scale laws. Ill-conditioned, growing,
constant/unidentifiable and nonconvergent numerical fits are reported, not
forced into a model. Finite-window agreement never proves convergence and
never authorizes erasing a residual or rewriting an exact Program.

Prime behavior, RH, zeta interpretations, progressive inference and optimization
speedups are not consequences of the representation theorem and are not
claimed as theorems here.
