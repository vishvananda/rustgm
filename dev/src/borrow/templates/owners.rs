//! Original-owner validation is shared without exposing proof-context lookup entries.
use super::{BodyKey, ClosureId, Error, HashMap, ItemId, Owner, Templates};
pub(super) struct Indices {
    pub functions: HashMap<ItemId, usize>,
    pub closures: HashMap<ClosureId, usize>,
    pub anonymous: HashMap<BodyKey, usize>,
}
pub(super) fn indices(templates: &Templates) -> Result<Indices, Error> {
    let mut index = HashMap::with_capacity_and_hasher(
        templates.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    let mut closures = HashMap::default();
    let mut anonymous = HashMap::default();
    for (ordinal, template) in templates.iter().enumerate() {
        let body = &template.body;
        let error = |message: &str| Error {
            node: Some(body.source.into()),
            message: message.into(),
        };
        let duplicate = match body.owner {
            Owner::Function { definition, .. } | Owner::Initializer { definition, .. } => {
                index.insert(ItemId(definition.0), ordinal).is_some()
            }
            Owner::Closure { identity, .. } => closures.insert(identity, ordinal).is_some(),
            Owner::AnonymousConstant => anonymous.insert(body.key(), ordinal).is_some(),
            Owner::InitializerInstance { .. }
            | Owner::AnonymousInstance(_)
            | Owner::ClosureAdapter { .. } => {
                return Err(error("template requires a declared owner"));
            }
        };
        if body.instance.is_some() || duplicate {
            return Err(error("template proof requires distinct original owners"));
        }
        if body.promotions.is_none()
            && body
                .values
                .iter()
                .any(|v| matches!(v.kind, crate::body::ir::Rvalue::Borrow { .. }))
        {
            return Err(error(
                "template proof requires completed promotion analysis",
            ));
        }
    }
    Ok(Indices {
        functions: index,
        closures,
        anonymous,
    })
}
