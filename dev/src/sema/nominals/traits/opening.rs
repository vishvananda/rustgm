//! Body proofs share declaration slots only with their producer's retained aliases.
use super::{Context, Environment, Obligations, RegionId, substitute};
use std::sync::Arc;

pub(super) enum Opening {
    Declaration,
    Body,
}
impl Opening {
    pub(super) fn fold(
        self,
        environment: Option<&Arc<Environment>>,
        regions: Vec<RegionId>,
        context: &mut Context,
        obligations: &mut Obligations,
    ) -> Result<substitute::Bound, &'static str> {
        let (Self::Body, Some(environment)) = (self, environment) else {
            return Ok(substitute::Bound::new(regions));
        };
        if let Some(aliases) = environment.body_aliases() {
            context
                .account_work(aliases.len())
                .map_err(|_| "body alias opening work exhausted")?;
            if aliases.len() != regions.len() {
                return Err("body alias scope changed declaration slot count");
            }
            obligations.equalities.extend(
                regions
                    .iter()
                    .copied()
                    .zip(aliases.ordered().iter().copied()),
            );
        }
        environment.open_body_scope(regions)
    }
}
