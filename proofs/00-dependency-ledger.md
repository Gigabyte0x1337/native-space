<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Core Proof Dependency Ledger

## Status

This is the authoritative dependency record for Native Space 1.0. Core entries
marked **Proved** are established in `01-core-algebra.md`; camera entries marked
**Open** remain unavailable as dependencies.

Proof status and execution status are separate:

- **Executable proof:** a closed finite `.ns` zero equality or Boolean
  tautology accepted by `native-space check`;
- **Operation derivation:** `native-space derive` expands source functions to
  primitive operations and reports their source/function trace; it carries no
  mathematical proof status;
- **Paper proof:** the written quantified argument is complete relative to its
  dependencies but is not checked by the current runtime.

Most universal algebraic and analytic entries in this ledger are paper proofs.
Executable finite witnesses live in [the examples directory](../examples/).
Function derivation is never silently promoted to an executable proof.

## Definition dependencies

```text
A-RF-1  real-field substrate ----> D-OS-1..7
                                      |
A-N-1   natural-number substrate -> D-IDX-1..8
                                      |
                  D-OS-* + D-IDX-* -> D-NS-1..12
```

The two branches meet only in the native-state definitions. No camera,
transform, prime fact, or application result is a permitted dependency.

## Oriented-scalar obligations

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-OS-1 | $\boxplus$ is closed, associative, and commutative | A-RF-1, D-OS-1, D-OS-2 | Proved |
| L-OS-2 | $\mathbf0$ is the ADD identity and $\boxminus z$ is the additive inverse | A-RF-1, D-OS-2 | Proved |
| L-OS-3 | $\boxtimes$ is closed, associative, and commutative | A-RF-1, D-OS-1, D-OS-3 | Proved |
| L-OS-4 | $\mathbf1$ is the MULTIPLY identity | A-RF-1, D-OS-3 | Proved |
| L-OS-5 | $\boxtimes$ distributes over $\boxplus$ | A-RF-1, D-OS-2, D-OS-3 | Proved |
| L-OS-6 | $\mathbf0$ absorbs $\boxtimes$ | A-RF-1, D-OS-1, D-OS-3 | Proved |
| L-OS-7 | $\nu(z)=0$ iff $z=\mathbf0$ | A-RF-1, D-OS-1, D-OS-5 | Proved |
| L-OS-8 | For $z\neq\mathbf0$, $z\boxtimes z^{[-1]}=\mathbf1$ | L-OS-7, D-OS-3, D-OS-6 | Proved |
| L-OS-9 | $\nu(z\boxtimes w)=\nu(z)\nu(w)$ | A-RF-1, D-OS-3, D-OS-5 | Proved |
| L-OS-10 | $\mathbf J^2=\mathbf{-1}$, $\mathbf J^3=\mathbf{-J}$, $\mathbf J^4=\mathbf1$ | D-OS-3, D-OS-4 | Proved |
| T-ORIENT-ZERO-1 | The four native orientations $\mathbf1,\mathbf J,\mathbf{-1},\mathbf{-J}$ ADD exactly to native zero | D-OS-1, D-OS-2, L-OS-10 | Proved |
| L-OS-11 | $\mathrm{orient}_{r+s}=\mathrm{orient}_r\circ\mathrm{orient}_s$ modulo four | L-OS-3, L-OS-4, L-OS-10, D-OS-7 | Proved |
| L-OS-12 | Conjugation preserves ADD and MULTIPLY, fixes identities, and is an involution | A-RF-1, D-OS-2 through D-OS-5 | Proved |
| T-REL-ORIENT-1 | Every native orientation locates every other by a relative orientation in the same algebra; relations compose and are invariant under a common turn | D-REL-ORIENT-1, L-OS-8, L-OS-10, L-OS-11 | Proved |
| T-PERSPECTIVE-ZERO-1 | Finite native cancellation is preserved and reflected from every native orientation perspective | D-REL-ORIENT-1, L-OS-5, L-OS-6, L-OS-8 | Proved |
| T-RJ-1 | The native number line embeds as a subfield, $\mathbf J^2=-\mathbf1$, and every oriented scalar has a unique line-plus-J decomposition | A-RF-1, D-RJ-1, D-OS-2 through D-OS-6 | Proved |
| T-OS-1 | $(\mathbb{O},\boxplus,\boxtimes)$ is a commutative field | L-OS-1 through L-OS-8 | Proved |
| T-QARITH-1 | Naturals, integers, and exact rationals constructed from A-IND-1 form the required ordered arithmetic and rational field | A-IND-1, D-PNAT-1, D-PINT-1, D-PRAT-1 | Proved relative |
| T-QOS-1 | Exact rational oriented scalars satisfy the native field, four-cycle, line/J, and perspective laws | T-QARITH-1, D-QOS-1 | Proved relative |
| T-QCORE-1 | Exact finite rational states satisfy all four-operation core laws relative only to A-IND-1 | T-QARITH-1, T-QOS-1, D-QIDX-1, D-QNS-1 | Proved relative |
| T-CLASSICAL-MODEL-1 | The finite native algebra is isomorphic to the classical finite monoid algebra and all four operations commute with the model | T-CX-2, D-NS-1 through D-NS-10, C-CLASSICAL-MODEL-1 | Proved |
| T-CLASSICAL-PERSPECTIVE-1 | Native observer views map to one common nonzero coefficient rotation | T-CLASSICAL-MODEL-1, D-REL-ORIENT-1 | Proved |
| T-CLASSICAL-RELATIVE-SOUND-1 | Finite native expression equality holds exactly when its classical formal-polynomial model equality holds | T-CLASSICAL-MODEL-1, D-LANG-2 | Proved |

The field theorem is internal. A later identification with conventional complex
numbers is a separate camera theorem and cannot be used above.

## Index obligations

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-IDX-1 | $\alpha\oplus_I\beta$ has finite support | A-N-1, D-IDX-3, D-IDX-6 | Proved |
| L-IDX-2 | $\oplus_I$ is associative and commutative | A-N-1, D-IDX-6 | Proved |
| L-IDX-3 | $\mathbf0_I$ is the identity | D-IDX-4, D-IDX-6 | Proved |
| L-IDX-4 | $d_j(\varepsilon_k\oplus_I\alpha)=d_j(\alpha)+\delta_{jk}$, where $\delta$ is the Kronecker delta | A-N-1, D-IDX-5 through D-IDX-7 | Proved |
| L-IDX-5 | Total depth is additive under $\oplus_I$ | A-N-1, L-IDX-1, D-IDX-7 | Proved |
| T-IDX-1 | $(\mathbb{I}_{\mathcal A},\oplus_I,\mathbf0_I)$ is a commutative monoid | L-IDX-1 through L-IDX-3 | Proved |

