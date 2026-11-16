//! One function body as the pinned reader builds it from `RustGM`'s text.
//!
//! The reader numbers temporaries from the parameters on, in the order the
//! text first names them (an instruction's result before its operands),
//! blocks in the order the text first names them (a label or a branch
//! target), slots in declaration order and literals one per occurrence in
//! parse order. Instructions and terminators share one buffer in text order,
//! and each call or invocation appends its arguments to the operand pool
//! after reading them. This module states exactly those records.
use crate::{
    Failure,
    literals::{self, Context},
    records::{
        ArgumentFact, BINARY_ADD, BINARY_AND, BINARY_DIV, BINARY_MOD, BINARY_MUL, BINARY_OR,
        BINARY_SHL, BINARY_SHR, BINARY_SUB, BINARY_U_DIV, BINARY_U_MOD, BINARY_U_SHR, BINARY_XOR,
        CMP_EQ, CMP_GE, CMP_GT, CMP_LE, CMP_LT, CMP_NE, CMP_UGE, CMP_UGT, CMP_ULE, CMP_ULT,
        CONVERT_FP_EXT, CONVERT_FP_TO_SI, CONVERT_FP_TO_UI, CONVERT_FP_TRUNC, CONVERT_S_EXT,
        CONVERT_SI_TO_FP, CONVERT_TRUNC, CONVERT_UI_TO_FP, CONVERT_Z_EXT, FLAG_EXACT,
        FLAG_SIGNATURE, FLAG_VOLATILE, Instruction as Record, Literal as Bits, NO_ID, OP_ADDR,
        OP_ASM, OP_ATOMIC_ADD_FETCH, OP_ATOMIC_COMPARE_EXCHANGE, OP_ATOMIC_EXCHANGE,
        OP_ATOMIC_LOAD, OP_ATOMIC_STORE, OP_BINARY, OP_BRANCH, OP_CALL, OP_COMPARE, OP_CONVERT,
        OP_COPY, OP_COPY_OBJECT, OP_INDEX, OP_JUMP, OP_LOAD, OP_RETURN, OP_STORE, OP_UNARY,
        OP_UNREACHABLE, OP_VA_ARG, OP_VA_START, OP_VECTOR, Slot, TYPE_PTR, TYPE_VOID, Type,
        UNARY_BIT_NOT, UNARY_COUNT_LEADING_ZEROS, UNARY_NEG, UNARY_NOT, VALUE_BLOCK, VALUE_SLOT,
        VALUE_SYMBOL, VALUE_TEMP, VECTOR_FROMI32, VECTOR_PAND, VECTOR_PCMPEQB, VECTOR_PCMPGTB,
        VECTOR_PMOVMSKB, VECTOR_POR, VECTOR_PSHUFD, Value,
    },
    stater::{self, Stater},
};
use rustgm::lowir::{
    boundary::{self, Classes},
    program::{
        Access, BlockId, BoundaryArgument, CallArguments, Function, Instruction, Operand,
        Operation, Range, ResultId, SignatureId, Storage, Terminator,
    },
    validation,
    vocabulary::{Binary, Convert, Predicate, Type as Scalar, Unary, Vector, VectorShape},
};

/// The construction vectors of one body, reused from function to function.
#[derive(Default)]
pub(crate) struct Scratch {
    instructions: Vec<Record>,
    operands: Vec<Value>,
    literals: Vec<Bits>,
    slots: Vec<Slot>,
    layout: Vec<u32>,
    facts: Vec<ArgumentFact>,
    arguments: Vec<Value>,
    /// The reader's identity of each scalar and object temporary, once named.
    scalars: Vec<u32>,
    objects: Vec<u32>,
    /// The reader's identity of each block, once named, and whether placed.
    blocks: Vec<u32>,
    placed: Vec<bool>,
    /// Whether each temporary after the parameters has its definition.
    defined: Vec<bool>,
}
impl Scratch {
    fn reset(&mut self, function: &Function) {
        self.instructions.clear();
        self.operands.clear();
        self.literals.clear();
        self.slots.clear();
        self.layout.clear();
        self.facts.clear();
        self.scalars.clear();
        self.scalars.resize(function.temporaries.len(), NO_ID);
        self.objects.clear();
        let objects = function
            .objects
            .as_ref()
            .map_or(0, |objects| objects.temporaries.len());
        self.objects.resize(objects, NO_ID);
        self.blocks.clear();
        self.blocks.resize(function.blocks.len(), NO_ID);
        self.placed.clear();
        self.defined.clear();
    }
}

