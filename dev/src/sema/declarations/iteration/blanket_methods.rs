//! Library impl methods retain the ordinary flattened source and callable binders.
use super::{Checker, Error, ItemId, Type};
use crate::{
    resolve::model::{BuiltinIteration, BuiltinIterationBlanket},
    support::{iteration::Blanket, library_types::LibraryType},
    types::{GenericArgs, RegionId, TypeId, arguments::Argument},
};
impl Checker<'_> {
    pub(super) fn iteration_blanket_method(
        &mut self,
        canonical: BuiltinIteration,
        declaration: BuiltinIterationBlanket,
        _parameter: TypeId,
        subject: TypeId,
        item: TypeId,
    ) -> Result<TypeId, Error> {
        let source = canonical.origin;
        let reference = declaration.kind == Blanket::MutableReference;
        let receiver = if reference {
            self.intern(
                source,
                Type::Reference {
                    referent: subject,
                    mutable: true,
                    region: RegionId::bound(0, 1).map_err(|e| self.error(source, e))?,
                },
            )?
        } else {
            subject
        };
        let result = if reference {
            let option = self
                .report
                .resolution
                .library_types
                .get(LibraryType::Option)
                .ok_or_else(|| self.error(source, "missing canonical iteration Option"))?
                .definition;
            let arguments = self
                .facts
                .types
                .intern_arguments(&[Argument::Type(item)])
                .map_err(|e| self.error(source, e))?;
            self.intern(source, Type::Adt(ItemId(option.0), arguments))?
        } else {
            subject
        };
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
                bound_regions: if reference { 2 } else { 0 },
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
        super::super::comparisons::LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .publish_library_method(
            super::super::comparison_methods::MethodContract {
                origin: source,
                owner: declaration.definition,
                self_parameter: declaration.parameter,
                definition: declaration.method,
                signature,
                output: None,
            },
            callable,
            receiver,
            if reference { 2 } else { 0 },
        )?;
        self.facts
            .associated
            .library_impl_member(declaration.definition, declaration.method)
            .map_err(|e| self.error(source, e))?;
        let scheme = crate::sema::lifetimes::FunctionRegions {
            total: if reference { 2 } else { 0 },
            captures: u32::from(reference),
            early: Vec::new(),
            late: if reference { vec![1] } else { Vec::new() },
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
