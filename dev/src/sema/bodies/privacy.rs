//! Semantic type visibility belongs to the physical body, independently of hygiene.
use super::{Checker, Declarations, Error, TypeId};
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, BuiltinTrait, DefId, DefKind, Target},
        package::ModuleId,
    },
    support::hash::HashSet,
    types::{
        GenericArgs, ItemId, Kind, ListId,
        arguments::Argument,
        opaque::OpaqueId,
        parameters::{clauses::Clause, predicates::Bound},
        traits::ApplicationId,
    },
};

struct Privacy<'a> {
    report: &'a Report,
    facts: &'a Declarations,
    module: ModuleId,
    ancestors: Option<HashSet<ModuleId>>,
    visited: HashSet<TypeId>,
    opaques: HashSet<OpaqueId>,
    work: Vec<TypeId>,
    steps: usize,
}

#[cfg(test)]
#[path = "../../../tests/invariants/physical_privacy.rs"]
mod tests;

impl Checker<'_> {
    pub(super) fn validate_type_privacy(&self) -> Result<(), Error> {
        let mut privacy = Privacy::new(self.report, self.facts, self.state.module);
        // Patterns and written annotations enter the same retained touched list.
        // Each graph node is inspected once in this immutable physical module.
        for &node in &self.state.touched {
            let key = self.key(node);
            let result = privacy.node(key);
            result.map_err(|message| self.error(node, message))?;
        }
        Ok(())
    }
}

impl<'a> Privacy<'a> {
    fn new(report: &'a Report, facts: &'a Declarations, module: ModuleId) -> Self {
        Self {
            report,
            facts,
            module,
            ancestors: None,
            visited: HashSet::default(),
            opaques: HashSet::default(),
            work: Vec::new(),
            steps: 0,
        }
    }
    fn charge(&mut self, amount: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(amount);
        if self.steps > 1_000_000 {
            return Err("semantic type privacy work budget exceeded");
        }
        Ok(())
    }

