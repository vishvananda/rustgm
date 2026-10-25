//! A borrowed report authenticates complete headers once, never captured callers.
use super::{Error, Location, Report, permission_frames::FrameView};
use crate::{
    expand::marks::ExpansionId,
    resolve::{
        bindings::{
            FeatureName,
            panic_contracts::{self, Frame, HeaderData},
        },
        unit::{NodeKey, contexts},
    },
    support::{hash::HashMap, names::NameId},
};
use std::{rc::Rc, sync::Arc};
#[derive(Clone)]
struct CachedHeader {
    data: Option<Rc<HeaderData>>,
    image: Arc<crate::support::source::OriginalImage>,
}

#[derive(Default)]
pub(super) struct Memo {
    frames: HashMap<ExpansionId, Frame>,
    headers: HashMap<NodeKey, CachedHeader>,
    origin_links: usize,
    origin_lookups: usize,
}

#[cfg(test)]
#[path = "../../../tests/invariants/allowance_queries.rs"]
mod tests;
impl Memo {
    fn payload(&self) -> usize {
        size_of::<Self>()
            .saturating_add(
                self.frames
                    .capacity()
                    .saturating_mul(size_of::<(ExpansionId, Frame)>()),
            )
            .saturating_add(
                self.headers
                    .capacity()
                    .saturating_mul(size_of::<(NodeKey, CachedHeader)>()),
            )
    }
}

/// Read-only provenance facts over one immutable report; no API use is authorized.
pub struct MacroAllowanceSession<'report> {
    report: &'report Report,
    memo: Option<Box<Memo>>,
}
impl Report {
    #[must_use]
    pub fn macro_allowances(&self) -> MacroAllowanceSession<'_> {
        MacroAllowanceSession {
            report: self,
            memo: None,
        }
    }
    /// Prepare a checked canonical feature symbol from this report's name arena.
    /// # Errors
    /// Rejects unknown IDs or names that are not valid feature identifiers.
    pub fn feature_name(&self, id: NameId) -> Result<FeatureName<'_>, Error> {
        if id.0 as usize >= self.package.names.len() {
            return Err(Error::unlocated("unknown canonical feature name"));
        }
        FeatureName::new(self.package.names.get(id)).map_err(Error::unlocated)
    }
}
impl MacroAllowanceSession<'_> {
    fn query(&mut self) -> Query<'_, '_> {
        Query {
            view: FrameView {
                package: &self.report.package,
                definitions: &self.report.definitions,
                invocations: &self.report.invocations,
                facts: &self.report.resolution,
            },
            memo: &mut self.memo,
        }
    }
    /// Use the same borrowed checker as expansion, retaining this report's memo.
    /// # Errors
    /// Rejects changed identities, invalid provenance and exhausted work/storage.
    pub fn allows(&mut self, site: Location, feature: &FeatureName<'_>) -> Result<bool, Error> {
        self.query().allows(site, feature)
    }
    /// Authenticate an actual retained token span through the resolver boundary.
    /// # Errors
    /// Rejects the same source, owner, frame and budget failures as `allows`.
    pub fn allows_at(
        &mut self,
        file: super::FileId,
        span: crate::syntax::token_tree::Span,
        feature: &FeatureName<'_>,
    ) -> Result<bool, Error> {
        self.allows(Location { file, span }, feature)
    }
    /// Inspect only the actual token's outer unsafe-code lint metadata.
    /// This does not authorize an unsafe operation or enable a crate feature.
    /// # Errors
    /// Rejects changed owners/frames/witnesses and exhausted global budgets.
    pub fn allows_unsafe_code_lint(&mut self, site: Location) -> Result<bool, Error> {
        self.query().allows_unsafe_code_lint(site)
    }
}

