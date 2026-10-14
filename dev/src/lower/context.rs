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
    types::{Kind, Primitive, Store, TypeId, aggregate::Definitions, instance::Instance},
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
    pub destruction: Option<&'a crate::borrow::destruction::Proof>,
    pub output: Program,
    pub data: data::Storage,
    body_indices: Vec<Option<usize>>,
    instance_indices: HashMap<Instance, usize>,
    symbols: Vec<Option<SymbolId>>,
    signatures: HashMap<TypeId, SignatureId>,
    queue: Vec<usize>,
    pub drop_queue: Vec<crate::body::drops::glue::Id>,
    pub drop_symbols: Vec<Option<SymbolId>>,
    abort: Option<SymbolId>,
    remainders: [Option<SymbolId>; 2],
}
pub(super) fn build(
    facts: &mut Declarations,
    bodies: &crate::borrow::Checked<'_>,
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
        checked: &'a crate::borrow::Checked<'_>,
        entry: DefId,
    ) -> Result<Program, Error> {
        let bodies = checked.program();
        let destruction = checked.destruction();
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
                Owner::AnonymousConstant => 0,
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
            destruction,
            output: Program::default(),
            data: data::Storage::default(),
            body_indices: vec![None; size],
            instance_indices: HashMap::new(),
            symbols: vec![None; bodies.bodies.len()],
            signatures: HashMap::new(),
            queue: Vec::new(),
            drop_queue: Vec::new(),
            drop_symbols: vec![None; destruction.map_or(0, |d| d.recipes().recipes().len())],
            abort: None,
            remainders: [None; 2],
        };
        context.index_bodies().map_err(|message| Error {
            node: source,
            message,
        })?;
        let root = context.demand_definition(entry).map_err(|message| Error {
            node: source,
            message,
        })?;
        let (mut next, mut next_data, mut next_drop) = (0, 0, 0);
        while next < context.queue.len()
            || next_data < context.data.pending.len()
            || next_drop < context.drop_queue.len()
        {
            if next == context.queue.len() {
                if next_drop < context.drop_queue.len() {
                    let id = context.drop_queue[next_drop];
                    let lowered =
                        super::cleanup::glue(&mut context, id).map_err(|message| Error {
                            node: source,
                            message,
                        })?;
                    let symbol = context.drop_symbols[id.0 as usize].ok_or(Error {
                        node: source,
                        message: "missing demanded cleanup symbol",
                    })?;
                    context.output.symbols[symbol.0 as usize].body = Some(lowered);
                    next_drop += 1;
                    continue;
                }
                context
                    .initialize_data(next_data)
                    .map_err(|message| Error {
                        node: source,
                        message,
                    })?;
                next_data += 1;
                continue;
            }
            let index = context.queue[next];
            let symbol = context.symbols[index].ok_or(Error {
                node: source,
                message: "missing demanded symbol",
            })?;
            let body = &bodies.bodies[index];
            let plan = destruction.and_then(|proof| proof.plan(index));
            let lowered = function::lower(&mut context, index, body, plan)?;
            context.output.symbols[symbol.0 as usize].body = Some(lowered);
            next += 1;
        }
        context.entry(root).map_err(|message| Error {
            node: source,
            message,
        })?;
        Ok(context.output)
    }
    fn index_bodies(&mut self) -> Result<(), &'static str> {
        for (index, body) in self.bodies.bodies.iter().enumerate() {
            let Owner::Function {
                definition,
                signature,
            } = body.owner
            else {
                if body.instance.is_some() {
                    return Err("non-function body has a callable instance");
                }
                continue;
            };
            if let Some(instance) = body.instance {
                if !matches!(self.types.kind(instance.ty()), Some(Kind::FunctionItem { item, signature: selected, .. }) if item.0 == definition.0 && selected == signature)
                {
                    return Err("specialized body differs from its callable instance");
                }
                if self.instance_indices.insert(instance, index).is_some() {
                    return Err("duplicate specialized callable body");
                }
            } else if self.body_indices[definition.0 as usize]
                .replace(index)
                .is_some()
            {
                return Err("duplicate function body");
            }
        }
        Ok(())
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
        if let Some(
            Kind::Reference { referent, .. }
            | Kind::Pointer {
                pointee: referent, ..
            },
        ) = self.types.kind(ty)
            && matches!(
                self.types.kind(referent),
                Some(Kind::Slice(_) | Kind::Primitive(Primitive::Str))
            )
        {
            let layout = self.layout(ty)?;
            return Ok(Representation::Object(MemorySpan {
                bytes: layout.size,
                align: layout.align,
            }));
        }
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
    pub fn demand_definition(&mut self, definition: DefId) -> Result<SymbolId, &'static str> {
        let ty = self
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .ok_or("missing demanded function definition")?;
        self.demand(Instance::new(self.types, ty)?)
    }
    pub fn demand(&mut self, instance: Instance) -> Result<SymbolId, &'static str> {
        let Some(Kind::FunctionItem { item, .. }) = self.types.kind(instance.ty()) else {
            return Err("invalid demanded callable instance");
        };
        let definition = DefId(item.0);
        let index = if let Some(&index) = self.instance_indices.get(&instance) {
            index
        } else {
            if self
                .definitions
                .get(definition.0 as usize)
                .copied()
                .flatten()
                .is_some_and(|ty| self.types.contains_parameters(ty) == Some(true))
            {
                return Err("generic callable execution requires demanded instances");
            }
            self.body_indices
                .get(definition.0 as usize)
                .copied()
                .flatten()
                .ok_or("missing called function body")?
        };
        if let Some(symbol) = self.symbols[index] {
            return Ok(symbol);
        }
        let Owner::Function { signature, .. } = self.bodies.bodies[index].owner else {
            return Err("call target is not a function");
        };
        let signature = self.signature(signature)?;
        let name = if self.bodies.bodies[index].instance.is_some() {
            format!("rg.{}.i{index}", definition.0)
        } else {
            format!("rg.{}", definition.0)
        };
        let symbol = self.output.declare(name, signature, false)?;
        self.symbols[index] = Some(symbol);
        self.queue.push(index);
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