No inverse is expected for a nonzero multi-index; index depth is one-way in the
1.0 carrier.

## Native-state obligations

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-NS-1 | State ADD preserves finite support | L-OS-1, D-NS-1, D-NS-5 | Proved |
| L-NS-2 | Native states form a commutative group under $\oplus$ | L-OS-1, L-OS-2, L-NS-1, D-NS-3, D-NS-6 | Proved |
| L-NS-3 | State MULTIPLY preserves finite support | L-OS-3, L-IDX-1, D-NS-1, D-NS-7 | Proved |
| L-NS-4 | Single-term multiplication has the stated reduction | D-NS-4, D-NS-7 | Proved |
| L-NS-5 | $\star$ is associative and commutative | L-OS-1, L-OS-3, L-OS-5, L-IDX-2, L-NS-3 | Proved |
| L-NS-6 | $\mathsf1$ is the $\star$ identity | L-OS-4, L-IDX-3, D-NS-3, D-NS-7 | Proved |
| L-NS-7 | $\star$ distributes over $\oplus$ | L-OS-5, L-NS-1, L-NS-3 | Proved |
| L-NS-8 | $\mathsf0$ absorbs $\star$ | L-OS-6, D-NS-3, D-NS-7 | Proved |
| T-NS-1 | $(\mathcal N_{\mathcal A},\oplus,\star)$ is a commutative ring | L-NS-2, L-NS-5 through L-NS-8 | Proved |

## Separation and self-representation obligations

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-SEP-1 | `ORIENT` preserves every term's multi-index | D-NS-8 | Proved |
| L-SEP-2 | `INDEX(k)` preserves coefficients and increments only depth $k$ | A-N-1, L-IDX-4, L-NS-4, D-NS-9, D-NS-10 | Proved |
| L-SEP-3 | `ORIENT(r)` equals multiplication by $\eta(\mathbf J^r)$ | L-IDX-3, L-NS-4, D-NS-4, D-NS-8 | Proved |
| L-SEP-4 | `ORIENT` and `INDEX(k)` commute | L-NS-5, L-SEP-2, L-SEP-3 | Proved |
| L-SEP-5 | State `ORIENT` composes modulo four and four turns act identically | D-NS-2, D-NS-8, L-OS-10, L-OS-11 | Proved |
| L-ACT-1 | $\mathsf A_F\circ\mathsf A_G=\mathsf A_{F\oplus G}$ | L-NS-2, D-NS-11 | Proved |
| L-ACT-2 | $\mathsf M_F\circ\mathsf M_G=\mathsf M_{F\star G}$ | L-NS-5, D-NS-11 | Proved |
| T-ACT-1 | Native states represent additive and multiplicative actions on their own carrier | L-NS-1, L-NS-3, L-NS-6, L-ACT-1, L-ACT-2 | Proved |
| T-SELF-PATTERN-1 | Every finite observation of a native self-modeled recurrence remains native, and each finite step composition is represented in the same carrier | D-SELF-PATTERN-1, T-ACT-1, L-ACT-1, L-ACT-2 | Proved |
| L-PAIR-1 | The coefficient pairing is additive in each argument | L-OS-1, L-OS-5, L-OS-12, D-NS-12 | Proved |
| L-PAIR-2 | An embedded scalar emerges from the first pairing argument conjugated and from the second unchanged | L-OS-3, L-OS-5, L-OS-12, L-NS-4, D-NS-4, D-NS-12 | Proved |

`L-PAIR-1` and `L-PAIR-2` do not establish full self-observation. The limited
coefficient-camera claim remains T-PAIR-1 in the Stage 4 obligations below.

## Prohibited dependency audit

No proof above may depend on:

- a mapping to $\mathbb{C}$;
- polar coordinates, trigonometry, logarithms, or exponentials;
- the cone identity;
- Fourier or Mellin analysis;
- unique prime factorization or any prime ordering;
- numerical experiments;
- an application behaving as hoped.

Using one of these as an independent cross-check is allowed only after the
native proof is complete and must be labelled as verification, not derivation.

## Completed core proof order

The shortest non-circular route is:

```text
L-OS-1..11  +  L-IDX-1..5
          |                 |
          +------v----------+
              L-NS-1..8
                   |
               T-NS-1
                   |
       L-SEP-* and L-ACT-*
                   |
               T-ACT-1
```

This graph is proved in `01-core-algebra.md`. The camera equivalences are proved
separately in `02-camera-equivalences.md` below this dependency boundary.

## Stage 4 camera obligations

