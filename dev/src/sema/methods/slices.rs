//! The registered slice method retains exact identity and typed prefix demand.
mod preparation;
use super::{DefId, Methods, NodeKey, TypeId};
use crate::support::hash::HashMap;
use crate::{
    resolve::names::NameId,
    sema::model::{Declarations, Function},
    types::{
        ItemId, Kind, RegionId,
        parameters::{ParameterId, ParameterKind},
    },
};
pub(in crate::sema) use preparation::prepare;
#[derive(Clone, Copy)]
pub(crate) struct Length {
    pub owner: DefId,
    pub definition: DefId,
    pub origin: NodeKey,
    pub parameter: ParameterId,
    pub signature: TypeId,
    pub receiver: TypeId,
}
pub(super) struct Library {
    length: Length,
    name: NameId,
    demanded: bool,
    prefixes: HashMap<NodeKey, NodeKey>,
}
impl Methods {
    pub(in crate::sema) fn install_length(
        &mut self,
        length: Length,
        name: NameId,
    ) -> Result<(), &'static str> {
        if self.library.is_some() {
            return Err("slice method metadata installed twice");
        }
        self.library = Some(Box::new(Library {
            length,
            name,
            demanded: false,
            prefixes: HashMap::default(),
        }));
        Ok(())
    }
    pub(crate) fn length_named(&self, name: NameId) -> Option<Length> {
        self.library
            .as_deref()
            .filter(|l| l.name == name)
            .map(|l| l.length)
    }
    pub(crate) fn length_definition(&self, definition: DefId) -> Option<Length> {
        self.library
            .as_deref()
            .filter(|l| l.length.definition == definition)
            .map(|l| l.length)
    }
    pub(crate) fn demanded_length(&self) -> Option<Length> {
        self.library
            .as_deref()
            .filter(|l| l.demanded)
            .map(|l| l.length)
    }
    pub(in crate::sema) fn demand_library_method(&mut self, definition: DefId) {
        if let Some(library) = self
            .library
            .as_deref_mut()
            .filter(|l| l.length.definition == definition)
        {
            library.demanded = true;
        }
    }
    pub(in crate::sema) fn length_prefix(&self, key: NodeKey) -> Option<NodeKey> {
        self.library.as_deref()?.prefixes.get(&key).copied()
    }
    pub(in crate::sema) fn retain_length_prefix(
        &mut self,
        key: NodeKey,
        prefix: NodeKey,
    ) -> Result<(), &'static str> {
        let library = self
            .library
            .as_deref_mut()
            .ok_or("missing slice method registry")?;
        library.demanded = true;
        library.prefixes.insert(key, prefix);
        Ok(())
    }
}
impl Length {
    pub(crate) fn validate(
        self,
        facts: &Declarations,
        function: &Function,
    ) -> Result<(), &'static str> {
        if function.definition != self.definition
            || function.signature != self.signature
            || function.body.is_some()
        {
            return Err("slice length original differs from registered declaration");
        }
        let formal = facts
            .types
            .parameter(self.parameter)
            .ok_or("missing slice element parameter")?;
        if formal.owner != ItemId(self.owner.0)
            || formal.index != 0
            || formal.kind != (ParameterKind::Type { sized: true })
        {
            return Err("slice element parameter differs from canonical impl");
        }
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
            bound_regions: 1,
        }) = facts.types.kind(self.signature)
        else {
            return Err("slice length has invalid callable contract");
        };
        if facts.types.list(parameters) != Some(&[self.receiver])
            || result != TypeId::primitive(crate::types::Primitive::Usize)
        {
            return Err("slice length has invalid parameters/result");
        }
        let Some(Kind::Reference {
            referent,
            mutable: false,
            region,
        }) = facts.types.kind(self.receiver)
        else {
            return Err("slice length receiver is not shared");
        };
        let Some(Kind::Slice(element)) = facts.types.kind(referent) else {
            return Err("slice length receiver is not a slice");
        };
        if region != RegionId::bound(0, 0)?
            || facts.types.kind(element) != Some(Kind::Parameter(self.parameter))
        {
            return Err("slice length has invalid binder/element identity");
        }
        let callable = facts
            .lifetimes
            .callables
            .get(ItemId(self.definition.0))
            .ok_or("missing slice length template")?;
        if callable.signature != self.signature
            || callable.parameters != 0
            || facts
                .lifetimes
                .callables
                .generic_parameters(ItemId(self.definition.0))
                != Some(&[self.parameter])
        {
            return Err("slice length callable template differs");
        }
        Ok(())
    }
}

pub(super) fn applicable(
    facts: &mut Declarations,
    context: &mut crate::types::infer::Context,
    actual: crate::types::infer::TermId,
    method: DefId,
) -> Result<bool, &'static str> {
    let length = facts
        .methods
        .length_definition(method)
        .ok_or("unregistered slice candidate")?;
    let argument = context
        .parameter_variable(&facts.types, length.parameter)
        .map_err(super::selection::inference)?;
    let mut bindings = crate::types::infer::Instantiation::new(
        context,
        &facts.types,
        &[(length.parameter, argument)],
    )
    .map_err(super::selection::inference)?;
    let expected = context
        .instantiate_with(&mut facts.types, length.receiver, &mut bindings)
        .map_err(super::selection::inference)?;
    match context.equate_shape(&facts.types, actual, expected) {
        Ok(()) => Ok(true),
        Err(crate::types::infer::Error::Mismatch | crate::types::infer::Error::Occurs) => Ok(false),
        Err(error) => Err(super::selection::inference(error)),
    }
}

pub(super) fn receiver_permission(
    facts: &Declarations,
    context: &mut crate::types::infer::Context,
    actual: crate::types::infer::TermId,
    mutable: bool,
) -> Result<bool, &'static str> {
    use crate::types::infer::Constructor;
    if !matches!(
        context
            .head(&facts.types, actual)
            .map_err(super::selection::inference)?,
        Some(Constructor::Reference { mutable: actual, .. }) if actual == mutable
    ) {
        return Ok(false);
    }
    let referent = context
        .child(&facts.types, actual, 0)
        .map_err(super::selection::inference)?;
    Ok(matches!(
        context
            .head(&facts.types, referent)
            .map_err(super::selection::inference)?,
        Some(Constructor::Slice)
    ))
}
