//! Concrete integer range impl identities are reserved after all source identities.
use super::{Context, DefId, DefKind, Error, Resolver, Traits};
use crate::{
    resolve::{package::ModuleId, unit::NodeKey},
    support::library_types::LibraryType,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeImplementation {
    pub primitive: DefId,
    pub definition: DefId,
    pub item: DefId,
    pub method: DefId,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Ranges {
    pub origin: NodeKey,
    pub nominal: DefId,
    pub implementations: [RangeImplementation; 12],
}
impl Traits {
    pub(in crate::resolve) fn note_range(&mut self, module: ModuleId, source: NodeKey) {
        self.note_iteration(module, source);
        self.iteration
            .get_or_insert_default()
            .range_origin
            .get_or_insert((module, source));
    }
    #[must_use]
    pub fn ranges(&self) -> Option<&Ranges> {
        self.iteration.as_deref()?.ranges.as_deref()
    }
}
impl Resolver<'_> {
    pub(in crate::resolve) fn reserve_ranges(&mut self) -> Result<(), Error> {
        let Some((module, origin)) = self
            .facts
            .builtin_traits
            .iteration
            .as_deref()
            .and_then(|r| r.range_origin)
        else {
            return Ok(());
        };
        if self.facts.builtin_traits.ranges().is_some() {
            return Ok(());
        }
        let c = Context {
            module,
            file: origin.file,
            scope: self.facts.module_scopes[module.0 as usize],
            foreign: None,
        };
        self.prepare_library_type(c, origin.node, LibraryType::Range)?;
        let nominal = self
            .facts
            .library_types
            .get(LibraryType::Range)
            .ok_or_else(|| self.error(c, origin.node, "missing canonical Range"))?
            .definition;
        let mut entries = Vec::new();
        for spelling in [
            "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        ] {
            let name = self
                .package
                .names
                .intern(spelling)
                .map_err(|e| self.error(c, origin.node, e))?;
            let primitive =
                *self.facts.primitives.get(&name).ok_or_else(|| {
                    self.error(c, origin.node, "missing integer primitive identity")
                })?;
            let scope = self.new_scope(None, ModuleId(0), true)?;
            let local = Context { scope, ..c };
            let definition =
                self.builtin_definition(local, origin.node, "Self", DefKind::SelfType, None)?;
            let item =
                self.builtin_definition(local, origin.node, "Item", DefKind::TypeAlias, None)?;
            let method =
                self.builtin_definition(local, origin.node, "next", DefKind::Function, None)?;
            entries.push(RangeImplementation {
                primitive,
                definition,
                item,
                method,
            });
        }
        let implementations = entries
            .try_into()
            .map_err(|_| self.error(c, origin.node, "invalid integer range owner count"))?;
        self.facts
            .builtin_traits
            .iteration
            .get_or_insert_default()
            .ranges = Some(Box::new(Ranges {
            origin,
            nominal,
            implementations,
        }));
        Ok(())
    }
}
