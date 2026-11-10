//! Structural well-formedness and sizedness under one immutable bound environment.
mod size_demands;
use super::environment::{BuiltinQuery, Environment};
use crate::support::hash::HashMap;
use crate::types::{
    Kind, Primitive, Store, TypeId, bound_uses::captures::Captures, lifetime_proof::ScopeId,
    substitute::lookup::Lookup,
};
pub(crate) use size_demands::{Progress, Request};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Properties {
    pub sized: bool,
    pub well_formed: bool,
}
/// The other header's Sized and `MetaSized` premises in a negative coherence
/// query, each sorted by identity, behind one shared handle. rustc 1.98
/// negates one impl's predicates under the other impl's `param_env` with each
/// unified header variable a placeholder, so a rigid parameter is Sized or
/// `MetaSized` only as a member, never by the declaration of the parameter
/// `freeze` made the representative.
#[derive(Clone)]
pub struct Placeholders {
    sets: Arc<Sets>,
}
struct Sets {
    sized: Arc<[TypeId]>,
    meta_sized: Arc<[TypeId]>,
}
impl Placeholders {
    /// Orders and deduplicates both premise sets by identity, so no lookup
    /// depends on its caller's order; an already ordered set is shared as it
    /// is. The work, for the caller's budget, is one comparison per premise of
    /// each set, at least one per set, plus `n (ilog2(n) + 1)` for each set
    /// that must be sorted.
    ///
    /// # Errors
    /// Rejects a failed allocation of an ordered copy.
    pub fn new(
        sized: Arc<[TypeId]>,
        meta_sized: Arc<[TypeId]>,
    ) -> Result<(Self, usize), &'static str> {
        let (sized, sized_work) = ordered(sized)?;
        let (meta_sized, meta_sized_work) = ordered(meta_sized)?;
        Ok((
            Self {
                sets: Arc::new(Sets { sized, meta_sized }),
            },
            sized_work.saturating_add(meta_sized_work),
        ))
    }
    /// Whether `ty` is a Sized premise, and the lookup's work.
    #[must_use]
    pub fn sized(&self, ty: TypeId) -> (bool, usize) {
        lookup(&self.sets.sized, ty)
    }
    /// Whether `ty` is a `MetaSized` premise, and the lookup's work.
    #[must_use]
    pub fn meta_sized(&self, ty: TypeId) -> (bool, usize) {
        lookup(&self.sets.meta_sized, ty)
    }
    /// Whether these premises are the same immutable caller handle.
    #[must_use]
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.sets, &other.sets)
    }
    /// Whether neither premise set has a member.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.sets.sized.is_empty() && self.sets.meta_sized.is_empty()
    }
}
/// One premise set, ordered by identity and deduplicated, and its work.
fn ordered(set: Arc<[TypeId]>) -> Result<(Arc<[TypeId]>, usize), &'static str> {
    let count = set.len();
    let check = count.max(1);
    if set.windows(2).all(|pair| pair[0].0 < pair[1].0) {
        return Ok((set, check));
    }
    let mut ordered = Vec::new();
    ordered
        .try_reserve_exact(count)
        .map_err(|_| "placeholder premises allocation failed")?;
    ordered.extend_from_slice(&set);
    ordered.sort_unstable_by_key(|ty| ty.0);
    ordered.dedup();
    let sorting = count.saturating_mul(count.max(1).ilog2() as usize + 1);
    Ok((Arc::from(ordered), check.saturating_add(sorting)))
}
/// A binary search of one ordered premise set, and its work.
fn lookup(set: &[TypeId], ty: TypeId) -> (bool, usize) {
    let work = set.len().max(1).ilog2() as usize + 1;
    (set.binary_search_by_key(&ty.0, |ty| ty.0).is_ok(), work)
}
/// Completed unshifted physical properties reachable from one pending alias.
/// Input proof authority stays with the caller; this snapshot contains no duties.
#[derive(Default)]
pub(crate) struct Completed {
    pub(crate) properties: Vec<(TypeId, Properties)>,
    pub(crate) steps: usize,
}
impl Shapes {
    /// Copy only this alias's completed, projection-free physical inputs. A
    /// callable's descendants belong to another scope and are not exported.
    pub(crate) fn snapshot_properties(
        &mut self,
        types: &Store,
        root: TypeId,
        normalized: bool,
    ) -> Result<Completed, &'static str> {
        let mut result = Completed::default();
        if types.contains_projected_sizes(root) != Some(true) {
            return Ok(result);
        }
        let query = self.query(normalized);
        if query.cache.is_empty() {
            return Ok(result);
        }
        let mut pending = vec![root];
        let mut seen = crate::support::hash::HashSet::default();
        while let Some(ty) = pending.pop() {
            result.steps = result.steps.saturating_add(1);
            if result.steps.saturating_add(pending.len()) > 1_000_000 {
                return Err("physical shape snapshot work budget exceeded");
            }
            if !seen.insert(ty) {
                continue;
            }
            let kind = types
                .kind(ty)
                .ok_or("invalid physical shape snapshot input")?;
            if types.contains_projected_sizes(ty) == Some(true)
                && types.contains_projections(ty) == Some(false)
                && let Some(&properties) = query.cached(ty, 0)
            {
                result.properties.push((ty, properties));
            }
            children(types, kind, 0, |child, depth| {
                result.steps = result.steps.saturating_add(1);
                if result.steps.saturating_add(pending.len()) >= 1_000_000 {
                    return Err("physical shape snapshot work budget exceeded");
                }
                if depth == 0 {
                    pending.push(child);
                }
                Ok(())
            })?;
        }
        Ok(result)
    }
}

