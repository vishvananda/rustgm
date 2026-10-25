//! Every consuming root checks its actual scope/body/frame against one summary.
use super::{BinderIndex, HeaderIndex, Summary, Types, Work, g, s};
use crate::support::crates::owner_catalog::{BodyRef, DeclarationRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in super::super) struct Context {
    pub owner: DeclarationRef,
    pub frame: Option<g::BinderRef>,
    pub body: Option<BodyRef>,
}
impl Types {
    pub(in super::super) fn root(
        &self,
        headers: &HeaderIndex<'_>,
        context: Context,
        ty: g::TypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        Self::check(headers, context, self.summary(ty, work)?, work)
    }
    pub(in super::super) fn function_application(
        &self,
        headers: &HeaderIndex<'_>,
        context: Context,
        application: s::FunctionApplicationRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if application.0 as usize >= self.function_count {
            return Err("source function use crosses its typed application namespace");
        }
        Self::check(
            headers,
            context,
            self.summaries[self.type_count + application.0 as usize],
            work,
        )
    }
    pub(in super::super) fn trait_application(
        &self,
        headers: &HeaderIndex<'_>,
        context: Context,
        application: s::TraitApplicationRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if application.0 as usize >= headers.rows().trait_applications.len() {
            return Err("source trait use crosses its typed application namespace");
        }
        Self::check(
            headers,
            context,
            self.summaries[self.type_count + self.function_count + application.0 as usize],
            work,
        )
    }
    pub(in super::super) fn closed_root(
        &self,
        ty: g::TypeRef,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        Ok(closed(self.summary(ty, work)?))
    }
    pub(in super::super) fn region_independent(
        &self,
        ty: g::TypeRef,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        let summary = self.summary(ty, work)?;
        Ok(summary.body.is_none() && summary.bound.is_none())
    }
    pub(in super::super) fn closed_application(
        &self,
        application: s::FunctionApplicationRef,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(1)?;
        if application.0 as usize >= self.function_count {
            return Err("source application classification crosses its typed namespace");
        }
        Ok(closed(
            self.summaries[self.type_count + application.0 as usize],
        ))
    }

    pub(in super::super) fn region(
        headers: &HeaderIndex<'_>,
        binders: &BinderIndex<'_>,
        context: Context,
        region: s::RegionRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let summary = super::node_rows::region_summary(headers, binders, region, work)?;
        Self::check(headers, context, summary, work)
    }
    fn check(
        headers: &HeaderIndex<'_>,
        context: Context,
        summary: Summary,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if let Some(slot) = summary.parameter
            && !headers.scopes().contains(context.owner, slot, work)?
        {
            return Err("source consuming type root captures another parameter owner");
        }
        if summary.body.is_some_and(|body| Some(body) != context.body) {
            return Err("source consuming type root captures another original allocation");
        }
        if summary
            .bound
            .is_some_and(|bound| Some(bound.frame) != context.frame)
        {
            return Err("source consuming type root belongs to another current binder frame");
        }
        Ok(())
    }
}

fn closed(summary: Summary) -> bool {
    summary.parameter.is_none() && summary.body.is_none() && summary.bound.is_none()
}
