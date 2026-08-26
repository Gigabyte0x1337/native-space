<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Method and Claim Status

This file controls how Native Space statements are recorded. It prevents a
definition from being reported as a discovery, a coordinate resemblance from
being called an equivalence, and numerical evidence from being called a proof.

## Foundational boundary

Native Space 1.0 is built relative to an ordinary real field
$\mathbb{R}$ and the nonnegative integers $\mathbb{N}_0$. The real field
laws, natural-number semiring laws, and ordinary finite/countable set and
function constructions are substrate assumptions. The project is not attempting
to reconstruct set theory, the real numbers, natural-number arithmetic, or
mathematical logic from nothing.

“From the ground up” therefore means:

> derive every Native Space law from the declared substrate and the project's
> typed definitions, without importing complex arithmetic, transforms, special
> functions, prime facts, or target theorems as hidden axioms.

Changing this boundary requires a documented design decision and a new version
of the core specification.

For the 1.0 work, the public decision record is intentionally kept
inside this method document and `language/SPEC.md` rather than a separate ADR
directory. The preserved decisions are:

1. the public algebra has exactly ADD, MULTIPLY, ORIENT, and INDEX;
2. the finite flat stack is the default lossless camera, while the cone/PSD
   view is a derived quadratic camera;
3. prime birth identity, wrapped orientation, and multiplicative depth are
   separate coordinates;
4. functions, cameras, and pattern recurrence are source-defined rather than
   privileged mathematical runtime names;
5. executable finite checks, paper proofs, open obligations, and experiments
   retain separate status.

Changing one of these invariants requires editing this section, the affected
formal definitions, and the language specification together. Private design
notes may explain development history, but they are not dependencies of the
published theory.

## Canonical notation

Native Space source is the primary notation in every document. A proof should
show its operation path as fenced source before using a classical formula:

```ns
let centered_re_perspective = () =>
axis_subtract(classic_p, exact_half)
```

Its proof status and theorem ID are recorded in the dependency ledger, not in
the language program.

The canonical vocabulary is:

| Native name | Meaning | Classical check only |
|---|---|---|
| `zeta_strip_coordinate` | paired native zeta value in the open critical strip | $\zeta(s)$ |
| `classical_perspective(value)` | a generic local `classic_p` and `classic_i` coefficient readout with exact zero-fill | $(\mathrm{Re},\mathrm{Im})$ |
| `classical_source_axis` | the one-coordinate classical input number line; camera-only missing coordinates are zero-filled | $x$ |
| `zeta_classical_pattern` | zeta's local two-axis value camera | $(\mathrm{Re}\zeta,\mathrm{Im}\zeta)$ |
| `centered_re_perspective` | real input axis translated by exact native one half | $\mathrm{Re}(s)-1/2$ |
| `centered_re_pattern` | reflected-prime realization with the same zero locus | $\Delta(s;k)$ |
| `re_classical_pattern` | the reflected-equality output's own local coefficient camera | classical coordinates of $\Delta(s;k)$ |
| `zeta_re_quadratic_cameras` | zeta's +45-degree projector and the perpendicular RE projector | $P_Z,P_R$ with $P_Z+P_R=I$ |

The canonical proved RE chain is

$$
\mathrm{re\_classical\_pattern}=0
\Longleftrightarrow
\mathrm{centered\_re\_pattern}=0
\Longleftrightarrow
\mathrm{centered\_re\_perspective}=0
\Longleftrightarrow
\mathrm{Re}(s)=\frac{1}{2}.
$$

This is a chain of zero-locus equivalences, not a claim that the three values
are identical away from zero. The separate zeta statement is an implication
into the left side of this chain and keeps its own status.

“Residual” and “total frame” are reserved for the generic sparse and
zero-filled camera constructions C-CAMERA-RESIDUAL-1 and
C-CAMERA-ZERO-FILL-1. They are not alternate names for zeta, RE, or a
perspective. Classical notation is allowed only to define a camera, verify a
native identity, or cite an established theorem. It is never the unstated
default language of a native proof.

## Statement labels

Every substantive statement uses exactly one primary label.

| Label | Meaning | Required evidence |
|---|---|---|
| **Definition** | Introduces a symbol or construction | Complete type, domain, and equality rule |
| **Axiom** | Accepted substrate assumption | Explicit scope and reason |
| **Proved** | Follows deductively from listed dependencies | Complete proof or checkable formal proof |
| **Reproduced** | Known result re-derived in native terms | Native proof and conventional back-translation |
| **Observed** | Reproducible finite experiment | Code, data, metric, uncertainty, and environment |
| **Experiment** | Frozen or planned empirical procedure, not an observed result | Dataset, baseline, metric, budget, seeds, and stopping rule |
| **Conjecture** | Precise unproved mathematical statement | Falsifiable formulation and dependencies |
| **Open obligation** | Required construction or proof interface not yet supplied | Exact target, missing dependencies, and completion gate |
| **Hypothesis** | Precise empirical expectation | Baseline, metric, budget, and failure criterion |
| **Speculation** | Direction not yet precise enough to test | Clear warning; no evidential use |
| **Refuted** | Contradicted by proof or experiment | Counterexample or reproducible negative result |

