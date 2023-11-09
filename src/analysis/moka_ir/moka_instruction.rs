use std::{collections::HashSet, fmt::Display};

use itertools::Itertools;

use crate::elements::{
    instruction::{Instruction, ProgramCounter},
    ConstantValue,
};

#[derive(Debug)]
pub enum MokaInstruction {
    Nop,
    Assignment {
        lhs: Identifier,
        rhs: Expression,
    },
    SideEffect {
        rhs: Expression,
    },
    Jump {
        target: ProgramCounter,
    },
    UnitaryConditionalJump {
        condition: ValueRef,
        target: ProgramCounter,
        instruction: Instruction,
    },
    BinaryConditionalJump {
        condition: [ValueRef; 2],
        target: ProgramCounter,
        instruction: Instruction,
    },
    Switch {
        condition: ValueRef,
        instruction: Instruction,
    },
    Return {
        value: Option<ValueRef>,
    },
    SubRoutineRet {
        target: ValueRef,
    },
}

impl Display for MokaInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MokaInstruction::Nop => write!(f, "nop"),
            MokaInstruction::Assignment { lhs, rhs } => write!(f, "{} = {}", lhs, rhs),
            MokaInstruction::SideEffect { rhs: op } => write!(f, "{}", op),
            MokaInstruction::Jump { target } => write!(f, "goto {}", target),
            MokaInstruction::UnitaryConditionalJump {
                condition,
                target,
                instruction,
            } => write!(f, "{}({}) goto {}", instruction.name(), condition, target),
            MokaInstruction::BinaryConditionalJump {
                condition,
                target,
                instruction,
            } => {
                write!(
                    f,
                    "{}({}, {}) goto {}",
                    instruction.name(),
                    condition[0],
                    condition[1],
                    target
                )
            }
            MokaInstruction::Switch {
                condition,
                instruction,
            } => write!(f, "{}({})", instruction.name(), condition),
            MokaInstruction::Return { value } => {
                if let Some(value) = value {
                    write!(f, "return {}", value)
                } else {
                    write!(f, "return")
                }
            }
            MokaInstruction::SubRoutineRet { target } => write!(f, "ret {}", target),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueRef {
    Def(Identifier),
    Phi(HashSet<Identifier>),
}

impl Display for ValueRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueRef::Def(id) => write!(f, "{}", id),
            ValueRef::Phi(ids) => {
                write!(
                    f,
                    "Phi({})",
                    ids.iter().map(|id| format!("{}", id)).join(", ")
                )
            }
        }
    }
}

impl From<Identifier> for ValueRef {
    fn from(value: Identifier) -> Self {
        Self::Def(value)
    }
}

#[derive(Debug)]
pub enum Expression {
    Const(ConstantValue),
    ReturnAddress(ProgramCounter),
    Expr {
        instruction: Instruction,
        arguments: Vec<ValueRef>,
    },
}
impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expression::*;
        match self {
            Const(c) => write!(f, "{:?}", c),
            ReturnAddress(pc) => write!(f, "{:?}", pc),
            Expr {
                instruction,
                arguments,
            } => {
                write!(
                    f,
                    "{}({})",
                    instruction.name(),
                    arguments.iter().map(|it| it.to_string()).join(", ")
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Identifier {
    Val(u16),
    This,
    Arg(u8),
    CaughtException,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Identifier::*;
        match self {
            Val(idx) => write!(f, "v{}", idx),
            This => write!(f, "this"),
            Arg(idx) => write!(f, "arg{}", idx),
            CaughtException => write!(f, "exception"),
        }
    }
}