These obligations are listed now so candidate mappings cannot acquire implied
theorem status. They may be proved only after their Stage 3 dependencies close.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| T-FLAT-STACK-1 | The default flat-stack camera is a bijection and carries ADD/MULTIPLY as exact collection/convolution | C-FLAT-STACK-1, D-NS-1, D-NS-2, D-NS-5, D-NS-7 | Proved |
| D-CAMERA-AXIS-1 | Every axis is owned by one camera codomain; cross-perspective comparison requires an explicit transition | camera domain and codomain declarations | Definition |
| C-CAMERA-RESIDUAL-1 | Every selected perspective automatically combines its own signed axes and exposes only the nonzero residual | D-CAMERA-AXIS-1, D-NS-2 | Definition |
| T-CAMERA-RESIDUAL-1 | Perspective residual normalization is unique and empty exactly at that perspective's zero | C-CAMERA-RESIDUAL-1, T-OS-1, D-NS-2 | Proved |
| C-CAMERA-ZERO-FILL-1 | Every perspective declares its axes and reads each declared but absent residual axis as exact zero when used | D-CAMERA-AXIS-1, C-CAMERA-RESIDUAL-1 | Definition |
| T-CAMERA-ZERO-FILL-1 | Zero-fill is the unique information-preserving total extension and is zero exactly when the sparse residual is empty | C-CAMERA-ZERO-FILL-1, T-CAMERA-RESIDUAL-1 | Proved |
| T-CAMERA-TRANSITION-1 | A proved commuting transition transfers or reflects zero exactly according to its proved zero fiber | D-CAMERA-AXIS-1 | Proved |
| T-CX-1 | $\kappa$ is a bijection with the stated inverse | C-CX-1 | Proved |
| T-CX-2 | $\kappa$ preserves ADD, MULTIPLY, identities, and inverses | T-OS-1, T-CX-1 | Proved |
| T-MAT-1 | $\mu$ is a linear bijection onto $\mathcal C_2$ | C-MAT-1 | Proved |
| T-MAT-2 | $\mu$ preserves ADD and MULTIPLY, and its matrix action realizes $\boxtimes$ | T-OS-1, T-MAT-1 | Proved |
| T-POLAR-1 | $W$ is surjective with exactly the stated $2\pi$-periodic fibers | C-POLAR-1, conventional exp/trig facts | Proved |
| T-POLAR-2 | $\overline W$ is bijective and wrapped MULTIPLY is coordinate addition | T-OS-1, T-POLAR-1 | Proved |
| T-PSD-1 | $H$ has image $\mathcal P_1$ and exactly the stated fibers | C-PSD-1, A-RF-1 | Proved |
| T-CONE-1 | $Q$ has image $\mathcal K$, satisfies the cone identity, and has exactly the stated fibers | C-CONE-1, A-RF-1 | Proved |
| T-CONE-2 | $\Lambda$ bijects $\mathcal P_1$ and $\mathcal K$, and $Q=\Lambda\circ H$ | T-PSD-1, T-CONE-1 | Proved |
| T-COEF-1 | $\pi_\alpha$ is additive and has the stated fibers | L-NS-2, C-COEF-1 | Proved |
| T-BLOCK-1 | $\Pi_S|_{\mathcal N_S}$ is bijective with the stated inverse | L-NS-2, C-BLOCK-1 | Proved |
| T-PAIR-1 | The state-selected camera equals coefficient readout | L-PAIR-1, L-PAIR-2, C-PAIR-1 | Proved |
| L-EVAL-1 | Assigned powers satisfy $z^{\alpha\oplus_I\beta}=z^\alpha\boxtimes z^\beta$ | L-OS-3, L-OS-4, L-IDX-2, C-EVAL-1 | Proved |
| T-EVAL-1 | $\mathrm{ev}_z$ preserves ADD, MULTIPLY, zero, and one | T-NS-1, L-EVAL-1, C-EVAL-1 | Proved |
| T-EVAL-2 | Evaluation is generally noninjective and collapses generator states as stated | D-NS-2, D-NS-4, D-NS-9, C-EVAL-1, T-EVAL-1 | Proved |

### Finite derivative obligations

These statements are proved in `03-native-derivatives.md` under the explicitly
limited A-AN-1 finite real-analysis substrate.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-DER-1 | Every finite native stratum is a real vector space | T-NS-1, D-DER-1, D-DER-2 | Proved |
| L-DER-2 | The coefficient-size formula is a compatible finite-stratum norm | L-OS-7, L-DER-1, D-DER-4, A-AN-1 | Proved |
| L-DER-3 | Native MULTIPLY obeys the stated finite bilinear bound | L-OS-9, T-NS-1, L-DER-2 | Proved |
| T-DER-ADD-1 | Derivative of native ADD | L-DER-1, D-DER-7 | Proved |
| T-DER-MUL-1 | Derivative of native MULTIPLY is the product rule | L-DER-3, T-NS-1, D-DER-7 | Proved |
| T-DER-ORIENT-1 | Derivative of ORIENT is its own linear action | L-SEP-3, L-DER-1, D-DER-7 | Proved |
| T-DER-INDEX-1 | Derivative of INDEX is its own linear action | L-SEP-2, L-DER-1, D-DER-7 | Proved |
| T-DER-POWER-1 | Native scalar powers obey the power rule | L-BIN-1, T-OS-1, T-DER-MUL-1, A-AN-1 | Proved |
| T-DER-INV-1 | Native scalar inversion obeys the inverse rule off zero | T-OS-1, T-DER-MUL-1, A-AN-1 | Proved |
| T-DER-CX-1 | Derivative of $\kappa$ is its constant real-linear map | T-CX-2, D-DER-7 | Proved |
| T-DER-MAT-1 | Derivative of $\mu$ is its constant real-linear map | T-MAT-1, D-DER-7 | Proved |
| T-DER-POLAR-1 | Derivative and local rank of $W$ | T-POLAR-1, A-AN-1 | Proved |
| T-DER-PSD-1 | Derivative and rank of $H$ | T-PSD-1, A-AN-1 | Proved |
| T-DER-CONE-1 | Derivative and rank of $Q$ | T-CONE-1, A-AN-1 | Proved |
| T-DER-READ-1 | Derivatives of coefficient, block, and state-selected readouts | T-COEF-1, T-BLOCK-1, T-PAIR-1, D-DER-7 | Proved |
| T-DER-EVAL-1 | Derivative of finite parameterized evaluation | T-EVAL-1, T-DER-ADD-1, T-DER-MUL-1, T-DER-POWER-1 | Proved |

## Stage 5 language obligations

These statements are proved in `04-language-semantics.md`. They establish the
1.0 finite-language algorithms on paper; implementation conformance remains tested,
not formally verified.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| T-LANG-INTERP-1 | Reference interpretation equals denotational semantics | D-LANG-1, D-LANG-2, D-NS-3 through D-NS-10, D-PRIME-3, D-BIRTH-1, T-ENC-1, T-BIRTH-1 | Proved |
| T-LANG-UTF8-1 | String lowering is injective and `decode_utf8` recovers the original Unicode string | D-LANG-UTF8-1, D-NS-2, L-SEP-2, uniqueness of UTF-8 | Proved |
| T-LANG-OPERATOR-1 | Definition-ordered binary operators lower exactly to ordinary source-function calls and cannot override grammar or core operations | D-LANG-1, D-LANG-2, D-LANG-OPERATOR-1 | Proved |
| T-LANG-IMPORT-1 | Relative function-library imports resolve deterministically, retain source locations, and reject cycles and duplicate names | D-LANG-IMPORT-1, finite map/set semantics | Proved |
| L-LANG-COMP-1 | Compiled expressions satisfy the stack-extension invariant | D-LANG-1 through D-LANG-4 | Proved |
| T-LANG-COMP-1 | Compiled VM execution equals reference interpretation | T-LANG-INTERP-1, L-LANG-COMP-1, D-LANG-4 | Proved |
| T-LANG-ZERO-1 | A closed finite zero equality is accepted exactly when its lowered residual is native zero | T-LANG-INTERP-1, T-LANG-COMP-1, D-NS-2, D-NS-3 | Proved |
| T-LANG-OPT-1 | Every admitted optimizer rewrite preserves denotation | D-LANG-5, L-NS-2, L-NS-5, L-NS-6, L-NS-8, L-SEP-5 | Proved |
| T-LANG-SER-1 | Valid AST, bytecode, and canonical-state artifacts round trip | D-LANG-6, D-NS-1, D-NS-2 | Proved |

