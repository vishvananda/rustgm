//! Only genuine source collection binds completion in a new actual consumer.
use super::*;
use crate::{
    resolve::crates::ConsumerRequest, sema::crates::authored_export::CollectedArtifact,
    support::crates::Interface,
};

fn artifact(suffix: &str, work: &mut ReservationWork) -> Result<(CollectedArtifact, Identity)> {
    artifact_with_alias(suffix, None, work)
}
fn artifact_with_alias(
    suffix: &str,
    alias: Option<&str>,
    work: &mut ReservationWork,
) -> Result<(CollectedArtifact, Identity)> {
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    source.push_str(suffix);
    let report = crate::test_source::source_provider(&source, "collected_consumer")?;
    let provider = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let inspection = crate::lower::instances::inspect_source_trait_inputs_with_numbering(
        &report,
        &mut analysis.facts,
        &provider,
        analysis.prepared,
    )?;
    let arena = analysis.facts.types.arena_identity();
    let alias = alias
        .map(|name| crate::support::crates::Dependency::new(name.to_owned(), provider.clone()))
        .transpose()?;
    let base = Interface::new(provider, Vec::new(), Vec::new())?;
    let artifact = if let Some(alias) = alias {
        inspection.into_named_artifact(base, alias, work)?
    } else {
        inspection.into_artifact(base, work)?
    };
    // Producer Report/facts retire here, before the current consumer exists.
    Ok((artifact, arena))
}
#[test]
fn named_collected_artifact_resolves_actual_qualified_calls_before_semantic_preparation() -> Result
{
    let mut work = ReservationWork::default();
    let (artifact, arena) = artifact_with_alias(HOLDER, Some("base"), &mut work)?;
    let context = context(artifact.inventory())?;
    let config = Config::default();
    let mut sources = NoFiles;
    let text = "#![feature(no_core)]#![no_core]fn main(){let value=<u8 as base::Holder<u8>>::carry::<u8>(7u8);let borrowed=<u8 as base::Holder<u8>>::borrow(&value);let _=*borrowed;}";
    let request = ConsumerRequest::new(input(text)?, &mut sources, &config, &context);
    let mut stage = ProofSession::from_collected(artifact, request, &mut work)?;
    let authority = &stage.session.authority;
    let view = authority
        .reservation
        .recipe_view(&authority.report, &mut work)?;
    let holder_declaration = named_trait(&stage, "Holder", &mut work)?;
    let definition = view.definition(holder_declaration, &mut work)?;
    assert!(authority.report.names_resolved);
    assert!(authority.report.resolution.extern_scope.is_some());
    assert_eq!(authority.report.resolution.qualified_paths.len(), 2);
    assert!(
        authority
            .report
            .resolution
            .references
            .iter()
            .any(|reference| reference.target
                == crate::resolve::model::Target::Definition(definition))
    );
    assert!(!authority.facts.types.has_identity(&arena));
    assert!(authority.facts.definitions.iter().all(Option::is_none));
    assert!(authority.facts.generics.traits.is_none());
    assert!(authority.facts.functions.is_empty());
    let carry = named(&stage, "carry", &mut work)?;
    let (returned, status) = stage.callable_template(carry, &mut work)?;
    stage = returned;
    assert_eq!(status, Demand::Ready(()));
    assert!(stage.session.authority.facts.generics.traits.is_none());
    assert!(stage.session.authority.facts.functions.is_empty());
    stage.session.authority.require(&mut work)?;
    let (returned, status) = stage.schema(holder_declaration, &mut work)?;
    stage = returned;
    assert_eq!(status, Demand::Ready(()));
    assert!(
        stage
            .session
            .authority
            .facts
            .associated
            .owner(definition)
            .is_none()
    );
    assert!(stage.session.authority.facts.generics.traits.is_none());
    stage = repeated_schema(stage, holder_declaration, &mut work)?;
    let (returned, status) = stage.trait_declaration(holder_declaration, &mut work)?;
    stage = returned;
    assert_eq!(status, Demand::Ready(()));
    assert!(
        stage
            .session
            .authority
            .facts
            .generics
            .traits
            .as_deref()
            .and_then(|contracts| contracts.header(crate::types::ItemId(definition.0)))
            .is_some()
    );
    assert!(
        stage
            .session
            .authority
            .facts
            .associated
            .owner(definition)
            .is_none()
    );
    let before = work.retained;
    let (returned, status) = stage.trait_declaration(holder_declaration, &mut work)?;
    stage = returned;
    assert_eq!(status, Demand::Ready(()));
    assert_eq!(work.retained, before);
    let (mut stage, status) = stage.trait_owner(holder_declaration, &mut work)?;
    assert_eq!(status, Demand::Ready(()));
    stage = repeated_owner(stage, holder_declaration, definition, &mut work)?;
    method_headers(stage, &mut work)?;
    owner_failed_precharge()?;
    actual_implicit_prelude_controls()?;
    Ok(())
}

