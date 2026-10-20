//! Parenthesized arguments reuse declaration typing and per-parameter elision work.
use super::{Checker, Error, NodeKey, Type};
use crate::{
    sema::{
        declarations::{Work, regions},
        lifetimes,
    },
    types::{GenericArgs, arguments::Argument},
};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn schedule_call_arguments(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let (arguments, _) = lifetimes::call_arguments(self.report, key)?
            .ok_or_else(|| self.error(key, "missing parenthesized call arguments"))?;
        if !self.call_paths.insert(key) {
            return Ok(());
        }
        let binder = context
            .binder
            .ok_or_else(|| self.error(key, "missing call-bound binder"))?;
        self.work.push(Work::CallTraitOutput(key, context));
        let ast = &self.report.package.unit(key.file).configured.ast;
        for node in ast.list(arguments).iter().rev().copied() {
            self.work.push(Work::Parameter(binder, true));
            self.work.push(Work::Node(
                NodeKey { node, ..key },
                false,
                regions::Context {
                    position: regions::Position::Input,
                    ..context
                },
            ));
            self.work.push(Work::Parameter(binder, false));
        }
        self.work.push(Work::CallTraitInputs(binder));
        Ok(())
    }

    pub(in crate::sema::declarations) fn call_trait_output(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let (_, result) = lifetimes::call_arguments(self.report, key)?
            .ok_or_else(|| self.error(key, "missing call output syntax"))?;
        if let Some(result) = result {
            self.node_work(
                key.file,
                result,
                regions::Context {
                    position: regions::Position::Output,
                    ..context
                },
            );
        }
        Ok(())
    }

    pub(in crate::sema::declarations) fn call_application_arguments(
        &mut self,
        key: NodeKey,
    ) -> Result<GenericArgs, Error> {
        let (list, _) = lifetimes::call_arguments(self.report, key)?
            .ok_or_else(|| self.error(key, "missing call application syntax"))?;
        let ast = &self.report.package.unit(key.file).configured.ast;
        let mut arguments = Vec::with_capacity(ast.list(list).len());
        let mut originals = Vec::with_capacity(arguments.capacity());
        for node in ast.list(list) {
            let key = NodeKey { node: *node, ..key };
            let ty = self.ty(key)?;
            arguments.push(ty);
            originals.push(self.facts.unreduced.node(key).unwrap_or(ty));
        }
        let fields = self
            .facts
            .types
            .intern_list(&arguments)
            .map_err(|e| self.error(key, e))?;
        let tuple = self.intern(key, Type::Tuple(fields))?;
        let result = self
            .facts
            .types
            .intern_arguments(&[Argument::Type(tuple)])
            .map_err(|e| self.error(key, e))?;
        if originals != arguments {
            let fields = self
                .facts
                .types
                .intern_list(&originals)
                .map_err(|e| self.error(key, e))?;
            let tuple = self.intern(key, Type::Tuple(fields))?;
            let original = self
                .facts
                .types
                .intern_arguments(&[Argument::Type(tuple)])
                .map_err(|e| self.error(key, e))?;
            self.facts.unreduced.retain_arguments(key, original);
        }
        Ok(result)
    }
}
