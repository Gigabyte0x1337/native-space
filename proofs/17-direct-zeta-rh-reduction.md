<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Direct-Zeta RH Function Reformulation

## Native definitions

The source has no completed-xi call and no geometry primitive. These are the
Native Space definitions involved:

```ns
let zeta_classical_pattern = () =>
classical_perspective(zeta_strip_coordinate)

let centered_re_pattern = () =>
centered_re_perspective()
dual_prime_alignment()

let re_classical_pattern = () =>
classical_perspective(centered_re_pattern)
```

`zeta_classical_pattern` is closed by T-ZETA-PAIR-CONV-1,
T-ZETA-PAIR-IDENTITY-1, L-ZETA-PAIR-DENOM-1, and T-ZETA-STRIP-1. The second
pattern has the critical-line zero locus proved by T-DUAL-ALIGN-1 and
T-NATIVE-RE-1. T-ZETA-OUTPUT-RESIDUAL-1 and T-RE-OUTPUT-RESIDUAL-1 normalize
their own sparse results automatically and expose only nonzero residuals.
T-ZETA-OUTPUT-FRAME-1 and T-RE-OUTPUT-FRAME-1 then zero-fill every declared
but absent classical axis. T-CLASSICAL-MODEL-1 proves that both explicit
`classical_perspective` calls use the same local coefficient convention.
They are not one geometric camera: T-ZETA-RE-ROTATION-1 proves zeta's
$+45^\circ$ projector and RE's perpendicular projector, while
T-ZETA-RE-POSITION-1 proves that both see the multiplicative identity at
quadratic position one half. T-CENTERED-RE-PERSPECTIVE-1 separately proves the
half-centered input coordinate, while T-DUAL-ALIGN-1 and T-NATIVE-RE-1 prove
that `centered_re_pattern` is zero exactly at $\mathrm{Re}(s)=1/2$.

The affine half belongs to the RE input coordinate; the quadratic half is the
identity position seen separately by both proved camera projectors.
D-RH-REFL-1 uses the multiplicative identity in
$\mathcal R(s)=\mathbf1\boxminus s^\dagger$, so it pairs $\sigma$ with
$1-\sigma$. T-CENTERED-RE-PERSPECTIVE-1 proves that
$1/2$ is their unique midpoint and the unique translation that makes this
reflection a sign reversal. Zeta remains tested against the ADD identity:
$\zeta(s)=0$, not $\zeta(s)=1/2$.

The proved RE side has three distinct layers:

$$
\mathrm{re\_classical\_pattern}(s;k)=0
\Longleftrightarrow
\Delta(s;k)=0
\Longleftrightarrow
\mathrm{centered\_re\_perspective}(s)=0
\Longleftrightarrow
\mathrm{Re}(s)=\frac{1}{2}.
$$

The output frame, reflected-prime mismatch, and centered input coordinate are
therefore zero-equivalent, but they are not one function with three names.
The direct-zeta obligation below is the implication into the left end of this
proved chain.

## Exact status

`native-space derive --source examples/math-functions.ns zeta_classical_pattern`
and the corresponding RE command expand the source calls to the four core
operations. Derivation does not assign proof status. The dependency ledger
records O-ZETA-RE-CLASSICAL-PATTERN-1 as open.

### O-ZETA-RE-CLASSICAL-PATTERN-1 -- zeta zero forces native RE zero in the classical perspective [Open]

Prove, for every nontrivial critical-strip zero,

$$
\boxed{
\zeta(s)=0
\Longrightarrow
\Delta(s;k)=0
\Longleftrightarrow
\mathrm{Re}(s)-\frac{1}{2}=0
}.
$$

In Native Space names, the open statement is: zero of
`zeta_classical_pattern` implies zero of `re_classical_pattern`. The first
implication below is O-ZETA-RE-CLASSICAL-PATTERN-1. The second
equivalence is T-DUAL-ALIGN-1 together with T-NATIVE-RE-1 and is already
proved. In the explicit source cameras, the first implication is

$$
\mathrm{Frame}_{\mathrm{cl}}(\mathcal Z_N^{\mathrm{pair}}(s))=0
\Longrightarrow
\mathrm{Frame}_{\mathrm{cl}}(\Delta(s;k))=0.
$$

By T-CAMERA-ZERO-FILL-1 this is equivalent to the sparse-residual implication.
The left side means both classical coordinates of the zeta pattern are zero;
the right side means both classical coordinates of the RE pattern are zero.
The perspective positions and rotations are now explicit. What remains is the
zeta-specific zero-location identity connecting the zero zeta value pair to
the separate centered RE position.

**Completion gate.** O-ZETA-RE-CLASSICAL-PATTERN-1 closes only when a complete
proof establishes this implication for every nontrivial critical-strip zeta
zero using the declared zeta and RE cameras. Shared axes, finite witnesses, and
successful source derivations do not close it.

### T-RH-DIRECT-EQ-1 -- the open zero-fiber theorem is equivalent to RH [Reproduced]

**Claim.** O-ZETA-RE-CLASSICAL-PATTERN-1 is equivalent to K-RH-1.

**Proof.** Assume O-ZETA-RE-CLASSICAL-PATTERN-1 and let $s$ be a nontrivial zeta zero.
T-ZETA-STRIP-1 identifies it with a zero of the paired native coordinate.
O-ZETA-RE-CLASSICAL-PATTERN-1 makes $\mathcal P_s$ zero. T-NATIVE-RE-1 then gives
$\mathrm{Re}(s)=\tfrac{1}{2}$, proving K-RH-1.

Conversely, assume K-RH-1. Every nontrivial zeta zero has
$\mathrm{Re}(s)=\tfrac{1}{2}$, and T-NATIVE-RE-1 makes
$\mathcal P_s\equiv\mathbf0$. This is O-ZETA-RE-CLASSICAL-PATTERN-1. $\square$

**Boundary.** The native language exposes the remaining statement exactly,
but renaming RH as a zero-fiber camera theorem does not prove it.
