//! Fresh consumer construction preserves actual owners while all proof stays unpaid.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        bindings::{Context, consumer_policy::ProviderAdmission},
        body_origin::{DeclarationOrigin, ImportedDeclarationId},
        crates::unpaid_provided::Inputs,
        expansion,
        load::{InputFile, Sources},
    },
    support::{
        bindings::{LangItem, Mode},
        crates::CrateIdentity,
        source::SourceFile,
    },
    unpaid_cases::Result,
};
use std::{fmt::Write as _, path::Path, sync::Arc};

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
struct Fixture {
    admission: ProviderAdmission,
    context: Context,
    producer_arena: Identity,
    old_root: ImportedDeclarationId,
}
fn fixture(suffix: &str) -> Result<Fixture> {
    fixture_data(suffix, |_| {})
}
fn fixture_data(
    suffix: &str,
    change: impl FnOnce(&mut crate::support::crates::generics::source::HeaderAttributeColumns),
) -> Result<Fixture> {
    let mut actual = crate::unpaid_cases::trait_extended(suffix)?;
    change(&mut actual.contracts);
    let mut facts = actual.facts;
    let producer_arena = facts.types.arena_identity();
    let provider = actual.inventory.artifacts()[0].identity().clone();
    let context = Context {
        crate_name: Some("consumer".into()),
        current: Some(CrateIdentity::new("consumer", "private_current")?),
        dependencies: Some(Arc::clone(&actual.inventory)),
        mode: Mode::SourceClient { provider },
    };
    let mut work = ReservationWork::default();
    let inputs = Inputs::reserve(actual.report, Arc::clone(&actual.inventory), 0, &mut work)?;
    let old_root = inputs
        .recipe_view(actual.inventory.as_ref(), 0, &mut work)?
        .root();
    let admission = ProviderAdmission::from_reserved(inputs, actual.contracts, &mut work)?;
    Ok(Fixture {
        admission,
        context,
        producer_arena,
        old_root,
    })
}
fn input(text: &str) -> Result<InputFile> {
    Ok(InputFile {
        identity: "private-consumer.rs".into(),
        path: "private-consumer.rs".into(),
        source: SourceFile::new(text.as_bytes().to_vec())?,
    })
}
fn construct(
    root: InputFile,
    sources: &mut impl Sources,
    config: &Config,
    context: &Context,
    admission: ProviderAdmission,
    work: &mut ReservationWork,
) -> std::result::Result<Box<ProofSession>, Error> {
    let input = ConsumerInput::new(root, sources, config, context, admission, work)?;
    ProofSession::new(input, work)
}
const MAIN: &str = "#![feature(no_core)]#![no_core]fn main(){}";
const HOLDER: &str = "pub trait Holder<V>{fn carry<U>(value:U)->U{value}fn borrow(value:&V)->&V{value}}impl Holder<u8> for u8{}";

#[test]
fn fresh_consumer_owns_actual_report_catalog_store_and_reserved_role_origins() -> Result {
    let actual = fixture(HOLDER)?;
    let config = Config::default();
    let error =
        expansion::analyze_with_context(input(MAIN)?, &mut NoFiles, &config, &actual.context)
            .err()
            .ok_or("public Provided client admitted")?;
    assert_eq!(
        error.message,
        "source client provider profile is unsupported"
    );
    let root = input(MAIN)?;
    let image = Arc::clone(root.source.original_image());
    let mut work = ReservationWork::default();
    let session = construct(
        root,
        &mut NoFiles,
        &config,
        &actual.context,
        actual.admission,
        &mut work,
    )?;
    let authority = &session.authority;
    assert!(Arc::ptr_eq(
        &image,
        authority.report.package.units[0].source.original_image()
    ));
    assert!(authority.report.names_resolved);
    assert!(authority.report.package.source_core().is_none());
    assert!(authority.report.package.source_client().is_none());
    assert!(authority.report.resolution.source_bindings.is_none());
    assert!(authority.report.resolution.extern_scope.is_none());
    assert!(!authority.facts.types.has_identity(&actual.producer_arena));
    assert!(authority.facts.definitions.iter().all(Option::is_none));
    assert!(
        authority
            .facts
            .function_header(crate::resolve::model::DefId(0))?
            .is_none()
    );
    assert!(
        authority
            .reservation
            .catalog()
            .declaration_identity(actual.old_root, crate::resolve::model::DefKind::Module)
            .is_err()
    );
    let roles = authority
        .facts
        .source_bindings
        .as_deref()
        .ok_or("actual reserved roles")?;
    roles.require_provider_report(&authority.report)?;
    let sized = roles
        .role(LangItem::Sized)?
        .ok_or("actual imported Sized")?;
    assert!(matches!(sized.origin, DeclarationOrigin::Imported(_)));
    assert!(roles.validated_role(LangItem::Sized).is_err());
    assert!(authority.facts.associated.owner(sized.definition).is_none());
    assert!(authority.facts.lifetimes.callables.is_empty());
    assert!(
        authority
            .facts
            .header_attribute_contracts(&authority.report, sized.definition)
            .is_err()
    );
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    session.require(&mut work)?;
    eprintln!(
        "consumer-layout session={} authority={} policy-source={:?} package={} role-layouts={:?} default-store-bound={} actual-node-tables={} reserved={} peak={} work={}",
        size_of::<ProofSession>(),
        size_of::<ConsumerAuthority>(),
        crate::resolve::bindings::consumer_policy::layouts(),
        size_of::<crate::resolve::package::Package>(),
        crate::sema::compiler_items::Catalog::binding_layouts(),
        crate::types::Store::initialization_allocation_bound(),
        authority.facts.types.node_table_allocation().bytes,
        work.retained,
        work.peak,
        work.steps
    );
    Ok(())
}

