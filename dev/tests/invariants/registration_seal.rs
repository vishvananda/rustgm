//! Reserved/validated headers cannot authorize Copy or warm-code CTFE publication.
use super::*;
use crate::resolve::{
    bindings::Context,
    load::{InputFile, Sources},
};
use crate::support::{bindings::Mode, crates::CrateIdentity, source::SourceFile};
use crate::{body, const_eval, expand::config::Config, resolve, sema};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = r#"#![feature(no_core,lang_items,const_trait_impl)]#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
const fn seven()->u8{7}const fn answer()->u8{seven()}
"#;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report_source(source: &str) -> Result<resolve::expansion::Report> {
    let identity = CrateIdentity::new("minicore", "destructor_readiness")?;
    Ok(resolve::analyze_expanded(
        resolve::expansion::analyze_with_context(
            InputFile {
                identity: "provider.rs".into(),
                path: "provider.rs".into(),
                source: SourceFile::new(source.as_bytes().to_vec())?,
            },
            &mut NoFiles,
            &Config::default(),
            &Context {
                crate_name: Some("minicore".into()),
                current: Some(identity.clone()),
                mode: Mode::Source { provider: identity },
                ..Default::default()
            },
        )?,
    )?)
}
fn report() -> Result<resolve::expansion::Report> {
    report_source(SOURCE)
}
#[test]
fn source_drop_headers_and_registration_are_distinct_states() -> Result {
    let report = report()?;
    let reserved = Declarations::new(&report)?;
    let catalog = reserved.source_bindings.as_deref().ok_or("catalog")?;
    assert!(catalog.language(LangItem::Drop).is_some());
    assert_eq!(
        catalog.require_source_destructors_prepared(),
        Err("source trait role requires completed declaration validation")
    );
    let facts = sema::declarations::analyze(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    assert!(catalog.validated_language(LangItem::Drop)?.is_some());
    assert_eq!(
        catalog.require_source_destructors_prepared(),
        Err("source destructor registration is not implemented")
    );
    println!(
        "Validated={} SourceCatalog={} Reservations={} Declarations={} Resolution={}",
        size_of::<super::super::Validated>(),
        size_of::<Catalog>(),
        size_of::<resolve::bindings::Reservations>(),
        size_of::<Declarations>(),
        size_of::<resolve::model::Resolution>()
    );
    Ok(())
}
#[test]
fn source_drop_unprepared_registration_blocks_copy_and_cold_or_warm_ctfe() -> Result {
    for warm in [false, true] {
        let report = report()?;
        let facts = sema::declarations::analyze(&report)?;
        let mut facts = sema::bodies::analyze(&report, facts)?;
        let mut templates = body::templates::Prepared::default();
        let mut contracts = body::regions::Collector::default();
        let functions = facts
            .functions
            .iter()
            .filter(|f| f.body.is_some())
            .copied()
            .collect::<Vec<_>>();
        let mut catalog = const_eval::code::Catalog::default();
        let mut answer = None;
        for function in &functions {
            let template = templates.function(&report, &facts, function)?;
            let b = &template.body;
            let body::ir::Owner::Function { definition, .. } = b.owner else {
                continue;
            };
            let name = report
                .package
                .names
                .get(report.resolution.definitions[definition.0 as usize].name);
            let mut contract = contracts.function(&facts, function)?.clone();
            assert_eq!(
                body::prepare_source_copy(&mut facts, b, &mut contract),
                Err("source destructor registration is not implemented".into())
            );
            // These intentionally unproved snapshots exercise the reached-call gate;
            // cache structure is not an execution permission.
            if name == "seven" && !warm {
                continue;
            }
            let index = catalog.insert(b)?;
            if name == "answer" {
                catalog.definitions.insert(definition, index);
                answer = Some(definition);
            }
        }
        let evaluator = sema::values::Evaluator::new(&report, &mut facts);
        let error =
            const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default())
                .definition(answer.ok_or("answer")?, &[])
                .err()
                .ok_or("unprepared execution accepted")?;
        assert!(
            error
                .message
                .contains("source destructor registration is not implemented"),
            "warm={warm}: {}",
            error.message
        );
    }
    Ok(())
}
#[test]
fn ordinary_destruct_evidence_does_not_pay_const_or_conditional_goals() -> Result {
    use crate::types::{
        GenericArgs, ItemId,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    };
    // Reuse the already independently qualified ordinary marker header;
    // these direct qualified-goal attempts are internal opposing proof controls.
    let source = SOURCE.replace(
        "#[lang=\"drop\"]pub const trait Drop{fn drop(&mut self);}",
        "#[lang=\"destruct\"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}",
    ).replace("const_trait_impl", "const_trait_impl,rustc_attrs");
    let modified = report_source(&source)?;
    let mut facts = sema::declarations::analyze(&modified)?;
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(LangItem::Destruct)?
        .ok_or("Destruct")?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(declaration.definition.0),
        arguments: GenericArgs::EMPTY,
    })?;
    for bound in [
        Bound::ConstTrait(application),
        Bound::MaybeConstTrait(application),
    ] {
        let mut state = sema::nominals::traits::State::body(&mut facts, None, 0, 1)?;
        assert!(state.prove(
            &mut facts,
            Predicate {
                ty: TypeId::primitive(crate::types::Primitive::U8),
                bound: Bound::Trait(application)
            },
            false
        )?);
        assert!(!destruct_predicate(
            &facts,
            Predicate {
                ty: TypeId::primitive(crate::types::Primitive::U8),
                bound
            }
        )?);
        assert!(
            state
                .prove(
                    &mut facts,
                    Predicate {
                        ty: TypeId::primitive(crate::types::Primitive::U8),
                        bound
                    },
                    false
                )
                .is_err()
        );
    }
    Ok(())
}
