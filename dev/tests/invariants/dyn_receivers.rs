//! Genuine source declarations and isolated receiver proofs keep execution owners distinct.
use super::{Error, Queries};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
    },
    sema::{declarations, model::Declarations},
    support::bindings::LangItem,
};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const FIXTURES: &str = "pa31/tests/spec/source-dispatch-from-dyn";
fn source(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "{FIXTURES}/{name}.rs"
    ))?)
}
fn provider(text: &str) -> Result<Report> {
    Ok(crate::test_source::source_provider(
        text,
        "dispatch_receiver",
    )?)
}
fn definition(report: &Report, name: &str) -> Result<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, value)| {
            value.kind == DefKind::Trait && report.package.names.get(value.name) == name
        })
        .and_then(|(index, _)| u32::try_from(index).ok())
        .map(DefId)
        .ok_or_else(|| "actual source trait".into())
}
fn checked(name: &str) -> Result<(Report, Declarations)> {
    let report = provider(&source(name)?)?;
    let facts = declarations::analyze(&report)?;
    Ok((report, facts))
}

#[test]
fn actual_roles_and_receiver_scopes_retain_genuine_source_identity() -> Result {
    let (report, mut facts) = checked("call-prefix-followup/actual-reference-call")?;
    let root = definition(&report, "Write")?;
    let actual = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(LangItem::DispatchFromDyn)?
        .ok_or("completed DispatchFromDyn")?;
    assert_eq!(actual.definition, definition(&report, "DispatchFromDyn")?);
    let mut queries = Queries::default();
    assert_eq!(queries.demand(&report, &mut facts, root)?.methods.len(), 1);
    assert!(queries.source.is_some());
    for _ in 0..16 {
        assert_eq!(queries.demand(&report, &mut facts, root)?.methods.len(), 1);
    }
    assert!(queries.steps < 100_000);

    let ordinary = crate::test_source::resolve(
        "trait Flag{} trait Methods{fn plain(&self);fn own<T>(&self,value:T);fn bounded(&self)where Self:Flag;}",
        true,
    )?;
    let ordinary_facts = declarations::analyze(&ordinary)?;
    let owner = definition(&ordinary, "Methods")?;
    let parent = ordinary_facts
        .generics
        .environments
        .get(&owner)
        .ok_or("completed parent scope")?;
    for (index, value) in ordinary.resolution.definitions.iter().enumerate() {
        if value.kind != DefKind::Function {
            continue;
        }
        let name = ordinary.package.names.get(value.name);
        if ["plain", "own", "bounded"].contains(&name) {
            let id = DefId(u32::try_from(index)?);
            let environment = ordinary_facts
                .generics
                .environments
                .get(&id)
                .ok_or("completed actual method scope")?;
            assert_eq!(
                std::sync::Arc::ptr_eq(parent, environment),
                name == "plain",
                "{name}"
            );
        }
    }
    Ok(())
}

#[test]
fn impl_shapes_preserve_written_bounds_regions_and_actual_changed_fields() -> Result {
    for name in ["raw-without-unsize-premise", "nominal-one-field"] {
        let _ = checked(&format!("library-prefix-followup/{name}"))?;
    }
    for (name, expected) in [
        ("reference-lifetime-mismatch", "must preserve"),
        ("raw-mutability-mismatch", "must preserve"),
        ("nominal-two-fields", "more than one changed"),
        ("nominal-c-layout", "unpacked non-C struct"),
    ] {
        let report = provider(&source(&format!("library-prefix-followup/{name}"))?)?;
        let error = declarations::analyze(&report)
            .err()
            .ok_or("invalid source impl accepted")?
            .to_string();
        assert!(error.contains(expected), "{name}: {error}");
    }
    let _ = checked("written-premise-followup/inherited-written-premise")?;
    let report = provider(&source(
        "written-premise-followup/unproved-written-premise",
    )?)?;
    let unproved = declarations::analyze(&report)
        .err()
        .ok_or("unproved written impl premise accepted")?
        .to_string();
    assert!(unproved.contains("dispatchable receiver"), "{unproved}");
    let report = provider(&source("library-prefix-followup/missing-receiver-impl")?)?;
    let error = declarations::analyze(&report)
        .err()
        .ok_or("missing receiver impl accepted")?
        .to_string();
    assert!(error.contains("dispatchable receiver"), "{error}");
    Ok(())
}

#[test]
fn real_source_call_passes_body_and_original_instance_lowering() -> Result {
    let report = provider(&source("call-prefix-followup/actual-reference-call")?)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let entry = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "observe"
        })
        .ok_or("real observer")?
        .definition;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[entry],
        },
        prepared,
    )?;
    assert!(!program.symbols.is_empty());
    assert!(
        program
            .symbols
            .iter()
            .filter(|symbol| symbol.body.is_some())
            .count()
            >= 3
    );
    Ok(())
}

