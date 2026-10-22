//! Each collection entry uses the same checked callable publication as source impls.
use super::{
    Argument, BuiltinCollectionDeclaration, BuiltinIteration, Error, GenericArgs, ItemId, RegionId,
    Type, TypeId,
};
impl super::super::super::comparisons::LibraryContracts<'_> {
    pub(super) fn collection_method(
        &mut self,
        iteration: BuiltinIteration,
        declaration: BuiltinCollectionDeclaration,
        subject: TypeId,
        item: TypeId,
        iterator: TypeId,
    ) -> Result<TypeId, Error> {
        let source = iteration.origin;
        let next = declaration.kind.next();
        let receiver = if declaration.kind.next() || declaration.kind.method() {
            self.intern(
                source,
                Type::Reference {
                    referent: subject,
                    mutable: next || declaration.kind.mutable(),
                    region: RegionId::bound(0, u16::from(next))
                        .map_err(|e| self.error(source, e))?,
                },
            )?
        } else {
            subject
        };
        let result = if next {
            let arguments = self
                .facts
                .types
                .intern_arguments(&[Argument::Type(item)])
                .map_err(|e| self.error(source, e))?;
            self.intern(source, Type::Adt(ItemId(iteration.option.0), arguments))?
        } else {
            iterator
        };
        let total = if next { 2 } else { 1 };
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
                bound_regions: total,
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
        super::super::super::comparisons::LibraryContracts {
            report: self.report,
            facts: self.facts,
        }
        .publish_library_method(
            super::super::super::comparison_methods::MethodContract {
                origin: source,
                owner: declaration.definition,
                self_parameter: declaration.parameter,
                definition: declaration.method,
                signature,
                output: None,
            },
            callable,
            receiver,
            total,
        )?;
        self.facts
            .associated
            .library_impl_member(declaration.definition, declaration.method)
            .map_err(|e| self.error(source, e))?;
        let scheme = crate::sema::lifetimes::FunctionRegions {
            total,
            captures: u32::from(declaration.lifetime.is_some()),
            early: Vec::new(),
            late: if next {
                vec![1]
            } else if declaration.kind.method() {
                vec![0]
            } else {
                Vec::new()
            },
            has_declared_late: false,
            bounds: Vec::new(),
        };
        self.callable_template(source, declaration.method, signature, &scheme)?;
        self.facts
            .lifetimes
            .functions
            .insert(declaration.method, scheme);
        Ok(signature)
    }
}
