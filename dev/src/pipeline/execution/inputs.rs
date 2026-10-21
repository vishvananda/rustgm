//! Temporary lending preserves each ordinary CFG's ownership and final order.
use super::{Engine, Proxy};
use crate::{
    body::{
        self,
        ir::{Owner, Program},
        templates::Templates,
    },
    resolve::{expansion::Report, model::DefId, package::Error, unit::FileId},
    sema::{model::Declarations, values::execution::Executor},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Default)]
pub(crate) struct Context {
    engine: Rc<RefCell<Engine>>,
}
enum Slot {
    Local(usize),
    Shared(DefId),
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
    pub(crate) fn transfer(
        &self,
        report: &Report,
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
            .transfer(report, prepared)
    }
    pub(crate) fn templates(&self, templates: Rc<Templates>) -> Result<(), &'static str> {
        let mut engine = self
            .engine
            .try_borrow_mut()
            .map_err(|_| "constant provider is still executing during template publication")?;
        engine.runtime_template_indices.clear();
        for (index, template) in templates.bodies.iter().enumerate() {
            if let Owner::Function { definition, .. } = template.body.owner {
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
                if let Owner::Function { definition, .. } = body.owner
                    && facts.is_const_function(definition)
                {
                    store.insert(definition, body);
                    slots.push(Slot::Shared(definition));
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
                slots.iter().filter_map(|slot| match slot {
                    Slot::Local(index) => local.get(*index),
                    Slot::Shared(definition) => store.get(definition),
                }),
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