/// The function a body belongs to.
#[derive(Clone, Copy)]
pub(crate) struct Owner {
    /// The function's library symbol.
    pub symbol: u32,
    /// Its signature in the program.
    pub signature: SignatureId,
    /// Whether the library declared it: a body is committed only then.
    pub declared: bool,
}

fn index(length: usize) -> Result<u32, Failure> {
    u32::try_from(length).map_err(|_| Failure::from("LowIR identity space exhausted"))
}

/// State one body and, unless it holds a form the bridge does not state,
/// commit it with its argument facts.
pub(crate) fn state(
    stater: &mut Stater<'_>,
    scratch: &mut Scratch,
    owner: Owner,
    function: &Function,
) -> Result<(), Failure> {
    scratch.reset(function);
    let noted = stater.forms.total();
    let parameters = index(stater.program.parameter_count(owner.signature)?)?;
    let mut body = Body {
        stater,
        scratch,
        function,
        owner: owner.signature,
        parameters,
        temporaries: parameters,
        block: 0,
    };
    body.slots()?;
    if function.order.is_empty() {
        body.unsupported("function without blocks");
    }
    for &id in &function.order {
        body.block_label(id)?;
    }
    body.check_names();
    let Body {
        stater,
        scratch,
        temporaries,
        ..
    } = body;
    if stater.forms.total() != noted || !owner.declared {
        return Ok(());
    }
    stater.module.body(
        owner.symbol,
        &crate::ffi::Body {
            instructions: &scratch.instructions,
            operands: &scratch.operands,
            literals: &scratch.literals,
            slots: &scratch.slots,
            layout: &scratch.layout,
            temporaries,
        },
    )?;
    if !scratch.facts.is_empty() {
        stater.module.argument_facts(owner.symbol, &scratch.facts)?;
    }
    Ok(())
}