pub struct Shapes {
    environment: Arc<Environment>,
    builtins: Option<Box<BuiltinQuery>>,
    cache: HashMap<TypeId, Properties>,
    normalized_queries: Option<Box<Self>>,
    scoped: Option<Box<Scoped>>,
    work: Vec<(TypeId, u16, ScopeId, bool)>,
    steps: usize,
    defer_local_sized: bool,
    nominal: size_demands::Sizes,
    placeholders: Option<Placeholders>,
}
#[derive(Default)]
struct Scoped {
    cache: HashMap<(TypeId, u16), Properties>,
    lookups: HashMap<u16, Lookup>,
    captures: HashMap<u16, Captures>,
}
impl Shapes {
    pub(crate) fn steps(&self) -> usize {
        self.steps
            + self
                .normalized_queries
                .as_ref()
                .map_or(0, |queries| queries.steps())
    }
    #[must_use]
    pub fn new(environment: Arc<Environment>) -> Self {
        Self {
            environment,
            builtins: None,
            cache: HashMap::default(),
            normalized_queries: None,
            scoped: None,
            work: Vec::new(),
            steps: 0,
            defer_local_sized: false,
            nominal: size_demands::Sizes::default(),
            placeholders: None,
        }
    }
    /// A negative coherence query's walk under the other header's Sized
    /// premises. It follows rustc 1.98's `sized_conditions`, decides only
    /// sizedness and reads no interned Sized or well-formedness property of a
    /// type with parameters; see [`Self::placeholder_composite`].
    #[must_use]
    pub fn with_placeholders(environment: Arc<Environment>, placeholders: Placeholders) -> Self {
        let mut shapes = Self::new(environment);
        shapes.placeholders = Some(placeholders);
        shapes
    }
    pub(crate) fn placeholders(&self) -> Option<&Placeholders> {
        self.placeholders.as_ref()
    }
    /// Declaration formation may defer a Sized requirement mentioning a local
    /// binder. Its result still reports actual sizedness; it grants no use proof.
    /// A negative coherence proof's clause terms keep its placeholders.
    pub(crate) fn for_clause_terms(
        environment: Arc<Environment>,
        placeholders: Option<Placeholders>,
    ) -> Self {
        let mut shapes = Self::new(environment);
        shapes.defer_local_sized = true;
        shapes.placeholders = placeholders;
        shapes
    }
    /// The one sub-walker for separately normalized roots. It inherits this
    /// walk's environment, local Sized deferral and placeholders, so no
    /// sub-walker can drop an assumption of its parent.
    fn normalized_query(&mut self) -> &mut Self {
        self.normalized_queries.get_or_insert_with(|| {
            let mut query = Self::new(Arc::clone(&self.environment));
            query.defer_local_sized = self.defer_local_sized;
            query.placeholders.clone_from(&self.placeholders);
            Box::new(query)
        })
    }
    /// # Errors
    /// Rejects invalid/foreign parameter types and exhausted graph work.
    pub fn demand(&mut self, types: &Store, ty: TypeId) -> Result<Properties, &'static str> {
        let result = self.try_demand(types, ty)?;
        if result.is_none() {
            self.cancel();
        }
        result.ok_or("associated type shape checking requires normalization")
    }
    /// A complete structural proof under the fixed assumptions avoids repeating
    /// normalization. None requests parent evidence without caching permission.
    pub(crate) fn try_demand(
        &mut self,
        types: &Store,
        ty: TypeId,
    ) -> Result<Option<Properties>, &'static str> {
        self.try_demand_inside(types, ty, 0)
    }
    /// The root is written beneath an explicit prefix of surrounding binders.
    pub(crate) fn try_demand_inside(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
    ) -> Result<Option<Properties>, &'static str> {
        self.demand_with(types, ty, depth, false)
    }
    /// The caller has normalized this entire root in this query's environment
    /// and retained every parent formation proof, including nested binders.
    pub(crate) fn normalized(
        &mut self,
        types: &Store,
        ty: TypeId,
    ) -> Result<Properties, &'static str> {
        self.normalized_inside(types, ty, 0)
    }
    pub(crate) fn normalized_inside(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
    ) -> Result<Properties, &'static str> {
        // Raw structural queries cannot reuse a cache populated under a
        // separately retained normalization proof.
        let query = self.normalized_query();
        let result = query.demand_with(types, ty, depth, true)?;
        if result.is_none() {
            query.cancel();
        }
        result.ok_or("missing normalized shape proof")
    }
    fn demand_with(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
        normalized: bool,
    ) -> Result<Option<Properties>, &'static str> {
        if let Some(value) = self.cached(ty, cache_depth(types, ty, depth)) {
            return Ok(Some(*value));
        }
        if !self.nominal.resume(ty, depth)? {
            self.work.clear();
            self.work.push((ty, depth, ScopeId::ROOT, false));
        }
        // A rejected root must not leave a resumable frontier behind. Completed
        // siblings still belong to this fixed environment and remain reusable.
        // Resume validation above leaves an existing, valid suspension untouched.
        let result = self.walk_with(types, ty, depth, normalized);
        if result.is_err() {
            self.cancel();
        }
        result
    }
    fn cancel(&mut self) {
        self.work.clear();
        self.nominal.cancel();
    }
    fn walk_with(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
        normalized: bool,
    ) -> Result<Option<Properties>, &'static str> {
        while let Some((ty, depth, scope, finish)) = self.work.pop() {
            self.steps += 1;
            if self.steps.saturating_add(self.work.len()) > 1_000_000 {
                return Err("generic shape work budget exceeded");
            }
            let key_depth = cache_depth(types, ty, depth);
            if self.cached(ty, key_depth).is_some() {
                continue;
            }
            let kind = types.kind(ty).ok_or("invalid generic shape type")?;
            if let Kind::Projection(application) = kind
                && !normalized
                && !self.assumes_trait(types, application, depth)?
            {
                self.nominal.root = None;
                return Ok(None);
            }
            if types.contains_parameters(ty) == Some(false)
                && types.contains_projections(ty) == Some(false)
                && types.contains_projected_sizes(ty) != Some(true)
                && (!self.defer_local_sized || types.is_well_formed(ty) == Some(true))
            {
                let sized =
                    types.is_sized(ty) == Some(true) || self.assumes_sized(types, ty, depth)?;
                self.save(
                    ty,
                    key_depth,
                    Properties {
                        sized,
                        well_formed: types.is_well_formed(ty) == Some(true),
                    },
                );
                continue;
            }
            if !finish {
                self.work.push((ty, depth, scope, true));
                if self.placeholders.is_some() {
                    sized_children(types, kind, |child| {
                        if self.steps.saturating_add(self.work.len()) >= 1_000_000 {
                            return Err("generic shape work budget exceeded");
                        }
                        self.work.push((child, depth, scope, false));
                        Ok(())
                    })?;
                    continue;
                }
                let child_scope = if types.contains_projected_sizes(ty) == Some(true) {
                    self.nominal.children(kind, scope)?
                } else {
                    scope
                };
                children(types, kind, depth, |child, depth| {
                    if self.steps.saturating_add(self.work.len()) >= 1_000_000 {
                        return Err("generic shape work budget exceeded");
                    }
                    self.work.push((child, depth, child_scope, false));
                    Ok(())
                })?;
                continue;
            }
            if let Kind::Adt(item, _) = kind
                && matches!(
                    types.nominal_size_rule(item),
                    Some(crate::types::sizedness::Rule::Projection(_))
                )
                && !self.nominal.answers.contains_key(&(ty, key_depth))
            {
                if self.premise_sized(types, ty, depth)? {
                    self.nominal.answers.insert((ty, key_depth), true);
                } else {
                    self.work.push((ty, depth, scope, true));
                    self.nominal.pending = Some(Request { ty, depth, scope });
                    return Ok(None);
                }
            }
            let value = if self.placeholders.is_some() {
                self.placeholder_composite(types, ty, kind, depth)?
            } else {
                self.composite(types, ty, kind, depth)?
            };
            self.save(ty, key_depth, value);
        }
        self.nominal.root = None;
        self.cached(ty, cache_depth(types, ty, depth))
            .copied()
            .map(Some)
            .ok_or("missing generic shape result")
    }
    fn composite(
        &mut self,
        types: &Store,
        ty: TypeId,
        kind: Kind,
        depth: u16,
    ) -> Result<Properties, &'static str> {
        let mut value = Properties {
            sized: true,
            well_formed: true,
        };
        let mut index = 0;
        let tuple_len = if let Kind::Tuple(list) = kind {
            types.list(list).ok_or("invalid shape tuple")?.len()
        } else {
            0
        };
        children(types, kind, depth, |child, depth| {
            let properties = *self
                .cached(child, cache_depth(types, child, depth))
                .ok_or("missing child shape")?;
            value.well_formed &= properties.well_formed;
            if matches!(
                kind,
                Kind::Tuple(_) | Kind::Array { .. } | Kind::ArrayTerm { .. } | Kind::Slice(_)
            ) {
                value.sized &= properties.sized;
            }
            if requires_sized_edge(kind, index, tuple_len, child) {
                value.well_formed &= properties.sized
                    || (!matches!(kind, Kind::Closure { .. })
                        && self.defers_sized(types, child, depth)?);
            }
            index += 1;
            Ok(())
        })?;
        match kind {
            Kind::Adt(item, arguments) => {
                use crate::types::sizedness::Rule;
                value.sized = match types.nominal_size_rule(item) {
                    None | Some(Rule::Sized) => true,
                    Some(Rule::Unsized) => false,
                    Some(Rule::Projection(_)) => *self
                        .nominal
                        .answers
                        .get(&(ty, cache_depth(types, ty, depth)))
                        .ok_or("missing projected nominal size proof")?,
                    Some(Rule::Argument(slot)) => {
                        let ty = sizedness_argument(types, arguments, slot)?;
                        self.cached(ty, cache_depth(types, ty, depth))
                            .ok_or("missing nominal sizedness argument proof")?
                            .sized
                    }
                };
            }
            Kind::Parameter(id) => {
                let abilities = self
                    .environment
                    .abilities(id)
                    .ok_or("shape parameter is outside its environment")?;
                value.sized = types.is_sized(ty) == Some(true) || abilities.clone_;
            }
            Kind::Foreign(_)
            | Kind::Slice(_)
            | Kind::Primitive(Primitive::Str)
            | Kind::Dynamic { .. } => {
                value.sized = false;
            }
            Kind::Projection(application) => {
                value.sized = types.is_sized(ty) == Some(true)
                    || self.environment.projection_promises(application.item).sized;
            }
            _ => {}
        }
        value.sized = value.sized || self.assumes_sized(types, ty, depth)?;
        Ok(value)
    }
    /// rustc 1.98's `sized_conditions` under the other header's premises: a
    /// tuple is Sized as its last element, a pattern as its base, a nominal by
    /// its sizedness rule and a projection or opaque by its declared bounds;
    /// the builtin Sized types always; a slice, `str`, a trait object, a
    /// foreign type and a rigid parameter only as a premise. The match lists
    /// every kind, so a new one cannot be Sized by default. Neither a
    /// parameter's declaration nor an interned Sized or
    /// well-formedness property of a type with parameters is read: they belong
    /// to the representative `freeze` chose. rustc's negative query checks no
    /// well-formedness (`resolve_regions` with no WF types), and each header
    /// was checked under its own declarations, so every type is well-formed.
    fn placeholder_composite(
        &mut self,
        types: &Store,
        ty: TypeId,
        kind: Kind,
        depth: u16,
    ) -> Result<Properties, &'static str> {
        use crate::types::sizedness::Rule;
        let child = |child: TypeId| {
            self.cached(child, cache_depth(types, child, depth))
                .map(|properties| properties.sized)
                .ok_or("missing child shape")
        };
        let sized = match kind {
            Kind::Tuple(list) => match types.list(list).ok_or("invalid shape tuple")?.last() {
                Some(&element) => child(element)?,
                None => true,
            },
            Kind::Pattern { base, .. } => child(base)?,
            Kind::Adt(item, arguments) => match types.nominal_size_rule(item) {
                None | Some(Rule::Sized) => true,
                Some(Rule::Unsized) => false,
                Some(Rule::Projection(_)) => *self
                    .nominal
                    .answers
                    .get(&(ty, cache_depth(types, ty, depth)))
                    .ok_or("missing projected nominal size proof")?,
                Some(Rule::Argument(slot)) => child(sizedness_argument(types, arguments, slot)?)?,
            },
            Kind::Parameter(_)
            | Kind::Foreign(_)
            | Kind::Slice(_)
            | Kind::Primitive(Primitive::Str)
            | Kind::Dynamic { .. } => false,
            Kind::Projection(application) => {
                types
                    .projection_schema(application.item)
                    .ok_or("missing associated type declaration")?
                    .sized
                    || self.environment.projection_promises(application.item).sized
            }
            // rustc negates under non-body analysis, where an opaque is a
            // rigid alias that its item bounds prove Sized, implicitly unless
            // relaxed. Those bounds are the opaque's own declaration, never a
            // representative's, whatever its arguments.
            Kind::Opaque(id, _) => {
                types
                    .opaque_declaration(id)
                    .ok_or("invalid opaque identity")?
                    .sized
            }
            // Builtin Sized types with no condition: a coroutine (an authored
            // future), an array (its element's Sized is formation, not
            // sizedness), a reference, a raw or function pointer, a function
            // item and a closure.
            Kind::Never
            | Kind::Primitive(_)
            | Kind::Future { .. }
            | Kind::Array { .. }
            | Kind::ArrayTerm { .. }
            | Kind::Reference { .. }
            | Kind::Pointer { .. }
            | Kind::Function { .. }
            | Kind::FunctionItem { .. }
            | Kind::Closure { .. } => true,
        };
        Ok(Properties {
            sized: sized || self.premise_sized(types, ty, depth)?,
            well_formed: true,
        })
    }
    fn defers_sized(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
    ) -> Result<bool, &'static str> {
        let root_depth = self.nominal.root.map_or(0, |(_, depth)| depth);
        if depth == 0 || !self.defer_local_sized && depth <= root_depth {
            return Ok(false);
        }
        let captures = self
            .scoped
            .get_or_insert_with(Default::default)
            .captures
            .entry(depth)
            .or_insert_with(|| Captures::new(depth));
        let before = captures.steps;
        let result = captures.collect(types, ty);
        let work = captures.steps.saturating_sub(before);
        self.account_lookup(work)?;
        result
    }
    fn cached(&self, ty: TypeId, depth: u16) -> Option<&Properties> {
        if depth == 0 {
            self.cache.get(&ty)
        } else {
            self.scoped.as_ref()?.cache.get(&(ty, depth))
        }
    }
    fn save(&mut self, ty: TypeId, depth: u16, value: Properties) {
        if depth == 0 {
            self.cache.insert(ty, value);
        } else {
            self.scoped
                .get_or_insert_with(Default::default)
                .cache
                .insert((ty, depth), value);
        }
    }
    fn account_lookup(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err("generic shape work budget exceeded");
        }
        Ok(())
    }
    fn lookup(&mut self, depth: u16) -> &mut Lookup {
        self.scoped
            .get_or_insert_with(Default::default)
            .lookups
            .entry(depth)
            .or_insert_with(|| Lookup::new(depth))
    }
    fn assumes_trait(
        &mut self,
        types: &Store,
        application: crate::types::projection::Application,
        depth: u16,
    ) -> Result<bool, &'static str> {
        if depth == 0 {
            return Ok(self
                .environment
                .assumes_trait(application.subject, application.trait_application));
        }
        let lookup = self.lookup(depth);
        let before = lookup.steps();
        let result = (|| {
            let Some(subject) = lookup.ty(types, application.subject)? else {
                return Ok(None);
            };
            Ok(lookup
                .application(types, application.trait_application)?
                .map(|id| (subject, id)))
        })();
        let work = lookup.steps().saturating_sub(before);
        self.account_lookup(work)?;
        let lowered: Option<_> = result?;
        Ok(lowered.is_some_and(|(subject, id)| self.environment.assumes_trait(subject, id)))
    }
    /// A written Sized premise: the environment's, or under placeholders the
    /// other header's, each lookup charged to this walk.
    fn premise_sized(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
    ) -> Result<bool, &'static str> {
        if self.assumes_sized(types, ty, depth)? {
            return Ok(true);
        }
        let Some(placeholders) = &self.placeholders else {
            return Ok(false);
        };
        let (member, work) = placeholders.sized(ty);
        self.account_lookup(work)?;
        Ok(member)
    }
    fn assumes_sized(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
    ) -> Result<bool, &'static str> {
        if self
            .environment
            .matching_sized(types, ty, depth, &mut self.builtins)?
        {
            return Ok(true);
        }
        if depth == 0 {
            return Ok(self.environment.assumes_sized(ty));
        }
        let lookup = self.lookup(depth);
        let before = lookup.steps();
        let result = lookup.ty(types, ty);
        let work = lookup.steps().saturating_sub(before);
        self.account_lookup(work)?;
        Ok(result?.is_some_and(|id| self.environment.assumes_sized(id)))
    }
}
fn cache_depth(types: &Store, ty: TypeId, depth: u16) -> u16 {
    if depth != 0 && types.escaping_binders(ty) == Some(0) {
        0
    } else {
        depth
    }
}
fn children(
    types: &Store,
    kind: Kind,
    depth: u16,
    mut visit: impl FnMut(TypeId, u16) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match kind {
        Kind::Array { element, .. }
        | Kind::ArrayTerm { element, .. }
        | Kind::Pattern { base: element, .. }
        | Kind::Slice(element)
        | Kind::Reference {
            referent: element, ..
        }
        | Kind::Pointer {
            pointee: element, ..
        } => visit(element, depth)?,
        Kind::Closure {
            signature,
            environment,
            arguments,
            ..
        } => {
            visit(signature, depth)?;
            visit(environment, depth)?;
            argument_types(types, arguments, depth, &mut visit)?;
        }
        Kind::Tuple(list) => {
            for ty in types.list(list).ok_or("invalid shape tuple")? {
                visit(*ty, depth)?;
            }
        }
        Kind::Function {
            parameters, result, ..
        } => {
            let depth = depth
                .checked_add(1)
                .ok_or("generic shape binder depth overflow")?;
            for ty in types.list(parameters).ok_or("invalid shape parameters")? {
                visit(*ty, depth)?;
            }
            visit(result, depth)?;
        }
        Kind::FunctionItem {
            signature,
            arguments,
            ..
        } => {
            visit(signature, depth)?;
            argument_types(types, arguments, depth, &mut visit)?;
        }
        Kind::Adt(_, args)
        | Kind::Opaque(_, args)
        | Kind::Future {
            arguments: args, ..
        } => {
            // Field declarations prove the nominal's storage shape once. Uses
            // separately prove its substituted bounds; structural WF includes
            // every written type argument even when it is stored indirectly.
            argument_types(types, args, depth, &mut visit)?;
        }
        Kind::Dynamic { arguments, .. } => {
            let nested = depth
                .checked_add(1)
                .ok_or("generic shape binder depth overflow")?;
            argument_types(types, arguments, nested, &mut visit)?;
        }
        Kind::Projection(application) => {
            // The environment supplies the exact trait premise. Every input must
            // still be well-formed; the opaque result grants only declared Sizedness.
            visit(application.subject, depth)?;
            let trait_ = types
                .trait_application(application.trait_application)
                .ok_or("invalid projected shape trait")?;
            argument_types(types, trait_.arguments, depth, &mut visit)?;
            argument_types(types, application.arguments, depth, &mut visit)?;
        }
        _ => {}
    }
    Ok(())
}

