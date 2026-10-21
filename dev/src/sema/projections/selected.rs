//! A selected member keeps its owner and complete arguments while its definition finishes.
use super::{Declarations, DefId, ItemId, Source, TypeId};
use crate::types::{GenericArgs, opaque::OpaqueId, substitute::Alias};

pub(in crate::sema) struct Definition {
    pub id: DefId,
    value: Value,
}
enum Value {
    Implementation {
        owner: DefId,
        arguments: GenericArgs,
        family: GenericArgs,
        result: Option<OpaqueId>,
    },
    Callable(TypeId),
}
impl Definition {
    pub fn select(
        facts: &Declarations,
        item: ItemId,
        family: GenericArgs,
        source: Source,
    ) -> Result<Self, &'static str> {
        if let Source::Callable { output } = source {
            if facts.associated.callable_output() != Some(DefId(item.0))
                || family != GenericArgs::EMPTY
            {
                return Err("callable evidence applies only to its Output member");
            }
            return Ok(Self {
                id: DefId(item.0),
                value: Value::Callable(output),
            });
        }
        let Source::Implementation { owner, arguments } = source else {
            return Err("associated type requires an unambiguous validated implementation");
        };
        Ok(Self {
            id: super::definition(facts, owner, item)?,
            value: Value::Implementation {
                owner,
                arguments,
                family,
                result: None,
            },
        })
    }

    pub fn alias(
        facts: &Declarations,
        alias: Alias,
        family: GenericArgs,
        source: Source,
    ) -> Result<Option<Self>, &'static str> {
        let Alias::TraitResult(id, _) = alias else {
            return Self::select(
                facts,
                alias.associated().ok_or("missing associated alias")?.item,
                family,
                source,
            )
            .map(Some);
        };
        let Source::Implementation { owner, arguments } = source else {
            return Err("trait output requires a selected implementation");
        };
        let method = facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("missing trait result declaration")?
            .owner;
        let target = facts
            .generics
            .implementations
            .as_ref()
            .and_then(|registry| registry.method(owner, method))
            .ok_or("missing selected trait result method")?;
        if target == method {
            return Ok(None);
        }
        Ok(Some(Self {
            id: target,
            value: Value::Implementation {
                owner,
                arguments,
                family,
                result: Some(id),
            },
        }))
    }

    pub fn result(&self, facts: &mut Declarations) -> Result<Option<TypeId>, &'static str> {
        let (owner, arguments, family, result) = match self.value {
            Value::Callable(ty) => return Ok(Some(ty)),
            Value::Implementation {
                owner,
                arguments,
                family,
                result,
            } => (owner, arguments, family, result),
        };
        if let Some(id) = result {
            return crate::sema::opaque::overrides::result(facts, owner, arguments, id, family);
        }
        facts.definitions[self.id.0 as usize]
            .map(|template| {
                facts.instantiate_associated_definition(owner, arguments, self.id, family, template)
            })
            .transpose()
    }
}
