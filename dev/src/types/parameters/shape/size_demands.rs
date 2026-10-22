//! Hidden storage conditions suspend one shape frontier, including its callable scopes.
use super::{Properties, Shapes, cache_depth};
use crate::support::hash::HashMap;
use crate::types::{Kind, Store, TypeId, lifetime_proof::ScopeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Request {
    pub ty: TypeId,
    pub depth: u16,
    pub scope: ScopeId,
}
pub(crate) enum Progress {
    Ready(Properties),
    Projection,
    Size(Request),
}

#[derive(Default)]
pub(super) struct Sizes {
    pub answers: HashMap<(TypeId, u16), bool>,
    pub pending: Option<Request>,
    pub root: Option<(TypeId, u16)>,
    binders: Vec<(ScopeId, u32)>,
}
impl Sizes {
    pub fn cancel(&mut self) {
        self.root = None;
        self.pending = None;
        self.binders.clear();
    }
    pub fn resume(&mut self, ty: TypeId, depth: u16) -> Result<bool, &'static str> {
        if let Some(root) = self.root {
            if root != (ty, depth) {
                return Err("unfinished shape query changed its root");
            }
            if self.pending.is_some() {
                return Err("shape query resumed without its size answer");
            }
            return Ok(true);
        }
        self.root = Some((ty, depth));
        Ok(false)
    }

    pub fn children(&mut self, kind: Kind, parent: ScopeId) -> Result<ScopeId, &'static str> {
        let Kind::Function { bound_regions, .. } = kind else {
            return Ok(parent);
        };
        if self.binders.len() >= 1_000_000 {
            return Err("shape binder work budget exceeded");
        }
        self.binders.push((parent, bound_regions));
        let index = u32::try_from(self.binders.len()).map_err(|_| "too many shape binders")?;
        Ok(ScopeId(index))
    }
}

