//! Intrinsic functions publish ordinary sized generic callable signatures.
mod atomic;
use super::{Checker, Error, ItemId, State, Type};
use crate::{
    resolve::model::BuiltinIntrinsic,
    sema::{generics::Parameter, model::Function},
    support::intrinsics::{Intrinsic, Safety},
    syntax::ast::List,
    types::{
        GenericArgs, TypeId,
        arguments::Argument,
        callable::Convention,
        parameters::{Parameter as Formal, ParameterId, ParameterKind},
    },
};
impl Checker<'_> {
    pub(super) fn prepare_intrinsics(&mut self) -> Result<(), Error> {
        for kind in Intrinsic::BOOTSTRAP {
            let latent_read = kind == Intrinsic::Read
                && self
                    .report
                    .resolution
                    .builtin_traits
                    .collections()
                    .and_then(|c| c.mutable.as_deref())
                    .is_some_and(|m| m.read_latent);
            if kind.is_pointer_offset() || latent_read {
                continue;
            }
            if let Some(declaration) = self.report.resolution.intrinsics.get(kind).copied() {
                self.intrinsic_signature(declaration)?;
            }
        }
        Ok(())
    }
    fn intrinsic_signature(&mut self, declaration: BuiltinIntrinsic) -> Result<(), Error> {
        if declaration.kind.is_atomic() {
            return self.atomic_signature(declaration);
        }
        super::comparisons::LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .intrinsic_signature(declaration)?;
        self.states[declaration.definition.0 as usize] = State::Complete;
        if let Some(parameter) = declaration.parameter {
            self.states[parameter.0 as usize] = State::Complete;
        }
        Ok(())
    }
}
impl super::comparisons::LibraryContracts<'_> {
    fn intrinsic_parameter(
        &mut self,
        declaration: BuiltinIntrinsic,
    ) -> Result<(ParameterId, TypeId), Error> {
        let BuiltinIntrinsic {
            origin,
            definition,
            parameter,
            ..
        } = declaration;
        let parameter =
            parameter.ok_or_else(|| self.error(origin, "missing intrinsic generic parameter"))?;
        let owner = ItemId(definition.0);
        let formal = self
            .facts
            .types
            .intern_parameter(Formal {
                owner,
                index: 0,
                kind: ParameterKind::Type { sized: true },
            })
            .map_err(|e| self.error(origin, e))?;
        let subject = self.intern(origin, Type::Parameter(formal))?;
        self.facts.definitions[parameter.0 as usize] = Some(subject);
        self.facts.generics.parameters.insert(
            definition,
            vec![Parameter {
                binding: parameter,
                anonymous: false,
                argument: Argument::Type(subject),
                default: None,
            }],
        );
        self.complete_source_environment(origin, definition)?;
        Ok((formal, subject))
    }
    pub(in crate::sema::declarations) fn intrinsic_signature(
        &mut self,
        declaration: BuiltinIntrinsic,
    ) -> Result<(), Error> {
        self.validate_pointer_intrinsic(declaration)?;
        if self
            .facts
            .function_indices
            .contains_key(&declaration.definition)
        {
            return Ok(());
        }
        let (formal, subject) = self.intrinsic_parameter(declaration)?;
        let BuiltinIntrinsic {
            kind,
            origin,
            definition,
            ..
        } = declaration;
        let owner = ItemId(definition.0);
        let pointer = self.intern(
            origin,
            Type::Pointer {
                mutable: kind.writes_pointer(),
                pointee: subject,
            },
        )?;
        let mut inputs = [
            pointer,
            subject,
            TypeId::primitive(crate::types::Primitive::Usize),
        ];
        let count = if kind.is_fill() {
            inputs[1] = TypeId::primitive(crate::types::Primitive::U8);
            3
        } else if kind.is_counted_memory() {
            inputs[1] = self.intern(
                origin,
                Type::Pointer {
                    mutable: true,
                    pointee: subject,
                },
            )?;
            3
        } else if kind.writes_pointer() {
            2
        } else {
            1
        };
        let parameters = self
            .facts
            .types
            .intern_list(&inputs[..count])
            .map_err(|e| self.error(origin, e))?;
        let signature = self.intern(
            origin,
            Type::Function {
                bound_regions: 0,
                parameters,
                result: if kind.writes_pointer() || kind.is_counted_memory() {
                    TypeId::UNIT
                } else {
                    subject
                },
                unsafe_: kind.descriptor().safety == Safety::Unsafe,
                convention: Convention::RUST,
            },
        )?;
        self.facts
            .lifetimes
            .callables
            .insert_generic(&self.facts.types, owner, signature, 0, &[], &[formal])
            .map_err(|e| self.error(origin, e))?;
        let callable = self.intern(
            origin,
            Type::FunctionItem {
                item: owner,
                arguments: GenericArgs::EMPTY,
                signature,
            },
        )?;
        self.facts.definitions[definition.0 as usize] = Some(callable);
        self.facts
            .function_indices
            .insert(definition, self.facts.functions.len());
        self.facts.functions.push(Function {
            definition,
            node: origin,
            signature,
            parameters: List { start: 0, end: 0 },
            body: None,
        });
        Ok(())
    }
    fn validate_pointer_intrinsic(&self, declaration: BuiltinIntrinsic) -> Result<(), Error> {
        let registered = self.report.resolution.intrinsics.get(declaration.kind);
        if registered.is_none_or(|r| {
            r.definition != declaration.definition
                || r.parameter != declaration.parameter
                || r.owner != declaration.owner
        }) || declaration.kind.is_atomic()
            || declaration.kind.is_pointer_offset()
        {
            return Err(self.error(
                declaration.origin,
                "unregistered pointer intrinsic signature",
            ));
        }
        Ok(())
    }
}
