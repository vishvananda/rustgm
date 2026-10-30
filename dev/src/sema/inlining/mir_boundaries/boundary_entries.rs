//! One touched declaration owns only its present transient annotations.
use super::{
    Declarations, Error, Growth, NodeKey, OriginalImage, Restriction, Span,
    reserve_source_contracts,
};
use std::sync::Arc;
#[derive(Clone, Copy)]
pub(super) struct Annotation {
    pub restriction: Restriction,
    pub span: Span,
}
pub(super) struct Entry {
    pub image: Arc<OriginalImage>,
    first: Annotation,
    #[expect(
        clippy::box_collection,
        reason = "only mixed declarations allocate this cold additional-annotation owner"
    )]
    extra: Option<Box<Vec<Annotation>>>,
    pub bound: bool,
}
impl Entry {
    pub fn new(
        image: Arc<OriginalImage>,
        restriction: Restriction,
        span: Span,
        bound: bool,
    ) -> Self {
        Self {
            image,
            first: Annotation { restriction, span },
            extra: None,
            bound,
        }
    }
    pub fn annotation_count(&self) -> usize {
        1 + self.extra.as_deref().map_or(0, Vec::len)
    }
    pub fn annotations(&self) -> impl Iterator<Item = &Annotation> {
        std::iter::once(&self.first).chain(self.extra.iter().flat_map(|rows| rows.iter()))
    }
    pub fn is_nominal(&self) -> bool {
        self.first.restriction == Restriction::Suspend
    }
    pub fn append(
        &mut self,
        facts: &mut Declarations,
        key: NodeKey,
        restriction: Restriction,
        span: Span,
    ) -> Result<(), Error> {
        let error = |message: &str| Error::at(key.file, span.start, message);
        let (len, capacity) = self
            .extra
            .as_deref()
            .map_or((0, 0), |rows| (rows.len(), rows.capacity()));
        let growth = Growth::vector::<Annotation>(len, capacity, 1);
        let owner = if self.extra.is_none() {
            size_of::<Vec<Annotation>>()
        } else {
            0
        };
        reserve_source_contracts(facts, 1 + growth.work, owner.saturating_add(growth.bytes))?;
        if let Some(rows) = self.extra.as_deref_mut() {
            rows.try_reserve(1)
                .map_err(|_| error("additional annotation allocation failed"))?;
            rows.push(Annotation { restriction, span });
        } else {
            let mut rows = Vec::new();
            rows.try_reserve(1)
                .map_err(|_| error("additional annotation allocation failed"))?;
            rows.push(Annotation { restriction, span });
            self.extra = Some(Box::new(rows));
        }
        self.bound = false;
        Ok(())
    }
}
