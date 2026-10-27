//! Actual source rows govern calls/coercions; ordinary and imported policy stays distinct.
use super::*;
use crate::{
    pipeline::{self, Analysis},
    support::crates::{self, CrateIdentity},
    syntax::{
        ast::Kind as Syntax,
        item::{ItemKind, Safety},
    },
    types::Kind,
};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn input(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa41/tests/spec/target-features/{name}.rs"
    ))?)
}
fn analyze(source: &str) -> TestResult<(Report, Analysis)> {
    let report = crate::test_source::resolve(source, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn named(report: &Report, name: &str) -> TestResult<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == DefKind::Function && report.package.names.get(d.name) == name)
        .map(|(i, _)| u32::try_from(i).map(DefId))
        .transpose()?
        .ok_or_else(|| format!("actual function {name}").into())
}
fn reject(name: &str, message: &str) -> TestResult {
    let source = input(name)?;
    let report = crate::test_source::resolve(&source, true)?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("expected actual source rejection")?;
    assert_eq!(error.message, message, "{name}");
    Ok(())
}
#[test]
fn declaration_forms_required_gates_and_all_owner_sites_are_checked() -> TestResult {
    for name in [
        "safe-declaration",
        "safe-c-abi",
        "duplicate-features",
        "unsafe-trait-method",
        "cfg-disabled",
        "gated/amx-enabled",
        "gated/avx10-enabled",
        "forwarded/literal-enabled",
        "forwarded/raw-enabled",
        "forwarded/ordinary-doc-cfg",
    ] {
        analyze(&input(name)?)?;
    }
    for name in ["unknown-feature", "empty-feature"] {
        reject(
            name,
            "target feature is invalid or forbidden for this target",
        )?;
    }
    reject(
        "malformed-enable",
        "target_feature requires a string literal",
    )?;
    reject("nonfunction-owner", "target_feature requires a function")?;
    reject(
        "safe-trait-method",
        "target_feature cannot be applied to a safe trait method",
    )?;
    for name in [
        "gated/amx-missing",
        "gated/avx10-missing",
        "gated/wrong-gate",
        "gated/literal-grant",
        "gated/captured-literal",
        "gated/captured-meta",
    ] {
        reject(
            name,
            "target feature requires its actual crate language gate",
        )?;
    }
    let (_, Analysis { facts, .. }) = analyze(&input("duplicate-features")?)?;
    let rows = facts.target_feature_functions.values().collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        return Err("one actual target row".into());
    };
    assert_eq!(row.sites.len(), 2);
    Ok(())
}
#[test]
fn direct_calls_preserve_body_features_declared_safety_and_constant_results() -> TestResult {
    for name in [
        "unsafe-block",
        "matching-caller",
        "implied-caller",
        "explicit-unsafe-block",
        "closure-inherits",
    ] {
        analyze(&input(name)?)?;
    }
    for name in [
        "missing-caller-feature",
        "insufficient-caller",
        "nested-function-separate",
        "baseline-no-waiver",
        "const-missing",
    ] {
        reject(
            name,
            "call requires an unsafe context or the callee's target features",
        )?;
    }
    for name in [
        "explicit-unsafe",
        "safe-body-remains-safe",
        "unsafe-pointer-call",
    ] {
        reject(name, "call to unsafe function requires an unsafe context")?;
    }
    let source =
        input("const-unsafe")? + "\n#[unsafe(no_mangle)]pub extern \"C\" fn value()->u8{C}";
    let (
        report,
        Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(&source)?;
    let f = named(&report, "f")?;
    assert!(
        matches!(
            facts.types.kind(
                facts.functions[*facts.function_indices.get(&f).ok_or("actual f index")?].signature
            ),
            Some(Kind::Function { unsafe_: true, .. })
        ),
        "canonical target signature remains unsafe"
    );
    let roots = [named(&report, "value")?];
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    assert!(program.symbols.iter().filter_map(|s|s.body.as_ref()).any(|body| body.blocks.iter().any(|block|matches!(block.terminator,crate::lowir::program::Terminator::Return(Some(value)) if matches!(value.operand,crate::lowir::program::Operand::Literal(id) if matches!(body.literals.get(id.0 as usize),Some(crate::lowir::program::Literal::Integer(3))))))),"genuine CTFE result reaches LowIR");
    Ok(())
}
#[test]
fn pointer_adjustment_uses_actual_item_context_and_retained_body_proof() -> TestResult {
    for name in ["unsafe-pointer", "safe-pointer-matching"] {
        analyze(&input(name)?)?;
    }
    reject(
        "safe-pointer-missing",
        "incompatible callable convention or safety",
    )?;
    let source = "#[target_feature(enable=\"sse2\")]fn f(x:u8)->u8{x}#[target_feature(enable=\"sse2\")]pub fn g(x:u8)->u8{let p:fn(u8)->u8=f;p(x)}";
    let (
        report,
        Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(source)?;
    let f = named(&report, "f")?;
    let g = named(&report, "g")?;
    assert!(matches!(query(&report,&facts,f)?,Policy::Local(Some(row)) if !row.declared_unsafe()));
    let mut reified = 0;
    for row in &facts.nodes {
        for node in row {
            if node
                .adjustment
                .is_some_and(|a| a.kind == crate::sema::model::AdjustmentKind::FunctionPointer)
            {
                reified += 1;
            }
        }
    }
    assert_eq!(reified, 1);
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[g],
        },
        prepared,
    )?;
    crate::lowir::writer::write(&program, &mut Vec::new())?;
    Ok(())
}
#[test]
fn current_images_declared_owners_and_imported_unknown_cannot_be_substituted() -> TestResult {
    let source = input("safe-declaration")?;
    let (report, Analysis { facts, .. }) = analyze(&source)?;
    let f = named(&report, "f")?;
    let other = crate::test_source::resolve(&source, true)?;
    assert!(
        query(&other, &facts, f).is_err(),
        "fresh equal bytes are a different incarnation"
    );
    let mut changed = crate::test_source::resolve(&source, true)?;
    let mut facts2 = crate::sema::declarations::analyze(&changed)?;
    let id = named(&changed, "f")?;
    let source = facts2
        .target_feature_functions
        .get(&id)
        .ok_or("row")?
        .source;
    let Syntax::Item(index) = changed.package.units[source.file.0 as usize]
        .configured
        .ast
        .node(source.node)
        .kind
    else {
        return Err("actual item".into());
    };
    if let ItemKind::Function { flags, .. } = &mut changed.package.units[source.file.0 as usize]
        .configured
        .ast
        .items[index as usize]
        .kind
    {
        flags.safety = Safety::Unsafe;
    }
    assert!(query(&changed, &facts2, id).is_err());
    facts2
        .target_feature_functions
        .get_mut(&id)
        .ok_or("row")?
        .source
        .node = crate::syntax::ast::NodeId::new(0);
    assert!(query(&changed, &facts2, id).is_err());
    for entry in [false, true] {
        let mut changed = crate::test_source::resolve(&input("safe-declaration")?, true)?;
        let facts = crate::sema::declarations::analyze(&changed)?;
        let id = named(&changed, "f")?;
        let row = facts
            .target_feature_functions
            .get(&id)
            .ok_or("actual retained row")?;
        let site = row.sites.first().ok_or("actual site")?;
        let file = site.owner.file.0 as usize;
        let capture = site.capture as usize;
        let span = site
            .input
            .iter()
            .find(|(_, tree)| {
                if entry {
                    matches!(
                        tree.kind,
                        crate::syntax::token_tree::TreeKind::Token(
                            crate::lex::token::Kind::Identifier
                        )
                    )
                } else {
                    matches!(
                        tree.kind,
                        crate::syntax::token_tree::TreeKind::Token(crate::lex::token::Kind::String)
                    )
                }
            })
            .ok_or("actual parsed input tree")?
            .1
            .span;
        let forest = changed.package.units[file].configured.ast.attributes[capture]
            .forest
            .map_spans(&mut |actual| {
                Ok(if actual == span {
                    crate::syntax::token_tree::Span {
                        start: actual.start + 1,
                        ..actual
                    }
                } else {
                    actual
                })
            })?;
        changed.package.units[file].configured.ast.attributes[capture].forest = forest;
        assert!(
            changed
                .lint_producer()
                .ok_or("actual generic owner producer")?
                .validate_owner(&changed.package, site.owner)
                .is_ok(),
            "path-only lint witness deliberately does not cover target input"
        );
        assert_eq!(
            query(&changed, &facts, id)
                .err()
                .ok_or("changed target input must reject")?
                .message,
            "target-feature input trees changed"
        );
    }
    let provider = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let interface = crate::source_packets::exported(&provider)?;
    let (client, analysis, _) = crate::source_packets::client_analysis(
        interface,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn keep()->u8{base::identity(7u8)}",
    )?;
    let imported = client
        .resolution
        .crates
        .as_ref()
        .ok_or("actual import")?
        .definitions()
        .find_map(|(external, id)| {
            external
                .path()
                .last()
                .is_some_and(|name| name == "identity")
                .then_some(id)
        })
        .ok_or("imported identity")?;
    assert!(matches!(
        query(&client, &analysis.facts, imported)?,
        Policy::ImportedUnknown
    ));
    Ok(())
}
#[test]
fn ordinary_library_mode_and_existing_transport_denials_remain_exact() -> TestResult {
    let source = "#![feature(x86_amx_intrinsics)]#[target_feature(enable=\"amx-tile\")]fn hidden(){}pub fn plain()->u8{7}";
    let (
        report,
        Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(source)?;
    assert!(report.package.source_core().is_none());
    assert!(!report.package.uses_source_bindings());
    assert!(facts.source_bindings.is_none());
    let hidden = named(&report, "hidden")?;
    assert!(matches!(
        facts.header_attribute_contracts(&report, hidden)?.must_use,
        header_attributes::ContractKnowledge::Unknown
    ));
    let identity = CrateIdentity::new(&report.package.crate_name, "ordinary_cpu")?;
    assert_eq!(
        crate::sema::crates::authored_export::Stage::begin(&report, &mut facts, &identity).err(),
        Some("source export requires its actual own-provider policy")
    );
    let (_, interface) =
        crate::lower::instances::build_library_prepared(&report, &mut facts, identity, prepared)?;
    assert!(
        interface.source_descriptor().is_none(),
        "ordinary library never acquired source profile"
    );
    let bytes = crates::encode(&interface)?;
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    let provider = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )? + "\n#[target_feature(enable=\"sse2\")]pub fn flagged()->u8{7}";
    let error = crate::source_packets::exported(&provider)
        .err()
        .ok_or("source target-feature export must remain closed")?;
    assert!(error.to_string().contains("target-feature"), "{error}");
    Ok(())
}
#[test]
fn cold_rows_cumulative_failures_and_common_pending_map_cost_are_measured() -> TestResult {
    let (_, Analysis { facts, .. }) = analyze("pub fn ordinary()->u8{7}")?;
    assert!(facts.target_feature_functions.is_empty());
    assert_eq!(facts.target_feature_functions.capacity(), 0);
    assert!(
        header_attributes::attribute_storage(&facts).is_none(),
        "ordinary functions need no target ledger"
    );
    for count in [1, 8, 32, 128] {
        let mut source = String::new();
        for index in 0..count {
            write!(source, "#[target_feature(enable=\"avx\")]fn f{index}(){{}}")?;
        }
        let report = crate::test_source::resolve(&source, true)?;
        let mut raw = Declarations::new(&report)?;
        let sources = collect(&report, &mut raw)?;
        let source_capacity = sources.capacity();
        let source_bytes = crate::support::retained::map(&sources);
        drop(sources);
        let Analysis { facts, .. } = pipeline::analyze_source(&report)?;
        let inputs = facts
            .target_feature_functions
            .values()
            .flat_map(|row| &row.sites)
            .map(|site| crate::support::retained::vector(&site.input))
            .sum::<usize>();
        let sites = facts
            .target_feature_functions
            .values()
            .map(|row| row.sites.capacity() * size_of::<Site>())
            .sum::<usize>();
        eprintln!(
            "target-rows={count} row={} site={} map-capacity={} map-bytes={} site-bytes={sites} input-bytes={inputs} source-capacity={source_capacity} source-bytes={source_bytes} ledger={:?}",
            size_of::<Contract>(),
            size_of::<Site>(),
            facts.target_feature_functions.capacity(),
            crate::support::retained::map(&facts.target_feature_functions),
            header_attributes::attribute_storage(&facts)
        );
    }
    for (count, depth) in [(1, 1), (8, 1), (32, 1), (1, 8), (1, 32)] {
        let mut source = String::from("fn id(x:u8)->u8{x}pub fn measured()->u8{let mut x=7u8;");
        for _ in 0..count {
            source.push_str("x=");
            for _ in 0..depth {
                source.push_str("id(");
            }
            source.push('x');
            for _ in 0..depth {
                source.push_str(")");
            }
            source.push(';');
        }
        source.push_str("x}");
        let report = crate::test_source::resolve(&source, true)?;
        eprintln!(
            "pending count={count} depth={depth} actual={:?}",
            crate::sema::bodies::observe_active_calls(&report, "measured")?
        );
    }
    let mut high = FeatureSet::default();
    high.insert(
        "xsaves",
        |_| Ok::<_, &'static str>(()),
        |_| Ok(true),
        std::convert::identity,
    )?;
    assert!(
        high.uses_upper_word(),
        "required feature identities are not truncated"
    );
    let mut xop = FeatureSet::default();
    xop.insert(
        "xop",
        |_| Ok::<_, &'static str>(()),
        |gate| Ok(gate == crate::support::cpu_features::Gate::Xop),
        std::convert::identity,
    )?;
    assert!(
        xop.insert(
            "fma4",
            |_| Ok::<_, &'static str>(()),
            |_| Ok(false),
            std::convert::identity
        )
        .is_err(),
        "implied presence cannot grant an explicit feature language gate"
    );
    let (report, Analysis { facts, .. }) = analyze(&input("safe-declaration")?)?;
    assert!(
        header_attributes::charge_attribute_retention(
            &facts,
            crate::support::crates::MAX_COMPONENTS + 1,
            0
        )
        .is_err()
    );
    assert!(header_attributes::charge_attribute_retention(&facts, 0, 0).is_err());
    assert!(
        query(&report, &facts, named(&report, "f")?).is_err(),
        "failed work remains cumulative"
    );
    eprintln!(
        "target enclosing: global features verified separately; declaration model={} source={} body={} report={}",
        size_of::<Declarations>(),
        size_of::<FeatureSet>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<Report>()
    );
    assert_eq!(crate::resolve::bindings::cpu_request_layouts().0, 8);
    assert_eq!(crate::resolve::bindings::cpu_request_layouts().1, 1);
    eprintln!(
        "request layouts={:?}",
        crate::resolve::bindings::cpu_request_layouts()
    );
    Ok(())
}
