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
Arithmetic on framed values decodes, operates, and re-encodes; changing the
camera is not the same operation as mutating P to TP.

The optional frame policy chooses a common rational scale that brings the
largest absolute coordinate to one, retaining its exact inverse. The numerical
scale probe infers one decaying layer from three samples and reports its error
on a fourth unseen sample. It is a hypothesis report, not a convergence proof
or permission to erase residuals. No general automatic progressive extraction
or performance improvement is asserted.
