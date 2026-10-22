//! A range method keeps its single independent public receiver binder.
use super::{Argument, BuiltinRangeImplementation, DefId, Error, GenericArgs, ItemId, Type};
use crate::{
    resolve::unit::NodeKey,
    types::{RegionId, TypeId},
};
impl super::super::super::comparisons::LibraryContracts<'_> {
    pub(super) fn range_method(
        &mut self,
        source: NodeKey,
        declaration: BuiltinRangeImplementation,
        subject: TypeId,
        element: TypeId,
        option: DefId,
    ) -> Result<TypeId, Error> {
        let receiver = self.intern(
            source,
            Type::Reference {
                referent: subject,
                mutable: true,
                region: RegionId::bound(0, 0).map_err(|e| self.error(source, e))?,
            },
        )?;
        let args = self
            .facts
            .types
            .intern_arguments(&[Argument::Type(element)])
            .map_err(|e| self.error(source, e))?;
        let result = self.intern(source, Type::Adt(ItemId(option.0), args))?;
        let parameters = self
            .facts
            .types
            .intern_list(&[receiver])
            .map_err(|e| self.error(source, e))?;
        let signature = self.intern(
            source,
            Type::Function {
                parameters,
                result,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
                bound_regions: 1,
            },
        )?;
        let callable = self.intern(
            source,
            Type::FunctionItem {
                item: ItemId(declaration.method.0),
                signature,
                arguments: GenericArgs::EMPTY,
            },
        )?;
        self.publish_library_method(
            super::super::super::comparison_methods::MethodContract {
                origin: source,
                owner: declaration.definition,
                self_parameter: declaration.definition,
                definition: declaration.method,
                signature,
                output: None,
            },
            callable,
            receiver,
            1,
        )?;
        self.facts
            .associated
            .library_impl_member(declaration.definition, declaration.method)
            .map_err(|e| self.error(source, e))?;
        self.facts
            .lifetimes
            .callables
            .insert_monomorphic(&self.facts.types, ItemId(declaration.method.0), signature)
            .map_err(|e| self.error(source, e))?;
        Ok(signature)
    }
}
