//! Temporary lending preserves each ordinary CFG's ownership and final order.
use super::{Engine, Proxy};
use crate::{
    body::{
        self,
        ir::{BodyKey, Owner, Program},
        templates::Templates,
    },
    resolve::{expansion::Report, model::DefId, package::Error, unit::FileId},
    sema::{model::Declarations, values::execution::Executor},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Default)]
pub(crate) struct Context {
    pub(super) engine: Rc<RefCell<Engine>>,
}
enum Slot {
    Local(usize),
    Shared(BodyKey),
}
impl Context {
    #[cfg(test)]
    pub(super) fn bounded(limits: crate::const_eval::Limits) -> Self {
        Self {
            engine: Rc::new(RefCell::new(Engine {
                limits,
                ..Engine::default()
            })),
        }
    }
    pub(crate) fn executor(&self) -> Box<dyn Executor> {
        Box::new(Proxy(Rc::clone(&self.engine)))
    }
    pub(crate) fn check_inline_bodies(
        &self,
        report: &Report,
        facts: &mut Declarations,
    ) -> Result<(), Error> {
        let count = facts
            .inline_constants
            .as_deref()
            .map_or(0, crate::sema::inline_constants::Registry::len);
        if count == 0 {
            return Ok(());
        }
        let mut engine = self.engine.try_borrow_mut().map_err(|_| {
            Error::at(
                FileId(0),
                0,
                "constant provider is active during inline body checking",
            )
        })?;
        for index in 0..count {
            let (source, result) = facts
                .inline_constants
                .as_deref()
                .and_then(|registry| registry.at(index))
                .and_then(|body| body.result.map(|result| (body.source, result)))
                .ok_or_else(|| Error::at(FileId(0), 0, "inline body type did not complete"))?;
            if let Some(registry) = facts.inline_constants.as_mut() {
                registry.preparation_visits += 1;
            }
            if facts
                .inline_constants
                .as_deref()
                .and_then(|r| r.get(source))
                .is_some_and(|b| {
                    b.evaluation == crate::sema::inline_constants::Evaluation::DeferredTemplate
                })
            {
                engine.retain_anonymous(report, facts, source)?;
                continue;
            }
            engine.root(
                report,
                facts,
                crate::sema::values::execution::Target::Expression {
                    source,
                    result,
                    context: None,
                },
            )?;
        }
        Ok(())
    }
    pub(crate) fn transfer(
        &self,
        report: &Report,
        facts: &mut Declarations,
        prepared: &mut crate::pipeline::Prepared,
    ) -> Result<(), Error> {
        self.engine
            .try_borrow_mut()
            .map_err(|_| {
                Error::at(
                    FileId(0),
                    0,
                    "constant provider is still executing during handoff",
                )
            })?
            .transfer(report, facts, prepared)
    }
    pub(crate) fn templates(&self, templates: Rc<Templates>) -> Result<(), &'static str> {
        let mut engine = self
            .engine
            .try_borrow_mut()
            .map_err(|_| "constant provider is still executing during template publication")?;
        engine.runtime_template_indices.clear();
        for (index, template) in templates.bodies.iter().enumerate() {
            if let Owner::Function { definition, .. } | Owner::Initializer { definition, .. } =
                template.body.owner
            {
                engine.runtime_template_indices.insert(definition, index);
            }
        }
        engine.runtime_templates = Some(templates);
        Ok(())
    }
    pub(crate) fn constants(
        &self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut Program,
        globals: &[DefId],
    ) -> Result<(), Error> {
        self.constants_except(
            report,
            facts,
            program,
            globals,
            &crate::support::hash::HashSet::default(),
        )
    }
    pub(crate) fn constants_except(
        &self,
        report: &Report,
        facts: &mut Declarations,
        program: &mut Program,
        globals: &[DefId],
        skip: &crate::support::hash::HashSet<BodyKey>,
    ) -> Result<(), Error> {
        let inputs = Rc::clone(
            &self
                .engine
                .try_borrow()
                .map_err(|_| {
                    Error::at(
                        FileId(0),
                        0,
                        "constant provider is still executing before CFG lending",
                    )
                })?
                .inputs,
        );
        let mut slots = Vec::with_capacity(program.bodies.len());
        let mut local = Vec::new();
        {
            let mut store = inputs
                .try_borrow_mut()
                .map_err(|_| Error::at(FileId(0), 0, "constant CFG store is already borrowed"))?;
            for body in program.bodies.drain(..) {
                let shared = match body.owner {
                    Owner::Function { definition, .. } => facts.is_const_function(definition),
                    Owner::AnonymousConstant => facts
                        .inline_constants
                        .as_deref()
                        .is_some_and(|registry| registry.get(body.source).is_some()),
                    Owner::Initializer { .. }
                    | Owner::Closure { .. }
                    | Owner::ClosureAdapter { .. }
                    | Owner::AnonymousInstance(_)
                    | Owner::InitializerInstance { .. } => false,
                };
                if shared {
                    let key = body.key();
                    if store.contains_key(&key) {
                        return Err(Error::at(
                            body.source.file,
                            0,
                            "duplicate lent constant CFG owner",
                        ));
                    }
                    store.insert(key, body);
                    slots.push(Slot::Shared(key));
                } else {
                    slots.push(Slot::Local(local.len()));
                    local.push(body);
                }
            }
        }
        let result = {
            let store = inputs
                .try_borrow()
                .map_err(|_| Error::at(FileId(0), 0, "constant CFG store is unavailable"))?;
            body::constants::analyze_bodies(
                report,
                facts,
                slots
                    .iter()
                    .filter_map(|slot| match slot {
                        Slot::Local(index) => local.get(*index),
                        Slot::Shared(definition) => store.get(definition),
                    })
                    .filter(|body| !skip.contains(&body.key())),
                globals,
            )
        };
        let mut store = inputs.try_borrow_mut().map_err(|_| {
            Error::at(
                FileId(0),
                0,
                "constant CFG lender retained an active borrow",
            )
        })?;
        let mut local = local.into_iter();
        for slot in slots {
            let body = match slot {
                Slot::Local(_) => local
                    .next()
                    .ok_or_else(|| Error::at(FileId(0), 0, "local CFG ownership was lost"))?,
                Slot::Shared(definition) => store
                    .remove(&definition)
                    .ok_or_else(|| Error::at(FileId(0), 0, "constant CFG ownership was lost"))?,
            };
            program.bodies.push(body);
        }
        result
    }
}
