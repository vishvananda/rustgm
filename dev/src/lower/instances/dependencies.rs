//! Callable demands come from executable uses and typed constant storage graphs.
use crate::support::hash::HashSet;
use crate::{
    abi::layout::Layouts,
    body::{
        self,
        ir::{Body, Rvalue},
    },
    sema::{
        model::{Cast, Declarations},
        values::{Constant, LocationId, Origin, Reader, View},
    },
    types::{Kind, TypeId, instance::Instance},
};

#[derive(Default)]
pub(super) struct Dependencies {
    values: HashSet<(TypeId, Constant)>,
    locations: HashSet<LocationId>,
    steps: usize,
}
enum Work {
    Value(TypeId, Constant),
    Location(LocationId),
}
pub(super) struct Demands {
    pub adapters: Vec<crate::body::invocation_entries::Entry>,
    pub closures: Vec<TypeId>,
    pub calls: Vec<Instance>,
    pub object_types: Vec<TypeId>,
}
#[derive(Default)]
struct Discovered {
    adapters: Vec<crate::body::invocation_entries::Entry>,
    closures: Vec<TypeId>,
    calls: Vec<Instance>,
    tables: Vec<(TypeId, TypeId)>,
    captured: Vec<(TypeId, [Option<crate::body::invocation_entries::Entry>; 2])>,
    upcasts: Vec<(TypeId, TypeId)>,
    operators: Vec<(TypeId, crate::support::call_traits::CallTrait, TypeId)>,
}
impl Discovered {
    fn conversion(
        &mut self,
        body: &Body,
        index: usize,
        conversion: crate::types::unsizing::Conversion<TypeId>,
    ) -> Result<(), &'static str> {
        match conversion {
            crate::types::unsizing::Conversion::Object { concrete, target } => {
                self.tables.push((concrete, target));
                let id = crate::body::ir::ValueId(
                    u32::try_from(index).map_err(|_| "too many conversion dependencies")?,
                );
                if let Some(entries) = body
                    .closure_calls
                    .as_deref()
                    .and_then(|entries| entries.conversion(id))
                {
                    self.captured.push((concrete, entries));
                }
            }
            crate::types::unsizing::Conversion::Upcast { source, target } => {
                self.upcasts.push((source, target));
            }
            crate::types::unsizing::Conversion::Slice(_) => {}
        }
        Ok(())
    }
    fn callable(
        &mut self,
        types: &crate::types::Store,
        body: &Body,
        ty: TypeId,
    ) -> Result<(), &'static str> {
        match types.kind(ty) {
            Some(Kind::FunctionItem { .. }) => {
                self.calls
                    .push(body.callable_target(Instance::new(types, ty)?));
            }
            Some(Kind::Closure { .. }) => self.closures.push(ty),
            _ => {}
        }
        Ok(())
    }
}
impl Dependencies {
    pub(super) fn collect(
        &mut self,
        report: &crate::resolve::expansion::Report,
        facts: &mut Declarations,
        body: &Body,
    ) -> Result<Demands, String> {
        let mut layouts = std::mem::take(&mut facts.layouts);
        let result = self.run(facts, &mut layouts, body);
        facts.layouts = layouts;
        let result = result?;
        Self::prepared(report, facts, result)
    }
    pub(super) fn global(
        &mut self,
        report: &crate::resolve::expansion::Report,
        facts: &mut Declarations,
        definition: crate::resolve::model::DefId,
    ) -> Result<Demands, String> {
        let value = Reader::new(facts)
            .definition(definition)
            .ok_or("missing completed global export value")?;
        let ty = facts
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .ok_or("missing global export type")?;
        let mut layouts = std::mem::take(&mut facts.layouts);
        let mut result = Discovered::default();
        let evaluated = self.follow(
            facts,
            &mut layouts,
            vec![Work::Value(ty, value)],
            &mut result,
        );
        facts.layouts = layouts;
        evaluated?;
        Self::prepared(report, facts, result)
    }
    fn prepared(
        report: &crate::resolve::expansion::Report,
        facts: &mut Declarations,
        mut result: Discovered,
    ) -> Result<Demands, String> {
        let mut concrete = Vec::new();
        let mut object_types = Vec::new();
        for instance in result.calls {
            if let Some(selection) = crate::sema::compiler_items::source_drop_glue(facts, instance)?
            {
                object_types.push(selection.element);
            }
            if matches!(facts.types.kind(instance.ty()), Some(Kind::FunctionItem { item, .. })
                if facts.intrinsics.identify(crate::resolve::model::DefId(item.0)).is_some())
            {
                continue;
            }
            if !crate::sema::vtable_plans::prepare_call(report, facts, instance)? {
                concrete.push(instance);
            }
        }
        for (concrete, entries) in result.captured {
            for entry in entries.into_iter().flatten() {
                if entry.adapter().is_some() {
                    result.adapters.push(entry);
                } else {
                    result.closures.push(entry.instance().ty());
                }
            }
            crate::sema::vtable_plans::prepare_captured(
                facts,
                concrete,
                entries.map(|entry| entry.map(crate::body::invocation_entries::Entry::target)),
            )?;
        }
        for (receiver, kind, signature) in result.operators {
            crate::sema::vtable_plans::prepare_operator(report, facts, receiver, kind, signature)?;
        }
        for (source, target) in result.tables {
            crate::sema::vtable_plans::prepare_table(report, facts, source, target)?;
        }
        for (source, target) in result.upcasts {
            crate::sema::vtable_plans::prepare_upcast(report, facts, source, target)?;
        }
        for id in crate::sema::vtable_plans::complete_tables(facts)? {
            let table = facts
                .objects
                .table(id)
                .ok_or("missing demanded object table")?;
            for target in table.methods.iter().flatten() {
                match *target {
                    crate::sema::vtable_plans::Target::Function(instance) => {
                        concrete.push(instance);
                    }
                    crate::sema::vtable_plans::Target::Callable(callable) => {
                        match facts.types.kind(callable.concrete) {
                            Some(Kind::FunctionItem { .. }) => {
                                concrete.push(Instance::new(&facts.types, callable.concrete)?);
                            }
                            Some(Kind::Closure { .. }) => result.closures.push(callable.concrete),
                            _ => {}
                        }
                    }
                    crate::sema::vtable_plans::Target::Captured(_) => {}
                }
            }
            object_types.push(table.concrete);
        }
        Ok(Demands {
            adapters: result.adapters,
            closures: result.closures,
            calls: concrete,
            object_types,
        })
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > 1_000_000 {
            return Err("callable dependency work budget exceeded");
        }
        Ok(())
    }
    fn run(
        &mut self,
        facts: &Declarations,
        layouts: &mut Layouts,
        body: &Body,
    ) -> Result<Discovered, &'static str> {
        let reader = Reader::new(facts);
        let mut result = Discovered::default();
        let mut work = Vec::new();
        self.charge(body.values.len())?;
        for (index, value) in body.values.iter().enumerate() {
            if let Rvalue::Unsize { conversion, .. } = value.kind {
                result.conversion(body, index, conversion)?;
            }
            if let Rvalue::Invoke {
                kind,
                signature,
                arguments,
            } = value.kind
            {
                let id = crate::body::ir::ValueId(
                    u32::try_from(index).map_err(|_| "too many callable values")?,
                );
                self.invocation(facts, body, id, (kind, signature, arguments), &mut result)?;
            }
            let operand = match value.kind {
                Rvalue::Reify(id)
                | Rvalue::Cast {
                    kind: Cast::FunctionPointer,
                    operand: id,
                    ..
                }
                | Rvalue::Call { callee: id, .. } => Some(id),
                _ => None,
            };
            if let Some(id) = operand {
                let ty = body
                    .values
                    .get(id.0 as usize)
                    .ok_or("invalid callable operand")?
                    .ty;
                result.callable(&facts.types, body, ty)?;
            }
        }
        for (id, static_) in body::constants::references(body) {
            self.charge(1)?;
            if static_
                && facts
                    .foreign
                    .as_deref()
                    .is_some_and(|r| r.imports_global(id))
            {
                continue;
            }
            let value = reader
                .definition(id)
                .ok_or("missing completed callable constant")?;
            let ty = facts
                .definitions
                .get(id.0 as usize)
                .copied()
                .flatten()
                .ok_or("missing callable constant type")?;
            work.push(Work::Value(ty, value));
        }
        for value in &body.values {
            if let Rvalue::SourceAssociated(source) = value.kind {
                self.charge(1)?;
                let (_, constant) = reader.source_associated(source)?;
                work.push(Work::Value(value.ty, constant));
            }
            if let Rvalue::AssociatedConstant(instance) = value.kind {
                self.charge(1)?;
                work.push(Work::Value(
                    value.ty,
                    reader
                        .associated(instance)
                        .ok_or("missing completed associated constant")?,
                ));
            }
        }
        if let Some(promotions) = &body.promotions {
            for (id, value) in promotions.entries() {
                self.charge(1)?;
                let ty = body
                    .values
                    .get(id.0 as usize)
                    .ok_or("invalid promoted callable value")?
                    .ty;
                work.push(Work::Value(ty, value));
            }
        }
        self.follow(facts, layouts, work, &mut result)?;
        Ok(result)
    }
    fn invocation(
        &mut self,
        facts: &Declarations,
        body: &Body,
        id: crate::body::ir::ValueId,
        invocation: (
            crate::support::call_traits::CallTrait,
            TypeId,
            crate::body::ir::ValueList,
        ),
        result: &mut Discovered,
    ) -> Result<(), &'static str> {
        if let Some(entry) = body
            .closure_calls
            .as_deref()
            .and_then(|entries| entries.get(id))
            && entry.adapter().is_some()
        {
            result.adapters.push(entry);
            return Ok(());
        }
        let (kind, signature, arguments) = invocation;
        let receiver = *body
            .arguments(arguments)
            .first()
            .ok_or("missing call operator receiver")?;
        let receiver = body.values[receiver.0 as usize].ty;
        let mut ty = facts.types.storage_head(receiver)?;
        while let Some(Kind::Reference { referent, .. }) = facts.types.kind(ty) {
            self.charge(1)?;
            ty = facts.types.storage_head(referent)?;
        }
        if matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })) {
            result.operators.push((receiver, kind, signature));
            return Ok(());
        }
        result.callable(&facts.types, body, ty)
    }
    fn image_dependencies(
        &mut self,
        reader: Reader<'_>,
        ty: TypeId,
        value: Constant,
        work: &mut Vec<Work>,
        result: &mut Discovered,
    ) -> Result<(), &'static str> {
        reader.require_publishable_array(value)?;
        let image = reader.image(value)?;
        if image.ty != ty {
            return Err("constant union image differs from dependency storage");
        }
        self.charge(image.fragments.len())?;
        for fragment in image.complete_words()? {
            match fragment.target {
                crate::sema::values::WordTarget::Value(value) => match reader.view(value)? {
                    View::Reference(id) => work.push(Work::Location(id)),
                    View::Function(instance) => result.calls.push(instance),
                    _ => return Err("constant image has an invalid dependency word"),
                },
                crate::sema::values::WordTarget::Vtable { concrete, target } => {
                    result.tables.push((concrete, target));
                }
            }
        }
        Ok(())
    }
    fn follow(
        &mut self,
        facts: &Declarations,
        layouts: &mut Layouts,
        mut work: Vec<Work>,
        result: &mut Discovered,
    ) -> Result<(), &'static str> {
        let reader = Reader::new(facts);
        while let Some(next) = work.pop() {
            self.charge(1)?;
            match next {
                Work::Location(id) => {
                    if !self.locations.insert(id) {
                        continue;
                    }
                    let location = reader.location(id)?;
                    match location.origin {
                        Origin::Projection { base, .. }
                        | Origin::Subsequence { base, .. }
                        | Origin::ByteOffset { base, .. } => {
                            work.push(Work::Location(base));
                        }
                        Origin::Allocation => work.push(Work::Value(
                            location.ty,
                            location.value.ok_or("missing allocation value")?,
                        )),
                        Origin::Static(id) => {
                            if !facts
                                .foreign
                                .as_deref()
                                .is_some_and(|r| r.imports_global(id))
                            {
                                work.push(Work::Value(
                                    location.ty,
                                    reader.definition(id).ok_or("missing static value")?,
                                ));
                            }
                        }
                    }
                }
                Work::Value(ty, value) => {
                    if !self.values.insert((ty, value)) {
                        continue;
                    }
                    if let Some(crate::sema::values::Metadata::Object { concrete, target }) =
                        value.metadata()
                    {
                        result.tables.push((concrete, target));
                    }
                    let layout = layouts
                        .demand(&facts.types, &facts.aggregates, ty)
                        .map_err(|e| e.message)?;
                    if layouts.at(layout).ok_or("missing constant layout")?.size == 0 {
                        continue;
                    }
                    match reader.view(value)? {
                        View::Function(instance) => result.calls.push(instance),
                        View::Image { .. } => {
                            self.image_dependencies(reader, ty, value, &mut work, result)?;
                        }
                        View::Reference(id) => work.push(Work::Location(id)),
                        View::Repeat { value, length } if length != 0 => {
                            let Some(Kind::Array { element, .. }) = facts.types.kind(ty) else {
                                return Err("repeated constant is not an array");
                            };
                            work.push(Work::Value(element, value));
                        }
                        View::Product { length, variant } => {
                            self.charge(
                                usize::try_from(length)
                                    .map_err(|_| "constant product exceeds host size")?,
                            )?;
                            for index in (0..length).rev() {
                                let child = field(facts, ty, variant, index)?;
                                work.push(Work::Value(child, reader.field(value, index)?));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
fn field(
    facts: &Declarations,
    ty: TypeId,
    variant: Option<crate::sema::members::VariantId>,
    index: u64,
) -> Result<TypeId, &'static str> {
    let index = usize::try_from(index).map_err(|_| "constant field exceeds host size")?;
    match facts.types.kind(ty) {
        Some(Kind::Array { element, .. }) => Ok(element),
        Some(Kind::Tuple(list)) => facts
            .types
            .list(list)
            .and_then(|l| l.get(index))
            .copied()
            .ok_or("invalid tuple constant field"),
        Some(Kind::Adt(item, _)) => {
            let ordinal = if let Some(id) = variant {
                let variant = facts
                    .members
                    .variants
                    .get(id.0 as usize)
                    .ok_or("invalid constant variant")?;
                if variant.owner != item {
                    return Err("constant variant belongs to another type");
                }
                variant.ordinal as usize
            } else {
                0
            };
            facts
                .aggregates
                .shape(&facts.types, ty)?
                .variant(ordinal)
                .and_then(|variant| facts.types.list(variant.fields))
                .and_then(|l| l.get(index))
                .copied()
                .ok_or("invalid nominal constant field")
        }
        _ => Err("constant product has no field types"),
    }
}
