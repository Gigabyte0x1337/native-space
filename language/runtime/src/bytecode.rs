// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Version 1 stack bytecode and independent exact-state virtual machine.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::core::{
    Expr, Goal, LanguageError, NativeScalar, NativeState, OutputKind, Program, Span,
    expand_functions, optimize,
};

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
    Orient,
    Index,
    Halt,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Operand {
    Integer(i64),
    Index { direction: u64, depth: u64 },
    Scalar { real: String, imag: String },
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

/// Compile a valid exact-state document without evaluating it.
///
/// # Errors
///
/// Returns a name-analysis diagnostic or a slot-capacity error.
pub fn compile(program: &Program) -> Result<BytecodeProgram, LanguageError> {
    // Reflection must observe the original source graph. Lower calls and trace
    // strands before theorem-authorized rewrites optimize the executable form.
    let expanded = expand_functions(program)?;
    let program = optimize(&expanded)?.program;
    let mut compiler = Compiler::default();
    for binding in &program.bindings {
        compiler.expression(&binding.value);
        let slot = i64::try_from(compiler.slot_names.len()).map_err(|_capacity_error| {
            LanguageError(crate::core::Diagnostic {
                code: "NSC001".into(),
                message: "program has too many bindings".into(),
                source_name: program.source_name.clone(),
                span: binding.span,
            })
        })?;
        compiler.slots.insert(binding.name.clone(), slot);
        compiler.slot_names.push(binding.name.clone());
        compiler.emit(Opcode::Store, Some(Operand::Integer(slot)), binding.span);
    }
    compiler.expression(&program.result);
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
    slots: BTreeMap<String, i64>,
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

    fn expression(&mut self, expression: &Expr) {
        match expression {
            Expr::Zero { span } => self.emit(Opcode::PushZero, None, *span),
            Expr::One { span } => self.emit(Opcode::PushOne, None, *span),
            Expr::Scalar { real, imag, span } => self.emit(
                Opcode::PushScalar,
                Some(Operand::Scalar {
                    real: real.clone(),
                    imag: imag.clone(),
                }),
                *span,
            ),
            Expr::Reference { name, span } => self.emit(
                Opcode::Load,
                Some(Operand::Integer(self.slots[name])),
                *span,
            ),
            Expr::Call { .. } => unreachable!("calls are erased before bytecode generation"),
            Expr::Trace { .. } => {
                unreachable!("trace is lowered before bytecode generation")
            }
            Expr::Untrace { .. } => {
                unreachable!("untrace is lowered before bytecode generation")
            }
            Expr::Add { operands, span } | Expr::Multiply { operands, span } => {
                for operand in operands {
                    self.expression(operand);
                }
                let opcode = if matches!(expression, Expr::Add { .. }) {
                    Opcode::Add
                } else {
                    Opcode::Multiply
                };
                self.emit(
                    opcode,
                    Some(Operand::Integer(
                        i64::try_from(operands.len()).expect("operand count fits i64"),
                    )),
                    *span,
                );
            }
            Expr::Orient { turns, value, span } => {
                self.expression(value);
                self.emit(Opcode::Orient, Some(Operand::Integer(*turns)), *span);
            }
            Expr::Index {
                direction,
                multiplicity,
                value,
                span,
            } => {
                self.expression(value);
                self.emit(
                    Opcode::Index,
                    Some(Operand::Index {
                        direction: *direction,
                        depth: *multiplicity,
                    }),
                    *span,
                );
            }
        }
    }
}

/// Execute schema-1 bytecode in the independent exact-state VM.
///
/// # Errors
///
/// Returns a located VM diagnostic for malformed or invalid bytecode.
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive opcode match keeps VM behavior closed and auditable"
)]
pub fn execute(program: &BytecodeProgram) -> Result<NativeState, LanguageError> {
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
            Opcode::PushZero => stack.push(NativeState::zero()),
            Opcode::PushOne => stack.push(NativeState::one()),
            Opcode::PushScalar => {
                let Operand::Scalar { real, imag } = required_operand(instruction, program)? else {
                    return Err(vm_error(
                        "NSV009",
                        "instruction requires scalar operand",
                        program,
                        instruction.span,
                    ));
                };
                let value = NativeScalar::from_text(real, imag)
                    .map_err(|message| vm_error("NSV009", &message, program, instruction.span))?;
                stack.push(NativeState::scalar(value));
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
                let values = stack.split_off(stack.len() - arity);
                let mut result = if instruction.opcode == Opcode::Add {
                    NativeState::zero()
                } else {
                    NativeState::one()
                };
                for value in values {
                    result = if instruction.opcode == Opcode::Add {
                        result.add(&value)
                    } else {
                        result.multiply(&value)
                    };
                }
                stack.push(result);
            }
            Opcode::Orient => {
                let Operand::Integer(turns) = required_operand(instruction, program)? else {
                    return Err(vm_error(
                        "NSV008",
                        "instruction requires integer operand",
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
                stack.push(value.orient(*turns));
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
        let source = "let x = index(2, scalar(3/2, 0))\noutput add(x, orient(2, x))";
        let ast = parse(source, "vm.ns").unwrap();
        let bytecode = compile(&ast).unwrap();
        assert_eq!(execute(&bytecode).unwrap(), interpret(&ast).unwrap());
        assert_eq!(
            BytecodeProgram::from_data(&bytecode.to_data()).unwrap(),
            bytecode
        );
    }
}
