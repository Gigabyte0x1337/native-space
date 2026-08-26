<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Separate Zeta and RE Perspective Proofs

## D-ZETA-RE-QUADRATIC-CAMERAS-1 -- the two camera poses

The classical multiplicative identity is the unit state

$$
e=(1,0)^T.
$$

Define zeta's quadratic camera and the perpendicular RE camera by

$$
P_Z=\frac{1}{2}
\begin{pmatrix}1&1\\1&1\end{pmatrix},
\qquad
P_R=\frac{1}{2}
\begin{pmatrix}1&-1\\-1&1\end{pmatrix}.
$$

These are the rational projector forms of the directions $(1,1)$ and
$(-1,1)$. The quadratic form avoids introducing $1/\sqrt{2}$ into the exact
Native Space language.

## T-ZETA-RE-ROTATION-1 -- the camera rotations [Proved]

Direct exact multiplication gives

$$
P_Z^2=P_Z,
\qquad
P_R^2=P_R,
\qquad
P_ZP_R=0,
\qquad
P_Z+P_R=I.
$$

Moreover,

$$
(1,1)\cdot(-1,1)=0.
$$

Thus the two cameras are perpendicular, neither camera overlaps the other,
and together they reconstruct the classical plane. The complete executable
proof is
[`../examples/zeta_re_perspective_rotation.ns`](../examples/zeta_re_perspective_rotation.ns).

## T-ZETA-RE-POSITION-1 -- both identity positions are one half [Proved]

Apply each camera to the classical multiplicative identity:

$$
P_Ze=\left(\frac{1}{2},\frac{1}{2}\right)^T,
\qquad
P_Re=\left(\frac{1}{2},-\frac{1}{2}\right)^T.
$$

Their separate quadratic positions are

$$
q_Z(e)=e^TP_Ze=\frac{1}{2},
\qquad
q_R(e)=e^TP_Re=\frac{1}{2}.
$$

They reconstruct the identity position:

$$
q_Z(e)+q_R(e)=e^T(P_Z+P_R)e=e^TIe=1.
$$

Therefore the half is not inserted as an arbitrary offset. It is the exact
position of the multiplicative identity seen through each of the two
perpendicular quadratic perspectives. The complete executable proof is
[`../examples/zeta_re_perspective_position.ns`](../examples/zeta_re_perspective_position.ns).

Run both proofs:

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --locked --bin native-space -- check examples/zeta_re_perspective_rotation.ns
cargo run --manifest-path language/runtime/Cargo.toml --locked --bin native-space -- check examples/zeta_re_perspective_position.ns
```

## T-ZETA-RE-WRAP-1 -- the wrappers reconstruct one state [Proved]

For one symbolic native coordinate pair $w=(x,y)^T$, define the separate
wrapped views

$$
w_Z=P_Zw,
\qquad
w_R=P_Rw.
$$

Completeness of the projectors gives

$$
w_Z+w_R=(P_Z+P_R)w=w.
$$

The quadratic measurements also reconstruct the full squared size:

$$
w^TP_Zw+w^TP_Rw=w^T(P_Z+P_R)w=w^Tw.
$$

The complete executable proof is
[`../examples/zeta_re_perspective_wrappers.ns`](../examples/zeta_re_perspective_wrappers.ns).

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --locked --bin native-space -- check examples/zeta_re_perspective_wrappers.ns
```

## T-ZETA-RE-POSITION-CANCEL-1 -- oriented positions cancel [Proved]

The two camera positions are derived separately from the multiplicative
identity and are both one half. Turn the RE position into the opposite
comparison orientation and apply native ADD:

$$
q_Z(e)+\mathrm{ORIENT}_2(q_R(e))
=\frac{1}{2}-\frac{1}{2}=0.
$$

The complete executable proof derives both positions through the projector
entries before performing the cancellation:
[`../examples/zeta_re_perspective_cancellation.ns`](../examples/zeta_re_perspective_cancellation.ns).

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --locked --bin native-space -- check examples/zeta_re_perspective_cancellation.ns
```

## T-ZETA-RE-VERTEX-PATH-1 -- perspective preserves the indexed path [Proved]

Let the native path be the ordered indexed pattern

$$
\Gamma=\bigoplus_k[k,v_k].
$$

Apply both cameras to each retained vertex before any flattening projection:

$$
\Gamma_Z=\bigoplus_k[k,P_Zv_k],
\qquad
\Gamma_R=\bigoplus_k[k,P_Rv_k].
$$

T-ZETA-RE-WRAP-1 applies independently at every INDEX, so

$$
\Gamma_Z+\Gamma_R
=\bigoplus_k[k,(P_Z+P_R)v_k]
=\bigoplus_k[k,v_k]
=\Gamma.
$$

The complete executable proof retains two consecutive outer vertex indices
and separate inner coordinate indices:
[`../examples/zeta_re_vertex_path.ns`](../examples/zeta_re_vertex_path.ns).

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --locked --bin native-space -- check examples/zeta_re_vertex_path.ns
```