struct Body<'a, 'p> {
    stater: &'a mut Stater<'p>,
    scratch: &'a mut Scratch,
    function: &'a Function,
    owner: SignatureId,
    parameters: u32,
    /// The next temporary identity.
    temporaries: u32,
    /// The block being read.
    block: u32,
}
impl Body<'_, '_> {
    fn unsupported(&mut self, form: &'static str) {
        self.stater.forms.note(form);
    }
    /// `slot $sN : T`, in index order.
    fn slots(&mut self) -> Result<(), Failure> {
        for storage in &self.function.slots {
            let ty = match *storage {
                Storage::Scalar(ty) => stater::scalar(ty),
                Storage::Object(id) => {
                    let span = *self
                        .function
                        .spans
                        .get(id.0 as usize)
                        .ok_or("invalid LowIR object span")?;
                    self.stater.object(span)
                }
            };
            self.scratch.slots.push(Slot {
                ty,
                ..Slot::default()
            });
        }
        Ok(())
    }
    /// The reader's identity of a block, named here if new.
    fn block(&mut self, id: BlockId) -> Value {
        let Some(known) = self.scratch.blocks.get_mut(id.0 as usize) else {
            self.unsupported("branch to a block that is not placed");
            return Value::default();
        };
        if *known == NO_ID {
            *known = u32::try_from(self.scratch.placed.len()).unwrap_or(NO_ID);
            self.scratch.placed.push(false);
        }
        Value {
            kind: VALUE_BLOCK,
            index: *known,
            ..Value::default()
        }
    }
    /// `block ^bN:`, its instructions and its terminator.
    fn block_label(&mut self, id: BlockId) -> Result<(), Failure> {
        let block = self
            .function
            .blocks
            .get(id.0 as usize)
            .ok_or("invalid LowIR block")?;
        let label = self.block(id).index;
        if self.scratch.placed[label as usize] {
            self.unsupported("block placed twice");
        }
        self.scratch.placed[label as usize] = true;
        self.scratch.layout.push(label);
        self.block = label;
        let instructions = self
            .function
            .instructions
            .get(block.instructions.start as usize..block.instructions.end as usize)
            .ok_or("invalid LowIR instruction range")?;
        for instruction in instructions {
            self.instruction(instruction)?;
        }
        validation::terminator(
            self.stater.program,
            self.function,
            self.owner,
            block.terminator,
        )?;
        self.terminator(block.terminator)
    }
    /// Every named temporary is defined and every named block placed.
    fn check_names(&mut self) {
        if self.scratch.defined.contains(&false) {
            self.unsupported("temporary used without a definition");
        }
        if self.scratch.placed.contains(&false) {
            self.unsupported("branch to a block that is not placed");
        }
    }
    /// The reader's identity of a temporary, named here if new.
    fn name(&mut self, scalar: bool, id: u32) -> Option<u32> {
        let names = if scalar {
            &mut self.scratch.scalars
        } else {
            &mut self.scratch.objects
        };
        let known = names.get_mut(id as usize)?;
        if *known == NO_ID {
            *known = self.temporaries;
            self.temporaries = self.temporaries.checked_add(1)?;
            self.scratch.defined.push(false);
        }
        Some(*known)
    }
    /// The reader's identity of an instruction's result, named first.
    fn result(&mut self, result: ResultId) -> Result<u32, Failure> {
        let named = match result {
            ResultId::Scalar(id) => self.name(true, id.0),
            ResultId::Object(id) => self.name(false, id.0),
        }
        .ok_or("invalid LowIR temporary")?;
        let position = (named - self.parameters) as usize;
        if self.scratch.defined[position] {
            self.unsupported("temporary defined twice");
        }
        self.scratch.defined[position] = true;
        Ok(named)
    }
    /// The type of a scalar result, as `copy` and `convert` spell it.
    fn result_type(&self, instruction: &Instruction) -> Result<Scalar, Failure> {
        let Some(ResultId::Scalar(id)) = instruction.result else {
            return Err("LowIR scalar instruction lacks scalar result".into());
        };
        Ok(*self
            .function
            .temporaries
            .get(id.0 as usize)
            .ok_or("invalid LowIR temporary")?)
    }
    /// One operand as the reader reads it in `context`.
    fn value(&mut self, operand: Operand, context: Context) -> Result<Value, Failure> {
        let temporary = |index| Value {
            kind: VALUE_TEMP,
            index,
            ..Value::default()
        };
        Ok(match operand {
            Operand::Temporary(id) => {
                temporary(self.name(true, id.0).ok_or("invalid scalar temporary")?)
            }
            Operand::ObjectTemporary(id) => {
                let span = self
                    .function
                    .objects
                    .as_ref()
                    .and_then(|objects| objects.temporaries.get(id.0 as usize))
                    .and_then(|span| self.function.spans.get(span.0 as usize));
                if span.is_none() {
                    return Err("invalid object temporary".into());
                }
                temporary(self.name(false, id.0).ok_or("invalid object temporary")?)
            }
            Operand::Parameter(index) => {
                if index >= self.parameters {
                    self.unsupported("parameter beyond its signature");
                }
                temporary(index)
            }
            Operand::Slot(id) => {
                if id.0 as usize >= self.function.slots.len() {
                    self.unsupported("undeclared slot");
                }
                Value {
                    kind: VALUE_SLOT,
                    index: id.0,
                    ..Value::default()
                }
            }
            Operand::Symbol(id) => symbol(self.stater.function_symbol(id)?),
            Operand::Global(id) => symbol(self.stater.global_symbol(id)?),
            Operand::Literal(id) => {
                let literal = *self
                    .function
                    .literals
                    .get(id.0 as usize)
                    .ok_or("invalid LowIR literal")?;
                self.pooled(literal, context)?
            }
        })
    }
    /// A literal, one pool entry per occurrence, as the reader reads it in `context`.
    fn pooled(
        &mut self,
        literal: rustgm::lowir::program::Literal,
        context: Context,
    ) -> Result<Value, Failure> {
        Ok(match literals::read(literal, context) {
            Ok(read) => {
                let index = index(self.scratch.literals.len())?;
                self.scratch.literals.push(read.bits);
                Value {
                    kind: read.kind,
                    float_kind: read.float_kind,
                    reserved: 0,
                    index,
                }
            }
            Err(form) => {
                self.unsupported(form);
                Value::default()
            }
        })
    }
    /// A record of the block being read, with no result.
    fn record(&self, op: u8, ty: Type) -> Record {
        Record {
            op,
            result: NO_ID,
            next: NO_ID,
            location: self.block,
            ty,
            ..Record::default()
        }
    }
    /// Append a record, binding its result.
    fn append(&mut self, mut record: Record, result: Option<u32>) {
        if let Some(result) = result {
            record.result = result;
        }
        self.scratch.instructions.push(record);
    }
    fn instruction(&mut self, instruction: &Instruction) -> Result<(), Failure> {
        let program = self.stater.program;
        validation::instruction(program, self.function, self.owner, instruction)?;
        let defines = match instruction.operation {
            Operation::Store { .. }
            | Operation::AtomicStore { .. }
            | Operation::VaStart { .. }
            | Operation::CopyObject { .. }
            | Operation::Asm { .. } => false,
            Operation::Call { signature, .. } => {
                program.result_type(signature)? != boundary::Type::Scalar(Scalar::Void)
            }
            _ => true,
        };
        if defines != instruction.result.is_some() {
            self.unsupported(if defines {
                "value instruction without a result"
            } else {
                "result of an instruction without a value"
            });
        }
        // The reader names the result before reading the operands.
        let result = match instruction.result {
            Some(result) if defines => Some(self.result(result)?),
            _ => None,
        };
        let record = match instruction.operation {
            Operation::Call { .. } => return self.call(instruction, result),
            Operation::Asm {
                template,
                arguments,
            } => self.invocation(template.0, arguments)?,
            Operation::Copy(_)
            | Operation::Address(_)
            | Operation::Unary { .. }
            | Operation::Binary { .. }
            | Operation::Compare { .. }
            | Operation::Convert { .. } => self.arithmetic(instruction)?,
            Operation::Vector {
                op,
                left,
                right,
                immediate,
            } => self.vector(op, left, right, immediate)?,
            _ => self.memory(instruction.operation)?,
        };
        self.append(record, result);
        Ok(())
    }
    fn arithmetic(&mut self, instruction: &Instruction) -> Result<Record, Failure> {
        Ok(match instruction.operation {
            Operation::Copy(value) => {
                let ty = self.result_type(instruction)?;
                let mut record = self.record(OP_COPY, stater::scalar(ty));
                record.ops[0] = self.value(value.operand, Context::of(ty))?;
                record
            }
            Operation::Address(operand) => {
                if !matches!(
                    operand,
                    Operand::Slot(_) | Operand::Symbol(_) | Operand::Global(_)
                ) {
                    self.unsupported("addr of a value that is not storage");
                }
                let mut record = self.record(OP_ADDR, TYPE_PTR);
                record.ops[0] = self.value(operand, Context::Open)?;
                record
            }
            Operation::Unary { op, value } => {
                let mut record = self.record(OP_UNARY, stater::scalar(value.ty));
                record.sub = unary(op);
                record.ops[0] = self.value(value.operand, Context::of(value.ty))?;
                record
            }
            Operation::Binary { op, left, right } => {
                let mut record = self.record(OP_BINARY, stater::scalar(left.ty));
                record.sub = binary(op);
                record.ops[0] = self.value(left.operand, Context::of(left.ty))?;
                record.ops[1] = self.value(right, Context::of(left.ty))?;
                record
            }
            Operation::Compare {
                predicate,
                left,
                right,
            } => {
                let mut record = self.record(OP_COMPARE, stater::scalar(left.ty));
                record.sub = compare(predicate);
                record.ops[0] = self.value(left.operand, Context::of(left.ty))?;
                record.ops[1] = self.value(right, Context::of(left.ty))?;
                record
            }
            Operation::Convert { op, value } => {
                let ty = self.result_type(instruction)?;
                let mut record = self.record(OP_CONVERT, stater::scalar(ty));
                record.sub = convert(op);
                if matches!(op, Convert::TruncExact) {
                    record.flags = FLAG_EXACT;
                }
                // The source type word, kept in the immediate.
                record.immediate = i64::from_ne_bytes(stater::scalar(value.ty).to_ne_bytes());
                record.ops[0] = self.value(value.operand, Context::of(value.ty))?;
                record
            }
            _ => return Err("expected an arithmetic LowIR instruction".into()),
        })
    }
    /// `vector op`: the operands in text order, a shape's immediate last as
    /// the `i32` literal the reader reads.
    fn vector(
        &mut self,
        op: Vector,
        left: Operand,
        right: Option<Operand>,
        immediate: u8,
    ) -> Result<Record, Failure> {
        let mut record = self.record(OP_VECTOR, stater::scalar(op.result()));
        record.sub = vector(op);
        record.ops[0] = self.value(left, Context::of(op.operand()))?;
        if let Some(right) = right {
            record.ops[1] = self.value(right, Context::of(Scalar::V128))?;
        }
        if op.shape() == VectorShape::UnaryImmediate {
            record.ops[2] = self.pooled(
                rustgm::lowir::program::Literal::Integer(u128::from(immediate)),
                Context::of(Scalar::I32),
            )?;
        }
        Ok(record)
    }
    fn memory(&mut self, operation: Operation) -> Result<Record, Failure> {
        let pointer = Context::Scalar(Scalar::Ptr);
        let flags = |access| {
            if access == Access::Volatile {
                FLAG_VOLATILE
            } else {
                0
            }
        };
        Ok(match operation {
            Operation::Load {
                access,
                ty,
                address,
            } => {
                let mut record = self.record(OP_LOAD, stater::scalar(ty));
                record.flags = flags(access);
                record.ops[0] = self.value(address, pointer)?;
                record
            }
            Operation::Store {
                access,
                value,
                address,
            } => {
                let mut record = self.record(OP_STORE, stater::scalar(value.ty));
                record.flags = flags(access);
                record.ops[0] = self.value(value.operand, Context::of(value.ty))?;
                record.ops[1] = self.value(address, pointer)?;
                record
            }
            Operation::Index {
                element,
                base,
                offset,
            } => {
                let mut record = self.record(OP_INDEX, stater::scalar(element));
                record.ops[0] = self.value(base, pointer)?;
                record.ops[1] = self.value(offset, Context::Open)?;
                record
            }
            Operation::CopyObject {
                span,
                source,
                destination,
            } => {
                let span = *self
                    .function
                    .spans
                    .get(span.0 as usize)
                    .ok_or("invalid LowIR copy span")?;
                let mut record = self.record(OP_COPY_OBJECT, TYPE_VOID);
                // `copyobj BxA`: the size in the immediate, log2 alignment in `sub`.
                record.immediate = i64::try_from(span.bytes).unwrap_or_else(|_| {
                    self.unsupported("object copy beyond 2^63 bytes");
                    0
                });
                record.sub = stater::alignment(span).map_or_else(
                    |form| {
                        self.unsupported(form);
                        0
                    },
                    |log2| u8::try_from(log2).unwrap_or(0),
                );
                record.ops[0] = self.value(source, pointer)?;
                record.ops[1] = self.value(destination, pointer)?;
                record
            }
            Operation::VaStart { list } => {
                let mut record = self.record(OP_VA_START, TYPE_VOID);
                record.ops[0] = self.value(list, pointer)?;
                record
            }
            Operation::VaArg { ty, list } => {
                let mut record = self.record(OP_VA_ARG, stater::scalar(ty));
                record.ops[0] = self.value(list, Context::Open)?;
                record
            }
            atomic => self.atomic(atomic)?,
        })
    }
    fn atomic(&mut self, operation: Operation) -> Result<Record, Failure> {
        let pointer = Context::Scalar(Scalar::Ptr);
        Ok(match operation {
            Operation::AtomicLoad {
                ty,
                address,
                ordering,
            } => {
                let mut record = self.record(OP_ATOMIC_LOAD, stater::scalar(ty));
                record.ops[0] = self.value(address, pointer)?;
                record.order = ordering.encoding();
                record
            }
            Operation::AtomicStore {
                value,
                address,
                ordering,
            } => {
                let mut record = self.record(OP_ATOMIC_STORE, stater::scalar(value.ty));
                record.ops[0] = self.value(value.operand, Context::of(value.ty))?;
                record.ops[1] = self.value(address, pointer)?;
                record.order = ordering.encoding();
                record
            }
            Operation::AtomicCompareExchange {
                value,
                address,
                expected,
                ordering,
            } => {
                let op = OP_ATOMIC_COMPARE_EXCHANGE;
                let mut record = self.record(op, stater::scalar(value.ty));
                record.ops[0] = self.value(address, pointer)?;
                record.ops[1] = self.value(expected, pointer)?;
                record.ops[2] = self.value(value.operand, Context::of(value.ty))?;
                // The success order, then the failure order in `sub`.
                record.order = ordering.success_encoding();
                record.sub = ordering.failure_encoding();
                record
            }
            Operation::AtomicExchange {
                value,
                address,
                ordering,
            }
            | Operation::AtomicAddFetch {
                value,
                address,
                ordering,
            } => {
                let op = if matches!(operation, Operation::AtomicExchange { .. }) {
                    OP_ATOMIC_EXCHANGE
                } else {
                    OP_ATOMIC_ADD_FETCH
                };
                let mut record = self.record(op, stater::scalar(value.ty));
                record.ops[0] = self.value(address, pointer)?;
                record.ops[1] = self.value(value.operand, Context::of(value.ty))?;
                record.order = ordering.encoding();
                record
            }
            _ => return Err("expected an atomic LowIR instruction".into()),
        })
    }
    /// The operand pool range of the arguments read into `scratch.arguments`.
    fn pool(&mut self, record: &mut Record) -> Result<(), Failure> {
        record.extra_first = index(self.scratch.operands.len())?;
        record.extra_count = index(self.scratch.arguments.len())?;
        self.scratch
            .operands
            .extend_from_slice(&self.scratch.arguments);
        Ok(())
    }
    /// `asm @asmN(arguments)`: every argument is read as `ptr`.
    fn invocation(&mut self, template: u32, arguments: Range) -> Result<Record, Failure> {
        let &(id, operands) = self
            .stater
            .templates
            .get(template as usize)
            .ok_or("LowIR assembly invocation without templates")?;
        let values = self
            .function
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("invalid LowIR assembly argument range")?;
        if values.len() != operands {
            return Err("LowIR assembly invocation differs from its template arity".into());
        }
        if id == NO_ID {
            self.unsupported("invocation of a template the bridge does not state");
        }
        let mut record = self.record(OP_ASM, TYPE_VOID);
        record.immediate = i64::from(id);
        let mut read = std::mem::take(&mut self.scratch.arguments);
        read.clear();
        for value in values {
            read.push(self.value(value.operand, Context::Scalar(Scalar::Ptr))?);
        }
        self.scratch.arguments = read;
        self.pool(&mut record)?;
        Ok(record)
    }
    /// `call R callee(arguments) [classes=...] [as (...)]`.
    fn call(&mut self, instruction: &Instruction, result: Option<u32>) -> Result<(), Failure> {
        let Operation::Call {
            signature,
            callee,
            arguments,
        } = instruction.operation
        else {
            return Err("expected call".into());
        };
        let ty = self.stater.result(signature)?;
        let mut record = self.record(OP_CALL, ty);
        if matches!(callee, Operand::Slot(_) | Operand::Literal(_)) {
            self.unsupported("call through a slot or a literal");
        }
        record.ops[0] = self.value(callee, Context::Open)?;
        let call = index(self.scratch.instructions.len())?;
        let mut read = std::mem::take(&mut self.scratch.arguments);
        read.clear();
        match arguments {
            CallArguments::Scalar(range) => {
                let values = self
                    .function
                    .arguments
                    .get(range.start as usize..range.end as usize)
                    .ok_or("invalid LowIR call argument range")?;
                for value in values {
                    read.push(self.value(value.operand, Context::Open)?);
                }
            }
            CallArguments::Object(range) => {
                let values = self
                    .function
                    .objects
                    .as_ref()
                    .and_then(|objects| {
                        objects
                            .arguments
                            .get(range.start as usize..range.end as usize)
                    })
                    .ok_or("missing object call arguments")?;
                for (argument, value) in values.iter().enumerate() {
                    let operand = match *value {
                        BoundaryArgument::Scalar(value) => value.operand,
                        BoundaryArgument::Object {
                            operand, classes, ..
                        } => {
                            if let Some(classes) = classes {
                                self.fact(call, index(argument)?, classes);
                            }
                            operand
                        }
                    };
                    read.push(self.value(operand, Context::Open)?);
                }
            }
        }
        self.scratch.arguments = read;
        self.pool(&mut record)?;
        // A call through a value states its own boundary, after its facts.
        if !matches!(callee, Operand::Symbol(_)) {
            record.flags |= FLAG_SIGNATURE;
            record.immediate = i64::from(self.stater.boundary(signature)?);
        }
        self.append(record, result);
        Ok(())
    }
    /// `classes=N:...` of one argument, read as for `obj<16x8>`.
    fn fact(&mut self, call: u32, argument: u32, classes: Classes) {
        if let Classes::Eightbytes { second: None, .. } = classes {
            // The reader reads call-site classes for two eightbytes.
            self.unsupported("call-site classes of one eightbyte");
            return;
        }
        self.scratch.facts.push(ArgumentFact {
            instruction: call,
            argument,
            classes: stater::classes(classes),
            ..ArgumentFact::default()
        });
    }
    fn terminator(&mut self, terminator: Terminator) -> Result<(), Failure> {
        let record = match terminator {
            Terminator::Jump(target) => {
                let mut record = self.record(OP_JUMP, TYPE_VOID);
                record.ops[0] = self.block(target);
                record
            }
            Terminator::Branch { condition, yes, no } => {
                let mut record = self.record(OP_BRANCH, TYPE_VOID);
                record.ops[0] = self.value(condition, Context::Scalar(Scalar::I64))?;
                record.ops[1] = self.block(yes);
                record.ops[2] = self.block(no);
                record
            }
            Terminator::Return(Some(value)) => {
                let mut record = self.record(OP_RETURN, stater::scalar(value.ty));
                record.ops[0] = self.value(value.operand, Context::of(value.ty))?;
                record
            }
            Terminator::Return(None) => self.record(OP_RETURN, TYPE_VOID),
            Terminator::ReturnObject { operand, span } => {
                let span = *self
                    .function
                    .spans
                    .get(span.0 as usize)
                    .ok_or("invalid LowIR object span")?;
                let ty = self.stater.object(span);
                let mut record = self.record(OP_RETURN, ty);
                record.ops[0] = self.value(operand, Context::Object)?;
                record
            }
            Terminator::Unreachable => self.record(OP_UNREACHABLE, TYPE_VOID),
        };
        self.append(record, None);
        Ok(())
    }
}

