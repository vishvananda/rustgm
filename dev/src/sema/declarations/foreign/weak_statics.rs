//! Authenticate the exact foreign declaration and the type of the weak symbol.
use super::{
    Abi, Checker, Convention, DefId, Error, ItemKind, NodeKey, TypeId, callable_convention,
};
use crate::{
    resolve::{model::DefKind, package::Error as SourceError},
    sema::foreign::WeakStatic,
    support::bindings::LangItem,
    types::{Kind, arguments::Argument},
};

impl Checker<'_> {
    pub(super) fn weak_static(
        &self,
        definition: DefId,
        key: NodeKey,
        ty: TypeId,
        mutable: bool,
    ) -> Result<Option<WeakStatic>, Error> {
        let Some(offset) = self
            .linkage_sources
            .get(&key)
            .and_then(|attributes| attributes.extern_weak)
        else {
            return Ok(None);
        };
        let error = |message| SourceError::at(key.file, offset, message);
        let resolution = &self.report.resolution;
        let actual = resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| error("weak static lost its actual declaration"))?;
        let &block = resolution
            .foreign_owners
            .get(&definition)
            .ok_or_else(|| error("extern_weak requires its actual foreign static owner"))?;
        if actual.node != Some(key)
            || actual.kind != DefKind::Static
            || key.file != block.file
            || !resolution.foreign_blocks.contains(&block)
        {
            return Err(error("extern_weak changed its actual foreign static owner"));
        }
        let ItemKind::ExternBlock { abi, .. } = self.item(block)?.kind else {
            return Err(error("extern_weak requires an actual external block"));
        };
        if callable_convention(self.report, block, Abi::Extern(abi))? != Convention::C {
            return Err(error("extern_weak currently requires a C foreign static"));
        }
        if mutable {
            return Err(error("extern mutable statics are not allowed with linkage"));
        }
        let marker = match self.facts.types.kind(ty) {
            Some(Kind::Pointer { .. }) => WeakStatic::Data,
            Some(Kind::Adt(item, args))
                if self
                    .facts
                    .source_bindings
                    .as_deref()
                    .and_then(|catalog| catalog.language(LangItem::Option))
                    .is_some_and(|role| role.definition.0 == item.0) =>
            {
                let mut arguments = self
                    .facts
                    .types
                    .arguments(args)
                    .ok_or_else(|| error("weak static Option lost its actual type argument"))?;
                let Some(Argument::Type(signature)) = arguments.next() else {
                    return Err(error("weak static Option requires a C function pointer"));
                };
                if arguments.next().is_some()
                    || !matches!(
                        self.facts.types.kind(signature),
                        Some(Kind::Function {
                            convention: Convention::C,
                            ..
                        })
                    )
                {
                    return Err(error("weak static Option requires a C function pointer"));
                }
                WeakStatic::Function(signature)
            }
            _ => {
                return Err(error(
                    "extern_weak requires a raw pointer or the actual Option of a C function pointer",
                ));
            }
        };
        Ok(Some(marker))
    }
}
