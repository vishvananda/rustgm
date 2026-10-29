//! Source role reservations and signatures remain distinct from executable proof.
use super::*;
use crate::{
    support::{
        bindings::SourceIntrinsic,
        crates::generics::{self as g, source as s},
        intrinsics::{ConstPermission, Intrinsic, Safety},
    },
    types::instance::Instance,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]pub trait PointeeSized{}#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}#[lang=\"sized\"]pub trait Sized:MetaSized{}";
const ROLES: [(LangItem, &str); 6] = [
    (LangItem::Not, "not"),
    (LangItem::Panic, "panic"),
    (LangItem::PanicFmt, "panic_fmt"),
    (LangItem::PanicImpl, "panic_impl"),
    (LangItem::PanicDisplay, "panic_display"),
    (LangItem::ConstPanicFmt, "const_panic_fmt"),
];
fn source_report(tail: &str) -> Result<Report> {
    Ok(crate::test_source::source_provider(
        &(SOURCE.to_owned() + tail),
        "panic_declarations",
    )?)
}

#[test]
fn actual_panic_claims_retain_kind_owner_and_uncompleted_role_status() -> Result {
    let tail = "#[lang=\"not\"]pub trait ActualNot{}#[lang=\"panic\"]fn p()->!{loop{}}#[lang=\"panic_fmt\"]fn pf()->!{loop{}}#[lang=\"panic_impl\"]fn pi()->!{loop{}}#[lang=\"panic_display\"]fn pd()->!{loop{}}#[lang=\"const_panic_fmt\"]fn cp()->!{loop{}}";
    let report = source_report(tail)?;
    let reserved = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("reservations")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    for (role, spelling) in ROLES {
        assert_eq!(LangItem::from_name(spelling), Some(role));
        let actual = reserved.language(role).ok_or("actual role declaration")?;
        assert_eq!(catalog.language(role), Some(actual));
        assert!(catalog.validated_language(role).is_err());
        assert!(
            report.package.unit(actual.origin.file).configured.active
                [actual.origin.node.get() as usize]
        );
    }
    assert!(crate::expand::builtin::Builtin::Assert.executable());
    for (role, spelling) in ROLES {
        let wrong = if role == LangItem::Not {
            format!("#[lang=\"{spelling}\"]fn wrong(){{}}")
        } else {
            format!("#[lang=\"{spelling}\"]trait Wrong{{}}")
        };
        assert!(source_report(&wrong).is_err(), "{wrong}");
        let disabled = format!("#[cfg(any())]{wrong}");
        let disabled = source_report(&disabled)?;
        assert!(
            disabled
                .resolution
                .source_bindings
                .as_deref()
                .ok_or("source")?
                .language(role)
                .is_none()
        );
    }
    assert!(source_report("#[lang=\"not\"]trait One{}#[lang=\"not\"]trait Two{}").is_err());
    assert!(
        crate::test_source::resolve(
            "#![feature(lang_items)]#[lang=\"not\"]trait Foreign{}",
            true
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn source_abort_signature_is_safe_nonconst_and_supplies_no_layout_or_const_permission() -> Result {
    let report = source_report("#[rustc_intrinsic]pub fn abort()->!;")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let declaration = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("reservations")?
        .intrinsics()
        .find(|d| d.kind == SourceIntrinsic::Abort)
        .ok_or("actual abort")?
        .declaration;
    assert_eq!(
        catalog.intrinsic_kind(declaration.definition)?,
        Some(SourceIntrinsic::Abort)
    );
    let ty = facts.definitions[declaration.definition.0 as usize].ok_or("actual function item")?;
    let instance = Instance::new(&facts.types, ty)?;
    assert!(catalog.selected_intrinsic(&facts, instance)?.is_none());
    assert!(catalog.check_const_call(declaration.definition).is_err());
    for signature in [
        "pub unsafe fn abort()->!;",
        "pub const fn abort()->!;",
        "pub extern \"C\" fn abort()->!;",
        "pub fn abort(x:u8)->!;",
        "pub fn abort<T>()->!;",
        "pub fn abort<'a>()->!;",
        "pub fn abort()->u8;",
        "pub fn abort()->!{loop{}}",
    ] {
        let source = format!("#[rustc_intrinsic]{signature}");
        let checked = source_report(&source)
            .and_then(|r| crate::sema::declarations::analyze(&r).map_err(Into::into));
        assert!(checked.is_err(), "signature={signature}");
    }
    let reached =
        source_report("#[rustc_intrinsic]pub fn abort()->!;pub fn reached()->!{abort()}")?;
    let owner = reached
        .package
        .source_core()
        .ok_or("source owner")?
        .provider
        .clone();
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&reached)?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            let actual = &reached.resolution.definitions[function.definition.0 as usize];
            reached.package.names.get(actual.name) == "reached"
        })
        .ok_or("actual reached caller")?
        .definition;
    let roots = [function];
    let program = crate::lower::instances::build_roots_prepared(
        &reached,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    assert!(program.symbols.iter().any(|symbol| symbol.body.is_some()));
    assert!(!program.symbols.iter().any(|symbol| symbol.name == "abort"));
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&reached)?;
    let error =
        crate::lower::instances::build_library_prepared(&reached, &mut facts, owner, prepared)
            .err()
            .ok_or("source abort acquired artifact permission from its declaration")?;
    assert_eq!(
        error.message,
        "source type form requires an explicitly supported transport row"
    );
    assert_eq!(Intrinsic::Abort.descriptor().safety, Safety::Safe);
    assert_eq!(
        Intrinsic::Abort.descriptor().constant,
        ConstPermission::RuntimeOnly
    );
    assert!(!Intrinsic::Abort.is_pointer());
    assert!(!Intrinsic::Abort.is_atomic());
    Ok(())
}

