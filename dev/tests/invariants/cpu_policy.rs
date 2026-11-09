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
        analyze(&input(name)?).map_err(|error| format!("{name}: {error}"))?;
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
    for name in [
        "forwarded/byte-string",
        "forwarded/c-string",
        "forwarded/nonliteral",
        "forwarded/concat-expression",
    ] {
        reject(name, "target_feature requires a string literal")?;
    }
    let (_, Analysis { facts, .. }) = analyze(&input("duplicate-features")?)?;
    let rows = facts.target_feature_functions.values().collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        return Err("one actual target row".into());
    };
    assert_eq!(row.sites.len(), 2);
    Ok(())
}
/// Pinned rustc rejects `#[inline(always)]` on a target-feature function,
/// including generic, unused and associated ones, at the inline attribute
/// (`predictions-inline-always.json`); the other two hints remain valid.
#[test]
fn always_inline_target_feature_functions_are_rejected_at_the_hint() -> TestResult {
    for name in ["inline-hint", "inline-never"] {
        analyze(&input(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "inline-always",
        "inline-always-generic",
        "inline-always-method",
    ] {
        let source = input(name)?;
        let report = crate::test_source::resolve(&source, true)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("expected actual source rejection")?;
        assert_eq!(
            error.message, "cannot use `#[inline(always)]` with `#[target_feature]`",
            "{name}"
        );
        let hint = source.find("#[inline(always)]").ok_or("inline hint")?;
        assert_eq!(error.span.start, u32::try_from(hint)?, "{name}");
    }
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
fn changed_input_tree(entry: bool) -> TestResult {
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
                    crate::syntax::token_tree::TreeKind::Token(crate::lex::token::Kind::Identifier)
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
        })
        .map_err(|error| error.message)?;
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
    changed_input_tree(false)?;
    changed_input_tree(true)?;
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
fn pending_map_scaling() -> TestResult {
    for (count, depth) in [(1, 1), (8, 1), (32, 1), (1, 8), (1, 32)] {
        let mut source = String::from("fn id(x:u8)->u8{x}pub fn measured()->u8{let mut x=7u8;");
        for _ in 0..count {
            source.push_str("x=");
            for _ in 0..depth {
                source.push_str("id(");
            }
            source.push('x');
            for _ in 0..depth {
                source.push(')');
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
    pending_map_scaling()?;
    assert!(feature_sets::index_is_complete());
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
    assert_eq!(crate::resolve::bindings::cpu_request_layouts().0, 16);
    assert_eq!(crate::resolve::bindings::cpu_request_layouts().1, 1);
    eprintln!(
        "request layouts={:?}",
        crate::resolve::bindings::cpu_request_layouts()
    );
    Ok(())
}
fn shared_sites(header: &str, literals: &[&str]) -> String {
    let mut source = String::from(header);
    for (index, literal) in literals.iter().enumerate() {
        source.push_str("#[target_feature(enable=\"");
        source.push_str(literal);
        source.push_str("\")]fn f");
        source.push_str(&index.to_string());
        source.push_str("(){}");
    }
    source
}
/// Collection-only ledger (work, attempted bytes) and the retained rows.
fn collected(source: &str) -> TestResult<((usize, usize), Vec<FeatureSet>)> {
    let report = crate::test_source::resolve(source, true)?;
    let mut fresh = Declarations::new(&report)?;
    let rows = collect(&report, &mut fresh)?;
    let features = rows.values().map(|row| row.features).collect();
    Ok((header_attributes::source_contract_costs(&fresh), features))
}
/// Uncounted ledger rounding aside, this is the name and implication work of one value.
fn closed(names: &str) -> TestResult<(FeatureSet, usize)> {
    let mut work = 0;
    let mut value = feature_sets::Closed::default();
    for name in names.split(',') {
        value.insert(
            name,
            |growth| {
                work += growth.work;
                Ok::<_, &'static str>(())
            },
            |_| Ok(true),
            std::convert::identity,
        )?;
    }
    Ok((value.features(), work))
}
const EXPENSIVE: &str = "avx512f,avx512vl,avx512bw,avx512dq,avx512cd,avx512ifma,avx512vbmi,avx512vbmi2,avx512vnni,avx512bitalg,avx512vpopcntdq,avx512bf16,avx512fp16,vaes,vpclmulqdq,gfni,sha,aes,pclmulqdq,bmi1,bmi2,lzcnt,popcnt,xsaves,xsavec,xsaveopt";
#[test]
fn shared_enable_spellings_close_once_while_each_site_keeps_its_own_work() -> TestResult {
    let (expected, validation) = closed(EXPENSIVE)?;
    let ((cheap_one, cheap_bytes_one), _) = collected(&shared_sites("", &["sse"]))?;
    let ((one, bytes_one), _) = collected(&shared_sites("", &[EXPENSIVE]))?;
    let lookup = 2 * (EXPENSIVE.len() - "sse".len());
    assert!(
        one - cheap_one > validation,
        "the first site validates fully"
    );
    for count in [8, 32, 128] {
        let ((cheap, cheap_bytes), _) = collected(&shared_sites("", &vec!["sse"; count]))?;
        let ((steps, bytes), rows) = collected(&shared_sites("", &vec![EXPENSIVE; count]))?;
        eprintln!(
            "target-literal shared={count} expensive={steps} cheap={cheap} validation={validation}"
        );
        assert_eq!(
            steps - cheap,
            one - cheap_one + (count - 1) * lookup,
            "only each site's own spelling read scales with sites"
        );
        assert_eq!(bytes - cheap_bytes, bytes_one - cheap_bytes_one);
        assert_eq!(rows.len(), count);
        assert!(
            rows.iter().all(|row| *row == expected),
            "every reusing row is closed"
        );
    }
    let gate = "#![feature(x86_amx_intrinsics)]";
    let ((gated_one, _), _) = collected(&shared_sites(gate, &["amx-tile"]))?;
    let ((plain_one, _), _) = collected(&shared_sites(gate, &["xsaveopt"]))?;
    let ((gated, _), _) = collected(&shared_sites(gate, &["amx-tile"; 16]))?;
    let ((plain, _), _) = collected(&shared_sites(gate, &["xsaveopt"; 16]))?;
    assert_eq!(
        gated - gated_one,
        plain - plain_one + 15 * 2,
        "each reusing site pays for its own crate gate result"
    );
    let duplicate = "#[target_feature(enable=\"sse2\")]#[target_feature(enable=\"sse2\")]fn f(){}";
    let report = crate::test_source::resolve(duplicate, true)?;
    let mut fresh = Declarations::new(&report)?;
    let rows = collect(&report, &mut fresh)?;
    let rows = rows.values().collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        return Err("one duplicate row".into());
    };
    assert_eq!(
        row.sites.len(),
        2,
        "a reused spelling still retains its site"
    );
    assert_eq!(row.features, closed("sse2")?.0);
    Ok(())
}
#[test]
fn distinct_spellings_failed_sites_and_exhaustion_keep_full_validation() -> TestResult {
    let names = ["avx512vl", "avx512bw", "avx512dq", "avx512cd", "avx512f"];
    let mut permutations = Vec::new();
    for first in 0..names.len() {
        for second in (0..names.len()).filter(|&index| index != first) {
            let rest = (0..names.len()).filter(|&index| index != first && index != second);
            let order = [first, second].into_iter().chain(rest);
            permutations.push(
                order
                    .map(|index| names[index])
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
    }
    let (expected, validation) = closed(&permutations[0])?;
    for count in [4, 20] {
        let spellings = permutations[..count]
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let ((distinct, _), rows) = collected(&shared_sites("", &spellings))?;
        let ((shared, _), _) =
            collected(&shared_sites("", &vec![permutations[0].as_str(); count]))?;
        eprintln!(
            "target-literal distinct={count} distinct={distinct} shared={shared} validation={validation}"
        );
        assert!(
            distinct - shared >= (count - 1) * validation,
            "every distinct spelling pays its own full validation"
        );
        assert!(rows.iter().all(|row| *row == expected));
    }
    let failing = "#[target_feature(enable=\"avx\")]fn a(){}#[target_feature(enable=\"avx\",enable=\"rustgm_unknown\")]fn b(){}";
    let report = crate::test_source::resolve(failing, true)?;
    let mut fresh = Declarations::new(&report)?;
    let error = collect(&report, &mut fresh)
        .err()
        .ok_or("a later unknown feature must still reject")?;
    assert_eq!(
        error.message,
        "target feature is invalid or forbidden for this target"
    );
    assert_eq!(
        error.span.start as usize,
        failing.rfind("enable").ok_or("actual failing entry")?,
        "the failing site's own entry is reported"
    );
    let source = shared_sites("", &["avx"; 4]);
    let ((total, _), _) = collected(&source)?;
    for (prior, admitted) in [(total + 1, true), (total, false)] {
        let report = crate::test_source::resolve(&source, true)?;
        let mut fresh = Declarations::new(&report)?;
        let (bound, _) = header_attributes::source_contract_bound(&fresh);
        // The reservation's own ledger step leaves exactly `total` or `total - 1`.
        header_attributes::reserve_source_contracts(&mut fresh, bound - prior, 0)?;
        assert_eq!(
            collect(&report, &mut fresh).is_ok(),
            admitted,
            "every reusing site's work reaches the shared limit"
        );
        assert_eq!(
            header_attributes::charge_source_contract_query(&fresh, 0).is_ok(),
            admitted,
            "failed attempted work stays consumed"
        );
    }
    Ok(())
}
/// `callees` target-feature functions, each with `attributes` sites, called
/// `calls` times each from one matching caller.
fn feature_calls(callees: usize, attributes: usize, calls: usize) -> String {
    let site = "#[target_feature(enable=\"sse2\")]";
    let mut source = String::new();
    let mut body = String::new();
    for index in 0..callees {
        source.push_str(&site.repeat(attributes));
        source.push_str("fn f");
        source.push_str(&index.to_string());
        source.push_str("(){}");
        for _ in 0..calls {
            body.push('f');
            body.push_str(&index.to_string());
            body.push_str("();");
        }
    }
    source + site + "pub fn g(){" + &body + "}"
}
#[test]
fn header_work_stays_inside_its_input_graph_bound_only_while_linear() -> TestResult {
    let mut previous = None;
    for count in [64, 256] {
        let source = feature_calls(count, 1, 1);
        let (_, Analysis { facts, .. }) = analyze(&source)?;
        let (steps, bytes) = header_attributes::source_contract_costs(&facts);
        let bound = header_attributes::source_contract_bound(&facts);
        eprintln!("linear callees={count} steps={steps} bytes={bytes} bound={bound:?}");
        assert!(
            steps * 2 <= bound.0 && bytes * 2 <= bound.1,
            "linear headers keep their headroom: {steps}/{bytes} of {bound:?}"
        );
        if let Some(smaller) = previous {
            assert!(bound.0 >= 3 * smaller, "the bound follows its input graph");
        }
        previous = Some(bound.0);
    }
    analyze(&feature_calls(1, 32, 32))?;
    let source = feature_calls(1, 512, 512);
    let report = crate::test_source::resolve(&source, true)?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("repeated whole-row validation per call must exhaust its linear bound")?;
    assert_eq!(
        error.message,
        "source header contract query work exceeds its input-graph bound"
    );
    Ok(())
}
#[test]
fn projected_bound_staging_keeps_linear_headroom_in_ordinary_crates() -> TestResult {
    for shape in ["wide", "deep"] {
        let mut costs = Vec::new();
        for count in [4, 16, 64] {
            let path = format!("pa20/tests/spec/assoc-bounds/{shape}{count}.rs");
            let (_, Analysis { facts, .. }) = analyze(&crate::test_source::fixture(&path)?)?;
            let (steps, bytes) = header_attributes::source_contract_costs(&facts);
            let bound = header_attributes::source_contract_bound(&facts);
            eprintln!("staged-bounds {shape}{count} steps={steps} bytes={bytes} bound={bound:?}");
            assert!(
                steps * 2 <= bound.0 && bytes * 3 <= bound.1 * 2,
                "{path}: written bounds keep their headroom: {steps}/{bytes} of {bound:?}"
            );
            costs.push(bytes);
        }
        // The cost per added written bound or nesting level stays flat.
        let early = (costs[1] - costs[0]) / 12;
        let late = (costs[2] - costs[1]) / 48;
        assert!(
            late * 10 <= early * 11,
            "{shape}: staging stays linear: {costs:?}"
        );
    }
    Ok(())
}
