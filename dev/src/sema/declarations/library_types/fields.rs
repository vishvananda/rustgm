//! Canonical nominal fields retain exact type, name and visibility schema.
use super::{Argument, Error, LibraryField, LibraryNominal, LibraryType, Type, TypeId};
use crate::{
    resolve::model::Access,
    sema::members::{Field, Name},
};
impl super::super::comparisons::LibraryContracts<'_> {
    pub(super) fn library_member_fields(
        &mut self,
        declaration: &LibraryNominal,
        field_types: &[TypeId],
    ) -> Result<Vec<Field>, Error> {
        if declaration.kind == LibraryType::Range {
            return self.range_fields(declaration, field_types);
        }
        let name = if declaration.kind.is_struct() {
            self.facts
                .members
                .names
                .intern("value")
                .map(Name::Named)
                .map_err(|e| self.error(declaration.origin, e))?
        } else {
            Name::Index(0)
        };
        let fields = field_types
            .iter()
            .copied()
            .map(|ty| Field {
                origin: crate::sema::members::Origin::Library(declaration.origin),
                name,
                ty,
                access: if declaration.kind.is_struct() {
                    Access::ExternalPrivate
                } else {
                    Access::Public
                },
            })
            .collect::<Vec<_>>();
        Ok(fields)
    }
    fn range_fields(
        &mut self,
        declaration: &LibraryNominal,
        field_types: &[TypeId],
    ) -> Result<Vec<Field>, Error> {
        let [start, end] = field_types else {
            return Err(self.error(declaration.origin, "invalid Range field count"));
        };
        if start != end {
            return Err(self.error(declaration.origin, "Range bounds differ"));
        }
        ["start", "end"]
            .into_iter()
            .map(|name| {
                let name = self
                    .facts
                    .members
                    .names
                    .intern(name)
                    .map(Name::Named)
                    .map_err(|e| self.error(declaration.origin, e))?;
                Ok(Field {
                    origin: crate::sema::members::Origin::Library(declaration.origin),
                    name,
                    ty: *start,
                    access: Access::Public,
                })
            })
            .collect()
    }
    pub(super) fn library_field(
        &mut self,
        declaration: &LibraryNominal,
        types: &[Argument],
        field: Option<LibraryField>,
    ) -> Result<Option<TypeId>, Error> {
        field
            .map(|field| match field {
                LibraryField::RangeBounds => Err(self.error(
                    declaration.origin,
                    "Range fields require complete record expansion",
                )),
                LibraryField::U32 => Ok(TypeId::primitive(crate::types::Primitive::U32)),
                LibraryField::Parameter(index) => match types.get(index as usize) {
                    Some(Argument::Type(ty)) => Ok(*ty),
                    _ => Err(self.error(declaration.origin, "invalid library field parameter")),
                },
                LibraryField::SharedSlice | LibraryField::MutableSlice => {
                    let [Argument::Lifetime(region), Argument::Type(element)] = types else {
                        return Err(
                            self.error(declaration.origin, "invalid shared slice field schema")
                        );
                    };
                    if declaration.kind
                        != if field == LibraryField::MutableSlice {
                            LibraryType::SliceIterMut
                        } else {
                            LibraryType::SliceIter
                        }
                    {
                        return Err(self.error(
                            declaration.origin,
                            "shared slice field requires canonical Iter",
                        ));
                    }
                    let slice = self.intern(declaration.origin, Type::Slice(*element))?;
                    self.intern(
                        declaration.origin,
                        Type::Reference {
                            mutable: field == LibraryField::MutableSlice,
                            region: *region,
                            referent: slice,
                        },
                    )
                }
            })
            .transpose()
    }
}
