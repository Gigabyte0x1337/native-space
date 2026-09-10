// SPDX-License-Identifier: AGPL-3.0-or-later
//! Checks native ALU source; Rust supplies test inputs and an independent oracle.
//!
//! No production Rust ALU is registered or called by the native program. The
//! finite bit domain makes exhaustive checking possible for this circuit width.

use native_space_language::core::{Expr, NativeScalar, NativeState};
use native_space_language::{Document, bytecode, core, expand_source};

const SOURCE: &str = include_str!("fixtures/alu.ns");
const PROOF: &str = include_str!("fixtures/alu-proof.ns");

fn definitions() -> &'static str {
    SOURCE.split("# Demo:").next().unwrap().trim_end()
}

fn scalar(value: u8) -> NativeState {
    NativeState::scalar(NativeScalar::from_text(&value.to_string(), "0").unwrap())
}

fn inputs(a: u8, b: u8, opcode: u8) -> Vec<NativeState> {
    (0..4)
        .map(|i| scalar((a >> i) & 1))
        .chain((0..4).map(|i| scalar((b >> i) & 1)))
        .chain((0..3).map(|i| scalar((opcode >> i) & 1)))
        .collect()
}

fn input_text(a: u8, b: u8, opcode: u8) -> String {
    (0..4)
        .map(|i| ((a >> i) & 1).to_string())
        .chain((0..4).map(|i| ((b >> i) & 1).to_string()))
        .chain((0..3).map(|i| ((opcode >> i) & 1).to_string()))
        .collect::<Vec<_>>()
        .join(", ")
}

// This oracle deliberately uses ordinary machine-word operators, not the native
// gate equations under test. It defines the entire nine-field result.
fn expected(a: u8, b: u8, opcode: u8) -> NativeState {
    let (value, carry, overflow) = match opcode {
        0 => {
            let value = (a + b) & 15;
            (value, a + b > 15, ((a ^ value) & (b ^ value) & 8) != 0)
        }
        1 => {
            let value = a.wrapping_sub(b) & 15;
            (value, a >= b, ((a ^ b) & (a ^ value) & 8) != 0)
        }
        2 => (a & b, false, false),
        3 => (a | b, false, false),
        4 => (a ^ b, false, false),
        5 => ((!a) & 15, false, false),
        6 => ((a << 1) & 15, a & 8 != 0, false),
        7 => (a >> 1, a & 1 != 0, false),
        _ => unreachable!("tests supply three-bit opcodes"),
    };
    let fields = [
        value & 1,
        (value >> 1) & 1,
        (value >> 2) & 1,
        (value >> 3) & 1,
        u8::from(carry),
        u8::from(value == 0),
        u8::from(value & 8 != 0),
        u8::from(overflow),
        value,
    ];
    fields
        .into_iter()
        .enumerate()
        .fold(NativeState::zero(), |state, (i, v)| {
            state.add(
                &scalar(v)
                    .index_power(u64::try_from(i + 1).unwrap(), 1)
                    .unwrap(),
            )
        })
}