#[test]
fn fresh_consumer_rejects_foreign_identity_inventory_source_configuration_and_catalog() -> Result {
    rejected_requests()?;
    replaced_owners()
}
fn rejected_requests() -> Result {
    let actual = fixture(HOLDER)?;
    let mut context = actual.context;
    context.current = match &context.mode {
        Mode::SourceClient { provider } => Some(provider.clone()),
        _ => None,
    };
    context.crate_name = context
        .current
        .as_ref()
        .map(|identity| identity.name().to_owned());
    let mut work = ReservationWork::default();
    assert!(
        construct(
            input(MAIN)?,
            &mut NoFiles,
            &Config::default(),
            &context,
            actual.admission,
            &mut work
        )
        .is_err()
    );
    assert!(work.steps > 0);
    let actual = fixture(HOLDER)?;
    let error = construct(
        input("#![feature(no_core)]#![no_core]fn main(){minicore::identity(7u8);}")?,
        &mut NoFiles,
        &Config::default(),
        &actual.context,
        actual.admission,
        &mut ReservationWork::default(),
    )
    .err()
    .ok_or("unsupported external binding admitted")?;
    assert!(!error.message.is_empty());
    let actual = fixture(HOLDER)?;
    let mut context = actual.context;
    context.mode = Mode::SourceClient {
        provider: CrateIdentity::new("foreign", "other_provider")?,
    };
    assert!(
        construct(
            input(MAIN)?,
            &mut NoFiles,
            &Config::default(),
            &context,
            actual.admission,
            &mut ReservationWork::default()
        )
        .is_err()
    );
    let actual = fixture(HOLDER)?;
    let mut context = actual.context;
    let other = fixture(HOLDER)?;
    assert!(!Arc::ptr_eq(
        context.dependencies.as_ref().ok_or("actual inventory")?,
        other
            .context
            .dependencies
            .as_ref()
            .ok_or("foreign inventory")?
    ));
    context.dependencies = other.context.dependencies;
    assert!(
        construct(
            input(MAIN)?,
            &mut NoFiles,
            &Config::default(),
            &context,
            actual.admission,
            &mut ReservationWork::default()
        )
        .is_err()
    );
    Ok(())
}
fn replaced_owners() -> Result {
    let mut work = ReservationWork::default();
    let actual = fixture(HOLDER)?;
    let mut first = construct(
        input(MAIN)?,
        &mut NoFiles,
        &Config::default(),
        &actual.context,
        actual.admission,
        &mut work,
    )?;
    let actual = fixture(HOLDER)?;
    let mut second = construct(
        input(MAIN)?,
        &mut NoFiles,
        &Config::default(),
        &actual.context,
        actual.admission,
        &mut work,
    )?;
    std::mem::swap(
        &mut first.authority.report.resolution.crates,
        &mut second.authority.report.resolution.crates,
    );
    assert!(first.require(&mut work).is_err());
    std::mem::swap(
        &mut first.authority.report.resolution.crates,
        &mut second.authority.report.resolution.crates,
    );
    std::mem::swap(
        &mut first.authority.facts.types,
        &mut second.authority.facts.types,
    );
    assert!(first.require(&mut work).is_err());
    std::mem::swap(
        &mut first.authority.facts.types,
        &mut second.authority.facts.types,
    );
    crate::resolve::bindings::consumer_policy::configuration_opponent(
        &mut first.authority.report.package,
    )?;
    first.require(&mut work)?;
    let unit = &mut first.authority.report.package.units[0];
    unit.source = unit.source.physical_copy();
    assert!(first.require(&mut work).is_err());
    Ok(())
}

