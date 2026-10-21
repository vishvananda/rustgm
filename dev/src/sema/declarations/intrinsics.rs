//! Intrinsic functions publish ordinary sized generic callable signatures.
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
        for kind in Intrinsic::ALL {
            if let Some(declaration) = self.report.resolution.intrinsics.get(kind).copied() {
                self.intrinsic_signature(declaration)?;
            }
        }
        Ok(())
    }
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
        self.states[parameter.0 as usize] = State::Complete;
        self.facts.generics.parameters.insert(
            definition,
            vec![Parameter {
                binding: parameter,
                anonymous: false,
                argument: Argument::Type(subject),
                default: None,
            }],
        );
        self.facts
            .generics
            .complete_environment(&mut self.facts.types, definition)
            .map_err(|e| self.error(origin, e))?;
        Ok((formal, subject))
    }
    fn intrinsic_signature(&mut self, declaration: BuiltinIntrinsic) -> Result<(), Error> {
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
                mutable: kind == Intrinsic::WriteVolatile,
                pointee: subject,
            },
        )?;
        let inputs = [pointer, subject];
        let count = if kind == Intrinsic::ReadVolatile {
            1
        } else {
            2
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
                result: if kind == Intrinsic::ReadVolatile {
                    subject
                } else {
                    TypeId::UNIT
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
        self.states[definition.0 as usize] = State::Complete;
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
}
