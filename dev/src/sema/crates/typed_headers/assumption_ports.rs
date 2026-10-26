//! The unpaid owner maps only actual canonical inputs, with selected growth costs.
use crate::{
    resolve::{
        body_origin::{ImportedDeclarationId, ProofSite},
        crates::ReservationWork,
        model::DefId,
        package::Error,
    },
    sema::crates::typed_headers::{
        error,
        predicate_remap::translation::{self, Mapping},
        recipe_cursor::{Cursor, Demand, Port, Reserved},
    },
    support::{
        crates::{
            generics::{self as g, source as s},
            owner_catalog::DeclarationRef,
        },
        retained::Growth,
    },
    types::{
        GenericArgs, RegionId, Store, TypeId,
        arguments::Argument,
        traits::{Application, ApplicationId},
    },
};

pub(super) struct Inputs<'rows, 'state> {
    pub port: Reserved<'rows, 'state>,
    pub cursor: &'state mut Cursor,
}
impl translation::sealed::Sealed for Inputs<'_, '_> {}
impl<'rows> Mapping<'rows, ReservationWork> for Inputs<'rows, '_> {
    fn rows(&self) -> s::ValidatedView<'rows> {
        self.port.rows()
    }
    fn trait_definition(
        &self,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<DefId, Error> {
        work.charge(1)?;
        if !matches!(
            self.port.rows().header(owner).map_err(error)?,
            Some(s::HeaderRef::Trait(_))
        ) {
            return Err(error(
                "private assumption lost its actual trait declaration",
            ));
        }
        self.port.definition(owner, work)
    }
    fn trait_declaration(
        &self,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<ImportedDeclarationId, Error> {
        self.port.declaration(owner, work)
    }
    fn site(
        &self,
        owner: DeclarationRef,
        site: u32,
        work: &mut ReservationWork,
    ) -> Result<ProofSite, Error> {
        self.port.input_site(owner, site, work)
    }
    fn ty(
        &mut self,
        store: &mut Store,
        root: g::TypeRef,
        work: &mut ReservationWork,
    ) -> Result<TypeId, Error> {
        match self.cursor.demand(&mut self.port, store, root, work)? {
            Demand::Ready(ty) => Ok(ty),
            Demand::Blocked(_) => Err(error(
                "private assumption requires its later actual type producer",
            )),
        }
    }
    fn region(&mut self, _: s::RegionRef, _: &mut ReservationWork) -> Result<RegionId, Error> {
        Err(error(
            "private assumption region requires its later defining domain",
        ))
    }
    fn arguments(
        &self,
        store: &mut Store,
        args: &[Argument],
        work: &mut ReservationWork,
    ) -> Result<GenericArgs, Error> {
        work.charge(args.len().saturating_add(1))?;
        if args.is_empty() {
            return Ok(GenericArgs::EMPTY);
        }
        if !args
            .iter()
            .all(|argument| matches!(argument, Argument::Type(_)))
        {
            return Err(error(
                "private assumption arguments require their later regional profile",
            ));
        }
        // A known miss is followed by the existing interner's same probe.
        // Prepay both walks, including real hash-collision comparisons.
        if let Some(existing) = store
            .find_mixed_arguments_charged(args, |count| {
                work.charge(count.saturating_mul(2))
                    .map_err(|_| "private argument probe work budget exceeded")
            })
            .map_err(error)?
        {
            return Ok(existing);
        }
        let before = store.trait_input_bytes();
        let reserved = work.retained;
        prepay(store.trait_arguments_growth(args.len()), work)?;
        work.charge(args.len())?;
        let value = store.intern_arguments(args).map_err(error)?;
        reconcile(store, before, reserved, work)?;
        Ok(value)
    }
    fn application(
        &self,
        store: &mut Store,
        application: Application,
        work: &mut ReservationWork,
    ) -> Result<ApplicationId, Error> {
        work.charge(1)?;
        if let Some(existing) = store.find_trait_application(application) {
            return Ok(existing);
        }
        let count = store
            .arguments(application.arguments)
            .ok_or_else(|| error("private assumption lost its actual arguments"))?
            .len();
        work.charge(count.saturating_add(3))?;
        let before = store.trait_input_bytes();
        let reserved = work.retained;
        prepay(store.trait_application_growth(), work)?;
        let value = store.intern_trait_application(application).map_err(error)?;
        reconcile(store, before, reserved, work)?;
        Ok(value)
    }
}
fn prepay(growth: Growth, work: &mut ReservationWork) -> Result<(), Error> {
    work.charge(growth.work)?;
    work.reserve(growth.bytes)
}
fn reconcile(
    store: &Store,
    before: usize,
    reserved: usize,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    let actual = store.trait_input_bytes().saturating_sub(before);
    if actual > work.retained.saturating_sub(reserved) {
        return Err(error(
            "private assumption interning exceeded its actual allocation preflight",
        ));
    }
    work.retained = reserved.saturating_add(actual);
    Ok(())
}

/// A selected impl's own application uses the same canonical clause cache.
pub(super) fn implementation_application(
    port: &mut Inputs<'_, '_>,
    applications: &mut impl translation::applications::Cache,
    owner: DeclarationRef,
    store: &mut Store,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    work.charge(1)?;
    let rows = port.port.rows();
    let Some(s::HeaderRef::Implementation(index)) = rows.header(owner).map_err(error)? else {
        return Ok(());
    };
    let raw = &rows.implementations[index as usize];
    if raw.self_definition != owner || raw.scope != owner {
        return Err(error(
            "selected implementation changed its actual Self/scope owner",
        ));
    }
    let application = raw.trait_application.ok_or_else(|| {
        error("selected inherent implementation requires its actual member producer")
    })?;
    let predicate = translation::application(port, applications, application, store, work)?;
    let actual = match port
        .cursor
        .demand(&mut port.port, store, raw.self_type, work)?
    {
        Demand::Ready(ty) => ty,
        Demand::Blocked(_) => {
            return Err(error(
                "selected implementation requires its actual pending Self type",
            ));
        }
    };
    if predicate.ty != actual {
        return Err(error(
            "selected implementation changed its exact Self/application pair",
        ));
    }
    Ok(())
}
