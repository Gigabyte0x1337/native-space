<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Multiplicative Flow Camera

This is a Stage 6 analytic extension of the finite algebraic core. It derives a
scale-orientation exponential camera from native MULTIPLY before identifying
its coordinate functions with conventional exponential and trigonometric
functions.

It does not alter Native Space 1.0 and is not a reconstruction of real
analysis from algebra alone. The one new analytic substrate assumption is
explicit below.

### A-ODE-1 -- finite linear-flow substrate [Axiom]

On a finite-dimensional real vector space, every constant-coefficient linear
ordinary differential equation has a unique global continuously
differentiable solution for each initial value. Standard real chain and product
rules hold. The conventional functions $e^t$, $\cos t$, and $\sin t$ are
uniquely characterized by their usual first-order real initial-value systems.

This assumption supplies existence and uniqueness, not exponential addition,
angle addition, polar fibers, or a native logarithm. Those cannot be used to
prove the flow laws below.

## Native flow definition

### D-FLOW-1 -- multiplication-generated flow [Definition]

For $a\in\mathbb{O}$, define

$$
\mathcal E_a:\mathbb{R}\to\mathbb{O}
$$

as the unique A-ODE-1 solution of

$$
\mathcal E_a'(t)=a\boxtimes\mathcal E_a(t),
\qquad
\mathcal E_a(0)=\mathbf1.
$$

For fixed $a$, D-OS-3 and the real-field laws make
$z\mapsto a\boxtimes z$ a constant real-linear map, so it lies inside the
declared A-ODE-1 domain.

The generator $a$, derivative, and result all live in the oriented-scalar
carrier. No power series, complex camera, exponential, sine, cosine, or polar
coordinate is part of this definition.

## Flow camera definition

### C-FLOW-1 -- native scale-orientation flow camera [Definition]

Every $a=(\rho,\theta)$ decomposes internally as

$$
a=(\rho,0)\boxplus(0,\theta)
=\rho\mathbf1\boxplus\theta\mathbf J,
$$

where real scalar multiplication is coordinatewise. Define

$$
\mathcal W_N:\mathbb{R}^2\to\mathbb{O},
\qquad
\mathcal W_N(\rho,\theta)
:=\mathcal E_{\rho\mathbf1\boxplus\theta\mathbf J}(1).
$$

The target is provisionally all of $\mathbb{O}$. Proving that every value is
nonzero, that parameter ADD maps to native MULTIPLY, and that
$\mathcal W_N$ equals the earlier conventional wrapping camera $W$ are
separate obligations.

## Proof obligations

| ID | Required statement | Status |
|---|---|---|
| L-FLOW-1 | Time translation composes by native MULTIPLY | Proved in `../proofs/05-native-flows.md` |
| L-FLOW-2 | Rescaling a generator rescales flow time | Proved in `../proofs/05-native-flows.md` |
| L-FLOW-3 | ADD of commuting generators becomes pointwise MULTIPLY of flows | Proved in `../proofs/05-native-flows.md` |
| T-FLOW-1 | $\mathcal W_N$ maps parameter ADD to native MULTIPLY and never reaches zero | Proved in `../proofs/05-native-flows.md` |
| T-FLOW-2 | $\mathcal W_N(\rho,\theta)=W(\rho,\theta)$ | Proved in `../proofs/05-native-flows.md` |

## Explicit exclusions

- Full self-observation is not obtained; the flow is a derived transformation,
  not yet encoded as one finite native state.
- A global single-valued logarithm is not defined.
- The exact $2\pi$ orientation fiber and full polar covering still use the
  conventional unit-circle theorem recorded in T-POLAR-1.
- No infinite-support native state or arbitrary matrix exponential is defined.
- The finite language remains exact-rational and does not execute this analytic
  camera.
