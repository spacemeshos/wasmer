//! A stack-based interpreter to execute instructions of WIT adapters.

mod instruction;
mod instructions;
pub mod stack;
pub mod wasm;

pub use instruction::Instruction;
use stack::Stack;
use std::{convert::TryFrom, marker::PhantomData};
use wasm::values::InterfaceValue;

/// Represents the `Runtime`, which is used by an adapter to execute
/// its instructions.
pub(crate) struct Runtime<'invocation, 'instance, Instance, Export, LocalImport, Memory, MemoryView>
where
    Export: wasm::structures::Export + 'instance,
    LocalImport: wasm::structures::LocalImport + 'instance,
    Memory: wasm::structures::Memory<MemoryView> + 'instance,
    MemoryView: wasm::structures::MemoryView,
    Instance: wasm::structures::Instance<Export, LocalImport, Memory, MemoryView> + 'instance,
{
    /// The invocation inputs are all the arguments received by an
    /// adapter.
    invocation_inputs: &'invocation [InterfaceValue],

    /// Each runtime (so adapter) has its own stack instance.
    stack: Stack<InterfaceValue>,

    /// The WebAssembly module instance. It is used by adapter's
    /// instructions.
    wasm_instance: &'instance mut Instance,

    /// Phantom data.
    _phantom: PhantomData<(Export, LocalImport, Memory, MemoryView)>,
}

/// Type alias for an executable instruction. It's an implementation
/// details, but an instruction is a boxed closure instance.
pub(crate) type ExecutableInstruction<Instance, Export, LocalImport, Memory, MemoryView> = Box<
    dyn Fn(&mut Runtime<Instance, Export, LocalImport, Memory, MemoryView>) -> Result<(), String>,
>;

/// An interpreter is the central piece of this crate. It is a set of
/// executable instructions. Each instruction takes the runtime as
/// argument. The runtime holds the invocation inputs, the stack, and
/// the WebAssembly instance.
///
/// When the interpreter executes the instructions, each of them can
/// query the WebAssembly instance, operates on the stack, or reads
/// the invocation inputs. At the end of the execution, the stack
/// supposedly contains a result. Since an interpreter is used by a
/// WIT adapter to execute its instructions, the result on the stack
/// is the result of the adapter.
///
/// # Example
///
/// ```rust,ignore
/// use std::{cell::Cell, collections::HashMap, convert::TryInto};
/// use wasmer_interface_types::interpreter::{
///     instructions::tests::{Export, Instance, LocalImport, Memory, MemoryView},
/// //  ^^^^^^^^^^^^ This is private and for testing purposes only.
/// //               It is basically a fake WebAssembly runtime.
///     stack::Stackable,
///     wasm::values::{InterfaceType, InterfaceValue},
///     Instruction, Interpreter,
/// };
///
/// # fn main() {
/// // 1. Creates an interpreter from a set of instructions. They will
/// //    be transformed into executable instructions.
/// let interpreter: Interpreter<Instance, Export, LocalImport, Memory, MemoryView> = (&vec![
///     Instruction::ArgumentGet { index: 1 },
///     Instruction::ArgumentGet { index: 0 },
///     Instruction::CallExport { export_name: "sum" },
/// ])
///     .try_into()
///     .unwrap();
///
/// // 2. Defines the arguments of the adapter.
/// let invocation_inputs = vec![InterfaceValue::I32(3), InterfaceValue::I32(4)];
///
/// // 3. Creates a WebAssembly instance.
/// let mut instance = Instance {
///     // 3.1. Defines one exported function: `fn sum(a: i32, b: i32) -> i32 { a + b }`.
///     exports: {
///         let mut hashmap = HashMap::new();
///         hashmap.insert(
///             "sum".into(),
///             Export {
///                 // Defines the argument types of the function.
///                 inputs: vec![InterfaceType::I32, InterfaceType::I32],
///
///                 // Defines the result types.
///                 outputs: vec![InterfaceType::I32],
///
///                 // Defines the function implementation.
///                 function: |arguments: &[InterfaceValue]| {
///                     let a: i32 = (&arguments[0]).try_into().unwrap();
///                     let b: i32 = (&arguments[1]).try_into().unwrap();
///
///                     Ok(vec![InterfaceValue::I32(a + b)])
///                 },
///             },
///         );
///     },
///     ..Default::default()
/// };
///
/// // 4. Executes the instructions.
/// let run = interpreter.run(&invocation_inputs, &mut instance);
///
/// assert!(run.is_ok());
///
/// let stack = run.unwrap();
///
/// // 5. Read the stack to get the result.
/// assert_eq!(stack.as_slice(), &[InterfaceValue::I32(7)]);
/// # }
/// ```
pub struct Interpreter<Instance, Export, LocalImport, Memory, MemoryView>
where
    Export: wasm::structures::Export,
    LocalImport: wasm::structures::LocalImport,
    Memory: wasm::structures::Memory<MemoryView>,
    MemoryView: wasm::structures::MemoryView,
    Instance: wasm::structures::Instance<Export, LocalImport, Memory, MemoryView>,
{
    executable_instructions:
        Vec<ExecutableInstruction<Instance, Export, LocalImport, Memory, MemoryView>>,
}