#[test]
fn cached_source_receivers_reject_foreign_facts_changed_headers_and_missing_completion() -> Result {
    let (mut report, mut facts) = checked("call-prefix-followup/actual-reference-call")?;
    let root = definition(&report, "Write")?;
    let mut queries = Queries::default();
    queries.demand(&report, &mut facts, root)?;
    let (other_report, mut other_facts) = checked("call-prefix-followup/actual-reference-call")?;
    let before = queries.steps;
    assert!(matches!(
        queries.demand(&other_report, &mut other_facts, root),
        Err(Error::Invalid(
            "dyn source query belongs to another declaration arena"
        ))
    ));
    assert!(queries.steps > before);
    let saved = facts.must_use_contracts.take();
    assert!(queries.demand(&report, &mut facts, root).is_err());
    facts.must_use_contracts = saved;
    let declaration = definition(&report, "DispatchFromDyn")?;
    let key = report.resolution.definitions[declaration.0 as usize]
        .node
        .ok_or("source header")?;
    let unit = &mut report.package.units[key.file.0 as usize];
    let crate::syntax::ast::Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err("actual trait item".into());
    };
    let crate::syntax::item::ItemKind::Trait { unsafe_, .. } =
        &mut unit.configured.ast.items[index as usize].kind
    else {
        return Err("actual trait syntax".into());
    };
    *unsafe_ = true;
    for _ in 0..2 {
        let before = crate::sema::header_attributes::source_contract_storage(&facts)
            .ok_or("actual source payer")?
            .0;
        assert!(queries.demand(&report, &mut facts, root).is_err());
        let after = crate::sema::header_attributes::source_contract_storage(&facts)
            .ok_or("actual source payer after failed authentication")?
            .0;
        assert!(after > before, "failed header work must remain cumulative");
    }
    Ok(())
}

#[test]
fn method_count_and_inherited_depth_share_scope_and_clause_prefixes() -> Result {
    let actual = source("call-prefix-followup/actual-reference-call")?;
    let (prefix, _) = actual
        .split_once("pub trait Write")
        .ok_or("real declaration foundation")?;
    for captured in [false, true] {
        for methods in [1, 4, 16] {
            for depth in [1, 4, 16] {
                let text = matrix_source(prefix, methods, depth, captured)?;
                let report = provider(&text)?;
                let mut facts = declarations::analyze(&report)?;
                let root = definition(&report, "Writer")?;
                let owner = facts.associated.owner(root).ok_or("real trait owner")?;
                let parent = facts
                    .generics
                    .environments
                    .get(&root)
                    .ok_or("actual parent environment")?;
                for &method in facts
                    .associated
                    .owner_members(owner)
                    .ok_or("real methods")?
                {
                    assert!(std::sync::Arc::ptr_eq(
                        parent,
                        facts
                            .generics
                            .environments
                            .get(&method)
                            .ok_or("actual method scope")?
                    ));
                }
                let before = crate::sema::header_attributes::source_contract_storage(&facts)
                    .ok_or("actual source budget")?;
                let mut queries = Queries::default();
                assert_eq!(
                    queries.demand(&report, &mut facts, root)?.methods.len(),
                    methods
                );
                let cold = queries.steps;
                let charged = crate::sema::header_attributes::source_contract_storage(&facts)
                    .ok_or("actual source budget")?;
                println!(
                    "receiver-paid captured={captured} methods={methods} depth={depth} steps={} bytes={}",
                    charged.0 - before.0,
                    charged.1 - before.1
                );
                for _ in 0..8 {
                    assert_eq!(
                        queries.demand(&report, &mut facts, root)?.methods.len(),
                        methods
                    );
                }
                println!(
                    "receiver-matrix captured={captured} methods={methods} depth={depth} cold={cold} warm={} states={} capacity={}",
                    queries.steps - cold,
                    queries.states.len(),
                    queries.states.capacity()
                );
                assert!(cold <= 10_000 * (methods + depth));
            }
        }
    }
    Ok(())
}

fn matrix_source(prefix: &str, methods: usize, depth: usize, captured: bool) -> Result<String> {
    let mut text = format!("{prefix}\npub trait B0<T:PointeeSized>{{}}\n");
    for index in 1..depth {
        writeln!(
            text,
            "pub trait B{index}<T:PointeeSized>:B{}<T>{{}}",
            index - 1
        )?;
    }
    let lifetime = if captured { "'a," } else { "" };
    write!(text, "pub trait Writer<{lifetime}")?;
    for index in 0..depth {
        write!(text, "T{index}:PointeeSized,")?;
    }
    write!(text, ">:B{}<T0>{{", depth - 1)?;
    let reference = if captured { "&'a T0" } else { "&T0" };
    for index in 0..methods {
        write!(
            text,
            "fn write{index}(&mut self,value:{reference},nested:for<'z>fn(&'z T0))->u8;"
        )?;
    }
    text.push('}');
    Ok(text)
}

#[test]
fn ordinary_receivers_keep_existing_no_source_allocations_and_hot_layouts() -> Result {
    let report = crate::test_source::resolve(
        "trait Write{fn write(&mut self)->u8;}struct Formatter<'a>{buf:&'a mut(dyn Write+'a)}",
        true,
    )?;
    let mut facts = declarations::analyze(&report)?;
    let mut queries = Queries::default();
    assert!(queries.source.is_none());
    let root = definition(&report, "Write")?;
    assert_eq!(queries.demand(&report, &mut facts, root)?.methods.len(), 1);
    assert!(queries.source.is_none());
    crate::types::parameters::environment::Environment::check_query_scope_controls()?;
    crate::sema::traits::assumptions::receiver_prefix_tests::controls()?;
    let counts = crate::types::parameters::environment::Environment::query_scope_layouts();
    println!(
        "DispatchQueries={} Environment={} ParameterPayload={} DeclaredMap={} Source={} Operation={} NodeFacts={} Kind={} Body={}",
        size_of::<Queries>(),
        counts.0,
        counts.1,
        counts.2,
        size_of::<crate::sema::traits::selection::Source>(),
        size_of::<crate::sema::model::Operation>(),
        size_of::<crate::sema::model::NodeFacts>(),
        size_of::<crate::types::Kind>(),
        size_of::<crate::body::ir::Body>()
    );
    assert_eq!(size_of::<crate::sema::traits::selection::Source>(), 16);
    assert_eq!(size_of::<crate::sema::model::Operation>(), 16);
    assert_eq!(size_of::<crate::body::ir::Body>(), 448);
    Ok(())
}
