//! Concrete source originals finish the exact owned plan across semantic selection.
use super::{Body, Environment, Error, Proved, Verifier, glue, regions};
use crate::{
    borrow::{check::cstage, loans::seal::Seal},
    types::substitute::Bound,
};
use std::{ptr::NonNull, sync::Arc};
pub(crate) type ConstOriginal<'a> = cstage::Prepared<'a, 'static>;
pub(crate) struct Completed {
    body: NonNull<Body>,
    plan: Option<Box<crate::borrow::destruction::Plan>>,
    targets: Option<Box<Bound>>,
}
/// The callback only borrows this owned immutable body. A completed result from
/// another allocation cannot be attached, and no mutable CFG view crosses it.
pub(crate) fn with_const_original(
    body: Body,
    check: impl FnOnce(&Body) -> Result<Completed, Error>,
) -> Result<Proved, Error> {
    let completed = check(&body)?;
    if completed.body != NonNull::from(&body) {
        return Err(Error {
            node: Some(body.source.into()),
            message: "completed const cleanup belongs to another owned body".into(),
        });
    }
    Ok(Proved {
        body,
        plan: completed.plan,
        destruction: true,
        static_targets: completed.targets,
    })
}
impl Verifier {
    pub(crate) fn prepare_const_original<'a, S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut Environment<'_, S>,
        body: &'a Body,
        contract: &regions::Body,
        catalog: &mut glue::Catalog<'_>,
    ) -> Result<ConstOriginal<'a>, Error> {
        let error = |message: &str| Error {
            node: Some(body.source.into()),
            message: message.into(),
        };
        Seal::require_arenas(environment).map_err(error)?;
        if !body.const_context
            || body.instance.is_some()
            || body.source.source().is_none()
            || !matches!(
                body.owner,
                crate::body::ir::Owner::Function {
                    execution: None,
                    ..
                }
            )
            || contract.key != body.key()
            || contract.source != body.source
            || contract.instance.is_some()
            || self.drops.is_none()
        {
            return Err(error(
                "const cleanup requires its actual ordinary source original",
            ));
        }
        super::super::require_body_promotions(body)?;
        let owner = self.const_owner.get_or_insert_with(|| Arc::new(()));
        cstage::prepare(
            environment,
            &mut self.reused,
            body,
            contract,
            catalog,
            super::super::structure::Use::Own,
            Some(Arc::clone(owner)),
        )
    }
    pub(crate) fn finish_const_original<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut Environment<'_, S>,
        prepared: ConstOriginal<'_>,
        contract: &regions::Body,
        rechecked: bool,
    ) -> Result<Completed, Error> {
        let body = NonNull::from(prepared.body());
        let error = |message: &str| Error {
            node: Some(prepared.body().source.into()),
            message: message.into(),
        };
        let targets = super::super::static_targets::prepare(contract).map_err(error)?;
        let drops = self
            .drops
            .as_mut()
            .ok_or_else(|| error("const cleanup lost its actual destructor environment"))?;
        let plan = cstage::finish(
            environment,
            &mut self.reused,
            drops,
            prepared,
            contract,
            self.const_owner.as_ref(),
            if rechecked {
                cstage::RegionUse::Reproved
            } else {
                cstage::RegionUse::Initial
            },
        )?;
        Ok(Completed {
            body,
            plan: plan.map(Box::new),
            targets,
        })
    }
}
