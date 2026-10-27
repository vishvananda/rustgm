//! Both actual original mappings preserve the same raw requirement and effect.
use super::{Graph, Predicate, RegionId, TypeId, s};
use crate::support::crates::generics::TypeRef;
use crate::types::parameters::predicates::Bound;

pub(super) trait Requests {
    fn application(&mut self, reference: s::TraitApplicationRef)
    -> Result<Predicate, &'static str>;
    fn ty(&mut self, reference: TypeRef) -> Result<TypeId, &'static str>;
    fn region(&mut self, reference: s::RegionRef) -> Result<RegionId, &'static str>;
}
pub(super) fn map(
    mapping: &mut impl Requests,
    raw: s::Requirement,
) -> Result<Predicate, &'static str> {
    match raw {
        s::Requirement::Trait {
            application,
            effect,
        } => {
            let mut predicate = mapping.application(application)?;
            let Bound::Trait(application) = predicate.bound else {
                return Err("source request lost its actual complete ordinary application");
            };
            predicate.bound = Bound::trait_application(application, effect);
            Ok(predicate)
        }
        s::Requirement::TypeOutlives { subject, shorter } => Ok(Predicate {
            ty: mapping
                .ty(subject)
                .map_err(|_| "source request has an unknown actual type")?,
            bound: Bound::Outlives(
                mapping
                    .region(shorter)
                    .map_err(|_| "source request has an unknown actual region")?,
            ),
        }),
        s::Requirement::Outlives(_) => {
            Err("standalone region request requires its wider source relation profile")
        }
    }
}
impl Requests for &Graph {
    fn application(
        &mut self,
        reference: s::TraitApplicationRef,
    ) -> Result<Predicate, &'static str> {
        Graph::application(self, reference)
    }
    fn ty(&mut self, reference: TypeRef) -> Result<TypeId, &'static str> {
        Graph::ty(self, reference).map_err(|_| "source request has an unknown actual type")
    }
    fn region(&mut self, reference: s::RegionRef) -> Result<RegionId, &'static str> {
        Graph::region(self, reference).map_err(|_| "source request has an unknown actual region")
    }
}
