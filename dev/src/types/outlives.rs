//! Well-formedness requirements over typed templates, with bounded recursive propagation.
pub(crate) mod storage;
use super::{GenericArgs, ItemId, Kind, RegionId, Store, TypeId, aggregate::Definitions};
use super::{arguments::Argument, parameters::ParameterId};
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
        Kind::Never | Kind::Primitive(_) | Kind::Adt(..) | Kind::Parameter(_) => {}
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
                    if types.region_args(arguments).is_none() {
                        return Err(
                            "generic nominal outlives checking requires instantiated field bounds",
                        );
                    }
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
}
fn instantiate(types: &Store, use_: Use, pair: Pair) -> Result<Option<Pair>, &'static str> {
    let argument = |region: RegionId| -> Result<Option<RegionId>, &'static str> {
        if region == RegionId::STATIC {
            return Ok(Some(region));
        }
        let Some((0, index)) = region.bound_parts() else {
            return Err("invalid outlives template region");
        };
        let argument = types
            .arguments(use_.arguments)
            .ok_or("missing outlives instance arguments")?
            .nth(usize::from(index));
        let Some(Argument::Lifetime(region)) = argument else {
            return Err("outlives template argument arity mismatch");
        };
        free(region, use_.depth)
    };
    Ok(argument(pair.0)?.zip(argument(pair.1)?))
}
#[derive(Default)]
pub struct Bounds {
    pairs: HashMap<ItemId, Vec<Pair>>,
    pub steps: usize,
}
impl Bounds {
    /// Infer nominal requirements once from all fields, including inactive enum
    /// variants. Recursive dependencies propagate only newly discovered pairs.
    ///
    /// # Errors
    /// Rejects malformed templates and exhausted finite propagation work.
    pub fn infer(
        types: &Store,
        definitions: &Definitions,
        explicit: &[(ItemId, Pair)],
    ) -> Result<Self, &'static str> {
        let mut result = Self::default();
        let mut unique = HashSet::new();
        let mut work = Vec::new();
        let mut reverse: HashMap<ItemId, Vec<(ItemId, Use)>> = HashMap::new();
        let mut scan = Scan::default();
        for definition in definitions {
            result.pairs.insert(definition.item, Vec::new());
            scan.clear();
            for variant in definitions.variants(definition) {
                for ty in types
                    .list(variant.fields)
                    .ok_or("missing outlives field templates")?
                {
                    if types.contains_parameters(*ty) == Some(true) {
                        return Err("generic outlives checking requires a parameter environment");
                    }
                    scan.visit(types, *ty)?;
                }
            }
            for pair in &scan.pairs {
                scan.budget.charge()?;
                if unique.insert((definition.item, *pair)) {
                    scan.budget.pending(work.len().saturating_add(1))?;
                    work.push((definition.item, *pair));
                }
            }
            for use_ in &scan.uses {
                scan.budget.charge()?;
                if use_.callable {
                    return Err("nominal field template contains an unnameable function item");
                }
                reverse
                    .entry(use_.item)
                    .or_default()
                    .push((definition.item, *use_));
            }
        }
        for &(item, pair) in explicit {
            scan.budget.charge()?;
            if useful(pair) && unique.insert((item, pair)) {
                scan.budget.pending(work.len().saturating_add(1))?;
                work.push((item, pair));
            }
        }
        while let Some((item, pair)) = work.pop() {
            scan.budget.charge()?;
            result
                .pairs
                .get_mut(&item)
                .ok_or("missing outlives nominal definition")?
                .push(pair);
            if let Some(dependents) = reverse.get(&item) {
                for (owner, use_) in dependents {
                    scan.budget.charge()?;
                    if let Some(pair) = instantiate(types, *use_, pair)?
                        && useful(pair)
                        && unique.insert((*owner, pair))
                    {
                        scan.budget.pending(work.len().saturating_add(1))?;
                        work.push((*owner, pair));
                    }
                }
            }
        }
        result.steps = scan.budget.0;
        Ok(result)
    }
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&[Pair]> {
        self.pairs.get(&item).map(Vec::as_slice)
    }
}
/// Cached requirements depend only on completed typed definitions, not on a
/// caller's assumption environment. Callers decide whether to assume or require.
pub struct Queries<'a> {
    types: &'a Store,
    bounds: &'a Bounds,
    callables: &'a super::callable::Templates,
    scan: Scan,
    cache: HashMap<TypeId, Vec<Pair>>,
    parameters: HashMap<TypeId, Vec<ParameterPair>>,
}
impl<'a> Queries<'a> {
    #[must_use]
    pub fn new(
        types: &'a Store,
        bounds: &'a Bounds,
        callables: &'a super::callable::Templates,
    ) -> Self {
        Self {
            types,
            bounds,
            callables,
            scan: Scan::default(),
            cache: HashMap::new(),
            parameters: HashMap::new(),
        }
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
        self.scan.components.get(
            self.types,
            ty,
            0,
            ComponentKind::TypeBound,
            &mut self.scan.budget,
        )
    }
    /// Generic requirements retain rigid parameter identities for a scoped proof.
    /// # Errors
    /// Rejects malformed templates, unsupported nominal fields and exhausted work.
    pub fn symbolic_requirements(&mut self, ty: TypeId) -> Result<Requirements<'_>, &'static str> {
        if !self.cache.contains_key(&ty) {
            self.scan.clear();
            self.scan.visit(self.types, ty)?;
            for index in 0..self.scan.uses.len() {
                let use_ = self.scan.uses[index];
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
                    self.scan.budget.charge()?;
                    if let Some(pair) = instantiate(self.types, use_, *pair)? {
                        self.scan.record(pair);
                    }
                }
            }
            self.cache.insert(ty, self.scan.pairs.clone());
            if !self.scan.parameter_pairs.is_empty() {
                self.parameters
                    .insert(ty, self.scan.parameter_pairs.clone());
            }
        }
        let regions = self
            .cache
            .get(&ty)
            .map(Vec::as_slice)
            .ok_or("missing well-formedness requirements")?;
        Ok(Requirements {
            regions,
            parameters: self.parameters.get(&ty).map_or(&[], Vec::as_slice),
        })
    }
}