Longer labels may be used in prose, but registries use the compact labels above.

## Identity rules

Definitions, lemmas, theorems, cameras, hypotheses, and experiments receive
stable identifiers:

```text
A-...   substrate axiom
D-...   definition
L-...   lemma
T-...   theorem
C-...   camera or mapping
R-...   reconstructed known result
K-...   mathematical conjecture
O-...   open construction or proof obligation
H-...   empirical hypothesis
E-...   experiment
```

Renaming a title does not change its identifier. A materially changed statement
receives a new identifier; the old one is marked superseded rather than silently
rewritten.

## Equivalence vocabulary

The following words are not interchangeable:

- **bijection:** one-to-one and onto as sets;
- **homomorphism:** preserves named operations;
- **isomorphism:** bijective homomorphism with an operation-preserving inverse;
- **embedding:** injective structure-preserving map;
- **covering:** locally reversible map with explicitly described multiple
  preimages;
- **quotient:** intentionally identifies states under a stated equivalence;
- **projection / camera:** readout that may discard information;
- **approximation:** has a stated error on a stated domain;
- **analogy:** explanatory only and carries no proof.

The word **equivalent** may be used only after the exact relation and domain are
proved. A visual match is never sufficient.

## Proof records

Each proof must state:

1. the exact statement and quantifiers;
2. all dependencies by stable identifier;
3. the substrate assumptions used;
4. the derivation;
5. edge and singular cases;
6. what the result does not establish.

The derivation must write the concrete definitions, equations, and native
operation sequence. A function name may aid navigation but cannot replace any of
those contents. Machine reports enforce the same rule: `primitive_steps`
contains no function calls, while `function_trace` is non-authoritative metadata.

Every program printed in a proof document must be valid `.ns` source. A fully
executable proof must run with `native-space check` and conclude a closed finite
zero equality or finite Boolean tautology. `native-space derive` is only an
executable operation expansion; it does not prove its paper theorem. Until the
language gains quantified arithmetic and
analysis kernels, universal prime, limit, holomorphic, and zeta arguments must
be labelled **paper proof; executable formalization unavailable**. They may not
be described as fully runnable.

If a conventional theorem is used to check a result, it belongs in a separate
verification paragraph and cannot also be a dependency of the native proof.

## Experiment records

Each experiment must state:

1. the hypothesis being tested;
2. theory dependencies;
3. dataset and licensing provenance;
4. train, calibration, validation, and test boundaries where applicable;
5. conventional baselines;
6. equal storage, compute, precision, and error budgets;
7. software and hardware environment;
8. metrics and uncertainty;
9. predetermined failure criteria;
10. links to code, data, and immutable results.

Total system cost includes encoding, transforms, selection, interaction,
decoding, training or fitting, memory movement, and required precision. Moving
cost outside the measured operation is not a gain.

## Current claim index

The [dependency ledger](../proofs/00-dependency-ledger.md) is the complete,
canonical claim register. The table below is a reader-facing grouped index. It
names every RH-facing camera and status separately, but it does not duplicate
every supporting lemma. If wording or status ever differs, the ledger must be
fixed first and this index updated in the same change.

