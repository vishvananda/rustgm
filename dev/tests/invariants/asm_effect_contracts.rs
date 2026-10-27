//! Source-owned asm retains typed effects while actual emission stays capability-checked.
use crate::{body, syntax::ast::Kind};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const DECL: &str = "#[rustc_builtin_macro(asm)]macro_rules!actual{($($t:tt)*)=>{9u8};}";
fn source(program: &str) -> Result<String> {
    let mut prefix = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    prefix.push_str("impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{} ");
    prefix.push_str(DECL);
    prefix.push_str(program);
    Ok(prefix)
}
fn expanded(program: &str) -> Result<crate::resolve::expansion::Report> {
    Ok(crate::test_source::source_provider(
        &source(program)?,
        "typed_asm",
    )?)
}
#[test]
fn actual_templates_keep_one_parser_and_source_role_without_hot_node_growth() -> Result {
    let report = expanded(
        "#[rustc_builtin_macro(concat)]macro_rules!concat{($($t:tt)*)=>{};}macro_rules!forward{($s:expr,$x:expr)=>{actual!($s,x=in(reg)$x,options(nostack,preserves_flags))};}pub fn f(x:usize){unsafe{forward!(concat!(\"/* {x}\",\" */\"),x);}}",
    )?;
    let unit = &report.package.units[0];
    let mut roots = 0;
    for (index, node) in unit.configured.ast.nodes.iter().enumerate() {
        if unit.configured.active[index] && matches!(node.kind, Kind::InlineAssembly(_)) {
            roots += 1;
            report.prepare_assembly(
                crate::resolve::unit::NodeKey {
                    file: crate::resolve::unit::FileId(0),
                    node: crate::syntax::ast::NodeId::new(u32::try_from(index)?),
                },
                1,
                0,
            )?;
        }
    }
    assert_eq!(roots, 1);
    let assembly_calls: Vec<_> = report
        .invocations
        .iter()
        .filter(|invocation| {
            matches!(
                report.definitions[invocation.definition].body,
                crate::resolve::expansion::Body::Builtin(crate::expand::builtin::Builtin::Asm)
            )
        })
        .collect();
    assert_eq!(assembly_calls.len(), 1);
    assert_eq!(assembly_calls[0].parser_calls, 1);
    let reducer = crate::test_source::fixture("pa41/tests/spec/asm-frontend/stream-ps.rs")?;
    let mut actual = source("")?;
    // Crate attributes remain at the actual crate root, before source declarations.
    actual = actual.replacen("#![feature(", "#![feature(decl_macro,staged_api,", 1);
    actual.insert_str(0, "#![stable(feature=\"core\",since=\"1.6.0\")]\n");
    actual.push_str(&reducer.lines().skip(2).collect::<Vec<_>>().join("\n"));
    let full = crate::test_source::source_provider(&actual, "actual_stream_asm")?;
    assert!(full.package.units.iter().any(|unit| {
        unit.configured
            .ast
            .nodes
            .iter()
            .enumerate()
            .any(|(i, n)| unit.configured.active[i] && matches!(n.kind, Kind::InlineAssembly(_)))
    }));

    assert_eq!(size_of::<Kind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    for bad in [
        "actual!();",
        "actual!(\"{missing}\");",
        "actual!(\"\",options(nomem,readonly));",
        "actual!(\"\",options(pure,nomem));",
        "actual!(\"\",x=in(\"rax\")0usize);",
        "actual!(\"\",label{});",
    ] {
        assert!(
            expanded(&format!("fn f(){{unsafe{{{bad}}}}}")).is_err(),
            "{bad}"
        );
    }
    Ok(())
}
#[test]
fn scalar_inputs_and_outputs_reach_real_original_proof_then_backend_denial() -> Result {
    let mut report = expanded(
        "pub fn main(){let mut x:usize=7;unsafe{actual!(\"add {x},1\",x=inout(reg)x,options(pure,nomem,nostack,preserves_flags));}let _=x;}",
    )?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let entry = crate::sema::entry::select(&mut report, &analysis.facts)?;
    let mut program = body::build_concrete(&report, &analysis.facts)?;
    let selected = program
        .bodies
        .iter_mut()
        .find(|b| matches!(b.owner,body::ir::Owner::Function{definition,..}if definition==entry))
        .ok_or("missing genuine main body")?;
    let data = selected
        .assembly
        .as_ref()
        .ok_or("missing retained asm payload")?;
    assert_eq!(data.instructions.len(), 1);
    assert_eq!(data.operands.len(), 1);
    assert!(data.operands[0].input.is_some());
    assert!(data.operands[0].output.is_some());
    assert!(
        selected
            .values
            .iter()
            .any(|v| matches!(v.kind, body::ir::Rvalue::AssemblyOutput { .. }))
    );
    super::validate(selected)?;
    let retained = selected.retained_bytes();
    selected.assembly.as_mut().ok_or("payload")?.templates[0]
        .text
        .reserve(32);
    assert!(selected.retained_bytes() > retained);
    let prepared = std::mem::take(&mut analysis.prepared);
    crate::lower::instances::inspect_prepared(&report, &mut analysis.facts, prepared, |_, _| ())?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let error = crate::lower::instances::build_prepared(
        &report,
        &mut analysis.facts,
        entry,
        analysis.prepared,
    )
    .err()
    .ok_or("unsupported backend silently emitted asm")?;
    assert!(
        error
            .to_string()
            .contains("unavailable locked backend capability"),
        "{error}"
    );
    Ok(())
}
#[test]
fn asm_place_unsafe_const_and_type_failures_do_not_become_unit_effects() -> Result {
    for (program, message) in [
        ("fn f(){actual!(\"nop\");}", "unsafe context"),
        ("const unsafe fn f(){unsafe{actual!(\"nop\");}}", "constant"),
        (
            "fn f(){unsafe{actual!(\"/* {0} */\",out(reg)1usize);}}",
            "place",
        ),
        (
            "fn f(x:bool){unsafe{actual!(\"/* {0} */\",in(reg)x);}}",
            "assembly register operand",
        ),
        (
            "fn f(){let x:usize=0;unsafe{actual!(\"/* {0} */\",out(reg)x);}}",
            "immutable",
        ),
    ] {
        let report = expanded(program)?;
        let result = crate::pipeline::analyze_source(&report).and_then(|mut analysis| {
            crate::lower::instances::inspect_prepared(
                &report,
                &mut analysis.facts,
                analysis.prepared,
                |_, _| (),
            )
        });
        let error = result
            .err()
            .ok_or("invalid asm became a proved empty effect")?;
        assert!(
            error.to_string().contains(message),
            "expected {message}: {error}"
        );
    }
    let report = expanded("fn f(p:*mut u8){unsafe{actual!(\"/* {0} */\",in(xmm_reg)p);}}")?;
    crate::pipeline::analyze_source(&report)?;
    Ok(())
}
#[test]
fn asm_storage_and_width_growth_remain_cold_and_body_owned() -> Result {
    assert_eq!(size_of::<body::ir::Body>(), 456);
    assert_eq!(size_of::<body::ir::Rvalue>(), 20);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    let report = crate::test_source::resolve("fn f(){}", true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let facts = crate::sema::bodies::analyze(&report, facts)?;
    let plain = body::build_concrete(&report, &facts)?;
    assert!(plain.bodies.iter().all(|b| b.assembly.is_none()));
    for width in [1usize, 8, 32] {
        let mut program = String::from("fn f(x:usize){unsafe{actual!(\"");
        for i in 0..width {
            use std::fmt::Write as _;
            write!(program, "{{a{i}}} ")?;
        }
        program.push('\"');
        for i in 0..width {
            use std::fmt::Write as _;
            write!(program, ",a{i}=in(reg)x")?;
        }
        program.push_str(",options(nomem,nostack));}}");
        let report = expanded(&program)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        let bodies = body::build_concrete(&report, &analysis.facts)?;
        let asm = bodies
            .bodies
            .iter()
            .find(|b| b.assembly.is_some())
            .ok_or("scaled body")?;
        let data = asm.assembly.as_ref().ok_or("data")?;
        assert_eq!(data.operands.len(), width);
        assert_eq!(data.pieces.len(), width);
        super::validate(asm)?;
        assert!(data.allocation_bytes() < 4096 + 1024 * width);
        eprintln!(
            "assembly width={width} payload={} body={} payload-owner={} instruction={} operand={}",
            data.allocation_bytes(),
            size_of_val(asm),
            size_of::<super::Payload>(),
            size_of::<super::Instruction>(),
            size_of::<super::Operand>()
        );
        let mut wrong = asm.clone();
        wrong.assembly.as_mut().ok_or("wrong data")?.instructions[0]
            .operands
            .end += 1;
        assert!(super::validate(&wrong).is_err());
        let mut foreign = asm.clone();
        foreign.assembly.as_mut().ok_or("foreign data")?.source = None;
        assert!(super::validate(&foreign).is_err());
    }
    let report = expanded("fn f(){unsafe{actual!(\"nop\");}}")?;
    let key = report.package.units[0]
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| matches!(node.kind, Kind::InlineAssembly(_)).then_some(index))
        .ok_or("actual asm payer root")?;
    let key = crate::resolve::unit::NodeKey {
        file: crate::resolve::unit::FileId(0),
        node: crate::syntax::ast::NodeId::new(u32::try_from(key)?),
    };
    let before = report.assembly_work().ok_or("payer")?;
    assert!(report.prepare_assembly(key, 1, usize::MAX).is_err());
    let failed = report.assembly_work().ok_or("failed payer")?;
    assert!(failed.0 > before.0);
    assert!(failed.1 > before.1);
    assert!(report.prepare_assembly(key, 1, 0).is_err());
    Ok(())
}
