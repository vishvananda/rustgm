//! Owned cold snapshots consume solved callable identities before catalog registration.
use super::{Body, Catalog, Code, HashMap, Instance, Owner, ValueId};

/// Created while the exact proved body and solved inputs are still borrowed.
/// This owns executable data, not a reference into a returned template family.
pub(crate) struct Prepared {
    pub(super) code: Code,
    bytes: usize,
}
pub(crate) struct Registration<'catalog> {
    catalog: &'catalog mut Catalog,
    code: std::rc::Rc<Code>,
    bytes: usize,
    index: usize,
    expression: Option<super::NodeKey>,
    scope: Option<crate::const_eval::lease_codes::Scope>,
    actual: crate::types::TypeId,
    source: Option<crate::sema::values::execution::SourceComputation>,
    existing: bool,
}
impl Prepared {
    pub(crate) const fn code(&self) -> &Code {
        &self.code
    }

    pub(crate) fn plain(body: &Body, budget: usize) -> Result<Self, &'static str> {
        Self::snapshot(body, HashMap::default(), budget)
    }
    pub(crate) fn snapshot(
        body: &Body,
        targets: HashMap<Instance, Instance>,
        budget: usize,
    ) -> Result<Self, &'static str> {
        if !targets.is_empty() && body.has_resolved_callables() {
            return Err("prepared code cannot replace an earlier callable proof");
        }
        let bytes =
            Code::storage_bytes(body)?.saturating_add(crate::support::retained::map(&targets));
        if bytes > budget {
            return Err("constant execution code exceeds 64 MiB budget");
        }
        let mut code = Code::snapshot(body)?;
        if !targets.is_empty() {
            code.callables = targets;
        }
        let bytes = Self::storage(&code);
        if bytes > budget {
            return Err("constant execution snapshot payload exceeds storage budget");
        }
        Ok(Self { code, bytes })
    }
    fn storage(code: &Code) -> usize {
        use crate::support::retained::{map, sum, vector};
        sum([
            size_of::<Code>(),
            vector(&code.locals),
            vector(&code.values),
            vector(&code.scalars),
            vector(&code.arguments),
            vector(&code.statements),
            vector(&code.projections),
            vector(&code.blocks),
            vector(&code.operands),
            code.caller_site_bytes(),
            map(&code.promotions),
            map(&code.callables),
        ])
    }
}
impl Catalog {
    pub(super) fn existing_registration(
        &mut self,
        index: usize,
    ) -> Result<Registration<'_>, &'static str> {
        let code = self
            .bodies
            .get(index)
            .map(std::rc::Rc::clone)
            .ok_or("source code index is stale")?;
        let bytes = self.bytes;
        let actual = code.result;
        Ok(Registration {
            catalog: self,
            code,
            bytes,
            index,
            expression: None,
            scope: None,
            actual,
            source: None,
            existing: true,
        })
    }
    pub(super) fn insert_body(
        &mut self,
        body: &Body,
        source_only: bool,
        requested: Option<crate::types::TypeId>,
        results: &[(ValueId, crate::types::TypeId)],
        scope: Option<crate::const_eval::lease_codes::Scope>,
    ) -> Result<usize, &'static str> {
        let prepared = Prepared::plain(body, (64 * 1024 * 1024usize).saturating_sub(self.bytes))?;
        self.publish(prepared, source_only, requested, results, scope)
    }
    pub(super) fn stage(
        &mut self,
        prepared: Prepared,
        source_only: bool,
        requested: Option<crate::types::TypeId>,
        results: &[(ValueId, crate::types::TypeId)],
        scope: Option<crate::const_eval::lease_codes::Scope>,
    ) -> Result<Registration<'_>, &'static str> {
        let Prepared { mut code, bytes } = prepared;
        if !source_only
            && matches!(code.owner, Owner::AnonymousInstance(instance)
            if matches!(instance.enclosing(), crate::sema::inline_constants::Context::SourceInitializer(_)))
        {
            return Err("source anonymous code requires checked source publication");
        }
        if u32::try_from(code.values.len()).is_err() {
            return Err("constant CFG value identity limit exceeded");
        }
        let published = requested.unwrap_or(code.result);
        let bytes = self.index_bytes(&code, source_only, bytes)?;
        let index = self.bodies.len();
        let expression = self.available(&code, published, source_only, scope)?;
        let bytes = self.scope_bytes(scope, code.owner, published, index, bytes)?;
        code.transport_inline_results(source_only, results)?;
        code.source_only = source_only;
        let actual = code.result;
        code.result = published;
        Ok(Registration {
            catalog: self,
            code: std::rc::Rc::new(code),
            bytes,
            index,
            expression,
            scope,
            actual,
            source: None,
            existing: false,
        })
    }
    pub(super) fn publish(
        &mut self,
        prepared: Prepared,
        source_only: bool,
        requested: Option<crate::types::TypeId>,
        results: &[(ValueId, crate::types::TypeId)],
        scope: Option<crate::const_eval::lease_codes::Scope>,
    ) -> Result<usize, &'static str> {
        self.stage(prepared, source_only, requested, results, scope)?
            .publish()
    }
    fn index_bytes(
        &self,
        code: &Code,
        source_only: bool,
        payload: usize,
    ) -> Result<usize, &'static str> {
        let extra = if matches!(code.owner, Owner::AnonymousConstant) {
            crate::support::retained::new_map::<super::NodeKey, usize>(1)
        } else if matches!(code.owner, Owner::AnonymousInstance(_)) {
            crate::support::retained::new_map::<super::InlineKey, usize>(1).saturating_add(
                if self.anonymous.is_none() {
                    size_of::<super::AnonymousIndex>()
                } else {
                    0
                },
            )
        } else if !source_only && matches!(code.owner, Owner::InitializerInstance { .. }) {
            crate::support::retained::new_map::<
                (crate::sema::values::Instance, crate::types::TypeId),
                usize,
            >(1)
            .saturating_add(if self.associated.is_none() {
                size_of::<super::AssociatedIndex>()
            } else {
                0
            })
        } else {
            0
        };
        self.bytes
            .checked_add(payload)
            .and_then(|n| n.checked_add(extra))
            .filter(|&n| n <= 64 * 1024 * 1024)
            .ok_or("constant execution code exceeds 64 MiB budget")
    }
    fn available(
        &self,
        code: &Code,
        published: crate::types::TypeId,
        source_only: bool,
        scope: Option<crate::const_eval::lease_codes::Scope>,
    ) -> Result<Option<super::NodeKey>, &'static str> {
        let mut expression = None;
        if let Some(instance) = code.instance {
            if self.functions.contains_key(&instance) {
                return Err("duplicate constant function instance");
            }
        } else if let Owner::Function { definition, .. } | Owner::Initializer { definition, .. } =
            code.owner
        {
            if self.definitions.contains_key(&definition) {
                return Err("duplicate constant body definition");
            }
        } else if matches!(code.owner, Owner::AnonymousConstant) {
            let source = code
                .source
                .source()
                .ok_or("imported anonymous Code requires its exact body-key index")?;
            if self.expressions.contains_key(&source) {
                return Err("duplicate anonymous constant body");
            }
            expression = Some(source);
        }
        if let Owner::AnonymousInstance(instance) = code.owner
            && scope.is_none()
            && self
                .anonymous
                .as_deref()
                .is_some_and(|map| map.ordinary.contains_key(&(instance, published)))
        {
            return Err("duplicate anonymous constant instance body");
        }
        if let Owner::InitializerInstance { instance, .. } = code.owner
            && !source_only
            && self
                .associated
                .as_deref()
                .is_some_and(|map| map.contains_key(&(instance, code.result)))
        {
            return Err("duplicate selected initializer execution body");
        }
        Ok(expression)
    }
    fn scope_bytes(
        &self,
        scope: Option<crate::const_eval::lease_codes::Scope>,
        owner: Owner,
        published: crate::types::TypeId,
        index: usize,
        bytes: usize,
    ) -> Result<usize, &'static str> {
        let Some(scope) = scope else {
            return Ok(bytes);
        };
        let Owner::AnonymousInstance(instance) = owner else {
            return Err("conditional code has no exact inline instance");
        };
        self.anonymous
            .as_deref()
            .and_then(|map| map.scoped.as_deref())
            .ok_or("conditional code has no live scope")?
            .insertion_bytes(scope, (instance, published), index, bytes)
    }
    fn register_scope(
        &mut self,
        scope: Option<crate::const_eval::lease_codes::Scope>,
        owner: Owner,
        published: crate::types::TypeId,
        index: usize,
    ) -> Result<(), &'static str> {
        let Some(scope) = scope else {
            return Ok(());
        };
        let Owner::AnonymousInstance(instance) = owner else {
            return Err("conditional code has no exact inline instance");
        };
        self.anonymous
            .as_mut()
            .and_then(|map| map.scoped.as_mut())
            .ok_or("conditional code has no live scope")?
            .insert(scope, (instance, published), index, &mut self.bytes)
    }
}

