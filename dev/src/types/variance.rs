//! Variance of lifetime/type parameters from completed nominal field templates.
mod reachable;
mod rigidity;
mod solver;
use super::{
    GenericArgs, ItemId, Kind, ListId, RegionId, Store, TypeId,
    aggregate::Definitions,
    arguments::{Argument, ConstId, ConstKind},
    parameters::{ParameterId, ParameterKind},
};
use crate::support::hash::{HashMap, HashSet};
pub(crate) use rigidity::RigidInputs;
use solver::{Solver, Term};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variance {
    Bivariant,
    Covariant,
    Contravariant,
    Invariant,
}
impl Variance {
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        if self == other || other == Self::Bivariant {
            self
        } else if self == Self::Bivariant {
            other
        } else {
            Self::Invariant
        }
    }
    #[must_use]
    pub fn compose(self, inner: Self) -> Self {
        match (self, inner) {
            (Self::Bivariant, _) | (_, Self::Bivariant) => Self::Bivariant,
            (Self::Covariant, value) | (value, Self::Covariant) => value,
            (Self::Contravariant, Self::Contravariant) => Self::Covariant,
            _ => Self::Invariant,
        }
    }
}
#[derive(Default)]
pub struct Variances {
    ranges: HashMap<ItemId, (usize, usize)>,
    values: Vec<Variance>,
    pub steps: usize,
}
impl Variances {
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&[Variance]> {
        let &(start, end) = self.ranges.get(&item)?;
        self.values.get(start..end)
    }

