//! Header identity stays separate from source implementations and endpoint execution.
use crate::{
    resolve::{expansion::Report, model::DefKind, unit::NodeKey},
    sema::{
        compiler_items::{Catalog, RangeHelper, SourceRangeMember, source_range_member},
        declarations, header_attributes,
        model::Declarations,
    },
    support::{bindings::LangItem, source::SourceFile},
    types::{Kind, Primitive, TypeId, callable::Convention},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn input(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/range-helper-roles/{name}.rs"
    ))?)
}
fn resolved(source: &str) -> TestResult<Report> {
    Ok(crate::test_source::source_provider(
        source,
        "source_range_roles",
    )?)
}
fn prepared() -> TestResult<(Report, Declarations)> {
    let report = resolved(&input("source-declarations")?)?;
    let facts = declarations::analyze(&report)?;
    Ok((report, facts))
}
fn source_site(facts: &Declarations) -> TestResult<NodeKey> {
    Ok(facts
        .source_bindings
        .as_deref()
        .ok_or("source catalog")?
        .language(LangItem::RangeMin)
        .ok_or("actual minimum")?
        .origin)
}
#[test]
fn original_three_associated_members_bind_actual_self_and_conditional_const_headers() -> TestResult
{
    let (report, mut facts) = prepared()?;
    let site = source_site(&facts)?;
    let mut parent = None;
    for (helper, role) in [
        (RangeHelper::Minimum, LangItem::RangeMin),
        (RangeHelper::Maximum, LangItem::RangeMax),
        (RangeHelper::Predecessor, LangItem::RangeSub),
    ] {
        let actual = facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .validated_language(role)?
            .ok_or("completed source role")?;
        let (owner, member) = match source_range_member(&report, &mut facts, site, helper)? {
            SourceRangeMember::Bound {
                parent,
                member,
                self_type,
            } => {
                let schema = facts
                    .associated
                    .schema(parent)
                    .ok_or("actual parent schema")?;
                assert_eq!(self_type, schema.self_type);
                assert_eq!(
                    report.resolution.definitions[member.0 as usize].kind,
                    DefKind::Const
                );
                (parent, member)
            }
            SourceRangeMember::Predecessor {
                parent,
                member,
                signature,
            } => {
                let Some(Kind::Function {
                    bound_regions: 0,
                    parameters,
                    result,
                    unsafe_: false,
                    convention: Convention::RUST,
                }) = facts.types.kind(signature)
                else {
                    return Err("exact source method signature".into());
                };
                let self_type = facts
                    .associated
                    .schema(parent)
                    .ok_or("actual parent schema")?
                    .self_type;
                assert_eq!(facts.types.list(parameters), Some(&[self_type][..]));
                assert_eq!(result, self_type);
                (parent, member)
            }
        };
        assert_eq!(member, actual.definition);
        assert_eq!(actual.parent, Some(owner));
        if let Some(expected) = parent {
            assert_eq!(owner, expected);
        } else {
            parent = Some(owner);
        }
        assert_eq!(facts.associated.constant_trait(owner), Some(true));
    }
    let renamed = input("source-declarations")?.replace("RangePattern", "GenuineDifferentName");
    let renamed = resolved(&renamed)?;
    let mut other = declarations::analyze(&renamed)?;
    let key = source_site(&other)?;
    source_range_member(&renamed, &mut other, key, RangeHelper::Minimum)?;
    missing_implementation()?;
    Ok(())
}
fn missing_implementation() -> TestResult {
    let missing = resolved(&input("missing-implementation")?)?;
    let wanted = missing
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, definition)| {
            (definition.kind == DefKind::Function
                && missing.package.names.get(definition.name) == "wanted")
                .then(|| crate::resolve::model::DefId(u32::try_from(index).unwrap_or(u32::MAX)))
        })
        .ok_or("actual wanted function")?;
    let selected = [wanted];
    let result = crate::pipeline::analyze_source(&missing).and_then(
        |crate::pipeline::Analysis {
             mut facts,
             prepared,
         }| {
            crate::lower::instances::build_roots_prepared(
                &missing,
                &mut facts,
                crate::lower::Roots {
                    entry: None,
                    exports: &selected,
                },
                prepared,
            )
        },
    );
    assert!(
        result.is_err(),
        "role registration invented an implementation or body"
    );
    Ok(())
}

