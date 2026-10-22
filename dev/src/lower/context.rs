//! Demand and ABI identities are indexed by already selected semantic definitions/types.
mod adapters;
mod c_boundary;
use super::{Error, data, function, remainder};
use crate::support::hash::HashMap;
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

pub(super) struct Context<'a> {
    pub intrinsics: &'a crate::resolve::model::Intrinsics,
    pub intrinsic_symbols: HashMap<Instance, SymbolId>,
    pub types: &'a Store,
    pub aggregates: &'a Definitions,
    pub members: &'a Members,
    pub constants: Reader<'a>,
    pub definitions: &'a [Option<TypeId>],
    pub initializers: &'a [Initializer],
    pub(super) foreign: Option<&'a crate::sema::foreign::Registry>,
    pub(super) crates: Option<&'a crate::sema::crates::Linkage>,
    imported_symbols: HashMap<DefId, SymbolId>,
    pub layouts: &'a mut Layouts,
    bodies: Bodies<'a>,
    pub destruction: Option<&'a crate::borrow::destruction::Proof>,
    /// Owned body plans, released with their body after emission.
    plans: Option<Plans>,
    /// Function lowering vectors reused across bodies.
    pub(super) lowering: function::Scratch,
    pub output: Program,
    pub data: data::Storage,
    pub storage_mutation: Option<crate::types::interior_mutability::Query<'a>>,
    body_indices: Vec<Option<usize>>,
    closure_indices: HashMap<crate::types::closure::ClosureId, usize>,
    adapter_indices: HashMap<crate::types::closure::Adapter, usize>,
    instance_indices: HashMap<Instance, usize>,
    symbols: Vec<Option<SymbolId>>,
    signatures: HashMap<TypeId, SignatureId>,
    representations: Vec<Option<Representation>>,
    c_abi: Option<Box<crate::abi::classification::Plans>>,
    queue: Vec<usize>,
    pub drop_queue: Vec<crate::body::drops::glue::Id>,
    pub drop_symbols: Vec<Option<SymbolId>>,
    pub objects: &'a crate::sema::vtable_plans::Objects,
    pub virtual_symbols: HashMap<(crate::sema::vtable_plans::ShapeId, u32), SymbolId>,
    pub object_callables: Option<Box<super::virtual_dispatch::Callables>>,
    pub table_symbols: HashMap<crate::sema::vtable_plans::TableId, crate::lowir::data::GlobalId>,
    pub table_queue: Vec<crate::sema::vtable_plans::TableId>,
    abort: Option<SymbolId>,
    remainders: [Option<SymbolId>; 2],
}
/// Owned per-body cleanup plans, indexed like the checked bodies.
type Plans = Vec<Option<Box<crate::borrow::destruction::Plan>>>;
/// Checked bodies in final proof order. Lowering consumes an owned body once and
/// leaves only the owner, instance and source header that later demands read.
enum Bodies<'a> {
    Shared(&'a [ir::Body]),
    Owned(Vec<ir::Body>),
}
#[expect(
    clippy::large_enum_variant,
    reason = "one short-lived local per lowered body moves the owned CFG without another allocation"
)]
enum Taken<'a> {
    Shared(&'a ir::Body),
    Owned(ir::Body),
}
impl std::ops::Deref for Taken<'_> {
    type Target = ir::Body;
    fn deref(&self) -> &ir::Body {
        match self {
            Self::Shared(body) => body,
            Self::Owned(body) => body,
        }
    }
}
impl<'a> Bodies<'a> {
    fn as_slice(&self) -> &[ir::Body] {
        match self {
            Self::Shared(bodies) => bodies,
            Self::Owned(bodies) => bodies,
        }
    }
    fn take(&mut self, index: usize) -> Option<Taken<'a>> {
        match self {
            Self::Shared(bodies) => {
                let bodies: &'a [ir::Body] = bodies;
                bodies.get(index).map(Taken::Shared)
            }
            Self::Owned(bodies) => {
                let slot = bodies.get_mut(index)?;
                let header = slot.header();
                Some(Taken::Owned(std::mem::replace(slot, header)))
            }
        }
    }
}
pub(super) fn build(
    facts: &mut Declarations,
    checked: &crate::borrow::Checked<'_>,
    roots: super::roots::Selected<'_>,
) -> Result<Program, Error> {
    let bodies = Bodies::Shared(&checked.program().bodies);
    run(facts, bodies, checked.destruction(), None, roots)
}
/// Lower owned proved bodies, releasing each body and plan after emission.
pub(super) fn build_owned(
    facts: &mut Declarations,
    bodies: Vec<ir::Body>,
    destruction: Option<&crate::borrow::destruction::Proof>,
    plans: Option<Plans>,
    roots: super::roots::Selected<'_>,
) -> Result<Program, Error> {
    run(facts, Bodies::Owned(bodies), destruction, plans, roots)
}
fn run(
    facts: &mut Declarations,
    bodies: Bodies<'_>,
    destruction: Option<&crate::borrow::destruction::Proof>,
    plans: Option<Plans>,
    roots: super::roots::Selected<'_>,
) -> Result<Program, Error> {
    let mut layouts = std::mem::take(&mut facts.layouts);
    let result = Context::build(facts, &mut layouts, (bodies, destruction, plans), roots);
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
    fn build(
        facts: &'a Declarations,
        layouts: &'a mut Layouts,
        (bodies, destruction, plans): (
            Bodies<'a>,
            Option<&'a crate::borrow::destruction::Proof>,
            Option<Plans>,
        ),
        roots: super::roots::Selected<'_>,
    ) -> Result<Program, Error> {
        let source = bodies.as_slice().first().map_or(
            NodeKey {
                file: FileId(0),
                node: NodeId(0),
            },
            |b| b.source,
        );
        let size = bodies
            .as_slice()
            .iter()
            .map(|body| match body.owner {
                Owner::AnonymousConstant
                | Owner::AnonymousInstance(_)
                | Owner::InitializerInstance { .. }
                | Owner::Closure { .. }
                | Owner::ClosureAdapter { .. } => 0,
                Owner::Function { definition, .. } | Owner::Initializer { definition, .. } => {
                    definition.0 as usize + 1
                }
            })
            .max()
            .unwrap_or(0);
        let mut context = Self {
            intrinsics: &facts.intrinsics,
            intrinsic_symbols: HashMap::default(),
            types: &facts.types,
            aggregates: &facts.aggregates,
            members: &facts.members,
            constants: Reader::new(facts),
            definitions: &facts.definitions,
            initializers: &facts.initializers,
            foreign: facts.foreign.as_deref(),
            crates: facts.crates.as_deref(),
            imported_symbols: HashMap::default(),
            layouts,
            symbols: vec![None; bodies.as_slice().len()],
            bodies,
            destruction,
            plans,
            lowering: function::Scratch::default(),
            output: Program::default(),
            data: data::Storage::default(),
            storage_mutation: None,
            body_indices: vec![None; size],
            closure_indices: HashMap::default(),
            adapter_indices: HashMap::default(),
            instance_indices: HashMap::default(),
            signatures: HashMap::default(),
            representations: Vec::new(),
            c_abi: None,
            queue: Vec::new(),
            drop_queue: Vec::new(),
            drop_symbols: vec![None; destruction.map_or(0, |d| d.recipes().recipes().len())],
            objects: &facts.objects,
            virtual_symbols: HashMap::default(),
            object_callables: None,
            table_symbols: HashMap::default(),
            table_queue: Vec::new(),
            abort: None,
            remainders: [None; 2],
        };
        context.index_bodies().map_err(|message| Error {
            node: source,
            message,
        })?;
        for definition in roots.functions.iter() {
            context
                .demand_definition(definition)
                .map_err(|message| Error {
                    node: source,
                    message,
                })?;
        }
        for &definition in roots.globals {
            context.static_data(definition).map_err(|message| Error {
                node: source,
                message,
            })?;
        }
        context.emit_pending(source)?;
        if let Some(entry) = roots.functions.entry {
            let root = context.demand_definition(entry).map_err(|message| Error {
                node: source,
                message,
            })?;
            context.entry(root).map_err(|message| Error {
                node: source,
                message,
            })?;
        } else if context.requires_address_initialization() {
            return Err(Error {
                node: source,
                message: "entry-free library requires packed address initialization support",
            });
        }
        Ok(context.output)
    }
    /// Demand queues own their identities before any dependency is emitted.
    /// Newly discovered functions, drop glue and data append without recursion.
    fn emit_pending(&mut self, source: NodeKey) -> Result<(), Error> {
        let destruction = self.destruction;
        let (mut next, mut next_data, mut next_drop, mut next_table) = (0, 0, 0, 0);
        while next < self.queue.len()
            || next_data < self.data.pending.len()
            || next_drop < self.drop_queue.len()
            || next_table < self.table_queue.len()
        {
            if next == self.queue.len() {
                if next_drop < self.drop_queue.len() {
                    let id = self.drop_queue[next_drop];
                    let lowered = super::cleanup::glue(self, id).map_err(|message| Error {
                        node: source,
                        message,
                    })?;
                    let symbol = self.drop_symbols[id.0 as usize].ok_or(Error {
                        node: source,
                        message: "missing demanded cleanup symbol",
                    })?;
                    self.output
                        .define(symbol, lowered)
                        .map_err(|message| Error {
                            node: source,
                            message,
                        })?;
                    next_drop += 1;
                    continue;
                }
                if next_table < self.table_queue.len() {
                    self.initialize_table(self.table_queue[next_table])
                        .map_err(|message| Error {
                            node: source,
                            message,
                        })?;
                    next_table += 1;
                    continue;
                }
                self.initialize_data(next_data).map_err(|message| Error {
                    node: source,
                    message,
                })?;
                next_data += 1;
                continue;
            }
            let index = self.queue[next];
            let symbol = self.symbols[index].ok_or(Error {
                node: source,
                message: "missing demanded symbol",
            })?;
            let body = self.bodies.take(index).ok_or(Error {
                node: source,
                message: "missing demanded body",
            })?;
            let owned_plan = self
                .plans
                .as_mut()
                .and_then(|plans| plans.get_mut(index))
                .and_then(Option::take);
            let plan = owned_plan
                .as_deref()
                .or_else(|| destruction.and_then(|proof| proof.plan(index)));
            let signature = self.output.symbols[symbol.0 as usize].signature;
            let lowered = function::lower(self, index, &body, plan, signature)?;
            self.output
                .define(symbol, lowered)
                .map_err(|message| Error {
                    node: body.source,
                    message,
                })?;
            // An owned body and plan are released here; only its header remains.
            next += 1;
        }
        Ok(())
    }
    fn index_bodies(&mut self) -> Result<(), &'static str> {
        for (index, body) in self.bodies.as_slice().iter().enumerate() {
            if let Owner::ClosureAdapter { adapter, .. } = body.owner {
                if body.instance != Some(adapter.instance())
                    || self.adapter_indices.insert(adapter, index).is_some()
                {
                    return Err("duplicate or inconsistent checked receiver adapter");
                }
                continue;
            }
            if let Owner::Closure { identity, .. } = body.owner {
                if let Some(instance) = body.instance {
                    if !matches!(self.types.kind(instance.ty()), Some(Kind::Closure { identity: selected, .. }) if selected == identity)
                        || self.instance_indices.insert(instance, index).is_some()
                    {
                        return Err("duplicate or inconsistent specialized closure body");
                    }
                } else if self.closure_indices.insert(identity, index).is_some() {
                    return Err("closure body requires a distinct source identity");
                }
                continue;
            }
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
    /// A checked body, or the header an already lowered owned body left behind.
    fn body(&self, index: usize) -> &ir::Body {
        &self.bodies.as_slice()[index]
    }
    pub fn signature(&mut self, ty: TypeId) -> Result<SignatureId, &'static str> {
        if let Some(id) = self.signatures.get(&ty) {
            return Ok(*id);
        }
        let Some(Kind::Function {
            bound_regions: _,
            parameters,
            result,
            unsafe_: _,
            convention,
        }) = self.types.kind(ty)
        else {
            return Err("missing selected function signature");
        };
        let mut native = Vec::new();
        let result_type = self.representation(result)?;
        if convention.abi == crate::types::callable::Abi::C
            && (matches!(result_type, Representation::Object(_))
                || self.non_scalar_boundary(result)?)
        {
            return self.object_signature(ty);
        }
        if matches!(result_type, Representation::Object(_)) {
            native.push(Type::Ptr);
        }
        for parameter in self
            .types
            .list(parameters)
            .ok_or("missing parameter list")?
        {
            let representation = self.representation(*parameter)?;
            if convention.abi == crate::types::callable::Abi::C
                && (matches!(representation, Representation::Object(_))
                    || self.non_scalar_boundary(*parameter)?)
            {
                return self.object_signature(ty);
            }
            let representation = representation.parameter();
            if representation != Type::Void {
                native.push(representation);
            }
        }
        let signature = self.output.signature_with_arity(
            &native,
            result_type.result(),
            result == TypeId::NEVER,
            convention.variadic,
        )?;
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
    /// Native value representation, memoized per canonical type: storage heads,
    /// layouts and pointer metadata are fixed once queried.
    pub fn representation(&mut self, ty: TypeId) -> Result<Representation, &'static str> {
        if let Some(&Some(known)) = self.representations.get(ty.0 as usize) {
            return Ok(known);
        }
        let known = self.represent(ty)?;
        let index = ty.0 as usize;
        if index >= self.representations.len() {
            self.representations
                .resize(index.max(self.types.len()) + 1, None);
        }
        self.representations[index] = Some(known);
        Ok(known)
    }
    fn represent(&mut self, ty: TypeId) -> Result<Representation, &'static str> {
        let ty = self.types.storage_head(ty)?;
        if let Some(
            Kind::Reference { referent, .. }
            | Kind::Pointer {
                pointee: referent, ..
            },
        ) = self.types.kind(ty)
        {
            if !self.wide_referent(referent)? {
                return Ok(Representation::Scalar(Type::Ptr));
            }
            let layout = self.layout(ty)?;
            return Ok(Representation::Object(MemorySpan {
                bytes: layout.size,
                align: layout.align,
            }));
        }
        if matches!(
            self.types.kind(ty),
            Some(Kind::Tuple(_) | Kind::Array { .. } | Kind::Adt(..) | Kind::Closure { .. })
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
    pub fn wide_referent(&mut self, ty: TypeId) -> Result<bool, &'static str> {
        use crate::abi::metadata::Metadata;
        match self
            .layouts
            .pointer_metadata(self.types, self.aggregates, ty)?
        {
            Metadata::Thin => Ok(false),
            Metadata::Length | Metadata::VTable(_) => Ok(true),
            Metadata::Unknown(_) => Err("lowering requires resolved pointer metadata"),
        }
    }
    pub fn demand_closure(&mut self, ty: TypeId) -> Result<SymbolId, &'static str> {
        if !matches!(
            self.types.kind(ty),
            Some(Kind::Closure {
                environment: TypeId::UNIT,
                ..
            })
        ) {
            return Err("closure requires a checked environment adapter");
        }
        self.closure_symbol(ty, None)
    }
    pub fn demand_closure_entry(
        &mut self,
        entry: crate::body::invocation_entries::Entry,
    ) -> Result<SymbolId, &'static str> {
        self.demand_captured(entry.target())
    }
    pub(in crate::lower) fn demand_captured(
        &mut self,
        entry: crate::sema::vtable_plans::Captured,
    ) -> Result<SymbolId, &'static str> {
        if entry.adapter.is_some() {
            return self.adapter_symbol(entry);
        }
        let requested = match entry.requested {
            crate::support::call_traits::CallTrait::Fn => crate::types::closure::CallKind::Shared,
            crate::support::call_traits::CallTrait::FnMut => {
                crate::types::closure::CallKind::Mutable
            }
            crate::support::call_traits::CallTrait::FnOnce => crate::types::closure::CallKind::Once,
        };
        let Some(Kind::Closure {
            identity,
            environment,
            ..
        }) = self.types.kind(entry.instance.ty())
        else {
            return Err("selected closure entry has no canonical identity");
        };
        if self.types.closure_kind(identity) != Some(requested) || environment == TypeId::UNIT {
            return Err("selected closure entry permission or environment differs");
        }
        self.closure_symbol(entry.instance.ty(), Some(entry.execution))
    }
    fn closure_symbol(
        &mut self,
        ty: TypeId,
        entry: Option<TypeId>,
    ) -> Result<SymbolId, &'static str> {
        let Some(Kind::Closure { identity, .. }) = self.types.kind(ty) else {
            return Err("closure requires a checked environment adapter");
        };
        let instance = Instance::closure(self.types, ty)?;
        let &index = self
            .instance_indices
            .get(&instance)
            .or_else(|| self.closure_indices.get(&identity))
            .ok_or("missing checked closure body")?;
        let Owner::Closure {
            signature,
            execution,
            ..
        } = self.body(index).owner
        else {
            return Err("closure body identity mismatch");
        };
        let signature = match (entry, execution) {
            (None, None) => signature,
            (Some(entry), Some(execution)) => {
                self.entry_abi(entry, execution)?;
                execution
            }
            _ => return Err("closure demand does not match checked receiver contract"),
        };
        if let Some(symbol) = self.symbols[index] {
            return Ok(symbol);
        }
        let signature = self.signature(signature)?;
        let name = if self.body(index).instance.is_some() {
            format!("rg.closure.{}.i{index}", identity.0)
        } else {
            format!("rg.closure.{}", identity.0)
        };
        let symbol = self
            .output
            .declare(self.internal_name(name), signature, false)?;
        self.symbols[index] = Some(symbol);
        self.queue.push(index);
        Ok(symbol)
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
        if matches!(self.types.kind(instance.ty()), Some(Kind::Closure { .. })) {
            return self.demand_closure(instance.ty());
        }
        if let Some(call) = self.objects.call(instance) {
            return self.virtual_method(instance, call);
        }
        let Some(Kind::FunctionItem { item, .. }) = self.types.kind(instance.ty()) else {
            return Err("invalid demanded callable instance");
        };
        let definition = DefId(item.0);
        if let Some(function) = self.foreign.and_then(|r| r.function(definition))
            && function.linkage == crate::support::linkage::Linkage::Import
        {
            if let Some(&symbol) = self.imported_symbols.get(&definition) {
                return Ok(symbol);
            }
            let signature = self.signature(function.signature)?;
            let name = super::linkage::select(
                self.foreign.map(crate::sema::foreign::Registry::names),
                function.naming,
                super::linkage::Identity::Callable(instance),
                |identity| self.rust_name(identity, None),
            )?;
            let symbol = self.output.declare(name, signature, false)?;
            self.imported_symbols.insert(definition, symbol);
            return Ok(symbol);
        }
        if let Some(intrinsic) = self.intrinsics.identify(definition) {
            return self.demand_intrinsic(intrinsic, instance);
        }
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
        let Owner::Function { signature, .. } = self.body(index).owner else {
            return Err("call target is not a function");
        };
        let signature = self.signature(signature)?;
        let name = super::linkage::select(
            self.foreign.map(crate::sema::foreign::Registry::names),
            self.foreign
                .and_then(|r| r.function(definition))
                .map_or(crate::support::linkage::Naming::Rust, |f| f.naming),
            super::linkage::Identity::Callable(instance),
            |identity| self.rust_name(identity, self.body(index).instance.map(|_| index)),
        )?;
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
        let symbol = self.output.declare(
            self.internal_name(format!("rg.rem.{}", ty.name())),
            signature,
            false,
        )?;
        self.output.define(symbol, remainder::build(ty)?)?;
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
        if self.output.noreturn(target)? {
            builder.terminate(Terminator::Unreachable)?;
        } else {
            let zero = builder.integer(Type::I64, 0)?;
            builder.terminate(Terminator::Return(Some(zero)))?;
        }
        self.output.define(symbol, builder.finish()?)?;
        Ok(())
    }
}
pub(super) fn representation(types: &Store, ty: TypeId) -> Result<Type, &'static str> {
    let ty = types.storage_head(ty)?;
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