pub(super) struct Query<'view, 'memo> {
    pub view: FrameView<'view>,
    pub memo: &'memo mut Option<Box<Memo>>,
}
impl Query<'_, '_> {
    fn view(&self) -> FrameView<'_> {
        FrameView {
            package: self.view.package,
            definitions: self.view.definitions,
            invocations: self.view.invocations,
            facts: self.view.facts,
        }
    }
    /// Check only the actual use token's outer expansion allowance.
    /// The result does not enable crate features or authorize API execution.
    /// # Errors
    /// Rejects changed source/owner/frame identities and exhausted work/storage.
    pub fn allows(&mut self, site: Location, feature: &FeatureName<'_>) -> Result<bool, Error> {
        let data = self.header(site, feature.spelling().len())?;
        Ok(data.as_ref().is_some_and(|data| data.contains(feature)))
    }
    /// Inspect only the actual token's outer unsafe-code lint metadata.
    /// This does not authorize an unsafe operation or enable a crate feature.
    /// # Errors
    /// Rejects changed owners/frames/witnesses and exhausted global budgets.
    pub fn allows_unsafe_code_lint(&mut self, site: Location) -> Result<bool, Error> {
        let data = self.header(site, 0)?;
        Ok(data
            .as_ref()
            .is_some_and(|data| data.declaration().unsafe_lint))
    }
    fn header(
        &mut self,
        site: Location,
        extra_work: usize,
    ) -> Result<Option<Rc<HeaderData>>, Error> {
        let (context, origin_links, origin_lookups) = self.context(site)?;
        let Some(context) = context else {
            return Ok(None);
        };
        let package = self.view.package;
        let error = |message| Error::at(site.file, site.span.start, message);
        let saved = self
            .memo
            .as_ref()
            .and_then(|memo| memo.frames.get(&context))
            .copied();
        let frame = if let Some(frame) = saved {
            if !self.view().matches(frame) {
                return Err(error("cached macro allowance frame identity changed"));
            }
            frame
        } else {
            self.view().frame(context)?
        };
        let Some(owner) = frame.owner else {
            return Ok(None);
        };
        let cached = self
            .memo
            .as_ref()
            .and_then(|memo| memo.headers.get(&owner))
            .cloned();
        if saved.is_some() && cached.is_none() {
            return Err(error("cached macro allowance header is missing"));
        }
        let data = if let Some(cached) = cached {
            let unit = package
                .units
                .get(owner.file.0 as usize)
                .ok_or_else(|| error("cached macro source image is missing"))?;
            if !Arc::ptr_eq(unit.source.original_image(), &cached.image) {
                return Err(error("cached macro source incarnation changed"));
            }
            cached.data
        } else {
            self.view().authenticate_source(frame.definition)?
        };
        if !self.view().matches_header(frame.definition, data.as_ref()) {
            return Err(error("cached macro allowance header identity changed"));
        }
        panic_contracts::published(package, owner, data.as_ref(), 1 + extra_work)?;
        if saved.is_none() {
            self.retain(frame, data.clone())?;
        }
        if let Some(memo) = &mut self.memo {
            memo.origin_links = memo.origin_links.saturating_add(origin_links);
            memo.origin_lookups = memo.origin_lookups.saturating_add(origin_lookups);
        }
        Ok(data)
    }
    fn context(&self, site: Location) -> Result<(Option<ExpansionId>, usize, usize), Error> {
        let package = self.view.package;
        let Some(source) = package.source_core() else {
            return Ok((None, 0, 0));
        };
        let error = |message| Error::at(site.file, site.span.start, message);
        source.macros.charge(1, 0).map_err(error)?;
        if site.span.start >= site.span.end
            || package
                .units
                .get(site.file.0 as usize)
                .and_then(|unit| {
                    unit.source
                        .text()
                        .get(site.span.start as usize..site.span.end as usize)
                })
                .is_none()
        {
            return Err(error("invalid macro feature use span"));
        }
        let mut origin_links = 0_usize;
        let mut origin_lookups = 0_usize;
        let raw = contexts::at_counted(
            &package.units,
            site.file,
            site.span.start,
            |steps, links, lookups| {
                origin_links = origin_links.saturating_add(links);
                origin_lookups = origin_lookups.saturating_add(lookups);
                source.macros.charge(steps, 0)
            },
        )?;
        let context = package.marks.outer(raw).map_err(error)?;
        Ok((context, origin_links, origin_lookups))
    }
    fn retain(&mut self, frame: Frame, data: Option<Rc<HeaderData>>) -> Result<(), Error> {
        let error = |message| Error::at(frame.call.file, frame.original.start, message);
        let source = self
            .view
            .package
            .source_core()
            .ok_or_else(|| error("missing macro allowance memo provider"))?;
        let owner = frame
            .owner
            .ok_or_else(|| error("unowned macro allowance memo"))?;
        source
            .macros
            .charge(1, 4 * size_of::<(ExpansionId, Frame)>())
            .map_err(error)?;
        if self.memo.is_none() {
            source
                .macros
                .charge(1, 4 * size_of::<Memo>())
                .map_err(error)?;
            *self.memo = Some(Box::default());
        }
        let memo = self
            .memo
            .as_mut()
            .ok_or_else(|| error("missing macro allowance memo"))?;
        memo.frames
            .try_reserve(1)
            .map_err(|_| error("macro frame memo allocation failed"))?;
        if !memo.headers.contains_key(&owner) {
            source
                .macros
                .charge(1, 4 * size_of::<(NodeKey, CachedHeader)>())
                .map_err(error)?;
            memo.headers
                .try_reserve(1)
                .map_err(|_| error("macro header memo allocation failed"))?;
        }
        source.macros.memo_payload(memo.payload()).map_err(error)?;
        let image = Arc::clone(
            self.view
                .package
                .units
                .get(owner.file.0 as usize)
                .ok_or_else(|| error("macro source image disappeared"))?
                .source
                .original_image(),
        );
        memo.headers
            .entry(owner)
            .or_insert(CachedHeader { data, image });
        memo.frames.insert(frame.context, frame);
        Ok(())
    }
}
