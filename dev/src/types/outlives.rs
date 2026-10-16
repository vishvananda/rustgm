//! Well-formedness requirements over typed templates, with bounded recursive propagation.
mod contracts;
pub(crate) mod storage;
use super::{GenericArgs, ItemId, Kind, RegionId, Store, TypeId};
use super::{arguments::Argument, parameters::ParameterId};
pub use contracts::Bounds;
use std::collections::{HashMap, HashSet};

pub type Pair = (RegionId, RegionId);
pub type ParameterPair = (ParameterId, RegionId);
/// Components of a type-outlives predicate, independent of an assumption scope.
#[derive(Clone, Copy)]
pub struct Parts<'a> {
    pub regions: &'a [RegionId],
    pub parameters: &'a [ParameterId],
}
impl Parts<'_> {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.regions.is_empty() && self.parameters.is_empty()
    }
    #[must_use]
    pub const fn len(&self) -> usize {
        self.regions.len().saturating_add(self.parameters.len())
    }
}
/// Well-formedness requirements; the caller chooses whether to assume or prove.
pub struct Requirements<'a> {
    pub regions: &'a [Pair],
    pub parameters: &'a [ParameterPair],
}
const LIMIT: usize = 1_000_000;
#[derive(Default)]
struct Budget(usize);
impl Budget {
    fn charge(&mut self) -> Result<(), &'static str> {
        self.0 += 1;
        if self.0 > LIMIT {
            return Err("type outlives work budget exceeded");
        }
        Ok(())
    }
    fn pending(&self, count: usize) -> Result<(), &'static str> {
        if self.0.saturating_add(count) > LIMIT {
            return Err("type outlives work budget exceeded");
        }
        Ok(())
    }
}
fn free(region: RegionId, depth: u16) -> Result<Option<RegionId>, &'static str> {
    match region.bound_parts() {
        Some((binder, index)) if binder >= depth => {
            RegionId::bound(binder - depth, index).map(Some)
        }
        Some(_) => Ok(None),
        None => Ok(Some(region)),
    }
}
fn useful(pair: Pair) -> bool {
    pair.0 != pair.1 && pair.0 != RegionId::STATIC
}
fn children(
    types: &Store,
    ty: TypeId,
    depth: u16,
    work: &mut Vec<(TypeId, u16)>,
    budget: &mut Budget,
) -> Result<(), &'static str> {
    match types.kind(ty).ok_or("invalid well-formed type identity")? {
        Kind::Reference { referent, .. } => work.push((referent, depth)),
        Kind::Pointer { pointee, .. } => work.push((pointee, depth)),
        Kind::Slice(element) | Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } => {
            work.push((element, depth));
        }
        Kind::Tuple(list) => {
            let children = types.list(list).ok_or("missing outlives tuple children")?;
            budget.pending(work.len().saturating_add(children.len()))?;
            work.extend(children.iter().rev().map(|ty| (*ty, depth)));
        }
        Kind::Function {
            parameters, result, ..
        } => {
            let depth = depth
                .checked_add(1)
                .ok_or("outlives binder depth overflow")?;
            let children = types
                .list(parameters)
                .ok_or("missing outlives function inputs")?;
            budget.pending(work.len().saturating_add(children.len()).saturating_add(1))?;
            work.push((result, depth));
            work.extend(children.iter().rev().map(|ty| (*ty, depth)));
        }
        Kind::FunctionItem {
            signature,
            arguments,
            ..
        } => {
            work.push((signature, depth));
            if types.region_args(arguments).is_none() {
                let arguments = types
                    .arguments(arguments)
                    .ok_or("invalid outlives item arguments")?;
                budget.pending(work.len().saturating_add(arguments.len()))?;
                for argument in arguments {
                    if let Argument::Type(ty) = argument {
                        work.push((ty, depth));
                    }
                }
            }
        }
        Kind::Adt(_, arguments) => {
            let arguments = types
                .arguments(arguments)
                .ok_or("invalid nominal outlives arguments")?;
            budget.pending(work.len().saturating_add(arguments.len()))?;
            for argument in arguments {
                if let Argument::Type(ty) = argument {
                    work.push((ty, depth));
                }
            }
        }
        Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => {}
        Kind::Projection(_) => {
            return Err("associated type outlives checking requires normalization");
        }
    }
    budget.pending(work.len())
}
#[derive(Default)]
struct Components {
    cache: HashMap<(TypeId, u16, ComponentKind), Vec<RegionId>>,
    parameters: HashMap<(TypeId, u16, ComponentKind), Vec<ParameterId>>,
    work: Vec<(TypeId, u16)>,
    seen: HashSet<(TypeId, u16)>,
    regions: HashSet<RegionId>,
    parameter_seen: HashSet<ParameterId>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ComponentKind {
    /// Minimal components, under separately required reference well-formedness.
    Outlives,
    /// Explicit type bounds include every referent component, even in assumptions.
    TypeBound,
    LiveValue,
}
/// All components a type bound can constrain, including nested referents and
/// zero-length element types. Unlike owned-drop queries, Copy removes nothing.
#[derive(Default)]
pub(crate) struct TypeComponents {
    components: Components,
    budget: Budget,
}
impl TypeComponents {
    pub(crate) fn get(&mut self, types: &Store, ty: TypeId) -> Result<Parts<'_>, &'static str> {
        self.components
            .get(types, ty, 0, ComponentKind::TypeBound, &mut self.budget)
    }
}
/// Cached runtime components, including nested referents and opaque type parameters.
/// Zero-length arrays contain no runtime reference values. Type well-formedness
/// still checks their element types independently through `Queries`.
#[derive(Default)]
pub struct LiveRegions {
    components: Components,
    budget: Budget,
}
impl LiveRegions {
    /// Queries belong to one immutable canonical type store.
    ///
    /// # Errors
    /// Rejects malformed types, binder overflow and bounded traversal exhaustion.
    pub fn get(&mut self, types: &Store, ty: TypeId) -> Result<&[RegionId], &'static str> {
        if types.contains_parameters(ty) == Some(true) {
            return Err("generic live-region checking requires instantiated value facts");
        }
        self.symbolic(types, ty).map(|parts| parts.regions)
    }
    /// Retain opaque type-parameter components alongside named lifetimes. The
    /// caller must resolve both kinds in the current body's rigid environment.
    /// This structural cache depends only on the immutable canonical type store.
    /// # Errors
    /// Rejects malformed types, binder overflow and bounded traversal exhaustion.
    pub fn symbolic(&mut self, types: &Store, ty: TypeId) -> Result<Parts<'_>, &'static str> {
        self.components
            .get(types, ty, 0, ComponentKind::LiveValue, &mut self.budget)
    }
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.budget.0
    }
}
impl Components {
    fn get(
        &mut self,
        types: &Store,
        root: TypeId,
        depth: u16,
        kind: ComponentKind,
        budget: &mut Budget,
    ) -> Result<Parts<'_>, &'static str> {
        if types.contains_projections(root) == Some(true) {
            return Err("associated type outlives checking requires normalization");
        }
        if types.contains_regions(root) == Some(false)
            && types.contains_parameters(root) == Some(false)
        {
            return Ok(Parts {
                regions: &[],
                parameters: &[],
            });
        }
        let key = (root, depth, kind);
        if !self.cache.contains_key(&key) {
            let (values, parameters) = self.collect(types, root, depth, kind, budget)?;
            self.cache.insert(key, values);
            if !parameters.is_empty() {
                self.parameters.insert(key, parameters);
            }
        }
        let regions = self
            .cache
            .get(&key)
            .map(Vec::as_slice)
            .ok_or("missing outlives component cache")?;
        Ok(Parts {
            regions,
            parameters: self.parameters.get(&key).map_or(&[], Vec::as_slice),
        })
    }
    fn collect(
        &mut self,
        types: &Store,
        root: TypeId,
        depth: u16,
        kind: ComponentKind,
        budget: &mut Budget,
    ) -> Result<(Vec<RegionId>, Vec<ParameterId>), &'static str> {
        self.work.clear();
        self.seen.clear();
        self.regions.clear();
        self.parameter_seen.clear();
        self.work.push((root, depth));
        let mut values = Vec::new();
        let mut parameters = Vec::new();
        while let Some((ty, depth)) = self.work.pop() {
            budget.charge()?;
            if !self.seen.insert((ty, depth)) {
                continue;
            }
            let mut record = |region| -> Result<(), &'static str> {
                if let Some(region) = free(region, depth)?
                    && region != RegionId::STATIC
                    && self.regions.insert(region)
                {
                    values.push(region);
                }
                Ok(())
            };
            match types.kind(ty).ok_or("invalid outlives component type")? {
                Kind::Parameter(id) => {
                    if self.parameter_seen.insert(id) {
                        parameters.push(id);
                    }
                }
                // Its referent's lifetime requirements follow transitively
                // from this reference's own well-formedness requirements.
                Kind::Reference {
                    region, referent, ..
                } => {
                    record(region)?;
                    if kind != ComponentKind::Outlives {
                        self.work.push((referent, depth));
                    }
                }
                Kind::Array { length: 0, .. } if kind == ComponentKind::LiveValue => {}
                Kind::Adt(_, args)
                | Kind::FunctionItem {
                    arguments: args, ..
                } => {
                    for argument in types.arguments(args).ok_or("missing outlives arguments")? {
                        budget.charge()?;
                        match argument {
                            Argument::Lifetime(region) => record(region)?,
                            Argument::Type(ty) => {
                                budget.pending(self.work.len().saturating_add(1))?;
                                self.work.push((ty, depth));
                            }
                            Argument::Const(_) => {}
                        }
                    }
                }
                _ => children(types, ty, depth, &mut self.work, budget)?,
            }
        }
        Ok((values, parameters))
    }
}
#[derive(Clone, Copy)]
struct Use {
    item: ItemId,
    arguments: GenericArgs,
    depth: u16,
    callable: bool,
}
#[derive(Default)]
struct Scan {
    budget: Budget,
    components: Components,
    work: Vec<(TypeId, u16)>,
    seen: HashSet<(TypeId, u16)>,
    pairs: Vec<Pair>,
    unique: HashSet<Pair>,
    parameter_pairs: Vec<ParameterPair>,
    unique_parameters: HashSet<ParameterPair>,
    uses: Vec<Use>,
}
impl Scan {
    fn clear(&mut self) {
        self.work.clear();
        self.pairs.clear();
        self.unique.clear();
        self.parameter_pairs.clear();
        self.unique_parameters.clear();
        self.uses.clear();
        self.seen.clear();
    }
    fn record(&mut self, pair: Pair) {
        if useful(pair) && self.unique.insert(pair) {
            self.pairs.push(pair);
        }
    }
    fn visit(&mut self, types: &Store, root: TypeId) -> Result<(), &'static str> {
        self.work.push((root, 0));
        while let Some((ty, depth)) = self.work.pop() {
            self.budget.charge()?;
            if !self.seen.insert((ty, depth))
                || (types.contains_regions(ty) == Some(false)
                    && types.contains_parameters(ty) == Some(false))
            {
                continue;
            }
            match types.kind(ty).ok_or("invalid well-formedness type")? {
                Kind::Projection(application) => {
                    self.projected_inputs(types, application, depth)?;
                    continue;
                }
                Kind::Reference {
                    region, referent, ..
                } => {
                    if let Some(shorter) = free(region, depth)? {
                        let components = self.components.get(
                            types,
                            referent,
                            depth,
                            ComponentKind::Outlives,
                            &mut self.budget,
                        )?;
                        for longer in components.regions {
                            self.budget.charge()?;
                            let pair = (*longer, shorter);
                            if useful(pair) && self.unique.insert(pair) {
                                self.pairs.push(pair);
                            }
                        }
                        for parameter in components.parameters {
                            self.budget.charge()?;
                            let pair = (*parameter, shorter);
                            if self.unique_parameters.insert(pair) {
                                self.parameter_pairs.push(pair);
                            }
                        }
                    }
                }
                Kind::Adt(item, arguments) if arguments != GenericArgs::EMPTY => {
                    self.uses.push(Use {
                        item,
                        arguments,
                        depth,
                        callable: false,
                    });
                }
                Kind::FunctionItem {
                    item, arguments, ..
                } if arguments != GenericArgs::EMPTY => {
                    self.uses.push(Use {
                        item,
                        arguments,
                        depth,
                        callable: true,
                    });
                }
                _ => {}
            }
            children(types, ty, depth, &mut self.work, &mut self.budget)?;
        }
        Ok(())
    }

    fn projected_inputs(
        &mut self,
        types: &Store,
        application: super::projection::Application,
        depth: u16,
    ) -> Result<(), &'static str> {
        // Formation includes input well-formedness. It does not reveal the opaque
        // result's outlives or live-value components, which remain separate queries.
        self.work.push((application.subject, depth));
        let trait_ = types
            .trait_application(application.trait_application)
            .ok_or("invalid well-formed projection trait")?;
        for arguments in [trait_.arguments, application.arguments] {
            let arguments = types
                .arguments(arguments)
                .ok_or("invalid projection arguments")?;
            self.budget
                .pending(self.work.len().saturating_add(arguments.len()))?;
            for argument in arguments {
                self.budget.charge()?;
                if let Argument::Type(ty) = argument {
                    self.work.push((ty, depth));
                }
            }
        }
        self.budget.pending(self.work.len())
    }
}
/// Cached requirements depend only on completed typed definitions, not on a
/// caller's assumption environment. Callers decide whether to assume or require.
pub struct Queries<'a> {
    types: &'a Store,
    bounds: &'a Bounds,
    callables: &'a super::callable::Templates,
    state: QueryCache,
}
/// Successful requirements in one append-only semantic environment. Missing
/// declarations never publish a cache entry; completed declarations cannot change.
#[derive(Default)]
pub struct QueryCache {
    scan: Scan,
    regions: HashMap<TypeId, Vec<Pair>>,
    parameters: HashMap<TypeId, Vec<ParameterPair>>,
}
impl QueryCache {
    #[must_use]
    pub fn attach<'a>(
        self,
        types: &'a Store,
        bounds: &'a Bounds,
        callables: &'a super::callable::Templates,
    ) -> Queries<'a> {
        Queries {
            types,
            bounds,
            callables,
            state: self,
        }
    }
}
impl<'a> Queries<'a> {
    #[must_use]
    pub fn new(
        types: &'a Store,
        bounds: &'a Bounds,
        callables: &'a super::callable::Templates,
    ) -> Self {
        QueryCache::default().attach(types, bounds, callables)
    }
    /// Release semantic borrows without discarding completed queries or budgets.
    #[must_use]
    pub fn into_cache(self) -> QueryCache {
        self.state
    }
    /// # Errors
    /// Rejects inconsistent nominal facts and exhausted type traversal work.
    pub fn requirements(&mut self, ty: TypeId) -> Result<&[Pair], &'static str> {
        if self.types.contains_parameters(ty) == Some(true) {
            return Err("generic outlives checking requires a parameter environment");
        }
        self.symbolic_requirements(ty)
            .map(|requirements| requirements.regions)
    }
    /// Decompose a type-outlives subject without assuming any of its components.
    /// # Errors
    /// Rejects invalid identities, escaping binder overflow and exhausted work.
    pub fn components(&mut self, ty: TypeId) -> Result<Parts<'_>, &'static str> {
        self.state.scan.components.get(
            self.types,
            ty,
            0,
            ComponentKind::TypeBound,
            &mut self.state.scan.budget,
        )
    }
    /// Generic requirements retain rigid parameter identities for a scoped proof.
    /// # Errors
    /// Rejects malformed templates, unsupported nominal fields and exhausted work.
    pub fn symbolic_requirements(&mut self, ty: TypeId) -> Result<Requirements<'_>, &'static str> {
        if !self.state.regions.contains_key(&ty) {
            self.state.scan.clear();
            self.state.scan.visit(self.types, ty)?;
            for index in 0..self.state.scan.uses.len() {
                let use_ = self.state.scan.uses[index];
                let pairs = if use_.callable {
                    let template = self
                        .callables
                        .get(use_.item)
                        .ok_or("missing callable lifetime template")?;
                    if self.types.arguments(use_.arguments).is_none_or(|args| {
                        args.filter(|arg| matches!(arg, Argument::Lifetime(_)))
                            .count()
                            != template.parameters as usize
                    }) {
                        return Err("callable lifetime argument arity mismatch");
                    }
                    self.callables
                        .bounds(use_.item)
                        .ok_or("missing callable lifetime bounds")?
                } else {
                    self.bounds
                        .get(use_.item)
                        .ok_or("missing nominal outlives facts")?
                };
                for pair in pairs {
                    self.state.scan.budget.charge()?;
                    if let Some(pair) = contracts::instantiate(self.types, use_, *pair)? {
                        self.state.scan.record(pair);
                    }
                }
                if !use_.callable {
                    for &pair in self
                        .bounds
                        .parameters(use_.item)
                        .ok_or("missing nominal parameter requirements")?
                    {
                        self.state.scan.apply_parameter(self.types, use_, pair)?;
                    }
                }
            }
            self.state.regions.insert(ty, self.state.scan.pairs.clone());
            if !self.state.scan.parameter_pairs.is_empty() {
                self.state
                    .parameters
                    .insert(ty, self.state.scan.parameter_pairs.clone());
            }
        }
        let regions = self
            .state
            .regions
            .get(&ty)
            .map(Vec::as_slice)
            .ok_or("missing well-formedness requirements")?;
        Ok(Requirements {
            regions,
            parameters: self.state.parameters.get(&ty).map_or(&[], Vec::as_slice),
        })
    }
}
