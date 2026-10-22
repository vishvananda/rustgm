//! Slice len is one canonical const method with an inherited element parameter.
use super::Length;
use crate::resolve::model::LibrarySliceLength;
use crate::{
    resolve::{expansion::Report, names::NameId},
    sema::model::Declarations,
    types::{ItemId, Kind as Type},
};
use crate::{
    sema::{generics::Parameter, lifetimes::FunctionRegions, model::Function},
    syntax::ast::List,
    types::{
        GenericArgs, RegionId, TypeId,
        arguments::Argument,
        callable::Convention,
        parameters::{Parameter as Formal, ParameterKind},
    },
};
struct Preparation<'a> {
    facts: &'a mut Declarations,
    declaration: LibrarySliceLength,
    name: NameId,
}
pub(in crate::sema) fn prepare(
    report: &Report,
    facts: &mut Declarations,
) -> Result<Length, &'static str> {
    let declaration = report
        .resolution
        .library_types
        .slice_length()
        .copied()
        .ok_or("missing canonical slice method identity")?;
    if let Some(length) = facts.methods.length_definition(declaration.definition) {
        return Ok(length);
    }
    let name = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or("invalid slice method definition")?
        .name;
    Preparation {
        facts,
        declaration,
        name,
    }
    .run()
}
impl Preparation<'_> {
    fn run(&mut self) -> Result<Length, &'static str> {
        let declaration = self.declaration;
        let origin = declaration.origin;
        let parameter = self.facts.types.intern_parameter(Formal {
            owner: ItemId(declaration.owner.0),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let element = self.facts.types.intern(Type::Parameter(parameter))?;
        let slice = self.facts.types.intern(Type::Slice(element))?;
        let receiver = self.facts.types.intern(Type::Reference {
            referent: slice,
            mutable: false,
            region: RegionId::bound(0, 0)?,
        })?;
        let parameters = self.facts.types.intern_list(&[receiver])?;
        let signature = self.facts.types.intern(Type::Function {
            parameters,
            result: TypeId::primitive(crate::types::Primitive::Usize),
            unsafe_: false,
            convention: Convention::RUST,
            bound_regions: 1,
        })?;
        let length = Length {
            owner: declaration.owner,
            definition: declaration.definition,
            origin,
            parameter,
            signature,
            receiver,
        };
        self.slice_method_environment(declaration, slice, element)?;
        self.publish_slice_method(declaration, length)?;
        Ok(length)
    }
    fn slice_method_environment(
        &mut self,
        declaration: LibrarySliceLength,
        slice: TypeId,
        element: TypeId,
    ) -> Result<(), &'static str> {
        for (definition, ty) in [(declaration.owner, slice), (declaration.parameter, element)] {
            self.facts.definitions[definition.0 as usize] = Some(ty);
        }
        self.facts.generics.parameters.insert(
            declaration.owner,
            vec![Parameter {
                binding: declaration.parameter,
                anonymous: false,
                argument: Argument::Type(element),
                default: None,
            }],
        );
        self.facts
            .generics
            .parameters
            .insert(declaration.definition, Vec::new());
        self.facts
            .generics
            .parents
            .insert(declaration.definition, declaration.owner);
        for owner in [declaration.owner, declaration.definition] {
            self.facts
                .generics
                .complete_environment(&mut self.facts.types, owner)?;
        }
        Ok(())
    }
    fn publish_slice_method(
        &mut self,
        declaration: LibrarySliceLength,
        length: Length,
    ) -> Result<(), &'static str> {
        let Length {
            origin,
            parameter,
            signature,
            receiver,
            ..
        } = length;
        self.facts.lifetimes.callables.insert_scoped(
            &self.facts.types,
            &[
                ItemId(declaration.owner.0),
                ItemId(declaration.definition.0),
            ],
            signature,
            0,
            &[],
            &[parameter],
        )?;
        self.facts.lifetimes.functions.insert(
            declaration.definition,
            FunctionRegions {
                total: 1,
                captures: 0,
                early: Vec::new(),
                late: vec![0],
                has_declared_late: false,
                bounds: Vec::new(),
            },
        );
        let callable = self.facts.types.intern(Type::FunctionItem {
            item: ItemId(declaration.definition.0),
            signature,
            arguments: GenericArgs::EMPTY,
        })?;
        self.facts.definitions[declaration.definition.0 as usize] = Some(callable);
        self.facts
            .methods
            .receivers
            .insert(declaration.definition, (receiver, 1));
        let function = Function {
            definition: declaration.definition,
            node: origin,
            signature,
            parameters: List { start: 0, end: 0 },
            body: None,
        };
        length.validate(self.facts, &function)?;
        self.facts.methods.install_length(length, self.name)?;
        self.facts
            .function_indices
            .insert(declaration.definition, self.facts.functions.len());
        self.facts.functions.push(function);
        self.facts.constant_functions.insert(declaration.definition);
        self.facts
            .typed_function_bodies
            .insert(declaration.definition);
        Ok(())
    }
}