#[test]
fn appended_claims_preserve_ordinary_storage_ordinals_and_closed_wire_profiles() -> Result {
    observe_role_layouts();
    for (index, kind) in Intrinsic::BOOTSTRAP.into_iter().enumerate() {
        assert_eq!(kind.index(), index);
        assert_eq!(Intrinsic::ALL[index], kind);
    }
    assert_eq!(Intrinsic::BOOTSTRAP.len(), 24);
    assert_eq!(Intrinsic::Abort.index(), 24);
    assert_eq!(LangItem::FormatArgument.index(), 21);
    assert_eq!(LangItem::Not.index(), 22);
    assert_eq!(LangItem::ConstPanicFmt.index(), 27);
    let report = crate::test_source::resolve("fn abort(){}fn ordinary(){abort();}", true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    assert!(report.resolution.source_bindings.is_none());
    assert!(facts.source_bindings.is_none());
    assert!(report.resolution.intrinsics.get(Intrinsic::Abort).is_none());
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2032);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::resolve::bindings::Reservations>(), 2224);
    assert_eq!(size_of::<crate::resolve::bindings::SourceCore>(), 176);
    assert_eq!(size_of::<Declaration>(), 20);
    assert_eq!(size_of::<Option<Declaration>>(), 20);
    assert_eq!(
        size_of::<crate::resolve::bindings::IntrinsicDeclaration>(),
        24
    );
    assert_eq!(size_of::<LangItem>(), 1);
    assert_eq!(size_of::<SourceIntrinsic>(), 1);
    assert_eq!(size_of::<Intrinsic>(), 1);
    let ordinary = crate::sema::bodies::analyze(&report, facts)?;
    assert!(ordinary.source_bindings.is_none());
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let interface = crate::module_packets::exported(&source)?;
    let (rows, attributes, modules) = crate::module_packets::raw(&interface)?;
    for role in ROLES.into_iter().map(|(role, _)| role).chain([
        LangItem::DispatchFromDyn,
        LangItem::Shl,
        LangItem::BitOr,
        LangItem::Add,
        LangItem::Sub,
        LangItem::ResultOk,
        LangItem::ResultErr,
        LangItem::Option,
        LangItem::OptionSome,
        LangItem::OptionNone,
        LangItem::SliceLen,
        LangItem::AllocLayout,
    ]) {
        let mut changed = rows.clone();
        changed
            .roles
            .first_mut()
            .ok_or("actual existing role")?
            .role = role;
        let owner = interface.identity();
        assert!(
            s::ValidatedRows::stage(owner, changed.clone(), 0, &mut g::Work::default()).is_err()
        );
        assert!(
            s::ValidatedRows::stage_contracts(
                owner,
                changed.clone(),
                attributes.clone(),
                0,
                &mut g::Work::default()
            )
            .is_err()
        );
        assert!(
            s::ValidatedRows::stage_provided(owner, changed.clone(), 0, &mut g::Work::default())
                .is_err()
        );
        assert!(
            s::ValidatedRows::stage_module_contracts(
                owner,
                changed,
                attributes.clone(),
                modules.clone(),
                0,
                &mut g::Work::default()
            )
            .is_err()
        );
    }
    eprintln!(
        "source claims Reservations={} SourceCore={} Resolution={} Package={} Declaration={} OptionDeclaration={} IntrinsicDeclaration={} Catalog={} bindings={:?}",
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<crate::resolve::bindings::SourceCore>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<Declaration>(),
        size_of::<Option<Declaration>>(),
        size_of::<crate::resolve::bindings::IntrinsicDeclaration>(),
        size_of::<Catalog>(),
        Catalog::binding_layouts()
    );
    Ok(())
}

fn observe_role_layouts() {
    eprintln!(
        "source roles={} Reservations={} Catalog={} Body={} Rvalue={}",
        LangItem::ALL.len(),
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<Catalog>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::body::ir::Rvalue>()
    );
}