/// The children whose sizedness decides their parent's under rustc 1.98's
/// `sized_conditions`: a tuple's last element, a pattern's base and a
/// nominal's sizedness argument. None of them enters a binder.
fn sized_children(
    types: &Store,
    kind: Kind,
    mut visit: impl FnMut(TypeId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match kind {
        Kind::Tuple(list) => {
            if let Some(&element) = types.list(list).ok_or("invalid shape tuple")?.last() {
                visit(element)?;
            }
        }
        Kind::Pattern { base, .. } => visit(base)?,
        Kind::Adt(item, arguments) => {
            if let Some(crate::types::sizedness::Rule::Argument(slot)) =
                types.nominal_size_rule(item)
            {
                visit(sizedness_argument(types, arguments, slot)?)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn sizedness_argument(
    types: &Store,
    arguments: crate::types::GenericArgs,
    slot: u32,
) -> Result<TypeId, &'static str> {
    let Some(crate::types::arguments::Argument::Type(ty)) = types
        .arguments(arguments)
        .and_then(|mut args| args.nth(slot as usize))
    else {
        return Err("missing nominal sizedness argument");
    };
    Ok(ty)
}

fn argument_types(
    types: &Store,
    arguments: crate::types::GenericArgs,
    depth: u16,
    mut visit: impl FnMut(TypeId, u16) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    for argument in types
        .arguments(arguments)
        .ok_or("invalid shape arguments")?
    {
        if let crate::types::arguments::Argument::Type(ty) = argument {
            visit(ty, depth)?;
        }
    }
    Ok(())
}

/// A formation edge, not a requirement on every generic argument or reference.
/// Tuple tails and callable signature positions introduce no implicit Sized.
pub(crate) fn requires_sized_edge(
    kind: Kind,
    index: usize,
    tuple_len: usize,
    child: TypeId,
) -> bool {
    match kind {
        Kind::Closure { environment, .. } => environment == child,
        Kind::Tuple(_) => index + 1 < tuple_len,
        Kind::Array { .. } | Kind::ArrayTerm { .. } | Kind::Slice(_) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GenericArgs, ItemId, projection, traits};

    #[test]
    fn normalized_shapes_cannot_authorize_a_raw_projection_query() -> Result<(), &'static str> {
        let mut types = Store::default();
        types.declare_projection(
            ItemId(2),
            projection::Schema {
                trait_item: ItemId(1),
                sized: true,
                copies: crate::types::copy::Properties::default(),
            },
        )?;
        let trait_application = types.intern_trait_application(traits::Application {
            item: ItemId(1),
            arguments: GenericArgs::EMPTY,
        })?;
        let projected = types.intern(Kind::Projection(projection::Application {
            item: ItemId(2),
            subject: TypeId::UNIT,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?;
        let tuple = types.intern_list(&[projected])?;
        let tuple = types.intern(Kind::Tuple(tuple))?;
        let environment = Arc::new(Environment::new(&types, &[])?);
        let mut shapes = Shapes::new(environment);
        // Exercise the internal normalized-root entry point as its proof-owning
        // caller would. Its cache must never become evidence in the raw path.
        assert!(shapes.normalized(&types, tuple)?.sized);
        for ty in [tuple, projected] {
            assert_eq!(shapes.try_demand(&types, ty)?, None);
            assert_eq!(
                shapes.demand(&types, ty),
                Err("associated type shape checking requires normalization")
            );
        }
        assert!(shapes.normalized(&types, tuple)?.well_formed);
        Ok(())
    }
}