#[test]
fn every_alu_operand_and_opcode_runs_in_native_source() {
    let program = core::parse(SOURCE, "alu.ns").unwrap();
    let alu = core::exact_function(&program, "alu").unwrap();
    let mut checked = 0;
    for opcode in 0..8 {
        for a in 0..16 {
            for b in 0..16 {
                assert_eq!(
                    alu.apply(&inputs(a, b, opcode)).unwrap(),
                    expected(a, b, opcode),
                    "opcode {opcode}, A {a}, B {b}"
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 2048);
    println!(
        "All 2048 native ALU input combinations matched, including every output bit and flag."
    );
}

#[test]
fn every_gate_and_full_adder_truth_table_is_native() {
    let program = core::parse(SOURCE, "alu.ns").unwrap();
    for a in 0..2 {
        assert_eq!(
            core::exact_function(&program, "bit_not")
                .unwrap()
                .apply(&[scalar(a)])
                .unwrap(),
            scalar(1 - a)
        );
        for b in 0..2 {
            for (name, value) in [("bit_and", a & b), ("bit_or", a | b), ("bit_xor", a ^ b)] {
                assert_eq!(
                    core::exact_function(&program, name)
                        .unwrap()
                        .apply(&[scalar(a), scalar(b)])
                        .unwrap(),
                    scalar(value)
                );
            }
            for c in 0..2 {
                for (name, value) in [
                    ("sum_bit", (a + b + c) & 1),
                    ("carry_bit", (a + b + c) >> 1),
                ] {
                    assert_eq!(
                        core::exact_function(&program, name)
                            .unwrap()
                            .apply(&[scalar(a), scalar(b), scalar(c)])
                            .unwrap(),
                        scalar(value)
                    );
                }
            }
        }
    }
}

#[test]
fn native_proof_is_self_contained_current_and_zero_in_both_engines() {
    assert!(
        PROOF.starts_with(definitions()),
        "proof must contain the current full circuit"
    );
    let program = core::parse(PROOF, "alu-proof.ns").unwrap();
    assert!(core::interpret(&program).unwrap().is_zero());
    let compiled = bytecode::lower(&program).unwrap();
    assert!(bytecode::execute(&compiled).unwrap().is_zero());
}

#[test]
fn compiled_alu_contains_native_operations_and_matches_all_inputs() {
    let program = core::parse(SOURCE, "compiled-alu.ns").unwrap();
    let artifact = native_space_language::compiled::compile(&program).unwrap();
    let artifact =
        native_space_language::compiled::Artifact::from_data(&artifact.to_data()).unwrap();
    let function = artifact.function("alu").unwrap();
    for opcode in 0..8 {
        for a in 0..16 {
            for b in 0..16 {
                // All inputs use one saved/reloaded function graph. Each retained
                // result must still contain operations and replay in the stack VM.
                let arguments = inputs(a, b, opcode)
                    .iter()
                    .map(|state| {
                        native_space_language::strand::execution::Value::State(
                            native_space_language::retained::State::from_projection(state),
                        )
                    })
                    .collect();
                let state = function.call(arguments).unwrap().native().unwrap();
                let compiled = bytecode::lower_state(
                    &state,
                    "compiled-alu.ns",
                    core::Goal::Emit,
                    core::OutputKind::Pattern,
                )
                .unwrap();
                for opcode in [
                    bytecode::Opcode::Add,
                    bytecode::Opcode::Multiply,
                    bytecode::Opcode::Phase,
                    bytecode::Opcode::Index,
                    bytecode::Opcode::Load,
                ] {
                    assert!(compiled.instructions.iter().any(|i| i.opcode == opcode));
                }
                assert_eq!(
                    bytecode::execute(&compiled).unwrap(),
                    expected(a, b, opcode)
                );
            }
        }
    }
    println!("All 2048 compiled ALU input combinations matched in the native VM.");
}

fn core_only(expression: &Expr) -> bool {
    match expression {
        Expr::Literal { .. } | Expr::Reference { .. } => true,
        Expr::Add { operands, .. } | Expr::Multiply { operands, .. } => {
            operands.iter().all(core_only)
        }
        Expr::Phase { value, .. } | Expr::Index { value, .. } => core_only(value),
        _ => false,
    }
}

#[test]
fn expanded_alu_has_no_host_alu_or_noncore_computation() {
    let program = core::parse(SOURCE, "alu.ns").unwrap();
    let expanded = expand_source(&Document::State(program)).unwrap();
    let expanded = core::parse(&expanded, "expanded-alu.ns").unwrap();
    assert!(expanded.functions.is_empty());
    assert!(
        expanded
            .bindings
            .iter()
            .all(|binding| core_only(&binding.value))
    );
    assert!(core_only(&expanded.result));
    assert_eq!(core::interpret(&expanded).unwrap(), expected(15, 1, 0));
}

#[test]
fn reflected_alu_executes_the_native_circuit() {
    let source = format!(
        "{}\noutput (alu)({}) as pattern",
        definitions(),
        input_text(7, 1, 0)
    );
    let program = core::parse(&source, "reflected-alu.ns").unwrap();
    assert_eq!(core::interpret(&program).unwrap(), expected(7, 1, 0));
    assert_eq!(
        bytecode::execute(&bytecode::lower(&program).unwrap()).unwrap(),
        expected(7, 1, 0)
    );
}