fn repeated_owner(
    mut stage: Box<HeaderDraft>,
    owner: DeclarationRef,
    definition: crate::resolve::model::DefId,
    work: &mut ReservationWork,
) -> Result<Box<HeaderDraft>> {
    let facts = &stage.session.authority.facts;
    let id = facts
        .associated
        .owner(definition)
        .ok_or("actual selected installed owner")?;
    let actual = *facts
        .associated
        .get(id)
        .ok_or("actual selected owner cell")?;
    let members = facts
        .associated
        .owner_members(id)
        .ok_or("actual selected members")?;
    assert_eq!(members.len(), 2);
    facts.associated.require_imported(actual)?;
    assert!(
        facts
            .associated
            .imported_owner()
            .ok_or("actual mapping")?
            .require_store(&crate::types::Store::default())
            .is_err()
    );
    let before = work.retained;
    let count = facts.associated.owners().len();
    for _ in 0..1000 {
        let (returned, status) = stage.trait_owner(owner, work)?;
        stage = returned;
        assert_eq!(status, Demand::Ready(()));
    }
    assert_eq!(work.retained, before);
    let facts = &stage.session.authority.facts;
    assert_eq!(facts.associated.owners().len(), count);
    assert_eq!(facts.associated.owner(definition), Some(id));
    assert!(facts.functions.is_empty());
    eprintln!(
        "selected-owner sum={} language-field={} selected={} inventory={} language={} draft={} warm1000-retained={} work={}",
        size_of::<crate::sema::crates::ImportedOwners>(),
        size_of::<Option<crate::sema::crates::ImportedOwners>>(),
        size_of::<crate::sema::crates::typed_headers::unpaid_provided::ProvidedOwners>(),
        size_of::<crate::sema::associated::Inventory>(),
        crate::sema::associated::imported_layouts().0,
        size_of::<HeaderDraft>(),
        work.retained.saturating_sub(before),
        work.steps
    );
    Ok(stage)
}

fn method_headers(mut stage: Box<HeaderDraft>, work: &mut ReservationWork) -> Result {
    for name in ["carry", "borrow"] {
        let owner = named(&stage, name, work)?;
        let definition = stage
            .session
            .authority
            .reservation
            .recipe_view(&stage.session.authority.report, work)?
            .definition(owner, work)?;
        let (returned, status) = stage.method_declaration(owner, work)?;
        stage = returned;
        assert_eq!(status, Demand::Ready(()));
        let facts = &stage.session.authority.facts;
        let linkage = facts.crates.as_deref().ok_or("actual method linkage")?;
        let declaration = linkage
            .original_declaration_header(definition)?
            .ok_or("actual selected declaration header")?;
        assert!(declaration.body.is_some());
        assert!(
            linkage.original_header(definition).is_err(),
            "body Environment remains genuinely pending"
        );
        linkage.original_inputs(&facts.generics, definition, false)?;
        let template = facts
            .lifetimes
            .callables
            .registration(crate::types::ItemId(definition.0))
            .ok_or("actual registered callable")?;
        assert_eq!(
            template.1.len(),
            if name == "carry" { 3 } else { 2 },
            "actual Self/V and own U remain complete"
        );
        let holder = named_trait(&stage, "Holder", work)?;
        let holder = stage
            .session
            .authority
            .reservation
            .recipe_view(&stage.session.authority.report, work)?
            .definition(holder, work)?;
        let inputs = linkage.original_inputs(&facts.generics, holder, false)?;
        let header = facts
            .generics
            .traits
            .as_deref()
            .and_then(|contracts| contracts.header(crate::types::ItemId(holder.0)))
            .ok_or("actual Holder input owner")?;
        let (all, _) = header.inputs.imported_parts(
            linkage.origin_catalog().ok_or("actual current catalog")?,
            holder,
        )?;
        assert!(std::ptr::eq(all.as_ref(), inputs));
        let before = work.retained;
        for _ in 0..1000 {
            let (returned, status) = stage.method_declaration(owner, work)?;
            stage = returned;
            assert_eq!(status, Demand::Ready(()));
        }
        assert_eq!(work.retained, before);
    }
    eprintln!(
        "method-header layout={} headers={} pending-env={} selected-owner={}",
        size_of::<crate::sema::crates::typed_headers::Header>(),
        size_of::<crate::sema::crates::typed_headers::Headers>(),
        size_of::<Option<Arc<crate::types::parameters::environment::Environment>>>(),
        size_of::<crate::sema::crates::typed_headers::unpaid_provided::ProvidedOwners>()
    );
    Ok(())
}

