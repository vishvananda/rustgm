//! `Fn(...) -> R` is an ordinary equality on the inherited `FnOnce::Output` member.
use super::{
    CallTrait, Checker, Error, GenericArgs, ItemId, Located, NodeKey, Predicate, Type, TypeId,
};
use crate::{
    sema::declarations::generics::bounds::lexical::Scope,
    types::{bound_uses::Uses, parameters::predicates::Bound, projection, traits::Application},
};
impl Checker<'_> {
    pub(in crate::sema::declarations) fn call_output_binding(
        &mut self,
        path: NodeKey,
        predicate: Predicate,
        original: Predicate,
        scope: Option<Scope>,
        output: &mut Vec<Located>,
    ) -> Result<(), Error> {
        let (_, result) = crate::sema::lifetimes::call_arguments(self.report, path)?
            .ok_or_else(|| self.error(path, "missing call output binding"))?;
        let result_source = result.map(|node| NodeKey { node, ..path });
        let result = result_source.map_or(Ok(TypeId::UNIT), |key| self.ty(key))?;
        let original_result = result_source
            .and_then(|key| self.facts.unreduced.node(key))
            .unwrap_or(result);
        let mut inputs = Uses::constraining();
        inputs
            .predicate(&self.facts.types, original)
            .map_err(|e| self.error(path, e))?;
        let mut outputs = Uses::default();
        outputs
            .collect(&self.facts.types, original_result)
            .map_err(|e| self.error(path, e))?;
        if !outputs.indices.is_subset(&inputs.indices) {
            return Err(self.error(
                path,
                "call output lifetime is unconstrained by the trait inputs",
            ));
        }
        let ty = self.call_output_projection(path, predicate)?;
        let original_ty = self.call_output_projection(path, original)?;
        let original = self.bound_clause(
            path,
            scope,
            Predicate {
                ty: original_ty,
                bound: Bound::ProjectionEqual(original_result),
            },
        )?;
        self.facts.unreduced.retain_predicate(path, original);
        output.push(Located {
            source: path,
            value: self.bound_clause(
                path,
                scope,
                Predicate {
                    ty,
                    bound: Bound::ProjectionEqual(result),
                },
            )?,
        });
        Ok(())
    }

    fn call_output_projection(
        &mut self,
        source: NodeKey,
        predicate: Predicate,
    ) -> Result<TypeId, Error> {
        let Some((id, _)) = predicate.bound.trait_parts() else {
            return Err(self.error(source, "call output requires a trait application"));
        };
        let parent = self
            .facts
            .types
            .trait_application(id)
            .ok_or_else(|| self.error(source, "missing call input application"))?;
        if !self.declared_callable(crate::resolve::model::DefId(parent.item.0)) {
            let member = self
                .direct_parenthesis_output(source, crate::resolve::model::DefId(parent.item.0))?;
            return self.intern(
                source,
                Type::Projection(projection::Application {
                    item: ItemId(member.0),
                    subject: predicate.ty,
                    trait_application: id,
                    arguments: GenericArgs::EMPTY,
                }),
            );
        }
        let (definition, member) =
            if let Some(reserved) = self.report.resolution.source_bindings.as_deref() {
                let definition = reserved
                    .language(crate::support::bindings::LangItem::FnOnce)
                    .ok_or_else(|| self.error(source, "missing source FnOnce declaration"))?;
                let member = reserved
                    .language(crate::support::bindings::LangItem::FnOnceOutput)
                    .filter(|m| m.parent == Some(definition.definition))
                    .ok_or_else(|| self.error(source, "missing source FnOnce Output"))?;
                (definition.definition, member.definition)
            } else {
                let declaration = self
                    .report
                    .resolution
                    .builtin_traits
                    .callable(CallTrait::FnOnce)
                    .ok_or_else(|| self.error(source, "missing FnOnce declaration"))?;
                (
                    declaration.definition,
                    declaration
                        .output
                        .ok_or_else(|| self.error(source, "missing FnOnce Output"))?,
                )
            };
        let trait_application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(definition.0),
                arguments: parent.arguments,
            })
            .map_err(|e| self.error(source, e))?;
        self.intern(
            source,
            Type::Projection(projection::Application {
                item: ItemId(member.0),
                subject: predicate.ty,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }),
        )
    }
}