#[test]
fn unused_partial_and_same_spelling_families_remain_pending_without_cold_seals() -> TestResult {
    for name in ["partial", "ordinary"] {
        let report = resolved(&input(name)?)?;
        let mut facts = declarations::analyze(&report)?;
        assert!(
            facts
                .source_bindings
                .as_deref()
                .ok_or("catalog")?
                .traits
                .as_deref()
                .is_none_or(|v| v.range.is_none())
        );
        let key = report
            .resolution
            .definitions
            .iter()
            .find_map(|d| {
                (d.kind == DefKind::Trait && report.package.names.get(d.name) == "RangePattern")
                    .then_some(d.node)
                    .flatten()
            })
            .ok_or("actual unprivileged trait")?;
        for helper in [
            RangeHelper::Minimum,
            RangeHelper::Maximum,
            RangeHelper::Predecessor,
        ] {
            assert!(source_range_member(&report, &mut facts, key, helper).is_err());
        }
    }
    Ok(())
}
#[test]
fn wrong_target_owner_self_abi_arity_and_method_contracts_never_complete() -> TestResult {
    for name in [
        "wrong-self",
        "wrong-abi",
        "wrong-arity",
        "unsafe",
        "nonconst",
        "generic",
        "default-method",
        "duplicate",
        "wrong-kind",
    ] {
        let result =
            resolved(&input(name)?).and_then(|r| declarations::analyze(&r).map_err(Into::into));
        let error = result
            .err()
            .ok_or_else(|| format!("unsupported contract accepted {name}"))?
            .to_string();
        assert!(!error.contains("budget"), "{name}: {error}");
        assert!(
            !error.contains("not a supported language item"),
            "{name}: {error}"
        );
        eprintln!("source range header {name}: {error}");
    }
    let split = input("source-declarations")?.replace("#[lang = \"RangeMax\"]", "")
        + "\n#[rustc_const_unstable(feature=\"pattern_type_range_trait\",issue=\"123646\")]\npub const trait Other {#[lang=\"RangeMax\"]const MAX:Self;}\n";
    let report = resolved(&split)?;
    assert!(
        declarations::analyze(&report).is_err(),
        "different actual owners completed one family"
    );
    Ok(())
}
#[test]
fn current_source_store_and_signature_witnesses_reject_foreign_or_mutated_queries() -> TestResult {
    let (mut report, mut facts) = prepared()?;
    let key = source_site(&facts)?;
    let foreign = resolved(&input("source-declarations")?)?;
    assert!(source_range_member(&foreign, &mut facts, key, RangeHelper::Minimum).is_err());
    let types = std::mem::take(&mut facts.types);
    assert!(source_range_member(&report, &mut facts, key, RangeHelper::Minimum).is_err());
    facts.types = types;
    let definition = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(LangItem::RangeSub)
        .ok_or("sub")?
        .definition;
    let position = facts
        .functions
        .iter()
        .position(|f| f.definition == definition)
        .ok_or("actual method header")?;
    let signature = facts.functions[position].signature;
    facts.functions[position].signature = TypeId::primitive(Primitive::U8);
    assert!(source_range_member(&report, &mut facts, key, RangeHelper::Predecessor).is_err());
    facts.functions[position].signature = signature;
    let source = SourceFile::new(report.package.units[0].source.text().as_bytes().to_vec())?;
    let old = std::mem::replace(&mut report.package.units[0].source, source);
    assert!(source_range_member(&report, &mut facts, key, RangeHelper::Minimum).is_err());
    report.package.units[0].source = old;
    source_range_member(&report, &mut facts, key, RangeHelper::Predecessor)?;
    Ok(())
}
#[test]
fn warm_queries_failed_payments_and_cold_role_arrays_keep_bounded_ownership() -> TestResult {
    let (report, mut facts) = prepared()?;
    let key = source_site(&facts)?;
    let mut previous = None;
    for count in [8, 32, 128] {
        let before =
            header_attributes::source_contract_storage(&facts).ok_or("actual header payer")?;
        for _ in 0..count {
            source_range_member(&report, &mut facts, key, RangeHelper::Minimum)?;
        }
        let after =
            header_attributes::source_contract_storage(&facts).ok_or("actual header payer")?;
        assert_eq!(
            (before.1, before.2, before.3, before.4),
            (after.1, after.2, after.3, after.4)
        );
        let work = after.0 - before.0;
        if let Some(old) = previous {
            assert_eq!(work, old * 4);
        }
        previous = Some(work);
        eprintln!("source range queries={count} work={work} storage={after:?}");
    }
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .provider
        .clone();
    let error = crate::sema::crates::authored_export::Stage::begin_module_feature_predicates(
        &report, &mut facts, &identity,
    )
    .err()
    .ok_or("source-only helper roles exported")?;
    assert_eq!(
        error,
        "source range helper contract transport is not implemented"
    );
    assert!(
        header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    let failed = header_attributes::source_contract_storage(&facts).ok_or("failed payer")?;
    assert!(source_range_member(&report, &mut facts, key, RangeHelper::Maximum).is_err());
    let retried = header_attributes::source_contract_storage(&facts).ok_or("retried payer")?;
    assert!(retried.0 > failed.0);
    assert_eq!(retried.1, failed.1);
    assert_eq!(LangItem::UnsafeUnpin.index(), 66);
    assert_eq!(LangItem::RangeMin.index(), 67);
    assert_eq!(LangItem::RangeMax.index(), 68);
    assert_eq!(LangItem::RangeSub.index(), 69);
    assert_eq!(size_of::<super::super::roles::RoleSet>(), 16);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    layouts();
    Ok(())
}

fn layouts() {
    let (bindings, imported, role) = Catalog::binding_layouts();
    let provider = size_of::<Option<crate::resolve::bindings::Declaration>>() * LangItem::ALL.len();
    assert_eq!(provider, 1420);
    assert_eq!(imported, 1208);
    assert!(provider.abs_diff(imported) <= 256);
    assert!(bindings >= provider);
    eprintln!(
        "range layouts Catalog={} Bindings={bindings} Provider={provider} Imported={imported} ImportedRole={role} Reservations={} Validated={} Contracts={} SourceImage={} Body={} Node={}",
        size_of::<Catalog>(),
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<super::super::roles::Validated>(),
        size_of::<super::super::roles::range_members::Contracts>(),
        size_of::<header_attributes::SourceImage>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::syntax::ast::Node>()
    );
}