fn owner_failed_precharge() -> Result {
    let mut work = ReservationWork::default();
    let (stage, _) = start(HOLDER, &mut work)?;
    let owner = named_trait(&stage, "Holder", &mut work)?;
    let (stage, status) = stage.trait_declaration(owner, &mut work)?;
    assert_eq!(status, Demand::Ready(()));
    work.retained = crate::support::crates::MAX_BYTES;
    let before = work.steps;
    assert!(stage.trait_owner(owner, &mut work).is_err());
    assert!(work.steps > before);
    assert!(work.peak > crate::support::crates::MAX_BYTES);
    Ok(())
}

fn repeated_schema(
    mut stage: Box<HeaderDraft>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Box<HeaderDraft>> {
    let before = (
        work.retained,
        stage
            .session
            .authority
            .facts
            .types
            .node_table_allocation()
            .bytes,
    );
    let steps = work.steps;
    for _ in 0..1000 {
        let (returned, status) = stage.schema(owner, work)?;
        stage = returned;
        assert_eq!(status, Demand::Ready(()));
    }
    assert_eq!(
        before,
        (
            work.retained,
            stage
                .session
                .authority
                .facts
                .types
                .node_table_allocation()
                .bytes
        )
    );
    assert!(stage.session.authority.facts.generics.traits.is_none());
    eprintln!(
        "schema actual warm1000={} reserved={} draft={} selected={}",
        work.steps - steps,
        work.retained,
        size_of::<HeaderDraft>(),
        size_of::<SelectedHeaders>()
    );
    Ok(stage)
}
fn actual_implicit_prelude_controls() -> Result {
    for (declarations, body, accepted) in [
        (
            "",
            "mod inner{pub fn f(){let _=<u8 as base::Holder<u8>>::carry::<u8>(7u8);}}fn main(){}",
            true,
        ),
        (
            "#![no_implicit_prelude]",
            "fn main(){let _=<u8 as base::Holder<u8>>::carry::<u8>(7u8);}",
            false,
        ),
        (
            "#![no_implicit_prelude]extern crate base;",
            "fn main(){let _=<u8 as base::Holder<u8>>::carry::<u8>(7u8);}",
            true,
        ),
        (
            "#![no_implicit_prelude]",
            "mod inner{pub fn f(){let _=<u8 as base::Holder<u8>>::carry::<u8>(7u8);}}fn main(){}",
            false,
        ),
        (
            "",
            "mod inner{#![no_implicit_prelude]pub fn f(){let _=<u8 as base::Holder<u8>>::carry::<u8>(7u8);}}fn main(){}",
            false,
        ),
    ] {
        let mut work = ReservationWork::default();
        let (artifact, _) = artifact_with_alias(HOLDER, Some("base"), &mut work)?;
        let context = context(artifact.inventory())?;
        let config = Config::default();
        let mut sources = NoFiles;
        let text = format!("#![feature(no_core)]#![no_core]{declarations}{body}");
        let request = ConsumerRequest::new(input(&text)?, &mut sources, &config, &context);
        let result = ProofSession::from_collected(artifact, request, &mut work);
        if accepted {
            let stage = result?;
            stage.session.authority.require(&mut work)?;
            assert!(stage.session.authority.facts.functions.is_empty());
        } else {
            let error = result
                .err()
                .ok_or("no_implicit_prelude unexpectedly exposed an alias")?;
            assert_eq!(error.message, "unresolved type name");
            assert!(work.steps > 0);
        }
    }
    Ok(())
}
fn context(inventory: &Arc<crate::support::crates::Dependencies>) -> Result<Context> {
    Ok(Context {
        crate_name: Some("consumer".into()),
        current: Some(CrateIdentity::new("consumer", "collected_current")?),
        dependencies: Some(Arc::clone(inventory)),
        mode: Mode::SourceClient {
            provider: inventory.artifacts()[0].identity().clone(),
        },
    })
}
fn start(suffix: &str, work: &mut ReservationWork) -> Result<(Box<HeaderDraft>, Identity)> {
    let (artifact, arena) = artifact(suffix, work)?;
    let context = context(artifact.inventory())?;
    let config = Config::default();
    let mut sources = NoFiles;
    let request = ConsumerRequest::new(input(MAIN)?, &mut sources, &config, &context);
    Ok((
        ProofSession::from_collected(artifact, request, work)?,
        arena,
    ))
}
fn named(stage: &HeaderDraft, name: &str, work: &mut ReservationWork) -> Result<DeclarationRef> {
    let authority = &stage.session.authority;
    let view = authority.reservation.recipe_view(&authority.report, work)?;
    let rows = view.rows()?;
    rows.functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .map(|header| header.declaration)
        .ok_or_else(|| "actual collected function".into())
}

fn named_trait(
    stage: &HeaderDraft,
    name: &str,
    work: &mut ReservationWork,
) -> Result<DeclarationRef> {
    let authority = &stage.session.authority;
    let view = authority.reservation.recipe_view(&authority.report, work)?;
    let rows = view.rows()?;
    rows.traits
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .map(|header| header.declaration)
        .ok_or_else(|| "actual collected trait".into())
}

fn check_member_contracts(
    rows: crate::support::crates::generics::source::ValidatedView<'_>,
    columns: &crate::support::crates::generics::source::HeaderAttributeColumns,
) -> Result {
    use crate::support::crates::generics::source as s;
    let member = rows
        .roles
        .iter()
        .find(|row| row.role == LangItem::CloneFn)
        .ok_or("actual clone_fn role")?
        .declaration;
    assert_eq!(member.artifact.0, 0);
    let mut counts = (0, 0);
    for (header, contract) in rows.functions.iter().zip(&columns.functions) {
        assert_eq!(header.declaration, contract.declaration);
        let name = rows.declarations[header.declaration.0 as usize]
            .name
            .as_deref();
        if header.declaration == member.declaration {
            assert_eq!(name, Some("clone"));
            assert_eq!(header.body, s::FunctionBody::Absent);
            assert!(
                contract
                    .must_use
                    .as_ref()
                    .and_then(|value| value.reason.as_deref())
                    .is_some_and(|reason| reason.contains("cloning"))
            );
            counts.0 += 1;
        } else if name == Some("clone") {
            let link = rows
                .member_links
                .iter()
                .find(|link| link.implementation == Some(header.declaration))
                .ok_or("actual clone implementation member link")?;
            assert_eq!(link.trait_member, Some(member));
            assert!(
                contract.must_use.is_none(),
                "an impl method has its own actual contract"
            );
            assert!(matches!(header.body, s::FunctionBody::Original(_)));
            let implementation = rows
                .implementations
                .iter()
                .find(|implementation| {
                    implementation
                        .members
                        .slice(&rows.member_links)
                        .is_ok_and(|links| links.contains(link))
                })
                .ok_or("actual clone implementation owner")?;
            assert!(
                !implementation.constant,
                "the actual u8 Clone implementation is ordinary"
            );
            counts.1 += 1;
        } else if matches!(name, Some("clone_from" | "identity")) {
            assert!(contract.inline.is_some());
        }
    }
    assert_eq!(counts, (1, 1));
    Ok(())
}

#[test]
fn collected_source_headers_bind_full_family_to_fresh_actual_consumer() -> Result {
    let mut work = ReservationWork::default();
    let (mut stage, arena) = start(HOLDER, &mut work)?;
    let authority = &stage.session.authority;
    assert!(!authority.facts.types.has_identity(&arena));
    assert!(authority.facts.definitions.iter().all(Option::is_none));
    let receipt = stage
        .completion
        .as_deref()
        .ok_or("actual moved completion")?;
    let columns = receipt.columns(&authority.report, &mut work)?;
    let view = authority
        .reservation
        .recipe_view(&authority.report, &mut work)?;
    let rows = view.rows()?;
    assert_eq!(
        (columns.functions.len(), columns.traits.len()),
        (rows.functions.len(), rows.traits.len())
    );
    assert_eq!(rows.roles.len(), 9);
    check_member_contracts(rows, columns)?;
    let carry = named(&stage, "carry", &mut work)?;
    let (next, status) = stage.callable_template(carry, &mut work)?;
    stage = next;
    assert_eq!(status, Demand::Ready(()));
    let authority = &stage.session.authority;
    let role = authority
        .facts
        .source_bindings
        .as_deref()
        .ok_or("actual roles")?
        .role(LangItem::Clone)?
        .ok_or("actual Clone")?;
    assert!(authority.facts.associated.owner(role.definition).is_none());
    assert!(
        authority
            .facts
            .header_attribute_contracts(&authority.report, role.definition)
            .is_err()
    );
    assert!(
        authority
            .facts
            .source_bindings
            .as_deref()
            .ok_or("roles")?
            .validated_role(LangItem::Clone)
            .is_err()
    );
    eprintln!(
        "collected-consumer draft={} receipt={} artifact={} source-policy={:?} retained={} peak={} steps={}",
        size_of::<HeaderDraft>(),
        size_of::<crate::sema::crates::authored_export::CollectedReceipt>(),
        size_of::<CollectedArtifact>(),
        crate::resolve::bindings::consumer_policy::layouts(),
        work.retained,
        work.peak,
        work.steps
    );
    Ok(())
}

fn swapped_columns(clear: bool) -> Result {
    let mut work = ReservationWork::default();
    let (artifact, _) = artifact(HOLDER, &mut work)?;
    let context = context(artifact.inventory())?;
    let inventory = Arc::clone(artifact.inventory());
    let (admission, receipt) = artifact.into_admission(&mut work)?;
    let original = construct(
        input(MAIN)?,
        &mut NoFiles,
        &Config::default(),
        &context,
        admission,
        &mut work,
    )?;
    let mut columns = original
        .authority
        .report
        .package
        .staged_consumer()
        .ok_or("actual policy")?
        .unpaid_header_columns()
        .clone();
    if clear {
        for row in &mut columns.functions {
            row.must_use = None;
        }
    }
    let other = ProviderAdmission::from_data(inventory, 0, columns, &mut work)?;
    let changed = construct(
        input(MAIN)?,
        &mut NoFiles,
        &Config::default(),
        &context,
        other,
        &mut work,
    )?
    .selected_headers(&mut work)?;
    let error = changed
        .source_headers(receipt, &mut work)
        .err()
        .ok_or("swapped sidecar admitted")?;
    assert!(
        error.message.contains("actual header-data allocation"),
        "{error}"
    );
    original.require(&mut work)?;
    Ok(())
}
#[test]
fn collected_source_headers_reject_unpaid_swapped_and_foreign_sidecars() -> Result {
    swapped_columns(false)?;
    swapped_columns(true)?;
    let mut work = ReservationWork::default();
    let (artifact, _) = artifact(HOLDER, &mut work)?;
    let (_, foreign) = artifact.into_admission(&mut work)?;
    let (stage, _) = start(HOLDER, &mut work)?;
    let error = foreign
        .columns(&stage.session.authority.report, &mut work)
        .err()
        .ok_or("foreign inventory admitted")?;
    assert!(error.message.contains("another actual consumer inventory"));
    let actual = fixture(HOLDER)?;
    let unpaid = construct(
        input(MAIN)?,
        &mut NoFiles,
        &Config::default(),
        &actual.context,
        actual.admission,
        &mut work,
    )?
    .selected_headers(&mut work)?;
    assert!(unpaid.completion.is_none());
    Ok(())
}

fn warm(count: usize) -> Result<usize> {
    let mut source = HOLDER.to_owned();
    for index in 0..count {
        write!(source, "pub fn unrelated_{index}<T>(value:T)->T{{value}}")?;
    }
    let mut work = ReservationWork::default();
    let (stage, _) = start(&source, &mut work)?;
    let owner = named(&stage, "carry", &mut work)?;
    let (mut stage, status) = stage.callable_template(owner, &mut work)?;
    assert_eq!(status, Demand::Ready(()));
    let before = (
        work.retained,
        work.peak,
        stage.session.authority.facts.types.interned_extent(),
    );
    let steps = work.steps;
    for _ in 0..1000 {
        let (next, status) = stage.callable_template(owner, &mut work)?;
        assert_eq!(status, Demand::Ready(()));
        stage = next;
    }
    assert_eq!(
        before,
        (
            work.retained,
            work.peak,
            stage.session.authority.facts.types.interned_extent()
        )
    );
    let delta = work.steps - steps;
    eprintln!(
        "collected-consumer unrelated={count} warm1000={delta} retained={} peak={}",
        work.retained, work.peak
    );
    Ok(delta)
}
#[test]
fn collected_source_headers_bound_failed_work_and_warm_storage() -> Result {
    let mut previous = None;
    for count in [1, 8, 32] {
        let steps = warm(count)?;
        if let Some(previous) = previous {
            assert_eq!(steps, previous);
        }
        previous = Some(steps);
    }
    let mut work = ReservationWork {
        retained: crate::support::crates::MAX_BYTES - 16,
        ..ReservationWork::default()
    };
    assert!(artifact(HOLDER, &mut work).is_err());
    assert!(work.steps > 0 && work.peak > crate::support::crates::MAX_BYTES);
    Ok(())
}
