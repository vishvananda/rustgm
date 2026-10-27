//! A declaration is neither manual call permission nor a reusable foreign epoch.
use super::*;
use crate::{
    support::crates::{self, CrateIdentity, Dependencies, Dependency},
    types::{TypeId, arguments::Argument, instance::Instance, substitute::Bound},
};
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn provider() -> Result<Vec<u8>> {
    provider_source("pub const fn id<T:Copy>(value:T)->T{value}")
}
fn provider_source(source: &str) -> Result<Vec<u8>> {
    let report = crate::test_source::resolve(source, true)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let (_, interface) = crate::lower::instances::build_library_prepared(
        &report,
        &mut facts,
        CrateIdentity::new("provider", "manual")?,
        prepared,
    )?;
    Ok(crates::encode(&interface)?)
}
struct NoFiles;
impl crate::resolve::load::Sources for NoFiles {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> std::result::Result<Option<crate::resolve::load::InputFile>, String> {
        Ok(None)
    }
}
fn client(bytes: &[u8]) -> Result<(Report, crate::pipeline::Analysis)> {
    client_source(bytes, "struct Token(u8);fn main(){}")
}
fn client_source(bytes: &[u8], text: &str) -> Result<(Report, crate::pipeline::Analysis)> {
    let interface = crates::decode(bytes)?;
    let identity = interface.identity().clone();
    let dependencies = Arc::new(Dependencies::new(
        vec![interface],
        vec![Dependency::new("base".into(), identity)?],
    )?);
    let source = crate::support::source::SourceFile::new(text.as_bytes().to_vec())?;
    let report = crate::resolve::expansion::analyze_with_crates(
        crate::resolve::load::InputFile {
            identity: "client.rs".into(),
            path: "client.rs".into(),
            source,
        },
        &mut NoFiles,
        &crate::expand::config::Config::default(),
        None,
        Some(dependencies),
    )
    .and_then(crate::resolve::analyze_expanded)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn instance(facts: &mut Declarations, definition: DefId, ty: TypeId) -> Result<Instance> {
    instance_arguments(facts, definition, &[ty])
}
fn instance_arguments(
    facts: &mut Declarations,
    definition: DefId,
    types: &[TypeId],
) -> Result<Instance> {
    let item = ItemId(definition.0);
    let signature = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("missing imported callable")?
        .signature;
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(item)
        .ok_or("missing actual imported formals")?;
    if parameters.len() != types.len() {
        return Err("wrong actual imported parameter inventory".into());
    }
    let binding = parameters
        .iter()
        .copied()
        .zip(types.iter().copied().map(Argument::Type))
        .collect::<Vec<_>>();
    let mut fold = Bound::parameters(&facts.types, &binding)?;
    let signature = fold.apply(&mut facts.types, signature)?;
    let arguments = facts.types.intern_arguments(
        &types
            .iter()
            .copied()
            .map(Argument::Type)
            .collect::<Vec<_>>(),
    )?;
    let target = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments,
    })?;
    Ok(Instance::new(&facts.types, target)?)
}
#[test]
fn manual_imported_instances_pay_debts_and_warm_code_rejects_foreign_lineage() -> Result {
    let bytes = provider()?;
    let (report, mut analysis) = client(&bytes)?;
    let facts = &mut analysis.facts;
    let imported = report.resolution.crates.as_ref().ok_or("missing imports")?;
    let external = crates::ExternalDef::new(
        CrateIdentity::new("provider", "manual")?,
        vec!["id".into()],
        crates::ExternalKind::Function,
    )?;
    let definition = imported
        .definition(&external)
        .ok_or("missing real imported function")?;
    let token = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| {
            report.package.names.get(d.name) == "Token"
                && d.kind == crate::resolve::model::DefKind::Struct
        })
        .map(|(i, _)| i)
        .ok_or("missing Token")?;
    let token = facts.definitions[token].ok_or("missing Token type")?;
    let paid = instance(
        facts,
        definition,
        TypeId::primitive(crate::types::Primitive::U8),
    )?;
    let unpaid = instance(facts, definition, token)?;
    let mut engine = Engine::default();
    assert!(engine.function(&report, facts, paid)?);
    assert!(engine.function(&report, facts, paid)?);
    assert!(
        engine.function(&report, facts, unpaid).is_err(),
        "manual selection used metadata declaration as Copy evidence"
    );
    assert!(!engine.catalog.functions.contains_key(&unpaid));
    let (_, mut foreign) = client(&bytes)?;
    facts.crates = foreign.facts.crates.take();
    assert!(
        engine.function(&report, facts, paid).is_err(),
        "warm Code accepted another catalog's coincident declaration IDs"
    );
    assert!(
        engine.catalog.functions.contains_key(&paid),
        "rejection need not destroy previously valid materialized Code"
    );
    Ok(())
}
#[test]
fn imported_owner_storage_is_cold_and_preserves_hot_layouts() {
    let engine = Engine::default();
    let prepared = crate::pipeline::Prepared::default();
    assert!(engine.imported.is_none());
    assert!(prepared.imported.is_none());
    let templates = body::templates::Templates::source(Vec::new());
    assert_eq!(templates.len(), 0);
    assert_eq!(templates.bodies.capacity(), 0);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    assert_eq!(size_of::<body::ir::ProofSite>(), 12);
    eprintln!(
        "ImportedOriginals={} Engine={} Prepared={} Templates={} Body={} Contract={} Code={} Value={}",
        size_of::<body::transported_originals::ImportedOriginals>(),
        size_of::<Engine>(),
        size_of::<crate::pipeline::Prepared>(),
        size_of::<body::templates::Templates>(),
        size_of::<Body>(),
        size_of::<Contract>(),
        size_of::<crate::const_eval::code::Code>(),
        size_of::<body::ir::Value>()
    );
}

