//! Actual no-core alias declarations retain their own Self and source sizing clauses.
use super::*;
use crate::{
    resolve::{expansion::Report, model::DefKind},
    sema::{declarations::regions, generics::Predicate},
    types::{
        ItemId, Kind, TypeId,
        arguments::Argument,
        parameters::{ParameterKind, clauses::Clause},
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn provider() -> TestResult<String> {
    Ok(crate::test_source::fixture(
        "pa31/tests/spec/alias-self-sizing/provider.rs",
    )?)
}
fn definition(report: &Report, name: &str, kind: DefKind) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|row| row.kind == kind && report.package.names.get(row.name) == name)
        .ok_or("actual named source declaration")?;
    Ok(DefId(u32::try_from(index)?))
}
fn prepared(report: &Report) -> TestResult<Checker<'_>> {
    let mut checker = Checker::new(report)?;
    let error = checker
        .run()
        .err()
        .ok_or("alias execution fence disappeared")?;
    assert!(
        error
            .message
            .contains("trait alias contract expansion is not implemented"),
        "{error}"
    );
    Ok(checker)
}
fn application(
    checker: &mut Checker<'_>,
    caller: &str,
    target: &str,
) -> TestResult<(TypeId, Vec<Predicate>)> {
    let caller = definition(checker.report, caller, DefKind::TraitAlias)?;
    let target = definition(checker.report, target, DefKind::TraitAlias)?;
    let self_id = checker.alias_self(caller).ok_or("actual alias Self")?;
    let subject = checker
        .alias_self_schema(self_id)?
        .ok_or("complete Self schema")?
        .self_type;
    let source = checker.report.resolution.definitions[caller.0 as usize]
        .node
        .ok_or("source alias")?;
    let ItemKind::TraitAlias { bounds, .. } = checker.item(source)?.kind else {
        return Err("actual caller is not an alias".into());
    };
    let ast = &checker.report.package.unit(source.file).configured.ast;
    let mut path = None;
    for &node in ast.list(bounds) {
        let crate::syntax::ast::Kind::Bound { value, .. } = ast.node(node).kind else {
            return Err("actual alias RHS bound".into());
        };
        if let Written::Alias {
            path: actual,
            definition,
        } = checker.trait_bound(NodeKey {
            node: value,
            ..source
        })? && definition == target
        {
            path = Some(actual);
        }
    }
    let context = regions::Context {
        binder: Some(checker.owner_binders[&self_id]),
        position: regions::Position::Output,
    };
    let edge = checker.canonical_alias_edge(
        path.ok_or("actual source alias application")?,
        target,
        subject,
        context,
        None,
    )?;
    Ok((subject, checker.expand_alias_edge(edge)?))
}
fn bits(checker: &Checker<'_>, clauses: &[Predicate], subject: TypeId) -> TestResult<u8> {
    let catalog = checker
        .facts
        .source_bindings
        .as_deref()
        .ok_or("actual source catalog")?;
    let meta = catalog
        .role(LangItem::MetaSized)?
        .ok_or("actual MetaSized")?
        .definition;
    let pointee = catalog
        .role(LangItem::PointeeSized)?
        .ok_or("actual PointeeSized")?
        .definition;
    let mut result = 0;
    for clause in clauses {
        let Clause::Predicate(predicate) = clause.value else {
            continue;
        };
        if predicate.ty != subject {
            continue;
        }
        if predicate.bound == Bound::Sized {
            result |= SIZED;
            continue;
        }
        if let Some((application, _)) = predicate.bound.trait_parts() {
            let item = checker
                .facts
                .types
                .trait_application(application)
                .ok_or("actual trait arguments")?
                .item;
            if item == ItemId(meta.0) {
                result |= META
            }
            if item == ItemId(pointee.0) {
                result |= POINTEE
            }
        }
    }
    Ok(result)
}
#[test]
fn alias_self_defaults_distinguish_direct_where_and_alias_mediated_roles() -> TestResult {
    let report = crate::test_source::source_provider(&provider()?, "alias_sizing")?;
    let mut checker = prepared(&report)?;
    for (caller, target, expected) in [
        ("ProbeDirect", "Direct", POINTEE),
        ("ProbeWhere", "InWhere", META | POINTEE),
        ("ProbeMediated", "Mediated", META | POINTEE),
        ("ProbeMeta", "WithMeta", META),
        ("ProbeSized", "WithSized", SIZED),
        ("ProbeLookalike", "Lookalike", META),
    ] {
        let (subject, clauses) = application(&mut checker, caller, target)?;
        assert_eq!(bits(&checker, &clauses, subject)?, expected, "{target}");
    }
    Ok(())
}
#[test]
fn unused_formals_and_sized_self_keep_distinct_actual_parameters() -> TestResult {
    let report = crate::test_source::source_provider(&provider()?, "alias_formals")?;
    let mut checker = prepared(&report)?;
    for (caller, target, sized, expected) in [
        ("ProbeUnused", "Unused", true, META),
        ("ProbeRelax", "RelaxUnused", false, POINTEE),
    ] {
        let owner = definition(&report, target, DefKind::TraitAlias)?;
        let self_id = checker.alias_self(owner).ok_or("real lexical Self")?;
        assert!(checker.facts.associated.owner(owner).is_none());
        assert!(checker.facts.associated.owner(self_id).is_none());
        let self_ty = checker
            .alias_self_schema(self_id)?
            .ok_or("Self schema")?
            .self_type;
        let Some(Kind::Parameter(id)) = checker.facts.types.kind(self_ty) else {
            return Err("real Self parameter".into());
        };
        assert_eq!(
            checker
                .facts
                .types
                .parameter(id)
                .ok_or("real Self owner")?
                .owner,
            ItemId(self_id.0)
        );
        assert_eq!(
            checker
                .facts
                .types
                .parameter(id)
                .ok_or("real Self kind")?
                .kind,
            ParameterKind::Type { sized: false }
        );
        let slots = &checker.facts.generics.parameters[&owner];
        assert_eq!(slots.len(), 1);
        let Argument::Type(ty) = slots[0].argument else {
            return Err("retained unused formal".into());
        };
        let Some(Kind::Parameter(id)) = checker.facts.types.kind(ty) else {
            return Err("real formal parameter".into());
        };
        assert_eq!(
            checker.facts.types.parameter(id).ok_or("formal kind")?.kind,
            ParameterKind::Type { sized }
        );
        let caller_id = definition(&report, caller, DefKind::TraitAlias)?;
        let Argument::Type(actual) = checker.facts.generics.parameters[&caller_id][0].argument
        else {
            return Err("actual source formal argument".into());
        };
        let (_, clauses) = application(&mut checker, caller, target)?;
        assert_eq!(bits(&checker, &clauses, actual)?, expected);
    }
    Ok(())
}
#[test]
fn wrong_source_owners_and_unsupported_relaxations_do_not_gain_alias_authority() -> TestResult {
    let source = provider()?;
    let report = crate::test_source::source_provider(&source, "alias_authority")?;
    let checker = prepared(&report)?;
    let direct = definition(&report, "Direct", DefKind::TraitAlias)?;
    let other = definition(&report, "WithMeta", DefKind::TraitAlias)?;
    let key = report.resolution.definitions[direct.0 as usize]
        .node
        .ok_or("actual source key")?;
    assert!(checker.source_alias_self(other, key).is_err());
    assert!(
        checker
            .source_alias_self(
                report.resolution.crate_root.ok_or("actual crate root")?,
                key
            )
            .is_err()
    );
    assert_eq!(
        checker.source_alias_self(direct, key)?,
        checker.alias_self(direct)
    );
    for suffix in [
        "trait Invalid=?Sized;",
        "trait Invalid=PointeeSized where Self:?Sized;",
        "trait Invalid:?Sized{}",
    ] {
        let report =
            crate::test_source::source_provider(&(source.clone() + suffix), "alias_relaxation")?;
        let mut checker = Checker::new(&report)?;
        let error = checker.run().err().ok_or("invalid relaxed Self admitted")?;
        assert!(
            error
                .message
                .contains("relaxed Sized bounds require a declared type parameter"),
            "{error}"
        );
    }
    assert!(checker.reject_alias_execution().is_err());
    Ok(())
}

#[test]
fn ordinary_sources_keep_the_same_absent_source_sizing_owner_and_layouts() -> TestResult {
    let report = crate::test_source::resolve("fn ordinary<T>(){}", true)?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    assert!(checker.facts.source_bindings.is_none());
    assert!(checker.trait_aliases.is_none());
    println!(
        "alias sizing layouts BoundSyntax={} Checker={} Declarations={} Package={} Unit={} Node={} Scope={} Definition={}",
        size_of::<Syntax>(),
        size_of::<Checker<'_>>(),
        size_of::<crate::sema::model::Declarations>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::unit::Unit>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::resolve::model::Scope>(),
        size_of::<crate::resolve::model::Definition>(),
    );
    Ok(())
}
