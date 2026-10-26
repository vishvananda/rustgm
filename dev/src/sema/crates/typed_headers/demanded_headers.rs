//! Real signature pairs are private data, never provisional semantic headers.
use super::super::{
    error,
    recipe_cursor::{
        Availability, Cursor, Demand, Dependency, Maps, MemoLookup, MemoReader, Port, Reserved,
    },
    signature_shapes,
};
use super::Session;
use crate::{
    resolve::{
        body_origin::{ImportedBodyId, ImportedDeclarationId},
        crates::{ReservationWork, unpaid_provided::RecipeView},
        expansion::Report,
        package::Error,
    },
    sema::model::Declarations,
    support::crates::{
        Dependencies,
        generics::{self as g, source as s},
        owner_catalog::DeclarationRef,
    },
    types::{Store, TypeId},
};
use std::{rc::Rc, sync::Arc};
#[cfg(test)]
#[path = "../../../../tests/invariants/mapped_signature_controls.rs"]
mod mapping_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/demanded_controls.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SignaturePair {
    pub declaration: ImportedDeclarationId,
    pub body: Option<ImportedBodyId>,
    pub original: TypeId,
    pub callable: TypeId,
    pub function_row: u32,
}
#[derive(Clone, Copy)]
enum Pair {
    Empty,
    Ready(SignaturePair),
    Blocked(Dependency),
}
pub(super) struct State {
    pub cursor: Cursor,
    pub maps: Rc<Maps>,
    pairs: Vec<Pair>,
    reader: Option<ReadMapping>,
    pub(super) callables: Option<Box<super::template_demands::Prepared>>,
}
struct ReadMapping {
    types: MemoReader,
    maps: Rc<Maps>,
}
impl ReadMapping {
    fn require(
        &self,
        state: &State,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(1)?;
        if !Rc::ptr_eq(&self.maps, &state.maps) {
            return Err(error(
                "private mapping changed its actual shared slot/region allocation",
            ));
        }
        state.cursor.require_reader(&self.types, view, store, work)
    }
}
pub(super) type Progress = (Box<Session>, Demand<()>);

