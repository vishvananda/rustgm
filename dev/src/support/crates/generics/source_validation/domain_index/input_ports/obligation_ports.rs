//! All retained goal roots keep their body/frame and allocation namespaces.
use super::{Context, DomainIndex, Work, s};

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
                self.membership(context, region, captures, work)
            }
            s::InputKind::LocalBorrow(region) => {
                self.types
                    .region(self.headers, &self.binders, context, region, work)
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
                self.types
                    .region(self.headers, &self.binders, context, shorter, work)
            }
        }
    }

    fn membership(
        &self,
        context: Context,
        region: s::RegionRef,
        captures: crate::support::crates::generics::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.types
            .region(self.headers, &self.binders, context, region, work)?;
        let rows = self.headers.rows();
        for &capture in captures.slice(&rows.arguments)? {
            match capture {
                s::Argument::Region(region) => {
                    self.types
                        .region(self.headers, &self.binders, context, region, work)?;
                }
                s::Argument::Type(ty) if self.declaration_policies => {
                    self.types.root(self.headers, context, ty, work)?;
                }
                s::Argument::Const(term) if self.declaration_policies => {
                    match rows.consts().get(term.0 as usize) {
                        Some(&s::ConstTerm::Value(value)) => {
                            self.types.root(self.headers, context, value.ty, work)?;
                            s::const_value(&rows.types, value)?;
                        }
                        Some(&s::ConstTerm::Parameter(slot))
                            if matches!(
                                rows.slots.get(slot.0 as usize).map(|row| row.kind),
                                Some(s::ParameterKind::Const { .. })
                            ) && self.headers.scopes().contains(
                                context.owner,
                                slot,
                                work,
                            )? => {}
                        _ => {
                            return Err("source membership capture has a foreign const argument");
                        }
                    }
                }
                _ => return Err("source membership captures must retain lifetime-only kinds"),
            }
        }
        Ok(())
    }
}