This theorem explains why flattened primitive sequences are not a valid test
of cross-perspective path equivalence. The native INDEX path must be compared
before either camera discards a coordinate.

## T-ZETA-RE-CLASSICAL-AXIS-ROTATION-1 -- relative camera law [Proved]

The classical source number line has one coordinate $x$. A two-coordinate
camera zero-fills its absent coordinate, so its calculation form is
$v=(x,0)^T$. This is a definition of the source perspective, not a
restriction imposed on a two-coordinate source. Then

$$
P_Zv=\left(\frac x2,\frac x2\right)^T,
\qquad
P_Rv=\left(\frac x2,-\frac x2\right)^T.
$$

The clockwise quarter-turn $R_{-90}(a,b)=(b,-a)$ therefore gives

$$
P_Rv=R_{-90}P_Zv.
$$

Since $R_{-90}$ is invertible,

$$
P_Zv=0\Longleftrightarrow P_Rv=0.
$$

The complete executable proof uses one symbolic indexed classical-axis
coordinate and no privileged runtime operation:
[`../examples/zeta_re_classical_axis_rotation.ns`](../examples/zeta_re_classical_axis_rotation.ns).

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --locked --bin native-space -- check examples/zeta_re_classical_axis_rotation.ns
```

A pair $(x,y)$ with independent nonzero $y$ belongs to a different
two-coordinate source perspective. It is not a state of the classical source
axis and cannot refute this theorem.

## D-NATIVE-PERSPECTIVE-ZERO-1 -- same-state camera zero statement [Definition]

For a legal state $\Gamma$ on the one-dimensional classical source axis,
retain the two camera types and define

$$
Z_N(\Gamma):=P_Z\Gamma,
\qquad
R_N(\Gamma):=P_R\Gamma.
$$

The **same-state camera zero statement** is

$$
Z_N(\Gamma)=0
\Longrightarrow
R_N(\Gamma)=0.
$$

The names $Z_N$ and $R_N$ include their cameras. Erasing those wrappers
produces a different, untyped comparison and is not this definition.

## T-NATIVE-PERSPECTIVE-ZERO-1 -- same-state camera zero implication [Proved]

T-ZETA-RE-CLASSICAL-AXIS-ROTATION-1 gives

$$
R_N(\Gamma)=R_{-90}Z_N(\Gamma).
$$

Hence $Z_N(\Gamma)=0$ gives

$$
R_N(\Gamma)=R_{-90}0=0.
$$

The inverse quarter-turn proves the converse as well. Therefore

$$
\boxed{Z_N(\Gamma)=0\Longleftrightarrow R_N(\Gamma)=0.}
$$

The executable proof is the rotation identity in
[`../examples/zeta_re_classical_axis_rotation.ns`](../examples/zeta_re_classical_axis_rotation.ns).

**RH boundary.** This is a theorem about two cameras applied to the same
one-dimensional value. It is not a theorem relating the analytic zeta
aggregate to the recursive reflected-prime pattern. The global interpretation
of those two different objects as these invertibly related views is disproved
in `24-rh-projection-factorization-audit.md`.

## K-PRIME-FLATTENING-1 -- recursive prime pattern does not factor through the classical camera [Conjecture]

The Native Space research hypothesis is that the recursive multiplicative
prime object—with intrinsic INDEX, multiplication-generated ORIENT, and
multiplicative depth kept distinct—does not factor through the flattened
classical camera. Symbolically, if $\pi_{\mathrm{cl}}$ is that camera, we do
not expect a classical reconstruction $D$ satisfying

$$
D\!\left(\pi_{\mathrm{cl}}(\Gamma_{\mathbb{P}})\right)
=\Gamma_{\mathbb{P}}
$$

on the complete generative prime pattern.

This is not an additional premise in T-NATIVE-PERSPECTIVE-ZERO-1. That theorem
already keeps $P_Z$ and $P_R$ inside its statement and is proved. The
conjecture instead explains why projecting away the recursive pattern may make
the same structure unavailable in the flattened classical representation.

**Completion condition.** Proving this conjecture requires an exact scope for
the classical representation and a proof that every candidate reconstruction
fails on the complete generative prime-pattern image. General camera
noninjectivity alone is insufficient; the collision must occur on that image
and must change the relevant recursive property.

## Wrapping boundary

The proved camera facts determine the correct wrapper order:

1. construct the zeta pattern in its own native operations;
2. apply $P_Z$ to that pattern;
3. construct the RE pattern in its own native operations;
4. apply $P_R$ to that pattern;
5. compare the two wrapped results only after both camera transforms.

This file proves every constant, position, rotation, and common-state
reconstruction used by those wrappers. The remaining theorem must prove that
the analytic zeta pattern and reflected-equality pattern are precisely the two
wrapped views of one native state. That identification is not silently
included in the camera definitions.
