//! Canonical caller permissions come from the actual sealed template owner.
use super::{BodyKey, Checked, ItemId};
use crate::{
    body::ir::{Body, Owner},
    types::parameters::environment::Environment,
};
use std::sync::Arc;
impl Checked<'_> {
    pub(crate) fn environment_for_body(
        &self,
        body: &Body,
    ) -> Result<Option<&Arc<Environment>>, &'static str> {
        let key = canonical_key(body)?;
        let index = match key {
            BodyKey::Declaration(definition) => self.state.index.get(&ItemId(definition.0)),
            BodyKey::Closure(identity) => self.state.closures.get(&identity),
            key @ (BodyKey::Anonymous(_) | BodyKey::Imported(_)) => self.state.anonymous.get(&key),
            _ => return Err("caller has no canonical checked template owner"),
        };
        let Some(&index) = index else {
            return Ok(None);
        };
        let template = self
            .templates
            .get(index)
            .ok_or("caller template index is stale")?;
        if template.body.source != body.source || template.body.key() != key {
            return Err("caller template source or canonical owner changed");
        }
        Ok(Some(&template.environment))
    }
}
fn canonical_key(body: &Body) -> Result<BodyKey, &'static str> {
    Ok(match body.owner {
        Owner::InitializerInstance { instance, .. } => BodyKey::Declaration(instance.definition),
        Owner::AnonymousInstance(_) => match body.source {
            crate::body::ir::BodyOrigin::Source(source) => BodyKey::Anonymous(source),
            crate::body::ir::BodyOrigin::Imported(body) => BodyKey::Imported(body.id()),
        },
        Owner::ClosureAdapter { .. } => {
            return Err("adapter caller requires its actual source permission bridge");
        }
        _ => body.key(),
    })
}
