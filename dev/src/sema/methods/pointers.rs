//! Canonical raw-pointer methods retain their impl parameter and full receiver identity.
mod preparation;
use super::{DefId, Methods, NodeKey, TypeId};
use crate::support::hash::HashMap;
use crate::{resolve::names::NameId, types::parameters::ParameterId};
pub(in crate::sema) use preparation::prepare;
#[derive(Clone, Copy)]
pub(crate) struct Offset {
    pub definition: DefId,
    pub parameter: ParameterId,
    pub receiver: TypeId,
}
#[derive(Default)]
pub(super) struct Library {
    methods: HashMap<DefId, Offset>,
    prefixes: HashMap<NodeKey, (NodeKey, DefId)>,
}
impl Methods {
    pub(in crate::sema) fn install_pointer_method(
        &mut self,
        method: Offset,
    ) -> Result<(), &'static str> {
        let library = self
            .pointers
            .get_or_insert_with(|| Box::new(Library::default()));
        if library.methods.insert(method.definition, method).is_some() {
            return Err("pointer method installed twice");
        }
        Ok(())
    }
    pub(in crate::sema) fn pointer_method(&self, definition: DefId) -> Option<Offset> {
        self.pointers.as_deref()?.methods.get(&definition).copied()
    }
    pub(in crate::sema) fn pointer_prefix(&self, key: NodeKey) -> Option<(NodeKey, Offset)> {
        let library = self.pointers.as_deref()?;
        let &(prefix, definition) = library.prefixes.get(&key)?;
        Some((prefix, *library.methods.get(&definition)?))
    }
    pub(in crate::sema) fn retain_pointer_prefix(
        &mut self,
        key: NodeKey,
        prefix: NodeKey,
        definition: DefId,
    ) -> Result<(), &'static str> {
        let library = self
            .pointers
            .as_deref_mut()
            .ok_or("missing pointer method registry")?;
        if !library.methods.contains_key(&definition) {
            return Err("unregistered pointer method prefix");
        }
        library.prefixes.insert(key, (prefix, definition));
        Ok(())
    }
}
pub(in crate::sema) fn named(
    report: &crate::resolve::expansion::Report,
    name: NameId,
    mutable: bool,
) -> Option<DefId> {
    let kind = if mutable {
        crate::support::intrinsics::Intrinsic::MutPointerAdd
    } else {
        crate::support::intrinsics::Intrinsic::ConstPointerAdd
    };
    report
        .resolution
        .intrinsics
        .get(kind)
        .filter(|d| report.resolution.definitions[d.definition.0 as usize].name == name)
        .map(|d| d.definition)
}
