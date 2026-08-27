<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Classical Frequency Synthesis Experiment

## Requirement

The runtime should test whether a finite native program can be replaced by a
smaller frequency description when only one lossy classical projection is
observable. This is compiler tooling, not a fifth Native Space operation.

## Preserved invariant

For a requested finite INDEX window, the synthesizer first evaluates the exact
Native Space source. It then converts each exact coefficient to the classical
complex `f64` camera. A generated frequency program is accepted only when a
fresh replay keeps every projected sample within the caller's declared maximum
absolute error.

Therefore acceptance means:

```text
maximum distance(classical(original), classical(frequency replay)) <= bound
```

It does not mean that the two native states are equal. INDEX types, exact
rational coefficients, and distinctions hidden by the camera need not survive.

## Algorithm

1. Read exactly one finite window of positive, one-depth INDEX directions.
2. Project its exact coefficients to finite classical real/imaginary pairs.
3. Compute the normalized finite discrete frequency coefficients.
4. Order modes by projected power, with frequency bin as the deterministic tie
   breaker.
5. Add modes until replay satisfies the declared error bound.
6. Reconstruct the selected program again and reject it if verification fails.

The first implementation uses a direct quadratic transform and accepts at most
4,096 samples. This keeps the experiment bounded and auditable. Replacing it
with an FFT requires separate agreement tests; the mathematical output contract
does not change.

## Performance boundary

For `N` samples and `M` retained modes:

- synthesis currently costs quadratic work in `N`;
- replaying the entire finite window costs work proportional to `N * M`;
- calculating one requested position costs work proportional to `M`;
- keeping all `N` modes gives no compression;
- a small `M` can help repeated or random-access replay, but this implementation
  does not yet claim a measured speedup.

Storing the original `N` outputs can still be faster than calculating modes.
The frequency artifact matters when the original computation is expensive,
the selected projection is the required observable, and a small mode set is
reused enough to repay synthesis cost.

## Accepted tradeoff

This design deliberately permits classical floating-point loss because the
requested target is already a lossy classical camera. The loss is never hidden:
the artifact records the camera, requested bound, observed error, retained
modes, and successful verification. Native equality and continuation beyond the
finite window remain unclaimed.

## Runnable experiment

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --bin native-space -- frequency examples/frequency-observations.ns --samples 16 --maximum-error 1e-12
```

The example is one exact quarter-turn mode sampled sixteen times. The current
runtime retains one mode at bin 4 and verifies all sixteen classical outputs.
