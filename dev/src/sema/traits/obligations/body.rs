//! Body proofs share the body's allocator and later directed lifetime validation.
use super::Obligations;
use crate::{
    resolve::unit::NodeKey,
    sema::model::{
        BodyRegion, BodyRegionOrigin, Declarations, Outlives, RegionEquality, TypeOutlives,
    },
};

impl Obligations {
    pub(crate) fn retain_body(
        self,
        facts: &mut Declarations,
        body: NodeKey,
        source: NodeKey,
    ) -> Result<(), &'static str> {
        if usize::try_from(self.regions.start).ok() != facts.body_regions.len().checked_add(1) {
            return Err("trait proof has foreign body region identities");
        }
        let region = BodyRegion {
            body,
            source,
            origin: BodyRegionOrigin::Inference,
        };
        facts.body_regions.extend(self.regions.map(|_| region));
        facts.region_equalities.extend(
            self.equalities
                .into_iter()
                .map(|(left, right)| RegionEquality { body, left, right }),
        );
        facts
            .outlives
            .extend(self.outlives.into_iter().map(|(longer, shorter)| Outlives {
                body,
                source,
                longer,
                shorter,
            }));
        facts
            .type_outlives
            .extend(self.types.into_iter().map(|(ty, shorter)| TypeOutlives {
                body,
                source,
                ty,
                shorter,
            }));
        Ok(())
    }
}