#[test]
fn operation_derived_coercion_rejects_an_unused_forged_generic_original() -> Result {
    let bytes = provider()?;
    let interface = crates::decode(&bytes)?;
    let payload = interface
        .generic_payload()
        .ok_or("missing real provider payload")?;
    let mut rows = payload.legacy_rows()?.clone();
    let unit = if let Some(index) = rows
        .types
        .iter()
        .position(|ty| *ty == crates::generics::Type::Scalar(crates::Scalar::Unit))
    {
        crates::generics::TypeRef(u32::try_from(index)?)
    } else {
        let index = crates::generics::TypeRef(u32::try_from(rows.types.len())?);
        rows.types
            .push(crates::generics::Type::Scalar(crates::Scalar::Unit));
        index
    };
    let body = &mut rows.originals[0];
    let value = u32::try_from(body.values.len())?;
    body.values.push(crates::generics::Value {
        ty: unit,
        kind: crates::generics::ValueKind::Unit,
    });
    body.values.push(crates::generics::Value {
        ty: body.result,
        kind: crates::generics::ValueKind::Constrain(value),
    });
    body.blocks[0].terminator = crates::generics::Terminator::Return(value + 1);
    let mut work = crates::generics::Work::default();
    // Structural metadata remains valid. The importer must independently derive
    // and prove Unit -> T; no encoded Contract exists from which to omit it.
    let payload = crates::generics::Payload::new(interface.identity(), rows, &mut work)?;
    let forged = crates::Interface::new(interface.identity().clone(), vec![], vec![])?
        .with_generic_payload(payload)?;
    let bytes = crates::encode(&forged)?;
    let Err(error) = client(&bytes) else {
        return Err("unused forged generic coercion was accepted".into());
    };
    let diagnostic = error
        .downcast_ref::<Error>()
        .ok_or("forged operation did not reach the frontend proof diagnostic")?;
    assert!(
        matches!(
            &diagnostic.file,
            crate::resolve::package::DiagnosticFile::Imported(_)
        ),
        "{diagnostic:?}"
    );
    assert_eq!(diagnostic.message, "invalid lifetime constraint identity");
    Ok(())
}

#[test]
fn imported_header_and_original_work_share_graphs_across_actual_uses() -> Result {
    let mut widths = Vec::new();
    for width in [8, 32, 128] {
        let parameters = (0..width)
            .map(|i| {
                if i == 0 {
                    "T0:Copy".into()
                } else {
                    format!("T{i}")
                }
            })
            .collect::<Vec<String>>()
            .join(",");
        let bytes = provider_source(&format!(
            "pub const fn id<{parameters}>(value:T0)->T0{{value}}"
        ))?;
        let arguments = vec!["u8"; width].join(",");
        let mut first = None;
        for uses in [1, 8, 32] {
            let calls = format!("let _=base::id::<{arguments}>(1u8);").repeat(uses);
            let (_, analysis) = client_source(&bytes, &format!("fn main(){{{calls}}}"))?;
            let observed = analysis
                .facts
                .crates
                .as_deref()
                .and_then(crate::sema::crates::Linkage::original_work)
                .ok_or("missing header work")?;
            let imported = analysis
                .prepared
                .imported
                .as_ref()
                .ok_or("missing shared originals")?;
            let counts = (observed, imported.steps(), imported.retained_bytes());
            if let Some(first) = first {
                assert_eq!(counts, first, "import preparation repeated for actual uses");
            } else {
                first = Some(counts);
            }
            assert!(observed.2 <= observed.1);
            assert!(observed.0 <= width * 40 + 1000);
            assert!(imported.steps() <= width * 40 + 1000);
            let original = imported.at(0).ok_or("missing actual original")?;
            let mut templates = body::templates::Templates::source(Vec::new());
            templates.attach(Some(std::rc::Rc::clone(imported)))?;
            assert!(std::ptr::eq(
                &raw const templates.get(0).ok_or("missing shared template")?.body,
                &raw const original.template().body,
            ));
        }
        let counts = first.ok_or("empty use axis")?;
        eprintln!(
            "width={width} header={:?} original_steps={} original_bytes={}",
            counts.0, counts.1, counts.2
        );
        widths.push(counts);
    }
    assert!(widths.windows(2).all(|pair| pair[1].0.0 > pair[0].0.0
        && pair[1].0.0 <= pair[0].0.0 * 4
        && pair[1].1 >= pair[0].1
        && pair[1].1 <= pair[0].1 * 4));
    Ok(())
}