## Native Space 1.0 pure-function obligations

These statements are proved in `14-analytic-language-semantics.md`. They cover
generic source expansion only. Mathematical state proofs still terminate in
the single exact zero-equality judgment.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| T-FLANG-PURE-1 | Every emitted primitive step is one of the four core operations | D-FLANG-1, D-FLANG-2 | Proved |
| T-FLANG-SELF-1 | Direct and mutual source self-reference derive as finite pattern graphs | D-FLANG-1, D-FLANG-2, finite source syntax | Proved |
| T-FLANG-NAME-1 | Consistent function renaming has no effect on the expanded primitive-operation sequence | D-FLANG-1, D-FLANG-2 | Proved |
| T-FLANG-LOC-1 | Every primitive step and function-trace entry retains its `.ns` source line and function path | D-FLANG-2 | Proved |
| T-ZERO-ONLY-1 | Exact mathematical state proofs have only the equality-to-zero goal | D-LANG-1, T-LANG-INTERP-1, T-LANG-COMP-1 | Proved |

## Native Space 1.0 finite Boolean-kernel obligations

These statements are proved in `15-boolean-logic-kernel.md`. They cover only
finite propositional formulas; they do not prove native algebraic or analytic
claims.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| T-BLOGIC-SOUND-1 | Every accepted formula is a tautology | D-BLOGIC-1, D-BLOGIC-2 | Proved |
| T-BLOGIC-COMPLETE-1 | Every supported tautology within the backend variable bound is accepted | D-BLOGIC-1, D-BLOGIC-2 | Proved |
| T-BLOGIC-COUNTER-1 | Rejection as non-tautological reports a valuation making the formula false | D-BLOGIC-1 | Proved |
| T-BLOGIC-VERIFY-1 | Accepted certificates reconstruct a supported tautology and exact metadata | D-BLOGIC-2, T-BLOGIC-SOUND-1 | Proved |

## Native Space 1.0 variadic function-application obligations

These statements are proved in `16-variadic-function-application.md`. They
concern concrete source-function expansion and primitive operation traces, not
analytic truth.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| T-FAPP-TRACE-1 | Variadic `apply` preserves argument order and concatenates the exact nested operation traces | D-FAPP-1, finite sequence iteration | Proved |
| D-OP-INLINE-1 | Define primitive expansion by deleting function markers and retaining every core operation | D-FAPP-1 | Definition |
| T-OP-INLINE-1 | Fully expanded primitive steps contain no function calls and preserve exact core-operation order | D-OP-INLINE-1, T-FAPP-TRACE-1 | Proved |

## Stage 6 finite-flow obligations

These statements are proved in `05-native-flows.md` under the explicitly
limited A-ODE-1 substrate. The conventional function identification enters
only T-FLOW-2; it is not a dependency of the operation-first flow laws.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-FLOW-1 | Time translation composes by native MULTIPLY | A-ODE-1, D-FLOW-1, T-OS-1 | Proved |
| L-FLOW-2 | Generator rescaling is time rescaling | A-RF-1, A-ODE-1, D-FLOW-1 | Proved |
| L-FLOW-3 | Generator ADD becomes pointwise flow MULTIPLY | A-ODE-1, D-FLOW-1, T-OS-1 | Proved |
| T-FLOW-1 | The native flow camera preserves parameter ADD as MULTIPLY and avoids zero | C-FLOW-1, L-FLOW-1, L-FLOW-3, T-OS-1 | Proved |
| T-FLOW-2 | The native flow camera equals the conventional polar wrapping camera | A-ODE-1, C-FLOW-1, L-FLOW-2, L-FLOW-3, D-OS-3, C-POLAR-1 | Proved |

## Stage 6 finite combinatorial obligations

This statement is proved in `06-native-binomial.md`. It closes the previously
implicit binomial-theorem dependency in T-DER-POWER-1.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-BIN-1 | Oriented scalars and native states obey the finite binomial theorem | A-N-1, D-BIN-1, D-BIN-2, T-OS-1, T-NS-1 | Proved |

## Stage 6 finite Fourier obligations

These statements are proved in `07-finite-fourier.md`. The modulo-depth camera
is explicitly lossy; the DFT on its cyclic codomain is bijective.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-ROOT-1 | $\omega_N$ has exact native order $N$ | D-DFT-1, L-FLOW-1, L-FLOW-2, T-FLOW-2, T-POLAR-1 | Proved |
| L-ORTH-1 | Finite powers of $\omega_N$ obey character orthogonality | L-ROOT-1, T-OS-1 | Proved |
| T-CYCLIC-1 | The modulo-depth fold is a surjective homomorphism to cyclic convolution with stated fibers | D-CYCLIC-1, C-CYCLIC-1, T-NS-1 | Proved |
| T-DFT-1 | The finite Fourier camera is a real-linear bijection with the stated inverse | C-DFT-1, L-ORTH-1, T-OS-1 | Proved |
| T-DFT-2 | The finite Fourier camera maps cyclic convolution to pointwise MULTIPLY | C-DFT-1, L-ROOT-1, T-OS-1 | Proved |

## Stage 6 finite Mellin-type obligations

These statements are proved in `08-finite-mellin-character.md`. They cover
only finite character evaluation; infinite Mellin/Dirichlet analysis remains
undefined.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-MELLIN-1 | Weighted multi-index depth is additive | A-RF-1, D-MELLIN-1, D-IDX-6, L-IDX-2 | Proved |
| L-MELLIN-2 | The flow-derived weighted-depth character is multiplicative | D-MELLIN-2, L-MELLIN-1, L-FLOW-1, L-FLOW-2 | Proved |
| T-MELLIN-1 | The finite Mellin-type camera preserves ADD, MULTIPLY, zero, and one | C-MELLIN-1, L-MELLIN-2, T-EVAL-1 | Proved |
| T-MELLIN-2 | The camera has the stated exponential coordinate form and is generally noninjective | D-MELLIN-2, L-FLOW-2, T-FLOW-1, T-FLOW-2, T-CX-2, T-EVAL-2 | Proved |