impl<Instance, Export, LocalImport, Memory, MemoryView>
    Interpreter<Instance, Export, LocalImport, Memory, MemoryView>
where
    Export: wasm::structures::Export,
    LocalImport: wasm::structures::LocalImport,
    Memory: wasm::structures::Memory<MemoryView>,
    MemoryView: wasm::structures::MemoryView,
    Instance: wasm::structures::Instance<Export, LocalImport, Memory, MemoryView>,
{
    fn iter(
        &self,
    ) -> impl Iterator<
        Item = &ExecutableInstruction<Instance, Export, LocalImport, Memory, MemoryView>,
    > + '_ {
        self.executable_instructions.iter()
    }

    /// Runs the interpreter, such as:
    ///   1. Create a fresh stack,
    ///   2. Create a fresh stack,
    ///   3. Execute the instructions one after the other, and
    ///      returns the stack.
    pub fn run(
        &self,
        invocation_inputs: &[InterfaceValue],
        wasm_instance: &mut Instance,
    ) -> Result<Stack<InterfaceValue>, String> {
        let mut runtime = Runtime {
            invocation_inputs,
            stack: Stack::new(),
            wasm_instance,
            _phantom: PhantomData,
        };

        for executable_instruction in self.iter() {
            match executable_instruction(&mut runtime) {
                Ok(_) => continue,
                Err(message) => return Err(message),
            }
        }

        Ok(runtime.stack)
    }
}

/// Transforms a `Vec<Instruction>` into an `Interpreter`.
impl<'binary_input, Instance, Export, LocalImport, Memory, MemoryView>
    TryFrom<&Vec<Instruction<'binary_input>>>
    for Interpreter<Instance, Export, LocalImport, Memory, MemoryView>
where
    Export: wasm::structures::Export,
    LocalImport: wasm::structures::LocalImport,
    Memory: wasm::structures::Memory<MemoryView>,
    MemoryView: wasm::structures::MemoryView,
    Instance: wasm::structures::Instance<Export, LocalImport, Memory, MemoryView>,
{
    type Error = String;

    fn try_from(instructions: &Vec<Instruction>) -> Result<Self, Self::Error> {
        let executable_instructions = instructions
            .iter()
            .map(|instruction| {
                let instruction_name = instruction.to_string();

                match instruction {
                    Instruction::ArgumentGet { index } => {
                        instructions::argument_get(*index, instruction_name)
                    }
                    Instruction::Call { function_index } => {
                        instructions::call(*function_index, instruction_name)
                    }
                    Instruction::CallExport { export_name } => {
                        instructions::call_export((*export_name).to_owned(), instruction_name)
                    }
                    Instruction::ReadUtf8 => instructions::read_utf8(instruction_name),
                    Instruction::WriteUtf8 { allocator_name } => {
                        instructions::write_utf8((*allocator_name).to_owned(), instruction_name)
                    }
                    _ => unimplemented!(),
                }
            })
            .collect();

        Ok(Interpreter {
            executable_instructions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{wasm::structures::EmptyMemoryView, Instruction, Interpreter};
    use std::convert::TryInto;

    #[test]
    fn test_interpreter_from_instructions() {
        let instructions = vec![
            Instruction::ArgumentGet { index: 0 },
            Instruction::ArgumentGet { index: 0 },
            Instruction::CallExport { export_name: "foo" },
            Instruction::ReadUtf8,
            Instruction::Call { function_index: 7 },
        ];
        let interpreter: Interpreter<(), (), (), (), EmptyMemoryView> =
            (&instructions).try_into().unwrap();

        assert_eq!(interpreter.executable_instructions.len(), 5);
    }
}
