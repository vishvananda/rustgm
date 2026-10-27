//! Genuine source binary declarations, inference, effects and scalar execution stay joined.
use super::*;
use crate::lowir::program::{Literal, Operand, Terminator};
use crate::{
    pipeline::{self, Analysis},
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        unit::NodeKey,
    },
    sema::{declarations, model::Operation},
    support::language_hooks::Hook,
    syntax::ast::{Kind as Syntax, NodeId},
    types::{Kind, Primitive, TypeId},
};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn input(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-shift-or/{name}.rs"
    ))?)
}
fn analyze(text: &str) -> TestResult<(Report, Analysis)> {
    let report = crate::test_source::source_provider(text, "source_binary")?;
    let facts = pipeline::analyze_source(&report)?;
    Ok((report, facts))
}
fn named(report: &Report, name: &str, kind: DefKind) -> TestResult<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == kind && report.package.names.get(d.name) == name)
        .map(|(i, _)| u32::try_from(i).map(DefId))
        .transpose()?
        .ok_or_else(|| format!("actual {name}").into())
}
fn lower(
    report: &Report,
    analysis: Analysis,
    names: &[&str],
) -> TestResult<crate::lowir::program::Program> {
    let Analysis {
        mut facts,
        prepared,
    } = analysis;
    let roots = names
        .iter()
        .map(|n| named(report, n, DefKind::Function))
        .collect::<TestResult<Vec<_>>>()?;
    Ok(crate::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?)
}
#[test]
fn real_source_operands_rhs_inference_and_output_precede_primitive_erasure() -> TestResult {
    let (report, Analysis { mut facts, .. }) = analyze(&input("runtime-overloads")?)?;
    let site = report.resolution.definitions
        [named(&report, "encoded", DefKind::Function)?.0 as usize]
        .node
        .ok_or("actual source")?;
    let header = source_binary(&report, &mut facts, site, Hook::Shl)?;
    assert!(matches!(
        facts.types.kind(header.rhs),
        Some(Kind::Parameter(_))
    ));
    let mut scalar = 0;
    let mut overloaded = 0;
    let mut inferred_i32 = 0;
    for (index, node) in report.package.units[0]
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
    {
        let Syntax::Binary {
            op: crate::lex::token::Punctuation::ShiftLeft | crate::lex::token::Punctuation::Or,
            left,
            right,
        } = node.kind
        else {
            continue;
        };
        let key = NodeKey {
            file: site.file,
            node: NodeId::new(u32::try_from(index)?),
        };
        let Some(left) = facts.node(NodeKey { node: left, ..key }).and_then(|f| f.ty) else {
            continue;
        };
        if matches!(facts.types.kind(left), Some(Kind::Primitive(_))) {
            scalar += 1;
            assert!(!facts.methods.calls.contains_key(&key));
            assert!(matches!(
                facts.node(key).and_then(|f| f.operation),
                Some(Operation::Binary { .. })
            ));
            if facts
                .node(NodeKey { node: right, ..key })
                .and_then(|f| f.ty)
                == Some(TypeId::primitive(Primitive::I32))
            {
                inferred_i32 += 1;
            }
        } else {
            overloaded += 1;
            assert!(facts.methods.calls.contains_key(&key));
        }
    }
    assert!(scalar >= 5 && overloaded >= 2 && inferred_i32 >= 1);
    assert!(facts.associated.operation(header.definition).is_none());
    assert!(
        report
            .resolution
            .builtin_traits
            .language(Hook::Shl)
            .is_none()
    );
    Ok(())
}
#[test]
fn actual_overloads_and_explicit_custom_bodies_reach_original_instance_lowering() -> TestResult {
    for (name, exports) in [
        ("runtime-overloads", ["encoded", "shifted", "ored"]),
        (
            "const-followup/custom-runtime-only",
            ["syntax", "explicit_shift", "explicit_or"],
        ),
    ] {
        let source = input(name)?;
        let (report, analysis) = analyze(&source)?;
        let program = lower(&report, analysis, &exports)?;
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/lib045-root-implementation-v1/operators-native")
            .join(name.replace('/', "_"));
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("provider.rs"), source)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(folder.join("provider.lowir"))?,
        )?;
        for name in exports {
            assert_eq!(
                program
                    .symbols
                    .iter()
                    .filter(|s| s.name == name && s.body.is_some())
                    .count(),
                1
            );
        }
        assert!(program.symbols.iter().filter(|s| s.body.is_some()).count() >= 4);
    }
    let source = input("runtime-overloads")?
        + "\n#[unsafe(no_mangle)]pub extern \"C\" fn let_encoded(len:usize)->usize{let tagged=len<<1|1;tagged}";
    let (report, analysis) = analyze(&source)?;
    let program = lower(&report, analysis, &["let_encoded"])?;
    assert!(
        program
            .symbols
            .iter()
            .any(|s| s.name == "let_encoded" && s.body.is_some())
    );
    Ok(())
}
#[test]
fn actual_const_effects_and_negative_overflow_faults_remain_required() -> TestResult {
    let source = input("const-followup/primitive-const-impl")?
        + "\n#[unsafe(no_mangle)]pub extern \"C\" fn const_value()->usize{ENCODED}";
    let (report, analysis) = analyze(&source)?;
    let program = lower(&report, analysis, &["const_value"])?;
    let body = program
        .symbols
        .iter()
        .find(|s| s.name == "const_value")
        .and_then(|s| s.body.as_ref())
        .ok_or("actual const body")?;
    assert!(body.blocks.iter().any(|b|matches!(b.terminator,Terminator::Return(Some(v)) if matches!(v.operand,Operand::Literal(id) if matches!(body.literals.get(id.0 as usize),Some(Literal::Integer(7)))))));
    let mut failures = Vec::new();
    for (name, expression, lhs, rhs) in [
        ("primitive-const", "3usize<<1|1", "usize", "i32"),
        (
            "explicit-nonconst",
            "<usize as Shl<i32>>::shl(3usize,1i32)",
            "usize",
            "i32",
        ),
        ("nonconst-overload", "Word(7usize)<<1u8", "Word", "u8"),
    ] {
        let (report, analysis) = analyze(&input(&format!("effect-pairs/{name}"))?)?;
        lower(&report, analysis, &["runtime_pair"])?;
        let source = input(name)?;
        let report = crate::test_source::source_provider(&source, "source_const_op")?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("invalid const accepted")?;
        eprintln!("source-const opponent {name}: {error:?}");
        let message = format!(
            "no implementation for `{lhs} << {rhs}`\nlabel: no implementation for `{lhs} << {rhs}`"
        );
        let start = u32::try_from(source.rfind(expression).ok_or("actual const call")?)?;
        if error.message != message || error.span.start != start {
            failures.push(format!(
                "{name}: expected {start}: {message}; actual {error}"
            ));
        }
    }
    for name in [
        "const-followup/const-negative-impl",
        "const-followup/const-overflow-impl",
    ] {
        let report = crate::test_source::source_provider(&input(name)?, "source_const_op")?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("invalid const accepted")?;
        eprintln!("source-const shift fault {name}: {error}");
        if !error
            .message
            .contains("constant shift amount is out of range")
        {
            failures.push(format!("{name}: did not reach real shift fault: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn source_binary_headers_reject_wrong_formals_members_and_replayed_images() -> TestResult {
    let original = input("runtime-overloads")?;
    for text in [
        original.replace("fn shl(self, rhs: Rhs)", "fn shl(&self, rhs: Rhs)"),
        original.replace("Shl<Rhs = Self>", "Shl<Rhs = u8>"),
        original.replace(
            "fn shl(self, rhs: Rhs) -> Self::Output",
            "fn shl(self, rhs: Rhs) -> bool",
        ),
    ] {
        assert!(analyze(&text).is_err());
    }
    // The original wrong-rhs also had independently rejected const items.
    // Use the separately accepted runtime prefix to reach this exact bad call.
    let wrong = input("const-followup/custom-runtime-only")?
        + "\npub fn wrong_rhs(n:usize)->usize{n<<true}";
    let error = analyze(&wrong)
        .err()
        .ok_or("boolean shift RHS accepted")?
        .to_string();
    assert!(
        error.contains("associated type requires an unambiguous validated implementation"),
        "actual wrong-Rhs cause: {error}"
    );
    let (mut report, Analysis { mut facts, .. }) = analyze(&original)?;
    let site = report.resolution.definitions
        [named(&report, "encoded", DefKind::Function)?.0 as usize]
        .node
        .ok_or("source site")?;
    let selected = source_binary(&report, &mut facts, site, Hook::Shl)?;
    let old = facts.body_work.canonical_steps;
    source_binary(&report, &mut facts, site, Hook::Shl)?;
    assert!(facts.body_work.canonical_steps > old);
    let old_signature = facts.definitions[selected.method.0 as usize].replace(TypeId::UNIT);
    assert!(source_binary(&report, &mut facts, site, Hook::Shl).is_err());
    facts.definitions[selected.method.0 as usize] = old_signature;
    report.package.units[0].source =
        crate::support::source::SourceFile::new(original.into_bytes())?;
    assert!(source_binary(&report, &mut facts, site, Hook::Shl).is_err());
    Ok(())
}
#[test]
fn source_operator_work_scales_and_ordinary_path_retains_hot_layouts() -> TestResult {
    let original = input("const-followup/custom-runtime-only")?;
    for count in [1, 8, 32, 128] {
        let mut source = original.clone();
        write!(source, "pub fn repeated(mut x:usize)->usize{{")?;
        for _ in 0..count {
            source.push_str("x=x<<1i32|1usize;");
        }
        source.push_str("x}");
        let (report, Analysis { mut facts, .. }) = analyze(&source)?;
        let site = report.resolution.definitions
            [named(&report, "repeated", DefKind::Function)?.0 as usize]
            .node
            .ok_or("actual repeated source")?;
        let before = facts.body_work.canonical_steps;
        for _ in 0..count {
            source_binary(&report, &mut facts, site, Hook::Shl)?;
        }
        let work = facts.body_work.canonical_steps - before;
        assert!(work <= count * 1000);
        println!(
            "binary-work operators={} header_work={work} body_work={}",
            2 * count,
            facts.body_work.canonical_steps
        );
    }
    for (name, accepted) in [
        ("shl", true),
        ("mixed-shl", true),
        ("bitor", true),
        ("bool-bitor", true),
        ("user-trait", true),
        ("wrong-output", false),
        ("mixed-arithmetic", false),
        ("float-shift", false),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa26/tests/spec/primitive-binary/{name}.rs"))?;
        let report = crate::test_source::resolve(&source, true)?;
        let result = (|| -> TestResult<_> {
            let facts = declarations::analyze(&report)?;
            assert!(facts.source_bindings.is_none());
            let analysis = pipeline::analyze(&report, facts)?;
            lower(&report, analysis, &["main"])
        })();
        assert_eq!(
            result.is_ok(),
            accepted,
            "ordinary {name}: {:?}",
            result.err()
        );
    }
    println!(
        "SourceBinary={} SourceUnary={} CallContexts={} Operation={} NodeFacts={} Body={} Source={}",
        size_of::<SourceBinary>(),
        size_of::<crate::sema::compiler_items::unary_contract::SourceUnary>(),
        size_of::<crate::sema::bodies::CallContexts>(),
        size_of::<Operation>(),
        size_of::<crate::sema::model::NodeFacts>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::sema::traits::selection::Source>()
    );
    assert_eq!(size_of::<Operation>(), 16);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn provisional_results_preserve_known_never_cfg_and_explicit_feature_frontier() -> TestResult {
    let mut failures = Vec::new();
    for (name, expected, site) in [
        (
            "block",
            "associated type projection requires normalization",
            "Delayed<<1",
        ),
        ("binding", "incompatible types", "{let _value="),
        (
            "branch",
            "associated type projection requires normalization",
            "if flag",
        ),
    ] {
        let source = input(&format!("delayed-never/{name}"))?;
        let report = crate::test_source::source_provider(&source, "source_binary")?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("delayed Never opponent accepted")?;
        eprintln!("delayed-Never {name}: {error:?}");
        let start = u32::try_from(source.rfind(site).ok_or("actual delayed site")?)?;
        if error.message != expected || error.span.start != start {
            failures.push(format!(
                "{name}: expected {start}: {expected}; actual {error}"
            ));
        }
    }
    let (report, analysis) = analyze(&input("delayed-never/known-return")?)?;
    let program = lower(&report, analysis, &["block", "binding", "branch"])?;
    let bodies=program.symbols.iter().filter(|symbol|matches!(program.signatures.get(symbol.signature.0 as usize),Some(crate::lowir::program::Signature::Scalar(s)) if s.result==crate::lowir::vocabulary::Type::U8)).filter_map(|s|s.body.as_ref()).collect::<Vec<_>>();
    assert_eq!(bodies.len(), 3);
    assert_eq!(
        bodies
            .iter()
            .filter(|body| body
                .blocks
                .iter()
                .all(|b| !matches!(b.terminator, Terminator::Return(_))))
            .count(),
        2
    );
    let branch = bodies
        .iter()
        .find(|body| {
            body.blocks
                .iter()
                .any(|b| matches!(b.terminator, Terminator::Return(_)))
        })
        .ok_or("actual branch return")?;
    assert!(
        branch
            .blocks
            .iter()
            .any(|b| matches!(b.terminator, Terminator::Unreachable))
    );
    merged_branch_returns_seven(branch)?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn merged_branch_returns_seven(body: &crate::lowir::program::Function) -> TestResult {
    use crate::lowir::program::{Operation as LowOperation, ResultId};
    let (return_block, returned) = body
        .blocks
        .iter()
        .enumerate()
        .find_map(|(index, block)| match block.terminator {
            Terminator::Return(Some(value)) => Some((index, value.operand)),
            _ => None,
        })
        .ok_or("actual known-Never branch return")?;
    let Operand::Temporary(returned) = returned else {
        return Err("branch merge must use its real temporary".into());
    };
    let slot = body
        .instructions
        .iter()
        .find_map(
            |instruction| match (instruction.result, instruction.operation) {
                (
                    Some(ResultId::Scalar(result)),
                    LowOperation::Load {
                        address: Operand::Slot(slot),
                        ..
                    },
                ) if result == returned => Some(slot),
                _ => None,
            },
        )
        .ok_or("actual return merge slot")?;
    let source_block = body.blocks.iter().find(|block| {
        body.instructions[block.instructions.start as usize..block.instructions.end as usize].iter().any(|instruction| {
            matches!(instruction.operation, LowOperation::Store { address: Operand::Slot(actual), value, access: crate::lowir::program::Access::Ordinary } if actual == slot && matches!(value.operand, Operand::Literal(id) if matches!(body.literals.get(id.0 as usize), Some(Literal::Integer(7)))))
        })
    }).ok_or("actual seven store into return merge slot")?;
    assert!(
        matches!(source_block.terminator, Terminator::Jump(target) if target.0 as usize == return_block)
    );
    assert!(
        body.blocks
            .iter()
            .any(|block| matches!(block.terminator, Terminator::Branch { .. }))
    );
    Ok(())
}

#[test]
fn nested_fallback_respects_predeclared_order_and_shared_frontiers() -> TestResult {
    for name in ["predeclared", "reversed"] {
        let (report, analysis) = analyze(&input(&format!("readiness/{name}"))?)?;
        let program = lower(&report, analysis, &["ordered"])?;
        assert!(
            program
                .symbols
                .iter()
                .any(|s| s.name == "ordered" && s.body.is_some())
        );
    }
    let error = analyze(&input("readiness/shared")?)
        .err()
        .ok_or("unimplemented shared numeric frontier accepted")?
        .to_string();
    assert!(
        error.contains(
            "trait obligation requires an unambiguous matching assumption or implementation"
        ),
        "shared: {error}"
    );
    let source = input("readiness/ordinary-closure")?;
    let report = crate::test_source::resolve(&source, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    assert!(analysis.facts.source_bindings.is_none());
    lower(&report, analysis, &["closure", "inline_constant"])?;
    for name in ["ambiguous", "cycle"] {
        let source = input(&format!("readiness/{name}"))?;
        assert!(analyze(&source).is_err(), "actual {name} opponent accepted");
    }
    Ok(())
}