## Stage 7 prime and integer-encoding obligations

These statements are proved in `09-prime-factorization.md`. The conventional
integer substrate does not assume unique factorization; it is proved before
the native encoding is classified as a bijection.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-BEZ-1 | Integer gcd has a Bézout representation | A-INT-1 | Proved |
| L-EUCLID-1 | A prime dividing a product divides one factor | D-PRIME-1, L-BEZ-1 | Proved |
| T-FTA-1 | Positive integers have unique finite prime factorization | A-INT-1, D-PRIME-1, L-EUCLID-1 | Proved |
| T-PRIME-INF-1 | Prime-predicate witnesses are unbounded, so the recursive value observation $p(k)$ is defined for every positive input | A-INT-1, T-FTA-1 | Proved |
| T-PI-1 | The finite native prime-count camera satisfies $\pi_N(p(k))=q(p(k))=k$ and follows the recurrence between consecutive value observations | D-PI-1, D-PRIME-2, D-PRIME-3, T-PRIME-INF-1 | Proved |
| L-VAL-1 | Prime valuations have finite support and add under multiplication | D-VAL-1, T-FTA-1 | Proved |
| T-ENC-1 | Positive-integer multiplication and prime native monomials are isomorphic monoids | D-ENC-1, D-DEC-1, L-VAL-1, T-FTA-1, L-NS-4 | Proved |
| T-NPRIME-1 | Intrinsic native prime monomials are exactly the one-depth generators, and Enc/Dec identify them exactly with classical primes | D-NPRIME-1, T-ENC-1, L-NS-4, L-IDX-5 | Proved |
| T-PRIME-LINE-1 | The native integer line is an exact classical ring camera, and the prime-value camera sends each intrinsic INDEX prime exactly to its classical number-line prime | C-PRIME-LINE-1, T-RJ-1, T-ENC-1, T-NPRIME-1 | Proved |
| T-BIRTH-1 | Wrapped birth orientation is $i^k$, four-periodic, and independent of depth | D-PRIME-3, D-BIRTH-1, L-OS-10, L-VAL-1, T-CX-2 | Proved |
| T-PRIME-PATTERN-1 | One `PrimePattern(k)` observation combines INDEX identity and wrapped orientation while keeping multiplicative depth distinct | D-PRIME-PATTERN-1, D-BIRTH-1, D-NS-10, L-SEP-3, L-OS-10, L-IDX-5 | Proved |
| T-PRIME-HELIX-1 | The flat observation camera is an injective quarter-turn helix; the prime-value camera is an injective axial deformation; the cone identifies opposite orientations | C-BIRTH-HELIX-1, C-PRIME-HELIX-1, C-PRIME-CONE-1, T-BIRTH-1, T-CONE-1 | Proved |
| T-PRIME-BRAID-1 | Three INDEX-distinct quarter-turn tracks form a disjoint cable with distances $\sqrt{2},\sqrt{2},2$ and three endpoints per transverse cut | C-PRIME-BRAID-1, T-BIRTH-1, T-CX-2 | Proved |
| T-PRIME-BRAID-PROJ-1 | Removing one signed orientation coordinate makes the classical cable camera noninjective and creates an exact apparent crossing | C-PRIME-BRAID-1, T-PRIME-BRAID-1 | Proved |

## Stage 7 finite arithmetic-function obligations

These statements are proved in `10-dirichlet-convolution.md`. The carrier is
restricted to finite-support arithmetic functions so it is exactly the finite
Native Space carrier; no completion or infinite convergence is implicit.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-DIR-1 | Pointwise operations and Dirichlet convolution preserve finite support | D-ARITH-1, D-ARITH-2, D-DIR-1 | Proved |
| T-ARITH-1 | The prime-index coefficient lift is an additive bijection preserving zero, negation, and one | C-ARITH-1, T-ENC-1, D-NS-6 | Proved |
| T-DIR-1 | Finite Dirichlet convolution is native MULTIPLY and the carriers are isomorphic rings | D-DIR-1, T-ARITH-1, T-ENC-1, D-NS-7 | Proved |
| T-DIR-CHAR-1 | Finite character evaluation preserves ADD and Dirichlet convolution | T-ARITH-1, T-DIR-1, T-MELLIN-1 | Proved |

## Stage 7 formal-completion obligations

These statements are proved in `11-formal-completion-and-euler-product.md`.
They are coefficientwise algebraic results. They do not discharge the separate
analytic-camera obligations.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| L-COMP-LOC-1 | Every fixed multi-index has finitely many two-part splits | D-IDX-5, D-IDX-6 | Proved |
| T-COMP-1 | Completed coefficient functions form a commutative ring containing finite Native Space | D-COMP-1, D-COMP-2, L-COMP-LOC-1, T-OS-1, T-NS-1 | Proved |
| T-DIR-INF-1 | All oriented arithmetic functions are isomorphic to completed prime-index states | D-ARITH-INF-1, C-ARITH-INF-1, T-ENC-1, L-COMP-LOC-1, T-COMP-1 | Proved |
| L-GEO-INF-1 | A one-prime formal geometric state inverts one minus its prime generator | D-GEO-INF-1, D-COMP-2, L-COMP-LOC-1 | Proved |
| T-ZETA-PATTERN-1 | One symbolic coefficient rule defines the generative coefficient-one zeta pattern without a stored range | D-ZETA-PATTERN-1, T-ENC-1 | Proved |
| T-EULER-F-1 | Finite products of successive pattern observations stabilize to the generative zeta pattern coefficientwise | D-ZETA-PATTERN-1, D-GEO-INF-1, L-GEO-INF-1, D-IDX-5 | Proved |

## Stage 7 analytic-zeta obligations