    fn node(&mut self, key: crate::resolve::unit::NodeKey) -> Result<(), &'static str> {
        let facts = self.facts;
        let node = facts.node(key).ok_or("missing typed privacy use")?;
        self.check(node.ty.ok_or("missing privacy use type")?)?;
        if let Some(adjustment) = node.adjustment {
            self.check(adjustment.target)?;
        }
        if let Some(adjustment) = facts.reference_adjustments.get(&key) {
            self.check(adjustment.borrow)?;
        }
        if let Some(&ty) = facts.pointer_adjustments.get(&key) {
            self.check(ty)?;
        }
        if let Some(&conversion) = facts.unsizing_adjustments.get(&key) {
            conversion.try_map(|ty| self.check(ty))?;
        }
        if let Some(call) = facts.methods.calls.get(&key) {
            self.item(ItemId(call.method.0))?;
            self.check(call.callable)?;
            self.check(call.receiver)?;
            if let Some(borrow) = call.borrow {
                self.check(borrow)?;
            }
        }
        if let Some(instance) = facts.constant_uses.get(&key)
            && facts.associated.owner(instance.definition).is_some()
        {
            self.item(ItemId(instance.definition.0))?;
            self.arguments(instance.arguments)?;
            self.drain()?;
        }
        if let Some(Target::Definition(id)) = node.reference
            && facts.associated.owner(id).is_some()
            && self
                .report
                .resolution
                .definitions
                .get(id.0 as usize)
                .is_some_and(|item| {
                    matches!(
                        item.kind,
                        DefKind::Function | DefKind::Const | DefKind::TypeAlias
                    )
                })
        {
            self.item(ItemId(id.0))?;
        }
        Ok(())
    }

    fn item(&mut self, item: ItemId) -> Result<(), &'static str> {
        self.charge(1)?;
        let declaration = self
            .report
            .resolution
            .definitions
            .get(item.0 as usize)
            .ok_or("unknown semantic privacy declaration")?;
        let accessible = match declaration.access {
            Access::Public => true,
            Access::Within(ancestor) => self.within(ancestor)?,
            Access::ExternalPrivate => false,
        };
        if !accessible {
            return Err("semantic type or associated item is private");
        }
        Ok(())
    }

    fn within(&mut self, ancestor: ModuleId) -> Result<bool, &'static str> {
        if ancestor == self.module {
            return Ok(true);
        }
        if self.ancestors.is_none() {
            let mut ancestors = HashSet::default();
            let mut current = Some(self.module);
            while let Some(module) = current {
                self.charge(1)?;
                if !ancestors.insert(module) {
                    return Err("cycle in physical privacy module ancestry");
                }
                current = self
                    .report
                    .package
                    .modules
                    .get(module.0 as usize)
                    .ok_or("unknown physical privacy module")?
                    .parent;
            }
            self.ancestors = Some(ancestors);
        }
        Ok(self
            .ancestors
            .as_ref()
            .is_some_and(|values| values.contains(&ancestor)))
    }

    fn push(&mut self, ty: TypeId) -> Result<(), &'static str> {
        self.charge(1)?;
        let kind = self
            .facts
            .types
            .kind(ty)
            .ok_or("unknown semantic privacy type")?;
        // Primitive-only scalar bodies allocate no graph or visited storage.
        if !matches!(kind, Kind::Never | Kind::Primitive(_) | Kind::Parameter(_))
            && self.visited.insert(ty)
        {
            self.work.push(ty);
        }
        Ok(())
    }

    fn check(&mut self, ty: TypeId) -> Result<(), &'static str> {
        self.push(ty)?;
        self.drain()
    }

    fn drain(&mut self) -> Result<(), &'static str> {
        while let Some(ty) = self.work.pop() {
            let kind = self
                .facts
                .types
                .kind(ty)
                .ok_or("missing queued privacy type")?;
            match kind {
                Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => {}
                Kind::Foreign(item) => self.item(item)?,
                Kind::Adt(item, arguments) => {
                    self.item(item)?;
                    self.arguments(arguments)?;
                }
                Kind::FunctionItem {
                    item,
                    arguments,
                    signature,
                } => {
                    self.callable(item, arguments, signature)?;
                }
                Kind::Projection(application) => {
                    self.item(application.item)?;
                    self.push(application.subject)?;
                    self.trait_application(application.trait_application)?;
                    self.arguments(application.arguments)?;
                }
                Kind::Opaque(id, arguments) => {
                    self.arguments(arguments)?;
                    self.opaque_bounds(id)?;
                }
                Kind::Dynamic {
                    shape, arguments, ..
                } => {
                    self.dynamic(shape)?;
                    self.arguments(arguments)?;
                }
                Kind::Tuple(list) => self.list(list)?,
                Kind::Array { element, .. } | Kind::Slice(element) => self.push(element)?,
                Kind::ArrayTerm { element, length } => {
                    self.push(element)?;
                    self.push(
                        self.facts
                            .types
                            .const_type(length)
                            .ok_or("missing array length type")?,
                    )?;
                }
                Kind::Pattern { base: ty, .. }
                | Kind::Reference { referent: ty, .. }
                | Kind::Pointer { pointee: ty, .. } => {
                    self.push(ty)?;
                }
                Kind::Function {
                    parameters, result, ..
                } => {
                    self.list(parameters)?;
                    self.push(result)?;
                }
                Kind::Closure {
                    arguments,
                    signature,
                    environment,
                    ..
                } => {
                    self.arguments(arguments)?;
                    self.push(signature)?;
                    self.push(environment)?;
                }
            }
        }
        Ok(())
    }

    fn callable(
        &mut self,
        item: ItemId,
        arguments: GenericArgs,
        signature: TypeId,
    ) -> Result<(), &'static str> {
        self.item(item)?;
        self.arguments(arguments)?;
        self.push(signature)?;
        // Static inherent methods need the impl's Self type even when no
        // explicit argument or receiver carries that nominal application.
        if let Some(owner) = self
            .facts
            .associated
            .owner(DefId(item.0))
            .and_then(|owner| self.facts.associated.get(owner))
            && matches!(owner.kind, crate::sema::associated::OwnerKind::Impl { .. })
        {
            let schema = self
                .facts
                .associated
                .schema(DefId(item.0))
                .ok_or("missing method privacy owner schema")?;
            self.push(schema.self_type)?;
        }
        Ok(())
    }

    fn arguments(&mut self, arguments: GenericArgs) -> Result<(), &'static str> {
        let types = &self.facts.types;
        let arguments = types
            .arguments(arguments)
            .ok_or("missing privacy arguments")?;
        self.charge(arguments.len())?;
        for argument in arguments {
            match argument {
                Argument::Lifetime(_) => {}
                Argument::Type(ty) => self.push(ty)?,
                Argument::Const(id) => {
                    self.push(types.const_type(id).ok_or("missing const argument type")?)?;
                }
            }
        }
        Ok(())
    }

    fn list(&mut self, list: ListId) -> Result<(), &'static str> {
        let types = &self.facts.types;
        for &ty in types.list(list).ok_or("missing privacy type list")? {
            self.push(ty)?;
        }
        Ok(())
    }

    fn trait_application(&mut self, id: ApplicationId) -> Result<(), &'static str> {
        let application = self
            .facts
            .types
            .trait_application(id)
            .ok_or("missing privacy trait application")?;
        self.item(application.item)?;
        self.arguments(application.arguments)
    }

    fn dynamic(&mut self, id: crate::types::dynamic::ShapeId) -> Result<(), &'static str> {
        let types = &self.facts.types;
        let shape = types
            .dynamic_shape(id)
            .ok_or("missing privacy object shape")?;
        if let Some(principal) = shape.principal {
            self.item(principal)?;
        }
        for binding in types
            .dynamic_bindings(id)
            .ok_or("missing privacy object bindings")?
        {
            self.item(binding.trait_item)?;
        }
        for &marker in types
            .dynamic_auto_traits(id)
            .ok_or("missing privacy object markers")?
        {
            if let Some(item) = self
                .report
                .resolution
                .builtin_traits
                .get(BuiltinTrait::Auto(marker))
            {
                self.item(ItemId(item.0))?;
            }
        }
        Ok(())
    }

    fn opaque_bounds(&mut self, id: OpaqueId) -> Result<(), &'static str> {
        if !self.opaques.insert(id) {
            return Ok(());
        }
        let facts = self.facts;
        let declaration = facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("missing opaque privacy declaration")?;
        for &clause in &declaration.bounds {
            self.charge(1)?;
            // This inspects declaration identities, not a proof or instantiated
            // value. Bound lifetime coordinates do not change their visibility.
            let predicate = match clause {
                Clause::UnstableFeature(_) => continue,
                Clause::Predicate(predicate) => predicate,
                Clause::ForAll(id) => {
                    facts
                        .types
                        .quantified(id)
                        .ok_or("missing privacy clause")?
                        .predicate
                }
            };
            self.push(predicate.ty)?;
            match predicate.bound {
                Bound::Trait(id) | Bound::ConstTrait(id) | Bound::MaybeConstTrait(id) => {
                    self.trait_application(id)?;
                }
                Bound::ProjectionEqual(ty) => self.push(ty)?,
                Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_) | Bound::Outlives(_) => {
                }
            }
        }
        Ok(())
    }
}