| ID | Label | Statement | Status |
|---|---|---|---|
| A-RF-1 | Axiom | The substrate is a real field | Accepted for 1.0 |
| A-N-1 | Axiom | Nonnegative integers obey their usual semiring laws | Accepted for 1.0 |
| A-AN-1 | Axiom | Standard finite-dimensional real limit theory | Accepted for Stage 4 finite calculus only |
| A-ODE-1 | Axiom | Standard global existence and uniqueness for finite constant-coefficient linear ODEs | Accepted for the Stage 6 native-flow reconstruction only |
| A-INT-1 | Axiom | Ordered integer domain, induction/well-ordering, and integer division algorithm | Accepted for finite Stage 7 arithmetic; unique factorization is not assumed |
| D-OS-1 through D-OS-7 | Definition | Oriented-scalar carrier and operations | Defined in `01-native-system.md` |
| D-PATTERN-ONTOLOGY-1 / D-SELF-PATTERN-1 / T-SELF-PATTERN-1 | Definition / Proved | Native patterns use finite observations and finite self-modeled actions; no range carrier or infinity element is added | Defined in `01-native-system.md`; finite-observation closure proved in `proofs/01-core-algebra.md` |
| D-RJ-1 / T-RJ-1 | Definition / Proved | The native number line embeds internally, $\mathbf J^2=-\mathbf1$, and every oriented scalar has a unique line-plus-J decomposition | Defined in `01-native-system.md`; proved without the complex camera in `proofs/01-core-algebra.md` |
| T-ORIENT-ZERO-1 | Proved | The four native orientations $\mathbf1,\mathbf J,\mathbf{-1},\mathbf{-J}$ ADD exactly to native zero | Proved from the native pair operations in `proofs/01-core-algebra.md`; no classical or prime model used |
| D-REL-ORIENT-1 / T-REL-ORIENT-1 / T-PERSPECTIVE-ZERO-1 | Definition / Proved | Every orientation describes every other inside the same algebra, relative relations compose, and zero cancellation is perspective-invariant | Defined in `01-native-system.md`; proved in `proofs/01-core-algebra.md` without a classical camera |
| A-IND-1 / T-QARITH-1 / T-QOS-1 / T-QCORE-1 | Axiom / Proved | Exact rational four-operation Native Space is constructed from a minimal explicit inductive substrate, without real, complex, or analytic premises | Strict optional trust-reduction profile in `12-foundational-purity.md` and `proofs/20-foundational-purity.md` |
| C-CLASSICAL-MODEL-1 / T-CLASSICAL-MODEL-1 / T-CLASSICAL-RELATIVE-SOUND-1 | Definition / Proved | One classical finite monoid algebra models all four operations and preserves and reflects finite expression equality | Default practical soundness profile in `13-classical-model.md` and `proofs/21-classical-model.md` |
| D-IDX-1 through D-IDX-8 | Definition | Primitive labels and finite multi-indices | Defined in `01-native-system.md` |
| D-NS-1 through D-NS-12 | Definition | Native states and four public operations | Defined in `01-native-system.md` |
| L-* / T-OS-* / T-IDX-* / T-NS-* / T-ACT-* | Proved | Core algebraic laws | Proved on paper in `proofs/01-core-algebra.md`; not machine-checked |
| C-FLAT-STACK-1 / T-FLAT-STACK-1 | Definition / Proved | Default lossless stack of signed orientation coordinates over exact INDEX locations | Defined in `02-cameras-and-mappings.md`; proved bijective in `proofs/02-camera-equivalences.md`; runtime schema `flat-stack-v1` |
| C-CX-1 through C-EVAL-1 | Definition | Typed coordinate cameras and native readouts | Defined in `02-cameras-and-mappings.md`; camera typing does not preclude later self-representation |
| T-CX-1 through T-EVAL-2 | Proved | Camera classifications and equivalences | Proved on paper in `proofs/02-camera-equivalences.md` |
| C-CAMERA-RESIDUAL-1 / T-CAMERA-RESIDUAL-1 | Definition / Proved | A perspective's canonical sparse residual is empty exactly at perspective zero | Defined in `02-cameras-and-mappings.md`; proved in `proofs/02-camera-equivalences.md` |
| C-CAMERA-ZERO-FILL-1 / T-CAMERA-ZERO-FILL-1 | Definition / Proved | Zero-fill is the unique information-preserving total extension of that sparse residual | Defined in `02-cameras-and-mappings.md`; proved in `proofs/02-camera-equivalences.md` |
| L-DER-* / T-DER-* | Proved | Finite native and camera derivative laws | Proved on paper in `proofs/03-native-derivatives.md`; no infinite-support calculus |
| D-LANG-1 through D-LANG-6 | Definition | Four-operation core AST, function lowering, VM, compiler, optimizer, and serialization | Native Space 1.0 defined in `proofs/04-language-semantics.md` and implemented under `language/runtime` |
| D-LANG-UTF8-1 / T-LANG-UTF8-1 | Definition / Proved | Strings lower injectively to ordered UTF-8 byte/position INDEX terms and decode exactly | Paper proof in `proofs/04-language-semantics.md`; Unicode, escape, and order tests in Rust |
| D-LANG-OPERATOR-1 / T-LANG-OPERATOR-1 | Definition / Proved | Definition-ordered binary operators lower to ordinary calls and cannot override grammar or core operations | Paper proof in `proofs/04-language-semantics.md`; parser, precedence, and shadowing tests in Rust |
| D-LANG-IMPORT-1 / T-LANG-IMPORT-1 | Definition / Proved | Relative function-library imports are deterministic, cycle-safe, duplicate-safe, and location-preserving | Paper proof in `proofs/04-language-semantics.md`; import graph tests in Rust |
| T-LANG-ZERO-1 | Proved | A closed finite equality is accepted exactly when its residual is native zero | Paper proof in `proofs/04-language-semantics.md`; evaluator/VM agreement and application witnesses are tested |
| L-LANG-* / T-LANG-* | Proved | Interpreter, compiler, optimizer, and serialization algorithms preserve 1.0 semantics | Proved on paper in `proofs/04-language-semantics.md`; implementation tested, not formally verified |
| D-FLANG-1 / D-FLANG-2 | Definition | Domain-neutral source functions and finite self-modeling pattern graphs | Native Space 1.0 defined in `language/SPEC.md` and `proofs/14-analytic-language-semantics.md` |
| T-FLANG-PURE-1 / T-FLANG-SELF-1 / T-FLANG-NAME-1 / T-FLANG-LOC-1 / T-ZERO-ONLY-1 | Proved | Function names have no hidden semantics, direct and mutual self-reference remain finite graph structure, emitted steps stay in the four-operation core, locations survive, and exact state proofs have one zero goal | Proved on paper in `proofs/14-analytic-language-semantics.md`; implementation tested, not formally verified |
| D-BLOGIC-1 / D-BLOGIC-2 | Definition | Separate finite Boolean formulas, valuations, and truth-table certificates | Native Space 1.0 defined in `language/SPEC.md` and `proofs/15-boolean-logic-kernel.md` |
| T-BLOGIC-* | Proved | Finite propositional soundness, completeness, counterexamples, and certificate reconstruction | Proved on paper in `proofs/15-boolean-logic-kernel.md`; no quantifiers, induction, native equality, or analytic proof rules |
| D-FLOW-1 / C-FLOW-1 | Definition | Multiplication-generated native flow and scale-orientation flow camera | Defined in `04-native-flow-camera.md`; analytic extension, not 1.0 core |
| L-FLOW-* / T-FLOW-* | Proved | Native flow composition and coordinate identification | Proved on paper in `proofs/05-native-flows.md` under A-ODE-1 |
| D-BIN-1 / D-BIN-2 | Definition | Natural coefficients in scalar/state carriers and recursive binomial coefficients | Defined in `proofs/06-native-binomial.md` |
| L-BIN-1 | Proved | Finite binomial theorem for native states | Proved on paper in `proofs/06-native-binomial.md` |
| D-CYCLIC-1 / C-CYCLIC-1 / D-DFT-1 / C-DFT-1 | Definition | One-direction cyclic fold and finite Fourier cameras | Defined in `05-finite-convolution-fourier-camera.md` |
| L-ROOT-1 / L-ORTH-1 / T-CYCLIC-1 / T-DFT-1 / T-DFT-2 | Proved | Finite root, cyclic-fold, DFT inverse, and convolution laws | Proved on paper in `proofs/07-finite-fourier.md` |
| D-MELLIN-* / C-MELLIN-1 | Definition | Weighted-depth flow character and finite scalar camera | Defined in `06-finite-mellin-character-camera.md`; no primes or infinite sums |
| L-MELLIN-* / T-MELLIN-* | Proved | Finite character additivity, multiplicativity, coordinates, and loss | Proved on paper in `proofs/08-finite-mellin-character.md` |
| D-PRIME-* / D-NPRIME-1 / D-PI-1 / C-PRIME-LINE-1 / D-VAL-1 / D-ENC-1 / D-DEC-1 / D-BIRTH-1 | Definition | Ordered primes, intrinsic native prime atoms, finite prime count, exact integer-line camera, birth index, valuation depth, integer encoding, and wrapped birth orientation | Defined in `07-native-prime-system.md` |
| L-BEZ-1 / L-EUCLID-1 / T-FTA-1 / T-PRIME-INF-1 | Proved | Conventional finite prime factorization foundation | Proved on paper in `proofs/09-prime-factorization.md` from A-INT-1 |
| T-PI-1 | Proved | The mathematical prime-count camera equals observation index at prime-value camera outputs | Proved in `proofs/09-prime-factorization.md`; no runtime prime-count command or complexity improvement is claimed |
| L-VAL-1 / T-ENC-1 / T-NPRIME-1 / T-PRIME-LINE-1 / T-BIRTH-1 | Proved | Native integer monoid encoding, exact INDEX/number-line/classical prime correspondence, and birth/depth separation | Proved on paper in `proofs/09-prime-factorization.md` |
| C-BIRTH-HELIX-1 / C-PRIME-HELIX-1 / C-PRIME-CONE-1 | Definition | Flat indexed/prime-value helix cameras and quadratic cone camera of the same typed prime object | Defined in `07-native-prime-system.md` |
| T-PRIME-HELIX-1 | Proved | Indexed primes form an injective quarter-turn helix; prime-value height is an injective axial deformation; the cone loses opposite sign | Proved in `proofs/09-prime-factorization.md` |
| T-PRIME-BRAID-1 / T-PRIME-BRAID-PROJ-1 | Proved | Three indexed orientation tracks form a disjoint cable; classical flattening creates a precisely described apparent crossing | Proved in `proofs/09-prime-factorization.md` |
| D-ARITH-* / D-DIR-1 / C-ARITH-1 | Definition | Finite-support arithmetic functions, Dirichlet convolution, and prime-index coefficient lift | Defined in `08-finite-arithmetic-functions.md` |
| L-DIR-1 / T-ARITH-1 / T-DIR-1 / T-DIR-CHAR-1 | Proved | Finite-support closure, native ring isomorphism, and finite character law | Proved on paper in `proofs/10-dirichlet-convolution.md`; no infinite-series claim |
| D-COMP-* / D-ARITH-INF-1 / C-ARITH-INF-1 | Definition | Formal coefficientwise completion and all arithmetic functions | Defined in `09-formal-completion-and-euler-boundary.md`; finite runtime unchanged |
| L-COMP-LOC-1 / T-COMP-1 / T-DIR-INF-1 | Proved | Local finiteness, completed ring laws, and unrestricted arithmetic-function isomorphism | Proved on paper in `proofs/11-formal-completion-and-euler-product.md` without analytic sums |
| D-GEO-INF-1 / D-ZETA-PATTERN-1 / T-ZETA-PATTERN-1 | Definition / Proved | Formal prime geometric factors and one generative coefficient-one zeta pattern | Defined and proved as one symbolic law in `09-formal-completion-and-euler-boundary.md` |
| L-GEO-INF-1 / T-EULER-F-1 | Proved | Formal geometric inverse and coefficientwise Euler factorization | Proved on paper in `proofs/11-formal-completion-and-euler-product.md`; not scalar convergence |
| A-CNS-1 / A-LOG-1 / A-HOL-1 | Axiom | Complete scalar-series substrate, positive logarithm/power laws, and standard finite-dimensional complex analysis | Accepted only for the Stage 7 analytic extension in `10-analytic-zeta-camera.md` |
| D-SUM-1 / C-AFLAT-1 / C-COMP-EVAL-1 / D-PLOG-1 / D-ZCHAR-1 | Definition | Default analytic weighted stack, explicit aggregate camera, and prime-log zeta character | Defined in `10-analytic-zeta-camera.md` |
| T-ZETA-FLAT-1 | Proved | The zeta character keeps the indexed weighted stack lossless before the explicit scalar aggregation | Proved in `proofs/12-analytic-zeta.md`; aggregate zero need not mean coefficientwise zero |
| T-CAM-COMP-1 / L-ZETA-WEIGHT-1 / L-ZETA-COORD-1 / L-P-SERIES-1 / T-ZETA-CONV-1 | Proved | Completed-camera product law and zeta-series convergence on Re(s) > 1 | Proved on paper in `proofs/12-analytic-zeta.md`; not proof-assistant checked |
| T-ZETA-EULER-1 | Reproduced | Native zeta series equals the ordered Euler camera on Re(s) > 1 | Reproduced in `proofs/12-analytic-zeta.md`; no continuation or zero claim |
| C-PLOG-HELIX-1 / L-PLOG-HELIX-1 | Definition / Proved | Prime-log height is the analytic scale camera of the same flat birth helix and remains injective | Defined in `10-analytic-zeta-camera.md`; proved in `proofs/12-analytic-zeta.md` |
| D-AMULT-1 | Definition | Analytic zero multiplicity is native MULTIPLY depth | Defined in `11-native-rh-geometry.md` |
| T-AMULT-1 | Proved | Native analytic multiplicity matches conventional zero multiplicity | Proved in `proofs/13-native-rh-geometry.md` |
| C-XI-REF-1 / L-XI-ZETA-ZERO-1 | Definition / Reproduced | Conventional completed-xi reference lift; xi and zeta have the same zeros in the open critical strip | Defined as the reconstruction target in `11-native-rh-geometry.md`; the zero correspondence is reproduced in `proofs/13-native-rh-geometry.md` |
| O-XI-NATIVE-1 | Open obligation | Construct completed xi operations-first with camera equivalence and reflection | Closes only with a source-defined construction and complete domain, coordinate-equivalence, and reflection proofs; analytic integration/theta/gamma dependencies remain open |
| T-ZETA-PAIR-CONV-1 / T-ZETA-PAIR-IDENTITY-1 / L-ZETA-PAIR-DENOM-1 | Proved | The paired series converges on its stated domain, agrees with the half-plane zeta camera, and has nonzero denominator in the strip | Proved in `proofs/12-analytic-zeta.md`; not proof-assistant checked |
| T-ZETA-STRIP-1 | Reproduced | The paired quotient reproduces the classical analytic continuation of zeta in the open critical strip | Reproduced in `proofs/12-analytic-zeta.md`; classical holomorphic substrate A-HOL-1 is explicit |
| C-ZETA-CLASSICAL-COORDS-1 | Definition | Place the paired zeta value on the shared classical real and rotated-imaginary axes | Defined in `10-analytic-zeta-camera.md` |
| T-ZETA-ZERO-PAIR-1 / T-ZETA-OUTPUT-RESIDUAL-1 / T-ZETA-OUTPUT-FRAME-1 | Proved | Paired zeta zero, empty classical residual, and zero-filled classical frame have the same zero locus | Proved in `proofs/12-analytic-zeta.md`; this does not imply native RE zero |
| D-NATIVE-RE-1 / T-NATIVE-RE-1 | Definition / Proved | The one symbolic reflected-equality pattern has one sign off the critical line and is the zero law exactly on it | Proved from one symbolic observation law in `proofs/18-finite-prime-pattern-route.md`; no native prime range |
| D-RH-REFL-1 | Definition | Critical reflection sends the classical coordinate $\sigma+it$ to $(1-\sigma)+it$ and negates the centered real coordinate | Defined in `11-native-rh-geometry.md` |
| D-CLASSIC-P-1 / T-CLASSIC-P-1 | Definition / Proved | `classic_p` and `classic_i` are exactly the real and imaginary coefficient cameras under $\kappa$ | Full basis proof in `proofs/22-projection-and-axis-subtraction.md` |
| C-CENTERED-RE-PERSPECTIVE-1 / T-CENTERED-RE-PERSPECTIVE-1 | Definition / Proved | `centered_re_perspective` translates the real input axis by half of the multiplicative identity, the unique center where reflection becomes negation; it is zero exactly at $\mathrm{Re}(s)=1/2$ | Defined in `11-native-rh-geometry.md`; proved in `proofs/13-native-rh-geometry.md` |
| D-ZETA-RE-QUADRATIC-CAMERAS-1 / T-ZETA-RE-ROTATION-1 / T-ZETA-RE-POSITION-1 / T-ZETA-RE-WRAP-1 / T-ZETA-RE-POSITION-CANCEL-1 / T-ZETA-RE-VERTEX-PATH-1 | Definition / Proved | Zeta's +45-degree projector and RE's perpendicular projector are idempotent, orthogonal, and complete; each places the multiplicative identity at quadratic position one half; together they reconstruct one wrapped native state and every retained INDEX vertex; their oriented comparison positions cancel | Full runnable zero proofs in the `examples/zeta_re_perspective_*.ns` files and `examples/zeta_re_vertex_path.ns`; written proof in `proofs/23-zeta-re-perspectives.md` |
| D-CLASSICAL-SOURCE-AXIS-1 / T-ZETA-RE-CLASSICAL-AXIS-ROTATION-1 | Definition / Proved | The classical source is one-dimensional, not a two-axis plane; after camera zero-fill, the RE view is exactly the zeta view rotated clockwise by 90 degrees and their zero loci agree | Full runnable zero proof in `examples/zeta_re_classical_axis_rotation.ns`; written proof in `proofs/23-zeta-re-perspectives.md` |
| D-NATIVE-PERSPECTIVE-ZERO-1 / T-NATIVE-PERSPECTIVE-ZERO-1 | Definition / Proved | With both camera wrappers retained, zero of one view is equivalent to zero of the perpendicular view of one legal classical-source value | Proved by the invertible quarter-turn in `proofs/23-zeta-re-perspectives.md`; camera theorem, not RH |
| K-PRIME-FLATTENING-1 | Conjecture | The complete recursive multiplicative prime pattern cannot be reconstructed from the flattened classical camera without reintroducing equivalent native structure | Stated with its completion condition in `proofs/23-zeta-re-perspectives.md`; general noninjectivity is proved, but non-factorization on the generative prime-pattern image is not |
| T-ZETA-RE-GLOBAL-FACTOR-NO-1 | Proved negative result | The invertibly rotated one-dimensional views cannot globally decode as actual zeta zero and centered-RE zero | Proved at $s=1/2$ in `proofs/24-rh-projection-factorization-audit.md`; this rules out the flattened-camera proof, not every possible classical proof |
| D-SHIFTED-RE-CENTER-1 / T-REFLECTION-CENTER-UNIQUE-1 | Definition / Proved | A fixed RE center c is compatible with multiplicative reflection exactly when c=1/2 | Universal paper proof and exact finite Native Space witness in `proofs/25-shifted-zero-center.md` and `examples/reflection-center.ns` |
| T-REVERSE-HALF-NO-1 | Proved negative result | RE zero on the half-centered line does not imply zeta zero | Proved at s=1/2 from the positive paired numerator in `proofs/25-shifted-zero-center.md` |
| T-OFFCENTER-RH-NO-1 | Reproduced consequence | Every fixed center other than one half makes the zeta-zero-to-RE-zero statement false | Uses the reproduced classical theorem that infinitely many zeta zeros lie on the critical line; the remaining half-centered direction is exactly K-RH-1 |
| C-RE-CLASSICAL-COORDS-1 / T-RE-OUTPUT-RESIDUAL-1 / T-RE-OUTPUT-FRAME-1 | Definition / Proved | Place native RE on the shared classical axes; its sparse residual is empty and its total frame is zero exactly at native RE zero | Defined in `11-native-rh-geometry.md`; proved in `proofs/22-projection-and-axis-subtraction.md` |
| D-FOUR-ORIENT-FUNCTIONS-1 / T-FOUR-AXIS-CANCEL-1 / T-CLASSICAL-FOUR-CAMERA-1 | Definition / Proved | Four transparent functions expose $(-i,i,1,-1)$; opposite signed lanes cancel by ADD | Proved in `proofs/22-projection-and-axis-subtraction.md` |
| D-AXIS-SUBTRACT-1 / T-AXIS-ADD-CANCEL-1 / T-AXIS-SUBTRACT-1 | Definition / Proved | Signed axes cancel by ADD; subtraction is ORIENT$_2$ followed by ADD and is zero exactly for equal axes | Proved in `proofs/22-projection-and-axis-subtraction.md` |
| D-DEPHASE-1 | Definition | Apply a possibly different inverse orientation unit to each retained INDEX coefficient | Distinct from one common native observer ORIENT |
| T-OPPOSITE-PAIR-1 | Proved | Birth orientations k and k+2 are opposite; their weighted terms cancel exactly when their gains agree | Positions alone do not imply weighted cancellation |
| T-ALL-PERSPECTIVES-1 | Proved | All common native observer views are invertible rotations of one zero equation | They preserve cancellation but do not produce coefficientwise zero equations |
| T-DEPHASE-STACK-1 | Proved | Strandwise inverse phase MULTIPLY preserves zero of the full indexed stack | INDEX labels must remain present |
| T-DEPHASE-AGG-NONCOMMUTE-1 | Proved negative result | Aggregate zero is not generally preserved by strandwise dephasing | Finite two-strand native counterexample in `proofs/19-zeta-zero-fiber-audit.md` |
| T-ZERO-FIBER-FACTOR-1 | Proved | A finite diagonal transform preserves every ADD-zero fiber exactly when all strand multipliers agree | Proved in `proofs/19-zeta-zero-fiber-audit.md` |
| T-TWO-PROJECTION-FIBER-1 | Proved negative result | Two scalar-zero projections do not identify the retained indexed states | Proved by finite counterexample in `proofs/19-zeta-zero-fiber-audit.md` |
| O-ZETA-RE-CLASSICAL-PATTERN-1 | Open wrapped-pattern relation | After applying $P_Z$ only to zeta and $P_R$ only to RE, derive that wrapped zeta zero forces wrapped RE zero | Closes only with a proof for every nontrivial critical-strip zeta zero; equivalent to K-RH-1, while the proved camera poses do not close it |
| C-RH-3D-1 | Definition | Default lossless two-line xi transform plus centered symmetry displacement | Defined in `11-native-rh-geometry.md` |
| T-RH-3D-EQ-1 | Reproduced | RH is equivalent to the centered 3D axis-exclusion statement | Equivalence proved in `proofs/13-native-rh-geometry.md`; does not prove the exclusion |
| D-AXIS-PROJ-1 | Definition | Native real and rotated-imaginary projections onto the signed 1/-1 axis | Defined in `11-native-rh-geometry.md` |
| T-AXIS-CANCEL-1 | Proved | A 2D native zero is exactly two signed 1D cancellations; birth orientations split by index mod 4 | Proved in `proofs/13-native-rh-geometry.md` |
| T-RH-FLAT-1 | Proved | The default centered RH transform is the two signed-axis projections and exactly reconstructs the xi value | Proved in `proofs/13-native-rh-geometry.md`; no zero-exclusion claim |
| D-DUAL-CHAR-1 / C-DUAL-FLAT-1 / C-DUAL-CONE-1 | Definition | Two reflected prime-log projections, their default signed flat stack, and optional mismatch-cone view | Defined in `11-native-rh-geometry.md` |
| T-DUAL-ALIGN-1 / L-DUAL-CONE-1 | Proved | Reflected prime strands align exactly on the critical line in the default flat stack; the cone is an optional same-zero-locus corollary | Proved in `proofs/13-native-rh-geometry.md` |
| O-ZERO-ALIGN-1 | Open obligation | Construct one completed-xi-to-flat-stack transform and prove its zero-fiber law | Closes only after O-XI-NATIVE-1 and a proof of the displayed zero-fiber implication for every nontrivial critical-strip input |
| T-RH-DIRECT-EQ-1 | Reproduced equivalence | The zeta-zero-to-native-RE implication is equivalent to K-RH-1 | Proved in `proofs/17-direct-zeta-rh-reduction.md`; no reduction in difficulty |
| T-RH-PATTERN-CONSEQUENCE-1 | Proved conditional consequence | O-ZETA-RE-CLASSICAL-PATTERN-1 makes classical RH a corollary of native RE | Proved in `proofs/18-finite-prime-pattern-route.md`; its stated premise remains open |
| T-RH-REDUCE-1 | Proved | Operations-first xi plus zero-to-prime alignment implies K-NATIVE-XI-1 and, through the required xi camera equivalence, K-RH-1 | Conditional reduction proved in `proofs/13-native-rh-geometry.md`; open premises not promoted |
| K-RH-1 | Conjecture | Every nontrivial classical zeta zero has real part one half | Precisely stated in `11-native-rh-geometry.md`; unproved |
| K-NATIVE-XI-1 | Conjecture | The future operations-first completed-xi coordinate has no off-symmetry-plane nontrivial zero | Separate native transfer target in `11-native-rh-geometry.md`; depends on O-XI-NATIVE-1 |
| R-SOS-1 | Reproduced | Sum-of-two-squares product identity from native squared-size multiplicativity | Reconstructed in `reconstructions/01-sum-of-two-squares-product.md`; invariant-revealing, not a novelty claim |
| R-GEO-1 | Reproduced | Finite geometric-series identity from INDEX-depth telescoping | Reconstructed in `reconstructions/02-finite-geometric-series.md`; no infinite-series claim |
| R-PYT-1 | Reproduced | Forward Euclid Pythagorean identity from native squared-size multiplicativity | Reconstructed in `reconstructions/03-pythagorean-cone.md`; no primitive-triple classification claim |
| R-EXP-1 | Reproduced | Exponential and angle-addition laws from multiplication-generated native flow | Reconstructed in `reconstructions/04-exponential-and-angle-addition.md`; imports explicit finite ODE substrate |
| R-BIN-1 | Reproduced | Finite binomial theorem in native depth coordinates | Reconstructed in `reconstructions/05-binomial-theorem.md`; finite commuting case only |
| R-CALC-1 | Reproduced | Power and reciprocal derivative rules in native operations | Reconstructed in `reconstructions/06-power-and-inverse-derivatives.md`; finite-dimensional and inverse excludes zero |
| R-DFT-1 | Reproduced | Finite DFT convolution theorem through an explicit cyclic depth quotient | Reconstructed in `reconstructions/07-finite-convolution-theorem.md`; no FFT or infinite-transform claim |
| R-MELLIN-1 | Reproduced | Finite Mellin/Dirichlet character product law | Reconstructed in `reconstructions/08-finite-mellin-character-product.md`; its finite boundary is later extended by R-ZETA-1 |
| R-ZETA-1 | Reproduced | Zeta Dirichlet series equals its ordered Euler product on Re(s) > 1 | Reconstructed in `reconstructions/10-zeta-euler-half-plane.md`; no continuation or RH result |
| H-MAT-1 / H-LLM-1 / H-DYN-1 / H-NBODY-1 / H-NS-1 / H-PROT-1 / H-PL-1 | Hypothesis | Falsifiable application expectations | Defined in `../applications`; no experiment executed |
| E-MAT-1 / E-LLM-1 / E-DYN-1 / E-NBODY-1 / E-NS-1 / E-PROT-1 / E-PL-1 | Experiment | Planned experiment or implementation procedures corresponding to those hypotheses | Draft procedures; no observed result and not locked until the evaluation protocol is complete |

This register is intentionally modest. The existence of a coherent definition
does not prove usefulness, novelty, compression, or explanatory power.