The direct absolute camera is discharged on
$\mathrm{Re}\kappa(s)>1$. The odd/even paired camera continues that
coordinate into the open critical strip. Neither result locates zeta zeros.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| T-ZETA-FLAT-1 | The default weighted zeta stack retains every INDEX coefficient before the explicit aggregation projection | C-AFLAT-1, C-COMP-EVAL-1, T-FLOW-1 | Proved |
| T-CAM-COMP-1 | The absolute completed camera is closed under MULTIPLY and multiplicative | A-CNS-1, D-SUM-1, C-COMP-EVAL-1, D-COMP-2, L-COMP-LOC-1, L-MELLIN-2 | Proved |
| L-ZETA-WEIGHT-1 | Prime-log weighted depth of $\alpha_n$ is $\log n$ | T-FTA-1, D-ENC-1, D-PLOG-1, D-MELLIN-1, A-LOG-1 | Proved |
| L-PLOG-HELIX-1 | Applying the prime-log scale to the flat prime helix preserves injectivity and gives a monotone axial deformation | C-PLOG-HELIX-1, T-PRIME-HELIX-1, A-LOG-1 | Proved |
| L-ZETA-COORD-1 | The zeta character at $\alpha_n$ has coordinate $n^{-s}$ and norm $n^{-\sigma}$ | T-MELLIN-2, L-ZETA-WEIGHT-1, A-LOG-1 | Proved |
| L-P-SERIES-1 | The real majorant $\sum n^{-\sigma}$ converges for $\sigma>1$ | A-CNS-1, A-LOG-1 | Proved |
| T-ZETA-CONV-1 | The generative zeta pattern is absolutely camera-summable for $\mathrm{Re}(s)>1$ | T-ENC-1, L-ZETA-COORD-1, L-P-SERIES-1, D-ZETA-PATTERN-1 | Proved |
| L-GEO-AN-1 | Every prime geometric state evaluates to the analytic geometric factor | L-MELLIN-2, L-ZETA-COORD-1, A-CNS-1, R-GEO-1 | Proved |
| T-ZETA-EULER-1 | The native zeta series equals the ordered Euler camera for $\mathrm{Re}(s)>1$ | T-CAM-COMP-1, L-GEO-AN-1, T-EULER-F-1, T-FTA-1, T-ZETA-CONV-1 | Reproduced |
| T-ZETA-PAIR-CONV-1 | The odd/even observation-pair series converges absolutely and locally uniformly for $\mathrm{Re}(s)>0$ | A-CNS-1, A-HOL-1, A-LOG-1, C-ZETA-PAIR-1 | Proved |
| T-ZETA-PAIR-IDENTITY-1 | The paired camera equals $(1-2^{1-s})\mathcal Z_N(s)$ for $\mathrm{Re}(s)>1$ | T-ZETA-CONV-1, T-ZETA-PAIR-CONV-1, A-CNS-1, L-ZETA-COORD-1 | Proved |
| L-ZETA-PAIR-DENOM-1 | $1-2^{1-s}$ is nonzero throughout $0<\mathrm{Re}(s)<1$ | A-LOG-1 | Proved |
| T-ZETA-STRIP-1 | The paired quotient is the analytic continuation of the direct zeta camera in the open critical strip | T-ZETA-PAIR-CONV-1, T-ZETA-PAIR-IDENTITY-1, L-ZETA-PAIR-DENOM-1, A-HOL-1 | Reproduced |
| T-ZETA-ZERO-PAIR-1 | In the open critical strip, the paired zeta coordinate is zero exactly when its odd/even paired ADD numerator is zero | C-ZETA-STRIP-1, L-ZETA-PAIR-DENOM-1 | Proved |
| T-ZETA-OUTPUT-RESIDUAL-1 | Automatic normalization of the zeta value on the classical axes leaves an empty residual exactly at zeta zero | C-ZETA-CLASSICAL-COORDS-1, T-CAMERA-RESIDUAL-1, D-CLASSIC-P-1, T-CLASSIC-P-1 | Proved |
| T-ZETA-OUTPUT-FRAME-1 | The explicit classical perspective gives a zero-filled zeta frame that is zero exactly at zeta zero | C-ZETA-CLASSICAL-COORDS-1, T-ZETA-OUTPUT-RESIDUAL-1, T-CAMERA-ZERO-FILL-1, T-CLASSICAL-MODEL-1 | Proved |