    /// Infer all declaration slots from completed field templates. Arguments are
    /// the declaration's own rigid parameters, in full declaration order.
    /// Bounds do not create occurrences: recursive-only type/lifetime parameters
    /// remain bivariant. Const slots are invariant, including unused ones.
    ///
    /// # Errors
    /// Rejects inconsistent templates/parameter identities and exhausted graph work.
    pub fn infer(
        types: &Store,
        definitions: &Definitions,
        parameters: &[(ItemId, GenericArgs)],
    ) -> Result<Self, &'static str> {
        Self::infer_with_fields(types, definitions, parameters, |_, _, fields| fields)
    }

    /// Select retained declaration field types independently of normalized storage.
    /// Projection inputs remain invariant even when an equality fixes their result.
    /// The reader preserves each variant's original field count and ordering.
    /// # Errors
    /// Rejects malformed field lists, changed arity and exhausted inference work.
    pub fn infer_with_fields(
        types: &Store,
        definitions: &Definitions,
        parameters: &[(ItemId, GenericArgs)],
        fields: impl FnMut(ItemId, u32, ListId) -> ListId,
    ) -> Result<Self, &'static str> {
        Self::infer_mode(types, definitions, parameters, fields, false)
    }
    /// Append an independent completed declaration using the same field solver.
    /// # Errors
    /// Rejects an existing entry, external nominals and ordinary inference errors.
    pub(crate) fn append_independent(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        item: ItemId,
        arguments: GenericArgs,
    ) -> Result<(), &'static str> {
        if self.ranges.contains_key(&item) {
            return Err("nominal variance is already complete");
        }
        let inferred = Self::infer_mode(
            types,
            definitions,
            &[(item, arguments)],
            |_, _, fields| fields,
            true,
        )?;
        let values = inferred
            .get(item)
            .ok_or("missing independent variance result")?;
        let steps = self
            .steps
            .checked_add(inferred.steps)
            .filter(|n| *n <= 1_000_000)
            .ok_or("variance work budget exceeded")?;
        let start = self.values.len();
        self.values.extend_from_slice(values);
        self.ranges.insert(item, (start, self.values.len()));
        self.steps = steps;
        Ok(())
    }
    fn infer_mode(
        types: &Store,
        definitions: &Definitions,
        parameters: &[(ItemId, GenericArgs)],
        mut fields: impl FnMut(ItemId, u32, ListId) -> ListId,
        independent: bool,
    ) -> Result<Self, &'static str> {
        let mut graph = Graph {
            independent,
            ..Graph::default()
        };
        for &(item, arguments) in parameters {
            graph.declaration(types, item, arguments)?;
        }
        for &(item, arguments) in parameters {
            let definition = definitions
                .get(item)
                .ok_or("missing variance aggregate definition")?;
            graph.seen.clear();
            if definition.kind == super::aggregate::Kind::PhantomData {
                let target = definitions.phantom_argument_from(types, item, arguments)?;
                graph.visit(types, item, target)?;
            }
            for (ordinal, variant) in (0..).zip(definitions.variants(definition)) {
                graph.solver.charge()?;
                let selected = fields(item, ordinal, variant.fields);
                let declared = types.list(selected).ok_or("missing variance field types")?;
                if selected != variant.fields
                    && declared.len()
                        != types
                            .list(variant.fields)
                            .ok_or("missing storage fields")?
                            .len()
                {
                    return Err("variance declaration field count differs from storage");
                }
                for ty in declared {
                    graph.visit(types, item, *ty)?;
                }
            }
        }
        graph.solver.solve()?;
        graph.result.values = graph
            .variables
            .iter()
            .map(|&id| graph.solver.value(id))
            .collect();
        graph.result.steps = graph.solver.steps;
        Ok(graph.result)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    Lifetime,
    Type,
    Const(TypeId),
}
#[derive(Default)]
struct Graph<'a> {
    skip_external_nominals: bool,
    independent: bool,
    known: Option<&'a Variances>,
    declarations: Option<&'a [Option<TypeId>]>,
    pending: Vec<ItemId>,
    result: Variances,
    slots: Vec<Slot>,
    variables: Vec<Term>,
    solver: Solver,
    work: Vec<(TypeId, Term, u16)>,
    seen: HashSet<(TypeId, Term, u16)>,
}
impl Graph<'_> {
    fn declaration(
        &mut self,
        types: &Store,
        item: ItemId,
        args: GenericArgs,
    ) -> Result<(), &'static str> {
        self.solver.charge()?;
        let start = self.slots.len();
        let arguments = types
            .arguments(args)
            .ok_or("invalid variance declaration arguments")?;
        let mut non_lifetime = false;
        for (index, argument) in arguments.enumerate() {
            self.solver.charge()?;
            let (slot, parameter) = match argument {
                Argument::Lifetime(region) if !non_lifetime => {
                    if region.bound_parts().map(|(d, i)| (d, usize::from(i))) != Some((0, index)) {
                        return Err("invalid variance lifetime declaration");
                    }
                    (Slot::Lifetime, None)
                }
                Argument::Type(ty) => {
                    let Some(Kind::Parameter(id)) = types.kind(ty) else {
                        return Err("variance declaration requires a rigid type parameter");
                    };
                    (Slot::Type, Some(id))
                }
                Argument::Const(id) => {
                    let Some(ConstKind::Parameter(parameter)) = types.const_kind(id) else {
                        return Err("variance declaration requires a rigid const parameter");
                    };
                    (
                        Slot::Const(types.const_type(id).ok_or("invalid variance const type")?),
                        Some(parameter),
                    )
                }
                Argument::Lifetime(_) => {
                    return Err("variance lifetimes must precede type and const slots");
                }
            };
            if let Some(id) = parameter {
                non_lifetime = true;
                let parameter = types.parameter(id).ok_or("missing variance parameter")?;
                if parameter.owner != item || parameter.index as usize != index {
                    return Err("variance parameter outside its declaration");
                }
            }
            self.slots.push(slot);
            self.variables
                .push(self.solver.variable(if matches!(slot, Slot::Const(_)) {
                    Variance::Invariant
                } else {
                    Variance::Bivariant
                })?);
        }
        if self
            .result
            .ranges
            .insert(item, (start, self.slots.len()))
            .is_some()
        {
            return Err("duplicate variance parameter declaration");
        }
        Ok(())
    }
    fn region(
        &self,
        owner: ItemId,
        region: RegionId,
        depth: u16,
    ) -> Result<Option<Term>, &'static str> {
        let Some((binder, index)) = region.bound_parts() else {
            return if region == RegionId::STATIC {
                Ok(None)
            } else {
                Err("inference lifetime in variance template")
            };
        };
        if binder < depth {
            return Ok(None);
        }
        let (start, end) = self.result.ranges[&owner];
        let slot = start + usize::from(index);
        if binder != depth || slot >= end || self.slots[slot] != Slot::Lifetime {
            return Err("variance lifetime outside its declaration");
        }
        Ok(Some(self.variables[slot]))
    }
    fn parameter(
        &self,
        types: &Store,
        owner: ItemId,
        id: ParameterId,
    ) -> Result<Term, &'static str> {
        let parameter = types.parameter(id).ok_or("missing variance parameter")?;
        let (start, end) = self.result.ranges[&owner];
        let slot = start + parameter.index as usize;
        let expected = match parameter.kind {
            ParameterKind::Type { .. } => Slot::Type,
            ParameterKind::Const { ty } => Slot::Const(ty),
        };
        if parameter.owner != owner || slot >= end || self.slots[slot] != expected {
            return Err("variance parameter outside its declaration");
        }
        Ok(self.variables[slot])
    }
    fn constant(&self, types: &Store, owner: ItemId, id: ConstId) -> Result<(), &'static str> {
        if let ConstKind::Parameter(id) = types.const_kind(id).ok_or("invalid variance constant")? {
            self.parameter(types, owner, id)?;
        }
        Ok(())
    }
    fn queue_type(&mut self, ty: TypeId, position: Term, depth: u16) -> Result<(), &'static str> {
        self.solver.charge()?;
        self.work.push((ty, position, depth));
        Ok(())
    }
    fn referent(&mut self, position: Term, mutable: bool) -> Result<Term, &'static str> {
        if mutable {
            self.solver
                .compose(position, Term::constant(Variance::Invariant))
        } else {
            Ok(position)
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive variance traversal keeps binder depth and input positions together"
    )]
    fn visit(&mut self, types: &Store, owner: ItemId, ty: TypeId) -> Result<(), &'static str> {
        self.queue_type(ty, Term::constant(Variance::Covariant), 0)?;
        while let Some((ty, position, depth)) = self.work.pop() {
            self.solver.charge()?;
            if self.independent
                && (matches!(types.kind(ty),Some(Kind::Adt(item,_)) if item!=owner)
                    || matches!(types.kind(ty), Some(Kind::Projection(_))))
            {
                return Err("independent variance append has an external nominal dependency");
            }
            if (!self.independent
                && types.escaping_binders(ty).ok_or("invalid variance type")? <= depth
                && !types
                    .contains_parameters(ty)
                    .ok_or("invalid variance type")?)
                || !self.seen.insert((ty, position, depth))
            {
                continue;
            }
            match types.kind(ty).ok_or("invalid variance type")? {
                Kind::Parameter(id) => {
                    let target = self.parameter(types, owner, id)?;
                    self.solver.constrain(target, position)?;
                }
                Kind::Reference {
                    mutable,
                    region,
                    referent,
                } => {
                    if let Some(target) = self.region(owner, region, depth)? {
                        self.solver.constrain(target, position)?;
                    }
                    let position = self.referent(position, mutable)?;
                    self.queue_type(referent, position, depth)?;
                }
                Kind::Pattern { base, .. } => self.queue_type(base, position, depth)?,
                Kind::Pointer { mutable, pointee } => {
                    let position = self.referent(position, mutable)?;
                    self.queue_type(pointee, position, depth)?;
                }
                Kind::ArrayTerm { element, length } => {
                    self.constant(types, owner, length)?;
                    self.queue_type(element, position, depth)?;
                }
                Kind::Array { element, .. } | Kind::Slice(element) => {
                    self.queue_type(element, position, depth)?;
                }
                Kind::Tuple(fields) => {
                    for field in types.list(fields).ok_or("invalid variance tuple")? {
                        self.queue_type(*field, position, depth)?;
                    }
                }
                Kind::Function {
                    parameters, result, ..
                } => {
                    let depth = depth
                        .checked_add(1)
                        .ok_or("variance binder depth overflow")?;
                    let input = self
                        .solver
                        .compose(position, Term::constant(Variance::Contravariant))?;
                    for parameter in types.list(parameters).ok_or("invalid variance signature")? {
                        self.queue_type(*parameter, input, depth)?;
                    }
                    self.queue_type(result, position, depth)?;
                }
                Kind::Closure {
                    arguments,
                    signature,
                    environment,
                    ..
                } => {
                    let invariant = self
                        .solver
                        .compose(position, Term::constant(Variance::Invariant))?;
                    self.invariant_arguments(types, owner, arguments, invariant, depth)?;
                    self.queue_type(signature, invariant, depth)?;
                    self.queue_type(environment, invariant, depth)?;
                }
                Kind::FunctionItem { signature, .. } => {
                    self.queue_type(signature, position, depth)?;
                }
                Kind::Adt(item, args) => {
                    self.nominal(types, owner, (item, args), position, depth)?;
                }
                Kind::Opaque(_, args) => {
                    let invariant = self
                        .solver
                        .compose(position, Term::constant(Variance::Invariant))?;
                    self.invariant_arguments(types, owner, args, invariant, depth)?;
                }
                Kind::Dynamic {
                    arguments, region, ..
                } => {
                    if let Some(target) = self.region(owner, region, depth)? {
                        self.solver.constrain(target, position)?;
                    }
                    let nested = depth
                        .checked_add(1)
                        .ok_or("variance binder depth overflow")?;
                    let invariant = self
                        .solver
                        .compose(position, Term::constant(Variance::Invariant))?;
                    self.invariant_arguments(types, owner, arguments, invariant, nested)?;
                }
                Kind::Projection(application) => {
                    let invariant = self
                        .solver
                        .compose(position, Term::constant(Variance::Invariant))?;
                    self.queue_type(application.subject, invariant, depth)?;
                    let trait_ = types
                        .trait_application(application.trait_application)
                        .ok_or("invalid variance projection trait")?;
                    for args in [trait_.arguments, application.arguments] {
                        self.invariant_arguments(types, owner, args, invariant, depth)?;
                    }
                }
                Kind::Never | Kind::Foreign(_) | Kind::Primitive(_) => {}
            }
        }
        Ok(())
    }
    fn invariant_arguments(
        &mut self,
        types: &Store,
        owner: ItemId,
        args: GenericArgs,
        position: Term,
        depth: u16,
    ) -> Result<(), &'static str> {
        for argument in types
            .arguments(args)
            .ok_or("invalid projection variance arguments")?
        {
            self.solver.charge()?;
            match argument {
                Argument::Type(ty) => self.queue_type(ty, position, depth)?,
                Argument::Lifetime(region) => {
                    if let Some(target) = self.region(owner, region, depth)? {
                        self.solver.constrain(target, position)?;
                    }
                }
                Argument::Const(id) => self.constant(types, owner, id)?,
            }
        }
        Ok(())
    }
    fn nominal(
        &mut self,
        types: &Store,
        owner: ItemId,
        nominal: (ItemId, GenericArgs),
        position: Term,
        depth: u16,
    ) -> Result<(), &'static str> {
        if !self.result.ranges.contains_key(&nominal.0) {
            if let Some(values) = self.known.and_then(|known| known.get(nominal.0)) {
                return self.completed_nominal(types, owner, nominal, position, depth, values);
            }
            if self.declarations.is_some() {
                self.reserve_reached(types, nominal.0)?;
            }
        }
        let arguments = types
            .arguments(nominal.1)
            .ok_or("invalid variance arguments")?;
        let Some(&(start, end)) = self.result.ranges.get(&nominal.0) else {
            if self.skip_external_nominals {
                // An unfinished external declaration grants no invariant-slot hint.
                return Ok(());
            }
            return Err("missing nominal variance parameters");
        };
        if arguments.len() != end - start {
            return Err("incorrect nominal variance arity");
        }
        for (source, argument) in (start..end).zip(arguments) {
            self.solver.charge()?;
            match (self.slots[source], argument) {
                (Slot::Lifetime, Argument::Lifetime(region)) => {
                    if let Some(target) = self.region(owner, region, depth)? {
                        let value = self.solver.compose(position, self.variables[source])?;
                        self.solver.constrain(target, value)?;
                    }
                }
                (Slot::Type, Argument::Type(ty)) => {
                    let value = self.solver.compose(position, self.variables[source])?;
                    self.queue_type(ty, value, depth)?;
                }
                (Slot::Const(expected), Argument::Const(id))
                    if types.const_type(id) == Some(expected) =>
                {
                    self.constant(types, owner, id)?;
                }
                _ => return Err("incorrect nominal variance argument kind"),
            }
        }
        Ok(())
    }
}
