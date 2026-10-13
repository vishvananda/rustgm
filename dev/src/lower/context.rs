//! Demand and ABI identities are indexed by already selected semantic definitions/types.
use super::{Error, data, function, remainder};
use crate::{
    abi::layout::{Layout, Layouts},
    body::ir::{self, Owner},
    lowir::{
        builder::Builder,
        program::{MemorySpan, Operand, Program, SignatureId, SymbolId, Terminator},
        vocabulary::Type,
    },
    resolve::{
        model::DefId,
        unit::{FileId, NodeKey},
    },
    sema::{
        members::Members,
        model::{Declarations, Initializer},
        values::Reader,
    },
    syntax::ast::NodeId,
    types::{Kind, Primitive, Store, TypeId, aggregate::Definitions},
};
use std::collections::HashMap;

pub(super) struct Context<'a> {
    pub types: &'a Store,
    pub aggregates: &'a Definitions,
    pub members: &'a Members,
    pub constants: Reader<'a>,
    pub definitions: &'a [Option<TypeId>],
    pub initializers: &'a [Initializer],
    pub layouts: &'a mut Layouts,
    pub bodies: &'a ir::Program,
    pub output: Program,
    pub data: data::Storage,
    body_indices: Vec<Option<usize>>,
    symbols: Vec<Option<SymbolId>>,
    signatures: HashMap<TypeId, SignatureId>,
    queue: Vec<DefId>,
    abort: Option<SymbolId>,
    remainders: [Option<SymbolId>; 2],
}
pub(super) fn build(
    facts: &mut Declarations,
    bodies: &ir::Program,
    entry: DefId,
) -> Result<Program, Error> {
    let mut layouts = std::mem::take(&mut facts.layouts);
    let result = Context::build(facts, &mut layouts, bodies, entry);
    facts.layouts = layouts;
    result
}
#[derive(Clone, Copy)]
pub(super) enum Representation {
    Scalar(Type),
    Object(MemorySpan),
}
impl Representation {
    pub const fn parameter(self) -> Type {
        match self {
            Self::Scalar(ty) => ty,
            Self::Object(_) => Type::Ptr,
        }
    }
    pub const fn result(self) -> Type {
        match self {
            Self::Scalar(ty) => ty,
            Self::Object(_) => Type::Void,
        }
    }
}
impl<'a> Context<'a> {
    pub fn build(
        facts: &'a Declarations,
        layouts: &'a mut Layouts,
        bodies: &'a ir::Program,
        entry: DefId,
    ) -> Result<Program, Error> {
        let source = bodies.bodies.first().map_or(
            NodeKey {
                file: FileId(0),
                node: NodeId(0),
            },
            |b| b.source,
        );
        let size = bodies
            .bodies
            .iter()
            .map(|body| match body.owner {
                Owner::Function { definition, .. } | Owner::Initializer { definition, .. } => {
                    definition.0 as usize + 1
                }
            })
            .max()
            .unwrap_or(0);
        let mut context = Self {
            types: &facts.types,
            aggregates: &facts.aggregates,
            members: &facts.members,
            constants: Reader::new(facts),
            definitions: &facts.definitions,
            initializers: &facts.initializers,
            layouts,
            bodies,
            output: Program::default(),
            data: data::Storage::default(),
            body_indices: vec![None; size],
            symbols: vec![None; size],
            signatures: HashMap::new(),
            queue: Vec::new(),
            abort: None,
            remainders: [None; 2],
        };
        for (index, body) in bodies.bodies.iter().enumerate() {
            if let Owner::Function { definition, .. } = body.owner {
                context.body_indices[definition.0 as usize] = Some(index);
            }
        }
        let root = context.demand(entry).map_err(|message| Error {
            node: source,
            message,
        })?;
        let (mut next, mut next_data) = (0, 0);
        while next < context.queue.len() || next_data < context.data.pending.len() {
            if next == context.queue.len() {
                context
                    .initialize_data(next_data)
                    .map_err(|message| Error {
                        node: source,
                        message,
                    })?;
                next_data += 1;
                continue;
            }
            let definition = context.queue[next];
            let index = context.body_indices[definition.0 as usize].ok_or(Error {
                node: source,
                message: "missing demanded body",
            })?;
            let symbol = context.symbols[definition.0 as usize].ok_or(Error {
                node: source,
                message: "missing demanded symbol",
            })?;
            let body = &bodies.bodies[index];
            let lowered = function::lower(&mut context, body)?;
            context.output.symbols[symbol.0 as usize].body = Some(lowered);
            next += 1;
        }
        context.entry(root).map_err(|message| Error {
            node: source,
            message,
        })?;
        Ok(context.output)
    }
    pub fn signature(&mut self, ty: TypeId) -> Result<SignatureId, &'static str> {
        if let Some(id) = self.signatures.get(&ty) {
            return Ok(*id);
        }
        let Some(Kind::Function {
            bound_regions: _,
            parameters,
            result,
            unsafe_,
        }) = self.types.kind(ty)
        else {
            return Err("missing selected function signature");
        };
        if unsafe_ {
            return Err("unsafe calls require later checking");
        }
        let mut native = Vec::new();
        let result_type = self.representation(result)?;
        if matches!(result_type, Representation::Object(_)) {
            native.push(Type::Ptr);
        }
        for ty in self
            .types
            .list(parameters)
            .ok_or("missing parameter list")?
        {
            let representation = self.representation(*ty)?.parameter();
            if representation != Type::Void {
                native.push(representation);
            }
        }
        let signature =
            self.output
                .signature(&native, result_type.result(), result == TypeId::NEVER)?;
        self.signatures.insert(ty, signature);
        Ok(signature)
    }
    pub fn layout(&mut self, ty: TypeId) -> Result<Layout, &'static str> {
        let id = self
            .layouts
            .demand(self.types, self.aggregates, ty)
            .map_err(|e| e.message)?;
        self.layouts
            .at(id)
            .copied()
            .ok_or("missing demanded storage layout")
    }
    pub fn representation(&mut self, ty: TypeId) -> Result<Representation, &'static str> {
        if matches!(
            self.types.kind(ty),
            Some(Kind::Tuple(_) | Kind::Array { .. } | Kind::Adt(..))
        ) && ty != TypeId::UNIT
        {
            let layout = self.layout(ty)?;
            return Ok(if layout.size == 0 {
                Representation::Scalar(Type::Void)
            } else {
                Representation::Object(MemorySpan {
                    bytes: layout.size,
                    align: layout.align,
                })
            });
        }
        representation(self.types, ty).map(Representation::Scalar)
    }
    pub fn demand(&mut self, definition: DefId) -> Result<SymbolId, &'static str> {
        if let Some(Some(symbol)) = self.symbols.get(definition.0 as usize) {
            return Ok(*symbol);
        }
        let index = self
            .body_indices
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .ok_or("missing called function body")?;
        let Owner::Function { signature, .. } = self.bodies.bodies[index].owner else {
            return Err("call target is not a function");
        };
        let signature = self.signature(signature)?;
        let symbol = self
            .output
            .declare(format!("rg.{}", definition.0), signature, false)?;
        self.symbols[definition.0 as usize] = Some(symbol);
        self.queue.push(definition);
        Ok(symbol)
    }
    pub fn abort(&mut self) -> Result<SymbolId, &'static str> {
        if let Some(symbol) = self.abort {
            return Ok(symbol);
        }
        let signature = self.output.signature(&[], Type::Void, true)?;
        let symbol = self.output.declare("abort".into(), signature, false)?;
        self.abort = Some(symbol);
        Ok(symbol)
    }
    pub fn remainder(&mut self, ty: Type) -> Result<SymbolId, &'static str> {
        let index = match ty {
            Type::F32 => 0,
            Type::F64 => 1,
            _ => return Err("nonfloating remainder helper"),
        };
        if let Some(symbol) = self.remainders[index] {
            return Ok(symbol);
        }
        let signature = self.output.signature(&[ty, ty], ty, false)?;
        let symbol = self
            .output
            .declare(format!("rg.rem.{}", ty.name()), signature, false)?;
        self.output.symbols[symbol.0 as usize].body = Some(remainder::build(ty)?);
        self.remainders[index] = Some(symbol);
        Ok(symbol)
    }
    fn entry(&mut self, root: SymbolId) -> Result<(), &'static str> {
        let signature = self.output.signature(&[], Type::I64, false)?;
        let symbol = self.output.declare("main".into(), signature, true)?;
        let target = self.output.symbols[root.0 as usize].signature;
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        self.initialize_addresses(&mut builder)?;
        builder.call(target, Type::Void, Operand::Symbol(root), &[])?;
        if self.output.signatures[target.0 as usize].noreturn {
            builder.terminate(Terminator::Unreachable)?;
        } else {
            let zero = builder.integer(Type::I64, 0)?;
            builder.terminate(Terminator::Return(Some(zero)))?;
        }
        self.output.symbols[symbol.0 as usize].body = Some(builder.finish()?);
        Ok(())
    }
}
pub(super) fn representation(types: &Store, ty: TypeId) -> Result<Type, &'static str> {
    if ty == TypeId::UNIT || ty == TypeId::NEVER {
        return Ok(Type::Void);
    }
    Ok(match types.kind(ty).ok_or("missing canonical type")? {
        Kind::FunctionItem { .. } => Type::Void,
        Kind::Function { .. } | Kind::Pointer { .. } => Type::Ptr,
        Kind::Primitive(p) => match p {
            Primitive::Bool => Type::I1,
            Primitive::Char | Primitive::U32 => Type::U32,
            Primitive::I8 => Type::I8,
            Primitive::U8 => Type::U8,
            Primitive::I16 => Type::I16,
            Primitive::U16 => Type::U16,
            Primitive::I32 => Type::I32,
            Primitive::I64 | Primitive::U64 | Primitive::Isize | Primitive::Usize => Type::I64,
            Primitive::I128 | Primitive::U128 => Type::I128,
            Primitive::F32 => Type::F32,
            Primitive::F64 => Type::F64,
            Primitive::Str => return Err("unsized values require later lowering"),
        },
        Kind::Reference { referent, .. } => {
            if types.is_sized(referent) != Some(true) {
                return Err("unsized pointer metadata requires later layout support");
            }
            Type::Ptr
        }
        _ => return Err("aggregate values require layout lowering"),
    })
}
