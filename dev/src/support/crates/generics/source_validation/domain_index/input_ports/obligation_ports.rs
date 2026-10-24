//! All retained goal roots keep their body/frame and allocation namespaces.
use super::{Context, DomainIndex, Types, Work, s};

impl DomainIndex<'_, '_> {
    pub(in super::super) fn obligation(
        &self,
        context: Context,
        original: &s::Original,
        kind: s::InputKind,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        match kind {
            s::InputKind::Variable {
                slot,
                needs_provenance,
            } => {
                work.charge(1)?;
                if original
                    .variables
                    .get(slot as usize)
                    .is_none_or(|variable| {
                        variable.allocation != (s::RegionAllocation::Inference { needs_provenance })
                    })
                {
                    return Err("source inference variable goal lost its exact raw allocation");
                }
                Ok(())
            }
            s::InputKind::Membership { region, captures } => {
                Types::region(self.headers, &self.binders, context, region, work)?;
                for capture in captures.slice(&self.headers.rows().arguments)? {
                    let s::Argument::Region(region) = capture else {
                        return Err("source membership captures must retain lifetime-only kinds");
                    };
                    Types::region(self.headers, &self.binders, context, *region, work)?;
                }
                Ok(())
            }
            s::InputKind::LocalBorrow(region) => {
                Types::region(self.headers, &self.binders, context, region, work)
            }
            s::InputKind::Equality(bound) | s::InputKind::Outlives(bound) => {
                self.bound(context, bound, work)
            }
            s::InputKind::Coercion {
                actual, expected, ..
            } => {
                self.types.root(self.headers, context, actual, work)?;
                self.types.root(self.headers, context, expected, work)
            }
            s::InputKind::WellFormed(ty) => self.types.root(self.headers, context, ty, work),
            s::InputKind::ScopedWellFormed { ty, frame } => {
                work.charge(1)?;
                let actual = self
                    .headers
                    .rows()
                    .binders
                    .get(frame.0 as usize)
                    .ok_or("source scoped formation binder is unknown")?;
                if actual.owner != context.owner
                    || actual.parent != context.frame
                    || actual.kind != s::BinderKind::Clause
                {
                    return Err("source scoped formation has a foreign actual binder");
                }
                self.types.root(
                    self.headers,
                    Context {
                        frame: Some(frame),
                        ..context
                    },
                    ty,
                    work,
                )
            }
            s::InputKind::TypeOutlives { ty, shorter } => {
                self.types.root(self.headers, context, ty, work)?;
                Types::region(self.headers, &self.binders, context, shorter, work)
            }
        }
    }
}
