//! Check the structural bounds and binder of each transported trait object.
use super::{BoundUse, Builder, HeaderRef, Summary, Work, node_rows::region_summary, s};
use crate::support::marker_traits::AutoTrait;

const MARKERS: [AutoTrait; 7] = [
    AutoTrait::Send,
    AutoTrait::Sync,
    AutoTrait::Unpin,
    AutoTrait::UnwindSafe,
    AutoTrait::RefUnwindSafe,
    AutoTrait::Freeze,
    AutoTrait::UnsafeUnpin,
];
pub(super) struct Markers([Option<s::DeclarationAddress>; MARKERS.len()]);
impl Markers {
    pub(super) fn new(rows: &s::SourceRows, work: &mut Work) -> Result<Self, &'static str> {
        let mut declarations = [None; MARKERS.len()];
        if !rows.cold().dynamics.is_empty() {
            work.charge(rows.roles.len().saturating_add(rows.diagnostics().len()))?;
            for role in &rows.roles {
                if let Some(index) = MARKERS
                    .iter()
                    .position(|marker| marker.lang_item() == role.role)
                {
                    declarations[index] = Some(role.declaration);
                }
            }
            for role in rows.diagnostics() {
                if role.role == s::DiagnosticRole::Send {
                    declarations[0] = Some(role.declaration);
                }
            }
        }
        Ok(Self(declarations))
    }
    fn has(&self, marker: s::DynamicMarker) -> bool {
        MARKERS
            .iter()
            .position(|&actual| actual == marker.marker)
            .is_some_and(|index| self.0[index] == Some(marker.declaration))
    }
}

#[derive(Default)]
pub(super) struct Inventory {
    objects: usize,
    bindings: usize,
    markers: usize,
}
impl Inventory {
    pub(super) fn record(
        &mut self,
        rows: &s::DynamicColumns,
        reference: s::DynamicTypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let row = rows
            .objects
            .get(self.objects)
            .filter(|_| reference.0 as usize == self.objects)
            .ok_or("source dynamic recipe occurs outside its actual type order")?;
        if row.bindings.start as usize != self.bindings
            || row.markers.start as usize != self.markers
        {
            return Err("source dynamic recipe lost the order of its owned bounds");
        }
        self.bindings += row.bindings.slice(&rows.bindings)?.len();
        self.markers += row.markers.slice(&rows.markers)?.len();
        self.objects += 1;
        Ok(())
    }
    pub(super) fn finish(self, rows: &s::DynamicColumns) -> Result<(), &'static str> {
        if self.objects != rows.objects.len()
            || self.bindings != rows.bindings.len()
            || self.markers != rows.markers.len()
        {
            return Err("source dynamic column has no actual type owner");
        }
        Ok(())
    }
}

