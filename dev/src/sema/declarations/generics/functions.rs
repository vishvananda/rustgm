//! Elaborate written function arguments once; body inference owns omitted slots.
use super::{Argument, Checker, DefId, Error, Kind, NodeKey, Work, regions};
use crate::{
    sema::generics::{FunctionUse, SourceArgument},
    syntax::ast::{List, NodeId},
};

impl Checker<'_> {
    fn function_argument_lists(
        &self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(Option<List>, Option<List>), Error> {
        let Some(list) = crate::sema::lifetimes::arguments(self.report, key, false)? else {
            return Ok((None, None));
        };
        let nodes = self.report.package.unit(key.file).configured.ast.list(list);
        let lifetimes = nodes
            .iter()
            .take_while(|node| {
                self.lifetime_argument(NodeKey {
                    node: **node,
                    ..key
                })
            })
            .count();
        let count = nodes.len() - lifetimes;
        if count != 0 && count != self.facts.generics.parameters[&owner].len() {
            return Err(self.error(key, "incorrect number of type and const arguments"));
        }
        let middle = list
            .start
            .checked_add(
                u32::try_from(lifetimes)
                    .map_err(|_| self.error(key, "too many lifetime arguments"))?,
            )
            .ok_or_else(|| self.error(key, "generic argument range overflow"))?;
        Ok((
            (lifetimes != 0).then_some(List {
                start: list.start,
                end: middle,
            }),
            (count != 0).then_some(List {
                start: middle,
                end: list.end,
            }),
        ))
    }
    pub(in super::super) fn schedule_function_arguments(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        self.demand_function_template(key, owner)?;
        let (_, supplied) = self.function_argument_lists(key, owner)?;
        self.work.push(Work::FunctionArguments(key, owner));
        let Some(list) = supplied else {
            return Ok(());
        };
        let ast = &self.report.package.unit(key.file).configured.ast;
        for (index, node) in ast.list(list).iter().copied().enumerate().rev() {
            let key = NodeKey { node, ..key };
            if matches!(self.kind(key), Kind::InferType) {
                continue;
            }
            match self.facts.generics.parameters[&owner][index].argument {
                Argument::Type(_) => {
                    self.annotation_holes(key, steps, limit, scan)?;
                    self.node_work(key.file, node, context);
                }
                Argument::Const(_) => self.constant_dependencies(key)?,
                Argument::Lifetime(_) => {
                    return Err(
                        self.error(key, "function parameter metadata contains a lifetime slot")
                    );
                }
            }
        }
        Ok(())
    }
    fn demand_function_template(&mut self, key: NodeKey, owner: DefId) -> Result<(), Error> {
        if self
            .facts
            .lifetimes
            .callables
            .get(crate::types::ItemId(owner.0))
            .is_some()
        {
            return Ok(());
        }
        let ty = self.facts.definitions[owner.0 as usize]
            .ok_or_else(|| self.error(key, "missing generic function type"))?;
        let Some(crate::types::Kind::FunctionItem { signature, .. }) = self.facts.types.kind(ty)
        else {
            return Err(self.error(key, "missing generic function signature"));
        };
        let scheme = self
            .facts
            .lifetimes
            .functions
            .get(&owner)
            .cloned()
            .ok_or_else(|| self.error(key, "missing generic function lifetime scheme"))?;
        self.callable_template(key, owner, signature, &scheme)
    }
    pub(in super::super) fn complete_function_arguments(
        &mut self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(), Error> {
        let (lifetimes, supplied) = self.function_argument_lists(key, owner)?;
        let mut arguments = Vec::new();
        if let Some(list) = supplied {
            let count = self
                .report
                .package
                .unit(key.file)
                .configured
                .ast
                .list(list)
                .len();
            for index in 0..count {
                let node = self.report.package.unit(key.file).configured.ast.list(list)[index];
                let key = NodeKey { node, ..key };
                arguments.push(if matches!(self.kind(key), Kind::InferType) {
                    SourceArgument::Infer
                } else {
                    match self.facts.generics.parameters[&owner][index].argument {
                        Argument::Type(_) => SourceArgument::Type(key),
                        Argument::Const(id) => {
                            let ty =
                                self.facts.types.const_type(id).ok_or_else(|| {
                                    self.error(key, "missing const parameter type")
                                })?;
                            SourceArgument::Const(self.generic_constant(key, ty, owner)?)
                        }
                        Argument::Lifetime(_) => {
                            return Err(self.error(key, "invalid function argument metadata"));
                        }
                    }
                });
            }
        }
        self.facts.generics.function_uses.insert(
            key,
            FunctionUse {
                lifetimes,
                arguments,
            },
        );
        Ok(())
    }
}
