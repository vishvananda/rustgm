//! Elaboration of lifetime-only nominal and alias parameters and path arguments.
use super::{Checker, DefId, Error, ItemId, NodeKey, RegionId, TypeId, regions};
use crate::types::GenericArgs;

impl Checker<'_> {
    pub(super) fn local_context(
        &self,
        key: NodeKey,
        inherited: regions::Context,
    ) -> regions::Context {
        let super::Kind::Item(index) = self.kind(key) else {
            return inherited;
        };
        let item = self.report.package.unit(key.file).configured.ast.items[index as usize];
        let (body, binder) = match item.kind {
            crate::syntax::item::ItemKind::Function {
                body: Some(body), ..
            } => (body, self.function_binders.get(&key).copied()),
            crate::syntax::item::ItemKind::Const {
                value: Some(body), ..
            }
            | crate::syntax::item::ItemKind::Static {
                value: Some(body), ..
            } => (body, None),
            _ => return inherited,
        };
        regions::Context {
            binder,
            position: regions::Position::Local(body),
        }
    }
    pub(super) fn lifetime_variances(&mut self) -> Result<(), Error> {
        let parameters = self
            .facts
            .aggregates
            .into_iter()
            .map(|definition| {
                let ty = self.facts.definitions[definition.item.0 as usize]
                    .ok_or("missing nominal variance identity")?;
                let Some(crate::types::Kind::Adt(item, arguments)) = self.facts.types.kind(ty)
                else {
                    return Err("invalid nominal variance identity");
                };
                Ok((item, arguments))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        self.facts.lifetimes.variances = crate::types::variance::Variances::infer(
            &self.facts.types,
            &self.facts.aggregates,
            &parameters,
        )
        .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        for (item, _) in parameters {
            if self
                .facts
                .lifetimes
                .variances
                .get(item)
                .is_some_and(|values| values.contains(&crate::types::variance::Variance::Bivariant))
            {
                let key = self.report.resolution.definitions[item.0 as usize]
                    .node
                    .ok_or_else(|| {
                        Error::at(
                            crate::resolve::unit::FileId(0),
                            0,
                            "missing lifetime declaration source",
                        )
                    })?;
                return Err(self.error(
                    key,
                    "nominal lifetime parameter is unused or only used recursively",
                ));
            }
        }
        Ok(())
    }
    pub(super) fn template_context(
        &mut self,
        id: DefId,
        key: NodeKey,
    ) -> Result<regions::Context, Error> {
        let item = self.item(key)?;
        if item.generics.start == item.generics.end
            && item.where_clause.start == item.where_clause.end
        {
            return Ok(regions::Context::ALIAS);
        }
        let declared = self.declared_regions(key, item.generics)?;
        let binder = self.regions.binder(None, declared.clone());
        // Nominal/alias parameter indices follow declaration order, including
        // unused alias parameters. Function elision retains its own ordering.
        for parameter in &declared {
            self.regions
                .named(Some(binder), *parameter)
                .map_err(|e| self.error(key, e))?;
        }
        let bounds = self.function_bounds(key, binder)?;
        if matches!(
            item.kind,
            crate::syntax::item::ItemKind::Struct { .. }
                | crate::syntax::item::ItemKind::Enum { .. }
        ) {
            self.facts
                .lifetimes
                .nominal_bounds
                .extend(bounds.into_iter().map(|bound| (ItemId(id.0), bound)));
        }
        // Rust parses/resolves alias bounds but does not enforce them at uses.
        // Requirements of the expanded underlying type still apply normally.
        self.facts.lifetimes.parameters.insert(id, declared);
        Ok(regions::Context {
            binder: Some(binder),
            position: regions::Position::Output,
        })
    }

    pub(super) fn template_arguments(
        &mut self,
        id: DefId,
        key: NodeKey,
    ) -> Result<GenericArgs, Error> {
        if let Some(parameters) = self.facts.generics.parameters.get(&id) {
            let arguments: Vec<_> = parameters.iter().map(|p| p.argument).collect();
            return self
                .facts
                .types
                .intern_arguments(&arguments)
                .map_err(|e| self.error(key, e));
        }
        let count = self.facts.lifetimes.parameters.get(&id).map_or(0, Vec::len);
        if count == 0 {
            return Ok(GenericArgs::EMPTY);
        }
        let arguments = (0..count)
            .map(|index| {
                RegionId::bound(
                    0,
                    u16::try_from(index).map_err(|_| "too many lifetime parameters")?,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| self.error(key, e))?;
        self.facts
            .types
            .intern_regions(&arguments)
            .map_err(|e| self.error(key, e))
    }

    pub(super) fn path_type(
        &mut self,
        key: NodeKey,
        id: DefId,
        context: regions::Context,
    ) -> Result<TypeId, Error> {
        if self.facts.destructors.self_type(id).is_some() {
            return self.drop_self_type(key, id, context);
        }
        if self.facts.generics.parameters.contains_key(&id) {
            return self.generic_path(key, id, context);
        }
        let ty = self.facts.definitions[id.0 as usize]
            .ok_or_else(|| self.error(key, "missing resolved declaration type"))?;
        let arguments = crate::sema::lifetimes::arguments(self.report, key, false)?;
        let count = self.facts.lifetimes.parameters.get(&id).map_or(0, Vec::len);
        let mut regions = Vec::with_capacity(count);
        if let Some(arguments) = arguments {
            let ast = &self.report.package.unit(key.file).configured.ast;
            let arguments = ast.list(arguments);
            if arguments.len() != count {
                return Err(self.error(key, "incorrect number of lifetime arguments"));
            }
            for argument in arguments {
                let region = self.region(key, Some(*argument), context)?;
                self.regions.record(context, region);
                regions.push(region);
            }
        } else {
            if count != 0 && matches!(context.position, regions::Position::Impl) {
                return Err(self.error(key, "impl target requires explicit lifetime arguments"));
            }
            for _ in 0..count {
                let region = self.region(key, None, context)?;
                self.regions.record(context, region);
                regions.push(region);
            }
        }
        if regions.is_empty() {
            return Ok(ty);
        }
        let args = self
            .facts
            .types
            .intern_regions(&regions)
            .map_err(|e| self.error(key, e))?;
        self.facts
            .lifetimes
            .substitute(&mut self.facts.types, args, ty)
            .map_err(|e| self.error(key, e))
    }

    pub(super) fn nominal_context(&self, item: ItemId) -> regions::Context {
        self.nominal_regions
            .get(&item)
            .copied()
            .unwrap_or(regions::Context::ALIAS)
    }
}