impl Builder<'_, '_> {
    pub(super) fn dynamic_row(
        &self,
        id: s::DynamicTypeRef,
    ) -> Result<s::DynamicType, &'static str> {
        if !self.profile.declaration_policies() {
            return Err("source trait object requires version fourteen");
        }
        self.headers
            .rows()
            .cold()
            .dynamics
            .objects
            .get(id.0 as usize)
            .copied()
            .ok_or("source trait object has no dynamic recipe")
    }

    pub(super) fn dynamic_summary(
        &self,
        reference: s::DynamicTypeRef,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        let row = self.dynamic_row(reference)?;
        let rows = self.headers.rows();
        let columns = &rows.cold().dynamics;
        let binder = rows
            .binders
            .get(row.binder.0 as usize)
            .filter(|binder| binder.kind == s::BinderKind::DynamicType)
            .ok_or("source trait object lost its actual inner trait binder")?;
        let markers = row.markers.slice(&columns.markers)?;
        self.dynamic_markers(markers, work)?;
        let bindings = row.bindings.slice(&columns.bindings)?;
        let (principal, mut arguments) = row
            .arguments
            .slice(&rows.arguments)?
            .split_at_checked(row.principal_arguments as usize)
            .ok_or("source trait object lost the complete principal arguments")?;
        let mut summary = Summary::default();
        if let Some(head) = row.principal {
            let header = self.dynamic_trait(head, work)?;
            if header.flags.bits() & s::TraitFlags::AUTO != 0 {
                return Err("source dynamic principal must be a non-auto trait");
            }
            summary = self.trait_arguments(header.scope, principal, summary, work)?;
        } else if !principal.is_empty() || !bindings.is_empty() || markers.is_empty() {
            return Err("source dynamic bounds require a principal or actual auto traits");
        }
        let mut previous = None;
        for &binding in bindings {
            work.charge(1)?;
            let member = self.dynamic_member(binding, work)?;
            if previous.is_some_and(|ordinal| ordinal >= member.0) {
                return Err("source trait object repeats or reorders an associated bound");
            }
            previous = Some(member.0);
            let (actual, rest) = arguments
                .split_at_checked(binding.arguments as usize)
                .ok_or("source dynamic associated bound lost its full parent arguments")?;
            let Some((s::Argument::Type(result), rest)) = rest.split_first() else {
                return Err("source dynamic associated bound requires one associated result type");
            };
            summary = self.trait_arguments(
                self.dynamic_trait(binding.owner, work)?.scope,
                actual,
                summary,
                work,
            )?;
            summary = self.merge(summary, self.child(*result, work)?, work)?;
            arguments = rest;
        }
        if !arguments.is_empty() {
            return Err("source trait object has unused dynamic input arguments");
        }
        if let Some(bound) = summary.bound {
            if bound.frame != row.binder {
                return Err("source dynamic arguments belong to a foreign trait binder");
            }
            summary.bound = if bound.depth == 0 {
                None
            } else {
                Some(BoundUse {
                    frame: binder
                        .parent
                        .ok_or("source dynamic bound escaped its enclosing binder")?,
                    depth: bound.depth - 1,
                })
            };
        }
        if let Some(depth) = summary.nominal_depth {
            if binder.parent.is_none() {
                return Err("source dynamic bound escaped its enclosing binder");
            }
            summary.nominal_depth = Some(depth.checked_sub(1).ok_or(
                "source dynamic bound captured a nominal region at an invalid binder depth",
            )?);
        }
        self.merge(
            summary,
            region_summary(
                self.headers,
                self.binders,
                row.region,
                self.profile.declaration_policies(),
                work,
            )?,
            work,
        )
    }

    fn dynamic_trait(
        &self,
        declaration: s::DeclarationAddress,
        work: &mut Work,
    ) -> Result<&s::TraitHeader, &'static str> {
        if declaration.artifact.0 != 0 {
            return Err("source dynamic bounds require authenticated dependency trait transport");
        }
        let Some(HeaderRef::Trait(index)) = self.headers.header(declaration.declaration, work)?
        else {
            return Err("source dynamic bound has no actual checked trait declaration");
        };
        Ok(&self.headers.rows().traits[index as usize])
    }

    fn dynamic_member(
        &self,
        binding: s::DynamicBinding,
        work: &mut Work,
    ) -> Result<crate::support::crates::owner_catalog::DeclarationRef, &'static str> {
        if binding.member.artifact != binding.owner.artifact || binding.member.artifact.0 != 0 {
            return Err("source dynamic associated type changed its defining artifact");
        }
        let Some(HeaderRef::AssociatedType(index)) =
            self.headers.header(binding.member.declaration, work)?
        else {
            return Err("source dynamic binding lost its actual associated type");
        };
        let header = &self.headers.rows().associated_types[index as usize];
        let own = self.headers.scopes().declared_slots(header.scope, work)?;
        if header.parent != binding.owner.declaration || own.start != own.end {
            return Err("source dynamic binding changed its declaring trait or associated scope");
        }
        Ok(binding.member.declaration)
    }

    fn dynamic_markers(
        &self,
        markers: &[s::DynamicMarker],
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let mut previous = None;
        for &marker in markers {
            work.charge(1)?;
            if previous.is_some_and(|last| last >= marker.marker)
                || !self.dynamic_markers.has(marker)
                || self.dynamic_trait(marker.declaration, work)?.flags.bits() & s::TraitFlags::AUTO
                    == 0
            {
                return Err("source trait object lacks an exact ordered auto-trait bound");
            }
            previous = Some(marker.marker);
        }
        Ok(())
    }
}