impl Shapes {
    /// A suspended query keeps its completed siblings and pending parent frames.
    /// The caller supplies only the selected nominal's sizedness, never its WF.
    pub(crate) fn advance(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
        normalized: bool,
    ) -> Result<Progress, &'static str> {
        let query = self.query(normalized);
        let result = query.demand_with(types, ty, depth, normalized)?;
        if let Some(value) = result {
            Ok(Progress::Ready(value))
        } else if let Some(request) = query.nominal.pending {
            Ok(Progress::Size(request))
        } else {
            query.nominal.root = None;
            Ok(Progress::Projection)
        }
    }

    pub(crate) fn answer_size(
        &mut self,
        types: &Store,
        normalized: bool,
        request: Request,
        sized: bool,
    ) -> Result<(), &'static str> {
        let query = self.query(normalized);
        if query.nominal.pending != Some(request) {
            return Err("nominal size response does not match its pending shape request");
        }
        query.nominal.answers.insert(
            (request.ty, cache_depth(types, request.ty, request.depth)),
            sized,
        );
        query.nominal.pending = None;
        Ok(())
    }

    pub(crate) fn size_binder(
        &mut self,
        normalized: bool,
        scope: ScopeId,
    ) -> Result<(ScopeId, u32), &'static str> {
        scope
            .0
            .checked_sub(1)
            .and_then(|id| self.query(normalized).nominal.binders.get(id as usize))
            .copied()
            .ok_or("invalid shape size binder")
    }

    fn query(&mut self, normalized: bool) -> &mut Self {
        if !normalized {
            return self;
        }
        self.normalized_queries.get_or_insert_with(|| {
            let mut query = Self::new(std::sync::Arc::clone(&self.environment));
            query.defer_local_sized = self.defer_local_sized;
            Box::new(query)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        GenericArgs, ItemId, Primitive,
        aggregate::{self, Definitions, Repr, Variant},
        parameters::environment::Environment,
        projection,
        sizedness::Rules,
        traits,
    };
    use std::sync::Arc;

    fn fixture(count: u32) -> Result<(Store, Vec<TypeId>), &'static str> {
        let mut types = Store::default();
        types.declare_projection(
            ItemId(1),
            projection::Schema {
                trait_item: ItemId(0),
                sized: false,
                copies: crate::types::copy::Properties::default(),
            },
        )?;
        let application = types.intern_trait_application(traits::Application {
            item: ItemId(0),
            arguments: GenericArgs::EMPTY,
        })?;
        let projected = types.intern(Kind::Projection(projection::Application {
            item: ItemId(1),
            subject: TypeId::primitive(Primitive::U8),
            trait_application: application,
            arguments: GenericArgs::EMPTY,
        }))?;
        let mut definitions = Definitions::default();
        let mut rules = Rules::default();
        let mut nominals = Vec::new();
        for index in 0..count {
            let item = ItemId(index + 2);
            let fields = types.intern_list(&[projected])?;
            definitions.insert(
                item,
                aggregate::Kind::Struct,
                Repr::default(),
                &[Variant {
                    fields,
                    discriminant: 0,
                }],
            )?;
            assert_eq!(
                rules.prepare(&mut types, &definitions, item, |_, _, _| Ok(false))?,
                None
            );
            nominals.push(types.intern(Kind::Adt(item, GenericArgs::EMPTY))?);
        }
        types.seal_nominal_sizes(Arc::new(rules))?;
        Ok((types, nominals))
    }

    #[test]
    fn terminal_failures_release_the_frontier_but_wrong_resume_preserves_it()
    -> Result<(), &'static str> {
        let (types, nominals) = fixture(1)?;
        let environment = Arc::new(Environment::new(&types, &[])?);
        let mut shapes = Shapes::new(environment);
        let byte = TypeId::primitive(Primitive::U8);
        assert!(shapes.demand(&types, nominals[0]).is_err());
        assert!(shapes.demand(&types, byte)?.sized);
        let Progress::Size(request) = shapes.advance(&types, nominals[0], 0, false)? else {
            return Err("missing test size suspension");
        };
        assert!(
            shapes
                .advance(&types, TypeId::primitive(Primitive::U16), 0, false)
                .is_err()
        );
        shapes.answer_size(&types, false, request, false)?;
        let Progress::Ready(answer) = shapes.advance(&types, nominals[0], 0, false)? else {
            return Err("valid suspension was lost");
        };
        assert!(!answer.sized && answer.well_formed);
        Ok(())
    }

    #[test]
    fn size_waits_preserve_siblings_in_linear_work() -> Result<(), &'static str> {
        for count in [128, 256, 512, 1024] {
            let (mut types, nominals) = fixture(count)?;
            let fields = types.intern_list(&nominals)?;
            let root = types.intern(Kind::Tuple(fields))?;
            let environment = Arc::new(Environment::new(&types, &[])?);
            let mut shapes = Shapes::new(environment);
            let mut requests = 0;
            loop {
                match shapes.advance(&types, root, 0, false)? {
                    Progress::Ready(value) => {
                        assert!(value.sized && value.well_formed);
                        break;
                    }
                    Progress::Size(request) => {
                        assert_eq!(request.scope, ScopeId::ROOT);
                        shapes.answer_size(&types, false, request, true)?;
                        requests += 1;
                    }
                    Progress::Projection => return Err("unexpected written projection"),
                }
            }
            assert_eq!(requests, count);
            assert!(shapes.steps() < count as usize * 5, "{}", shapes.steps());
            let before = shapes.steps();
            assert!(matches!(
                shapes.advance(&types, root, 0, false)?,
                Progress::Ready(_)
            ));
            assert_eq!(shapes.steps(), before);
        }
        Ok(())
    }

    #[test]
    fn requests_retain_empty_and_nested_callable_binders() -> Result<(), &'static str> {
        let (mut types, nominals) = fixture(1)?;
        let array = types.intern(Kind::Array {
            element: nominals[0],
            length: 2,
        })?;
        let parameters = types.intern_list(&[array])?;
        let inner = types.intern(Kind::Function {
            parameters,
            result: TypeId::UNIT,
            bound_regions: 2,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        let parameters = types.intern_list(&[inner])?;
        let root = types.intern(Kind::Function {
            parameters,
            result: TypeId::UNIT,
            bound_regions: 0,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        let mut shapes = Shapes::new(Arc::new(Environment::new(&types, &[])?));
        let Progress::Size(request) = shapes.advance(&types, root, 0, false)? else {
            return Err("missing size request");
        };
        assert_eq!(request.depth, 2);
        let (parent, count) = shapes.size_binder(false, request.scope)?;
        assert_eq!(count, 2);
        assert_eq!(shapes.size_binder(false, parent)?, (ScopeId::ROOT, 0));
        shapes.answer_size(&types, false, request, false)?;
        let Progress::Ready(value) = shapes.advance(&types, root, 0, false)? else {
            return Err("missing resumed shape");
        };
        assert!(value.sized);
        assert!(!value.well_formed);
        Ok(())
    }
}