## Native multiplicity and RH-geometry obligations

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| T-AMULT-1 | Native local MULTIPLY depth agrees with analytic zero multiplicity | D-AMULT-1, T-CX-2 | Proved |
| L-XI-ZETA-ZERO-1 | Conventional xi and zeta have exactly the same zeros in the open critical strip | C-XI-REF-1, conventional gamma nonvanishing | Reproduced |
| T-RH-3D-EQ-1 | Conventional RH is equivalent to the centered native 3D axis-exclusion statement | C-XI-REF-1, L-XI-ZETA-ZERO-1, C-RH-3D-1, conventional RH definition | Reproduced equivalence only |
| T-AXIS-CANCEL-1 | A 2D zero is exactly two signed 1D cancellations, with birth orientations separated by index mod 4 | D-AXIS-PROJ-1, D-OS-11, L-OS-10, T-BIRTH-1, A-CNS-1 for infinite families | Proved |
| T-OPPOSITE-PAIR-1 | Birth orientations at k and k+2 are opposite, and their weighted pair cancels exactly when the gains agree | T-BIRTH-1, L-OS-10, L-NS-8 | Proved |
| T-ALL-PERSPECTIVES-1 | Every common native observer gives a unit rotation of the same aggregate-zero equation, not an independent coefficient equation | T-PERSPECTIVE-ZERO-1, D-REL-ORIENT-1, L-OS-5, L-OS-8 | Proved |
| T-DEPHASE-STACK-1 | Coefficientwise inverse phase MULTIPLY is a zero-preserving bijection while INDEX labels are retained | D-DEPHASE-1, T-OS-1, D-NS-2 | Proved |
| T-DEPHASE-AGG-NONCOMMUTE-1 | Scalar aggregation does not generically preserve its zero fiber under strandwise dephasing | D-DEPHASE-1, C-COMP-EVAL-1, D-NS-2 | Proved negative result |
| T-ZERO-FIBER-FACTOR-1 | A diagonal finite transform preserves every ADD-zero fiber exactly when all strand multipliers are equal | T-OS-1, D-NS-2 | Proved |
| T-TWO-PROJECTION-FIBER-1 | Two scalar-zero projections force only scalar zero of their difference, not equality of the retained indexed states | T-OS-1, D-NS-2, C-COMP-EVAL-1 | Proved negative result |
| T-RH-FLAT-1 | The default RH transform is the two signed-axis projections plus centered displacement and exactly reconstructs its xi value | C-RH-3D-1, D-AXIS-PROJ-1, T-AXIS-CANCEL-1, L-OS-10 | Proved |
| O-XI-NATIVE-1 | Construct completed xi operations-first and prove its coordinate equivalence and reflection | analytic integration/theta/gamma substrate not yet defined | Open; closes with the source construction plus complete domain, coordinate-equivalence, and reflection proofs |
| D-NATIVE-RE-1 | Define the one symbolic dephased reflected-equality pattern $\mathcal P_s(k)$ | D-PRIME-PATTERN-1, D-DUAL-CHAR-1, D-RH-REFL-1 | Definition |
| T-NATIVE-RE-1 | The native RE pattern has the sign of `1/2 - sigma` and is the zero law exactly at `sigma = 1/2` | D-NATIVE-RE-1, D-DUAL-CHAR-1, D-RH-REFL-1, L-ZETA-COORD-1, A-LOG-1 | Proved |
| D-CLASSIC-P-1 | Define the real and rotated-imaginary signed coefficient cameras | D-RJ-1, T-CX-2 | Definition |
| T-CLASSIC-P-1 | `classic_p` and `classic_i` agree with classical real and imaginary coefficient readout for every native scalar | D-CLASSIC-P-1, T-CX-2, T-RJ-1 | Proved |
| C-CENTERED-RE-PERSPECTIVE-1 | Define the RE perspective itself as the classical real coordinate translated by exact native $1/2$ | D-CLASSIC-P-1, T-QOS-1, D-RH-REFL-1 | Definition |
| T-CENTERED-RE-PERSPECTIVE-1 | Half of the multiplicative identity is the unique real-axis center at which reflection becomes negation; the centered RE perspective is zero exactly at $\mathrm{Re}(s)=1/2$ | C-CENTERED-RE-PERSPECTIVE-1, T-CLASSIC-P-1, T-QOS-1, T-AXIS-SUBTRACT-1, T-RJ-1, D-RH-REFL-1 | Proved |
| D-ZETA-RE-QUADRATIC-CAMERAS-1 | Define zeta's +45-degree projector and RE's perpendicular projector | T-QOS-1, D-CLASSIC-P-1 | Definition |
| T-ZETA-RE-ROTATION-1 | The two projectors are idempotent, orthogonal, and complete; their directions have zero dot product and are 90 degrees apart | D-ZETA-RE-QUADRATIC-CAMERAS-1, exact finite ADD/MULTIPLY evaluation | Proved |
| T-ZETA-RE-POSITION-1 | Each quadratic camera places the classical multiplicative identity at one half, and the two positions reconstruct one | D-ZETA-RE-QUADRATIC-CAMERAS-1, T-ZETA-RE-ROTATION-1, T-QOS-1 | Proved |
| T-ZETA-RE-WRAP-1 | Applying the separate zeta and RE projectors to one symbolic native state reconstructs both the state and its quadratic size | D-ZETA-RE-QUADRATIC-CAMERAS-1, T-ZETA-RE-ROTATION-1, exact finite ADD/MULTIPLY evaluation | Proved |
| T-ZETA-RE-POSITION-CANCEL-1 | The separately derived half-identity camera positions cancel after RE is turned into the opposite comparison orientation | T-ZETA-RE-POSITION-1, L-OS-10, exact finite ADD evaluation | Proved |
| T-ZETA-RE-VERTEX-PATH-1 | Complementary zeta and RE cameras reconstruct every coordinate of one ordered indexed native vertex path without flattening or merging vertices | T-ZETA-RE-WRAP-1, D-NS-2, exact finite INDEX/ADD/MULTIPLY evaluation | Proved |
| D-CLASSICAL-SOURCE-AXIS-1 | The classical source number line has one coordinate; absent coordinates introduced by a later camera are zero-filled rather than treated as additional source axes | D-CLASSIC-P-1, T-CAMERA-ZERO-FILL-1 | Definition |
| T-ZETA-RE-CLASSICAL-AXIS-ROTATION-1 | On every one-coordinate classical source vertex, the RE camera equals the zeta camera followed by a clockwise 90-degree rotation, so their zeros are equivalent | D-CLASSICAL-SOURCE-AXIS-1, D-ZETA-RE-QUADRATIC-CAMERAS-1, T-ZETA-RE-ROTATION-1, exact finite INDEX/ADD/MULTIPLY/ORIENT evaluation | Proved |
| D-NATIVE-PERSPECTIVE-ZERO-1 | Define the same-state camera statement as zero of one wrapped view implying zero of the perpendicular wrapped view of one legal classical-source value | D-CLASSICAL-SOURCE-AXIS-1, D-ZETA-RE-QUADRATIC-CAMERAS-1 | Definition |
| T-NATIVE-PERSPECTIVE-ZERO-1 | The same-state camera zero implication, and its converse, follow from the invertible quarter-turn relating the two views | D-NATIVE-PERSPECTIVE-ZERO-1, T-ZETA-RE-CLASSICAL-AXIS-ROTATION-1 | Proved; camera theorem, not RH |
| K-PRIME-FLATTENING-1 | The complete recursive multiplicative prime pattern does not factor through the flattened classical camera without reintroducing equivalent INDEX, MULTIPLY-recursion, and ORIENT structure | D-ZETA-PATTERN-1, D-BIRTH-1, T-PRIME-PATTERN-1, T-EVAL-2 | Conjecture; requires a pattern-image-specific non-factorization proof |
| T-ZETA-RE-GLOBAL-FACTOR-NO-1 | The invertibly rotated one-dimensional cameras cannot globally decode as zeta zero and centered-RE zero on the whole critical strip | T-NATIVE-PERSPECTIVE-ZERO-1, T-ZETA-STRIP-1, T-CENTERED-RE-PERSPECTIVE-1 | Proved negative result in `24-rh-projection-factorization-audit.md` |
| D-SHIFTED-RE-CENTER-1 | For fixed real c, define the typed RE residual R_c(s)=Re(s)-c and its camera-owned zero | D-CLASSIC-P-1, D-CAMERA-AXIS-1 | Definition |
| T-REFLECTION-CENTER-UNIQUE-1 | Half of the multiplicative identity is the unique fixed center at which reflection becomes residual negation | D-SHIFTED-RE-CENTER-1, D-RH-REFL-1, T-QOS-1 | Proved; exact finite witness in `examples/reflection-center.ns` |
| T-REVERSE-HALF-NO-1 | Centered RE zero does not imply zeta zero | T-ZETA-STRIP-1, T-REFLECTION-CENTER-UNIQUE-1 | Proved negative result at s=1/2 |
| T-OFFCENTER-RH-NO-1 | The zeta-zero-to-RE-zero statement is false for every fixed center other than one half | reproduced existence of critical-line zeta zeros, D-SHIFTED-RE-CENTER-1 | Reproduced consequence; the half-centered case remains K-RH-1 |
| T-RE-OUTPUT-RESIDUAL-1 | Automatic normalization of the RE pattern on the classical axes leaves an empty residual exactly at native RE zero | C-RE-CLASSICAL-COORDS-1, T-CAMERA-RESIDUAL-1, D-CLASSIC-P-1, T-CLASSIC-P-1 | Proved |
| T-RE-OUTPUT-FRAME-1 | The explicit classical perspective gives a zero-filled RE frame that is zero exactly at native RE zero | C-RE-CLASSICAL-COORDS-1, T-RE-OUTPUT-RESIDUAL-1, T-CAMERA-ZERO-FILL-1, T-CLASSICAL-MODEL-1 | Proved |
| D-FOUR-ORIENT-FUNCTIONS-1 | Define the four number-line-gain functions with native orientations seen classically as $(-i,i,1,-1)$ | D-OS-4, D-OS-7, L-OS-10 | Definition |
| T-AXIS-ADD-CANCEL-1 | Opposite signed axis values cancel by native ADD | L-OS-2 | Proved |
| T-FOUR-AXIS-CANCEL-1 | The equal-gain $-i/i$ and $1/-1$ function pairs cancel separately by finite ADD | D-FOUR-ORIENT-FUNCTIONS-1, T-AXIS-ADD-CANCEL-1, L-OS-10 | Proved |
| T-CLASSICAL-FOUR-CAMERA-1 | The classical camera sees $(-ai,ai,a,-a)$; real and rotated-imaginary projections see complementary cancelling pairs | D-FOUR-ORIENT-FUNCTIONS-1, D-CLASSIC-P-1, T-CLASSIC-P-1 | Proved |
| D-AXIS-SUBTRACT-1 | Axis subtraction turns the second axis by ORIENT$_2$ and then applies ADD | D-OS-7, D-NS-6 | Definition |
| T-AXIS-SUBTRACT-1 | Axis subtraction maps to classical subtraction and is zero exactly when its two inputs are equal | D-AXIS-SUBTRACT-1, T-CLASSICAL-MODEL-1, T-PERSPECTIVE-ZERO-1 | Proved |
| O-ZETA-RE-CLASSICAL-PATTERN-1 | Apply the proved zeta and RE quadratic wrappers separately and derive that wrapped zeta zero forces wrapped RE zero | T-ZETA-RE-ROTATION-1, T-ZETA-RE-POSITION-1, T-ZETA-OUTPUT-FRAME-1, T-RE-OUTPUT-FRAME-1 plus a zeta-specific wrapped zero identity | Open; closes with the universal critical-strip implication proof and is equivalent to K-RH-1 |
| T-DUAL-ALIGN-1 | The two reflected prime-pattern cameras share orientation and their symbolic axis subtraction is zero exactly on the critical line | D-DUAL-CHAR-1, D-AXIS-SUBTRACT-1, T-AXIS-SUBTRACT-1, D-RH-REFL-1, L-ZETA-COORD-1, A-LOG-1, L-MELLIN-2, C-DUAL-FLAT-1, T-AXIS-CANCEL-1 | Proved |
| L-DUAL-CONE-1 | The optional mismatch-cone view has the same symbolic pattern-law zero locus as the default flat camera | C-DUAL-CONE-1, T-CONE-1, T-AXIS-CANCEL-1, T-DUAL-ALIGN-1 | Proved |
| O-ZERO-ALIGN-1 | Construct one completed-xi-to-flat-stack transform and prove its zero-fiber law | O-XI-NATIVE-1 plus a derived zeta-specific signed-balance or coefficientwise transform law | Open; closes after O-XI-NATIVE-1 and a universal nontrivial-strip zero-fiber proof |
| T-RH-REDUCE-1 | O-XI-NATIVE-1 and O-ZERO-ALIGN-1 imply K-NATIVE-XI-1; the required coordinate equivalence then transfers it to K-RH-1 | T-PRIME-INF-1, T-DUAL-ALIGN-1, T-RH-3D-EQ-1 | Proved conditional reduction |
| T-RH-DIRECT-EQ-1 | O-ZETA-RE-CLASSICAL-PATTERN-1 is equivalent to K-RH-1 without xi or a geometry primitive | T-ZETA-STRIP-1, T-DUAL-ALIGN-1, T-FAPP-TRACE-1 | Reproduced equivalence only |
| T-RH-PATTERN-CONSEQUENCE-1 | A proved direct-zeta classical-pattern implication makes K-RH-1 a consequence of T-NATIVE-RE-1 | T-ZETA-STRIP-1, O-ZETA-RE-CLASSICAL-PATTERN-1, T-NATIVE-RE-1 | Proved conditional consequence |
| K-RH-1 | Every nontrivial classical zeta zero has real part one half | conventional zeta and nontrivial-zero definitions | Conjecture |
| K-NATIVE-XI-1 | Every nontrivial zero of the future operations-first completed-xi coordinate has centered real coordinate zero | O-XI-NATIVE-1, O-ZERO-ALIGN-1, T-RH-REDUCE-1 | Conjecture |

## Stage 8 application corollaries

These are finite consequences of the core. Their proofs and executable closed
instances are in `applications/`. They do not prove any application camera or
empirical gain.

| ID | Required statement | Dependencies | Status |
|---|---|---|---|
| T-MAT-CORE-1 | Native operator action distributes over ADD | L-NS-7, L-SEP-3, L-NS-2 | Proved |
| T-DYN-CORE-1 | Finite affine native steps are equivariant under common quarter ORIENT | L-SEP-3, L-NS-7, L-NS-5, L-NS-2 | Proved |
| T-NBODY-CORE-1 | A finite zero ADD remains and reflects zero under native perspective | T-PERSPECTIVE-ZERO-1 | Proved |
| T-NS-CORE-1 | Finite indexed interactions associate consistently | L-IDX-2, L-OS-1, L-OS-3, L-NS-5 | Proved |
| T-PROT-CORE-1 | Finite interaction ADD is independent of input order | L-NS-2 | Proved |
| T-PL-CORE-1 | Finite tagged AST-field ADD is independent of construction order | L-NS-2 | Proved |
