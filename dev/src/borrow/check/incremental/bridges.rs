//! The existing paid factory/adapter origin map moves with its immutable proved bodies.
use crate::{
    body::ir::{Body, Owner},
    support::hash::HashMap,
    types::{Kind, Store},
};

pub(crate) enum Bridge {
    Factory(crate::body::factories::Permit),
    Adapter(usize),
}
pub(crate) type Forwarding = HashMap<usize, Bridge>;
const _: [(); 16] = [(); size_of::<Bridge>()];
const _: [(); 32] = [(); size_of::<Forwarding>()];
const _: [(); 24] = [(); size_of::<(usize, Bridge)>()];
impl Bridge {
    pub(crate) fn caller<'a>(
        &self,
        types: &Store,
        index: usize,
        get: impl Fn(usize) -> Result<&'a Body, &'static str>,
    ) -> Result<Option<&'a Body>, &'static str> {
        let body = get(index)?;
        match self {
            Self::Factory(permit) => {
                if let Some(future) = permit.future() {
                    future.require_current(types, body)?;
                    return Ok(None);
                }
                let instance = permit
                    .instance()
                    .ok_or("ordinary factory lost its actual selected instance")?;
                if body.instance != Some(instance)
                    || !matches!((body.owner, types.kind(instance.ty())),
                        (Owner::Function { definition, signature, .. }, Some(Kind::FunctionItem { item, signature: actual, .. }))
                        if definition.0 == item.0 && signature == actual)
                {
                    return Err("factory caller changed its proved selected instance");
                }
                // A compiler factory owns no lexical feature premises. Its
                // checked diagnostic anchor cannot confer source permissions.
                Ok(None)
            }
            Self::Adapter(source) => {
                if *source >= index {
                    return Err("adapter caller has no earlier proved source body");
                }
                let original = get(*source)?;
                let Owner::ClosureAdapter { adapter, .. } = body.owner else {
                    return Err("adapter caller changed its proved wrapper owner");
                };
                if body.instance != Some(adapter.instance())
                    || body.source != original.source
                    || !matches!((original.owner, types.kind(adapter.instance().ty())),
                        (Owner::Closure { identity, .. }, Some(Kind::Closure { identity: selected, .. }))
                        if identity == selected)
                {
                    return Err("adapter caller changed its actual proved closure source");
                }
                Ok(Some(original))
            }
        }
    }
}
