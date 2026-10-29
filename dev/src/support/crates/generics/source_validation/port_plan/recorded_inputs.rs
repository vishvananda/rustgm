//! Coverage records requests, not their satisfaction or a universal-alias identity.
use super::{
    Context, DomainIndex, Work,
    nodes::{Arena, NodeId, Rewrite},
    s, uses,
};
use crate::support::{
    constness::Constness,
    crates::owner_catalog::{BodyRef, DeclarationRef},
    hash::HashSet,
    retained,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Site {
    Body(u32),
    Declaration(DeclarationRef, u32),
}
impl From<s::InputSite> for Site {
    fn from(site: s::InputSite) -> Self {
        match site {
            s::InputSite::Body(i) => Self::Body(i),
            s::InputSite::Declaration { owner, site } => Self::Declaration(owner, site),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Relation {
    body: BodyRef,
    site: Site,
    actual: NodeId,
    expected: NodeId,
}
#[derive(Default)]
pub(super) struct Coverage {
    variables: HashSet<(BodyRef, u32, bool)>,
    relations: HashSet<Relation>,
    formed: HashSet<(BodyRef, NodeId)>,
    copies: HashSet<(BodyRef, NodeId)>,
    local_borrows: HashSet<(BodyRef, s::RegionRef)>,
}
impl Coverage {
    pub(super) fn new(
        domains: &DomainIndex<'_, '_>,
        arena: &mut Arena,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let mut result = Self::default();
        let rows = domains.headers.rows();
        let mut actual_copy = None;
        for role in rows.policy.roles.slice(&rows.roles)? {
            work.charge(1)?;
            if role.role == crate::support::bindings::LangItem::Copy
                && actual_copy.replace(role.declaration).is_some()
            {
                return Err("source Copy role has duplicate actual bindings");
            }
        }
        for input in &rows.body_inputs {
            work.charge(1)?;
            let original = &rows.originals[input.body.0 as usize];
            let context = Context {
                owner: original.declaration,
                frame: Some(input.frame),
                body: Some(input.body),
            };
            for raw in &input.obligations {
                result.obligation(domains, arena, context, raw, work)?;
            }
            let copy = input
                .builtins
                .as_ref()
                .and_then(|b| b.source_copy.as_ref())
                .ok_or("source original lost its active Copy query carrier")?;
            if copy.declaration != actual_copy {
                return Err("source original Copy carrier changed its exact policy role binding");
            }
            for goal in &copy.goals {
                work.charge(1)?;
                let s::Requirement::Trait {
                    application,
                    effect: Constness::Ordinary,
                } = goal
                else {
                    return Err("source Copy query changed its ordinary actual trait request");
                };
                let application = &rows.trait_applications[application.0 as usize];
                if Some(application.declaration) != copy.declaration {
                    return Err("source Copy query belongs to another actual declaration");
                }
                let key = uses::key(
                    domains,
                    application.subject,
                    Rewrite::Original,
                    Some(context),
                    work,
                )?;
                let ty = arena.root(domains, key, work)?;
                arena.reserve(retained::new_map::<(BodyRef, NodeId), ()>(1), 0, work)?;
                result.copies.insert((input.body, ty));
            }
        }
        Ok(result)
    }
    fn obligation(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        arena: &mut Arena,
        context: Context,
        input: &s::Obligation,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let body = context
            .body
            .ok_or("source coverage has no original body owner")?;
        match input.kind {
            s::InputKind::Variable {
                slot,
                needs_provenance,
            } => {
                arena.reserve(retained::new_map::<(BodyRef, u32, bool), ()>(1), 0, work)?;
                self.variables.insert((body, slot, needs_provenance));
            }
            s::InputKind::Coercion {
                actual,
                expected,
                unsize: false,
                reference: None,
            } => {
                let actual = arena.root(
                    domains,
                    uses::key(domains, actual, Rewrite::Original, Some(context), work)?,
                    work,
                )?;
                let expected = arena.root(
                    domains,
                    uses::key(domains, expected, Rewrite::Original, Some(context), work)?,
                    work,
                )?;
                arena.reserve(retained::new_map::<Relation, ()>(1), 0, work)?;
                self.relations.insert(Relation {
                    body,
                    site: input.site.into(),
                    actual,
                    expected,
                });
            }
            s::InputKind::Coercion { .. } => {
                return Err(
                    "source adjusted/unsizing coercion requires a complete wider input profile",
                );
            }
            s::InputKind::WellFormed(root) => {
                let node = arena.root(
                    domains,
                    uses::key(domains, root, Rewrite::Original, Some(context), work)?,
                    work,
                )?;
                arena.reserve(retained::new_map::<(BodyRef, NodeId), ()>(1), 0, work)?;
                self.formed.insert((body, node));
            }
            s::InputKind::LocalBorrow(region) => {
                arena.reserve(retained::new_map::<(BodyRef, s::RegionRef), ()>(1), 0, work)?;
                self.local_borrows.insert((body, region));
            }
            s::InputKind::ScopedWellFormed { .. } => {
                return Err(
                    "source scoped operation formation requires its complete wider input profile",
                );
            }
            s::InputKind::Membership { .. }
            | s::InputKind::Equality(_)
            | s::InputKind::Outlives(_)
            | s::InputKind::TypeOutlives { .. } => {}
        }
        Ok(())
    }
    pub(super) fn variable(
        &self,
        body: BodyRef,
        slot: u32,
        provenance: bool,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if !self.variables.contains(&(body, slot, provenance)) {
            return Err("source operation omitted its exact required variable/provenance input");
        }
        Ok(())
    }
    pub(super) fn relation(
        &self,
        body: BodyRef,
        site: s::InputSite,
        actual: NodeId,
        expected: NodeId,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if !self.relations.contains(&Relation {
            body,
            site: site.into(),
            actual,
            expected,
        }) {
            return Err("source Constrain omitted its exact operation coercion input");
        }
        Ok(())
    }
    pub(super) fn copy(
        &self,
        body: BodyRef,
        ty: NodeId,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if !self.copies.contains(&(body, ty)) {
            return Err("source read omitted its exact active Copy query input");
        }
        Ok(())
    }
    pub(super) fn formed(
        &self,
        body: BodyRef,
        ty: NodeId,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(1)?;
        Ok(self.formed.contains(&(body, ty)))
    }
    pub(super) fn local_borrow(
        &self,
        body: BodyRef,
        region: s::RegionRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if !self.local_borrows.contains(&(body, region)) {
            return Err("source local borrow omitted its exact function-storage upper-bound input");
        }
        Ok(())
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.variables
            .capacity()
            .saturating_mul(size_of::<(BodyRef, u32, bool)>())
            .saturating_add(
                self.relations
                    .capacity()
                    .saturating_mul(size_of::<Relation>()),
            )
            .saturating_add(
                self.formed
                    .capacity()
                    .saturating_mul(size_of::<(BodyRef, NodeId)>()),
            )
            .saturating_add(
                self.copies
                    .capacity()
                    .saturating_mul(size_of::<(BodyRef, NodeId)>()),
            )
            .saturating_add(
                self.local_borrows
                    .capacity()
                    .saturating_mul(size_of::<(BodyRef, s::RegionRef)>()),
            )
    }
}
