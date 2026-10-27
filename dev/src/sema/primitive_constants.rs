//! Primitive constants are typed language values, not source initializers or methods.
use super::model::Declarations;
use crate::support::hash::HashMap;
use crate::{
    resolve::unit::NodeKey,
    types::{
        Kind, Primitive, Store, TypeId,
        arguments::{ConstId, Constant},
    },
};
#[cfg(test)]
#[path = "../../tests/invariants/integer_limits.rs"]
mod tests;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema) enum Member {
    Max,
    Min,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Descriptor {
    primitive: Primitive,
    member: Member,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Selected {
    prefix: NodeKey,
    descriptor: Descriptor,
    value: ConstId,
}
#[derive(Default)]
pub(crate) struct Uses {
    selected: HashMap<NodeKey, Selected>,
}
impl Descriptor {
    fn value(self) -> Result<Constant, &'static str> {
        let bits = if self.primitive == Primitive::Char {
            match self.member {
                Member::Min => 0,
                Member::Max => 0x10_FFFF,
            }
        } else {
            let (signed, width) = self
                .primitive
                .integer()
                .filter(|_| self.member == Member::Max)
                .ok_or("primitive constant requires supported integer MAX or char MIN/MAX")?;
            let bits = u128::MAX >> (128 - width);
            if signed { bits >> 1 } else { bits }
        };
        Ok(Constant {
            ty: TypeId::primitive(self.primitive),
            bits,
        })
    }
}
impl Uses {
    pub(in crate::sema) fn publish(
        &mut self,
        types: &mut Store,
        key: NodeKey,
        prefix: NodeKey,
        ty: TypeId,
        member: Member,
    ) -> Result<(), &'static str> {
        let Some(Kind::Primitive(primitive)) = types.kind(ty) else {
            return Err("primitive constant has a nonprimitive prefix");
        };
        let descriptor = Descriptor { primitive, member };
        let constant = descriptor.value()?;
        if let Some(previous) = self.selected.get(&key) {
            return if previous.prefix == prefix
                && previous.descriptor == descriptor
                && types.constant(previous.value) == Some(constant)
            {
                Ok(())
            } else {
                Err("primitive constant selection changed")
            };
        }
        if self.selected.len() >= 1_000_000 {
            return Err("primitive constant use budget exceeded");
        }
        let value = types.intern_constant(constant)?;
        let selected = Selected {
            prefix,
            descriptor,
            value,
        };
        self.selected.insert(key, selected);
        Ok(())
    }
}
pub(crate) fn selected(
    facts: &Declarations,
    key: NodeKey,
) -> Result<Option<ConstId>, &'static str> {
    let Some(use_) = facts
        .primitive_constants
        .as_deref()
        .and_then(|uses| uses.selected.get(&key))
    else {
        return Ok(None);
    };
    let ty = facts
        .node(use_.prefix)
        .and_then(|fact| fact.ty)
        .ok_or("primitive constant lacks its typed prefix")?;
    if ty != TypeId::primitive(use_.descriptor.primitive)
        || facts.types.constant(use_.value) != Some(use_.descriptor.value()?)
    {
        return Err("primitive constant differs from its selected prefix or value");
    }
    Ok(Some(use_.value))
}
