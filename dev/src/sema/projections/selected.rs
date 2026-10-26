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
    Ready(TypeId),
}
impl Definition {
    pub fn select(
        facts: &Declarations,
        item: ItemId,
        family: GenericArgs,
        source: Source,
    ) -> Result<Self, &'static str> {
        if let Source::LanguageOperation { descriptor } = source {
            return Ok(Self {
                id: DefId(item.0),
                value: Value::Ready(descriptor.output(facts, item, family)?),
            });
        }
        if let Source::Callable { output } = source {
            if facts.associated.callable_output() != Some(DefId(item.0))
                || family != GenericArgs::EMPTY
            {
                return Err("callable evidence applies only to its Output member");
            }
            return Ok(Self {
                id: DefId(item.0),
                value: Value::Ready(output),
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
        facts: &mut Declarations,
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
        let selected = Self::method(facts, alias, source)?
            .ok_or("trait output requires a selected implementation")?;
        let target = selected.definition();
        let Some((owner, arguments)) = selected.implementation() else {
            return Ok(None);
        };
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

    /// Both result proof consumers and result formation use this same typed
    /// target producer. Selecting it alone grants no inherited body permission.
    pub(in crate::sema) fn method(
        facts: &mut Declarations,
        alias: Alias,
        source: Source,
    ) -> Result<Option<crate::sema::specialization::SelectedMethod>, &'static str> {
        let (Alias::TraitResult(id, _), Source::Implementation { owner, arguments }) =
            (alias, source)
        else {
            return Ok(None);
        };
        let method = facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("missing trait result declaration")?
            .owner;
        crate::sema::specialization::selected_method(facts, owner, arguments, method).map(Some)
    }

    pub fn result(&self, facts: &mut Declarations) -> Result<Option<TypeId>, &'static str> {
        let (owner, arguments, family, result) = match self.value {
            Value::Ready(ty) => return Ok(Some(ty)),
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