impl State {
    pub(super) fn new(
        view: &RecipeView<'_>,
        store: &mut Store,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        let rows = view.rows()?;
        work.charge(
            rows.types
                .len()
                .saturating_mul(3)
                .saturating_add(rows.slots.len())
                .saturating_add(rows.regions.len())
                .saturating_add(rows.functions.len())
                .saturating_add(6),
        )?;
        let growth = size_of::<Self>()
            .saturating_add(Cursor::scratch_bytes(rows.types.len()))
            .saturating_sub(size_of::<Cursor>())
            .saturating_add(Maps::allocation_bound(rows.slots.len(), rows.regions.len()))
            .saturating_add(rows.functions.len().saturating_mul(size_of::<Pair>()))
            .saturating_add(
                store.node_allocation_bound(rows.types.len().saturating_add(rows.slots.len())),
            )
            .saturating_add(store.parameter_allocation_bound(rows.slots.len()));
        work.reserve(growth)?;
        let state = Self {
            cursor: Cursor::prepaid(
                rows.types.len(),
                Availability::DeclaredSignatures,
                store.arena_identity(),
                view.root(),
            ),
            maps: Maps::prepaid(view, store)?,
            pairs: vec![Pair::Empty; rows.functions.len()],
            reader: None,
            callables: None,
        };
        if state.retained_bytes() > growth {
            return Err(error(
                "private recipe memo exceeded its actual allocation preflight",
            ));
        }
        Ok(state)
    }
    pub(super) fn pair(
        &mut self,
        view: RecipeView<'_>,
        store: &mut Store,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Demand<SignaturePair>, Error> {
        if let Some(reader) = &self.reader {
            reader.require(self, &view, store, work)?;
        }
        let mut port = Reserved::new(view, &self.maps, store, work)?;
        self.cursor.require_owner(&port, store, work)?;
        work.charge(1)?;
        let rows = port.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(owner).map_err(error)? else {
            return Err(error(
                "signature demand requires its actual function declaration",
            ));
        };
        let memo = self
            .pairs
            .get_mut(index as usize)
            .ok_or_else(|| error("signature demand lost its dense actual header row"))?;
        match *memo {
            Pair::Ready(pair) => return Ok(Demand::Ready(pair)),
            Pair::Blocked(dependency) => return Ok(Demand::Blocked(dependency)),
            Pair::Empty => {}
        }
        if let Some(dependency) = self.cursor.blocked() {
            return Ok(Demand::Blocked(dependency));
        }
        let raw = &rows.functions[index as usize];
        let (declaration, body) = match header_port(&port, owner, raw, work)? {
            Demand::Ready(header) => header,
            Demand::Blocked(dependency) => {
                *memo = Pair::Blocked(dependency);
                return Ok(Demand::Blocked(dependency));
            }
        };
        let original = match signature(
            &mut self.cursor,
            &mut port,
            store,
            owner,
            s::BinderKind::OriginalFunction,
            raw.original,
            work,
        )? {
            Demand::Ready(signature) => signature,
            Demand::Blocked(dependency) => {
                *memo = Pair::Blocked(dependency);
                return Ok(Demand::Blocked(dependency));
            }
        };
        let callable = match signature(
            &mut self.cursor,
            &mut port,
            store,
            owner,
            s::BinderKind::PublicFunction,
            raw.callable,
            work,
        )? {
            Demand::Ready(signature) => signature,
            Demand::Blocked(dependency) => {
                *memo = Pair::Blocked(dependency);
                return Ok(Demand::Blocked(dependency));
            }
        };
        let pair = SignaturePair {
            declaration,
            body,
            original,
            callable,
            function_row: index,
        };
        *memo = Pair::Ready(pair);
        Ok(Demand::Ready(pair))
    }
    pub(super) fn memo(&self) -> Result<&MemoReader, Error> {
        self.reader
            .as_ref()
            .map(|reader| &reader.types)
            .ok_or_else(|| error("private selected mapping lacks its actual shared memo reader"))
    }
    pub(super) fn require_mapping(
        &mut self,
        view: &RecipeView<'_>,
        store: &Store,
        cold: bool,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        self.maps.require(view, store, work)?;
        if self.reader.is_none() && cold {
            let types = self.cursor.reader(view, store, work)?;
            work.charge(1)?;
            self.reader = Some(ReadMapping {
                types,
                maps: Rc::clone(&self.maps),
            });
        }
        let reader = self
            .reader
            .as_ref()
            .ok_or_else(|| error("private callable readiness lost its actual memo reader"))?;
        reader.require(self, view, store, work)
    }
    pub(super) fn require_signatures(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        header: s::FunctionHeader,
        pair: SignaturePair,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        let reader = self
            .reader
            .as_ref()
            .ok_or_else(|| error("private callable signature lacks its actual memo reader"))?;
        reader.require(self, view, store, work)?;
        let rows = view.rows()?;
        for (raw, kind, actual) in [
            (
                header.original,
                s::BinderKind::OriginalFunction,
                pair.original,
            ),
            (
                header.callable,
                s::BinderKind::PublicFunction,
                pair.callable,
            ),
        ] {
            // Pay the complete selected argument/result scan before any lookup.
            work.charge(
                raw.parameters
                    .slice(&rows.roots)
                    .map_err(error)?
                    .len()
                    .saturating_add(1),
            )?;
            let mut steps = 0usize;
            signature_shapes::require(
                rows,
                store,
                signature_shapes::SignaturePort {
                    owner: header.declaration,
                    kind,
                    raw,
                },
                actual,
                &mut steps,
                |reference| match reader.types.get(view, store, reference, work) {
                    Ok(MemoLookup::Ready(ty)) => Ok(ty),
                    Ok(MemoLookup::Pending {
                        owner: pending_owner,
                        root,
                    }) => {
                        if pending_owner != view.root() || root != reference {
                            Err("private signature changed its actual pending dependency")
                        } else {
                            Err("private signature requires a pending actual source type")
                        }
                    }
                    Err(_) => Err("private signature lost its actual source memo"),
                },
            )
            .map_err(error)?;
        }
        Ok(())
    }
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .cursor
                .retained_bytes()
                .saturating_sub(size_of::<Cursor>())
            + self.maps.retained_bytes()
            + crate::support::retained::vector(&self.pairs)
            + self
                .callables
                .as_deref()
                .map_or(0, super::template_demands::Prepared::retained_bytes)
    }
}
fn header_port(
    port: &Reserved<'_, '_>,
    owner: DeclarationRef,
    raw: &s::FunctionHeader,
    work: &mut ReservationWork,
) -> Result<Demand<(ImportedDeclarationId, Option<ImportedBodyId>)>, Error> {
    let rows = port.rows();
    let declaration = port.declaration(owner, work)?;
    if raw.regions.captures != 0 || raw.regions.early.start != raw.regions.early.end {
        let dependency = Dependency::DeclaredLifetime(declaration);
        return Ok(Demand::Blocked(dependency));
    }
    for &slot in raw.regions.late.slice(&rows.region_slots).map_err(error)? {
        work.charge(1)?;
        let slot = rows
            .slots
            .get(slot.0 as usize)
            .ok_or_else(|| error("signature demand has an unknown late declaration slot"))?;
        if slot.definition.is_some() {
            let dependency = Dependency::DeclaredLifetime(declaration);
            return Ok(Demand::Blocked(dependency));
        }
    }
    let body = match raw.body {
        s::FunctionBody::Original(reference) => Some(port.body(reference, work)?),
        s::FunctionBody::Absent => {
            let parent = raw
                .parent
                .ok_or_else(|| error("required signature demand lacks its actual trait owner"))?;
            if !matches!(
                rows.header(parent).map_err(error)?,
                Some(s::HeaderRef::Trait(_))
            ) || rows.declarations[owner.0 as usize].definition.parent != Some(parent)
            {
                return Err(error(
                    "signature body absence is not its actual required trait member",
                ));
            }
            None
        }
    };
    Ok(Demand::Ready((declaration, body)))
}
fn signature(
    cursor: &mut Cursor,
    port: &mut Reserved<'_, '_>,
    store: &mut Store,
    owner: DeclarationRef,
    kind: s::BinderKind,
    raw: g::Signature,
    work: &mut ReservationWork,
) -> Result<Demand<TypeId>, Error> {
    work.charge(1)?;
    port.definition(owner, work)?;
    let rows = port.rows();
    let count = signature_shapes::check(rows, Some((owner, kind)), raw)?;
    let roots = raw.parameters.slice(&rows.roots).map_err(error)?;
    work.charge(roots.len().saturating_add(1))?;
    work.reserve(
        roots
            .len()
            .saturating_mul(4 * size_of::<TypeId>())
            .saturating_add(store.node_allocation_bound(1)),
    )?;
    let mut parameters = Vec::with_capacity(roots.len());
    for &root in roots {
        match cursor.demand(port, store, root, work)? {
            Demand::Ready(ty) => parameters.push(ty),
            Demand::Blocked(dependency) => return Ok(Demand::Blocked(dependency)),
        }
    }
    let result = match cursor.demand(port, store, raw.result, work)? {
        Demand::Ready(ty) => ty,
        Demand::Blocked(dependency) => return Ok(Demand::Blocked(dependency)),
    };
    signature_shapes::intern(store, count, raw, &parameters, result).map(Demand::Ready)
}
impl Session {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private signature-pair entry awaits the real callable-template consumer; no Provided Graph or execution admission"
        )
    )]
    pub(super) fn select_signature(
        report: Report,
        facts: Declarations,
        inventory: Arc<Dependencies>,
        provider_index: u32,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Progress, Error> {
        Self::prepare(report, facts, inventory, provider_index, work)?
            .boxed(work)?
            .signature_pair(owner, work)
    }
    pub(super) fn boxed(self, work: &mut ReservationWork) -> Result<Box<Self>, Error> {
        work.reserve(size_of::<Self>())?;
        Ok(Box::new(self))
    }
    pub(super) fn signature_pair(
        mut self: Box<Self>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Progress, Error> {
        self.require_owner(work)?;
        let view = self
            .inputs
            .recipe_view(&self.inventory, self.provider_index, work)?;
        work.charge(1)?;
        if !matches!(
            view.rows()?.header(owner).map_err(error)?,
            Some(s::HeaderRef::Function(_))
        ) {
            return Err(error(
                "signature demand requires its actual function declaration",
            ));
        }
        if self.recipes.is_none() {
            self.recipes = Some(Box::new(State::new(&view, &mut self.facts.types, work)?));
        }
        let state = self
            .recipes
            .as_mut()
            .ok_or_else(|| error("private signature memo disappeared"))?;
        let status = match state.pair(view, &mut self.facts.types, owner, work)? {
            Demand::Ready(_) => Demand::Ready(()),
            Demand::Blocked(dependency) => Demand::Blocked(dependency),
        };
        Ok((self, status))
    }
}
