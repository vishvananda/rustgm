//! Structural well-formedness and sizedness under one immutable bound environment.
use super::environment::Environment;
use crate::types::{Kind, Primitive, Store, TypeId, substitute::lookup::Lookup};
use std::{collections::HashMap, sync::Arc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Properties {
    pub sized: bool,
    pub well_formed: bool,
}
pub struct Shapes {
    environment: Arc<Environment>,
    cache: HashMap<TypeId, Properties>,
    normalized_queries: Option<Box<Self>>,
    scoped: Option<Box<Scoped>>,
    work: Vec<(TypeId, u16, bool)>,
    steps: usize,
}
#[derive(Default)]
struct Scoped {
    cache: HashMap<(TypeId, u16), Properties>,
    lookups: HashMap<u16, Lookup>,
}
impl Shapes {
    #[must_use]
    pub fn new(environment: Arc<Environment>) -> Self {
        Self {
            environment,
            cache: HashMap::new(),
            normalized_queries: None,
            scoped: None,
            work: Vec::new(),
            steps: 0,
        }
    }
    /// # Errors
    /// Rejects invalid/foreign parameter types and exhausted graph work.
    pub fn demand(&mut self, types: &Store, ty: TypeId) -> Result<Properties, &'static str> {
        self.try_demand(types, ty)?
            .ok_or("associated type shape checking requires normalization")
    }
    /// A complete structural proof under the fixed assumptions avoids repeating
    /// normalization. None requests parent evidence without caching permission.
    pub(crate) fn try_demand(
        &mut self,
        types: &Store,
        ty: TypeId,
    ) -> Result<Option<Properties>, &'static str> {
        self.demand_with(types, ty, false)
    }
    /// The caller has normalized this entire root in this query's environment
    /// and retained every parent formation proof, including nested binders.
    pub(crate) fn normalized(
        &mut self,
        types: &Store,
        ty: TypeId,
    ) -> Result<Properties, &'static str> {
        // Raw structural queries cannot reuse a cache populated under a
        // separately retained normalization proof.
        self.normalized_queries
            .get_or_insert_with(|| Box::new(Self::new(Arc::clone(&self.environment))))
            .demand_with(types, ty, true)?
            .ok_or("missing normalized shape proof")
    }
    fn demand_with(
        &mut self,
        types: &Store,
        ty: TypeId,
        normalized: bool,
    ) -> Result<Option<Properties>, &'static str> {
        if let Some(value) = self.cache.get(&ty) {
            return Ok(Some(*value));
        }
        self.work.clear();
        self.work.push((ty, 0, false));
        while let Some((ty, depth, finish)) = self.work.pop() {
            self.steps += 1;
            if self.steps.saturating_add(self.work.len()) > 1_000_000 {
                return Err("generic shape work budget exceeded");
            }
            if self.cached(ty, depth).is_some() {
                continue;
            }
            let kind = types.kind(ty).ok_or("invalid generic shape type")?;
            if let Kind::Projection(application) = kind
                && !normalized
                && !self.assumes_trait(types, application, depth)?
            {
                return Ok(None);
            }
            if types.contains_parameters(ty) == Some(false)
                && types.contains_projections(ty) == Some(false)
            {
                let sized =
                    types.is_sized(ty) == Some(true) || self.assumes_sized(types, ty, depth)?;
                self.save(
                    ty,
                    depth,
                    Properties {
                        sized,
                        well_formed: types.is_well_formed(ty) == Some(true),
                    },
                );
                continue;
            }
            if !finish {
                self.work.push((ty, depth, true));
                children(types, kind, depth, |child, depth| {
                    if self.steps.saturating_add(self.work.len()) >= 1_000_000 {
                        return Err("generic shape work budget exceeded");
                    }
                    self.work
                        .push((child, cache_depth(types, child, depth), false));
                    Ok(())
                })?;
                continue;
            }
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
                let child = self
                    .cached(child, cache_depth(types, child, depth))
                    .ok_or("missing child shape")?;
                value.well_formed &= child.well_formed;
                match kind {
                    Kind::Tuple(_) => {
                        value.sized &= child.sized;
                        value.well_formed &= child.sized || index + 1 == tuple_len;
                    }
                    Kind::Array { .. } | Kind::ArrayTerm { .. } | Kind::Slice(_) => {
                        value.sized &= child.sized;
                        value.well_formed &= child.sized;
                    }
                    _ => {}
                }
                index += 1;
                Ok(())
            })?;
            match kind {
                Kind::Parameter(id) => {
                    let abilities = self
                        .environment
                        .abilities(id)
                        .ok_or("shape parameter is outside its environment")?;
                    value.sized = types.is_sized(ty) == Some(true) || abilities.clone_;
                }
                Kind::Slice(_) | Kind::Primitive(Primitive::Str) => value.sized = false,
                Kind::Projection(application) => {
                    value.sized = types.is_sized(ty) == Some(true)
                        || self.environment.projection_promises(application.item).sized;
                }
                _ => {}
            }
            value.sized = value.sized || self.assumes_sized(types, ty, depth)?;
            self.save(ty, depth, value);
        }
        self.cache
            .get(&ty)
            .copied()
            .map(Some)
            .ok_or("missing generic shape result")
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
    fn assumes_sized(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
    ) -> Result<bool, &'static str> {
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
        | Kind::Slice(element)
        | Kind::Reference {
            referent: element, ..
        }
        | Kind::Pointer {
            pointee: element, ..
        } => visit(element, depth)?,
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
        Kind::Adt(_, args) => {
            // Field declarations prove the nominal's storage shape once. Uses
            // separately prove its substituted bounds; structural WF includes
            // every written type argument even when it is stored indirectly.
            argument_types(types, args, depth, &mut visit)?;
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
