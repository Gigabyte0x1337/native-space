<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Optional GPU backend

## Requirement

CPU-only users must not compile the GPU dependency tree. GPU execution remains
an explicit backend and must never silently fall back to CPU.

## Build contract

The default feature set is CPU-only:

```powershell
cargo build --manifest-path language/runtime/Cargo.toml --release --bin native-space
```

Enable exact GPU batch execution explicitly:

```powershell
cargo build --manifest-path language/runtime/Cargo.toml --features gpu --release --bin native-space
```

The `gpu` feature enables only the optional `wgpu` and `bytemuck`
dependencies. The public Rust API and `--backend gpu` CLI value remain visible
without the feature. Requesting GPU execution from a CPU-only binary returns
`NSG001` with the build instruction; it does not execute on CPU.

Official release binaries are built with `--features gpu`. The feature exists
so library users and local builders can choose a smaller CPU-only compilation.

## Preserved invariant

Feature selection changes backend availability, not Native Space semantics.
The enabled backend still accepts only its documented exact signed-32-bit
subset, detects overflow, and rejects unsupported coordinates and operations.

The GPU calculates classical observations. The host constructs the native
operation graphs for those same steps and retains inputs and scopes for
feedback. Primitive graph construction is lazy; explicit staged reflection routing
remain host work. This is not a GPU-resident native graph or a claim of faster
execution. `GpuBatchResult.results` contains device observations and `states`
contains the retained states; batch JSON exposes both separately.

## Verification

The default test suite checks the unavailable-backend diagnostic without
compiling GPU dependencies. The all-features suite compiles and tests the real
shader backend. Hardware execution is a separate opt-in test and fails rather
than silently skipping if no adapter is available:

```sh
cargo test --manifest-path language/runtime/Cargo.toml --features gpu --lib hardware_observations_match_cpu -- --ignored --nocapture
```

It checks device values against CPU observations and compares complete native
graphs for distinct cancelled inputs after three feedback steps.
