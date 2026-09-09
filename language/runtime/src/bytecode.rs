// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Version 1 stack bytecode and independent exact-state virtual machine.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::core::{
    Goal, LanguageError, NativeState, OutputKind, Program, Span, is_canonical_phase,
};
use crate::retained::Operation;

pub const BYTECODE_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Opcode {
    PushZero,
    PushOne,
    PushScalar,
    Load,
    Store,
    Add,
    Multiply,
    Phase,
    Index,
    /// Select indexed coordinates at evaluation time, retaining the source graph.
    Camera,
    /// Preserve scope inputs; this is a storage instruction, not an arithmetic primitive.
    Retain,
    Halt,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Operand {
    Integer(i64),
    Index {
        direction: u64,
        depth: u64,
    },
    Scalar {
        coordinates: crate::retained::ScalarData,
    },
    Camera {
        from: u64,
        to: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Option<Operand>,
    pub span: Option<Span>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BytecodeProgram {
    pub version: u64,
    pub source_name: String,
    pub goal: Goal,
    pub output_kind: OutputKind,
    pub slot_names: Vec<String>,
    pub instructions: Vec<Instruction>,
}

impl BytecodeProgram {
    #[must_use]
    pub fn to_data(&self) -> Value {
        json!({
            "schema": "native-space-bytecode",
            "version": self.version,
            "source_name": self.source_name,
            "goal": self.goal,
            "output_kind": self.output_kind,
            "slot_names": self.slot_names,
            "instructions": self.instructions,
        })
    }

    /// Decode schema-1 bytecode.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed data or an unsupported schema.
    pub fn from_data(value: &Value) -> Result<Self, String> {
        let root = value.as_object().ok_or("bytecode root must be an object")?;
        if root.get("schema").and_then(Value::as_str) != Some("native-space-bytecode")
            || root.get("version").and_then(Value::as_u64) != Some(BYTECODE_VERSION)
        {
            return Err("unsupported Native Space bytecode schema or version".into());
        }
        serde_json::from_value(json!({
            "version": root.get("version"),
            "source_name": root.get("source_name"),
            "goal": root.get("goal"),
            "output_kind": root.get("output_kind"),
            "slot_names": root.get("slot_names"),
            "instructions": root.get("instructions"),
        }))
        .map_err(|error| error.to_string())
    }
}

/// Compile a native operation graph, preserving branches and camera read dependencies.
///
/// Source functions construct the graph; scalar arithmetic stays behind the
/// classical camera. Staged discovery may observe values while constructing it;
/// indexed reads remain deferred. The VM independently replays every record.
///
/// # Errors
///
/// Returns a name-analysis diagnostic or a slot-capacity error.
///
/// # Panics
/// Panics only if an internal graph invariant is violated after validation.
pub fn compile(program: &Program) -> Result<BytecodeProgram, LanguageError> {
    let state = crate::retained::interpret(program)?;
    let mut compiler = Compiler::default();
    for step in state.plan() {
        let slot = i64::try_from(compiler.slot_names.len()).map_err(|_capacity_error| {
            LanguageError(crate::core::Diagnostic {
                code: "NSC001".into(),
                message: "program has too many bindings".into(),
                source_name: program.source_name.clone(),
                span: step.span,
            })
        })?;
        for input in &step.inputs {
            compiler.load(*input, step.span);
        }
        match step.operator {
            Operation::Scalar { coordinates } => {
                compiler.emit(
                    Opcode::PushScalar,
                    Some(Operand::Scalar { coordinates }),
                    step.span,
                );
            }
            Operation::Add => compiler.emit(Opcode::Add, Some(Operand::Integer(2)), step.span),
            Operation::Multiply => {
                compiler.emit(Opcode::Multiply, Some(Operand::Integer(2)), step.span);
            }
            Operation::Phase { turns } => {
                compiler.emit(Opcode::Phase, Some(Operand::Integer(turns)), step.span);
            }
            Operation::Index { direction, depth } => compiler.emit(
                Opcode::Index,
                Some(Operand::Index { direction, depth }),
                step.span,
            ),
            Operation::Camera { from, to } => compiler.emit(
                Opcode::Camera,
                Some(Operand::Camera { from, to }),
                step.span,
            ),
        }
        if !step.retained.is_empty() {
            for input in &step.retained {
                compiler.load(*input, step.span);
            }
            compiler.emit(
                Opcode::Retain,
                Some(Operand::Integer(
                    i64::try_from(step.retained.len()).expect("allocated graph fits i64"),
                )),
                step.span,
            );
        }
        compiler.slot_names.push(format!("branch_{slot}"));
        compiler.emit(Opcode::Store, Some(Operand::Integer(slot)), step.span);
    }
    compiler.load(compiler.slot_names.len() - 1, program.result.span());
    compiler.emit(Opcode::Halt, None, program.result.span());
    Ok(BytecodeProgram {
        version: BYTECODE_VERSION,
        source_name: program.source_name.clone(),
        goal: program.goal,
        output_kind: program.output_kind,
        slot_names: compiler.slot_names,
        instructions: compiler.instructions,
    })
}

#[derive(Debug, Default)]
struct Compiler {
    slot_names: Vec<String>,
    instructions: Vec<Instruction>,
}

impl Compiler {
    fn emit(&mut self, opcode: Opcode, operand: Option<Operand>, span: Option<Span>) {
        self.instructions.push(Instruction {
            opcode,
            operand,
            span,
        });
    }

    fn load(&mut self, slot: usize, span: Option<Span>) {
        self.emit(
            Opcode::Load,
            Some(Operand::Integer(
                i64::try_from(slot).expect("allocated slot fits i64"),
            )),
            span,
        );
    }
}

/// Execute bytecode and explicitly return its classical projection.
///
/// Use `execute_retained` when feeding the full native result into another step.
///
/// # Errors
///
/// Returns a located VM diagnostic for malformed or invalid bytecode.
pub fn execute(program: &BytecodeProgram) -> Result<NativeState, LanguageError> {
    execute_retained(program).map(|state| state.project().clone())
}

/// Execute bytecode while retaining every primitive's input relationships.
///
/// # Errors
/// Returns a located VM diagnostic for malformed or invalid bytecode.
///
/// # Panics
/// Panics only if an internal stack invariant is violated after validation.
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive opcode match keeps VM behavior closed and auditable"
)]
pub fn execute_retained(
    program: &BytecodeProgram,
) -> Result<crate::retained::State, LanguageError> {
    use crate::retained::State;
    if program.version != BYTECODE_VERSION {
        return Err(vm_error(
            "NSV011",
            "unsupported bytecode version",
            program,
            None,
        ));
    }
    let mut stack = Vec::new();
    let mut slots = vec![None; program.slot_names.len()];
    for instruction in &program.instructions {
        match instruction.opcode {
            Opcode::PushZero => stack.push(State::zero()),
            Opcode::PushOne => stack.push(State::one()),
            Opcode::PushScalar => {
                let Operand::Scalar { coordinates } = required_operand(instruction, program)?
                else {
                    return Err(vm_error(
                        "NSV009",
                        "instruction requires scalar operand",
                        program,
                        instruction.span,
                    ));
                };
                let value = crate::retained::Scalar::from_data(coordinates)
                    .map_err(|message| vm_error("NSV009", &message, program, instruction.span))?;
                stack.push(State::native_scalar(&value));
            }
            Opcode::Load => {
                let slot = required_slot(instruction, program, slots.len())?;
                stack.push(slots[slot].clone().ok_or_else(|| {
                    vm_error("NSV002", "uninitialized slot", program, instruction.span)
                })?);
            }
            Opcode::Store => {
                let slot = required_slot(instruction, program, slots.len())?;
                slots[slot] = Some(stack.pop().ok_or_else(|| {
                    vm_error(
                        "NSV001",
                        "bytecode stack underflow",
                        program,
                        instruction.span,
                    )
                })?);
            }
            Opcode::Add | Opcode::Multiply => {
                let Operand::Integer(arity) = required_operand(instruction, program)? else {
                    return Err(vm_error(
                        "NSV008",
                        "instruction requires integer operand",
                        program,
                        instruction.span,
                    ));
                };
                let arity = usize::try_from(*arity).map_err(|_conversion_error| {
                    vm_error(
                        "NSV003",
                        "invalid operation arity",
                        program,
                        instruction.span,
                    )
                })?;
                if arity < 2 || stack.len() < arity {
                    return Err(vm_error(
                        "NSV001",
                        "bytecode stack underflow or invalid arity",
                        program,
                        instruction.span,
                    ));
                }
                let mut values = stack.split_off(stack.len() - arity).into_iter();
                let mut result = values.next().expect("validated nonempty arity");
                for value in values {
                    result = if instruction.opcode == Opcode::Add {
                        result.add(&value)
                    } else {
                        result.multiply(&value)
                    };
                }
                stack.push(result);
            }
            Opcode::Retain => {
                let Operand::Integer(count) = required_operand(instruction, program)? else {
                    return Err(vm_error(
                        "NSV008",
                        "RETAIN requires an input count",
                        program,
                        instruction.span,
                    ));
                };
                let count = usize::try_from(*count).map_err(|_capacity_error| {
                    vm_error(
                        "NSV003",
                        "invalid retained input count",
                        program,
                        instruction.span,
                    )
                })?;
                if count == 0 || count >= stack.len() {
                    return Err(vm_error(
                        "NSV001",
                        "RETAIN requires a result and its inputs",
                        program,
                        instruction.span,
                    ));
                }
                let inputs = stack.split_off(stack.len() - count);
                let value = stack.pop().expect("validated result below retained inputs");
                stack.push(value.retaining(&inputs));
            }
            Opcode::Phase => {
                let Operand::Integer(turns) = required_operand(instruction, program)? else {
                    return Err(vm_error(
                        "NSV008",
                        "instruction requires integer operand",
                        program,
                        instruction.span,
                    ));
                };
                if !is_canonical_phase(*turns) {
                    return Err(vm_error(
                        "NSV012",
                        "phase turns must be an integer from 0 through 3; retain repeated counts with INDEX",
                        program,
                        instruction.span,
                    ));
                }
                let value = stack.pop().ok_or_else(|| {
                    vm_error(
                        "NSV001",
                        "bytecode stack underflow",
                        program,
                        instruction.span,
                    )
                })?;
                stack.push(value.phase(*turns));
            }
            Opcode::Index => {
                let Operand::Index { direction, depth } = required_operand(instruction, program)?
                else {
                    return Err(vm_error(
                        "NSV010",
                        "instruction requires index operand",
                        program,
                        instruction.span,
                    ));
                };
                let value = stack.pop().ok_or_else(|| {
                    vm_error(
                        "NSV001",
                        "bytecode stack underflow",
                        program,
                        instruction.span,
                    )
                })?;
                stack.push(
                    value.index_power(*direction, *depth).map_err(|message| {
                        vm_error("NSV004", &message, program, instruction.span)
                    })?,
                );
            }
            Opcode::Camera => {
                let Operand::Camera { from, to } = required_operand(instruction, program)? else {
                    return Err(vm_error(
                        "NSV010",
                        "CAMERA requires source and destination directions",
                        program,
                        instruction.span,
                    ));
                };
                let value = stack.pop().ok_or_else(|| {
                    vm_error(
                        "NSV001",
                        "bytecode stack underflow",
                        program,
                        instruction.span,
                    )
                })?;
                stack.push(value.camera(*from, *to));
            }
            Opcode::Halt => {
                if stack.len() != 1 {
                    return Err(vm_error(
                        "NSV005",
                        "HALT requires exactly one result",
                        program,
                        instruction.span,
                    ));
                }
                return stack.pop().ok_or_else(|| {
                    vm_error(
                        "NSV005",
                        "HALT result disappeared",
                        program,
                        instruction.span,
                    )
                });
            }
        }
        // Replaying a node must preserve the primitive's location, not just the
        // final output location. Rounded execution reports failures from this graph.
        if !matches!(instruction.opcode, Opcode::Load | Opcode::Store) {
            let value = stack.pop().expect("node instruction produced a state");
            stack.push(value.at_span(instruction.span));
        }
    }
    Err(vm_error(
        "NSV007",
        "bytecode ended without HALT",
        program,
        None,
    ))
}

fn required_operand<'a>(
    instruction: &'a Instruction,
    program: &BytecodeProgram,
) -> Result<&'a Operand, LanguageError> {
    instruction.operand.as_ref().ok_or_else(|| {
        vm_error(
            "NSV008",
            "instruction operand is missing",
            program,
            instruction.span,
        )
    })
}
fn required_slot(
    instruction: &Instruction,
    program: &BytecodeProgram,
    count: usize,
) -> Result<usize, LanguageError> {
    let Operand::Integer(slot) = required_operand(instruction, program)? else {
        return Err(vm_error(
            "NSV008",
            "instruction requires integer operand",
            program,
            instruction.span,
        ));
    };
    let slot = usize::try_from(*slot).map_err(|_conversion_error| {
        vm_error("NSV002", "invalid slot", program, instruction.span)
    })?;
    if slot >= count {
        return Err(vm_error(
            "NSV002",
            "invalid slot",
            program,
            instruction.span,
        ));
    }
    Ok(slot)
}
fn vm_error(
    code: &str,
    message: &str,
    program: &BytecodeProgram,
    span: Option<Span>,
) -> LanguageError {
    LanguageError(crate::core::Diagnostic {
        code: code.into(),
        message: message.into(),
        source_name: program.source_name.clone(),
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{interpret, parse};
    #[test]
    fn vm_is_independent_and_round_trips() {
        let source = "let x = index(2, scalar(3/2, 0))\noutput add(x, phase(2, x))";
        let ast = parse(source, "vm.ns").unwrap();
        let bytecode = compile(&ast).unwrap();
        assert_eq!(execute(&bytecode).unwrap(), interpret(&ast).unwrap());
        assert_eq!(
            BytecodeProgram::from_data(&bytecode.to_data()).unwrap(),
            bytecode
        );
    }

    #[test]
    fn vm_rejects_noncanonical_phase_bytecode() {
        let ast = parse("output phase(1, one)", "invalid-phase-bytecode.ns").unwrap();
        let mut bytecode = compile(&ast).unwrap();
        let instruction = bytecode
            .instructions
            .iter_mut()
            .find(|instruction| instruction.opcode == Opcode::Phase)
            .expect("compiled source contains one phase instruction");
        instruction.operand = Some(Operand::Integer(4));

        let error = execute(&bytecode).unwrap_err();
        assert_eq!(error.0.code, "NSV012");
        assert!(error.0.span.is_some());
    }
}