fn unpaid_initialization_opponents() -> Result {
    let actual = fixture_data(HOLDER, |columns| {
        // Structurally valid unpaid data is not a positive source-completion receipt.
        for row in &mut columns.functions {
            row.must_use = None;
        }
    })?;
    let mut work = ReservationWork::default();
    let session = construct(
        input(MAIN)?,
        &mut NoFiles,
        &Config::default(),
        &actual.context,
        actual.admission,
        &mut work,
    )?;
    let role = session
        .authority
        .facts
        .source_bindings
        .as_deref()
        .ok_or("reserved catalog")?
        .role(LangItem::CloneFn)?
        .ok_or("actual clone member")?;
    assert!(
        session
            .authority
            .facts
            .header_attribute_contracts(&session.authority.report, role.definition)
            .is_err()
    );
    let invalid = fixture_data(HOLDER, |columns| {
        columns.functions.pop();
    });
    assert!(invalid.is_err());
    let extent = session.authority.facts.types.interned_extent();
    let mut initialization = ReservationWork {
        retained: crate::support::crates::MAX_BYTES - 32,
        ..ReservationWork::default()
    };
    assert!(Declarations::new_staged(&session.authority.report, &mut initialization).is_err());
    assert!(initialization.steps > 0 && initialization.peak > crate::support::crates::MAX_BYTES);
    assert_eq!(session.authority.facts.types.interned_extent(), extent);
    Ok(())
}
fn warm_reservation(count: usize) -> Result<usize> {
    let mut source = HOLDER.to_owned();
    for index in 0..count {
        write!(source, "pub fn unrelated_{index}<T>(value:T)->T{{value}}")?;
    }
    let actual = fixture(&source)?;
    let mut work = ReservationWork::default();
    let session = construct(
        input(MAIN)?,
        &mut NoFiles,
        &Config::default(),
        &actual.context,
        actual.admission,
        &mut work,
    )?;
    let state = (
        session.authority.facts.types.len(),
        work.retained,
        work.peak,
    );
    let before = work.steps;
    for _ in 0..1000 {
        session.require(&mut work)?;
    }
    let delta = work.steps - before;
    assert_eq!(
        state,
        (
            session.authority.facts.types.len(),
            work.retained,
            work.peak
        )
    );
    eprintln!(
        "consumer-warm unrelated={count} steps={delta} retained={} peak={}",
        work.retained, work.peak
    );
    Ok(delta)
}
#[test]
fn fresh_consumer_bounds_single_reservation_warm_work_and_failed_initialization() -> Result {
    unpaid_initialization_opponents()?;
    let mut warm = None;
    for count in [1, 8, 32] {
        let delta = warm_reservation(count)?;
        if let Some(previous) = warm {
            assert_eq!(delta, previous);
        }
        warm = Some(delta);
    }
    let actual = fixture(HOLDER)?;
    let mut storage = ReservationWork {
        retained: crate::support::crates::MAX_BYTES,
        ..ReservationWork::default()
    };
    assert!(
        construct(
            input(MAIN)?,
            &mut NoFiles,
            &Config::default(),
            &actual.context,
            actual.admission,
            &mut storage
        )
        .is_err()
    );
    assert!(storage.peak > crate::support::crates::MAX_BYTES && storage.steps > 0);
    let actual = fixture(HOLDER)?;
    let mut steps = ReservationWork {
        steps: crate::support::crates::MAX_COMPONENTS,
        ..ReservationWork::default()
    };
    assert!(
        construct(
            input(MAIN)?,
            &mut NoFiles,
            &Config::default(),
            &actual.context,
            actual.admission,
            &mut steps
        )
        .is_err()
    );
    assert!(steps.steps > crate::support::crates::MAX_COMPONENTS);
    Ok(())
}