fn symbol(index: u32) -> Value {
    Value {
        kind: VALUE_SYMBOL,
        index,
        ..Value::default()
    }
}

fn unary(op: Unary) -> u8 {
    match op {
        Unary::Neg => UNARY_NEG,
        Unary::BitNot => UNARY_BIT_NOT,
        Unary::Not => UNARY_NOT,
        Unary::Clz => UNARY_COUNT_LEADING_ZEROS,
    }
}

fn binary(op: Binary) -> u8 {
    match op {
        Binary::Add => BINARY_ADD,
        Binary::Sub => BINARY_SUB,
        Binary::Mul => BINARY_MUL,
        Binary::Div => BINARY_DIV,
        Binary::Mod => BINARY_MOD,
        Binary::Udiv => BINARY_U_DIV,
        Binary::Umod => BINARY_U_MOD,
        Binary::And => BINARY_AND,
        Binary::Or => BINARY_OR,
        Binary::Xor => BINARY_XOR,
        Binary::Shl => BINARY_SHL,
        Binary::Shr => BINARY_SHR,
        Binary::Ushr => BINARY_U_SHR,
    }
}

fn compare(predicate: Predicate) -> u8 {
    match predicate {
        Predicate::Eq => CMP_EQ,
        Predicate::Ne => CMP_NE,
        Predicate::Lt => CMP_LT,
        Predicate::Le => CMP_LE,
        Predicate::Gt => CMP_GT,
        Predicate::Ge => CMP_GE,
        Predicate::Ult => CMP_ULT,
        Predicate::Ule => CMP_ULE,
        Predicate::Ugt => CMP_UGT,
        Predicate::Uge => CMP_UGE,
    }
}

fn vector(op: Vector) -> u8 {
    match op {
        Vector::Pand => VECTOR_PAND,
        Vector::Por => VECTOR_POR,
        Vector::Pcmpeqb => VECTOR_PCMPEQB,
        Vector::Pcmpgtb => VECTOR_PCMPGTB,
        Vector::Pmovmskb => VECTOR_PMOVMSKB,
        Vector::Pshufd => VECTOR_PSHUFD,
        Vector::FromI32 => VECTOR_FROMI32,
    }
}

fn convert(op: Convert) -> u8 {
    match op {
        Convert::Sext => CONVERT_S_EXT,
        Convert::Zext => CONVERT_Z_EXT,
        Convert::Trunc | Convert::TruncExact => CONVERT_TRUNC,
        Convert::Sitofp => CONVERT_SI_TO_FP,
        Convert::Uitofp => CONVERT_UI_TO_FP,
        Convert::Fptosi => CONVERT_FP_TO_SI,
        Convert::Fptoui => CONVERT_FP_TO_UI,
        Convert::Fpext => CONVERT_FP_EXT,
        Convert::Fptrunc => CONVERT_FP_TRUNC,
    }
}