impl Registration<'_> {
    pub(crate) fn publish_source(
        self,
        facts: &mut crate::sema::model::Declarations,
        inputs: &mut Option<Box<crate::const_eval::use_inputs::Inputs>>,
        source: Option<crate::const_eval::use_inputs::Source>,
        selected: Option<Instance>,
    ) -> Result<usize, &'static str> {
        let Some(source) = source else {
            return self.publish();
        };
        if self.existing {
            inputs
                .as_deref()
                .ok_or("cached source code lost its actual input owner")?
                .get(self.index, self.catalog)?;
            return Ok(self.index);
        }
        let inputs = inputs.get_or_insert_with(Default::default);
        let plan = inputs.plan(
            facts,
            self.index,
            std::rc::Rc::clone(&self.code),
            source,
            selected,
        )?;
        inputs.check_plan(&plan, self.index, &self.code)?;
        let index = self.publish()?;
        // Inputs::plan reserved this exact row's map slot; no fallible work or
        // callback occurs between canonical Code publication and this insertion.
        inputs.commit(plan);
        Ok(index)
    }
    pub(crate) fn publish(self) -> Result<usize, &'static str> {
        let Self {
            catalog,
            code,
            bytes,
            index,
            expression,
            scope,
            actual,
            source,
            existing,
        } = self;
        if existing {
            return Ok(index);
        }
        // The only remaining fallible operation runs before any Code/Row index.
        catalog.register_scope(scope, code.owner, code.result, index)?;
        if let Some(instance) = code.instance {
            catalog.functions.insert(instance, index);
        } else if let Owner::Function { definition, .. } | Owner::Initializer { definition, .. } =
            code.owner
        {
            catalog.definitions.insert(definition, index);
        } else if let Some(source) = expression {
            catalog.expressions.insert(source, index);
        }
        if let Owner::AnonymousInstance(instance) = code.owner
            && scope.is_none()
        {
            catalog
                .anonymous
                .get_or_insert_with(Default::default)
                .ordinary
                .insert((instance, code.result), index);
        }
        if let Owner::InitializerInstance { instance, .. } = code.owner
            && !code.source_only
        {
            catalog
                .associated
                .get_or_insert_with(Default::default)
                .insert((instance, actual), index);
        }
        if let Some(source) = source {
            catalog
                .sources
                .get_or_insert_with(Default::default)
                .insert(source, (index, actual));
        }
        catalog.bytes = bytes;
        catalog.bodies.push(code);
        Ok(index)
    }
    pub(super) fn source(
        mut self,
        source: crate::sema::values::execution::SourceComputation,
    ) -> Result<Self, &'static str> {
        let extra = crate::support::retained::new_map::<
            crate::sema::values::execution::SourceComputation,
            (usize, crate::types::TypeId),
        >(1)
        .saturating_add(if self.catalog.sources.is_none() {
            size_of::<super::SourceIndex>()
        } else {
            0
        });
        self.bytes = self
            .bytes
            .checked_add(extra)
            .filter(|&n| n <= 64 * 1024 * 1024)
            .ok_or("source code index storage budget exceeded")?;
        self.source = Some(source);
        Ok(self)
    }
}
