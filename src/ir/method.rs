use crate::jvm::{
    class::ClassReference,
    code::{ExceptionTableEntry, InstructionList},
    method::{MethodAccessFlags, MethodDescriptor},
};

use super::MokaInstruction;

/// Represents a JVM method where the instructions have been converted to Moka IR.
#[derive(Debug, Clone)]
pub struct MokaIRMethod {
    /// The access flags of the method.
    pub access_flags: MethodAccessFlags,
    /// The name of the method.
    pub name: String,
    /// The descriptor of the method.
    pub descriptor: MethodDescriptor,
    /// The class that contains the method.
    pub owner: ClassReference,
    /// The body of the method.
    pub instructions: InstructionList<MokaInstruction>,
    /// The exception table of the method.
    pub exception_table: Vec<ExceptionTableEntry>,
}