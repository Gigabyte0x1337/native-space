<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# R-BIN-1: The Binomial Theorem in Native Depth Coordinates

**Status:** Reproduced  
**Known result:** finite binomial theorem  
**Dependencies:** D-BIN-1, D-BIN-2, L-BIN-1, T-EVAL-1, T-CX-2  
**Native invariant:** distributive branching collects equal depth profiles by
natural multiplicity

## Conventional statement

For commuting quantities $x,y$ and $n\in\mathbb{N}_0$,

$$
(x+y)^n=\sum_{j=0}^{n}{n\choose j}x^{n-j}y^j.
$$

## Native derivation

For native states $F,G$, L-BIN-1 proves the more directly typed statement

$$
(F\oplus G)^{\star n}
\mathrel{=}
\mathop{\bigoplus}_{j=0}^{n}
\underline{{n\choose j}}
\star F^{\star(n-j)}
\star G^{\star j}.
$$

The proof expands one additional MULTIPLY step. Each branch chooses either
$F$ or $G$; commutativity makes branches with the same pair of depths land
on the same state monomial, and native ADD collects their natural
multiplicity. Pascal recursion records exactly how the two preceding depth
classes merge.

For primitive generators $X_k$ and $X_l$, the term indexed by $j$ has
directional depths

$$
d_k=n-j,
\qquad
d_l=j.
$$

The primitive identities $k,l$ remain distinct from those depths.

## Back-translation

Apply the evaluation homomorphism T-EVAL-1 with assignments
$X_k\mapsto x$ and $X_l\mapsto y$. It preserves the finite ADD and
MULTIPLY operations and sends native powers to ordinary powers, giving

$$
(x\boxplus y)^{\boxtimes n}
=\mathop{\boxplus}_{j=0}^{n}
\widehat{{n\choose j}}
\boxtimes x^{\boxtimes(n-j)}
\boxtimes y^{\boxtimes j}.
$$

The complex camera T-CX-2 turns this into the conventional complex binomial
theorem; restriction to real coordinates gives the ordinary real formula.

## Comparison

**Classification:** invariant-revealing and equivalent as a commutative-ring
proof.

Native depth coordinates make the exponent pair $(n-j,j)$ explicit and keep
it separate from primitive identity. The induction and Pascal collection are
the standard commutative-ring proof, so no shorter total derivation or broader
domain is claimed.

## Executable cross-check

[`../language/runtime/tests/reconstructions.rs`](../language/runtime/tests/reconstructions.rs) checks
the native identity exactly for powers zero through seven on two finite states.
This validates implementation agreement, not the proof.

## What is not established

- The theorem requires commuting multiplication; it does not apply unchanged
  to a future noncommutative extension.
- No infinite binomial series or non-natural exponent is defined.
- No computational advantage follows from the expansion.
