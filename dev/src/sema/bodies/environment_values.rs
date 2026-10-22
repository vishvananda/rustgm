//! Builtin library values select canonical aggregates without resolving invented paths.
use super::{Checker, Error, NodeId, Operation, Primitive, TermId, Type, TypeId};
use crate::{
    support::library_types::{LibraryType, LibraryValue},
    types::{ItemId, RegionId, arguments::Argument},
};
impl Checker<'_> {
    pub(super) fn library_value(
        &mut self,
        node: NodeId,
        kind: LibraryValue,
        value: Option<NodeId>,
    ) -> Result<TermId, Error> {
        match kind {
            LibraryValue::EnvironmentOption => self.environment_option(node, value),
        }
    }
    fn environment_option(&mut self, node: NodeId, value: Option<NodeId>) -> Result<TermId, Error> {
        let declaration = self
            .report
            .resolution
            .library_types
            .get(LibraryType::Option)
            .ok_or_else(|| self.error(node, "missing Option library identity"))?;
        let definition = declaration.definition;
        let constructor = declaration
            .variants
            .get(usize::from(value.is_some()))
            .ok_or_else(|| self.error(node, "missing Option library variant"))?
            .constructor
            .ok_or_else(|| self.error(node, "Option library variant requires a constructor"))?;
        let variant = self
            .facts
            .members
            .constructor(constructor)
            .ok_or_else(|| self.error(node, "missing Option library member"))?;
        if self.facts.members.variants[variant.0 as usize].owner != ItemId(definition.0) {
            return Err(self.error(node, "Option constructor belongs to another declaration"));
        }
        let string = self
            .facts
            .types
            .intern(Type::Reference {
                mutable: false,
                region: RegionId::STATIC,
                referent: TypeId::primitive(Primitive::Str),
            })
            .map_err(|e| self.error(node, e))?;
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Type(string)])
            .map_err(|e| self.error(node, e))?;
        let ty = self
            .facts
            .types
            .intern(Type::Adt(ItemId(definition.0), arguments))
            .map_err(|e| self.error(node, e))?;
        self.facts
            .aggregates
            .prepare(&mut self.facts.types, ty)
            .map_err(|e| self.error(node, e))?;
        if let Some(value) = value {
            let expected = self.known(node, string)?;
            self.coerce(value, expected)?;
        }
        self.selected(node, Operation::Construct { variant })?;
        self.known(node, ty)
    }
}
