//! Shared registration for ordinary source-free library method declarations.
use super::{Checker, DefId, Error, ItemId, NodeKey, State, Type};
use crate::{
    syntax::ast::List,
    types::{GenericArgs, TypeId, arguments::Argument},
};
#[derive(Clone, Copy)]
pub(super) struct MethodContract {
    pub origin: NodeKey,
    pub owner: DefId,
    pub self_parameter: DefId,
    pub definition: DefId,
    pub signature: TypeId,
    pub output: Option<DefId>,
}
impl Checker<'_> {
    pub(super) fn register_library_method(
        &mut self,
        method: MethodContract,
        subject: TypeId,
        inputs: &[Argument],
    ) -> Result<(), Error> {
        super::comparisons::LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .register_library_method(method, subject, inputs)?;
        self.states[method.definition.0 as usize] = State::Complete;
        Ok(())
    }
}
impl super::comparisons::LibraryContracts<'_> {
    pub(super) fn register_library_method(
        &mut self,
        method: MethodContract,
        subject: TypeId,
        inputs: &[Argument],
    ) -> Result<(), Error> {
        let MethodContract {
            origin,
            owner,
            self_parameter,
            definition,
            signature,
            output: _,
        } = method;
        let Type::Function {
            parameters,
            bound_regions,
            ..
        } = self
            .facts
            .types
            .kind(signature)
            .ok_or_else(|| self.error(origin, "missing library method signature"))?
        else {
            return Err(self.error(origin, "library method is not callable"));
        };
        let receiver = *self
            .facts
            .types
            .list(parameters)
            .and_then(|p| p.first())
            .ok_or_else(|| self.error(origin, "library method has no receiver"))?;
        let callable = self.intern(
            origin,
            Type::FunctionItem {
                item: ItemId(definition.0),
                signature,
                arguments: GenericArgs::EMPTY,
            },
        )?;
        let generic_parameters = std::iter::once(subject)
            .chain(inputs.iter().filter_map(|arg| {
                if let Argument::Type(ty) = arg {
                    Some(*ty)
                } else {
                    None
                }
            }))
            .map(|ty| crate::sema::generics::parameter_id(&self.facts.types, Argument::Type(ty)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| self.error(origin, e))?;
        self.facts
            .lifetimes
            .callables
            .insert_scoped(
                &self.facts.types,
                &[
                    ItemId(self_parameter.0),
                    ItemId(owner.0),
                    ItemId(definition.0),
                ],
                signature,
                0,
                &[],
                &generic_parameters,
            )
            .map_err(|e| self.error(origin, e))?;
        self.publish_library_method(method, callable, receiver, bound_regions)
    }
    fn publish_library_method(
        &mut self,
        method: MethodContract,
        callable: TypeId,
        receiver: TypeId,
        bound_regions: u32,
    ) -> Result<(), Error> {
        let MethodContract {
            origin,
            owner,
            definition,
            signature,
            output,
            ..
        } = method;
        self.facts
            .methods
            .trait_names
            .insert(self.report.resolution.definitions[definition.0 as usize].name);
        self.facts.definitions[definition.0 as usize] = Some(callable);
        self.facts
            .generics
            .parameters
            .insert(definition, Vec::new());
        self.facts.generics.parents.insert(definition, owner);
        let mut referent = receiver;
        let mut references = 0u32;
        while let Some(Type::Reference {
            referent: inner, ..
        }) = self.facts.types.kind(referent)
        {
            references = references
                .checked_add(1)
                .ok_or_else(|| self.error(origin, "too many receiver references"))?;
            referent = inner;
        }
        self.facts
            .methods
            .receivers
            .insert(definition, (receiver, references));
        self.facts.lifetimes.functions.insert(
            definition,
            crate::sema::lifetimes::FunctionRegions {
                total: bound_regions,
                captures: 0,
                early: Vec::new(),
                late: (0..u16::try_from(bound_regions)
                    .map_err(|_| self.error(origin, "too many library method lifetimes"))?)
                    .collect(),
                has_declared_late: false,
                bounds: Vec::new(),
            },
        );
        self.facts.associated.register_language_members(
            owner,
            origin,
            crate::sema::model::Function {
                definition,
                node: origin,
                signature,
                parameters: List { start: 0, end: 0 },
                body: None,
            },
            output.map(|output| {
                (
                    output,
                    self.report.resolution.definitions[output.0 as usize].name,
                )
            }),
        );
        Ok(())
    }
}