#[test]
fn imported_declaration_growth_reserves_only_its_owned_header_graphs() -> Result {
    let mut previous = None;
    for count in [8, 32, 128] {
        let mut source = String::new();
        for i in 0..count {
            write!(&mut source, "pub const fn id{i:08}<T:Copy>(x:T)->T{{x}}")?;
        }
        let (report, mut analysis) = client(&provider_source(&source)?)?;
        let headers = analysis
            .facts
            .crates
            .as_deref()
            .and_then(crate::sema::crates::Linkage::original_work)
            .ok_or("missing actual header accounting")?;
        let originals = analysis
            .prepared
            .imported
            .as_ref()
            .ok_or("missing originals")?;
        assert_eq!(originals.len(), count);
        assert!(headers.2 <= headers.1);
        assert!(headers.1 < count * 16_384);
        if let Some((work, bytes, steps)) = previous {
            assert!(headers.0 > work && headers.0 <= work * 4);
            assert!(headers.1 > bytes && headers.1 <= bytes * 4);
            assert!(originals.steps() > steps && originals.steps() <= steps * 4);
        }
        eprintln!(
            "declarations={count} header={headers:?} original_steps={} original_bytes={}",
            originals.steps(),
            originals.retained_bytes()
        );
        previous = Some((headers.0, headers.1, originals.steps()));
        analysis.facts.constant_work.template_bytes = 64 * 1024 * 1024 - 1;
        let steps = analysis.facts.body_work.canonical_steps;
        // Aggregate quota failure happens before any body or Rc allocation.
        let result = body::transported_originals::build(
            &report,
            &mut analysis.facts,
            &mut body::promote::Scratch::default(),
        );
        let Err(error) = result else {
            return Err("template quota was bypassed".into());
        };
        assert_eq!(
            error.message,
            "imported original construction storage budget exceeded"
        );
        assert!(analysis.facts.body_work.canonical_steps > steps);
    }
    Ok(())
}

#[test]
fn unused_formal_sizedness_is_paid_per_instance_and_relaxation_is_preserved() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa29/tests/spec/generic-contracts");
    for (provider, relaxed) in [
        ("minimal-unused-library", false),
        ("minimal-unsized-library", true),
    ] {
        let source = std::fs::read_to_string(root.join(format!("{provider}.rs")))?;
        let bytes = provider_source(&source)?;
        let (report, mut analysis) = client(&bytes)?;
        let facts = &mut analysis.facts;
        let definition = report
            .resolution
            .crates
            .as_ref()
            .ok_or("missing imports")?
            .definitions()
            .find(|(external, _)| external.path().last().is_some_and(|s| s == "id"))
            .map(|(_, definition)| definition)
            .ok_or("missing imported id")?;
        let scalar = TypeId::primitive(crate::types::Primitive::U8);
        let slice = facts.types.intern(Kind::Slice(scalar))?;
        let paid = instance_arguments(facts, definition, &[scalar, scalar])?;
        let selected = instance_arguments(facts, definition, &[scalar, slice])?;
        let mut engine = Engine::default();
        assert!(engine.function(&report, facts, paid)?);
        assert!(engine.function(&report, facts, paid)?);
        let result = engine.function(&report, facts, selected);
        if relaxed {
            assert!(result?);
            assert!(engine.function(&report, facts, selected)?);
        } else {
            assert!(
                result.is_err(),
                "unused Sized parameter was admitted as an assumption"
            );
            assert!(!engine.catalog.functions.contains_key(&selected));
            assert!(engine.function(&report, facts, selected).is_err());
        }
    }
    Ok(())
}

#[path = "application_debts.rs"]
mod calls;
