//! Retained provider roles encode optional identities without widening their public data.
use crate::{
    resolve::{
        bindings::{Declaration, Reservations},
        model::DefId,
        unit::NodeKey,
    },
    support::bindings::LangItem,
};
use std::num::NonZeroU32;

#[derive(Clone, Copy)]
struct Cell {
    definition: NonZeroU32,
    parent: Option<NonZeroU32>,
    origin: NodeKey,
}
pub(in crate::sema::compiler_items) struct Provider {
    roles: [Option<Cell>; LangItem::ALL.len()],
}
impl Provider {
    pub(in crate::sema::compiler_items) fn new(
        reserved: &Reservations,
    ) -> Result<Self, &'static str> {
        let mut roles = [None; LangItem::ALL.len()];
        for kind in LangItem::ALL {
            roles[kind.index()] = reserved.language(kind).map(Cell::encode).transpose()?;
        }
        Ok(Self { roles })
    }
    pub(in crate::sema::compiler_items) fn get(&self, kind: LangItem) -> Option<Declaration> {
        self.roles[kind.index()].map(Cell::decode)
    }
    #[cfg(test)]
    pub(in crate::sema::compiler_items) fn replace(
        &mut self,
        kind: LangItem,
        declaration: Option<Declaration>,
    ) -> Result<(), &'static str> {
        self.roles[kind.index()] = declaration.map(Cell::encode).transpose()?;
        Ok(())
    }
}
fn identity(id: DefId) -> Result<NonZeroU32, &'static str> {
    id.0.checked_add(1)
        .and_then(NonZeroU32::new)
        .ok_or("source role identity exceeds compact declaration storage")
}
impl Cell {
    fn encode(declaration: Declaration) -> Result<Self, &'static str> {
        Ok(Self {
            definition: identity(declaration.definition)?,
            parent: declaration.parent.map(identity).transpose()?,
            origin: declaration.origin,
        })
    }
    fn decode(self) -> Declaration {
        Declaration {
            definition: DefId(self.definition.get() - 1),
            parent: self.parent.map(|id| DefId(id.get() - 1)),
            origin: self.origin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_claim_cells_preserve_zero_parent_origin_and_checked_identity_limits()
    -> Result<(), &'static str> {
        let origin = NodeKey {
            file: crate::resolve::unit::FileId(17),
            node: crate::syntax::ast::NodeId::new(31),
        };
        for (definition, parent) in [
            (0, None),
            (0, Some(0)),
            (7, Some(2)),
            (u32::MAX - 1, Some(u32::MAX - 1)),
        ] {
            let declaration = Declaration {
                definition: DefId(definition),
                parent: parent.map(DefId),
                origin,
            };
            assert_eq!(Cell::encode(declaration)?.decode(), declaration);
        }
        for (definition, parent) in [(u32::MAX, None), (0, Some(u32::MAX))] {
            assert!(
                Cell::encode(Declaration {
                    definition: DefId(definition),
                    parent: parent.map(DefId),
                    origin
                })
                .is_err()
            );
        }
        assert_eq!(size_of::<Option<Cell>>(), 16);
        assert_eq!(size_of::<Provider>(), 16 * LangItem::ALL.len());
        Ok(())
    }
}
