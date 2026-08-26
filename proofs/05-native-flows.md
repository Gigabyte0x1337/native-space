<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Multiplicative Flow Proofs

## Permitted dependencies

These proofs use A-RF-1, A-ODE-1, the oriented-scalar field T-OS-1, D-FLOW-1,
and C-FLOW-1. Conventional exponential and trigonometric functions enter only in
T-FLOW-2, after the operation-first flow laws are closed. No exponential or
angle-addition formula is a dependency.

### L-FLOW-1 -- time translation is native multiplication [Proved]

**Claim.** For all $a\in\mathbb{O}$ and $s,t\in\mathbb{R}$,

$$
\mathcal E_a(t+s)=\mathcal E_a(t)\boxtimes\mathcal E_a(s).
$$

**Proof.** Fix $s$. As functions of $t$, let

$$
F(t)=\mathcal E_a(t+s),
\qquad
G(t)=\mathcal E_a(t)\boxtimes\mathcal E_a(s).
$$

D-FLOW-1 and the product rule give

$$
F'(t)=a\boxtimes F(t),
\qquad
G'(t)=a\boxtimes G(t).
$$

At $t=0$, both equal $\mathcal E_a(s)$. A-ODE-1 uniqueness therefore
gives $F=G$. $\square$

### L-FLOW-2 -- generator rescaling is time rescaling [Proved]

**Claim.** For all $a\in\mathbb{O}$ and $r,t\in\mathbb{R}$,

$$
\mathcal E_{ra}(t)=\mathcal E_a(rt).
$$

**Proof.** The right side has value $\mathbf1$ at $t=0$. The chain rule
and D-FLOW-1 give derivative

$$
r\bigl(a\boxtimes\mathcal E_a(rt)\bigr)
=(ra)\boxtimes\mathcal E_a(rt).
$$

It therefore solves the defining initial-value problem for
$\mathcal E_{ra}$, and A-ODE-1 uniqueness gives the claim. $\square$

### L-FLOW-3 -- generator ADD becomes flow MULTIPLY [Proved]

**Claim.** For all $a,b\in\mathbb{O}$ and $t\in\mathbb{R}$,

$$
\mathcal E_{a\boxplus b}(t)
=\mathcal E_a(t)\boxtimes\mathcal E_b(t).
$$

**Proof.** Let $F(t)=\mathcal E_a(t)\boxtimes\mathcal E_b(t)$. The product
rule, D-FLOW-1, commutativity, associativity, and distributivity give

$$
\begin{aligned}
F'(t)
&=(a\boxtimes\mathcal E_a(t))\boxtimes\mathcal E_b(t)
 \boxplus
 \mathcal E_a(t)\boxtimes(b\boxtimes\mathcal E_b(t))\\
&=(a\boxplus b)\boxtimes F(t).
\end{aligned}
$$

Also $F(0)=\mathbf1\boxtimes\mathbf1=\mathbf1$. Uniqueness identifies
$F$ with $\mathcal E_{a\boxplus b}$. $\square$

**Boundary.** This lemma uses commutativity of oriented-scalar MULTIPLY. A
noncommutative extension would require a commutation condition or an ordered
exponential and is not covered by this proof.

### T-FLOW-1 -- the native flow camera is a multiplicative homomorphism [Proved]

**Claim.** For $p,q\in\mathbb{R}^2$,

$$
\mathcal W_N(p+q)=\mathcal W_N(p)\boxtimes\mathcal W_N(q),
$$

and $\mathcal W_N(p)\neq\mathbf0$.

**Proof.** The generator assigned by C-FLOW-1 is real-linear in $p$, so
parameter addition becomes generator ADD. L-FLOW-3 at time one gives the
displayed law.

By L-FLOW-1,

$$
\mathcal E_a(1)\boxtimes\mathcal E_a(-1)
=\mathcal E_a(0)=\mathbf1.
$$

Thus every flow-camera value has a multiplicative inverse and cannot be zero.
$\square$

### T-FLOW-2 -- identification with conventional scale and angle [Proved]

**Claim.** For all $(\rho,\theta)\in\mathbb{R}^2$,

$$
\mathcal W_N(\rho,\theta)
=\left(e^\rho\cos\theta,e^\rho\sin\theta\right)
=W(\rho,\theta).
$$

**Proof.** Write $U(t)=\mathcal E_{\mathbf1}(t)=(u(t),v(t))$. D-OS-3 and
D-FLOW-1 give

$$
u'=u,\qquad v'=v,\qquad (u(0),v(0))=(1,0).
$$

A-ODE-1 uniqueness identifies $u(t)=e^t$ and $v(t)=0$. Hence

$$
\mathcal E_{\mathbf1}(t)=(e^t,0).
$$

Likewise write $R(t)=\mathcal E_{\mathbf J}(t)=(c(t),s(t))$. Its native
flow equation is

$$
c'=-s,\qquad s'=c,\qquad (c(0),s(0))=(1,0).
$$

By the A-ODE-1 characterization,

$$
\mathcal E_{\mathbf J}(t)=(\cos t,\sin t).
$$

Now L-FLOW-2 and L-FLOW-3 give

$$
\begin{aligned}
\mathcal W_N(\rho,\theta)
&=\mathcal E_{\rho\mathbf1\boxplus\theta\mathbf J}(1)\\
&=\mathcal E_{\mathbf1}(\rho)
  \boxtimes\mathcal E_{\mathbf J}(\theta)\\
&=(e^\rho,0)\boxtimes(\cos\theta,\sin\theta)\\
&=(e^\rho\cos\theta,e^\rho\sin\theta).
\end{aligned}
$$

The final expression is C-POLAR-1's $W$. No addition formula for
exponential, sine, or cosine was used. $\square$

## What these proofs do not establish

- A-ODE-1 is an explicit analytic substrate, not a theorem of the finite ring.
- The coordinate identification does not derive real analysis, $\pi$, or the
  complete unit-circle fiber theorem.
- The flow camera is not represented by one finite native state in 1.0.
- The finite exact-rational operation traces are executable in Native Space
  1.0. The imported ODE existence boundary A-ODE-1 remains a paper-level
  analytic assumption and is not proved by the runtime.
