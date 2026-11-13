//! Full selected identities share one cold source scope, without constructor impersonation.
use super::{Body, BodyKey, BodyOrigin, Contract, Declarations, Instance, Original, TypeId};
use crate::body::ir::Owner;

#[derive(Clone, Copy)]
pub(super) enum Selected {
    Callable(Instance),
    AsyncBody(TypeId),
}
impl Selected {
    pub(super) fn original(
        self,
        facts: &Declarations,
        body: &Body,
        contract: &Contract,
        aliases: bool,
    ) -> Result<(), &'static str> {
        let definition = match (self, body.owner) {
            (Self::Callable(_), _) if aliases => return Ok(()),
            (
                Self::Callable(_),
                Owner::Function {
                    definition,
                    signature,
                    execution: None,
                },
            ) => {
                let header = facts
                    .function_header(definition)?
                    .ok_or("instance trait scope lost its original function header")?;
                if header.signature != signature || header.universals != contract.universals {
                    return Err(
                        "instance trait scope changed its original signature or universals",
                    );
                }
                definition
            }
            (Self::AsyncBody(future), Owner::AsyncBody { .. }) => {
                let definition = crate::body::coroutines::original_contract(facts, body, contract)?;
                if facts.checked_future_definition(future)? != definition {
                    return Err("selected async scope belongs to another actual defining function");
                }
                definition
            }
            _ => return Err("trait instance scope requires its actual original owner"),
        };
        if facts
            .function_header(definition)?
            .is_none_or(|header| header.body != Some(body.source))
        {
            return Err("instance source scope changed its actual function body");
        }
        Ok(())
    }
    pub(super) const fn key(self, original: BodyKey) -> BodyKey {
        match self {
            Self::Callable(_) => original,
            Self::AsyncBody(future) => BodyKey::AsyncApplication(future),
        }
    }
    pub(super) fn body(
        self,
        facts: &Declarations,
        original: &Original,
        body: &Body,
        contract: &Contract,
        signatures: (TypeId, TypeId),
    ) -> Result<(), &'static str> {
        let key = self.key(original.key);
        if body.key() != key
            || body.source != original.source
            || contract.key != key
            || contract.source != original.source
            || contract.signature != Some(signatures.1)
        {
            return Err("materialization changed its authenticated original identity");
        }
        match self {
            Self::Callable(instance) => {
                callable_body(facts, original, body, instance, signatures)?;
            }
            Self::AsyncBody(future) => {
                let definition = facts.checked_future_definition(future)?;
                let Some(crate::types::Kind::Future { identity, .. }) = facts.types.kind(future)
                else {
                    return Err("selected async scope lost its genuine future application");
                };
                if original.key != BodyKey::AsyncBody(identity)
                    || body.instance.is_some()
                    || signatures.0 != signatures.1
                    || !matches!(body.owner, Owner::AsyncApplication { identity: owner, future: actual, signature } if owner == identity && actual == future && signature == signatures.1)
                    || facts
                        .function_header(definition)?
                        .is_none_or(|header| header.body != Some(original.source))
                {
                    return Err(
                        "selected async scope changed its actual owner, capture frame or signature",
                    );
                }
            }
        }
        Ok(())
    }
}

/// The selected public identity and the specialized body interface are distinct.
/// Both function and closure instances remain bound to their exact original.
fn callable_body(
    facts: &Declarations,
    original: &Original,
    body: &Body,
    instance: Instance,
    signatures: (TypeId, TypeId),
) -> Result<(), &'static str> {
    let (key, signature, owned) = match (facts.types.kind(instance.ty()), body.owner) {
        (
            Some(crate::types::Kind::FunctionItem {
                item, signature, ..
            }),
            Owner::Function {
                definition,
                signature: public,
                execution: Some(execution),
            },
        ) if definition.0 == item.0 => (
            BodyKey::Declaration(definition),
            signature,
            (public, execution),
        ),
        (
            Some(crate::types::Kind::Closure {
                identity,
                signature,
                ..
            }),
            Owner::Closure {
                identity: selected,
                signature: public,
                execution,
            },
        ) if identity == selected => {
            let source = facts
                .closures
                .identity(identity)
                .ok_or("instance scope lost its original closure")?;
            if BodyOrigin::from(source.semantic_source()) != original.source {
                return Err("instance scope changed its original closure source");
            }
            (
                BodyKey::Closure(identity),
                signature,
                (public, execution.unwrap_or(public)),
            )
        }
        _ => return Err("alias materialization requires its actual callable owner"),
    };
    if original.key != key
        || signature != signatures.0
        || owned != signatures
        || body.instance != Some(instance)
    {
        return Err("alias materialization changed its selected public signature");
    }
    Ok(())
}

pub(super) fn owner(
    facts: &Declarations,
    key: BodyKey,
    source: BodyOrigin,
) -> Result<crate::resolve::model::DefId, &'static str> {
    let definition = match key {
        BodyKey::Declaration(definition) => definition,
        BodyKey::AsyncBody(identity) => {
            let declaration = facts
                .opaques
                .declarations
                .get(&identity)
                .ok_or("async source scope lost its actual declaration")?;
            let future = declaration
                .hidden
                .ok_or("async source scope lost its checked publication")?;
            let definition = facts.checked_future_definition(future)?;
            if declaration.owner != definition {
                return Err("async source scope changed its actual owner");
            }
            definition
        }
        _ => return Err("nonalias trait scope requires its actual function owner"),
    };
    if facts
        .function_header(definition)?
        .is_none_or(|header| header.body != Some(source))
    {
        return Err("instance source scope changed its actual original header");
    }
    Ok(definition)
}
